//! wireguard vpn tüneli açıp kapatma api rotaları.

use super::wireguard_manager;
use crate::profile::record_active_profile_activity;
use crate::server::{Response, json_error, json_ok};
use crate::wireguard::{self, WireGuardConfig, WireGuardManager};
use serde::Deserialize;
use serde_json::json;
use std::path::Path;
use std::sync::MutexGuard;

#[derive(Deserialize)]
struct WireGuardFileRequest {
    config_file: String,
}

fn lock_manager() -> MutexGuard<'static, WireGuardManager> {
    wireguard_manager()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// WireGuard config isteğini doğrular ve config dosyasını üretir.
pub fn wireguard_config_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct WireGuardConfigRequest {
        config_file: String,
        private_key: Option<String>,
        public_key: Option<String>,
        endpoint: String,
        allowed_ips: Option<String>,
        address: Option<String>,
        dns: Option<String>,
        keepalive: Option<u16>,
    }

    let request: WireGuardConfigRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    if request.config_file.trim().is_empty() {
        return json_error(400, "config_file is required");
    }
    if request.endpoint.trim().is_empty() {
        return json_error(400, "endpoint is required");
    }
    let config = WireGuardConfig {
        private_key: request.private_key.as_deref().unwrap_or_default().trim(),
        public_key: request.public_key.as_deref().unwrap_or_default().trim(),
        endpoint: request.endpoint.trim(),
        allowed_ips: request.allowed_ips.as_deref().unwrap_or("0.0.0.0/0").trim(),
        address: request.address.as_deref().unwrap_or("10.0.0.2/24").trim(),
        dns: request.dns.as_deref().unwrap_or("1.1.1.1").trim(),
        keepalive: request.keepalive.unwrap_or(25),
    };

    let _ = record_active_profile_activity(
        "wireguard_config",
        request.config_file.trim(),
        None,
        Some(request.endpoint.trim()),
    );

    match wireguard::create_config(request.config_file.trim(), &config) {
        Ok(path) => json_ok(json!({ "path": path })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Seçilen WireGuard config dosyasıyla bağlantıyı başlatır.
pub fn wireguard_start_endpoint(body: &[u8]) -> Response {
    let request: WireGuardFileRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let config_file = request.config_file.trim();
    if config_file.is_empty() {
        return json_error(400, "config_file is required");
    }
    let config_path = Path::new(config_file);
    if !config_path.is_file() {
        return json_error(404, "wireguard config file not found");
    }

    let _ = record_active_profile_activity("wireguard_start", config_file, None, None);

    let mut guard = lock_manager();
    match guard.start(config_file) {
        Ok(()) => json_ok(json!({
            "active": guard.is_active(),
            "config_file": guard.config_file,
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Aktif WireGuard bağlantısını durdurur.
pub fn wireguard_stop_endpoint() -> Response {
    let mut guard = lock_manager();
    let interface_name = guard.interface_name.clone();

    let _ = record_active_profile_activity("wireguard_stop", &interface_name, None, None);

    match guard.stop() {
        Ok(()) => json_ok(json!({ "active": guard.is_active() })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// WireGuard manager'ın mevcut aktiflik durumunu döndürür.
pub fn wireguard_status_endpoint() -> Response {
    let guard = lock_manager();
    json_ok(json!({
        "interface_name": guard.interface_name,
        "config_file": guard.config_file,
        "active": guard.active,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wireguard_status_endpoint() {
        let response = wireguard_status_endpoint();
        assert_eq!(response.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&response.body).unwrap();
        assert_eq!(val["interface_name"], "wg0");
        assert!(val.get("active").is_some());
    }

    #[test]
    fn test_wireguard_config_endpoint_validation() {
        assert_eq!(wireguard_config_endpoint(b"").status, 400);
        assert_eq!(wireguard_config_endpoint(b"not json").status, 400);
        assert_eq!(
            wireguard_config_endpoint(br#"{"config_file":"","endpoint":"1.1.1.1:51820"}"#).status,
            400
        );
        assert_eq!(
            wireguard_config_endpoint(br#"{"config_file":"/tmp/wg.conf","endpoint":""}"#).status,
            400
        );
    }

    #[test]
    fn test_wireguard_config_endpoint_success() {
        let temp_dir = std::env::temp_dir().join(format!("amele_wg_test_{}", std::process::id()));
        let config_file = temp_dir.join("test_wg.conf");
        let req = json!({
            "config_file": config_file.to_str().unwrap(),
            "endpoint": "10.0.0.1:51820",
            "private_key": "dummy_private",
            "public_key": "dummy_public"
        })
        .to_string();

        let response = wireguard_config_endpoint(req.as_bytes());
        assert_eq!(response.status, 200);
        assert!(config_file.is_file());

        let content = fs::read_to_string(&config_file).unwrap();
        assert!(content.contains("[Interface]"));
        assert!(content.contains("[Peer]"));
        assert!(content.contains("10.0.0.1:51820"));

        let _ = fs::remove_file(&config_file);
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_wireguard_start_endpoint_validation() {
        assert_eq!(wireguard_start_endpoint(b"").status, 400);
        assert_eq!(wireguard_start_endpoint(b"not json").status, 400);
        assert_eq!(
            wireguard_start_endpoint(br#"{"config_file":""}"#).status,
            400
        );
        assert_eq!(
            wireguard_start_endpoint(br#"{"config_file":"/nonexistent/path/to/wg0.conf"}"#).status,
            404
        );
    }

    #[test]
    fn test_wireguard_stop_endpoint() {
        let response = wireguard_stop_endpoint();
        assert!(response.status == 200 || response.status == 500);
    }

    #[test]
    fn test_lock_manager_poison_resilience() {
        let _ = std::panic::catch_unwind(|| {
            let _guard = lock_manager();
            panic!("intentional test panic to poison mutex");
        });

        let guard = lock_manager();
        assert_eq!(guard.interface_name, "wg0");
    }
}
