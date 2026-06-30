//! WireGuard VPN yapılandırmasını doğrular ve config dosyasına dönüştürür.
use crate::error::{AmeleError, AmeleResult, HataKodu};
use crate::logging::{LogLevel, runtime_log};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(windows))]
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// WireGuard bağlantısının aktif config ve arayüz durumunu tutar.
pub struct WireGuardManager {
    pub interface_name: String,
    pub config_file: Option<PathBuf>,
    pub active: bool,
}

impl Default for WireGuardManager {
    fn default() -> Self {
        Self {
            interface_name: "wg0".to_string(),
            config_file: None,
            active: false,
        }
    }
}

impl WireGuardManager {
    /// Varsayılan WireGuard manager oluşturur.
    pub fn new() -> Self {
        Self::default()
    }

    /// wg-quick up ile WireGuard bağlantısını başlatır.
    pub fn start(&mut self, config_file: impl AsRef<Path>) -> AmeleResult<()> {
        if self.active {
            runtime_log(LogLevel::Info, "wireguard", "WireGuard tüneli zaten aktif.");
            return Ok(());
        }

        #[cfg(windows)]
        {
            let _ = config_file;
            let err = AmeleError::new(
                HataKodu::Genel,
                "Windows surumunde WireGuard otomatik baslatma desteklenmiyor",
            );
            runtime_log(
                LogLevel::Error,
                "wireguard",
                format!("Baslatma hatasi (Windows): {:?}", err),
            );
            return Err(err);
        }

        #[cfg(not(windows))]
        {
            let config_file = config_file.as_ref().to_path_buf();
            runtime_log(
                LogLevel::Info,
                "wireguard",
                format!(
                    "WireGuard tüneli baslatiliyor (wg-quick up): {}",
                    config_file.display()
                ),
            );
            let status = Command::new("wg-quick")
                .arg("up")
                .arg(&config_file)
                .status()
                .map_err(|err| {
                    let w_err = AmeleError::io(HataKodu::Genel, "wg-quick baslatilamadi", err);
                    runtime_log(
                        LogLevel::Error,
                        "wireguard",
                        format!("wg-quick spawn hatasi: {:?}", w_err),
                    );
                    w_err
                })?;
            if !status.success() {
                let err = AmeleError::new(
                    HataKodu::Genel,
                    format!("WireGuard baslatilamadi: {status}"),
                );
                runtime_log(
                    LogLevel::Error,
                    "wireguard",
                    format!("wg-quick hatasi: {:?}", err),
                );
                return Err(err);
            }
            self.config_file = Some(config_file);
            self.active = true;
            runtime_log(
                LogLevel::Info,
                "wireguard",
                "WireGuard tüneli basariyla baslatildi.",
            );
            Ok(())
        }
    }

    /// wg-quick down ile aktif WireGuard bağlantısını durdurur.
    pub fn stop(&mut self) -> AmeleResult<()> {
        if !self.active {
            runtime_log(
                LogLevel::Info,
                "wireguard",
                "WireGuard tüneli aktif degil, durdurma atlandi.",
            );
            return Ok(());
        }

        #[cfg(windows)]
        {
            self.active = false;
            let err = AmeleError::new(
                HataKodu::Genel,
                "Windows surumunde WireGuard otomatik durdurma desteklenmiyor",
            );
            runtime_log(
                LogLevel::Error,
                "wireguard",
                format!("Durdurma hatasi (Windows): {:?}", err),
            );
            return Err(err);
        }

        #[cfg(not(windows))]
        {
            let config = self.config_file.clone().ok_or_else(|| {
                let err = AmeleError::new(HataKodu::Genel, "WireGuard config bilinmiyor");
                runtime_log(LogLevel::Error, "wireguard", format!("Hata: {:?}", err));
                err
            })?;
            runtime_log(
                LogLevel::Info,
                "wireguard",
                format!(
                    "WireGuard tüneli durduruluyor (wg-quick down): {}",
                    config.display()
                ),
            );
            let status = Command::new("wg-quick")
                .arg("down")
                .arg(&config)
                .status()
                .map_err(|err| {
                    let w_err = AmeleError::io(HataKodu::Genel, "wg-quick durdurulamadi", err);
                    runtime_log(
                        LogLevel::Error,
                        "wireguard",
                        format!("wg-quick spawn hatasi: {:?}", w_err),
                    );
                    w_err
                })?;
            self.active = false;
            if !status.success() {
                let err = AmeleError::new(
                    HataKodu::Genel,
                    format!("WireGuard durdurulamadi: {status}"),
                );
                runtime_log(
                    LogLevel::Error,
                    "wireguard",
                    format!("wg-quick hatasi: {:?}", err),
                );
                return Err(err);
            }
            runtime_log(
                LogLevel::Info,
                "wireguard",
                "WireGuard tüneli basariyla durduruldu.",
            );
            Ok(())
        }
    }

    /// Manager'ın aktif bağlantı durumunu döndürür.
    pub fn is_active(&self) -> bool {
        self.active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// WireGuard config dosyasına yazılacak Interface ve Peer alanlarını taşır.
pub struct WireGuardConfig<'a> {
    pub private_key: &'a str,
    pub public_key: &'a str,
    pub endpoint: &'a str,
    pub allowed_ips: &'a str,
    pub address: &'a str,
    pub dns: &'a str,
    pub keepalive: u16,
}

impl Default for WireGuardConfig<'_> {
    fn default() -> Self {
        Self {
            private_key: "YOUR_PRIVATE_KEY",
            public_key: "SERVER_PUBLIC_KEY",
            endpoint: "192.168.1.1:51820",
            allowed_ips: "0.0.0.0/0, ::/0",
            address: "10.0.0.2/24",
            dns: "1.1.1.1",
            keepalive: 25,
        }
    }
}

/// WireGuard config dosyasını diske yazar.
pub fn create_config(path: impl AsRef<Path>, config: &WireGuardConfig<'_>) -> AmeleResult<PathBuf> {
    let path = path.as_ref();
    runtime_log(
        LogLevel::Info,
        "wireguard",
        format!(
            "WireGuard yapilandirma dosyasi olusturuluyor: {}",
            path.display()
        ),
    );
    if let Some(parent) = path.parent() {
        runtime_log(
            LogLevel::Debug,
            "wireguard",
            format!(
                "WireGuard yapilandirma klasoru olusturuluyor: {}",
                parent.display()
            ),
        );
        fs::create_dir_all(parent).map_err(|err| {
            let w_err = AmeleError::io(
                HataKodu::DosyaYazma,
                "WireGuard config klasoru olusturulamadi",
                err,
            );
            runtime_log(
                LogLevel::Error,
                "wireguard",
                format!("Klasor olusturma hatasi: {:?}", w_err),
            );
            w_err
        })?;
    }

    let content = format!(
        "[Interface]\n\
         PrivateKey = {}\n\
         Address = {}\n\
         DNS = {}\n\n\
         [Peer]\n\
         PublicKey = {}\n\
         Endpoint = {}\n\
         AllowedIPs = {}\n\
         PersistentKeepalive = {}\n",
        fallback(config.private_key, "YOUR_PRIVATE_KEY"),
        fallback(config.address, "10.0.0.2/24"),
        fallback(config.dns, "1.1.1.1"),
        fallback(config.public_key, "SERVER_PUBLIC_KEY"),
        fallback(config.endpoint, "192.168.1.1:51820"),
        fallback(config.allowed_ips, "0.0.0.0/0, ::/0"),
        if config.keepalive > 0 {
            config.keepalive
        } else {
            25
        }
    );
    runtime_log(
        LogLevel::Debug,
        "wireguard",
        format!(
            "WireGuard yapilandirma verileri yaziliyor: {}",
            path.display()
        ),
    );
    fs::write(path, content).map_err(|err| {
        let w_err = AmeleError::io(HataKodu::DosyaYazma, "WireGuard config yazilamadi", err);
        runtime_log(
            LogLevel::Error,
            "wireguard",
            format!("Dosya yazma hatasi: {:?}", w_err),
        );
        w_err
    })?;
    runtime_log(
        LogLevel::Info,
        "wireguard",
        "WireGuard yapilandirma dosyasi basariyla yazildi.",
    );
    Ok(path.to_path_buf())
}

/// Boş config değerlerinde güvenli placeholder varsayılanı kullanır.
fn fallback<'a>(value: &'a str, default: &'a str) -> &'a str {
    if value.is_empty() { default } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_wireguard_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wg0.conf");
        create_config(&path, &WireGuardConfig::default()).unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.contains("[Interface]"));
        assert!(content.contains("[Peer]"));
    }
}
