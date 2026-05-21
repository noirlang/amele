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
    ("dumpsys_telephony", "dumpsys_telephony.txt"),
    ("dumpsys_location", "dumpsys_location.txt"),
    ("dumpsys_netstats", "dumpsys_netstats.txt"),
    ("dumpsys_activity", "dumpsys_activity.txt"),
    ("dumpsys_meminfo", "dumpsys_meminfo.txt"),
    ("dumpsys_appops", "dumpsys_appops.txt"),
    ("dumpsys_package", "dumpsys_package.txt"),
    ("dumpsys_diskstats", "dumpsys_diskstats.txt"),
    ("dumpsys_deviceidle", "dumpsys_deviceidle.txt"),
    ("dumpsys_alarm", "dumpsys_alarm.txt"),
    ("dumpsys_jobscheduler", "dumpsys_jobscheduler.txt"),
    ("dumpsys_procstats", "dumpsys_procstats.txt"),
    ("dumpsys_sensorservice", "dumpsys_sensorservice.txt"),
    ("dumpsys_power", "dumpsys_power.txt"),
    ("dumpsys_window", "dumpsys_window.txt"),
    ("dumpsys_clipboard", "dumpsys_clipboard.txt"),
    ("dumpsys_batterystats", "dumpsys_batterystats.txt"),
    ("dumpsys_keystore", "dumpsys_keystore.txt"),
    ("device_settings", "device_settings.txt"),
    ("network_info", "network_info.txt"),
    ("processes", "processes.txt"),
    ("disk_usage", "disk_usage.txt"),
    ("content_sms", "content_sms.txt"),
    ("content_calls", "content_calls.txt"),
    ("content_contacts", "content_contacts.txt"),
    ("content_user_dictionary", "content_user_dictionary.txt"),
    ("content_calendar", "content_calendar.txt"),
    ("content_media_images", "content_media_images.txt"),
    ("content_media_videos", "content_media_videos.txt"),
    ("content_media_audio", "content_media_audio.txt"),
    ("content_media_files", "content_media_files.txt"),
    (
        "content_telephony_carriers",
        "content_telephony_carriers.txt",
    ),
    ("screenshot", "screenshot.png"),
    ("whatsapp_media", "whatsapp_media"),
    ("telegram_media", "telegram_media"),
    ("app_media", "app_media"),
    ("all_app_media", "all_app_media"),
    ("adb_backup", "adb_backup.ab"),
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
    let target_path = dir.join("bugreport.zip");
    let target = target_path.to_string_lossy().into_owned();
    match run_adb_file_command(serial, &["bugreport", &target]) {
        Ok(()) => {
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

    // 1. dumpsys notification --noredact to capture full message text
    content.push_str("=== dumpsys notification --noredact ===\n");
    match run_adb_command(serial, &["shell", "dumpsys", "notification", "--noredact"]) {
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
        ("com.facebook.katana", "facebook"),
        ("com.viber.voip", "viber"),
        ("com.google.android.apps.messaging", "google_messages"),
        ("com.twitter.android", "x_twitter"),
        ("com.snapchat.android", "snapchat"),
        ("com.zhiliaoapp.musically", "tiktok"),
        ("com.ss.android.ugc.trill", "tiktok_alt"),
        ("com.discord", "discord"),
        ("com.linkedin.android", "linkedin"),
        ("com.pinterest", "pinterest"),
        ("com.reddit.frontpage", "reddit"),
        ("com.spotify.music", "spotify"),
        ("org.thoughtcrime.securesms", "signal"),
        ("com.skype.raider", "skype"),
        ("us.zoom.videomeetings", "zoom"),
        ("com.microsoft.teams", "teams"),
        ("com.turkcell.bip", "bip"),
        ("com.wire", "wire"),
        ("org.telegram.plus", "telegram_plus"),
        ("com.kakao.talk", "kakaotalk"),
        ("jp.naver.line.android", "line"),
        ("com.tencent.mm", "wechat"),
        ("com.imo.android.imoim", "imo"),
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

/// Dynamically scan /sdcard/Android/media/ and pull ALL app directories found.
/// This catches any app not in the hardcoded list above.
fn collect_all_app_media(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let all_media_dir = dir.join("all_app_media");
    let _ = std::fs::create_dir_all(&all_media_dir);

    // List all directories under /sdcard/Android/media/
    let listing = run_adb_command(serial, &["shell", "ls", "/sdcard/Android/media/"]);
    let packages: Vec<String> = match listing {
        Ok(output) => output
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty() && l.contains('.'))
            .collect(),
        Err(err) => {
            return AcquisitionItem {
                category: "all_app_media".to_string(),
                file_name: "all_app_media".to_string(),
                size: 0,
                success: false,
                error: Some(format!("Media dizini listelenemedi: {err}")),
            };
        }
    };

    let mut total_size = 0_u64;
    let mut found_any = false;

    for package in &packages {
        // Use package name as folder name, replacing dots with underscores for safety
        let folder_name = package.replace('.', "_");
        let remote = format!("/sdcard/Android/media/{package}/");
        let target = all_media_dir.join(&folder_name);
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
                    let _ = std::fs::remove_dir(&target);
                }
            }
        }
    }

    AcquisitionItem {
        category: "all_app_media".to_string(),
        file_name: "all_app_media".to_string(),
        size: total_size,
        success: found_any,
        error: if found_any {
            None
        } else {
            Some("Hicbir ek uygulama medyasi bulunamadi".to_string())
        },
    }
}

/// Collect device settings (system, secure, global).
fn collect_device_settings(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let sections = [
        (
            "=== settings list system ===",
            vec!["shell", "settings", "list", "system"],
        ),
        (
            "=== settings list secure ===",
            vec!["shell", "settings", "list", "secure"],
        ),
        (
            "=== settings list global ===",
            vec!["shell", "settings", "list", "global"],
        ),
    ];
    let mut content = String::new();
    for (header, args) in &sections {
        content.push_str(header);
        content.push('\n');
        match run_adb_command(serial, args.as_slice()) {
            Ok(output) => content.push_str(&output),
            Err(err) => content.push_str(&format!("Hata: {err}")),
        }
        content.push_str("\n\n");
    }
    let path = dir.join("device_settings.txt");
    match std::fs::write(&path, &content) {
        Ok(()) => AcquisitionItem {
            category: "device_settings".to_string(),
            file_name: "device_settings.txt".to_string(),
            size: content.len() as u64,
            success: true,
            error: None,
        },
        Err(err) => AcquisitionItem {
            category: "device_settings".to_string(),
            file_name: "device_settings.txt".to_string(),
            size: 0,
            success: false,
            error: Some(format!("Dosya yazilamadi: {err}")),
        },
    }
}

/// Try to query a content provider URI.
/// On modern Android without default-app permissions, this may fail — we attempt anyway.
fn collect_content_query(
    serial: &str,
    category: &str,
    file_name: &str,
    uri: &str,
    dir: &std::path::Path,
) -> AcquisitionItem {
    collect_shell_output(
        serial,
        category,
        file_name,
        &["shell", "content", "query", "--uri", uri],
        dir,
    )
}

/// Attempt adb backup (deprecated on modern Android but still yields data on older devices).
fn collect_adb_backup(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let target = dir.join("adb_backup.ab");
    let target_str = target.to_string_lossy().into_owned();
    // -all: backup all apps, -shared: include shared storage, -nosystem: skip system apps
    // -f: output file
    match run_adb_file_command(
        serial,
        &["backup", "-all", "-shared", "-nosystem", "-f", &target_str],
    ) {
        Ok(()) => {
            let size = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
            // Minimum valid backup is ~50 bytes (header only = empty)
            if size > 100 {
                AcquisitionItem {
                    category: "adb_backup".to_string(),
                    file_name: "adb_backup.ab".to_string(),
                    size,
                    success: true,
                    error: None,
                }
            } else {
                AcquisitionItem {
                    category: "adb_backup".to_string(),
                    file_name: "adb_backup.ab".to_string(),
                    size,
                    success: false,
                    error: Some("Backup dosyasi bos veya cihaz tarafindan reddedildi".to_string()),
                }
            }
        }
        Err(err) => AcquisitionItem {
            category: "adb_backup".to_string(),
            file_name: "adb_backup.ab".to_string(),
            size: 0,
            success: false,
            error: Some(err),
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
            "dumpsys_telephony" => collect_dumpsys(
                serial,
                "telephony.registry",
                "dumpsys_telephony.txt",
                output_dir,
            ),
            "dumpsys_location" => {
                collect_dumpsys(serial, "location", "dumpsys_location.txt", output_dir)
            }
            "dumpsys_netstats" => {
                collect_dumpsys(serial, "netstats", "dumpsys_netstats.txt", output_dir)
            }
            "dumpsys_activity" => {
                collect_dumpsys(serial, "activity", "dumpsys_activity.txt", output_dir)
            }
            "dumpsys_meminfo" => {
                collect_dumpsys(serial, "meminfo", "dumpsys_meminfo.txt", output_dir)
            }
            "dumpsys_appops" => collect_dumpsys(serial, "appops", "dumpsys_appops.txt", output_dir),
            "dumpsys_package" => {
                collect_dumpsys(serial, "package", "dumpsys_package.txt", output_dir)
            }
            "dumpsys_diskstats" => {
                collect_dumpsys(serial, "diskstats", "dumpsys_diskstats.txt", output_dir)
            }
            "dumpsys_deviceidle" => {
                collect_dumpsys(serial, "deviceidle", "dumpsys_deviceidle.txt", output_dir)
            }
            "dumpsys_alarm" => collect_dumpsys(serial, "alarm", "dumpsys_alarm.txt", output_dir),
            "dumpsys_jobscheduler" => collect_dumpsys(
                serial,
                "jobscheduler",
                "dumpsys_jobscheduler.txt",
                output_dir,
            ),
            "dumpsys_procstats" => {
                collect_dumpsys(serial, "procstats", "dumpsys_procstats.txt", output_dir)
            }
            "dumpsys_sensorservice" => collect_dumpsys(
                serial,
                "sensorservice",
                "dumpsys_sensorservice.txt",
                output_dir,
            ),
            "dumpsys_power" => collect_dumpsys(serial, "power", "dumpsys_power.txt", output_dir),
            "dumpsys_window" => collect_dumpsys(serial, "window", "dumpsys_window.txt", output_dir),
            "dumpsys_clipboard" => {
                collect_dumpsys(serial, "clipboard", "dumpsys_clipboard.txt", output_dir)
            }
            "dumpsys_batterystats" => collect_dumpsys(
                serial,
                "batterystats",
                "dumpsys_batterystats.txt",
                output_dir,
            ),
            "dumpsys_keystore" => {
                collect_dumpsys(serial, "keystore", "dumpsys_keystore.txt", output_dir)
            }
            "device_settings" => collect_device_settings(serial, output_dir),
            "network_info" => collect_network_info(serial, output_dir),
            "processes" => collect_processes(serial, output_dir),
            "disk_usage" => collect_disk_usage(serial, output_dir),
            "content_sms" => collect_content_query(
                serial,
                "content_sms",
                "content_sms.txt",
                "content://sms",
                output_dir,
            ),
            "content_calls" => collect_content_query(
                serial,
                "content_calls",
                "content_calls.txt",
                "content://call_log/calls",
                output_dir,
            ),
            "content_contacts" => collect_content_query(
                serial,
                "content_contacts",
                "content_contacts.txt",
                "content://contacts/phones",
                output_dir,
            ),
            "content_user_dictionary" => collect_content_query(
                serial,
                "content_user_dictionary",
                "content_user_dictionary.txt",
                "content://user_dictionary/words",
                output_dir,
            ),
            "content_calendar" => collect_content_query(
                serial,
                "content_calendar",
                "content_calendar.txt",
                "content://com.android.calendar/events",
                output_dir,
            ),
            "content_media_images" => collect_content_query(
                serial,
                "content_media_images",
                "content_media_images.txt",
                "content://media/external/images/media",
                output_dir,
            ),
            "content_media_videos" => collect_content_query(
                serial,
                "content_media_videos",
                "content_media_videos.txt",
                "content://media/external/video/media",
                output_dir,
            ),
            "content_media_audio" => collect_content_query(
                serial,
                "content_media_audio",
                "content_media_audio.txt",
                "content://media/external/audio/media",
                output_dir,
            ),
            "content_media_files" => collect_content_query(
                serial,
                "content_media_files",
                "content_media_files.txt",
                "content://media/external/file",
                output_dir,
            ),
            "content_telephony_carriers" => collect_content_query(
                serial,
                "content_telephony_carriers",
                "content_telephony_carriers.txt",
                "content://telephony/carriers",
                output_dir,
            ),
            "screenshot" => collect_screenshot(serial, output_dir),
            "whatsapp_media" => collect_whatsapp_media(serial, output_dir),
            "telegram_media" => collect_telegram_media(serial, output_dir),
            "app_media" => collect_app_media(serial, output_dir),
            "all_app_media" => collect_all_app_media(serial, output_dir),
            "adb_backup" => collect_adb_backup(serial, output_dir),
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

#[derive(Debug, Clone, Serialize)]
pub struct FilesystemAcquisitionResult {
    pub output_file: std::path::PathBuf,
    pub total_bytes: u64,
    pub sha256: String,
}

pub fn filesystem_acquisition<F, C>(
    serial: &str,
    output_dir: &std::path::Path,
    has_root: bool,
    mut progress: F,
    cancelled: C,
) -> Result<FilesystemAcquisitionResult, String>
where
    F: FnMut(u32, u32, &str),
    C: Fn() -> bool,
{
    std::fs::create_dir_all(output_dir)
        .map_err(|err| format!("Cikti dizini olusturulamadi: {err}"))?;

    // Step 0: Root check or attempt
    progress(0, 3, "Root yetkisi kontrol ediliyor...");

    let mut is_root = false;
    let mut use_su = false;

    // Check current root status
    if let Ok(out) = run_adb_command(serial, &["shell", "id"]) {
        if out.contains("uid=0(root)") || out.contains("root") {
            is_root = true;
        }
    }

    if !is_root {
        if let Ok(out) = run_adb_command(serial, &["shell", "su", "-c", "id"]) {
            if out.contains("uid=0(root)") || out.contains("root") {
                is_root = true;
                use_su = true;
            }
        }
    }

    // If not rooted and we are allowed to attempt adb root
    if !is_root && !has_root {
        progress(0, 3, "Root yetkisi bulunamadi, 'adb root' deneniyor...");
        let _ = Command::new("adb").args(["-s", serial, "root"]).output();
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Re-check
        if let Ok(out) = run_adb_command(serial, &["shell", "id"]) {
            if out.contains("uid=0(root)") || out.contains("root") {
                is_root = true;
            }
        }

        if !is_root {
            if let Ok(out) = run_adb_command(serial, &["shell", "su", "-c", "id"]) {
                if out.contains("uid=0(root)") || out.contains("root") {
                    is_root = true;
                    use_su = true;
                }
            }
        }
    }

    if !is_root {
        return Err("Cihazda root yetkisi alinamadi. Dosya sistemi imaji ancak root yetkisine sahip cihazlarda alinabilir.".to_string());
    }

    progress(1, 3, "Root yetkisi doğrulandı. Bölüm bilgileri analiz ediliyor...");

    // Try to resolve the userdata block device
    let mut block_device = None;
    let mount_cmd = if use_su {
        "su -c 'cat /proc/mounts'"
    } else {
        "cat /proc/mounts"
    };

    if let Ok(out) = run_adb_command(serial, &["shell", mount_cmd]) {
        for line in out.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 && parts[1] == "/data" {
                block_device = Some(parts[0].to_string());
                break;
            }
        }
    }

    let (output_file_name, use_block_copy) = if block_device.is_some() {
        (format!("userdata.img"), true)
    } else {
        (format!("filesystem.tar"), false)
    };

    let output_path = output_dir.join(&output_file_name);
    let mut file = std::fs::File::create(&output_path)
        .map_err(|err| format!("Hedef dosya olusturulamadi: {err}"))?;

    let mut cmd = Command::new("adb");
    cmd.args(["-s", serial]);

    if use_block_copy {
        let dev = block_device.clone().unwrap();
        let status_msg = format!("Bölüm aygıtı bulundu: {dev}. Disk imajı (dd) aktarılıyor...");
        progress(1, 3, &status_msg);

        if use_su {
            cmd.args(["exec-out", &format!("su -c 'dd if={} bs=4096 2>/dev/null'", dev)]);
        } else {
            cmd.args(["exec-out", &format!("dd if={} bs=4096 2>/dev/null", dev)]);
        }
    } else {
        progress(1, 3, "Bölüm aygıtı bulunamadı. Dosya arşivi (tar) aktarılıyor...");

        if use_su {
            cmd.args(["exec-out", "su -c 'tar -cf - /data 2>/dev/null'"]);
        } else {
            cmd.args(["exec-out", "tar -cf - /data 2>/dev/null"]);
        }
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());

    let mut child = cmd.spawn().map_err(|err| format!("ADB baslatilamadi: {err}"))?;
    let mut stdout = child.stdout.take().ok_or_else(|| "ADB cikti akisi alinamadi".to_string())?;

    let mut buffer = [0; 65536];
    let mut total_bytes = 0_u64;
    let mut last_progress_bytes = 0_u64;

    use std::io::{Read, Write};

    loop {
        if cancelled() {
            let _ = child.kill();
            return Err("Kullanici tarafindan iptal edildi".to_string());
        }

        let bytes_read = stdout.read(&mut buffer)
            .map_err(|err| format!("Cihazdan veri okunurken hata olustu: {err}"))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|err| format!("Dosyaya veri yazilirken hata olustu: {err}"))?;

        total_bytes += bytes_read as u64;

        if total_bytes - last_progress_bytes > 10 * 1024 * 1024 {
            last_progress_bytes = total_bytes;
            let mb = total_bytes / (1024 * 1024);
            let mode_str = if use_block_copy { "Disk İmajı" } else { "Dosya Arşivi" };
            progress(1, 3, &format!("{} aktarılıyor: {} MB", mode_str, mb));
        }
    }

    let _ = child.wait();

    if total_bytes == 0 {
        return Err("Aktarilan veri bos (0 byte). Gecersiz imaj.".to_string());
    }

    // Step 2: Hashing
    progress(2, 3, "İmaj aktarımı tamamlandı, bütünlük doğrulama özeti (SHA-256) hesaplanıyor...");

    let mut file = std::fs::File::open(&output_path)
        .map_err(|err| format!("Olusturulan imaj dosyasi acilamadi: {err}"))?;
    
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    
    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|err| format!("Hash hesabi icin dosya okunurken hata: {err}"))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let sha256 = crate::hash::to_hex(&hasher.finalize());

    // Write SHA-256 sidecar file
    let sidecar_path = output_dir.join(format!("{}.sha256", output_file_name));
    let _ = std::fs::write(&sidecar_path, format!("{sha256}  {}\n", output_file_name));

    progress(3, 3, "İşlem başarıyla tamamlandı!");

    Ok(FilesystemAcquisitionResult {
        output_file: output_path,
        total_bytes,
        sha256,
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
