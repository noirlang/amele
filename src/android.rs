//! ============================================================================
//! # ANDROİD MOBİL ADLİ BİLİŞİM VE MANTIKSAL EDİNİM MİMARİSİ (src/android.rs)
//! ============================================================================
//!
//! Bu modül, Android akıllı telefon ve tabletlerden adli veri toplama,
//! yetenek analizi (capability check), non-root mantıksal edinim ve
//! Türkiye odaklı uygulama katalog taramasını tek noktadan koordine eder.
//!
//! ## 📱 MOBİL ADLİ MİMARİ:
//!
//! 1. **Fiziksel vs. Mantıksal Edinim (Physical vs. Logical Extraction):**
//!    Modern Android cihazlarda donanımsal dosya tabanlı şifreleme (FBE - File-Based
//!    Encryption) nedeniyle root yetkisi olmadan ham çip dökümü almak kısıtlıdır.
//!    Amele, Android Debug Bridge (ADB) üzerinden çalışan mantıksal edinim hattıyla:
//!    - Cihaz ve donanım metaverileri (`getprop`),
//!    - Kurulu paketler ve UID listeleri (`pm list packages -f -U`),
//!    - Çalışan süreçler ve bellek durumları (`ps -A`, `dumpsys meminfo`),
//!    - Sistem logları (`logcat -d`, `dmesg`),
//!    - Hesaplar (`dumpsys account`),
//!    - SD kart ve paylaşımlı medya dizinlerini (`/sdcard/DCIM`, Download vb.)
//!    bütünlük manifestolarıyla birlikte toplar.
//!
//! 2. **Hedef Odaklı Türkiye Uygulama Kataloğu (`app_catalog`):**
//!    WhatsApp, Telegram, Signal gibi genel haberleşme araçlarının yanı sıra;
//!    Türkiye'de yaygın kullanılan bankacılık, kamu (e-Devlet) ve yerel mesajlaşma
//!    uygulamalarının veri yolları ve varlıkları otomatik taranır.
//!
//! 3. **Taşıma Katmanı Soyutlaması (`session`):**
//!    Cihaz doğrudan USB kablosuyla veya yerel ağ üzerinden TCP portu (`adb connect`)
//!    ile bağlı olsa da tekdüze bir oturum arabirimi üzerinden yönetilir.
//! ============================================================================

mod adb;
mod app_catalog;
mod capability;
mod errors;
mod extractors;
mod filesystem;
mod logical;
mod manifest;
mod orchestrator;
mod profile;
mod ram;
mod remote;
mod session;
// TODO: Android Chipset Seviyesi Fiziksel Edinim (Qualcomm EDL 05c6:9008 & MediaTek BROM 0e8d:0003/2000) destegi ekle: src/android/chipset.rs modulu, USB VID/PID taramasi, API router endpointleri ve UI physical mode entegrasyonu

pub use adb::{AdbInstallResult, AdbStatus, AndroidDevice, adb_status, install_adb, list_devices};
pub use capability::{
    AndroidCapabilityCheck, AndroidCapabilityLevel, AndroidCapabilityReport,
    build_android_capability_report,
};
pub use errors::explain_android_error;
pub use extractors::{
    AndroidAcquisitionProfile, AndroidExtractorStep, FULL_LOGICAL_STEPS, logical_steps_for_profile,
};
pub use filesystem::{FilesystemAcquisitionResult, filesystem_acquisition};
pub use logical::{
    AcquisitionItem, LogicalAcquisitionResult, logical_acquisition,
    logical_acquisition_with_profile,
};
pub use manifest::{AndroidAcquisitionManifest, AndroidManifestArtifact, write_android_manifest};
pub use orchestrator::{
    AndroidOrchestratedAcquisitionResult, AndroidOrchestratedFilesystemResult,
    AndroidOrchestratedRamResult, orchestrated_acquisition, orchestrated_filesystem_acquisition,
    orchestrated_ram_acquisition,
};
pub use profile::{AndroidDeviceProfile, detect_device_profile};
pub use ram::{
    AndroidRamAcquisitionResult, AndroidRamMode, ram_acquisition, ram_acquisition_with_mode,
};
pub use remote::{
    LemonPreflight, RemoteAndroidEndpoint, RemoteConnectResult, RemoteEndpointKind,
    connect_remote_endpoint, disconnect_remote_endpoint, lemon_preflight,
};
pub use session::{AndroidSession, AndroidTransport, AndroidTransportKind, build_android_session};
// TODO: Android Chipset Seviyesi Fiziksel Edinim (Qualcomm EDL & MediaTek BROM)
