use serde::Serialize;
use std::io;
use std::process::Command;

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

fn first_non_empty(value: &str) -> Option<String> {
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

// ---------------------------------------------------------------------------
// Logical acquisition
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct AcquisitionItem {
    pub category: String,
    pub file_name: String,
    pub size: u64,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogicalAcquisitionResult {
    pub output_dir: std::path::PathBuf,
    pub items: Vec<AcquisitionItem>,
    pub total_bytes: u64,
    pub sha256: Option<String>,
    pub errors: Vec<String>,
}

const LOGICAL_STEPS: &[(&str, &str)] = &[
    ("device_info", "device_info.txt"),
    ("packages", "packages.txt"),
    ("logcat", "logcat.txt"),
    ("dumpsys_battery", "dumpsys_battery.txt"),
    ("dumpsys_wifi", "dumpsys_wifi.txt"),
    ("dumpsys_bluetooth", "dumpsys_bluetooth.txt"),
    ("dumpsys_usagestats", "dumpsys_usagestats.txt"),
    ("dumpsys_account", "dumpsys_account.txt"),
    ("dumpsys_connectivity", "dumpsys_connectivity.txt"),
    ("dumpsys_notification", "dumpsys_notification.txt"),
    ("network_info", "network_info.txt"),
    ("processes", "processes.txt"),
    ("disk_usage", "disk_usage.txt"),
    ("screenshot", "screenshot.png"),
    ("whatsapp_media", "whatsapp_media"),
    ("telegram_media", "telegram_media"),
    ("app_media", "app_media"),
    ("bugreport", "bugreport.zip"),
    ("shared_storage", "shared_storage"),
];

/// Run an ADB shell command targeting `serial` and return stdout as a String.
fn run_adb_command(serial: &str, args: &[&str]) -> Result<String, String> {
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

/// Run an ADB command that writes output to a file (e.g. `adb pull`, `adb bugreport`).
fn run_adb_file_command(serial: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new("adb")
        .args(["-s", serial])
        .args(args)
        .output()
        .map_err(|err| format!("ADB komutu calistirilamadi: {err}"))?;

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

fn collect_shell_output(
    serial: &str,
    category: &str,
    file_name: &str,
    shell_args: &[&str],
    dir: &std::path::Path,
) -> AcquisitionItem {
    match run_adb_command(serial, shell_args) {
        Ok(content) => {
            let path = dir.join(file_name);
            match std::fs::write(&path, &content) {
                Ok(()) => AcquisitionItem {
                    category: category.to_string(),
                    file_name: file_name.to_string(),
                    size: content.len() as u64,
                    success: true,
                    error: None,
                },
                Err(err) => AcquisitionItem {
                    category: category.to_string(),
                    file_name: file_name.to_string(),
                    size: 0,
                    success: false,
                    error: Some(format!("Dosya yazilamadi: {err}")),
                },
            }
        }
        Err(err) => AcquisitionItem {
            category: category.to_string(),
            file_name: file_name.to_string(),
            size: 0,
            success: false,
            error: Some(err),
        },
    }
}

fn collect_device_info(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    collect_shell_output(
        serial,
        "device_info",
        "device_info.txt",
        &["shell", "getprop"],
        dir,
    )
}

fn collect_packages(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    collect_shell_output(
        serial,
        "packages",
        "packages.txt",
        &["shell", "pm", "list", "packages", "-f"],
        dir,
    )
}

fn collect_logcat(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    collect_shell_output(serial, "logcat", "logcat.txt", &["logcat", "-d"], dir)
}

fn collect_dumpsys(
    serial: &str,
    service: &str,
    file_name: &str,
    dir: &std::path::Path,
) -> AcquisitionItem {
    let category = format!("dumpsys_{service}");
    collect_shell_output(
        serial,
        &category,
        file_name,
        &["shell", "dumpsys", service],
        dir,
    )
}

fn collect_bugreport(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let target = dir.to_string_lossy().into_owned();
    match run_adb_file_command(serial, &["bugreport", &target]) {
        Ok(()) => {
            // `adb bugreport <dir>` creates a file named bugreport-*.zip in <dir>.
            // Find it and report its size.
            let mut found: Option<std::path::PathBuf> = None;
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with("bugreport") && name.ends_with(".zip") {
                        found = Some(entry.path());
                        break;
                    }
                }
            }
            let (file_name, size) = match found {
                Some(path) => {
                    let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "bugreport.zip".to_string());
                    (name, size)
                }
                None => ("bugreport.zip".to_string(), 0),
            };
            AcquisitionItem {
                category: "bugreport".to_string(),
                file_name,
                size,
                success: true,
                error: None,
            }
        }
        Err(err) => AcquisitionItem {
            category: "bugreport".to_string(),
            file_name: "bugreport.zip".to_string(),
            size: 0,
            success: false,
            error: Some(err),
        },
    }
}

fn collect_shared_storage(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let target = dir.join("shared_storage");
    let _ = std::fs::create_dir_all(&target);
    let target_str = target.to_string_lossy().into_owned();
    match run_adb_file_command(serial, &["pull", "/sdcard/", &target_str]) {
        Ok(()) => {
            let size = dir_size(&target);
            AcquisitionItem {
                category: "shared_storage".to_string(),
                file_name: "shared_storage".to_string(),
                size,
                success: true,
                error: None,
            }
        }
        Err(err) => {
            // partial pull is still useful — report what we got
            let size = dir_size(&target);
            AcquisitionItem {
                category: "shared_storage".to_string(),
                file_name: "shared_storage".to_string(),
                size,
                success: size > 0,
                error: if size > 0 {
                    Some(format!("Kismi basarili: {err}"))
                } else {
                    Some(err)
                },
            }
        }
    }
}

fn collect_notification_history(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let mut content = String::new();

    // 1. Standard dumpsys notification (always available)
    content.push_str("=== dumpsys notification ===\n");
    match run_adb_command(serial, &["shell", "dumpsys", "notification"]) {
        Ok(output) => content.push_str(&output),
        Err(err) => content.push_str(&format!("Hata: {err}")),
    }
    content.push_str("\n\n");

    // 2. Check if notification history feature is enabled
    let history_enabled = run_adb_command(
        serial,
        &[
            "shell",
            "settings",
            "get",
            "secure",
            "notification_history_enabled",
        ],
    )
    .map(|v| v.trim() == "1")
    .unwrap_or(false);

    if history_enabled {
        content.push_str("=== Bildirim gecmisi etkin — notification history ===\n");
        match run_adb_command(serial, &["shell", "cmd", "notification", "dump_history"]) {
            Ok(output) => content.push_str(&output),
            Err(err) => content.push_str(&format!("Hata: {err}")),
        }
    } else {
        content.push_str(
            "=== Bildirim gecmisi etkin degil (settings > notification_history_enabled != 1) ===\n",
        );
    }

    let path = dir.join("dumpsys_notification.txt");
    match std::fs::write(&path, &content) {
        Ok(()) => AcquisitionItem {
            category: "dumpsys_notification".to_string(),
            file_name: "dumpsys_notification.txt".to_string(),
            size: content.len() as u64,
            success: true,
            error: None,
        },
        Err(err) => AcquisitionItem {
            category: "dumpsys_notification".to_string(),
            file_name: "dumpsys_notification.txt".to_string(),
            size: 0,
            success: false,
            error: Some(format!("Dosya yazilamadi: {err}")),
        },
    }
}

fn collect_network_info(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let commands = [
        ("=== ip addr ===", vec!["shell", "ip", "addr"]),
        ("=== ip route ===", vec!["shell", "ip", "route"]),
        ("=== netstat ===", vec!["shell", "netstat", "-tlnp"]),
        ("=== ip neigh ===", vec!["shell", "ip", "neigh"]),
    ];
    let mut content = String::new();
    for (header, args) in &commands {
        content.push_str(header);
        content.push('\n');
        match run_adb_command(serial, args.as_slice()) {
            Ok(output) => content.push_str(&output),
            Err(err) => content.push_str(&format!("Hata: {err}")),
        }
        content.push_str("\n\n");
    }
    let path = dir.join("network_info.txt");
    match std::fs::write(&path, &content) {
        Ok(()) => AcquisitionItem {
            category: "network_info".to_string(),
            file_name: "network_info.txt".to_string(),
            size: content.len() as u64,
            success: true,
            error: None,
        },
        Err(err) => AcquisitionItem {
            category: "network_info".to_string(),
            file_name: "network_info.txt".to_string(),
            size: 0,
            success: false,
            error: Some(format!("Dosya yazilamadi: {err}")),
        },
    }
}

fn collect_processes(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    collect_shell_output(
        serial,
        "processes",
        "processes.txt",
        &["shell", "ps", "-A"],
        dir,
    )
}

fn collect_disk_usage(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    collect_shell_output(
        serial,
        "disk_usage",
        "disk_usage.txt",
        &["shell", "df", "-h"],
        dir,
    )
}

fn collect_screenshot(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let remote_path = "/sdcard/worm_screenshot.png";
    // Take screenshot on device
    if let Err(err) = run_adb_command(serial, &["shell", "screencap", "-p", remote_path]) {
        return AcquisitionItem {
            category: "screenshot".to_string(),
            file_name: "screenshot.png".to_string(),
            size: 0,
            success: false,
            error: Some(err),
        };
    }
    let local_path = dir.join("screenshot.png");
    let local_str = local_path.to_string_lossy().into_owned();
    let result = run_adb_file_command(serial, &["pull", remote_path, &local_str]);
    // Clean up remote file
    let _ = run_adb_command(serial, &["shell", "rm", "-f", remote_path]);
    match result {
        Ok(()) => {
            let size = std::fs::metadata(&local_path).map(|m| m.len()).unwrap_or(0);
            AcquisitionItem {
                category: "screenshot".to_string(),
                file_name: "screenshot.png".to_string(),
                size,
                success: true,
                error: None,
            }
        }
        Err(err) => AcquisitionItem {
            category: "screenshot".to_string(),
            file_name: "screenshot.png".to_string(),
            size: 0,
            success: false,
            error: Some(err),
        },
    }
}

/// Pull an app media directory from `/sdcard/Android/media/<package>/`.
fn collect_app_media_dir(
    serial: &str,
    category: &str,
    package: &str,
    target_name: &str,
    dir: &std::path::Path,
) -> AcquisitionItem {
    let remote = format!("/sdcard/Android/media/{package}/");
    let target = dir.join(target_name);
    let _ = std::fs::create_dir_all(&target);
    let target_str = target.to_string_lossy().into_owned();
    match run_adb_file_command(serial, &["pull", &remote, &target_str]) {
        Ok(()) => {
            let size = dir_size(&target);
            AcquisitionItem {
                category: category.to_string(),
                file_name: target_name.to_string(),
                size,
                success: true,
                error: None,
            }
        }
        Err(err) => {
            let size = dir_size(&target);
            if size > 0 {
                AcquisitionItem {
                    category: category.to_string(),
                    file_name: target_name.to_string(),
                    size,
                    success: true,
                    error: Some(format!("Kismi basarili: {err}")),
                }
            } else {
                AcquisitionItem {
                    category: category.to_string(),
                    file_name: target_name.to_string(),
                    size: 0,
                    success: false,
                    error: Some(err),
                }
            }
        }
    }
}

fn collect_whatsapp_media(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    collect_app_media_dir(
        serial,
        "whatsapp_media",
        "com.whatsapp",
        "whatsapp_media",
        dir,
    )
}

fn collect_telegram_media(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    collect_app_media_dir(
        serial,
        "telegram_media",
        "org.telegram.messenger",
        "telegram_media",
        dir,
    )
}

/// Collect media from well-known messaging and social apps.
/// Pulls from /sdcard/Android/media/<package>/ for a list of common apps.
fn collect_app_media(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let apps: &[(&str, &str)] = &[
        ("com.whatsapp.w4b", "whatsapp_business"),
        ("com.instagram.android", "instagram"),
        ("com.facebook.orca", "messenger"),
        ("com.viber.voip", "viber"),
        ("com.google.android.apps.messaging", "google_messages"),
    ];
    let app_media_dir = dir.join("app_media");
    let _ = std::fs::create_dir_all(&app_media_dir);
    let mut total_size = 0_u64;
    let mut found_any = false;

    for (package, folder_name) in apps {
        let remote = format!("/sdcard/Android/media/{package}/");
        let target = app_media_dir.join(folder_name);
        let _ = std::fs::create_dir_all(&target);
        let target_str = target.to_string_lossy().into_owned();
        match run_adb_file_command(serial, &["pull", &remote, &target_str]) {
            Ok(()) => {
                let size = dir_size(&target);
                if size > 0 {
                    found_any = true;
                    total_size += size;
                }
            }
            Err(_) => {
                let size = dir_size(&target);
                if size > 0 {
                    found_any = true;
                    total_size += size;
                } else {
                    // remove empty dir
                    let _ = std::fs::remove_dir(&target);
                }
            }
        }
    }

    AcquisitionItem {
        category: "app_media".to_string(),
        file_name: "app_media".to_string(),
        size: total_size,
        success: found_any,
        error: if found_any {
            None
        } else {
            Some("Hicbir uygulama medyasi bulunamadi".to_string())
        },
    }
}

fn dir_size(path: &std::path::Path) -> u64 {
    let mut total = 0_u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                total += dir_size(&entry_path);
            } else {
                total += std::fs::metadata(&entry_path).map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

fn write_manifest(
    dir: &std::path::Path,
    items: &[AcquisitionItem],
    serial: &str,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    let manifest = serde_json::json!({
        "serial": serial,
        "timestamp": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "items": items,
    });
    let content = serde_json::to_string_pretty(&manifest)
        .map_err(|err| format!("Manifest olusturulamadi: {err}"))?;

    let manifest_path = dir.join("manifest.json");
    std::fs::write(&manifest_path, &content)
        .map_err(|err| format!("Manifest yazilamadi: {err}"))?;

    let hash = crate::hash::to_hex(&Sha256::digest(content.as_bytes()));
    let sidecar = dir.join("manifest.json.sha256");
    let _ = std::fs::write(&sidecar, format!("{hash}  manifest.json\n"));
    Ok(hash)
}

/// Run a full logical acquisition for the given device.
///
/// `progress` is called with `(completed_step, total_steps, category_name)`.
/// `cancelled` is called before each step — return `true` to abort.
pub fn logical_acquisition<F, C>(
    serial: &str,
    output_dir: &std::path::Path,
    mut progress: F,
    cancelled: C,
) -> Result<LogicalAcquisitionResult, String>
where
    F: FnMut(u32, u32, &str),
    C: Fn() -> bool,
{
    std::fs::create_dir_all(output_dir)
        .map_err(|err| format!("Cikti dizini olusturulamadi: {err}"))?;

    let total = LOGICAL_STEPS.len() as u32;
    let mut items = Vec::with_capacity(LOGICAL_STEPS.len());
    let mut errors = Vec::new();

    for (step_index, (category, _file_name)) in LOGICAL_STEPS.iter().enumerate() {
        if cancelled() {
            errors.push("Kullanici tarafindan iptal edildi".to_string());
            break;
        }

        progress(step_index as u32, total, category);

        let item = match *category {
            "device_info" => collect_device_info(serial, output_dir),
            "packages" => collect_packages(serial, output_dir),
            "logcat" => collect_logcat(serial, output_dir),
            "dumpsys_battery" => {
                collect_dumpsys(serial, "battery", "dumpsys_battery.txt", output_dir)
            }
            "dumpsys_wifi" => collect_dumpsys(serial, "wifi", "dumpsys_wifi.txt", output_dir),
            "dumpsys_bluetooth" => collect_dumpsys(
                serial,
                "bluetooth_manager",
                "dumpsys_bluetooth.txt",
                output_dir,
            ),
            "dumpsys_usagestats" => {
                collect_dumpsys(serial, "usagestats", "dumpsys_usagestats.txt", output_dir)
            }
            "dumpsys_account" => {
                collect_dumpsys(serial, "account", "dumpsys_account.txt", output_dir)
            }
            "dumpsys_connectivity" => collect_dumpsys(
                serial,
                "connectivity",
                "dumpsys_connectivity.txt",
                output_dir,
            ),
            "dumpsys_notification" => collect_notification_history(serial, output_dir),
            "network_info" => collect_network_info(serial, output_dir),
            "processes" => collect_processes(serial, output_dir),
            "disk_usage" => collect_disk_usage(serial, output_dir),
            "screenshot" => collect_screenshot(serial, output_dir),
            "whatsapp_media" => collect_whatsapp_media(serial, output_dir),
            "telegram_media" => collect_telegram_media(serial, output_dir),
            "app_media" => collect_app_media(serial, output_dir),
            "bugreport" => collect_bugreport(serial, output_dir),
            "shared_storage" => collect_shared_storage(serial, output_dir),
            _ => continue,
        };

        if !item.success {
            if let Some(err) = &item.error {
                errors.push(format!("{category}: {err}"));
            }
        }
        items.push(item);
    }

    // Final progress tick
    progress(total, total, "manifest");

    let total_bytes = items.iter().map(|i| i.size).sum();
    let sha256 = write_manifest(output_dir, &items, serial).ok();

    Ok(LogicalAcquisitionResult {
        output_dir: output_dir.to_path_buf(),
        items,
        total_bytes,
        sha256,
        errors,
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
