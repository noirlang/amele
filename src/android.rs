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
    ("root_status", "root_status.txt"),
    ("procfs_summary", "procfs_summary.txt"),
    ("proc_memory_maps", "proc_memory_maps"),
    ("heapdump_candidates", "heapdump_candidates.txt"),
    ("debug_heap_dumps", "debug_heap_dumps"),
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

fn run_adb_command_timeout(
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

fn run_adb_file_command_timeout(
    serial: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<(), String> {
    run_adb_command_timeout(serial, args, timeout).map(|_| ())
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

fn collect_root_status(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let commands: &[(&str, &[&str])] = &[
        ("id", &["shell", "id"]),
        ("su -c id", &["shell", "su", "-c", "id"]),
        ("adb root probe", &["root"]),
        ("getenforce", &["shell", "getenforce"]),
        ("ro.debuggable", &["shell", "getprop", "ro.debuggable"]),
        ("ro.secure", &["shell", "getprop", "ro.secure"]),
        ("kernel", &["shell", "cat", "/proc/version"]),
    ];
    let mut content = String::new();

    for (label, args) in commands {
        content.push_str("=== ");
        content.push_str(label);
        content.push_str(" ===\n");
        match run_adb_command_timeout(serial, args, Duration::from_secs(8)) {
            Ok(output) => content.push_str(output.trim()),
            Err(err) => {
                content.push_str("Hata: ");
                content.push_str(&err);
            }
        }
        content.push_str("\n\n");
    }

    write_text_acquisition_item(dir, "root_status", "root_status.txt", content)
}

fn collect_procfs_summary(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let commands: &[(&str, &str)] = &[
        ("meminfo", "cat /proc/meminfo"),
        ("vmstat", "cat /proc/vmstat"),
        ("uptime", "cat /proc/uptime"),
        (
            "pressure memory",
            "cat /proc/pressure/memory 2>/dev/null || true",
        ),
        ("cpuinfo", "cat /proc/cpuinfo"),
        ("mounts", "cat /proc/mounts"),
        ("net tcp", "cat /proc/net/tcp 2>/dev/null || true"),
        ("net tcp6", "cat /proc/net/tcp6 2>/dev/null || true"),
        ("net udp", "cat /proc/net/udp 2>/dev/null || true"),
        ("net unix", "cat /proc/net/unix 2>/dev/null || true"),
    ];
    let mut content = String::new();

    for (label, shell) in commands {
        content.push_str("=== /proc ");
        content.push_str(label);
        content.push_str(" ===\n");
        match run_adb_command_timeout(
            serial,
            &["shell", "sh", "-c", shell],
            Duration::from_secs(10),
        ) {
            Ok(output) => content.push_str(output.trim()),
            Err(err) => {
                content.push_str("Hata: ");
                content.push_str(&err);
            }
        }
        content.push_str("\n\n");
    }

    write_text_acquisition_item(dir, "procfs_summary", "procfs_summary.txt", content)
}

fn collect_proc_memory_maps(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let target_dir = dir.join("proc_memory_maps");
    let _ = std::fs::create_dir_all(&target_dir);
    let ps_output =
        match run_adb_command_timeout(serial, &["shell", "ps", "-A"], Duration::from_secs(10)) {
            Ok(output) => output,
            Err(err) => {
                return AcquisitionItem {
                    category: "proc_memory_maps".to_string(),
                    file_name: "proc_memory_maps".to_string(),
                    size: 0,
                    success: false,
                    error: Some(format!("Process listesi alinamadi: {err}")),
                };
            }
        };

    let processes = parse_process_rows(&ps_output);
    let mut index = String::new();
    let mut captured = 0_usize;
    let mut failed = 0_usize;

    for (pid, name) in processes.into_iter().take(80) {
        let shell = format!("cat /proc/{pid}/maps 2>&1 | head -n 2000");
        let output = run_adb_command_timeout(
            serial,
            &["shell", "sh", "-c", &shell],
            Duration::from_secs(4),
        );
        match output {
            Ok(maps) if maps.contains('-') && !maps.contains("Permission denied") => {
                let safe_name = sanitize_file_component(&name);
                let file_name = format!("{pid}_{safe_name}.maps");
                let path = target_dir.join(&file_name);
                if std::fs::write(&path, &maps).is_ok() {
                    captured += 1;
                    index.push_str(&format!("{pid}\t{name}\t{file_name}\n"));
                } else {
                    failed += 1;
                    index.push_str(&format!("{pid}\t{name}\twrite_failed\n"));
                }
            }
            Ok(output) => {
                failed += 1;
                let detail = first_non_empty(&output).unwrap_or_else(|| "empty".to_string());
                index.push_str(&format!("{pid}\t{name}\t{detail}\n"));
            }
            Err(err) => {
                failed += 1;
                index.push_str(&format!("{pid}\t{name}\t{err}\n"));
            }
        }
    }

    let _ = std::fs::write(
        target_dir.join("index.tsv"),
        format!("pid\tname\tmaps_file_or_error\n{index}"),
    );
    let size = dir_size(&target_dir);
    AcquisitionItem {
        category: "proc_memory_maps".to_string(),
        file_name: "proc_memory_maps".to_string(),
        size,
        success: captured > 0,
        error: if captured > 0 {
            Some(format!("{captured} process maps alindi, {failed} atlandi"))
        } else {
            Some(format!("Process maps okunamadi, {failed} process atlandi"))
        },
    }
}

fn collect_heapdump_candidates(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let dumpsys = match run_adb_command_timeout(
        serial,
        &["shell", "dumpsys", "package"],
        Duration::from_secs(45),
    ) {
        Ok(output) => output,
        Err(err) => {
            return AcquisitionItem {
                category: "heapdump_candidates".to_string(),
                file_name: "heapdump_candidates.txt".to_string(),
                size: 0,
                success: false,
                error: Some(format!("Paket bilgisi alinamadi: {err}")),
            };
        }
    };

    let packages = parse_debuggable_packages(&dumpsys);
    let mut content = String::new();
    content.push_str("# Debuggable package candidates for adb shell am dumpheap\n");
    content.push_str("# Full process memory still requires root or ptrace privileges.\n\n");
    for package in &packages {
        content.push_str("package=");
        content.push_str(package);
        content.push('\n');
    }
    if packages.is_empty() {
        content.push_str("No debuggable package was detected from dumpsys package.\n");
    }

    let mut item = write_text_acquisition_item(
        dir,
        "heapdump_candidates",
        "heapdump_candidates.txt",
        content,
    );
    item.success = true;
    item.error = Some(format!("{} aday bulundu", packages.len()));
    item
}

fn collect_debug_heap_dumps(serial: &str, dir: &std::path::Path) -> AcquisitionItem {
    let candidates_path = dir.join("heapdump_candidates.txt");
    let packages = std::fs::read_to_string(&candidates_path)
        .ok()
        .map(|content| parse_candidate_package_lines(&content))
        .filter(|packages| !packages.is_empty())
        .unwrap_or_else(|| {
            run_adb_command_timeout(
                serial,
                &["shell", "dumpsys", "package"],
                Duration::from_secs(45),
            )
            .map(|output| parse_debuggable_packages(&output))
            .unwrap_or_default()
        });

    let target_dir = dir.join("debug_heap_dumps");
    let _ = std::fs::create_dir_all(&target_dir);
    let mut log = String::new();
    let mut dumped = 0_usize;
    let mut failed = 0_usize;

    for package in packages.iter().take(5) {
        let pid_output =
            run_adb_command_timeout(serial, &["shell", "pidof", package], Duration::from_secs(5));
        let pid = match pid_output
            .ok()
            .and_then(|output| output.split_whitespace().next().map(ToOwned::to_owned))
            .filter(|value| value.chars().all(|ch| ch.is_ascii_digit()))
        {
            Some(pid) => pid,
            None => {
                failed += 1;
                log.push_str(&format!("{package}\tpid_not_running\n"));
                continue;
            }
        };

        let safe_package = sanitize_file_component(package);
        let remote_path = format!("/sdcard/Download/worm_heap_{safe_package}_{pid}.hprof");
        let local_file = format!("{safe_package}_{pid}.hprof");
        let local_path = target_dir.join(&local_file);
        let local_arg = local_path.to_string_lossy().into_owned();

        match run_adb_command_timeout(
            serial,
            &["shell", "am", "dumpheap", &pid, &remote_path],
            Duration::from_secs(60),
        ) {
            Ok(_) => {}
            Err(err) => {
                failed += 1;
                log.push_str(&format!("{package}\t{pid}\tdumpheap_failed\t{err}\n"));
                let _ = run_adb_command_timeout(
                    serial,
                    &["shell", "rm", "-f", &remote_path],
                    Duration::from_secs(5),
                );
                continue;
            }
        }

        match run_adb_file_command_timeout(
            serial,
            &["pull", &remote_path, &local_arg],
            Duration::from_secs(120),
        ) {
            Ok(()) => {
                let size = std::fs::metadata(&local_path).map(|m| m.len()).unwrap_or(0);
                if size > 0 {
                    dumped += 1;
                    log.push_str(&format!("{package}\t{pid}\t{local_file}\t{size}\n"));
                } else {
                    failed += 1;
                    log.push_str(&format!("{package}\t{pid}\tempty_hprof\n"));
                }
            }
            Err(err) => {
                failed += 1;
                log.push_str(&format!("{package}\t{pid}\tpull_failed\t{err}\n"));
            }
        }

        let _ = run_adb_command_timeout(
            serial,
            &["shell", "rm", "-f", &remote_path],
            Duration::from_secs(5),
        );
    }

    let _ = std::fs::write(
        target_dir.join("heapdump_log.tsv"),
        format!("package\tpid\tfile_or_status\tsize_or_error\n{log}"),
    );
    let size = dir_size(&target_dir);
    AcquisitionItem {
        category: "debug_heap_dumps".to_string(),
        file_name: "debug_heap_dumps".to_string(),
        size,
        success: dumped > 0,
        error: if dumped > 0 {
            Some(format!("{dumped} HPROF alindi, {failed} hedef atlandi"))
        } else {
            Some("HPROF alinamadi; debuggable ve calisan uygulama gerekir".to_string())
        },
    }
}

fn write_text_acquisition_item(
    dir: &std::path::Path,
    category: &str,
    file_name: &str,
    content: String,
) -> AcquisitionItem {
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

fn parse_debuggable_packages(dumpsys_package: &str) -> Vec<String> {
    let mut current_package: Option<String> = None;
    let mut packages = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in dumpsys_package.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("Package [") {
            current_package = rest.split(']').next().map(ToOwned::to_owned);
            continue;
        }
        let debuggable = trimmed.contains("DEBUGGABLE")
            || trimmed.contains("FLAG_DEBUGGABLE")
            || trimmed.contains("debuggable=true");
        if debuggable {
            if let Some(package) = &current_package {
                if seen.insert(package.clone()) {
                    packages.push(package.clone());
                }
            }
        }
    }

    packages
}

fn parse_candidate_package_lines(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("package="))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn parse_process_rows(ps_output: &str) -> Vec<(u32, String)> {
    ps_output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                return None;
            }
            let pid_index = parts.iter().position(|part| part.parse::<u32>().is_ok())?;
            let pid = parts.get(pid_index)?.parse::<u32>().ok()?;
            let name = parts.last().copied().unwrap_or("process").to_string();
            Some((pid, name))
        })
        .collect()
}

fn sanitize_file_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
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

    let total = (LOGICAL_STEPS.len() + 2) as u32;
    let mut items = Vec::with_capacity(LOGICAL_STEPS.len() + 1);
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
            "root_status" => collect_root_status(serial, output_dir),
            "procfs_summary" => collect_procfs_summary(serial, output_dir),
            "proc_memory_maps" => collect_proc_memory_maps(serial, output_dir),
            "heapdump_candidates" => collect_heapdump_candidates(serial, output_dir),
            "debug_heap_dumps" => collect_debug_heap_dumps(serial, output_dir),
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

    progress(LOGICAL_STEPS.len() as u32, total, "mft_archive");
    match crate::android_mft::write_logical_mft_bundle(serial, output_dir) {
        Ok(bundle) => {
            items.push(AcquisitionItem {
                category: "mft_archive".to_string(),
                file_name: bundle.file_name.clone(),
                size: bundle.size,
                success: true,
                error: None,
            });
        }
        Err(err) => {
            errors.push(format!("mft_archive: {err}"));
            items.push(AcquisitionItem {
                category: "mft_archive".to_string(),
                file_name: "evidence.mft".to_string(),
                size: 0,
                success: false,
                error: Some(err),
            });
        }
    }

    progress(LOGICAL_STEPS.len() as u32 + 1, total, "analysis_outputs");
    match crate::android_mft::write_logical_analysis_outputs(serial, output_dir) {
        Ok(outputs) => {
            for output in outputs {
                items.push(AcquisitionItem {
                    category: "analysis_output".to_string(),
                    file_name: output.file_name,
                    size: output.size,
                    success: true,
                    error: None,
                });
            }
        }
        Err(err) => {
            errors.push(format!("analysis_outputs: {err}"));
            items.push(AcquisitionItem {
                category: "analysis_output".to_string(),
                file_name: "analysis_outputs".to_string(),
                size: 0,
                success: false,
                error: Some(err),
            });
        }
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

    progress(
        1,
        3,
        "Root yetkisi doğrulandı. Bölüm bilgileri analiz ediliyor...",
    );

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
            cmd.args([
                "exec-out",
                &format!("su -c 'dd if={} bs=4096 2>/dev/null'", dev),
            ]);
        } else {
            cmd.args(["exec-out", &format!("dd if={} bs=4096 2>/dev/null", dev)]);
        }
    } else {
        progress(
            1,
            3,
            "Bölüm aygıtı bulunamadı. Dosya arşivi (tar) aktarılıyor...",
        );

        if use_su {
            cmd.args(["exec-out", "su -c 'tar -cf - /data 2>/dev/null'"]);
        } else {
            cmd.args(["exec-out", "tar -cf - /data 2>/dev/null"]);
        }
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|err| format!("ADB baslatilamadi: {err}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "ADB cikti akisi alinamadi".to_string())?;

    let mut buffer = [0; 65536];
    let mut total_bytes = 0_u64;
    let mut last_progress_bytes = 0_u64;

    use std::io::{Read, Write};

    loop {
        if cancelled() {
            let _ = child.kill();
            return Err("Kullanici tarafindan iptal edildi".to_string());
        }

        let bytes_read = stdout
            .read(&mut buffer)
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
            let mode_str = if use_block_copy {
                "Disk İmajı"
            } else {
                "Dosya Arşivi"
            };
            progress(1, 3, &format!("{} aktarılıyor: {} MB", mode_str, mb));
        }
    }

    let _ = child.wait();

    if total_bytes == 0 {
        return Err("Aktarilan veri bos (0 byte). Gecersiz imaj.".to_string());
    }

    // Step 2: Hashing
    progress(
        2,
        3,
        "İmaj aktarımı tamamlandı, bütünlük doğrulama özeti (SHA-256) hesaplanıyor...",
    );

    let mut file = std::fs::File::open(&output_path)
        .map_err(|err| format!("Olusturulan imaj dosyasi acilamadi: {err}"))?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();

    let mut hashed_bytes = 0_u64;
    let mut last_hash_progress_bytes = 0_u64;

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| format!("Hash hesabi icin dosya okunurken hata: {err}"))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        hashed_bytes += bytes_read as u64;

        if hashed_bytes - last_hash_progress_bytes > 50 * 1024 * 1024 {
            last_hash_progress_bytes = hashed_bytes;
            let mb = hashed_bytes / (1024 * 1024);
            progress(
                2,
                3,
                &format!("Doğrulama özeti (SHA-256) hesaplanıyor: {} MB", mb),
            );
        }
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

#[derive(Debug, Clone, Serialize)]
pub struct AndroidRamAcquisitionResult {
    pub output_file: std::path::PathBuf,
    pub total_bytes: u64,
    pub sha256: String,
}

pub fn ram_acquisition<F, C>(
    serial: &str,
    output_dir: &std::path::Path,
    has_root: bool,
    mut progress: F,
    cancelled: C,
) -> Result<AndroidRamAcquisitionResult, String>
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
        return Err("Cihazda root yetkisi alinamadi. Bellek (RAM) imaji ancak root yetkisine sahip cihazlarda alinabilir.".to_string());
    }

    progress(
        1,
        3,
        "Root yetkisi doğrulandı. Bellek kaynakları analiz ediliyor...",
    );

    // Find readable memory interface
    let mut memory_source = None;
    let check_cmd = |path: &str| -> bool {
        let cmd = if use_su {
            format!("su -c 'test -r {} && echo OK'", path)
        } else {
            format!("test -r {} && echo OK", path)
        };
        run_adb_command(serial, &["shell", &cmd])
            .map(|out| out.trim() == "OK")
            .unwrap_or(false)
    };

    if check_cmd("/proc/kcore") {
        memory_source = Some("/proc/kcore");
    } else if check_cmd("/dev/mem") {
        memory_source = Some("/dev/mem");
    } else if check_cmd("/dev/kmem") {
        memory_source = Some("/dev/kmem");
    }

    let source = match memory_source {
        Some(src) => src,
        None => {
            progress(
                1,
                3,
                "Fiziksel bellek arayüzleri kısıtlı. Canlı uçucu bellek (Logical Process RAM) moduna otomatik geçiş yapılıyor...",
            );

            // Execute logical process memory dump!
            // 1. Create a script file locally
            let script_path = output_dir.join("memdump.sh");
            let script_content = r#"#!/system/bin/sh
# WORM Forensic Suite - Logical RAM & Volatile Memory Dumper
OUT_DIR="/data/local/tmp/worm_ram_dumps"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

echo "[WORM] Uçucu bellek analizi başlatıldı..."

# List of interesting process names or package patterns
ps -A -o PID,NAME | while read -r pid name; do
  [ "$pid" = "PID" ] && continue
  [ -z "$pid" ] && continue
  
  if [ -f "/proc/$pid/maps" ] && [ -f "/proc/$pid/mem" ]; then
    is_interesting=0
    case "$name" in
      *.*|system_server|init|zygote*|*chrome*|*browser*|*webview*|*telegram*|*whatsapp*|*signal*|*discord*)
        is_interesting=1
        ;;
    esac
    
    [ $is_interesting -eq 0 ] && continue
    
    echo "[WORM_PROGRESS] Dumping: $name ($pid)"
    PID_DIR="$OUT_DIR/$pid"
    mkdir -p "$PID_DIR"
    echo "$name" > "$PID_DIR/name.txt"
    cat "/proc/$pid/maps" > "$PID_DIR/maps"
    
    cat "/proc/$pid/maps" | grep -E "r-p|rw-p" | while read -r start_end perm offset dev inode path; do
      case "$path" in
        /system/*|/vendor/*|/apex/*|/system_ext/*|/data/app/*/*.so|/data/app/*/*.apk|*.so|*.apk|*.ttf|*.otf|*font*)
          continue
          ;;
      esac
      
      start=$(echo "$start_end" | cut -d'-' -f1)
      end=$(echo "$start_end" | cut -d'-' -f2)
      
      start_dec=$((0x$start))
      end_dec=$((0x$end))
      size=$((end_dec - start_dec))
      
      if [ $size -gt 0 ]; then
        dd if="/proc/$pid/mem" of="$PID_DIR/${start}_${end}.bin" bs=4096 skip=$((start_dec / 4096)) count=$((size / 4096)) 2>/dev/null
      fi
    done
  fi
done

echo "[WORM] Uçucu bellekler paketleniyor..."
cd /data/local/tmp
tar -cf worm_ram_dumps.tar worm_ram_dumps
rm -rf worm_ram_dumps
echo "[WORM] DONE"
"#;

            std::fs::write(&script_path, script_content)
                .map_err(|err| format!("Umut yerel betik olusturulamadi: {err}"))?;

            // 2. Push script to device
            progress(1, 3, "Uçucu bellek analiz betiği cihaza gönderiliyor...");
            let push_out = Command::new("adb")
                .args([
                    "-s",
                    serial,
                    "push",
                    script_path.to_str().unwrap(),
                    "/data/local/tmp/memdump.sh",
                ])
                .output()
                .map_err(|err| format!("Betiği cihaza push etme başarısız: {err}"))?;

            if !push_out.status.success() {
                return Err(format!(
                    "Betiği cihaza yükleme başarısız: {}",
                    String::from_utf8_lossy(&push_out.stderr)
                ));
            }

            // Remove local temp script
            let _ = std::fs::remove_file(&script_path);

            // 3. Make executable and run as root on the device
            progress(1, 3, "Uçucu bellek analiz betiği çalıştırılıyor...");
            let chmod_cmd = if use_su {
                "su -c 'chmod 755 /data/local/tmp/memdump.sh'"
            } else {
                "chmod 755 /data/local/tmp/memdump.sh"
            };
            let _ = Command::new("adb")
                .args(["-s", serial, "shell", chmod_cmd])
                .output();

            let run_cmd = if use_su {
                "su -c '/data/local/tmp/memdump.sh'"
            } else {
                "/data/local/tmp/memdump.sh"
            };

            let mut run_proc = Command::new("adb")
                .args(["-s", serial, "shell", run_cmd])
                .stdout(std::process::Stdio::piped())
                .spawn()
                .map_err(|err| format!("Uçucu bellek betiği başlatılamadı: {err}"))?;

            let run_stdout = run_proc
                .stdout
                .take()
                .ok_or_else(|| "Betiğin çıktı akışı alınamadı".to_string())?;
            let reader = std::io::BufReader::new(run_stdout);

            use std::io::BufRead;
            for line in reader.lines() {
                if cancelled() {
                    let _ = run_proc.kill();
                    return Err("Kullanici tarafindan iptal edildi".to_string());
                }
                if let Ok(l) = line {
                    if l.starts_with("[WORM_PROGRESS]") {
                        let proc_info = l.trim_start_matches("[WORM_PROGRESS] ").trim();
                        progress(1, 3, &format!("RAM imajı aktarılıyor: {}", proc_info));
                    }
                }
            }

            let _ = run_proc.wait();

            // 4. Pull the tar dump file
            progress(
                2,
                3,
                "Toplanan uçucu bellek verileri cihaza indiriliyor (tar arşivi)...",
            );
            let output_file_name = "android_logical_ram.tar";
            let output_path = output_dir.join(output_file_name);

            let pull_out = Command::new("adb")
                .args([
                    "-s",
                    serial,
                    "pull",
                    "/data/local/tmp/worm_ram_dumps.tar",
                    output_path.to_str().unwrap(),
                ])
                .output()
                .map_err(|err| format!("Bellek paketini çekme başarısız: {err}"))?;

            if !pull_out.status.success() {
                return Err(format!(
                    "Bellek paketini çekme başarısız: {}",
                    String::from_utf8_lossy(&pull_out.stderr)
                ));
            }

            // Cleanup remote temp files
            let rm_cmd = if use_su {
                "su -c 'rm -f /data/local/tmp/memdump.sh /data/local/tmp/worm_ram_dumps.tar'"
            } else {
                "rm -f /data/local/tmp/memdump.sh /data/local/tmp/worm_ram_dumps.tar"
            };
            let _ = Command::new("adb")
                .args(["-s", serial, "shell", rm_cmd])
                .output();

            let metadata = std::fs::metadata(&output_path)
                .map_err(|err| format!("Bellek dosyası metaverisi okunamadı: {err}"))?;
            let total_bytes = metadata.len();

            // Step 2: Hashing
            progress(
                2,
                3,
                "RAM imajı tamamlandı, bütünlük doğrulama özeti (SHA-256) hesaplanıyor...",
            );

            let mut file = std::fs::File::open(&output_path)
                .map_err(|err| format!("Olusturulan RAM imaj dosyasi acilamadi: {err}"))?;

            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            let mut buffer = [0; 65536];
            let mut hashed_bytes = 0_u64;
            let mut last_hash_progress_bytes = 0_u64;

            loop {
                if cancelled() {
                    return Err("Kullanici tarafindan iptal edildi".to_string());
                }
                let bytes_read = file
                    .read(&mut buffer)
                    .map_err(|err| format!("Hash hesabi icin dosya okunurken hata: {err}"))?;
                if bytes_read == 0 {
                    break;
                }
                hasher.update(&buffer[..bytes_read]);
                hashed_bytes += bytes_read as u64;

                if hashed_bytes - last_hash_progress_bytes > 50 * 1024 * 1024 {
                    last_hash_progress_bytes = hashed_bytes;
                    let mb = hashed_bytes / (1024 * 1024);
                    progress(
                        2,
                        3,
                        &format!("Doğrulama özeti (SHA-256) hesaplanıyor: {} MB", mb),
                    );
                }
            }

            let sha256 = crate::hash::to_hex(&hasher.finalize());

            // Write SHA-256 sidecar file
            let sidecar_path = output_dir.join(format!("{}.sha256", output_file_name));
            let _ = std::fs::write(&sidecar_path, format!("{sha256}  {}\n", output_file_name));

            progress(3, 3, "İşlem başarıyla tamamlandı!");

            return Ok(AndroidRamAcquisitionResult {
                output_file: output_path,
                total_bytes,
                sha256,
            });
        }
    };

    let output_file_name = "android_ram.bin";
    let output_path = output_dir.join(output_file_name);
    let mut file = std::fs::File::create(&output_path)
        .map_err(|err| format!("Hedef dosya olusturulamadi: {err}"))?;

    let mut cmd = Command::new("adb");
    cmd.args(["-s", serial]);

    let status_msg = format!("Bellek kaynağı bulundu: {source}. RAM imajı aktarılıyor...");
    progress(1, 3, &status_msg);

    if use_su {
        cmd.args([
            "exec-out",
            &format!("su -c 'dd if={} bs=4096 2>/dev/null'", source),
        ]);
    } else {
        cmd.args(["exec-out", &format!("dd if={} bs=4096 2>/dev/null", source)]);
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|err| format!("ADB baslatilamadi: {err}"))?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| "ADB cikti akisi alinamadi".to_string())?;

    let mut buffer = [0; 65536];
    let mut total_bytes = 0_u64;
    let mut last_progress_bytes = 0_u64;

    use std::io::{Read, Write};

    loop {
        if cancelled() {
            let _ = child.kill();
            return Err("Kullanici tarafindan iptal edildi".to_string());
        }

        let bytes_read = stdout
            .read(&mut buffer)
            .map_err(|err| format!("Cihazdan bellek verisi okunurken hata olustu: {err}"))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|err| format!("Dosyaya bellek verisi yazilirken hata olustu: {err}"))?;

        total_bytes += bytes_read as u64;

        if total_bytes - last_progress_bytes > 10 * 1024 * 1024 {
            last_progress_bytes = total_bytes;
            let mb = total_bytes / (1024 * 1024);
            progress(1, 3, &format!("RAM imajı aktarılıyor: {} MB", mb));
        }
    }

    let _ = child.wait();

    if total_bytes == 0 {
        return Err("Aktarilan bellek verisi bos (0 byte). RAM imaji basarisiz.".to_string());
    }

    // Step 2: Hashing
    progress(
        2,
        3,
        "RAM imajı tamamlandı, bütünlük doğrulama özeti (SHA-256) hesaplanıyor...",
    );

    let mut file = std::fs::File::open(&output_path)
        .map_err(|err| format!("Olusturulan RAM imaj dosyasi acilamadi: {err}"))?;

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    let mut hashed_bytes = 0_u64;
    let mut last_hash_progress_bytes = 0_u64;

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|err| format!("Hash hesabi icin dosya okunurken hata: {err}"))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
        hashed_bytes += bytes_read as u64;

        if hashed_bytes - last_hash_progress_bytes > 50 * 1024 * 1024 {
            last_hash_progress_bytes = hashed_bytes;
            let mb = hashed_bytes / (1024 * 1024);
            progress(
                2,
                3,
                &format!("Doğrulama özeti (SHA-256) hesaplanıyor: {} MB", mb),
            );
        }
    }

    let sha256 = crate::hash::to_hex(&hasher.finalize());

    // Write SHA-256 sidecar file
    let sidecar_path = output_dir.join(format!("{}.sha256", output_file_name));
    let _ = std::fs::write(&sidecar_path, format!("{sha256}  {}\n", output_file_name));

    progress(3, 3, "İşlem başarıyla tamamlandı!");

    Ok(AndroidRamAcquisitionResult {
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

    #[test]
    fn parses_debuggable_packages_from_dumpsys() {
        let packages = parse_debuggable_packages(
            "Package [com.example.normal] (abc):\n\
             pkgFlags=[ HAS_CODE ALLOW_CLEAR_USER_DATA ]\n\
             Package [com.example.debug] (def):\n\
             pkgFlags=[ HAS_CODE DEBUGGABLE ALLOW_CLEAR_USER_DATA ]\n",
        );

        assert_eq!(packages, vec!["com.example.debug"]);
    }

    #[test]
    fn parses_process_rows_from_ps_output() {
        let rows = parse_process_rows(
            "USER PID PPID VSZ RSS WCHAN ADDR S NAME\n\
             u0_a123 2345 123 100 20 0 0 S com.example.app\n\
             root 1 0 0 0 0 0 S init\n",
        );

        assert_eq!(rows[0], (2345, "com.example.app".to_string()));
        assert_eq!(rows[1], (1, "init".to_string()));
    }
}
