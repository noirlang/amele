use super::adb::{
    first_non_empty, run_adb_command, run_adb_command_timeout, run_adb_file_command,
    run_adb_file_command_timeout,
};
use serde::Serialize;
use std::time::Duration;

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

#[cfg(test)]
mod tests {
    use super::*;

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
