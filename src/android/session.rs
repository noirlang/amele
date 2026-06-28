//! Android hedef oturumunu ve ADB taşıma tipini standartlaştırır.
use super::adb::run_adb_command_timeout;
use super::profile::AndroidDeviceProfile;
use serde::Serialize;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
/// Android cihazına hangi ADB taşıma yolu ile erişildiğini belirtir.
pub enum AndroidTransportKind {
    Usb,
    TcpAdb,
    Mesh,
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
/// ADB hedefinin bağlantı tipi ve kullanıcıya gösterilecek açıklamasıdır.
pub struct AndroidTransport {
    pub kind: AndroidTransportKind,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
/// Bir edinim sırasında kullanılan cihaz, profil ve ADB durumunu birlikte taşır.
pub struct AndroidSession {
    pub serial: String,
    pub created_at: String,
    pub adb_state: Option<String>,
    pub connected: bool,
    pub transport: AndroidTransport,
    pub device_profile: AndroidDeviceProfile,
}

/// Cihaz profili üzerinden edinim oturumu oluşturur.
pub fn build_android_session(serial: &str, device_profile: AndroidDeviceProfile) -> AndroidSession {
    let adb_state = run_adb_command_timeout(serial, &["get-state"], Duration::from_secs(5))
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let connected = adb_state.as_deref() == Some("device");

    AndroidSession {
        serial: serial.trim().to_string(),
        created_at: chrono::Local::now().to_rfc3339(),
        adb_state,
        connected,
        transport: detect_transport(serial),
        device_profile,
    }
}

/// Oturum bilgisini Android çıktı klasörüne yazar.
pub fn write_android_session(output_dir: &Path, session: &AndroidSession) -> Result<(), String> {
    let path = output_dir.join("android_session.json");
    let content = serde_json::to_vec_pretty(session)
        .map_err(|err| format!("Android oturumu JSON'a cevrilemedi: {err}"))?;
    std::fs::write(&path, content).map_err(|err| format!("Android oturumu yazilamadi: {err}"))
}

/// Serial biçiminden USB, TCP ADB veya MESH taşımasını tahmin eder.
pub fn detect_transport(serial: &str) -> AndroidTransport {
    let serial = serial.trim();
    if serial.starts_with("mesh://") || serial.starts_with("mesh:") {
        AndroidTransport {
            kind: AndroidTransportKind::Mesh,
            label: "MESH remote ADB".to_string(),
        }
    } else if serial.contains(':') {
        AndroidTransport {
            kind: AndroidTransportKind::TcpAdb,
            label: "TCP/IP ADB".to_string(),
        }
    } else if !serial.is_empty() {
        AndroidTransport {
            kind: AndroidTransportKind::Usb,
            label: "USB ADB".to_string(),
        }
    } else {
        AndroidTransport {
            kind: AndroidTransportKind::Unknown,
            label: "Bilinmeyen ADB hedefi".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AndroidTransportKind, detect_transport};

    #[test]
    fn detects_tcp_adb_transport() {
        let transport = detect_transport("192.168.1.20:5555");
        assert_eq!(transport.kind, AndroidTransportKind::TcpAdb);
    }

    #[test]
    fn detects_usb_transport() {
        let transport = detect_transport("R5CT123ABC");
        assert_eq!(transport.kind, AndroidTransportKind::Usb);
    }
}
