//! Android cihazın hangi forensic modülleri desteklediğini raporlar.
use super::adb::run_adb_command_timeout;
use super::profile::AndroidDeviceProfile;
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Modülün çalıştırılabilirlik seviyesini UI ve manifest için standartlaştırır.
pub enum AndroidCapabilityLevel {
    Supported,
    Partial,
    Unsupported,
}

#[derive(Debug, Clone, Serialize)]
/// Tek bir Android forensic kabiliyetinin durumunu ve sebebini taşır.
pub struct AndroidCapabilityCheck {
    pub level: AndroidCapabilityLevel,
    pub available: bool,
    pub reason: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
/// AndroidQF, Lemon ve MESH odaklı gelecek modüller için ortak ön kontrol raporudur.
pub struct AndroidCapabilityReport {
    pub serial: String,
    pub generated_at: String,
    pub adb_authorized: AndroidCapabilityCheck,
    pub logical_acquisition: AndroidCapabilityCheck,
    pub shared_storage: AndroidCapabilityCheck,
    pub bugreport: AndroidCapabilityCheck,
    pub adb_backup: AndroidCapabilityCheck,
    pub filesystem_non_root: AndroidCapabilityCheck,
    pub filesystem_root: AndroidCapabilityCheck,
    pub volatile_memory: AndroidCapabilityCheck,
    pub process_memory_root: AndroidCapabilityCheck,
    pub physical_memory_probe: AndroidCapabilityCheck,
    pub lemon_physical_memory: AndroidCapabilityCheck,
    pub remote_mesh_transport: AndroidCapabilityCheck,
}

/// Profil ve hafif ADB kontrollerinden cihaz kabiliyet raporu üretir.
pub fn build_android_capability_report(
    serial: &str,
    profile: &AndroidDeviceProfile,
) -> AndroidCapabilityReport {
    let adb_ready = adb_echo_ok(serial);
    let shared_storage_ready = shell_returns_yes(serial, "test -d /sdcard && echo yes || echo no");
    let bugreport_ready = shell_has_any(
        serial,
        "ls /system/bin/bugreportz /system/bin/bugreport 2>/dev/null | head -n 1",
    );
    let root_ready = profile.is_rooted;
    let supported_abi = profile
        .abi
        .as_deref()
        .map(is_lemon_supported_abi)
        .unwrap_or(false);

    AndroidCapabilityReport {
        serial: serial.trim().to_string(),
        generated_at: chrono::Local::now().to_rfc3339(),
        adb_authorized: if adb_ready {
            supported("ADB cihaza komut gonderebiliyor")
        } else {
            unsupported(
                "ADB cihaza komut gonderemedi",
                "Cihaz ekraninda USB hata ayiklama iznini onaylayin ve tekrar deneyin.",
            )
        },
        logical_acquisition: if adb_ready {
            supported("ADB logical artifact toplama icin hazir")
        } else {
            unsupported(
                "Logical edinim icin ADB baglantisi gerekli",
                "ADB baglantisini duzeltmeden Android logical edinim baslatilamaz.",
            )
        },
        shared_storage: if shared_storage_ready {
            supported("/sdcard paylasimli depolama okunabilir")
        } else {
            partial(
                "/sdcard yolu dogrulanamadi",
                "Cihaz kilidini acin; islem sadece diger logical artifactlerle devam edebilir.",
            )
        },
        bugreport: if bugreport_ready {
            supported("Bugreport komutu cihazda mevcut")
        } else {
            partial(
                "Bugreport komutu bulunamadi veya cihaz izin vermedi",
                "Dumpsys, logcat ve paket modulleri yine de calistirilabilir.",
            )
        },
        adb_backup: backup_capability(profile.api_level),
        filesystem_non_root: if shared_storage_ready {
            supported("Non-root dosya indeksi ve paylasimli depolama toplanabilir")
        } else {
            partial(
                "Non-root dosya toplama sinirli",
                "Cihaz kilidini acin veya sadece metadata/log modulleriyle devam edin.",
            )
        },
        filesystem_root: if root_ready {
            supported("Root dosya sistemi toplama denenebilir")
        } else {
            unsupported(
                "Root dosya sistemi icin root veya adbd root gerekli",
                "Root yoksa tam /data edinimi yerine non-root dosya toplama kullanin.",
            )
        },
        volatile_memory: if adb_ready {
            supported("Root gerektirmeyen ucucu veri snapshot'i alinabilir")
        } else {
            unsupported(
                "Ucucu veri icin ADB baglantisi gerekli",
                "ADB baglantisi ve cihaz yetkilendirmesini kontrol edin.",
            )
        },
        process_memory_root: if root_ready {
            partial(
                "Root ile sinirli proses bellegi denenebilir",
                "Modern Android SELinux politikalari proses bellegini yine de engelleyebilir.",
            )
        } else {
            unsupported(
                "Proses bellegi icin root gerekli",
                "Root yoksa ucucu veri snapshot modunu kullanin.",
            )
        },
        physical_memory_probe: if root_ready {
            partial(
                "Klasik fiziksel bellek arayuzleri denenebilir",
                "/proc/kcore veya /dev/mem cogu modern Android cihazda kapali olabilir.",
            )
        } else {
            unsupported(
                "Fiziksel bellek denemesi root gerektirir",
                "Root yoksa fiziksel RAM edinimi bu cihazda desteklenmez.",
            )
        },
        lemon_physical_memory: lemon_capability(root_ready, supported_abi, profile.abi.as_deref()),
        remote_mesh_transport: partial(
            "MESH uzak ADB tasimasi icin ayri eslestirme gerekir",
            "Ilk asamada MESH yerine adb connect <ip:port> hedefi kullanilabilir.",
        ),
    }
}

/// Kabiliyet raporunu Android çıktı klasörüne yazar.
pub fn write_android_capability_report(
    output_dir: &Path,
    report: &AndroidCapabilityReport,
) -> Result<(), String> {
    let path = output_dir.join("android_capabilities.json");
    let content = serde_json::to_vec_pretty(report)
        .map_err(|err| format!("Android kabiliyet raporu JSON'a cevrilemedi: {err}"))?;
    std::fs::write(&path, content)
        .map_err(|err| format!("Android kabiliyet raporu yazilamadi: {err}"))
}

/// ADB shell komutu çalışabiliyor mu diye hızlı kontrol yapar.
fn adb_echo_ok(serial: &str) -> bool {
    run_adb_command_timeout(
        serial,
        &["shell", "echo worm_adb_ok"],
        Duration::from_secs(5),
    )
    .map(|output| output.contains("worm_adb_ok"))
    .unwrap_or(false)
}

/// Shell komutunun yes döndürüp döndürmediğini kontrol eder.
fn shell_returns_yes(serial: &str, command: &str) -> bool {
    run_adb_command_timeout(serial, &["shell", command], Duration::from_secs(5))
        .map(|output| output.lines().any(|line| line.trim() == "yes"))
        .unwrap_or(false)
}

/// Shell komutundan boş olmayan çıktı gelirse aracı mevcut sayar.
fn shell_has_any(serial: &str, command: &str) -> bool {
    run_adb_command_timeout(serial, &["shell", command], Duration::from_secs(5))
        .map(|output| output.lines().any(|line| !line.trim().is_empty()))
        .unwrap_or(false)
}

/// Desteklenen durumu kısa yoldan üretir.
fn supported(reason: impl Into<String>) -> AndroidCapabilityCheck {
    AndroidCapabilityCheck {
        level: AndroidCapabilityLevel::Supported,
        available: true,
        reason: reason.into(),
        recommendation: None,
    }
}

/// Kısmi destek durumunu kısa yoldan üretir.
fn partial(reason: impl Into<String>, recommendation: impl Into<String>) -> AndroidCapabilityCheck {
    AndroidCapabilityCheck {
        level: AndroidCapabilityLevel::Partial,
        available: true,
        reason: reason.into(),
        recommendation: Some(recommendation.into()),
    }
}

/// Desteklenmeyen durumu kısa yoldan üretir.
fn unsupported(
    reason: impl Into<String>,
    recommendation: impl Into<String>,
) -> AndroidCapabilityCheck {
    AndroidCapabilityCheck {
        level: AndroidCapabilityLevel::Unsupported,
        available: false,
        reason: reason.into(),
        recommendation: Some(recommendation.into()),
    }
}

/// Android yedekleme komutunun modern API'lerde ne kadar kullanışlı olduğunu değerlendirir.
fn backup_capability(api_level: Option<u32>) -> AndroidCapabilityCheck {
    match api_level {
        Some(level) if level <= 30 => partial(
            "ADB backup bu API seviyesinde denenebilir",
            "Uygulamalar allowBackup=false ise veri donmeyebilir.",
        ),
        Some(level) => partial(
            format!("ADB backup Android API {level} icin ciddi sekilde kisitli"),
            "Bugreport, packages, shared storage ve app media modullerini tercih edin.",
        ),
        None => partial(
            "API seviyesi okunamadi; ADB backup sonucu garanti edilemez",
            "Once cihaz profilini yenileyin.",
        ),
    }
}

/// Lemon fiziksel RAM modunun mimari ve root açısından mümkün olup olmadığını değerlendirir.
fn lemon_capability(
    root_ready: bool,
    supported_abi: bool,
    abi: Option<&str>,
) -> AndroidCapabilityCheck {
    if !root_ready {
        return unsupported(
            "Lemon fiziksel RAM modu root gerektirir",
            "Root yoksa ucucu veri snapshot modunu kullanin.",
        );
    }
    if !supported_abi {
        return unsupported(
            format!(
                "Lemon icin desteklenen ABI bulunamadi: {}",
                abi.unwrap_or("bilinmiyor")
            ),
            "arm64-v8a veya armeabi-v7a cihazlarda tekrar deneyin.",
        );
    }
    partial(
        "Lemon icin root ve desteklenen ABI mevcut",
        "eBPF/BTF ve kernel kilitleme durumu ayrica kontrol edilmelidir.",
    )
}

/// Lemon binary seçiminde desteklenen Android ABI'lerini tanır.
fn is_lemon_supported_abi(abi: &str) -> bool {
    matches!(abi.trim(), "arm64-v8a" | "armeabi-v7a" | "x86_64" | "x86")
}

#[cfg(test)]
mod tests {
    use super::{
        AndroidCapabilityLevel, backup_capability, is_lemon_supported_abi, lemon_capability,
    };

    #[test]
    fn marks_modern_adb_backup_as_partial() {
        let check = backup_capability(Some(34));
        assert_eq!(check.level, AndroidCapabilityLevel::Partial);
        assert!(check.reason.contains("34"));
    }

    #[test]
    fn validates_lemon_abi_support() {
        assert!(is_lemon_supported_abi("arm64-v8a"));
        assert!(!is_lemon_supported_abi("mips"));
    }

    #[test]
    fn requires_root_for_lemon() {
        let check = lemon_capability(false, true, Some("arm64-v8a"));
        assert_eq!(check.level, AndroidCapabilityLevel::Unsupported);
    }
}
