use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

pub static NEXT_ACQUISITION_JOB_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct AcquisitionJob {
    pub status: String,
    pub done: u64,
    pub total: u64,
    pub message: String,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub control: crate::ram::CancellationToken,
}

pub fn acquisition_jobs() -> &'static Mutex<HashMap<String, AcquisitionJob>> {
    static ACQUISITION_JOBS: OnceLock<Mutex<HashMap<String, AcquisitionJob>>> = OnceLock::new();
    ACQUISITION_JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn create_acquisition_job(message: &str) -> (String, crate::ram::CancellationToken) {
    let id = NEXT_ACQUISITION_JOB_ID.fetch_add(1, Ordering::SeqCst);
    let job_id = format!("acq-{id}");
    let control = crate::ram::CancellationToken::default();
    let job = AcquisitionJob {
        status: "running".to_string(),
        done: 0,
        total: 0,
        message: message.to_string(),
        result: None,
        error: None,
        control: control.clone(),
    };
    if let Ok(mut jobs) = acquisition_jobs().lock() {
        jobs.insert(job_id.clone(), job);
    }
    (job_id, control)
}

pub fn update_acquisition_progress(job_id: &str, done: u64, total: u64) {
    update_acquisition_progress_message(job_id, done, total, "Imaj alma sürüyor");
}

pub fn update_acquisition_progress_message(job_id: &str, done: u64, total: u64, label: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.status = "running".to_string();
        job.done = done;
        job.total = total;
        job.message = if total > 0 {
            format!("{label}: {}%", progress_percent(done, total))
        } else {
            label.to_string()
        };
    }
}

pub fn update_acquisition_message(job_id: &str, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.message = message.to_string();
    }
}

pub fn finish_acquisition_job_with_message(job_id: &str, result: Value, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.status = "completed".to_string();
        if job.total == 0 {
            job.total = 1;
            job.done = 1;
        } else {
            job.done = job.total;
        }
        job.message = message.to_string();
        job.result = Some(result);
        job.error = None;
    }
}

pub fn fail_acquisition_job_with_message(job_id: &str, error: String, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.status = "failed".to_string();
        job.message = message.to_string();
        job.error = Some(error);
    }
}

fn progress_percent(done: u64, total: u64) -> u64 {
    if total == 0 { 0 } else { done * 100 / total }
}
