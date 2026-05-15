use crate::error::{HataKodu, WormError, WormResult};
use crate::evidence::EvidenceVault;
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportInfo {
    pub title: String,
    pub description: String,
    pub creator: String,
    pub source: String,
    pub hash_sha256: String,
    pub date: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    Txt,
    Json,
}

impl ReportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            ReportFormat::Txt => "txt",
            ReportFormat::Json => "json",
        }
    }
}

pub fn new_report_file_name(case_name: &str, format: ReportFormat) -> String {
    format!(
        "rapor_{}_{}.{}",
        case_name,
        Local::now().format("%Y%m%d_%H%M%S"),
        format.extension()
    )
}

pub fn file_summary(path: impl AsRef<Path>) -> WormResult<String> {
    let path = path.as_ref();
    let metadata = fs::metadata(path)
        .map_err(|err| WormError::io(HataKodu::DosyaAcilamadi, "Dosya bulunamadi", err))?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let modified = metadata
        .modified()
        .ok()
        .map(chrono::DateTime::<Local>::from)
        .map(|time| time.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "bilinmiyor".to_string());
    let (size_value, size_unit) = human_size(metadata.len());

    Ok(format!(
        "Dosya: {name}\nBoyut: {size_value:.2} {size_unit} ({} bayt)\nDegistirme: {modified}\n",
        metadata.len()
    ))
}

pub fn append_system_info(path: impl AsRef<Path>) -> WormResult<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path.as_ref())
        .map_err(|err| WormError::io(HataKodu::DosyaAcilamadi, "Rapor dosyasi acilamadi", err))?;
    let info = system_info();
    writeln!(file, "\n========================================")
        .and_then(|_| writeln!(file, "SISTEM BILGISI"))
        .and_then(|_| writeln!(file, "========================================"))
        .and_then(|_| writeln!(file, "Isletim Sistemi: {}", info.os))
        .and_then(|_| writeln!(file, "Surum: {}", info.version))
        .and_then(|_| writeln!(file, "Makine: {}", info.machine))
        .and_then(|_| writeln!(file, "Hostname: {}", info.hostname))
        .and_then(|_| writeln!(file, "\nKullanici: {}", info.user))
        .and_then(|_| writeln!(file, "PID: {}", std::process::id()))
        .and_then(|_| {
            writeln!(
                file,
                "Rapor Tarihi: {}",
                Local::now().format("%Y-%m-%d %H:%M:%S")
            )
        })
        .map_err(|err| WormError::io(HataKodu::DosyaYazma, "Sistem bilgisi yazilamadi", err))
}

pub fn create_report(
    info: &ReportInfo,
    format: ReportFormat,
    target: impl AsRef<Path>,
    vault: Option<&EvidenceVault>,
) -> WormResult<PathBuf> {
    let target = target.as_ref();
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            WormError::io(HataKodu::DosyaYazma, "Rapor klasoru olusturulamadi", err)
        })?;
    }

    match format {
        ReportFormat::Txt => {
            fs::write(target, render_txt(info))
                .map_err(|err| WormError::io(HataKodu::DosyaYazma, "Rapor yazilamadi", err))?;
            append_system_info(target)?;
        }
        ReportFormat::Json => {
            let system = system_info();
            let content = json!({
                "tur": "adli_bilisim_raporu",
                "versiyon": "1.0",
                "baslik": info.title,
                "aciklama": info.description,
                "olusturan": info.creator,
                "kaynak": info.source,
                "tarih": info.date,
                "hash_sha256": info.hash_sha256,
                "sistem": system,
            });
            fs::write(target, serde_json::to_string_pretty(&content)?)
                .map_err(|err| WormError::io(HataKodu::DosyaYazma, "JSON rapor yazilamadi", err))?;
        }
    }

    if let Some(vault) = vault {
        if let Some(logger) = &vault.logger {
            logger.info(format!("Rapor olusturuldu: {}", target.display()));
        }
    }

    Ok(target.to_path_buf())
}

fn render_txt(info: &ReportInfo) -> String {
    let mut out = String::new();
    out.push_str("========================================\n");
    out.push_str("    ADLI BILISIM TEKNIK RAPORU\n");
    out.push_str("========================================\n\n");
    out.push_str(&format!("BASLIK: {}\n", info.title));
    out.push_str(&format!("ACIKLAMA: {}\n", info.description));
    out.push_str(&format!("OLUSTURAN: {}\n", info.creator));
    out.push_str(&format!("KAYNAK: {}\n", info.source));
    out.push_str(&format!("TARIH: {}\n", info.date));
    if !info.hash_sha256.is_empty() {
        out.push_str("\n----------------------------------------\n");
        out.push_str("HASH DEGERI (SHA-256):\n");
        out.push_str(&info.hash_sha256);
        out.push_str("\n----------------------------------------\n");
    }
    out.push_str("\n========================================\n");
    out.push_str("Sistem tarafindan olusturulmustur.\n");
    out.push_str("========================================\n");
    out
}

fn human_size(bytes: u64) -> (f64, &'static str) {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    if bytes >= GB {
        (bytes as f64 / GB as f64, "GB")
    } else if bytes >= MB {
        (bytes as f64 / MB as f64, "MB")
    } else if bytes >= KB {
        (bytes as f64 / KB as f64, "KB")
    } else {
        (bytes as f64, "B")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemInfo {
    os: String,
    version: String,
    machine: String,
    hostname: String,
    user: String,
}

fn system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        version: os_version(),
        machine: std::env::consts::ARCH.to_string(),
        hostname: hostname(),
        user: std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "bilinmiyor".to_string()),
    }
}

#[cfg(unix)]
fn os_version() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|value| value.trim().to_string())
        .unwrap_or_else(|_| "bilinmiyor".to_string())
}

#[cfg(windows)]
fn os_version() -> String {
    "Windows".to_string()
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| "bilinmiyor".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_report_stays_valid_json() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("report.json");
        let info = ReportInfo {
            title: "T".to_string(),
            description: "D".to_string(),
            creator: "C".to_string(),
            source: "S".to_string(),
            hash_sha256: "abc".to_string(),
            date: "2026-05-15 00:00:00".to_string(),
        };
        create_report(&info, ReportFormat::Json, &target, None).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&target).unwrap()).unwrap();
        assert_eq!(parsed["tur"], "adli_bilisim_raporu");
    }
}
