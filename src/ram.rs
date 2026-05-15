use crate::error::{HataKodu, WormError, WormResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RamToolStatus {
    pub tool_present: bool,
    pub admin_privilege: bool,
    pub ram_size: u64,
    pub tool_path: Option<PathBuf>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RamAcquisitionResult {
    pub output_file: PathBuf,
    pub bytes_written: u64,
}

#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

pub fn avml_status(candidate: Option<&Path>) -> RamToolStatus {
    let tool = find_avml(candidate);
    RamToolStatus {
        tool_present: tool.is_some(),
        admin_privilege: is_root_or_admin(),
        ram_size: physical_ram_size(),
        tool_path: tool.clone(),
        message: if tool.is_some() {
            "AVML ready".to_string()
        } else {
            "AVML not found".to_string()
        },
    }
}

pub fn acquire_with_avml<F>(
    output_file: impl AsRef<Path>,
    candidate: Option<&Path>,
    cancellation: &CancellationToken,
    mut progress: F,
) -> WormResult<RamAcquisitionResult>
where
    F: FnMut(u64, u64),
{
    #[cfg(windows)]
    {
        let _ = output_file;
        let _ = candidate;
        let _ = cancellation;
        let _ = progress;
        return Err(WormError::new(
            HataKodu::YetkisizErisim,
            "AVML Windows uzerinde kullanilmaz",
        ));
    }

    #[cfg(not(windows))]
    {
        if !is_root_or_admin() {
            return Err(WormError::new(
                HataKodu::YetkisizErisim,
                "RAM edinimi icin root yetkisi gerekli",
            ));
        }

        let avml = find_avml(candidate)
            .ok_or_else(|| WormError::new(HataKodu::DosyaAcilamadi, "AVML bulunamadi"))?;
        if let Some(parent) = output_file.as_ref().parent() {
            fs::create_dir_all(parent).map_err(|err| {
                WormError::io(
                    HataKodu::DosyaYazma,
                    "RAM cikti klasoru olusturulamadi",
                    err,
                )
            })?;
        }

        let total = physical_ram_size();
        let mut child = Command::new(&avml)
            .arg(output_file.as_ref())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| WormError::io(HataKodu::Genel, "AVML baslatilamadi", err))?;
        monitor_child_file(
            &mut child,
            output_file.as_ref(),
            total,
            Duration::from_secs(7200),
            cancellation,
            &mut progress,
        )
    }
}

pub fn winpmem_status(candidate: Option<&Path>) -> RamToolStatus {
    let tool = find_winpmem(candidate);
    RamToolStatus {
        tool_present: tool.is_some(),
        admin_privilege: is_root_or_admin(),
        ram_size: physical_ram_size(),
        tool_path: tool.clone(),
        message: if tool.is_some() {
            "WinPMEM ready".to_string()
        } else {
            "WinPMEM not found".to_string()
        },
    }
}

pub fn acquire_with_winpmem<F>(
    output_file: impl AsRef<Path>,
    candidate: Option<&Path>,
    cancellation: &CancellationToken,
    mut progress: F,
) -> WormResult<RamAcquisitionResult>
where
    F: FnMut(u64, u64),
{
    #[cfg(not(windows))]
    {
        let _ = output_file;
        let _ = candidate;
        let _ = cancellation;
        let _ = &mut progress;
        return Err(WormError::new(
            HataKodu::YetkisizErisim,
            "WinPMEM sadece Windows uzerinde desteklenir",
        ));
    }

    #[cfg(windows)]
    {
        if !is_root_or_admin() {
            return Err(WormError::new(
                HataKodu::YetkisizErisim,
                "Administrator privileges required",
            ));
        }
        let winpmem = find_winpmem(candidate)
            .ok_or_else(|| WormError::new(HataKodu::DosyaAcilamadi, "WinPMEM bulunamadi"))?;
        if let Some(parent) = output_file.as_ref().parent() {
            fs::create_dir_all(parent).map_err(|err| {
                WormError::io(
                    HataKodu::DosyaYazma,
                    "RAM cikti klasoru olusturulamadi",
                    err,
                )
            })?;
        }

        let total = physical_ram_size();
        let commands: Vec<Vec<std::ffi::OsString>> = vec![
            vec![
                winpmem.clone().into_os_string(),
                "acquire".into(),
                output_file.as_ref().as_os_str().to_os_string(),
            ],
            vec![
                winpmem.clone().into_os_string(),
                "acquire".into(),
                "--output".into(),
                output_file.as_ref().as_os_str().to_os_string(),
            ],
            vec![
                winpmem.clone().into_os_string(),
                "-o".into(),
                output_file.as_ref().as_os_str().to_os_string(),
                "-1".into(),
            ],
        ];

        let mut last_error = String::new();
        for command in commands {
            let mut iter = command.into_iter();
            let executable = iter.next().expect("command has executable");
            let mut child = Command::new(executable)
                .args(iter)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();
            match child.as_mut() {
                Ok(child) => {
                    let result = monitor_child_file(
                        child,
                        output_file.as_ref(),
                        total,
                        Duration::from_secs(3600),
                        cancellation,
                        &mut progress,
                    );
                    if result.is_ok() {
                        return result;
                    }
                    last_error = result.err().map(|err| err.to_string()).unwrap_or_default();
                }
                Err(err) => {
                    last_error = err.to_string();
                }
            }
        }

        Err(WormError::new(
            HataKodu::Genel,
            format!("WinPMEM komutu baslatilamadi: {last_error}"),
        ))
    }
}

fn monitor_child_file<F>(
    child: &mut Child,
    output_file: &Path,
    total: u64,
    timeout: Duration,
    cancellation: &CancellationToken,
    progress: &mut F,
) -> WormResult<RamAcquisitionResult>
where
    F: FnMut(u64, u64),
{
    let started = Instant::now();
    loop {
        if cancellation.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(WormError::new(HataKodu::Genel, "RAM edinimi iptal edildi"));
        }

        if let Ok(Some(status)) = child.try_wait() {
            let size = fs::metadata(output_file)
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            if status.success() && size > 0 {
                progress(size, total);
                return Ok(RamAcquisitionResult {
                    output_file: output_file.to_path_buf(),
                    bytes_written: size,
                });
            }
            return Err(WormError::new(
                HataKodu::DosyaYazma,
                format!("RAM araci basarisiz oldu: {status}"),
            ));
        }

        if started.elapsed() > timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(WormError::new(HataKodu::Genel, "RAM edinimi zaman asimi"));
        }

        if let Ok(metadata) = fs::metadata(output_file) {
            progress(metadata.len(), total);
        }

        thread::sleep(Duration::from_secs(1));
    }
}

pub fn find_avml(candidate: Option<&Path>) -> Option<PathBuf> {
    if let Some(path) = candidate {
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    find_in_path("avml").or_else(|| {
        ["/usr/bin/avml", "/usr/local/bin/avml"]
            .iter()
            .map(PathBuf::from)
            .find(|path| path.exists())
    })
}

pub fn find_winpmem(candidate: Option<&Path>) -> Option<PathBuf> {
    const WINPMEM_NAME: &str = "go-winpmem_amd64_1.0-rc2_signed.exe";
    if let Some(path) = candidate {
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }

    find_in_path(WINPMEM_NAME).or_else(|| {
        [
            PathBuf::from(WINPMEM_NAME),
            PathBuf::from(r"C:\Forensics\go-winpmem_amd64_1.0-rc2_signed.exe"),
            PathBuf::from(r"C:\Tools\go-winpmem_amd64_1.0-rc2_signed.exe"),
        ]
        .into_iter()
        .find(|path| path.exists())
    })
}

fn find_in_path(binary: &str) -> Option<PathBuf> {
    let paths = std::env::var_os("PATH")?;
    std::env::split_paths(&paths)
        .map(|dir| dir.join(binary))
        .find(|path| path.exists())
}

pub fn physical_ram_size() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(meminfo) = fs::read_to_string("/proc/meminfo") {
            for line in meminfo.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    let kb = rest
                        .split_whitespace()
                        .next()
                        .and_then(|value| value.parse::<u64>().ok())
                        .unwrap_or(0);
                    return kb * 1024;
                }
            }
        }
        0
    }

    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Memory::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
        let mut info = MEMORYSTATUSEX {
            dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
            dwMemoryLoad: 0,
            ullTotalPhys: 0,
            ullAvailPhys: 0,
            ullTotalPageFile: 0,
            ullAvailPageFile: 0,
            ullTotalVirtual: 0,
            ullAvailVirtual: 0,
            ullAvailExtendedVirtual: 0,
        };
        let ok = unsafe { GlobalMemoryStatusEx(&mut info) };
        if ok != 0 { info.ullTotalPhys } else { 0 }
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        0
    }
}

pub fn is_root_or_admin() -> bool {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", "net", "session"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(not(any(unix, windows)))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linux_ram_size_is_nonzero_on_linux() {
        #[cfg(target_os = "linux")]
        assert!(physical_ram_size() > 0);
    }
}
