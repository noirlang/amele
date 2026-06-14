use serde::Serialize;
use std::io;
use std::process::{Command, Stdio};
use std::time::Duration;

#[derive(Debug, Clone, Serialize)]
pub struct AdbStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AndroidDevice {
    pub serial: String,
    pub state: String,
    pub model: Option<String>,
    pub product: Option<String>,
    pub device: Option<String>,
    pub transport_id: Option<String>,
    pub raw: String,
}

pub fn adb_status() -> AdbStatus {
    match Command::new("adb").arg("version").output() {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let version = parse_adb_version(&stdout);
            let path = parse_adb_path(&stdout).or_else(|| command_path("adb"));
            let detail = version
                .clone()
                .or_else(|| path.clone())
                .unwrap_or_else(|| "adb".to_string());
            AdbStatus {
                installed: true,
                version,
                path,
                message: format!("ADB bulundu: {}", detail),
            }
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let message = first_non_empty(&stderr)
                .or_else(|| first_non_empty(&stdout))
                .unwrap_or_else(|| "ADB calistirildi ama surum alinamadi".to_string());
            AdbStatus {
                installed: false,
                version: None,
                path: command_path("adb"),
                message,
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => AdbStatus {
            installed: false,
            version: None,
            path: None,
            message: "ADB bulunamadi".to_string(),
        },
        Err(err) => AdbStatus {
            installed: false,
            version: None,
            path: None,
            message: format!("ADB kontrol edilemedi: {err}"),
        },
    }
}

pub fn list_devices() -> Result<Vec<AndroidDevice>, String> {
    let output = Command::new("adb")
        .args(["devices", "-l"])
        .output()
        .map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                "ADB bulunamadi".to_string()
            } else {
                format!("ADB cihaz listesi alinamadi: {err}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(first_non_empty(&stderr)
            .or_else(|| first_non_empty(&stdout))
            .unwrap_or_else(|| "ADB cihaz listesi basarisiz oldu".to_string()));
    }

    Ok(parse_adb_devices(&String::from_utf8_lossy(&output.stdout)))
}

pub(super) fn run_adb_command(serial: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("adb")
        .args(["-s", serial])
        .args(args)
        .output()
        .map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                "ADB bulunamadi".to_string()
            } else {
                format!("ADB komutu calistirilamadi: {err}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = first_non_empty(&stderr)
            .or_else(|| first_non_empty(&stdout))
            .unwrap_or_else(|| "ADB komutu basarisiz oldu".to_string());
        return Err(detail);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(super) fn run_adb_command_timeout(
    serial: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<String, String> {
    let mut child = Command::new("adb")
        .args(["-s", serial])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                "ADB bulunamadi".to_string()
            } else {
                format!("ADB komutu calistirilamadi: {err}")
            }
        })?;

    let started = std::time::Instant::now();
    loop {
        if let Ok(Some(_)) = child.try_wait() {
            let output = child
                .wait_with_output()
                .map_err(|err| format!("ADB komutu sonucu alinamadi: {err}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                return Err(first_non_empty(&stderr)
                    .or_else(|| first_non_empty(&stdout))
                    .unwrap_or_else(|| "ADB komutu basarisiz oldu".to_string()));
            }
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!(
                "ADB komutu zaman asimina ugradi: {}s",
                timeout.as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

pub(super) fn run_adb_file_command(serial: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", serial])
        .args(args)
        .output()
        .map_err(|err| {
            if err.kind() == io::ErrorKind::NotFound {
                "ADB bulunamadi".to_string()
            } else {
                format!("ADB komutu calistirilamadi: {err}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = first_non_empty(&stderr)
            .or_else(|| first_non_empty(&stdout))
            .unwrap_or_else(|| "ADB dosya komutu basarisiz oldu".to_string());
        return Err(detail);
    }
    Ok(())
}

pub(super) fn run_adb_file_command_timeout(
    serial: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<(), String> {
    run_adb_command_timeout(serial, args, timeout).map(|_| ())
}

fn parse_adb_version(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .find(|line| {
            line.starts_with("Android Debug Bridge version") || line.starts_with("Version ")
        })
        .map(ToOwned::to_owned)
}

fn parse_adb_path(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("Installed as ").map(ToOwned::to_owned))
}

fn command_path(command: &str) -> Option<String> {
    let output = if cfg!(windows) {
        Command::new("where").arg(command).output().ok()?
    } else {
        Command::new("sh")
            .args(["-c", &format!("command -v {command}")])
            .output()
            .ok()?
    };
    if !output.status.success() {
        return None;
    }
    first_non_empty(&String::from_utf8_lossy(&output.stdout))
}

pub(super) fn first_non_empty(value: &str) -> Option<String> {
    value
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn parse_adb_devices(output: &str) -> Vec<AndroidDevice> {
    output.lines().filter_map(parse_adb_device_line).collect()
}

fn parse_adb_device_line(line: &str) -> Option<AndroidDevice> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with("List of devices")
        || trimmed.starts_with("* daemon")
    {
        return None;
    }

    let mut parts = trimmed.split_whitespace();
    let serial = parts.next()?.to_string();
    let state = parts.next().unwrap_or("unknown").to_string();
    let mut model = None;
    let mut product = None;
    let mut device = None;
    let mut transport_id = None;

    for part in parts {
        let Some((key, value)) = part.split_once(':') else {
            continue;
        };
        match key {
            "model" => model = Some(value.replace('_', " ")),
            "product" => product = Some(value.to_string()),
            "device" => device = Some(value.to_string()),
            "transport_id" => transport_id = Some(value.to_string()),
            _ => {}
        }
    }

    Some(AndroidDevice {
        serial,
        state,
        model,
        product,
        device,
        transport_id,
        raw: trimmed.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_adb_devices_output() {
        let devices = parse_adb_devices(
            "List of devices attached\n\
             emulator-5554 device product:sdk_gphone_x86 model:sdk_gphone_x86 device:generic_x86 transport_id:1\n\
             R58M123ABC unauthorized usb:1-2 product:a52 model:SM_A525F device:a52q transport_id:2\n",
        );

        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].serial, "emulator-5554");
        assert_eq!(devices[0].state, "device");
        assert_eq!(devices[0].model.as_deref(), Some("sdk gphone x86"));
        assert_eq!(devices[1].state, "unauthorized");
        assert_eq!(devices[1].transport_id.as_deref(), Some("2"));
    }
}
