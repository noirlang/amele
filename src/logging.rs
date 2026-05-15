use crate::error::{HataKodu, WormError, WormResult};
use chrono::Local;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        }
    }
}

#[derive(Clone)]
pub struct Logger {
    case_name: String,
    log_dir: PathBuf,
    active_file: PathBuf,
    min_level: LogLevel,
    file: Arc<Mutex<File>>,
}

impl Logger {
    pub fn start(case_name: impl AsRef<str>, log_dir: impl AsRef<Path>) -> WormResult<Self> {
        let case_name = case_name.as_ref().to_string();
        let log_dir = log_dir.as_ref().to_path_buf();
        fs::create_dir_all(&log_dir).map_err(|err| {
            WormError::io(HataKodu::DosyaYazma, "Gunluk klasoru olusturulamadi", err)
        })?;

        let file_name = format!("{}_{}.log", case_name, Local::now().format("%Y%m%d_%H%M%S"));
        let active_file = log_dir.join(file_name);
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&active_file)
            .map_err(|err| {
                WormError::io(HataKodu::DosyaAcilamadi, "Gunluk dosyasi acilamadi", err)
            })?;

        let logger = Self {
            case_name,
            log_dir,
            active_file,
            min_level: LogLevel::Debug,
            file: Arc::new(Mutex::new(file)),
        };
        logger.info(format!(
            "Gunluk sistemi baslatildi: {}",
            logger.active_file.display()
        ));
        Ok(logger)
    }

    pub fn active_file(&self) -> &Path {
        &self.active_file
    }

    pub fn case_name(&self) -> &str {
        &self.case_name
    }

    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }

    pub fn log(&self, level: LogLevel, message: impl AsRef<str>) {
        if level < self.min_level {
            return;
        }

        let line = format!(
            "[{}] | {} | {}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            level.as_str(),
            message.as_ref()
        );

        if let Ok(mut file) = self.file.lock() {
            let _ = file.write_all(line.as_bytes());
            let _ = file.flush();
        }
    }

    pub fn system(&self, level: LogLevel, message: impl AsRef<str>) {
        self.log(level, format!("[SISTEM] {}", message.as_ref()));
    }

    pub fn info(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Info, message);
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Warn, message);
    }

    pub fn error(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Error, message);
    }

    pub fn debug(&self, message: impl AsRef<str>) {
        self.log(LogLevel::Debug, message);
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.info("Gunluk sistemi kapatiliyor");
    }
}
