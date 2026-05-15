use chrono::Local;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::{Mutex, OnceLock};

pub type WormResult<T> = Result<T, WormError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum HataKodu {
    Ok = 0,
    Genel = -1,
    Baglanti = -100,
    BaglantiZamanAsimi = -101,
    BaglantiKesildi = -102,
    BaglantiTekrarDene = -103,
    Dosya = -200,
    DosyaAcilamadi = -201,
    DosyaYazma = -202,
    DosyaOkuma = -203,
    Disk = -300,
    DiskErisim = -301,
    DiskOkuma = -302,
    DiskBoyut = -303,
    Ag = -400,
    AgGonderme = -401,
    AgAlma = -402,
    Protokol = -500,
    ProtokolJson = -501,
    ProtokolVersiyon = -502,
    Guvenlik = -600,
    TokenGecersiz = -601,
    YetkisizErisim = -602,
    Icerik = -700,
    IcerikGecersiz = -701,
    IcerikBuyuk = -702,
}

impl HataKodu {
    pub fn text(self) -> &'static str {
        match self {
            HataKodu::Ok => "Basarili",
            HataKodu::Genel => "Genel hata",
            HataKodu::Baglanti => "Connection error",
            HataKodu::BaglantiZamanAsimi => "Connection timeout",
            HataKodu::BaglantiKesildi => "Connection lost",
            HataKodu::BaglantiTekrarDene => "Connection should be retried",
            HataKodu::Dosya => "Dosya hatasi",
            HataKodu::DosyaAcilamadi => "Dosya acilamadi",
            HataKodu::DosyaYazma => "Dosya yazma hatasi",
            HataKodu::DosyaOkuma => "Dosya okuma hatasi",
            HataKodu::Disk => "Disk error",
            HataKodu::DiskErisim => "Disk access error",
            HataKodu::DiskOkuma => "Disk read error",
            HataKodu::DiskBoyut => "Disk size error",
            HataKodu::Ag => "Ag hatasi",
            HataKodu::AgGonderme => "Ag gonderme hatasi",
            HataKodu::AgAlma => "Ag alma hatasi",
            HataKodu::Protokol => "Protokol hatasi",
            HataKodu::ProtokolJson => "JSON protokol hatasi",
            HataKodu::ProtokolVersiyon => "Protokol versiyon uyumsuzlugu",
            HataKodu::Guvenlik => "Guvenlik hatasi",
            HataKodu::TokenGecersiz => "Gecersiz token",
            HataKodu::YetkisizErisim => "Yetkisiz erisim",
            HataKodu::Icerik => "Icerik hatasi",
            HataKodu::IcerikGecersiz => "Gecersiz icerik",
            HataKodu::IcerikBuyuk => "Icerik boyutu cok buyuk",
        }
    }

    pub fn is_severe(self) -> bool {
        (self as i32) <= (HataKodu::DiskErisim as i32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: HataKodu,
    pub message: String,
    pub detail: String,
    pub timestamp: String,
    pub source_file: String,
    pub source_line: u32,
}

impl Default for ErrorInfo {
    fn default() -> Self {
        Self {
            code: HataKodu::Ok,
            message: HataKodu::Ok.text().to_string(),
            detail: String::new(),
            timestamp: String::new(),
            source_file: String::new(),
            source_line: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WormError {
    pub code: HataKodu,
    pub message: String,
}

impl WormError {
    pub fn new(code: HataKodu, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn genel(message: impl Into<String>) -> Self {
        Self::new(HataKodu::Genel, message)
    }

    pub fn io(code: HataKodu, context: impl Into<String>, err: std::io::Error) -> Self {
        Self::new(code, format!("{}: {}", context.into(), err))
    }
}

impl Display for WormError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code.text(), self.message)
    }
}

impl Error for WormError {}

impl From<std::io::Error> for WormError {
    fn from(value: std::io::Error) -> Self {
        Self::io(HataKodu::Dosya, "IO hatasi", value)
    }
}

impl From<serde_json::Error> for WormError {
    fn from(value: serde_json::Error) -> Self {
        Self::new(HataKodu::ProtokolJson, value.to_string())
    }
}

static LAST_ERROR: OnceLock<Mutex<ErrorInfo>> = OnceLock::new();

fn last_error_cell() -> &'static Mutex<ErrorInfo> {
    LAST_ERROR.get_or_init(|| Mutex::new(ErrorInfo::default()))
}

pub fn record_error(
    code: HataKodu,
    message: Option<&str>,
    detail: Option<&str>,
    source_file: &str,
    source_line: u32,
) {
    let info = ErrorInfo {
        code,
        message: message.unwrap_or_else(|| code.text()).to_string(),
        detail: detail.unwrap_or_default().to_string(),
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        source_file: source_file.to_string(),
        source_line,
    };

    if let Ok(mut guard) = last_error_cell().lock() {
        *guard = info;
    }
}

pub fn last_error() -> ErrorInfo {
    last_error_cell()
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

#[macro_export]
macro_rules! hata_kaydet {
    ($code:expr, $message:expr) => {
        $crate::error::record_error($code, Some($message), None, file!(), line!())
    };
    ($code:expr, $message:expr, $detail:expr) => {
        $crate::error::record_error($code, Some($message), Some($detail), file!(), line!())
    };
}
