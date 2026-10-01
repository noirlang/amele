//! docker konteyner ve imaj kopyalama api rotaları.

use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use std::thread;

use crate::docker::{
    self, DockerAcquisitionRequest, check_docker_status, get_container_logs, list_containers,
};
use crate::remote::RemoteConnection;
use crate::server::{Response, json_error, json_ok};

use super::{
    append_acquisition_log, create_acquisition_job, evidence_vault_for_output,
    fail_acquisition_job_with_message, finish_acquisition_job_with_message,
    update_acquisition_progress_message,
};

#[derive(Deserialize)]
/// Konteyner detay veya log talebi için gelen gövdeyi taşır.
pub struct ContainerTargetRequest {
    pub container_id: String,
    pub custom_docker_root: Option<String>,
    pub tail: Option<usize>,
}

#[derive(Deserialize)]
/// Uzak Docker bağlantı talebini taşır.
pub struct RemoteDockerRequest {
    pub ip: String,
    pub port: u16,
    pub token: Option<String>,
    pub container_id: Option<String>,
    pub tail: Option<usize>,
}

#[derive(Deserialize)]
/// Uzak Docker edinim talebini taşır.
pub struct RemoteDockerAcquisitionRequest {
    pub ip: String,
    pub port: u16,
    pub token: Option<String>,
    pub container_id: String,
    pub container_name: Option<String>,
    pub acquire_diff: Option<bool>,
    pub acquire_logs: Option<bool>,
    pub acquire_config: Option<bool>,
    pub case_name: Option<String>,
}

fn parse_docker_root_path(custom_root: Option<&str>) -> Option<&Path> {
    custom_root
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(Path::new)
}

pub(crate) fn container_short_id(id: &str) -> &str {
    let trimmed = id.trim();
    if trimmed.len() <= 12 {
        return trimmed;
    }
    match trimmed.char_indices().nth(12) {
        Some((idx, _)) => &trimmed[..idx],
        None => trimmed,
    }
}

/// Yerel veya bağlanmış imajdaki Docker sistem durumunu döner.
pub fn docker_status_endpoint(custom_root: Option<&str>) -> Response {
    let path_opt = parse_docker_root_path(custom_root);
    let status = check_docker_status(path_opt);
    json_ok(json!({
        "durum": "ok",
        "status": status,
    }))
}

/// Yerel veya bağlanmış imajdaki Docker konteynerlerini listeler ve güvenlik analizini döner.
pub fn docker_containers_endpoint(custom_root: Option<&str>) -> Response {
    let path_opt = parse_docker_root_path(custom_root);
    match list_containers(path_opt) {
        Ok(containers) => json_ok(json!({
            "durum": "ok",
            "containers": containers,
            "toplam": containers.len(),
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Belirli bir konteynerin loglarını okur ve döner.
pub fn docker_logs_endpoint(body: &[u8]) -> Response {
    let req: ContainerTargetRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(err) => return json_error(400, format!("Gecersiz istek JSON: {}", err)),
    };

    if req.container_id.trim().is_empty() {
        return json_error(400, "container_id parametresi gereklidir.");
    }

    let path_opt = parse_docker_root_path(req.custom_docker_root.as_deref());
    let tail = req.tail.unwrap_or(200);

    match get_container_logs(&req.container_id, tail, path_opt) {
        Ok(logs) => json_ok(json!({
            "durum": "ok",
            "container_id": req.container_id,
            "logs": logs,
            "toplam": logs.len(),
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Yerel konteyner delillerini vaka klasörüne toplar (Arka planda iş başlatır).
pub fn docker_acquire_local_endpoint(body: &[u8]) -> Response {
    let mut req: DockerAcquisitionRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(err) => return json_error(400, format!("Gecersiz istek JSON: {}", err)),
    };

    if req.container_id.trim().is_empty() {
        return json_error(400, "container_id parametresi gereklidir.");
    }

    let vault = match evidence_vault_for_output(req.case_name.as_deref()) {
        Ok(v) => v,
        Err(err) => return json_error(500, format!("Vaka olusturulamadi: {}", err)),
    };

    let base_dir = vault
        .case_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| vault.case_dir.clone());
    req.case_name = Some(vault.case_name.clone());

    let _ = crate::profile::record_active_profile_activity(
        "docker",
        "local_acquisition",
        Some(&vault.case_name),
        Some(&format!("Konteyner: {}", req.container_id)),
    );

    let short_id = container_short_id(&req.container_id);
    let (job_id, _control) = create_acquisition_job(&format!("Docker: {}", short_id));

    let job_id_clone = job_id.clone();
    let container_id = req.container_id.clone();

    thread::spawn(move || {
        append_acquisition_log(
            &job_id_clone,
            &format!("Yerel Docker edinimi başlatıldı: {}", container_id),
        );

        let res = docker::acquire_container_evidence(&req, &base_dir, |msg, done, total| {
            update_acquisition_progress_message(&job_id_clone, done, total, msg);
            append_acquisition_log(&job_id_clone, msg);
        });

        match res {
            Ok(result) => {
                append_acquisition_log(
                    &job_id_clone,
                    &format!(
                        "Docker edinimi tamamlandı. Vaka yolu: {} (SHA256: {})",
                        result.case_path,
                        result.diff_sha256.as_deref().unwrap_or("-")
                    ),
                );
                let result_json = serde_json::to_value(&result).unwrap_or(Value::Null);
                finish_acquisition_job_with_message(
                    &job_id_clone,
                    result_json,
                    &format!("Konteyner delilleri toplandı: {}", result.container_name),
                );
            }
            Err(err) => {
                append_acquisition_log(&job_id_clone, &format!("Docker edinim hatası: {}", err));
                fail_acquisition_job_with_message(
                    &job_id_clone,
                    err.to_string(),
                    "Docker edinim hatası",
                );
            }
        }
    });

    json_ok(json!({
        "durum": "ok",
        "is_id": job_id,
        "mesaj": "Docker edinim işi başlatıldı.",
    }))
}

/// Uzak agent'tan Docker durumunu çeker.
pub fn docker_remote_status_endpoint(body: &[u8]) -> Response {
    let req: RemoteDockerRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(err) => return json_error(400, format!("Gecersiz istek JSON: {}", err)),
    };

    if req.ip.trim().is_empty() || req.port == 0 {
        return json_error(400, "Gecersiz IP adresi veya port.");
    }

    match RemoteConnection::connect(&req.ip, req.port, req.token) {
        Ok(mut conn) => match conn.docker_status() {
            Ok(status) => json_ok(json!({
                "durum": "ok",
                "status": status,
            })),
            Err(err) => json_error(500, err.to_string()),
        },
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Uzak agent'taki Docker konteynerlerini listeler.
pub fn docker_remote_containers_endpoint(body: &[u8]) -> Response {
    let req: RemoteDockerRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(err) => return json_error(400, format!("Gecersiz istek JSON: {}", err)),
    };

    if req.ip.trim().is_empty() || req.port == 0 {
        return json_error(400, "Gecersiz IP adresi veya port.");
    }

    match RemoteConnection::connect(&req.ip, req.port, req.token) {
        Ok(mut conn) => match conn.list_docker_containers() {
            Ok(containers) => json_ok(json!({
                "durum": "ok",
                "containers": containers,
                "toplam": containers.len(),
            })),
            Err(err) => json_error(500, err.to_string()),
        },
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Uzak agent'taki belirli bir konteynerin loglarını çeker.
pub fn docker_remote_logs_endpoint(body: &[u8]) -> Response {
    let req: RemoteDockerRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(err) => return json_error(400, format!("Gecersiz istek JSON: {}", err)),
    };

    if req.ip.trim().is_empty() || req.port == 0 {
        return json_error(400, "Gecersiz IP adresi veya port.");
    }

    let container_id = match req.container_id {
        Some(ref id) if !id.trim().is_empty() => id.clone(),
        _ => return json_error(400, "container_id parametresi gereklidir."),
    };

    match RemoteConnection::connect(&req.ip, req.port, req.token) {
        Ok(mut conn) => {
            match conn.get_docker_container_logs(&container_id, req.tail.unwrap_or(200)) {
                Ok(logs) => json_ok(json!({
                    "durum": "ok",
                    "container_id": container_id,
                    "logs": logs,
                    "toplam": logs.len(),
                })),
                Err(err) => json_error(500, err.to_string()),
            }
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Uzak agent üzerindeki konteyner delillerini yerel vaka klasörüne indirir.
pub fn docker_remote_acquire_endpoint(body: &[u8]) -> Response {
    let req: RemoteDockerAcquisitionRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(err) => return json_error(400, format!("Gecersiz istek JSON: {}", err)),
    };

    if req.ip.trim().is_empty() || req.port == 0 {
        return json_error(400, "Gecersiz IP adresi veya port.");
    }
    if req.container_id.trim().is_empty() {
        return json_error(400, "container_id parametresi gereklidir.");
    }

    let vault = match evidence_vault_for_output(req.case_name.as_deref()) {
        Ok(v) => v,
        Err(err) => return json_error(500, format!("Vaka olusturulamadi: {}", err)),
    };

    let _ = crate::profile::record_active_profile_activity(
        "docker",
        "remote_acquisition",
        Some(&vault.case_name),
        Some(&format!(
            "Konteyner: {} ({}:{})",
            req.container_id, req.ip, req.port
        )),
    );

    let short_id = container_short_id(&req.container_id);
    let name = req.container_name.as_deref().unwrap_or("container");
    let target_dir = vault.docker_dir.join(format!("{}_{}", name, short_id));

    let target_tar_path = target_dir.join("docker_evidence.tar.gz");

    let (job_id, _control) =
        create_acquisition_job(&format!("Uzak Docker: {} ({})", name, short_id));

    let job_id_clone = job_id.clone();
    let container_id = req.container_id.clone();

    thread::spawn(move || {
        append_acquisition_log(
            &job_id_clone,
            &format!(
                "Uzak Docker edinimi başlatıldı: {} ({}:{})",
                container_id, req.ip, req.port
            ),
        );

        let mut conn = match RemoteConnection::connect(&req.ip, req.port, req.token) {
            Ok(c) => c,
            Err(err) => {
                fail_acquisition_job_with_message(
                    &job_id_clone,
                    err.to_string(),
                    "Agent bağlantı hatası",
                );
                return;
            }
        };

        let acquire_diff = req.acquire_diff.unwrap_or(true);
        let acquire_logs = req.acquire_logs.unwrap_or(true);
        let acquire_config = req.acquire_config.unwrap_or(true);

        let job_ref = job_id_clone.clone();
        let res = conn.acquire_remote_docker(
            &container_id,
            acquire_diff,
            acquire_logs,
            acquire_config,
            &target_tar_path,
            Some(&job_id_clone),
            move |done, total| {
                update_acquisition_progress_message(
                    &job_ref,
                    done,
                    total,
                    "Uzak delil paketi aktarılıyor",
                );
            },
        );

        match res {
            Ok(result) => {
                let _ = std::fs::create_dir_all(&target_dir);
                let manifest_path = target_dir.join("manifest.csv");
                let manifest_content = format!(
                    "Dosya_Adi,Boyut_Byte,SHA256\ndocker_evidence.tar.gz,{},{}\n",
                    result.bytes_transferred,
                    result.sha256.as_deref().unwrap_or("HATA")
                );
                let _ = std::fs::write(&manifest_path, manifest_content);

                append_acquisition_log(
                    &job_id_clone,
                    &format!(
                        "Uzak Docker edinimi başarıyla tamamlandı. Dosya: {} (SHA256: {})",
                        result.target_path.display(),
                        result.sha256.as_deref().unwrap_or("-")
                    ),
                );
                let result_json = serde_json::to_value(&result).unwrap_or(Value::Null);
                finish_acquisition_job_with_message(
                    &job_id_clone,
                    result_json,
                    &format!(
                        "Uzak Docker delili aktarıldı ({} bytes)",
                        result.bytes_transferred
                    ),
                );
            }
            Err(err) => {
                append_acquisition_log(
                    &job_id_clone,
                    &format!("Uzak Docker edinim hatası: {}", err),
                );
                fail_acquisition_job_with_message(
                    &job_id_clone,
                    err.to_string(),
                    "Uzak Docker edinim hatası",
                );
            }
        }
    });

    json_ok(json!({
        "durum": "ok",
        "is_id": job_id,
        "mesaj": "Uzak Docker edinim işi başlatıldı.",
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_short_id() {
        assert_eq!(container_short_id("123456789012345678"), "123456789012");
        assert_eq!(container_short_id("12345"), "12345");
        assert_eq!(container_short_id("   abc   "), "abc");
        assert_eq!(container_short_id(""), "");
        assert_eq!(container_short_id("türkçekarakterler12345"), "türkçekarakt");
    }

    #[test]
    fn test_parse_docker_root_path() {
        assert!(parse_docker_root_path(None).is_none());
        assert!(parse_docker_root_path(Some("")).is_none());
        assert!(parse_docker_root_path(Some("   ")).is_none());
        assert_eq!(
            parse_docker_root_path(Some("/var/lib/docker")),
            Some(Path::new("/var/lib/docker"))
        );
    }

    #[test]
    fn test_docker_status_endpoint() {
        let resp = docker_status_endpoint(Some("/tmp/nonexistent_docker_dir_12345"));
        assert_eq!(resp.status, 200);
        let val: Value = serde_json::from_slice(&resp.body).expect("JSON expected");
        assert_eq!(val["durum"], "ok");
        assert_eq!(val["status"]["docker_available"], false);
    }

    #[test]
    fn test_docker_containers_endpoint() {
        let resp = docker_containers_endpoint(Some("/tmp/nonexistent_docker_dir_12345"));
        assert_eq!(resp.status, 200);
        let val: Value = serde_json::from_slice(&resp.body).expect("JSON expected");
        assert_eq!(val["durum"], "ok");
        assert_eq!(val["toplam"], 0);
    }

    #[test]
    fn test_docker_logs_endpoint_validation() {
        let resp = docker_logs_endpoint(b"not json");
        assert_eq!(resp.status, 400);

        let resp = docker_logs_endpoint(br#"{"container_id": ""}"#);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_docker_acquire_local_endpoint_validation() {
        let resp = docker_acquire_local_endpoint(b"invalid");
        assert_eq!(resp.status, 400);

        let resp = docker_acquire_local_endpoint(
            br#"{"container_id": "", "acquire_diff": false, "acquire_logs": false, "acquire_config": false}"#,
        );
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_docker_remote_endpoints_validation() {
        let resp = docker_remote_status_endpoint(b"invalid");
        assert_eq!(resp.status, 400);

        let resp = docker_remote_status_endpoint(br#"{"ip": "", "port": 0}"#);
        assert_eq!(resp.status, 400);

        let resp = docker_remote_containers_endpoint(br#"{"ip": "127.0.0.1", "port": 0}"#);
        assert_eq!(resp.status, 400);

        let resp = docker_remote_logs_endpoint(br#"{"ip": "127.0.0.1", "port": 8080}"#);
        assert_eq!(resp.status, 400);

        let resp = docker_remote_logs_endpoint(
            br#"{"ip": "127.0.0.1", "port": 8080, "container_id": "   "}"#,
        );
        assert_eq!(resp.status, 400);

        let resp = docker_remote_acquire_endpoint(
            br#"{"ip": "127.0.0.1", "port": 8080, "container_id": ""}"#,
        );
        assert_eq!(resp.status, 400);

        let resp = docker_remote_acquire_endpoint(
            br#"{"ip": "127.0.0.1", "port": 0, "container_id": "test"}"#,
        );
        assert_eq!(resp.status, 400);
    }
}
