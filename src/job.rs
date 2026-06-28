//! Genel amaçlı iş kuyruğu ve iş durum takibi altyapısını sağlar.
use crate::logging::Logger;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Condvar, Mutex};

static NEXT_JOB_ID: AtomicI32 = AtomicI32::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Kuyruktaki işin yaşam döngüsü durumunu belirtir.
pub enum JobStatus {
    Bekliyor,
    Calisiyor,
    Tamamlandi,
    Hata,
    IptalEdildi,
}

impl JobStatus {
    /// İş durumunu arayüz/rapor için Türkçe metne çevirir.
    pub fn text(self) -> &'static str {
        match self {
            JobStatus::Bekliyor => "Bekliyor",
            JobStatus::Calisiyor => "Calisiyor",
            JobStatus::Tamamlandi => "Tamamlandi",
            JobStatus::Hata => "Hata",
            JobStatus::IptalEdildi => "Iptal Edildi",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Kuyruğa eklenebilen genel iş türlerini belirtir.
pub enum JobType {
    DiskEdinim,
    HashHesapla,
    Dogrulama,
    SistemBilgisi,
    AgTransferi,
    RaporOlustur,
}

impl JobType {
    /// İş türünü arayüz/rapor için Türkçe metne çevirir.
    pub fn text(self) -> &'static str {
        match self {
            JobType::DiskEdinim => "Disk Edinim",
            JobType::HashHesapla => "Hash Hesaplama",
            JobType::Dogrulama => "Dogrulama",
            JobType::SistemBilgisi => "Sistem Bilgisi",
            JobType::AgTransferi => "Ag Transferi",
            JobType::RaporOlustur => "Rapor Olustur",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Tek bir arka plan işinin kimlik, durum, süre ve çıktı bilgisini taşır.
pub struct Job {
    pub id: i32,
    pub job_type: JobType,
    pub status: JobStatus,
    pub description: String,
    pub started_at: Option<DateTime<Local>>,
    pub finished_at: Option<DateTime<Local>>,
    pub progress_percent: u8,
    pub output_dir: Option<PathBuf>,
    pub produced_files: Vec<PathBuf>,
    pub error_message: Option<String>,
}

impl Job {
    /// Yeni işi benzersiz ID ile bekliyor durumunda oluşturur.
    pub fn new(job_type: JobType, description: impl Into<String>) -> Self {
        Self {
            id: NEXT_JOB_ID.fetch_add(1, Ordering::SeqCst),
            job_type,
            status: JobStatus::Bekliyor,
            description: description.into(),
            started_at: None,
            finished_at: None,
            progress_percent: 0,
            output_dir: None,
            produced_files: Vec::new(),
            error_message: None,
        }
    }

    /// Durumu, ilerlemeyi ve başlangıç/bitiş zamanlarını günceller.
    pub fn update_status(&mut self, status: JobStatus, progress: Option<u8>) {
        self.status = status;
        if let Some(progress) = progress {
            self.progress_percent = progress.min(100);
        }

        if status == JobStatus::Calisiyor && self.started_at.is_none() {
            self.started_at = Some(Local::now());
        }

        if matches!(
            status,
            JobStatus::Tamamlandi | JobStatus::Hata | JobStatus::IptalEdildi
        ) && self.finished_at.is_none()
        {
            self.finished_at = Some(Local::now());
        }
    }

    /// İşi başarılı tamamlandı olarak işaretler.
    pub fn complete(&mut self, output_dir: Option<PathBuf>) {
        self.update_status(JobStatus::Tamamlandi, Some(100));
        self.output_dir = output_dir;
    }

    /// İşi hata durumuna alır ve hata mesajını saklar.
    pub fn fail(&mut self, message: impl Into<String>) {
        self.update_status(JobStatus::Hata, None);
        self.error_message = Some(message.into());
    }

    /// İşi iptal edildi olarak işaretler.
    pub fn cancel(&mut self) {
        self.update_status(JobStatus::IptalEdildi, None);
    }

    /// İşin ürettiği çıktı dosyasını kayıt altına alır.
    pub fn add_output_file(&mut self, path: impl Into<PathBuf>) {
        self.produced_files.push(path.into());
    }
}

pub type SharedJob = Arc<Mutex<Job>>;

/// Çoklu thread arasında paylaşılan basit iş kuyruğudur.
pub struct JobQueue {
    queue: Mutex<VecDeque<SharedJob>>,
    condvar: Condvar,
    running: Mutex<bool>,
    logger: Option<Logger>,
}

impl JobQueue {
    /// Yeni kuyruk ve koşul değişkeni oluşturur.
    pub fn new(logger: Option<Logger>) -> Self {
        if let Some(logger) = &logger {
            logger.info("Is kuyrugu olusturuldu");
        }
        Self {
            queue: Mutex::new(VecDeque::new()),
            condvar: Condvar::new(),
            running: Mutex::new(true),
            logger,
        }
    }

    /// İşi kuyruğa ekler ve bekleyen worker'ı uyandırır.
    pub fn push(&self, job: Job) -> SharedJob {
        let shared = Arc::new(Mutex::new(job));
        if let Ok(mut queue) = self.queue.lock() {
            if let Some(logger) = &self.logger
                && let Ok(job) = shared.lock()
            {
                logger.info(format!("Is eklendi: {} (ID: {})", job.description, job.id));
            }
            queue.push_back(shared.clone());
            self.condvar.notify_one();
        }
        shared
    }

    /// İş gelene kadar bekler veya kuyruk kapanırsa None döner.
    pub fn pop_wait(&self) -> Option<SharedJob> {
        let mut queue = self.queue.lock().ok()?;
        loop {
            if let Some(job) = queue.pop_front() {
                return Some(job);
            }
            if !*self.running.lock().ok()? {
                return None;
            }
            queue = self.condvar.wait(queue).ok()?;
        }
    }

    /// Kuyruğu durdurur ve bekleyen thread'leri uyandırır.
    pub fn stop(&self) {
        if let Ok(mut running) = self.running.lock() {
            *running = false;
        }
        self.condvar.notify_all();
        if let Some(logger) = &self.logger {
            logger.info("Is kuyrugu kapatildi");
        }
    }
}
