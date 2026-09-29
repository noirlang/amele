//! geliştirici konsolu canlı log akışı ve durum api rotası.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::server::{Response, json_error, json_ok};

use super::acquisition_jobs;

#[derive(Deserialize)]
/// UI veya frontend tarafının developer log'a eklemek istediği satırı taşır.
struct DeveloperLogRequest {
    level: Option<String>,
    scope: Option<String>,
    message: String,
}

/// Developer mod penceresinin okuyacağı runtime log ve iş özetini döndürür.
pub fn developer_logs_endpoint() -> Response {
    json_ok(json!({
        "logs": crate::logging::runtime_logs(1000),
        "log_file": crate::logging::runtime_log_file_path(),
        "jobs": developer_job_snapshot(),
    }))
}

/// download.amele.noirlang.tr üzerinden developers.json çeker.
pub fn developers_endpoint() -> Response {
    let url = "https://download.amele.noirlang.tr/developers.json";
    let bust_url = format!("{url}?t={}", chrono::Utc::now().timestamp());
    let output = std::process::Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--silent")
        .arg("--show-error")
        .arg("--max-time")
        .arg("10")
        .arg(&bust_url)
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            if let Ok(val) = serde_json::from_slice::<Value>(&out.stdout) {
                return json_ok(val);
            }
        }
    }

    // Fallback varsayılan geliştirici
    json_ok(json!({
        "developers": [
            {
                "id": "melih-emik",
                "name": "Melih Emik",
                "role": "BDFL & Maintainer",
                "avatar_url": "https://amele.noirlang.tr/contributors/melih-emik.webp",
                "website": "https://melihemik.com.tr",
                "github": "https://github.com/melihemik",
                "linkedin": "https://linkedin.com/in/melihemik"
            }
        ]
    }))
}

/// Frontend tarafındaki hata ve kritik olayları backend runtime log'una işler.
pub fn developer_log_endpoint(body: &[u8]) -> Response {
    let request: DeveloperLogRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let level = match request
        .level
        .as_deref()
        .unwrap_or("info")
        .to_ascii_lowercase()
        .as_str()
    {
        "error" => crate::logging::LogLevel::Error,
        "warn" | "warning" => crate::logging::LogLevel::Warn,
        "debug" => crate::logging::LogLevel::Debug,
        _ => crate::logging::LogLevel::Info,
    };
    let scope = request.scope.as_deref().unwrap_or("ui");
    crate::logging::runtime_log(level, scope, request.message);
    json_ok(json!({ "ok": true }))
}

/// Developer paneli için aktif/son işlerin sade özetini üretir.
fn developer_job_snapshot() -> Vec<Value> {
    acquisition_jobs()
        .lock()
        .map(|jobs| {
            jobs.iter()
                .map(|(id, job)| {
                    json!({
                        "id": id,
                        "status": job.status,
                        "done": job.done,
                        "total": job.total,
                        "message": job.message,
                        "log_count": job.logs.len(),
                        "error": job.error,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_developers_endpoint_returns_json() {
        let resp = developers_endpoint();
        assert_eq!(resp.status, 200);
        let val: Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val.get("developers").is_some() || val.is_array());
    }

    #[test]
    fn test_developer_logs_endpoint_returns_json() {
        let resp = developer_logs_endpoint();
        assert_eq!(resp.status, 200);
        let val: Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val.get("logs").is_some());
        assert!(val.get("jobs").is_some());
    }
}
