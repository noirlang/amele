use crate::error::{HataKodu, WormError, WormResult};
use crate::hash::{to_hex, write_sha256_sidecar};
use digest::Digest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub const DEFAULT_READ_CHUNK: usize = 4 * 1024 * 1024;

static DISK_ACQUISITION_CANCELLED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: PathBuf,
    pub total_size: u64,
    pub used_size: u64,
    pub accessible: bool,
}

#[derive(Debug, Clone)]
pub struct DiskAcquisitionTask {
    pub source: PathBuf,
    pub target: PathBuf,
    pub start_offset: u64,
    pub size: Option<u64>,
    pub chunk_size: usize,
    pub calculate_hash: bool,
    pub full_disk: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskAcquisitionResult {
    pub target: PathBuf,
    pub bytes_copied: u64,
    pub total_bytes: u64,
    pub sha256: Option<String>,
    pub partial_path: Option<PathBuf>,
}

impl DiskAcquisitionTask {
    pub fn new(source: impl Into<PathBuf>, target: impl Into<PathBuf>) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            start_offset: 0,
            size: None,
            chunk_size: DEFAULT_READ_CHUNK,
            calculate_hash: true,
            full_disk: true,
        }
    }
}

pub fn disk_size(path: impl AsRef<Path>) -> WormResult<u64> {
    disk_size_impl(path.as_ref())
}

pub fn list_disks() -> WormResult<Vec<DiskInfo>> {
    list_disks_impl()
}

pub fn run_disk_acquisition<F>(
    task: &DiskAcquisitionTask,
    progress: F,
) -> WormResult<DiskAcquisitionResult>
where
    F: FnMut(u64, u64),
{
    DISK_ACQUISITION_CANCELLED.store(false, Ordering::SeqCst);
    run_disk_acquisition_with_control(task, progress, || {
        if DISK_ACQUISITION_CANCELLED.load(Ordering::SeqCst) {
            DiskAcquisitionControl::Cancel
        } else {
            DiskAcquisitionControl::Continue
        }
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskAcquisitionControl {
    Continue,
    Pause,
    Cancel,
}

pub fn run_disk_acquisition_with_control<F, C>(
    task: &DiskAcquisitionTask,
    mut progress: F,
    mut control: C,
) -> WormResult<DiskAcquisitionResult>
where
    F: FnMut(u64, u64),
    C: FnMut() -> DiskAcquisitionControl,
{
    let source_size = disk_size(&task.source)?;
    if source_size == 0 {
        return Err(WormError::new(
            HataKodu::DiskBoyut,
            "Kaynak boyut alinamadi",
        ));
    }

    if let Some(parent) = task.target.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            WormError::io(HataKodu::DosyaYazma, "Hedef klasor olusturulamadi", err)
        })?;
    }

    let mut source = File::open(&task.source)
        .map_err(|err| WormError::io(HataKodu::DiskErisim, "Kaynak acilamadi", err))?;
    if task.start_offset > 0 {
        source
            .seek(SeekFrom::Start(task.start_offset))
            .map_err(|err| WormError::io(HataKodu::DiskOkuma, "Kaynak konumlanamadi", err))?;
    }

    let mut target = File::create(&task.target)
        .map_err(|err| WormError::io(HataKodu::DosyaYazma, "Hedef dosya olusturulamadi", err))?;

    let available = source_size.saturating_sub(task.start_offset);
    let total = if task.full_disk {
        available
    } else {
        task.size.unwrap_or(available).min(available)
    };

    if total == 0 {
        return Err(WormError::new(HataKodu::DiskBoyut, "Kopyalanacak veri yok"));
    }

    let chunk_size = task.chunk_size.max(4096);
    let mut buffer = vec![0_u8; chunk_size];
    let mut copied = 0_u64;
    let mut sha256 = task.calculate_hash.then(Sha256::new);
    let mut success = false;
    let mut cancelled = false;

    while copied < total {
        match control() {
            DiskAcquisitionControl::Continue => {}
            DiskAcquisitionControl::Pause => {
                thread::sleep(Duration::from_millis(200));
                continue;
            }
            DiskAcquisitionControl::Cancel => {
                cancelled = true;
                break;
            }
        }

        let to_read = (total - copied).min(chunk_size as u64) as usize;
        let read = match source.read(&mut buffer[..to_read]) {
            Ok(read) => read,
            Err(err) => {
                let _ = target.flush();
                drop(target);
                let partial = mark_partial(&task.target)?;
                return Err(WormError::io(
                    HataKodu::DiskOkuma,
                    format!("Disk okuma hatasi, partial={}", partial.display()),
                    err,
                ));
            }
        };
        if read == 0 {
            break;
        }

        if let Err(err) = target.write_all(&buffer[..read]) {
            let _ = target.flush();
            drop(target);
            let partial = mark_partial(&task.target)?;
            return Err(WormError::io(
                HataKodu::DosyaYazma,
                format!("Yazma hatasi, partial={}", partial.display()),
                err,
            ));
        }

        if let Some(ctx) = &mut sha256 {
            ctx.update(&buffer[..read]);
        }

        copied += read as u64;
        progress(copied, total);
    }

    if let Err(err) = target.flush() {
        drop(target);
        let partial = mark_partial(&task.target)?;
        return Err(WormError::io(
            HataKodu::DosyaYazma,
            format!("Hedef dosya flush edilemedi, partial={}", partial.display()),
            err,
        ));
    }
    drop(target);

    let mut hash_value = None;
    if copied == total && !cancelled {
        if let Some(ctx) = sha256 {
            let hash = to_hex(&ctx.finalize());
            write_sha256_sidecar(&task.target, &hash)?;
            hash_value = Some(hash);
        }
        success = true;
    }

    if success {
        Ok(DiskAcquisitionResult {
            target: task.target.clone(),
            bytes_copied: copied,
            total_bytes: total,
            sha256: hash_value,
            partial_path: None,
        })
    } else {
        let partial = mark_partial(&task.target)?;
        Err(WormError::new(
            if cancelled {
                HataKodu::Genel
            } else {
                HataKodu::DiskOkuma
            },
            format!(
                "Imaj alma yarida kesildi: {}/{} bayt, partial={}",
                copied,
                total,
                partial.display()
            ),
        ))
    }
}

pub fn verify_image(image_path: impl AsRef<Path>, expected_sha256: &str) -> WormResult<bool> {
    let actual = crate::hash::calculate_file_hash(image_path, crate::hash::HashAlgorithm::Sha256)?;
    Ok(actual.eq_ignore_ascii_case(expected_sha256))
}

pub fn cancel_disk_acquisition() {
    DISK_ACQUISITION_CANCELLED.store(true, Ordering::SeqCst);
}

fn mark_partial(path: &Path) -> WormResult<PathBuf> {
    let partial = PathBuf::from(format!("{}.partial", path.display()));
    if path.exists() {
        fs::rename(path, &partial)
            .map_err(|err| WormError::io(HataKodu::DosyaYazma, "Partial dosya tasinamadi", err))?;
    }
    Ok(partial)
}

#[cfg(unix)]
fn disk_size_impl(path: &Path) -> WormResult<u64> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::FileTypeExt;

    let file = File::open(path)
        .map_err(|err| WormError::io(HataKodu::DiskErisim, "Disk/dosya acilamadi", err))?;
    let metadata = file
        .metadata()
        .map_err(|err| WormError::io(HataKodu::DiskBoyut, "Disk metadata okunamadi", err))?;

    if metadata.is_file() {
        return Ok(metadata.len());
    }

    if metadata.file_type().is_block_device() {
        let mut bytes: u64 = 0;
        const BLKGETSIZE64: libc::c_ulong = 0x8008_1272;
        let rc = unsafe { libc::ioctl(file.as_raw_fd(), BLKGETSIZE64, &mut bytes) };
        if rc == 0 && bytes > 0 {
            return Ok(bytes);
        }
    }

    Err(WormError::new(HataKodu::DiskBoyut, "Disk boyutu alinamadi"))
}

#[cfg(windows)]
fn disk_size_impl(path: &Path) -> WormResult<u64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_SEQUENTIAL_SCAN, FILE_SHARE_READ,
        FILE_SHARE_WRITE, GENERIC_READ, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::IO::DeviceIoControl;

    if !path.to_string_lossy().starts_with(r"\\.\PhysicalDrive") {
        return std::fs::metadata(path)
            .map(|metadata| metadata.len())
            .map_err(|err| WormError::io(HataKodu::DosyaAcilamadi, "Dosya boyutu alinamadi", err));
    }

    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        CreateFileW(
            wide.as_ptr(),
            GENERIC_READ,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL | FILE_FLAG_SEQUENTIAL_SCAN,
            0,
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err(WormError::new(HataKodu::DiskErisim, "Disk acilamadi"));
    }

    let mut output = [0_u8; 8];
    let mut returned = 0_u32;
    const IOCTL_DISK_GET_LENGTH_INFO: u32 = 0x0007_405c;
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_GET_LENGTH_INFO,
            std::ptr::null_mut(),
            0,
            output.as_mut_ptr().cast(),
            output.len() as u32,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    unsafe {
        CloseHandle(handle);
    }

    if ok == 0 {
        return Err(WormError::new(HataKodu::DiskBoyut, "Disk boyutu alinamadi"));
    }

    Ok(u64::from_le_bytes(output))
}

#[cfg(unix)]
fn list_disks_impl() -> WormResult<Vec<DiskInfo>> {
    let mut candidates = Vec::new();

    for letter in b'a'..=b'p' {
        candidates.push(PathBuf::from(format!("/dev/sd{}", letter as char)));
    }
    for index in 0..8 {
        candidates.push(PathBuf::from(format!("/dev/nvme{index}n1")));
    }
    for letter in b'a'..=b'h' {
        candidates.push(PathBuf::from(format!("/dev/vd{}", letter as char)));
    }

    let mut disks = Vec::new();
    for candidate in candidates {
        if let Ok(size) = disk_size(&candidate)
            && size > 0
        {
            let accessible = OpenOptions::new().read(true).open(&candidate).is_ok();
            disks.push(DiskInfo {
                device: candidate,
                total_size: size,
                used_size: size,
                accessible,
            });
        }
    }
    Ok(disks)
}

#[cfg(windows)]
fn list_disks_impl() -> WormResult<Vec<DiskInfo>> {
    let mut disks = Vec::new();
    for index in 0..32 {
        let device = PathBuf::from(format!(r"\\.\PhysicalDrive{index}"));
        if let Ok(size) = disk_size(&device) {
            if size > 0 {
                disks.push(DiskInfo {
                    device,
                    total_size: size,
                    used_size: size,
                    accessible: true,
                });
            }
        }
    }
    Ok(disks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn copies_regular_file_and_writes_sha256() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.bin");
        let target = dir.path().join("target.img");
        let mut file = File::create(&source).unwrap();
        file.write_all(b"disk-data").unwrap();

        let mut task = DiskAcquisitionTask::new(&source, &target);
        task.full_disk = true;
        let result = run_disk_acquisition(&task, |_done, _total| {}).unwrap();

        assert_eq!(result.bytes_copied, 9);
        assert_eq!(fs::read(&target).unwrap(), b"disk-data");
        assert!(PathBuf::from(format!("{}.sha256", target.display())).exists());
        assert!(verify_image(&target, result.sha256.as_ref().unwrap()).unwrap());
    }
}
