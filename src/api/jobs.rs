//! arkadaki işlerin durumunu ve hızını dönen api rotası.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

pub static NEXT_ACQUISITION_JOB_ID: AtomicU64 = AtomicU64::new(1);

/// Tek bir edinim işinin UI'ye dönen canlı durumunu temsil eder.
#[derive(Clone)]
pub struct AcquisitionJob {
    pub status: String,
    pub done: u64,
    pub total: u64,
    pub message: String,
    pub logs: Vec<String>,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub phase: Option<String>,
    pub control: crate::ram::CancellationToken,
}

/// Global edinim iş haritasını tek sefer oluşturup paylaşır.
pub fn acquisition_jobs() -> &'static Mutex<HashMap<String, AcquisitionJob>> {
    static ACQUISITION_JOBS: OnceLock<Mutex<HashMap<String, AcquisitionJob>>> = OnceLock::new();
    ACQUISITION_JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Yeni bir arka plan edinim işi oluşturur ve kontrol token'ını döndürür.
pub fn create_acquisition_job(message: &str) -> (String, crate::ram::CancellationToken) {
    let id = NEXT_ACQUISITION_JOB_ID.fetch_add(1, Ordering::SeqCst);
    let job_id = format!("acq-{id}");
    let control = crate::ram::CancellationToken::default();
    let job = AcquisitionJob {
        status: "running".to_string(),
        done: 0,
        total: 0,
        message: message.to_string(),
        logs: vec![message.to_string()],
        result: None,
        error: None,
        phase: None,
        control: control.clone(),
    };
    if let Ok(mut jobs) = acquisition_jobs().lock() {
        jobs.insert(job_id.clone(), job);
    }
    crate::logging::runtime_log(
        crate::logging::LogLevel::Info,
        "job",
        format!("{job_id} baslatildi: {message}"),
    );
    (job_id, control)
}

/// İşin ilerlemesini varsayılan "imaj alma sürüyor" mesajıyla günceller.
pub fn update_acquisition_progress(job_id: &str, done: u64, total: u64) {
    update_acquisition_progress_message(job_id, done, total, "Imaj alma sürüyor");
}

/// İşin ilerlemesini özel mesajla günceller.
pub fn update_acquisition_progress_message(job_id: &str, done: u64, total: u64, label: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        let effective_total = total.max(done);
        let effective_done = done.min(effective_total);
        let previous_percent = progress_percent(job.done, job.total);
        job.status = "running".to_string();
        job.done = effective_done;
        job.total = effective_total;
        let next_percent = progress_percent(effective_done, effective_total);
        job.message = if effective_total > 0 {
            format!("{label}: {next_percent}%")
        } else {
            label.to_string()
        };
        if should_log_progress(
            previous_percent,
            next_percent,
            effective_done,
            effective_total,
        ) {
            crate::logging::runtime_log(
                crate::logging::LogLevel::Debug,
                "job-progress",
                format!(
                    "{job_id} | {} | done={effective_done} total={effective_total}",
                    job.message
                ),
            );
        }
    }
}

/// İşin aktif fazını (örn. "RAM edinimi", "SHA-256 hesaplanıyor") günceller.
pub fn update_acquisition_phase(job_id: &str, phase: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.phase = Some(phase.to_string());
    }
}

/// İşin anlık durum mesajını log'a da ekleyerek değiştirir.
pub fn update_acquisition_message(job_id: &str, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.message = message.to_string();
        push_log(job, message);
        crate::logging::runtime_log(
            crate::logging::LogLevel::Info,
            "job-message",
            format!("{job_id} | {message}"),
        );
    }
}

/// Canlı konsola ek bir satır yazmak için iş log'una mesaj ekler.
pub fn append_acquisition_log(job_id: &str, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        push_log(job, message);
        crate::logging::runtime_log(
            crate::logging::LogLevel::Debug,
            "job-log",
            format!("{job_id} | {message}"),
        );
    }
}

/// İşi başarılı tamamlanmış olarak işaretler ve sonucu saklar.
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
        push_log(job, message);
        job.result = Some(result);
        job.error = None;
        crate::logging::runtime_log(
            crate::logging::LogLevel::Info,
            "job",
            format!("{job_id} tamamlandi: {message}"),
        );
    }
}

/// İşi başarısız olarak işaretler ve hata mesajını log'a yazar.
pub fn fail_acquisition_job_with_message(job_id: &str, error: String, message: &str) {
    let error = crate::diagnostics::error_with_advice(&error);
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.status = "failed".to_string();
        job.message = message.to_string();
        push_log(job, message);
        push_log(job, &error);
        crate::logging::runtime_log(
            crate::logging::LogLevel::Error,
            "job",
            format!("{job_id} basarisiz: {message} | {error}"),
        );
        job.error = Some(error);
    }
}

/// Aynı mesajı tekrarlamadan sınırlı uzunlukta iş log'u tutar.
fn push_log(job: &mut AcquisitionJob, message: &str) {
    let clean = message.trim();
    if clean.is_empty() {
        return;
    }
    if job.logs.last().is_some_and(|last| last == clean) {
        return;
    }
    job.logs.push(clean.to_string());
    let overflow = job.logs.len().saturating_sub(400);
    if overflow > 0 {
        job.logs.drain(0..overflow);
    }
}

fn progress_percent(done: u64, total: u64) -> u64 {
    if total == 0 {
        0
    } else {
        (done * 100 / total).min(100)
    }
}

/// Bayt sayısını arayüzde okunur metne çevirir.
pub fn format_job_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    let value = bytes as f64;
    let text = if value >= GB {
        format!("{:.1} GB", value / GB)
    } else if value >= MB {
        format!("{:.1} MB", value / MB)
    } else if value >= KB {
        format!("{:.1} KB", value / KB)
    } else {
        format!("{bytes} B")
    };
    text.replace('.', ",")
}

/// Saniye süresini kısa Türkçe süre metnine çevirir.
pub fn format_job_eta(total_secs: u64) -> String {
    if total_secs < 60 {
        format!("{total_secs} sn")
    } else if total_secs < 3600 {
        format!("{} dk {} sn", total_secs / 60, total_secs % 60)
    } else {
        format!("{} sa {} dk", total_secs / 3600, (total_secs % 3600) / 60)
    }
}

/// Hash gibi edinim sonu uzun fazları hiz ve kalan süreyle raporlar.
/// Konsol ve aktif islemler kutusu buradan beslenir.
pub struct PhaseProgress {
    job_id: String,
    phase: &'static str,
    started: Instant,
    last_log: Instant,
    logged_start: bool,
}

impl PhaseProgress {
    /// Fazı başlatır, toplam boyutu konsola yazar.
    pub fn start(job_id: &str, phase: &'static str, total: u64) -> Self {
        update_acquisition_phase(job_id, phase);
        let reporter = Self {
            job_id: job_id.to_string(),
            phase,
            started: Instant::now(),
            last_log: Instant::now() - Duration::from_secs(10),
            logged_start: false,
        };
        if total > 0 {
            update_acquisition_message(
                job_id,
                &format!("{} başladı ({})", phase, format_job_bytes(total)),
            );
        } else {
            update_acquisition_message(job_id, &format!("{phase} başladı"));
        }
        reporter
    }

    /// Okunan baytı hiz ve tahmini süreyle ilerleme mesajına yazar.
    pub fn report(&mut self, done: u64, total: u64, phase: &'static str) {
        self.phase = phase;
        update_acquisition_phase(&self.job_id, phase);
        let elapsed = self.started.elapsed().as_secs_f64().max(0.5);
        let speed = done as f64 / elapsed;
        let speed_text = if speed > 0.0 {
            format!(
                " • {} MB/s",
                format!("{:.1}", speed / 1_048_576.0).replace('.', ",")
            )
        } else {
            String::new()
        };
        let eta_text = if total > 0 && done > 0 && (total.saturating_sub(done)) > 0 && speed > 0.0 {
            let remaining = ((total - done) as f64 / speed) as u64;
            format!(" • kalan {}", format_job_eta(remaining))
        } else {
            String::new()
        };
        let label = if total > 0 {
            format!(
                "{} • {}/{}{speed_text}{eta_text}",
                self.phase,
                format_job_bytes(done),
                format_job_bytes(total),
            )
        } else {
            self.phase.to_string()
        };
        update_acquisition_progress_message(&self.job_id, done, total, &label);
        if !self.logged_start || self.last_log.elapsed() >= Duration::from_secs(2) {
            self.logged_start = true;
            self.last_log = Instant::now();
            let pct = progress_percent(done, total);
            if total > 0 {
                append_acquisition_log(
                    &self.job_id,
                    &format!(
                        "{} %{pct} • {}/{}{speed_text}{eta_text}",
                        self.phase,
                        format_job_bytes(done),
                        format_job_bytes(total),
                    ),
                );
            } else {
                append_acquisition_log(&self.job_id, &format!("{} sürüyor", self.phase));
            }
        }
    }
}

fn should_log_progress(previous: u64, next: u64, done: u64, total: u64) -> bool {
    if total == 0 {
        return false;
    }
    done == 0 || done >= total || previous / 10 != next / 10
}
