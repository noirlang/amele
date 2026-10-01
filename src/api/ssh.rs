//! Uzak SSH adli edinim ve analiz API uç noktaları.

use std::path::PathBuf;
use std::thread;

use serde::Deserialize;
use serde_json::json;

use crate::evidence::EvidenceVault;
use crate::output_format::AcquisitionOutputFormat;
use crate::server::{Response, json_error, json_ok};
use crate::ssh::{SshConnection, SshConnectionParams};

use super::{
    create_acquisition_job, default_case_base_dir, fail_acquisition_job_with_message,
    finish_acquisition_job_with_message, sanitize_case_name, set_current_evidence_case,
    update_acquisition_message, update_acquisition_progress,
};

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct SshConnectRequest {
    pub ip: String,
    pub port: Option<u16>,
    pub user: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct SshImageRequest {
    pub ip: String,
    pub port: Option<u16>,
    pub user: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub disk_path: String,
    pub output: Option<String>,
    pub case_name: Option<String>,
    pub output_format: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct SshRamRequest {
    pub ip: String,
    pub port: Option<u16>,
    pub user: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
    pub output: Option<String>,
    pub case_name: Option<String>,
    pub output_format: Option<String>,
}

fn to_params(
    req_ip: String,
    port: Option<u16>,
    user: String,
    password: Option<String>,
    key_path: Option<String>,
) -> SshConnectionParams {
    SshConnectionParams {
        ip: req_ip,
        port: port.unwrap_or(22),
        user,
        password,
        key_path,
    }
}

/// SSH bağlantı testini ve kimlik doğrulamasını yapar.
pub fn ssh_connect_endpoint(body: &[u8]) -> Response {
    let req: SshConnectRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => return json_error(400, err.to_string()),
    };

    if req.ip.trim().is_empty() || req.user.trim().is_empty() {
        return json_error(400, "ip ve user alanları zorunludur");
    }

    let params = to_params(req.ip, req.port, req.user, req.password, req.key_path);
    match SshConnection::connect(&params) {
        Ok(mut conn) => {
            let uname = conn
                .exec_command("uname -a 2>/dev/null || echo 'Linux'")
                .unwrap_or_else(|_| "Linux".to_string());
            json_ok(json!({
                "connected": true,
                "host": conn.host,
                "port": conn.port,
                "user": conn.user,
                "system_info": uname.trim(),
            }))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

/// SSH üzerinden hedefteki diskleri listeler.
pub fn ssh_disks_endpoint(body: &[u8]) -> Response {
    let req: SshConnectRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => return json_error(400, err.to_string()),
    };

    if req.ip.trim().is_empty() || req.user.trim().is_empty() {
        return json_error(400, "ip ve user alanları zorunludur");
    }

    let params = to_params(req.ip, req.port, req.user, req.password, req.key_path);
    match SshConnection::connect(&params) {
        Ok(mut conn) => match conn.list_disks() {
            Ok(disks) => json_ok(json!({
                "connected": true,
                "disks": disks,
            })),
            Err(err) => json_error(500, err.to_string()),
        },
        Err(err) => json_error(500, err.to_string()),
    }
}

/// SSH üzerinden hedefteki RAM araçlarını kontrol eder.
pub fn ssh_tool_check_endpoint(body: &[u8]) -> Response {
    let req: SshConnectRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => return json_error(400, err.to_string()),
    };

    if req.ip.trim().is_empty() || req.user.trim().is_empty() {
        return json_error(400, "ip ve user alanları zorunludur");
    }

    let params = to_params(req.ip, req.port, req.user, req.password, req.key_path);
    match SshConnection::connect(&params) {
        Ok(mut conn) => match conn.check_ram_tools() {
            Ok(status) => json_ok(json!({
                "connected": true,
                "status": status,
            })),
            Err(err) => json_error(500, err.to_string()),
        },
        Err(err) => json_error(500, err.to_string()),
    }
}

/// SSH üzerinden disk imajı alma işini başlatır.
pub fn ssh_image_endpoint(body: &[u8]) -> Response {
    let req: SshImageRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => return json_error(400, err.to_string()),
    };

    if req.ip.trim().is_empty() || req.user.trim().is_empty() || req.disk_path.trim().is_empty() {
        return json_error(400, "ip, user ve disk_path alanları zorunludur");
    }

    let format = match AcquisitionOutputFormat::parse(req.output_format.as_deref()) {
        Ok(f) => f,
        Err(err) => return json_error(400, format!("Geçersiz çıktı formatı: {err}")),
    };

    let _ = crate::profile::record_active_profile_activity(
        "ssh",
        "disk_image",
        req.case_name.as_deref(),
        Some(&format!("ip={} disk={}", req.ip, req.disk_path)),
    );

    let (job_id, _control) = create_acquisition_job("SSH üzerinden disk imajı başlatıldı");
    let thread_job_id = job_id.clone();

    thread::spawn(move || {
        let params = to_params(req.ip, req.port, req.user, req.password, req.key_path);
        let mut conn = match SshConnection::connect(&params) {
            Ok(c) => c,
            Err(err) => {
                fail_acquisition_job_with_message(
                    &thread_job_id,
                    err.to_string(),
                    "SSH bağlantısı kurulamadı",
                );
                return;
            }
        };

        let out_dir =
            resolve_output_dir(req.output.as_deref(), req.case_name.as_deref(), "ciktilar");
        let job_id_clone = thread_job_id.clone();

        match conn.acquire_disk(
            &req.disk_path,
            &out_dir,
            req.case_name.as_deref(),
            format,
            move |done, _total| {
                update_acquisition_progress(&job_id_clone, done, 0);
                let msg = format!("SSH veri akışı: {done} bayt aktarıldı");
                update_acquisition_message(&job_id_clone, &msg);
            },
        ) {
            Ok(result) => {
                let base_dir = default_case_base_dir();
                if let Some(case) = req.case_name.as_deref() {
                    if let Ok(vault) = EvidenceVault::create(&base_dir, case) {
                        let _ = vault.write_case_manifest();
                    }
                }
                finish_acquisition_job_with_message(
                    &thread_job_id,
                    json!({
                        "target_path": result.target_path.display().to_string(),
                        "sha256": result.sha256,
                        "md5": result.md5,
                        "bytes_transferred": result.bytes_transferred,
                    }),
                    "SSH disk imajı başarıyla tamamlandı",
                );
            }
            Err(err) => {
                fail_acquisition_job_with_message(
                    &thread_job_id,
                    err.to_string(),
                    "SSH disk imajı başarısız oldu",
                );
            }
        }
    });

    json_ok(json!({
        "job_id": job_id,
        "status": "running"
    }))
}

/// SSH üzerinden RAM dökümü alma işini başlatır.
pub fn ssh_ram_endpoint(body: &[u8]) -> Response {
    let req: SshRamRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => return json_error(400, err.to_string()),
    };

    if req.ip.trim().is_empty() || req.user.trim().is_empty() {
        return json_error(400, "ip ve user alanları zorunludur");
    }

    let format = match AcquisitionOutputFormat::parse(req.output_format.as_deref()) {
        Ok(f) => f,
        Err(err) => return json_error(400, format!("Geçersiz çıktı formatı: {err}")),
    };

    let _ = crate::profile::record_active_profile_activity(
        "ssh",
        "ram_image",
        req.case_name.as_deref(),
        Some(&format!("ip={}", req.ip)),
    );

    let (job_id, _control) = create_acquisition_job("SSH üzerinden RAM dökümü başlatıldı");
    let thread_job_id = job_id.clone();

    thread::spawn(move || {
        let params = to_params(req.ip, req.port, req.user, req.password, req.key_path);
        let mut conn = match SshConnection::connect(&params) {
            Ok(c) => c,
            Err(err) => {
                fail_acquisition_job_with_message(
                    &thread_job_id,
                    err.to_string(),
                    "SSH bağlantısı kurulamadı",
                );
                return;
            }
        };

        let out_dir = resolve_output_dir(req.output.as_deref(), req.case_name.as_deref(), "ram");
        let job_id_clone = thread_job_id.clone();

        match conn.acquire_ram(
            &out_dir,
            req.case_name.as_deref(),
            format,
            move |done, _total| {
                update_acquisition_progress(&job_id_clone, done, 0);
                let msg = format!("SSH RAM akışı: {done} bayt aktarıldı");
                update_acquisition_message(&job_id_clone, &msg);
            },
        ) {
            Ok(result) => {
                let base_dir = default_case_base_dir();
                if let Some(case) = req.case_name.as_deref() {
                    if let Ok(vault) = EvidenceVault::create(&base_dir, case) {
                        let _ = vault.write_case_manifest();
                    }
                }
                finish_acquisition_job_with_message(
                    &thread_job_id,
                    json!({
                        "target_path": result.target_path.display().to_string(),
                        "sha256": result.sha256,
                        "md5": result.md5,
                        "bytes_transferred": result.bytes_transferred,
                    }),
                    "SSH RAM dökümü başarıyla tamamlandı",
                );
            }
            Err(err) => {
                fail_acquisition_job_with_message(
                    &thread_job_id,
                    err.to_string(),
                    "SSH RAM dökümü başarısız oldu",
                );
            }
        }
    });

    json_ok(json!({
        "job_id": job_id,
        "status": "running"
    }))
}

fn resolve_output_dir(
    explicit_output: Option<&str>,
    case_name: Option<&str>,
    subdir: &str,
) -> PathBuf {
    if let Some(name) = case_name.map(str::trim).filter(|s| !s.is_empty()) {
        let safe_name = sanitize_case_name(name);
        let base = default_case_base_dir();
        let case_dir = base.join(&safe_name);
        set_current_evidence_case(base, safe_name);
        let target = case_dir.join(subdir);
        let _ = std::fs::create_dir_all(&target);
        return target;
    }
    if let Some(out) = explicit_output.map(str::trim).filter(|s| !s.is_empty()) {
        let path = PathBuf::from(out);
        if path.is_dir() {
            return path;
        }
        if path.extension().is_none() {
            let _ = std::fs::create_dir_all(&path);
            if path.is_dir() {
                return path;
            }
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                return parent.to_path_buf();
            }
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_params() {
        let params = to_params(
            "192.168.1.100".to_string(),
            None,
            "root".to_string(),
            Some("secret".to_string()),
            None,
        );
        assert_eq!(params.ip, "192.168.1.100");
        assert_eq!(params.port, 22);
        assert_eq!(params.user, "root");
        assert_eq!(params.password.as_deref(), Some("secret"));
        assert!(params.key_path.is_none());

        let params2 = to_params(
            "10.0.0.1".to_string(),
            Some(2222),
            "admin".to_string(),
            None,
            Some("/id_rsa".to_string()),
        );
        assert_eq!(params2.port, 2222);
        assert_eq!(params2.key_path.as_deref(), Some("/id_rsa"));
    }

    #[test]
    fn test_ssh_connect_endpoint_validation() {
        assert_eq!(ssh_connect_endpoint(b"").status, 400);
        assert_eq!(ssh_connect_endpoint(b"invalid json").status, 400);
        assert_eq!(
            ssh_connect_endpoint(br#"{"ip":"","user":"root"}"#).status,
            400
        );
        assert_eq!(
            ssh_connect_endpoint(br#"{"ip":"127.0.0.1","user":"   "}"#).status,
            400
        );
    }

    #[test]
    fn test_ssh_disks_endpoint_validation() {
        assert_eq!(ssh_disks_endpoint(b"").status, 400);
        assert_eq!(ssh_disks_endpoint(b"not json").status, 400);
        assert_eq!(
            ssh_disks_endpoint(br#"{"ip":"","user":"root"}"#).status,
            400
        );
        assert_eq!(
            ssh_disks_endpoint(br#"{"ip":"127.0.0.1","user":""}"#).status,
            400
        );
    }

    #[test]
    fn test_ssh_tool_check_endpoint_validation() {
        assert_eq!(ssh_tool_check_endpoint(b"").status, 400);
        assert_eq!(ssh_tool_check_endpoint(b"not json").status, 400);
        assert_eq!(
            ssh_tool_check_endpoint(br#"{"ip":"","user":"root"}"#).status,
            400
        );
        assert_eq!(
            ssh_tool_check_endpoint(br#"{"ip":"127.0.0.1","user":""}"#).status,
            400
        );
    }

    #[test]
    fn test_ssh_image_endpoint_validation() {
        assert_eq!(ssh_image_endpoint(b"").status, 400);
        assert_eq!(ssh_image_endpoint(b"not json").status, 400);
        assert_eq!(
            ssh_image_endpoint(br#"{"ip":"","user":"root","disk_path":"/dev/sda"}"#).status,
            400
        );
        assert_eq!(
            ssh_image_endpoint(br#"{"ip":"10.0.0.1","user":"","disk_path":"/dev/sda"}"#).status,
            400
        );
        assert_eq!(
            ssh_image_endpoint(br#"{"ip":"10.0.0.1","user":"root","disk_path":""}"#).status,
            400
        );
        let resp = ssh_image_endpoint(br#"{"ip":"10.0.0.1","user":"root","disk_path":"/dev/sda","output_format":"invalid_xyz"}"#);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_ssh_ram_endpoint_validation() {
        assert_eq!(ssh_ram_endpoint(b"").status, 400);
        assert_eq!(ssh_ram_endpoint(b"not json").status, 400);
        assert_eq!(ssh_ram_endpoint(br#"{"ip":"","user":"root"}"#).status, 400);
        assert_eq!(
            ssh_ram_endpoint(br#"{"ip":"10.0.0.1","user":""}"#).status,
            400
        );
        let resp =
            ssh_ram_endpoint(br#"{"ip":"10.0.0.1","user":"root","output_format":"invalid_xyz"}"#);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_resolve_output_dir() {
        let temp = std::env::temp_dir();
        let target_dir = temp.join("amele_ssh_test_dir");
        let dir = resolve_output_dir(Some(target_dir.to_str().unwrap()), None, "ciktilar");
        assert_eq!(dir, target_dir);
        let _ = std::fs::remove_dir_all(&target_dir);

        let fallback = resolve_output_dir(None, None, "ciktilar");
        assert!(fallback.is_dir() || fallback.as_os_str() == ".");
    }
}
