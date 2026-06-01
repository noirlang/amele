use crate::server::{Response, json_error};
use chrono::Local;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Clone)]
pub struct EvidenceCaseState {
    pub base_dir: PathBuf,
    pub case_name: String,
}

#[derive(Clone)]
pub struct ImageMountState {
    #[cfg(windows)]
    pub image_path: PathBuf,
    pub mount_dir: PathBuf,
    #[cfg(target_os = "linux")]
    pub loop_device: Option<PathBuf>,
}

pub fn current_evidence_case() -> &'static Mutex<Option<EvidenceCaseState>> {
    static CURRENT_EVIDENCE_CASE: OnceLock<Mutex<Option<EvidenceCaseState>>> = OnceLock::new();
    CURRENT_EVIDENCE_CASE.get_or_init(|| Mutex::new(None))
}

pub fn current_image_mount() -> &'static Mutex<Option<ImageMountState>> {
    static CURRENT_IMAGE_MOUNT: OnceLock<Mutex<Option<ImageMountState>>> = OnceLock::new();
    CURRENT_IMAGE_MOUNT.get_or_init(|| Mutex::new(None))
}

pub fn wireguard_manager() -> &'static Mutex<crate::wireguard::WireGuardManager> {
    static WIREGUARD_MANAGER: OnceLock<Mutex<crate::wireguard::WireGuardManager>> = OnceLock::new();
    WIREGUARD_MANAGER.get_or_init(|| Mutex::new(crate::wireguard::WireGuardManager::new()))
}

pub fn default_case_base_dir() -> PathBuf {
    #[cfg(test)]
    if let Some(path) = test_case_base_dir()
        .lock()
        .ok()
        .and_then(|current| current.clone())
    {
        return path;
    }

    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Worm")
        .join("Vakalar")
}

pub fn default_case_name() -> String {
    format!("Case_{}", Local::now().format("%Y%m%d_%H%M%S"))
}

pub fn sanitize_case_name(value: &str) -> String {
    let sanitized = sanitize_file_stem(value);
    if sanitized.is_empty() {
        String::new()
    } else {
        sanitized
    }
}

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

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

pub fn set_current_evidence_case(base_dir: PathBuf, case_name: String) {
    if let Ok(mut current) = current_evidence_case().lock() {
        *current = Some(EvidenceCaseState {
            base_dir,
            case_name,
        });
    }
}

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
    } else if let Some(state) = current_evidence_case()
        .lock()
        .ok()
        .and_then(|current| current.clone())
    {
        (state.base_dir, state.case_name)
    } else {
        (default_case_base_dir(), default_case_name())
    };

    let vault = crate::evidence::EvidenceVault::create(&base_dir, &case_name)
        .map_err(|err| err.to_string())?;
    set_current_evidence_case(base_dir, case_name);
    Ok(vault)
}

pub fn current_evidence_vault() -> Result<crate::evidence::EvidenceVault, Response> {
    let state = current_evidence_case()
        .lock()
        .ok()
        .and_then(|current| current.clone())
        .ok_or_else(|| json_error(400, "case is not created"))?;
    crate::evidence::EvidenceVault::create(&state.base_dir, &state.case_name)
        .map_err(|err| json_error(500, err.to_string()))
}

pub fn report_evidence_vault(
    case_name: Option<&str>,
) -> Result<crate::evidence::EvidenceVault, Response> {
    let explicit_case = case_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_case_name)
        .filter(|value| !value.is_empty());

    let (base_dir, case_name) = if let Some(case_name) = explicit_case {
        (default_case_base_dir(), case_name)
    } else if let Some(state) = current_evidence_case()
        .lock()
        .ok()
        .and_then(|current| current.clone())
    {
        (state.base_dir, state.case_name)
    } else {
        (default_case_base_dir(), default_case_name())
    };

    let vault = crate::evidence::EvidenceVault::create(&base_dir, &case_name)
        .map_err(|err| json_error(500, err.to_string()))?;
    set_current_evidence_case(base_dir, case_name);
    Ok(vault)
}

pub fn evidence_subdir(value: &str) -> &'static str {
    match value {
        "gunlukler" | "logs" => "gunlukler",
        "raporlar" | "reports" => "raporlar",
        "hash" => "hash",
        "notlar" | "notes" => "notlar",
        "disk_imajlari" | "ciktilar" | "outputs" | "images" => "ciktilar",
        "ram" => "ram",
        "android" => "android",
        _ => "ciktilar",
    }
}

#[cfg(test)]
pub fn test_case_base_dir() -> &'static Mutex<Option<PathBuf>> {
    static TEST_CASE_BASE_DIR: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    TEST_CASE_BASE_DIR.get_or_init(|| Mutex::new(None))
}
