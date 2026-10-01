//! ios cihaz yedekleme api rotası.

use crate::api::{
    append_acquisition_log, create_acquisition_job, evidence_vault_for_output,
    fail_acquisition_job_with_message, finish_acquisition_job_with_message, sanitize_file_stem,
    update_acquisition_progress_message,
};
use crate::hash::HashAlgorithm;
use crate::ios;
use crate::ram;
use crate::server::{Response, json_error, json_ok};
use chrono::Local;
use serde::Deserialize;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::thread;

/// iOS backup klasörünü hızlıca doğrular ve cihaz/backup profilini döndürür.
pub fn ios_backup_profile_endpoint(body: &[u8]) -> Response {
    if let Some(response) = crate::api::profile::require_mobile_tools_response() {
        return response;
    }

    #[derive(Deserialize)]
    struct IosProfileRequest {
        backup_path: String,
    }

    let request: IosProfileRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let backup_path = request.backup_path.trim();
    if backup_path.is_empty() {
        return json_error(400, "backup_path is required");
    }

    let path = Path::new(backup_path);
    if !path.exists() {
        return json_error(
            404,
            format!("iOS backup klasoru bulunamadi: {}", backup_path),
        );
    }
    if !path.is_dir() {
        return json_error(
            400,
            format!("iOS backup yolu bir klasor olmalidir: {}", backup_path),
        );
    }

    let _ = crate::profile::record_active_profile_activity(
        "ios",
        "backup_profile",
        None,
        Some(backup_path),
    );

    match ios::inspect_backup(path) {
        Ok(info) => json_ok(json!({ "info": info })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// iOS backup normalizasyon işini arka planda başlatır.
pub fn ios_backup_normalize_endpoint(body: &[u8]) -> Response {
    if let Some(response) = crate::api::profile::require_mobile_tools_response() {
        return response;
    }

    let request: IosNormalizeRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let backup_path_str = request.backup_path.trim();
    if backup_path_str.is_empty() {
        return json_error(400, "backup_path is required");
    }

    let path = Path::new(backup_path_str);
    if !path.exists() {
        return json_error(
            404,
            format!("iOS backup klasoru bulunamadi: {}", backup_path_str),
        );
    }
    if !path.is_dir() {
        return json_error(
            400,
            format!("iOS backup yolu bir klasor olmalidir: {}", backup_path_str),
        );
    }

    let _ = crate::profile::record_active_profile_activity(
        "ios",
        "backup_normalize",
        request.case_name.as_deref(),
        Some(backup_path_str),
    );

    let (job_id, control) = create_acquisition_job("iOS backup normalizasyonu baslatildi");
    let thread_job_id = job_id.clone();
    thread::spawn(move || run_ios_backup_job(thread_job_id, request, control));

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

#[derive(Debug, Clone, Deserialize)]
struct IosNormalizeRequest {
    backup_path: String,
    case_name: Option<String>,
    hash_algorithms: Option<Vec<String>>,
}

fn run_ios_backup_job(
    job_id: String,
    request: IosNormalizeRequest,
    control: ram::CancellationToken,
) {
    let backup_path = PathBuf::from(request.backup_path.trim());
    crate::logging::runtime_log(
        crate::logging::LogLevel::Info,
        "ios",
        format!(
            "IS BASLADI | job_id={job_id} | backup={} | vaka={}",
            backup_path.display(),
            request.case_name.as_deref().unwrap_or("(otomatik)")
        ),
    );

    let vault = match evidence_vault_for_output(request.case_name.as_deref()) {
        Ok(vault) => vault,
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err, "iOS backup normalizasyonu basarisiz");
            return;
        }
    };

    let output_dir = ios_edinim_klasoru(&vault.ios_dir, &backup_path);
    let algorithms = parse_hash_algorithms(request.hash_algorithms.as_deref());
    let progress_job_id = job_id.clone();
    let log_job_id = job_id.clone();
    let pause_control = control.clone();
    let cancel_control = control.clone();

    match ios::normalize_backup(
        &backup_path,
        &output_dir,
        &algorithms,
        |done, total, label| {
            update_acquisition_progress_message(&progress_job_id, done, total, label);
        },
        |line| {
            append_acquisition_log(&log_job_id, line);
        },
        || pause_control.is_paused(),
        || cancel_control.is_cancelled(),
    ) {
        Ok(result) => {
            finish_acquisition_job_with_message(
                &job_id,
                json!({
                    "message": "iOS backup normalizasyonu tamamlandi",
                    "output_dir": result.output_dir,
                    "log_path": result.log_path,
                    "manifest_path": result.manifest_path,
                    "manifest_sha256": result.manifest_sha256,
                    "total_entries": result.total_entries,
                    "files_copied": result.files_copied,
                    "directories": result.directories,
                    "symlinks": result.symlinks,
                    "missing": result.missing,
                    "errors": result.errors,
                    "total_bytes": result.total_bytes,
                    "encrypted": result.encrypted,
                    "case_name": vault.case_name,
                }),
                "iOS backup normalizasyonu tamamlandi",
            );
        }
        Err(err) => {
            fail_acquisition_job_with_message(
                &job_id,
                err.to_string(),
                "iOS backup normalizasyonu basarisiz",
            );
        }
    }
}

fn parse_hash_algorithms(values: Option<&[String]>) -> Vec<HashAlgorithm> {
    let mut parsed: Vec<HashAlgorithm> = Vec::new();
    if let Some(list) = values {
        for value in list {
            if let Some(algorithm) = HashAlgorithm::parse(value) {
                if matches!(
                    algorithm,
                    HashAlgorithm::Md5 | HashAlgorithm::Sha1 | HashAlgorithm::Sha256
                ) && !parsed.contains(&algorithm)
                {
                    parsed.push(algorithm);
                }
            }
        }
    }
    if parsed.is_empty() {
        vec![
            HashAlgorithm::Md5,
            HashAlgorithm::Sha1,
            HashAlgorithm::Sha256,
        ]
    } else {
        parsed
    }
}

fn ios_edinim_klasoru(base_dir: &Path, backup_path: &Path) -> PathBuf {
    let source_name = backup_path
        .file_name()
        .and_then(|value| value.to_str())
        .map(sanitize_file_stem)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "ios_backup".to_string());
    let stamp = Local::now().format("%Y%m%d_%H%M%S");
    base_dir.join(format!("{source_name}_{stamp}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hash_algorithms_default() {
        let algs = parse_hash_algorithms(None);
        assert_eq!(
            algs,
            vec![
                HashAlgorithm::Md5,
                HashAlgorithm::Sha1,
                HashAlgorithm::Sha256,
            ]
        );

        let algs_empty = parse_hash_algorithms(Some(&[]));
        assert_eq!(
            algs_empty,
            vec![
                HashAlgorithm::Md5,
                HashAlgorithm::Sha1,
                HashAlgorithm::Sha256,
            ]
        );
    }

    #[test]
    fn test_parse_hash_algorithms_deduplicated_and_filtered() {
        let input = vec![
            "sha256".to_string(),
            "MD5".to_string(),
            "sha256".to_string(),
            "sha512".to_string(),
            "sha1".to_string(),
        ];
        let algs = parse_hash_algorithms(Some(&input));
        assert_eq!(
            algs,
            vec![
                HashAlgorithm::Sha256,
                HashAlgorithm::Md5,
                HashAlgorithm::Sha1,
            ]
        );
    }

    #[test]
    fn test_ios_edinim_klasoru() {
        let base = Path::new("/tmp/cases/case1/ios");
        let backup = Path::new("/var/backup/iPhone_15_Pro");
        let dir = ios_edinim_klasoru(base, backup);
        let dir_str = dir.to_string_lossy();
        assert!(dir_str.starts_with("/tmp/cases/case1/ios/iPhone_15_Pro_"));
    }

    #[test]
    fn test_ios_endpoints_require_mobile_tools_or_validate() {
        let resp = ios_backup_profile_endpoint(b"");
        assert!(resp.status == 400 || resp.status == 403);

        let resp2 = ios_backup_normalize_endpoint(b"");
        assert!(resp2.status == 400 || resp2.status == 403);
    }
}
