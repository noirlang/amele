//! Depolama alanı ön kontrolleri ve aktif bağlama (mount) noktası yönetimi API uç noktaları.

use crate::api::lock_current_evidence_case;
use crate::mount_tracker::{cleanup_all_mounts, cleanup_case_mounts, list_active_mounts};
use crate::server::{Response, json_error, json_ok, json_serialize};
use crate::storage_guard::preflight_check;
use serde::Deserialize;
use std::path::PathBuf;

/// Depolama ön kontrolü istek parametreleri.
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct PreflightRequest {
    source_path: String,
    source_type: String,
    target_path: Option<String>,
}

/// Disk veya imaj edinimi öncesinde depolama alanı uygunluk kontrolü yapar.
pub fn preflight_storage_check_endpoint(body: &[u8]) -> Response {
    let req: PreflightRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(e) => return json_error(400, e.to_string()),
    };

    if req.source_path.trim().is_empty() || req.source_type.trim().is_empty() {
        return json_error(400, "source_path ve source_type alanları zorunludur");
    }

    let target_path = req
        .target_path
        .map(PathBuf::from)
        .or_else(|| {
            lock_current_evidence_case()
                .as_ref()
                .map(|s| s.base_dir.join(&s.case_name))
        })
        .unwrap_or_else(|| PathBuf::from("."));

    let res = preflight_check(&req.source_path, &req.source_type, &target_path);
    json_serialize(&res)
}

/// Aktif olarak bağlanmış imaj ve loop cihazlarının listesini döndürür.
pub fn active_mounts_endpoint() -> Response {
    json_serialize(&list_active_mounts())
}

/// Bağlama temizleme isteği parametresi.
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct CleanupRequest {
    case_name: Option<String>,
}

/// Bağlı imajları sistemden güvenli biçimde ayırır ve temizler.
pub fn cleanup_mounts_endpoint(body: &[u8]) -> Response {
    let case_name = serde_json::from_slice::<CleanupRequest>(body)
        .ok()
        .and_then(|r| r.case_name)
        .filter(|s| !s.trim().is_empty());

    let cleaned = match case_name.as_deref() {
        Some(name) => cleanup_case_mounts(name),
        None => cleanup_all_mounts(),
    };

    let _ = crate::profile::record_active_profile_activity(
        "storage",
        "cleanup_mounts",
        case_name.as_deref(),
        Some(&format!("cleaned_count={}", cleaned.len())),
    );

    json_ok(serde_json::json!({ "cleaned_mounts": cleaned }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preflight_storage_check_endpoint_validation() {
        assert_eq!(preflight_storage_check_endpoint(b"").status, 400);
        assert_eq!(
            preflight_storage_check_endpoint(b"invalid json").status,
            400
        );
        assert_eq!(
            preflight_storage_check_endpoint(br#"{"source_path":"","source_type":"disk"}"#).status,
            400
        );
        assert_eq!(
            preflight_storage_check_endpoint(br#"{"source_path":"/dev/sda","source_type":""}"#)
                .status,
            400
        );
    }

    #[test]
    fn test_preflight_storage_check_endpoint_success() {
        let resp = preflight_storage_check_endpoint(
            br#"{"source_path":"/tmp","source_type":"folder","target_path":"/tmp"}"#,
        );
        assert_eq!(resp.status, 200);
    }

    #[test]
    fn test_active_mounts_endpoint() {
        let resp = active_mounts_endpoint();
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val.is_array());
    }

    #[test]
    fn test_cleanup_mounts_endpoint_all() {
        let resp = cleanup_mounts_endpoint(b"{}");
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val.get("cleaned_mounts").is_some());
    }

    #[test]
    fn test_cleanup_mounts_endpoint_case() {
        let resp = cleanup_mounts_endpoint(br#"{"case_name":"Case_Unit_Test"}"#);
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val.get("cleaned_mounts").is_some());
    }
}
