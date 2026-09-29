//! disk ve ram kopyalama başlatma durdurma api uç noktası.

use serde::Deserialize;
use serde_json::{Value, json};

use crate::disk;
use crate::remote::RemoteConnection;
use crate::server::{Response, json_error, json_ok};

use super::{AcquisitionJob, acquisition_jobs};

/// Tek bir edinim işinin durumunu UI için JSON nesnesine formatlar.
fn format_job_json(job_id: &str, job: &AcquisitionJob) -> Value {
    json!({
        "job_id": job_id,
        "status": job.status,
        "done": job.done,
        "total": job.total,
        "message": job.message,
        "logs": job.logs,
        "result": job.result,
        "error": job.error,
        "phase": job.phase,
    })
}

/// Edinim işinin canlı durumunu UI'ye döndürür.
pub fn acquisition_status_endpoint(body: &[u8]) -> Response {
    let jobs = match acquisition_jobs().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

    if body.is_empty() {
        let mut map = serde_json::Map::new();
        for (id, job) in jobs.iter() {
            map.insert(id.clone(), format_job_json(id, job));
        }
        return json_ok(json!({ "jobs": map }));
    }

    #[derive(Deserialize)]
    struct StatusRequest {
        job_id: String,
    }

    let request: StatusRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    match jobs.get(&request.job_id) {
        Some(job) => json_ok(format_job_json(&request.job_id, job)),
        None => json_error(404, "acquisition job not found"),
    }
}

/// Pause/resume/stop komutlarını ilgili edinim işine uygular.
pub fn acquisition_control_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct ControlRequest {
        ip: Option<String>,
        port: Option<u16>,
        token: Option<String>,
        job_id: String,
        action: String,
    }

    let request: ControlRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    if request.job_id.trim().is_empty() {
        return json_error(400, "job_id is required");
    }
    if !matches!(request.action.as_str(), "pause" | "resume" | "stop") {
        return json_error(400, "action must be pause, resume, or stop");
    }

    let ip = request.ip.unwrap_or_default();
    let remote_control = !ip.trim().is_empty();

    if remote_control {
        let port = request.port.unwrap_or_default();
        if port == 0 {
            return json_error(400, "port is required");
        }
        match RemoteConnection::connect(&ip, port, request.token) {
            Ok(connection) => match connection.control_job(&request.job_id, &request.action) {
                Ok(message) => {
                    apply_local_acquisition_control(&request.job_id, &request.action);
                    json_ok(json!({
                        "job_id": request.job_id,
                        "action": request.action,
                        "message": message,
                    }))
                }
                Err(err) => json_error(500, err.to_string()),
            },
            Err(err) => json_error(500, err.to_string()),
        }
    } else {
        match apply_local_acquisition_control(&request.job_id, &request.action) {
            Some(message) => json_ok(json!({
                "job_id": request.job_id,
                "action": request.action,
                "message": message,
            })),
            None => json_error(404, "acquisition job not found"),
        }
    }
}

/// Yerel edinim işine pause/resume/stop kontrolü uygular.
fn apply_local_acquisition_control(job_id: &str, action: &str) -> Option<String> {
    let mut jobs = acquisition_jobs().lock().ok()?;
    let job = jobs.get_mut(job_id)?;
    let msg = match action {
        "pause" => {
            job.control.pause();
            job.status = "paused".to_string();
            "Duraklatma komutu uygulandı"
        }
        "resume" => {
            job.control.resume();
            job.status = "running".to_string();
            "Devam komutu uygulandı"
        }
        "stop" => {
            job.control.cancel();
            // Disk operasyonlarında thread-level atomic bayrak kontrol edildiği için iptal tetiklenir.
            disk::cancel_disk_acquisition();
            "Durdurma komutu uygulandı"
        }
        _ => return None,
    };
    job.message = msg.to_string();
    Some(msg.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::create_acquisition_job;

    #[test]
    fn test_acquisition_status_empty_body() {
        let resp = acquisition_status_endpoint(&[]);
        assert_eq!(resp.status, 200);
        let val: Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val.get("jobs").is_some());
    }

    #[test]
    fn test_acquisition_status_specific_job() {
        let (job_id, _ctrl) = create_acquisition_job("Test job for status");
        let body = serde_json::to_vec(&json!({ "job_id": job_id })).unwrap();
        let resp = acquisition_status_endpoint(&body);
        assert_eq!(resp.status, 200);
        let val: Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert_eq!(val.get("job_id").and_then(|v| v.as_str()), Some(job_id.as_str()));
    }

    #[test]
    fn test_acquisition_status_not_found() {
        let body = serde_json::to_vec(&json!({ "job_id": "non-existent-job-id-999" })).unwrap();
        let resp = acquisition_status_endpoint(&body);
        assert_eq!(resp.status, 404);
    }

    #[test]
    fn test_acquisition_control_actions() {
        let (job_id, _ctrl) = create_acquisition_job("Test job for control");

        // Test pause
        let body_pause = serde_json::to_vec(&json!({ "job_id": job_id, "action": "pause" })).unwrap();
        let resp_pause = acquisition_control_endpoint(&body_pause);
        assert_eq!(resp_pause.status, 200);

        // Test resume
        let body_resume = serde_json::to_vec(&json!({ "job_id": job_id, "action": "resume" })).unwrap();
        let resp_resume = acquisition_control_endpoint(&body_resume);
        assert_eq!(resp_resume.status, 200);

        // Test stop
        let body_stop = serde_json::to_vec(&json!({ "job_id": job_id, "action": "stop" })).unwrap();
        let resp_stop = acquisition_control_endpoint(&body_stop);
        assert_eq!(resp_stop.status, 200);

        // Test invalid action
        let body_invalid = serde_json::to_vec(&json!({ "job_id": job_id, "action": "invalid" })).unwrap();
        let resp_invalid = acquisition_control_endpoint(&body_invalid);
        assert_eq!(resp_invalid.status, 400);
    }
}
