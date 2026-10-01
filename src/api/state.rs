//! Aktif görevler, paylaşılan durum nesneleri ve kasa (vault) durum yöneticisi.

use crate::server::{Response, json_error};
use chrono::Local;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
/// Aktif vaka adını ve taban klasörünü global API durumunda tutar.
pub struct EvidenceCaseState {
    pub base_dir: PathBuf,
    pub case_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Aktif bağlı imajın dosya, mount klasörü ve varsa loop cihaz bilgisidir.
pub struct ImageMountState {
    pub image_path: PathBuf,
    pub mount_dir: PathBuf,
    #[cfg(target_os = "linux")]
    pub loop_device: Option<PathBuf>,
}

/// Aktif vaka durumunu saklayan global mutex'i döndürür.
pub fn current_evidence_case() -> &'static Mutex<Option<EvidenceCaseState>> {
    static CURRENT_EVIDENCE_CASE: OnceLock<Mutex<Option<EvidenceCaseState>>> = OnceLock::new();
    CURRENT_EVIDENCE_CASE.get_or_init(|| Mutex::new(None))
}

/// Aktif vaka mutex kilidini zehirlenmeye karşı korumalı olarak alır.
pub fn lock_current_evidence_case() -> std::sync::MutexGuard<'static, Option<EvidenceCaseState>> {
    current_evidence_case()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

/// Aktif imaj mount durumunu saklayan global mutex'i döndürür.
pub fn current_image_mount() -> &'static Mutex<Option<ImageMountState>> {
    static CURRENT_IMAGE_MOUNT: OnceLock<Mutex<Option<ImageMountState>>> = OnceLock::new();
    CURRENT_IMAGE_MOUNT.get_or_init(|| Mutex::new(None))
}

/// Aktif imaj mount mutex kilidini zehirlenmeye karşı korumalı olarak alır.
pub fn lock_current_image_mount() -> std::sync::MutexGuard<'static, Option<ImageMountState>> {
    current_image_mount()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
}

/// WireGuard yöneticisini saklayan global mutex'i döndürür.
pub fn wireguard_manager() -> &'static Mutex<crate::wireguard::WireGuardManager> {
    static WIREGUARD_MANAGER: OnceLock<Mutex<crate::wireguard::WireGuardManager>> = OnceLock::new();
    WIREGUARD_MANAGER.get_or_init(|| Mutex::new(crate::wireguard::WireGuardManager::new()))
}

/// Aktif profile göre varsayılan vaka taban klasörünü döndürür.
pub fn default_case_base_dir() -> PathBuf {
    #[cfg(test)]
    if let Some(path) = test_case_base_dir()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .clone()
    {
        return path;
    }

    if let Some(path) = crate::profile::active_case_base_dir() {
        return path;
    }

    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Amele")
        .join("Vakalar")
}

/// Tarih damgalı varsayılan vaka adı üretir.
pub fn default_case_name() -> String {
    format!("Case_{}", Local::now().format("%Y%m%d_%H%M%S"))
}

/// Kullanıcıdan gelen vaka adını güvenli dosya adına çevirir.
pub fn sanitize_case_name(value: &str) -> String {
    let sanitized = sanitize_file_stem(value);
    let trimmed = sanitized
        .trim_matches(|c| c == '.' || c == '_' || c == '-')
        .to_string();
    if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        String::new()
    } else {
        trimmed
    }
}

/// Dosya adı kökü için güvenli karakter setine indirger.
pub fn sanitize_file_stem(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    sanitized.trim_matches('_').to_string()
}

/// HOME/USERPROFILE ortam değişkeninden home klasörünü bulur.
pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Aktif vaka durumunu global state içine yazar.
pub fn set_current_evidence_case(base_dir: PathBuf, case_name: String) {
    let mut current = lock_current_evidence_case();
    *current = Some(EvidenceCaseState {
        base_dir,
        case_name,
    });
}

/// Çıktı üretirken açık vaka yoksa yeni vaka oluşturarak kasa döndürür.
pub fn evidence_vault_for_output(
    case_name: Option<&str>,
) -> Result<crate::evidence::EvidenceVault, String> {
    let explicit_case = case_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_case_name)
        .filter(|value| !value.is_empty());

    let (base_dir, case_name) = if let Some(case_name) = explicit_case {
        (default_case_base_dir(), case_name)
    } else if let Some(state) = lock_current_evidence_case().clone() {
        (state.base_dir, state.case_name)
    } else {
        (default_case_base_dir(), default_case_name())
    };

    let vault = crate::evidence::EvidenceVault::create(&base_dir, &case_name)
        .map_err(|err| err.to_string())?;
    set_current_evidence_case(base_dir, case_name);
    Ok(vault)
}

/// Mutlaka mevcut aktif vaka isteyen endpointler için kasa döndürür.
pub fn current_evidence_vault() -> Result<crate::evidence::EvidenceVault, Response> {
    let state = lock_current_evidence_case()
        .clone()
        .ok_or_else(|| json_error(400, "case is not created"))?;
    crate::evidence::EvidenceVault::create(&state.base_dir, &state.case_name)
        .map_err(|err| json_error(500, err.to_string()))
}

/// Rapor üretiminde vaka seçimi yoksa aktif veya yeni vaka kasası döndürür.
pub fn report_evidence_vault(
    case_name: Option<&str>,
) -> Result<crate::evidence::EvidenceVault, Response> {
    evidence_vault_for_output(case_name).map_err(|err| json_error(500, err))
}

/// Sunucu port numarasını saklar (developer konsol penceresi için).
pub fn current_server_port() -> u16 {
    server_port_static()
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .unwrap_or(0)
}

/// Sunucu port numarasını ayarlar (server.rs başlangıçta çağırır).
pub fn set_server_port(port: u16) {
    let mut p = server_port_static()
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    *p = Some(port);
}

/// Port static'ini tek bir yerde tanımlar.
fn server_port_static() -> &'static Mutex<Option<u16>> {
    static PORT: OnceLock<Mutex<Option<u16>>> = OnceLock::new();
    PORT.get_or_init(|| Mutex::new(None))
}

/// UI/API alt klasör adlarını kasa içindeki gerçek klasörlere eşler.
pub fn evidence_subdir(value: &str) -> &'static str {
    match value {
        "gunlukler" | "logs" => "gunlukler",
        "raporlar" | "reports" => "raporlar",
        "hash" => "hash",
        "notlar" | "notes" => "notlar",
        "disk_imajlari" | "ciktilar" | "outputs" | "images" => "ciktilar",
        "ram" => "ram",
        "android" => "android",
        "ios" => "ios",
        "docker" => "docker",
        _ => "ciktilar",
    }
}

#[cfg(test)]
/// Testlerde varsayılan vaka taban klasörünü geçici klasöre yönlendirir.
pub fn test_case_base_dir() -> &'static Mutex<Option<PathBuf>> {
    static TEST_CASE_BASE_DIR: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    TEST_CASE_BASE_DIR.get_or_init(|| Mutex::new(None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_case_name() {
        assert_eq!(sanitize_case_name("Case_2026"), "Case_2026");
        assert_eq!(sanitize_case_name("../../../etc/passwd"), "etc_passwd");
        assert_eq!(sanitize_case_name(".."), "");
        assert_eq!(sanitize_case_name("."), "");
        assert_eq!(sanitize_case_name("---___..."), "");
        assert_eq!(
            sanitize_case_name("Vaka 123 (Şüpheli)"),
            "Vaka_123____pheli"
        );
    }

    #[test]
    fn test_sanitize_file_stem() {
        assert_eq!(
            sanitize_file_stem("hello_world-123.txt"),
            "hello_world-123.txt"
        );
        assert_eq!(sanitize_file_stem("___test___"), "test");
        assert_eq!(sanitize_file_stem("bad*chars?here"), "bad_chars_here");
    }

    #[test]
    fn test_evidence_subdir() {
        assert_eq!(evidence_subdir("gunlukler"), "gunlukler");
        assert_eq!(evidence_subdir("logs"), "gunlukler");
        assert_eq!(evidence_subdir("raporlar"), "raporlar");
        assert_eq!(evidence_subdir("reports"), "raporlar");
        assert_eq!(evidence_subdir("hash"), "hash");
        assert_eq!(evidence_subdir("notlar"), "notlar");
        assert_eq!(evidence_subdir("notes"), "notlar");
        assert_eq!(evidence_subdir("ciktilar"), "ciktilar");
        assert_eq!(evidence_subdir("disk_imajlari"), "ciktilar");
        assert_eq!(evidence_subdir("outputs"), "ciktilar");
        assert_eq!(evidence_subdir("images"), "ciktilar");
        assert_eq!(evidence_subdir("ram"), "ram");
        assert_eq!(evidence_subdir("android"), "android");
        assert_eq!(evidence_subdir("ios"), "ios");
        assert_eq!(evidence_subdir("docker"), "docker");
        assert_eq!(evidence_subdir("unknown_xyz"), "ciktilar");
    }

    #[test]
    fn test_set_and_get_current_evidence_case() {
        let base = PathBuf::from("/tmp/test_amele_cases");
        let name = "Test_Case_001".to_string();
        set_current_evidence_case(base.clone(), name.clone());

        let state = lock_current_evidence_case()
            .clone()
            .expect("case should be set");
        assert_eq!(state.base_dir, base);
        assert_eq!(state.case_name, name);
    }

    #[test]
    fn test_server_port_state() {
        set_server_port(8088);
        assert_eq!(current_server_port(), 8088);
    }

    #[test]
    fn test_default_case_name_format() {
        let name = default_case_name();
        assert!(name.starts_with("Case_"));
        assert!(name.len() > 10);
    }

    #[test]
    fn test_report_evidence_vault_delegation() {
        let temp_dir = std::env::temp_dir().join("amele_state_test_vault");
        let _ = std::fs::create_dir_all(&temp_dir);

        *test_case_base_dir()
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(temp_dir.clone());

        let result = report_evidence_vault(Some("Direct_Case_Test"));
        assert!(result.is_ok());
        let vault = result.unwrap();
        assert_eq!(vault.case_name, "Direct_Case_Test");

        *test_case_base_dir()
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = None;
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
