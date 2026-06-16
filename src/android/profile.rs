//! Android cihaz profilini, root durumunu ve sistem özelliklerini ADB ile algılar.
use super::adb::{first_non_empty, run_adb_command, run_adb_command_timeout};
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
/// Android cihazın model, build, root ve şifreleme durumunu temsil eder.
pub struct AndroidDeviceProfile {
    pub serial: String,
    pub product: Option<String>,
    pub model: Option<String>,
    pub device: Option<String>,
    pub abi: Option<String>,
    pub api_level: Option<u32>,
    pub build: Option<String>,
    pub fingerprint: Option<String>,
    pub security_patch: Option<String>,
    pub selinux: Option<String>,
    pub encryption: Option<String>,
    pub kernel_version: Option<String>,
    pub is_rooted: bool,
    pub adb_root: bool,
    pub su_available: bool,
}

const PROFILE_PROPS: &[&str] = &[
    "ro.product.name",
    "ro.product.model",
    "ro.product.device",
    "ro.product.cpu.abi",
    "ro.build.version.sdk",
    "ro.build.version.security_patch",
    "ro.build.display.id",
    "ro.build.fingerprint",
];

/// ADB üzerinden cihaz profilini okuyup root durumuyla birlikte döndürür.
pub fn detect_device_profile(serial: &str) -> Result<AndroidDeviceProfile, String> {
    let serial = serial.trim();
    if serial.is_empty() {
        return Err("serial is required".to_string());
    }

    let props = read_profile_props(serial);
    let (adb_root, su_available) = detect_root(serial);

    Ok(AndroidDeviceProfile {
        serial: serial.to_string(),
        product: prop(&props, "ro.product.name"),
        model: prop(&props, "ro.product.model"),
        device: prop(&props, "ro.product.device"),
        abi: prop(&props, "ro.product.cpu.abi"),
        api_level: prop(&props, "ro.build.version.sdk").and_then(|value| value.parse().ok()),
        build: prop(&props, "ro.build.display.id"),
        fingerprint: prop(&props, "ro.build.fingerprint"),
        security_patch: prop(&props, "ro.build.version.security_patch"),
        selinux: read_first_shell_line(serial, "getenforce").ok(),
        encryption: read_encryption_state(serial).ok(),
        kernel_version: read_first_shell_line(serial, "cat /proc/version").ok(),
        is_rooted: adb_root || su_available,
        adb_root,
        su_available,
    })
}

/// Profil için gerekli getprop anahtarlarını cihazdan okur.
fn read_profile_props(serial: &str) -> BTreeMap<String, String> {
    let mut props = BTreeMap::new();
    for key in PROFILE_PROPS {
        let command = format!("getprop {key}");
        if let Ok(value) = read_first_shell_line(serial, &command)
            && !value.is_empty()
        {
            props.insert((*key).to_string(), value);
        }
    }
    props
}

/// Boş olmayan getprop değerini map içinden güvenli şekilde alır.
fn prop(props: &BTreeMap<String, String>, key: &str) -> Option<String> {
    props
        .get(key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// ADB shell komutundan ilk anlamlı satırı döndürür.
fn read_first_shell_line(serial: &str, command: &str) -> Result<String, String> {
    run_adb_command_timeout(serial, &["shell", command], Duration::from_secs(8)).map(|output| {
        first_non_empty(&output)
            .unwrap_or_default()
            .trim()
            .to_string()
    })
}

/// adb shell id ve su -c id komutlarıyla root erişimini test eder.
fn detect_root(serial: &str) -> (bool, bool) {
    let adb_root = run_adb_command(serial, &["shell", "id"])
        .map(|output| output_has_root(&output))
        .unwrap_or(false);
    let su_available =
        run_adb_command_timeout(serial, &["shell", "su -c id"], Duration::from_secs(8))
            .map(|output| output_has_root(&output))
            .unwrap_or(false);
    (adb_root, su_available)
}

/// id çıktısında root UID bilgisini arar.
fn output_has_root(output: &str) -> bool {
    output.contains("uid=0(root)") || output.contains("uid=0 ")
}

/// Android şifreleme tipini property ve mount çıktılarından okur.
fn read_encryption_state(serial: &str) -> Result<String, String> {
    let output = run_adb_command_timeout(
        serial,
        &[
            "shell",
            "getprop ro.crypto.type; getprop ro.crypto.state; cat /proc/mounts | grep -E 'encrypt|fbe|fde' 2>/dev/null || true",
        ],
        Duration::from_secs(8),
    )?;
    Ok(parse_encryption_state(&output))
}

/// Ham şifreleme çıktısını none/FBE/FDE/ encrypted etiketine indirger.
fn parse_encryption_state(output: &str) -> String {
    let lower = output.to_lowercase();
    if lower.contains("unencrypted") {
        "none".to_string()
    } else if lower.contains("file") || lower.contains("fbe") {
        "FBE".to_string()
    } else if lower.contains("block") || lower.contains("fde") {
        "FDE".to_string()
    } else if lower.contains("encrypted") {
        "encrypted".to_string()
    } else {
        "none".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{output_has_root, parse_encryption_state};

    #[test]
    fn parses_android_encryption_state() {
        assert_eq!(parse_encryption_state("file\nencrypted\n"), "FBE");
        assert_eq!(parse_encryption_state("block\nencrypted\n"), "FDE");
        assert_eq!(parse_encryption_state("unencrypted\n"), "none");
    }

    #[test]
    fn detects_root_id_output() {
        assert!(output_has_root("uid=0(root) gid=0(root) groups=0(root)"));
        assert!(!output_has_root("uid=2000(shell) gid=2000(shell)"));
    }
}
