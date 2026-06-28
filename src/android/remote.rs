//! Uzak Android cihazlara TCP/IP ADB ve MESH üzerinden bağlantı yönetimi.
//!
//! Bu modül doğrudan MESH kodu içermez (AGPL lisans nedeniyle).
//! Bunun yerine MESH'i harici bir remote ADB transport noktası olarak kullanır:
//! kullanıcı MESH endpoint'ini belirtir, Worm standart `adb connect ip:port` ile bağlanır.

use super::adb::run_adb_command_timeout;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::time::Duration;

// ---------------------------------------------------------------------------
// Veri yapıları
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Uzak Android bağlantısının türünü belirtir.
pub enum RemoteEndpointKind {
    /// Doğrudan IP:port ile TCP/IP ADB bağlantısı.
    TcpAdb,
    /// MESH altyapısı üzerinden sağlanan ADB endpoint.
    MeshRelay,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Kullanıcının tanımladığı uzak Android ADB endpoint bilgisi.
pub struct RemoteAndroidEndpoint {
    /// Görünen ad (kullanıcı tanımlı, ör. "Lab cihazı", "Saha - Telefon 2")
    pub label: String,
    /// Bağlanılacak host. IP adresi veya MESH peer hostname.
    pub host: String,
    /// ADB dinleme portu (genellikle 5555).
    pub port: u16,
    /// Endpoint türü.
    pub kind: RemoteEndpointKind,
}

impl RemoteAndroidEndpoint {
    /// ADB bağlantısı için `host:port` formatında serial döndürür.
    pub fn serial(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Serialize)]
/// `adb connect` veya `adb disconnect` işleminin sonucunu taşır.
pub struct RemoteConnectResult {
    pub serial: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
/// Lemon fiziksel RAM aracı için ön kontrol raporu.
pub struct LemonPreflight {
    /// Cihaz ABI, örn. "arm64-v8a".
    pub abi: Option<String>,
    /// ABI Lemon tarafından destekleniyor mu?
    pub abi_supported: bool,
    /// Root (su veya adb root) mevcut mu?
    pub root_available: bool,
    /// Kernel sürümü, örn. "5.10.43-android12".
    pub kernel_version: Option<String>,
    /// Çekirdek eBPF desteği var mı? (/sys/kernel/btf/vmlinux kontrolü)
    pub ebpf_btf_available: bool,
    /// /proc/kcore veya /dev/mem erişilebilir mi?
    pub kcore_available: bool,
    /// Cihazda yeterli boş alan var mı? (Geçici çalışma alanı için)
    pub storage_ok: bool,
    /// Tahmini fiziksel RAM boyutu (MB).
    pub ram_mb: Option<u64>,
    /// SoC uyarısı: Exynos/MediaTek çekirdekte yeniden başlatmaya neden olabilir.
    pub soc_warning: Option<String>,
    /// Genel preflight özeti: Lemon çalıştırılabilir mi?
    pub ready: bool,
    /// Hazır değilse kullanıcıya gösterilecek açıklama.
    pub reason: Option<String>,
}

// ---------------------------------------------------------------------------
// Bağlantı fonksiyonları
// ---------------------------------------------------------------------------

/// Belirtilen endpoint'e `adb connect` komutuyla bağlanır.
pub fn connect_remote_endpoint(endpoint: &RemoteAndroidEndpoint) -> RemoteConnectResult {
    let serial = endpoint.serial();
    let result = Command::new("adb").args(["connect", &serial]).output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let combined = if stderr.is_empty() {
                stdout.clone()
            } else {
                format!("{stdout} {stderr}")
            };

            // adb connect başarılıysa "connected to" veya "already connected" döner.
            let success = output.status.success()
                && (combined.contains("connected to") || combined.contains("already connected"));

            RemoteConnectResult {
                serial,
                success,
                message: combined,
            }
        }
        Err(err) => RemoteConnectResult {
            serial,
            success: false,
            message: format!("ADB başlatılamadı: {err}"),
        },
    }
}

/// Belirtilen serial'ı ADB cihaz listesinden çıkarır.
pub fn disconnect_remote_endpoint(serial: &str) -> RemoteConnectResult {
    let result = Command::new("adb").args(["disconnect", serial]).output();

    match result {
        Ok(output) => {
            let msg = String::from_utf8_lossy(&output.stdout).trim().to_string();
            RemoteConnectResult {
                serial: serial.to_string(),
                success: output.status.success(),
                message: msg,
            }
        }
        Err(err) => RemoteConnectResult {
            serial: serial.to_string(),
            success: false,
            message: format!("ADB bağlantısı kesilemedi: {err}"),
        },
    }
}

// ---------------------------------------------------------------------------
// Lemon preflight
// ---------------------------------------------------------------------------

/// Cihazın Lemon fiziksel RAM dump'ına uygunluğunu kontrol eder.
///
/// Lemon eBPF tabanlı olduğu için şu koşullar aranır:
/// - ARM64 veya x86_64 ABI
/// - Root erişimi (su veya adb root)
/// - Kernel eBPF + BTF desteği (/sys/kernel/btf/vmlinux)
/// - Yeterli depolama alanı
/// - Exynos/MediaTek SoC uyarısı
pub fn lemon_preflight(serial: &str) -> LemonPreflight {
    let timeout = Duration::from_secs(8);

    // ABI kontrolü
    let abi = run_adb_command_timeout(serial, &["shell", "getprop", "ro.product.cpu.abi"], timeout)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty());

    let abi_supported = abi.as_deref().map(is_lemon_supported_abi).unwrap_or(false);

    // Root kontrolü
    let adb_root = run_adb_command_timeout(serial, &["shell", "id"], timeout)
        .map(|out| out.contains("uid=0(root)"))
        .unwrap_or(false);
    let su_root = if !adb_root {
        run_adb_command_timeout(serial, &["shell", "su", "-c", "id"], timeout)
            .map(|out| out.contains("uid=0(root)"))
            .unwrap_or(false)
    } else {
        false
    };
    let root_available = adb_root || su_root;

    // Kernel sürümü
    let kernel_version =
        run_adb_command_timeout(serial, &["shell", "cat", "/proc/version"], timeout)
            .ok()
            .map(|v| v.lines().next().unwrap_or("").trim().to_string())
            .filter(|v| !v.is_empty());

    // eBPF/BTF desteği
    let ebpf_btf_available = run_adb_command_timeout(
        serial,
        &[
            "shell",
            "test",
            "-f",
            "/sys/kernel/btf/vmlinux",
            "&&",
            "echo",
            "ok",
        ],
        timeout,
    )
    .map(|out| out.trim() == "ok")
    .unwrap_or(false);

    // /proc/kcore erişimi
    let kcore_available = run_adb_command_timeout(
        serial,
        &["shell", "test", "-r", "/proc/kcore", "&&", "echo", "ok"],
        timeout,
    )
    .map(|out| out.trim() == "ok")
    .unwrap_or(false);

    // RAM boyutu
    let ram_mb = run_adb_command_timeout(serial, &["shell", "cat", "/proc/meminfo"], timeout)
        .ok()
        .and_then(|out| {
            out.lines()
                .find(|l| l.starts_with("MemTotal:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
                .map(|kb| kb / 1024)
        });

    // Boş alan kontrolü (en az RAM kadar gerek)
    let storage_ok = {
        let needed_mb = ram_mb.unwrap_or(4096) + 512;
        run_adb_command_timeout(serial, &["shell", "df", "/data/local/tmp"], timeout)
            .ok()
            .and_then(|out| {
                out.lines()
                    .skip(1)
                    .next()
                    .and_then(|l| l.split_whitespace().nth(3))
                    .and_then(|v| v.parse::<u64>().ok())
            })
            .map(|kb_free| kb_free / 1024 >= needed_mb)
            .unwrap_or(false)
    };

    // SoC uyarısı
    let soc_warning = detect_risky_soc(serial, timeout);

    // Genel hazır olma durumu
    let ready = abi_supported && root_available && ebpf_btf_available;
    let reason = if !abi_supported {
        Some(format!(
            "Cihaz mimarisi ({}) Lemon tarafından desteklenmiyor. arm64-v8a veya x86_64 gerekli.",
            abi.as_deref().unwrap_or("bilinmiyor")
        ))
    } else if !root_available {
        Some(
            "Root erişimi bulunamadı. Lemon fiziksel RAM dump için su veya adb root gerektirir."
                .to_string(),
        )
    } else if !ebpf_btf_available {
        Some("Kernel eBPF/BTF desteği tespit edilemedi (/sys/kernel/btf/vmlinux). Lemon bu cihazda çalışmayabilir.".to_string())
    } else {
        None
    };

    LemonPreflight {
        abi,
        abi_supported,
        root_available,
        kernel_version,
        ebpf_btf_available,
        kcore_available,
        storage_ok,
        ram_mb,
        soc_warning,
        ready,
        reason,
    }
}

/// Lemon'ın desteklediği ABI listesini kontrol eder.
fn is_lemon_supported_abi(abi: &str) -> bool {
    matches!(abi, "arm64-v8a" | "x86_64" | "armeabi-v7a")
}

/// Exynos veya MediaTek SoC tespit edildiyse uyarı döndürür.
///
/// Bu SoC'larda EL2 (hypervisor) seviyesinde korumalar Lemon'ın
/// cihazı yeniden başlatmasına neden olabilir.
fn detect_risky_soc(serial: &str, timeout: Duration) -> Option<String> {
    let hardware = run_adb_command_timeout(serial, &["shell", "getprop", "ro.hardware"], timeout)
        .unwrap_or_default()
        .to_lowercase();
    let board =
        run_adb_command_timeout(serial, &["shell", "getprop", "ro.board.platform"], timeout)
            .unwrap_or_default()
            .to_lowercase();
    let combined = format!("{hardware} {board}");

    if combined.contains("exynos") || combined.contains("samsung") && combined.contains("s5e") {
        Some(
            "Exynos SoC tespit edildi. Lemon bu donanımda EL2 korumalarını tetikleyebilir ve cihazı yeniden başlatabilir. Dikkatli olun.".to_string(),
        )
    } else if combined.contains("mt") || combined.contains("mediatek") || combined.contains("helio")
    {
        Some(
            "MediaTek SoC tespit edildi. Lemon bu donanımda kararlılık sorunlarına neden olabilir."
                .to_string(),
        )
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Testler
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{RemoteAndroidEndpoint, RemoteEndpointKind, is_lemon_supported_abi};

    #[test]
    fn remote_endpoint_serial_format() {
        let ep = RemoteAndroidEndpoint {
            label: "Test".to_string(),
            host: "192.168.1.50".to_string(),
            port: 5555,
            kind: RemoteEndpointKind::TcpAdb,
        };
        assert_eq!(ep.serial(), "192.168.1.50:5555");
    }

    #[test]
    fn lemon_abi_support() {
        assert!(is_lemon_supported_abi("arm64-v8a"));
        assert!(is_lemon_supported_abi("x86_64"));
        assert!(!is_lemon_supported_abi("armeabi"));
        assert!(!is_lemon_supported_abi("mips"));
    }
}
