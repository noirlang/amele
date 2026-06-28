//! Yerel disk listeleme, imaj alma, duraklatma/durdurma ve hash üretimini yapar.
use crate::error::{HataKodu, WormError, WormResult};
use crate::hash::{to_hex, write_sha256_sidecar};
use crate::logging::{LogLevel, runtime_log};
use digest::Digest;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
#[cfg(unix)]
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub const DEFAULT_READ_CHUNK: usize = 4 * 1024 * 1024;

static DISK_ACQUISITION_CANCELLED: AtomicBool = AtomicBool::new(false);

/// UI ve API'ye dönen yerel disk özet bilgisidir.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: PathBuf,
    pub total_size: u64,
    pub used_size: u64,
    pub accessible: bool,
}

/// Disk imajı alırken kullanılacak kaynak, hedef ve okuma ayarlarını taşır.
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

/// Disk imajı alma tamamlandığında veya kısmi kaldığında dönen sonuçtur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiskAcquisitionResult {
    pub target: PathBuf,
    pub bytes_copied: u64,
    pub total_bytes: u64,
    pub sha256: Option<String>,
    pub partial_path: Option<PathBuf>,
}

impl DiskAcquisitionTask {
    /// Kaynak ve hedef ile varsayılan disk edinim görevi oluşturur.
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

/// Dosya veya blok cihaz boyutunu platforma uygun yöntemle hesaplar.
pub fn disk_size(path: impl AsRef<Path>) -> WormResult<u64> {
    disk_size_impl(path.as_ref())
}

/// Platforma göre yerel diskleri listeler.
pub fn list_disks() -> WormResult<Vec<DiskInfo>> {
    list_disks_impl()
}

/// Basit iptal bayrağıyla disk imajı alma işlemini çalıştırır.
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

/// Disk imajı alırken dışarıdan gelen devam/duraklat/iptal durumudur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiskAcquisitionControl {
    Continue,
    Pause,
    Cancel,
}

/// Diskten hedef dosyaya parçalı okuma/yazma yapar, ilerleme ve SHA256 üretir.
pub fn run_disk_acquisition_with_control<F, C>(
    task: &DiskAcquisitionTask,
    mut progress: F,
    mut control: C,
) -> WormResult<DiskAcquisitionResult>
where
    F: FnMut(u64, u64),
    C: FnMut() -> DiskAcquisitionControl,
{
    runtime_log(
        LogLevel::Info,
        "disk",
        format!(
            "Disk imaj alma baslatildi. Kaynak: {}, Hedef: {}",
            task.source.display(),
            task.target.display()
        ),
    );

    let source_size = disk_size(&task.source)?;
    if source_size == 0 {
        let err = WormError::new(HataKodu::DiskBoyut, "Kaynak boyut alinamadi");
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("Kaynak boyut sifir: {:?}", err),
        );
        return Err(err);
    }

    if let Some(parent) = task.target.parent() {
        runtime_log(
            LogLevel::Info,
            "disk",
            format!("Hedef klasor olusturuluyor: {}", parent.display()),
        );
        fs::create_dir_all(parent).map_err(|err| {
            let w_err = WormError::io(HataKodu::DosyaYazma, "Hedef klasor olusturulamadi", err);
            runtime_log(
                LogLevel::Error,
                "disk",
                format!("Klasor olusturma hatasi: {:?}", w_err),
            );
            w_err
        })?;
    }

    let mut source = File::open(&task.source).map_err(|err| {
        let w_err = WormError::io(HataKodu::DiskErisim, "Kaynak acilamadi", err);
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("Kaynak acma hatasi: {:?}", w_err),
        );
        w_err
    })?;

    if task.start_offset > 0 {
        runtime_log(
            LogLevel::Info,
            "disk",
            format!(
                "Kaynak dosyasinda offset'e seek ediliyor: {}",
                task.start_offset
            ),
        );
        source
            .seek(SeekFrom::Start(task.start_offset))
            .map_err(|err| {
                let w_err = WormError::io(HataKodu::DiskOkuma, "Kaynak konumlanamadi", err);
                runtime_log(LogLevel::Error, "disk", format!("Seek hatasi: {:?}", w_err));
                w_err
            })?;
    }

    let mut target = File::create(&task.target).map_err(|err| {
        let w_err = WormError::io(HataKodu::DosyaYazma, "Hedef dosya olusturulamadi", err);
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("Hedef dosya olusturma hatasi: {:?}", w_err),
        );
        w_err
    })?;

    let available = source_size.saturating_sub(task.start_offset);
    let total = if task.full_disk {
        available
    } else {
        task.size.unwrap_or(available).min(available)
    };

    if total == 0 {
        let err = WormError::new(HataKodu::DiskBoyut, "Kopyalanacak veri yok");
        runtime_log(LogLevel::Error, "disk", format!("Hata: {:?}", err));
        return Err(err);
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
                runtime_log(
                    LogLevel::Warn,
                    "disk",
                    "Disk imaj alma kullanici tarafindan iptal edildi.",
                );
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
                let w_err = WormError::io(
                    HataKodu::DiskOkuma,
                    format!("Disk okuma hatasi, partial={}", partial.display()),
                    err,
                );
                runtime_log(
                    LogLevel::Error,
                    "disk",
                    format!("Okuma hatasi: {:?}", w_err),
                );
                return Err(w_err);
            }
        };
        if read == 0 {
            break;
        }

        if let Err(err) = target.write_all(&buffer[..read]) {
            let _ = target.flush();
            drop(target);
            let partial = mark_partial(&task.target)?;
            let w_err = WormError::io(
                HataKodu::DosyaYazma,
                format!("Yazma hatasi, partial={}", partial.display()),
                err,
            );
            runtime_log(
                LogLevel::Error,
                "disk",
                format!("Yazma hatasi: {:?}", w_err),
            );
            return Err(w_err);
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
        let w_err = WormError::io(
            HataKodu::DosyaYazma,
            format!("Hedef dosya flush edilemedi, partial={}", partial.display()),
            err,
        );
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("Flush hatasi: {:?}", w_err),
        );
        return Err(w_err);
    }
    drop(target);

    let mut hash_value = None;
    if copied == total && !cancelled {
        if let Some(ctx) = sha256 {
            let hash = to_hex(&ctx.finalize());
            runtime_log(
                LogLevel::Info,
                "disk",
                format!("SHA256 hash hesaplandi: {}. Sidecar yaziliyor.", hash),
            );
            write_sha256_sidecar(&task.target, &hash).map_err(|err| {
                runtime_log(
                    LogLevel::Error,
                    "disk",
                    format!("Sidecar yazma hatasi: {:?}", err),
                );
                err
            })?;
            hash_value = Some(hash);
        }
        success = true;
    }

    if success {
        runtime_log(
            LogLevel::Info,
            "disk",
            format!(
                "Disk imaj alma basariyla tamamlandi. Toplam {} bayt kopyalandi.",
                copied
            ),
        );
        Ok(DiskAcquisitionResult {
            target: task.target.clone(),
            bytes_copied: copied,
            total_bytes: total,
            sha256: hash_value,
            partial_path: None,
        })
    } else {
        let partial = mark_partial(&task.target)?;
        let w_err = WormError::new(
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
        );
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("Imaj alma yarida kaldi: {:?}", w_err),
        );
        Err(w_err)
    }
}

/// Oluşturulan imajın SHA256 değerini beklenen değerle karşılaştırır.
pub fn verify_image(image_path: impl AsRef<Path>, expected_sha256: &str) -> WormResult<bool> {
    runtime_log(
        LogLevel::Info,
        "disk",
        format!(
            "Imaj dogrulamasi baslatildi: {}",
            image_path.as_ref().display()
        ),
    );
    let actual = crate::hash::calculate_file_hash(image_path, crate::hash::HashAlgorithm::Sha256)
        .map_err(|err| {
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("Hash hesaplama hatasi: {:?}", err),
        );
        err
    })?;
    let matched = actual.eq_ignore_ascii_case(expected_sha256);
    if matched {
        runtime_log(
            LogLevel::Info,
            "disk",
            "Imaj dogrulamasi basarili (SHA256 eslesti).",
        );
    } else {
        runtime_log(
            LogLevel::Warn,
            "disk",
            format!(
                "Imaj dogrulamasi basarisiz. Beklenen: {}, Bulunan: {}",
                expected_sha256, actual
            ),
        );
    }
    Ok(matched)
}

/// Eski tekil disk edinim akışını iptal etmek için global bayrağı işaretler.
pub fn cancel_disk_acquisition() {
    runtime_log(
        LogLevel::Info,
        "disk",
        "Disk imaj alma iptal talebi alindi.",
    );
    DISK_ACQUISITION_CANCELLED.store(true, Ordering::SeqCst);
}

/// Başarısız veya iptal edilmiş imaj dosyasını .partial uzantısıyla korur.
fn mark_partial(path: &Path) -> WormResult<PathBuf> {
    let partial = PathBuf::from(format!("{}.partial", path.display()));
    if path.exists() {
        runtime_log(
            LogLevel::Info,
            "disk",
            format!(
                "Yarida kalan dosya tasiniyor: {} -> {}",
                path.display(),
                partial.display()
            ),
        );
        fs::rename(path, &partial).map_err(|err| {
            let w_err = WormError::io(HataKodu::DosyaYazma, "Partial dosya tasinamadi", err);
            runtime_log(
                LogLevel::Error,
                "disk",
                format!("Rename hatasi: {:?}", w_err),
            );
            w_err
        })?;
    }
    Ok(partial)
}

#[cfg(unix)]
/// Unix sistemlerde dosya veya blok cihaz boyutunu ioctl/metaveri ile hesaplar.
fn disk_size_impl(path: &Path) -> WormResult<u64> {
    use std::os::fd::AsRawFd;
    use std::os::unix::fs::FileTypeExt;

    runtime_log(
        LogLevel::Debug,
        "disk",
        format!("Disk boyutu sorgulaniyor (Unix): {}", path.display()),
    );

    let file = File::open(path).map_err(|err| {
        let w_err = WormError::io(HataKodu::DiskErisim, "Disk/dosya acilamadi", err);
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("Disk acilamadi: {:?}", w_err),
        );
        w_err
    })?;
    let metadata = file.metadata().map_err(|err| {
        let w_err = WormError::io(HataKodu::DiskBoyut, "Disk metadata okunamadi", err);
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("Metadata okuma hatasi: {:?}", w_err),
        );
        w_err
    })?;

    if metadata.is_file() {
        return Ok(metadata.len());
    }

    if metadata.file_type().is_block_device() {
        let mut bytes: u64 = 0;
        const BLKGETSIZE64: libc::c_ulong = 0x8008_1272;
        let rc = unsafe { libc::ioctl(file.as_raw_fd(), BLKGETSIZE64, &mut bytes) };
        if rc == 0 && bytes > 0 {
            runtime_log(
                LogLevel::Info,
                "disk",
                format!("Blok cihazi boyutu ioctl ile okundu: {} bayt", bytes),
            );
            return Ok(bytes);
        }
    }

    let w_err = WormError::new(HataKodu::DiskBoyut, "Disk boyutu alinamadi");
    runtime_log(
        LogLevel::Error,
        "disk",
        format!("Disk boyutu okunamadi: {:?}", w_err),
    );
    Err(w_err)
}

#[cfg(windows)]
/// Windows sistemlerde PhysicalDrive boyutunu DeviceIoControl ile hesaplar.
fn disk_size_impl(path: &Path) -> WormResult<u64> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Foundation::{CloseHandle, GENERIC_READ, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_FLAG_SEQUENTIAL_SCAN, FILE_SHARE_READ,
        FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows_sys::Win32::System::IO::DeviceIoControl;

    runtime_log(
        LogLevel::Debug,
        "disk",
        format!("Disk boyutu sorgulaniyor (Windows): {}", path.display()),
    );

    if !path.to_string_lossy().starts_with(r"\\.\PhysicalDrive") {
        return std::fs::metadata(path)
            .map(|metadata| metadata.len())
            .map_err(|err| {
                let w_err = WormError::io(HataKodu::DosyaAcilamadi, "Dosya boyutu alinamadi", err);
                runtime_log(
                    LogLevel::Error,
                    "disk",
                    format!("Windows metadata okuma hatasi: {:?}", w_err),
                );
                w_err
            });
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
            std::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        let w_err = WormError::new(HataKodu::DiskErisim, "Disk acilamadi");
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("PhysicalDrive acilamadi: {:?}", w_err),
        );
        return Err(w_err);
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
        let w_err = WormError::new(HataKodu::DiskBoyut, "Disk boyutu alinamadi");
        runtime_log(
            LogLevel::Error,
            "disk",
            format!("DeviceIoControl boyutu alamadi: {:?}", w_err),
        );
        return Err(w_err);
    }

    Ok(u64::from_le_bytes(output))
}

#[cfg(unix)]
/// Linux/Unix sistemlerde bilinen blok cihaz adlarını tarayarak disk listesi üretir.
fn list_disks_impl() -> WormResult<Vec<DiskInfo>> {
    runtime_log(
        LogLevel::Info,
        "disk",
        "Linux/Unix disk listesi taraniyor...",
    );
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
    runtime_log(
        LogLevel::Info,
        "disk",
        format!("Disk listeleme bitti. Bulunan disk sayisi: {}", disks.len()),
    );
    Ok(disks)
}

#[cfg(windows)]
/// Windows sistemlerde PhysicalDrive0..31 aralığını tarayarak disk listesi üretir.
fn list_disks_impl() -> WormResult<Vec<DiskInfo>> {
    runtime_log(LogLevel::Info, "disk", "Windows disk listesi taraniyor...");
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
    runtime_log(
        LogLevel::Info,
        "disk",
        format!(
            "Disk listeleme bitti (Windows). Bulunan disk sayisi: {}",
            disks.len()
        ),
    );
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
