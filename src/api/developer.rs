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
/// `since_seq` verilirse yalnızca o sıra numarasından sonraki yeni log satırları döndürülür.
pub fn developer_logs_endpoint(since_seq: Option<u64>) -> Response {
    let logs = match since_seq {
        Some(seq) if seq > 0 => crate::logging::runtime_logs_since(seq, 200),
        _ => crate::logging::runtime_logs(200),
    };
    json_ok(json!({
        "logs": logs,
        "log_file": crate::logging::runtime_log_file_path(),
        "jobs": developer_job_snapshot(),
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
    fn test_developer_logs_endpoint_returns_json() {
        let resp = developer_logs_endpoint(None);
        assert_eq!(resp.status, 200);
        let val: Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val.get("logs").is_some());
        assert!(val.get("jobs").is_some());

        let resp_since = developer_logs_endpoint(Some(999999));
        assert_eq!(resp_since.status, 200);
        let val_since: Value = serde_json::from_slice(&resp_since.body).expect("valid json");
        assert_eq!(val_since.get("logs").and_then(|l| l.as_array()).map(|a| a.len()), Some(0));
    }
}
