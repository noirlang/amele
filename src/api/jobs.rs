//! Edinim işlerinin (disk, ram, ios vb.) durumunu, ilerlemesini, loglarını ve hızını yöneten durum modülü.

use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, MutexGuard, OnceLock};
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

/// Mutex zehirlenmesine dayanıklı iş kilidi edinir.
fn lock_acquisition_jobs() -> MutexGuard<'static, HashMap<String, AcquisitionJob>> {
    match acquisition_jobs().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
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
    lock_acquisition_jobs().insert(job_id.clone(), job);
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
    let mut jobs = lock_acquisition_jobs();
    if let Some(job) = jobs.get_mut(job_id) {
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
    let mut jobs = lock_acquisition_jobs();
    if let Some(job) = jobs.get_mut(job_id) {
        job.phase = Some(phase.to_string());
    }
}

/// İşin anlık durum mesajını log'a da ekleyerek değiştirir.
pub fn update_acquisition_message(job_id: &str, message: &str) {
    let mut jobs = lock_acquisition_jobs();
    if let Some(job) = jobs.get_mut(job_id) {
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
    let mut jobs = lock_acquisition_jobs();
    if let Some(job) = jobs.get_mut(job_id) {
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
    let mut jobs = lock_acquisition_jobs();
    if let Some(job) = jobs.get_mut(job_id) {
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
    let mut jobs = lock_acquisition_jobs();
    if let Some(job) = jobs.get_mut(job_id) {
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
    } else if total_secs < 86400 {
        format!("{} sa {} dk", total_secs / 3600, (total_secs % 3600) / 60)
    } else {
        format!("{} gün {} sa", total_secs / 86400, (total_secs % 86400) / 3600)
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
            last_log: Instant::now(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_lifecycle_job() {
        let (job_id, control) = create_acquisition_job("Test edinim işi");
        assert!(job_id.starts_with("acq-"));
        assert!(!control.is_cancelled());

        let jobs = lock_acquisition_jobs();
        let job = jobs.get(&job_id).expect("iş bulunmalı");
        assert_eq!(job.status, "running");
        assert_eq!(job.message, "Test edinim işi");
        assert_eq!(job.logs, vec!["Test edinim işi".to_string()]);
    }

    #[test]
    fn test_progress_updates_and_percent() {
        let (job_id, _) = create_acquisition_job("Ilerleme testi");
        update_acquisition_progress(&job_id, 50, 100);

        {
            let jobs = lock_acquisition_jobs();
            let job = jobs.get(&job_id).expect("iş bulunmalı");
            assert_eq!(job.done, 50);
            assert_eq!(job.total, 100);
            assert_eq!(job.message, "Imaj alma sürüyor: 50%");
        }

        update_acquisition_phase(&job_id, "Hash hesaplama");
        {
            let jobs = lock_acquisition_jobs();
            let job = jobs.get(&job_id).expect("iş bulunmalı");
            assert_eq!(job.phase.as_deref(), Some("Hash hesaplama"));
        }
    }

    #[test]
    fn test_push_log_dedup_and_overflow() {
        let (job_id, _) = create_acquisition_job("Log testi");
        append_acquisition_log(&job_id, "Aynı log");
        append_acquisition_log(&job_id, "Aynı log");
        append_acquisition_log(&job_id, "  ");

        let jobs = lock_acquisition_jobs();
        let job = jobs.get(&job_id).expect("iş bulunmalı");
        assert_eq!(job.logs, vec!["Log testi".to_string(), "Aynı log".to_string()]);
    }

    #[test]
    fn test_finish_and_fail_job() {
        let (job1, _) = create_acquisition_job("Bitecek iş");
        finish_acquisition_job_with_message(&job1, serde_json::json!({ "ok": true }), "Başarıyla bitti");
        {
            let jobs = lock_acquisition_jobs();
            let job = jobs.get(&job1).expect("iş bulunmalı");
            assert_eq!(job.status, "completed");
            assert_eq!(job.done, 1);
            assert_eq!(job.total, 1);
            assert_eq!(job.result, Some(serde_json::json!({ "ok": true })));
        }

        let (job2, _) = create_acquisition_job("Başarısız iş");
        fail_acquisition_job_with_message(&job2, "Erişim reddedildi".to_string(), "Kopyalama hatası");
        {
            let jobs = lock_acquisition_jobs();
            let job = jobs.get(&job2).expect("iş bulunmalı");
            assert_eq!(job.status, "failed");
            assert!(job.error.as_deref().unwrap().contains("Erişim reddedildi"));
        }
    }

    #[test]
    fn test_format_job_bytes() {
        assert_eq!(format_job_bytes(0), "0 B");
        assert_eq!(format_job_bytes(512), "512 B");
        assert_eq!(format_job_bytes(1024), "1,0 KB");
        assert_eq!(format_job_bytes(1_572_864), "1,5 MB");
        assert_eq!(format_job_bytes(16_106_127_360), "15,0 GB");
    }

    #[test]
    fn test_format_job_eta() {
        assert_eq!(format_job_eta(30), "30 sn");
        assert_eq!(format_job_eta(150), "2 dk 30 sn");
        assert_eq!(format_job_eta(3720), "1 sa 2 dk");
        assert_eq!(format_job_eta(90000), "1 gün 1 sa");
    }

    #[test]
    fn test_phase_progress() {
        let (job_id, _) = create_acquisition_job("Faz testi");
        let mut phase = PhaseProgress::start(&job_id, "SHA-256", 1000);
        phase.report(500, 1000, "SHA-256");

        let jobs = lock_acquisition_jobs();
        let job = jobs.get(&job_id).expect("iş bulunmalı");
        assert_eq!(job.phase.as_deref(), Some("SHA-256"));
        assert_eq!(job.done, 500);
        assert_eq!(job.total, 1000);
    }
}
