//! Komut satırı girişini işler ve UI, browser veya helper modlarını başlatır.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
use amele::android;
use amele::android_analysis;
use amele::api;
use amele::disk;
use amele::disk_analysis;
use amele::docker::{self, DockerAcquisitionRequest};
use amele::evidence::EvidenceVault;
use amele::hash::{self, HashAlgorithm};
use amele::ios;
use amele::output_format::{self, AcquisitionOutputFormat};
use amele::ram;
use amele::ram_analysis;
use amele::remote::RemoteConnection;
use amele::server;
use amele::settings::AppSettings;
use amele::ssh::{SshConnection, SshConnectionParams};
use amele::wireguard::{self, WireGuardConfig};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

static IS_ENGLISH: AtomicBool = AtomicBool::new(false);

fn set_cli_english(val: bool) {
    IS_ENGLISH.store(val, Ordering::SeqCst);
}

fn is_cli_english() -> bool {
    IS_ENGLISH.load(Ordering::SeqCst)
}

fn t_cli(tr: &str, en: &str) -> String {
    if is_cli_english() {
        en.to_string()
    } else {
        tr.to_string()
    }
}

fn main() {
    install_error_reporting();

    let mut raw_args: Vec<String> = std::env::args().skip(1).collect();

    // Standart --version / -V / version kontrolü (logo ve profil gerektirmez, çıkış kodu 0)
    if raw_args.iter().any(|a| a == "--version" || a == "-V")
        || raw_args.first().map(|s| s.as_str()) == Some("version")
    {
        println!("amele {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let quiet = extract_flag(&mut raw_args, &["--quiet", "-q", "--no-logo"]);
    let verbose = extract_flag(&mut raw_args, &["--verbose", "-v"]);
    if verbose {
        amele::logging::set_verbose(true);
    }

    let explicit_lang = extract_global_lang_flag(&mut raw_args);
    if let Some(lang) = &explicit_lang {
        set_cli_english(lang == "en");
    }

    let profile_arg = extract_global_profile(&mut raw_args);
    if let Some(username) = profile_arg {
        if let Err(err) = amele::profile::select_profile(&username, false) {
            report_fatal_error(&err.to_string());
            eprintln!("{err}");
            std::process::exit(determine_exit_code(&err.to_string()));
        }
    } else {
        let _ = amele::profile::bootstrap_profiles();
    }

    if explicit_lang.is_none() {
        if let Some(prof) = amele::profile::active_profile() {
            if prof.language == "en" {
                set_cli_english(true);
            }
        }
    }

    let first_cmd = raw_args.first().map(|s| s.as_str());
    if !quiet && !is_silent_or_helper_command(first_cmd, &raw_args) {
        println!("{AMELE_ASCII_LOGO}\n");
    }

    // Profil gerektiren komutlarda aktif profil yoksa seçim veya oluşturma iste
    if !is_profile_exempt_command(first_cmd, &raw_args) {
        if let Err(err) = prompt_and_ensure_active_profile() {
            report_fatal_error(&err);
            eprintln!("{err}\n");
            std::process::exit(determine_exit_code(&err));
        }
    }

    // If only --lang <target> was supplied and raw_args is now empty
    if let Some(lang) = explicit_lang {
        if raw_args.is_empty() {
            if let Err(err) = lang_set_command(lang) {
                eprintln!("{err}");
                std::process::exit(2);
            }
            return;
        }
    }

    let mut args = raw_args.into_iter();
    let result = match args.next().as_deref() {
        Some("lang") | Some("--lang") => {
            let target = args.next().unwrap_or_else(|| "tr".to_string());
            lang_set_command(target)
        }
        Some("linux") => linux_cli_command(args.collect()),
        Some("windows") => windows_cli_command(args.collect()),
        Some("ram") => {
            let rem: Vec<String> = args.collect();
            #[cfg(target_os = "windows")]
            {
                let mut sub = vec!["ram".to_string()];
                sub.extend(rem);
                windows_cli_command(sub)
            }
            #[cfg(not(target_os = "windows"))]
            {
                let mut sub = vec!["ram".to_string()];
                sub.extend(rem);
                linux_cli_command(sub)
            }
        }
        Some("disk") => {
            let rem: Vec<String> = args.collect();
            #[cfg(target_os = "windows")]
            {
                let mut sub = vec!["disk".to_string()];
                sub.extend(rem);
                windows_cli_command(sub)
            }
            #[cfg(not(target_os = "windows"))]
            {
                let mut sub = vec!["disk".to_string()];
                sub.extend(rem);
                linux_cli_command(sub)
            }
        }
        Some("android") => android_cli_command(args.collect()),
        Some("ios") => ios_cli_command(args.collect()),
        Some("docker") => docker_cli_command(args.collect()),
        Some("profile") => profile_cli_command(args.collect()),
        Some("case") => case_cli_command(args.collect()),
        Some("mount") => mount_cli_command(args.collect()),
        Some("wireguard") => wireguard_config_command(args.collect()),
        Some("settings-default") => print_default_settings(),
        Some("profiles") | Some("profile-list") => profile_list_command(),
        Some("profile-create") => profile_create_command(args.collect()),
        Some("profile-use") | Some("profile-select") => profile_use_command(args.collect()),
        Some("profile-logout") => profile_logout_command(),
        Some("profile-online-sync") | Some("online-sync") => profile_online_sync_command(),
        Some("hash") => hash_command(args.collect()),
        Some("disk-list") => disk_list_command_with_args(args.collect()),
        Some("local-image") => local_image_command(args.collect()),
        Some("local-ram") => local_ram_command(args.collect()),
        Some("remote-ram") => remote_ram_command(args.collect()),
        Some("image-analyze") => image_analyze_command(args.collect()),
        Some("ram-summary") => ram_summary_command(args.collect()),
        Some("ram-strings") => ram_strings_command(args.collect()),
        Some("ram-carve") => ram_carve_command(args.collect()),
        Some("ram-processes") => ram_processes_command(args.collect()),
        Some("adb-status") | Some("android-adb-status") => android_adb_status_command(),
        Some("adb-install") | Some("android-adb-install") => android_adb_install_command(),
        Some("android-devices") => android_devices_command(),
        Some("android-profile") => android_profile_command(args.collect()),
        Some("android-logical") => android_logical_command(args.collect()),
        Some("android-filesystem") => android_filesystem_command(args.collect()),
        Some("android-ram") => android_ram_command(args.collect()),
        Some("android-capabilities") => android_capabilities_command(args.collect()),
        Some("android-lemon-preflight") => android_lemon_preflight_command(args.collect()),
        Some("android-remote-connect") => android_remote_connect_command(args.collect()),
        Some("android-remote-disconnect") => android_remote_disconnect_command(args.collect()),
        Some("android-case-analysis") => android_case_analysis_command(args.collect()),
        Some("ios-backup-profile") => ios_backup_profile_command(args.collect()),
        Some("ios-backup-normalize") => ios_backup_normalize_command(args.collect()),
        Some("docker-status") => docker_status_command(args.collect()),
        Some("docker-list") | Some("docker-containers") => docker_list_command(args.collect()),
        Some("docker-logs") => docker_logs_command(args.collect()),
        Some("docker-acquire") => docker_acquire_command(args.collect()),
        Some("docker-remote-status") => docker_remote_status_command(args.collect()),
        Some("docker-remote-list") => docker_remote_list_command(args.collect()),
        Some("docker-remote-logs") => docker_remote_logs_command(args.collect()),
        Some("docker-remote-acquire") => docker_remote_acquire_command(args.collect()),
        Some("disk-list-helper") => disk_list_helper_command(args.collect()),
        Some("image-helper") => image_helper_command(args.collect()),
        Some("ram-helper") => ram_helper_command(args.collect()),
        Some("avml-install-helper") => avml_install_helper_command(args.collect()),
        Some("winpmem-install-helper") => winpmem_install_helper_command(args.collect()),
        Some("mount-helper") => mount_helper_command(args.collect()),
        Some("disk-size") => disk_size_command(args.collect()),
        Some("verify") => verify_command(args.collect()),
        Some("remote-disks") => remote_disks_command(args.collect()),
        Some("remote-image") => remote_image_command(args.collect()),
        Some("remote-tool-check") => remote_tool_check_command(args.collect()),
        Some("ssh-disks") => ssh_disks_command(args.collect()),
        Some("ssh-tool-check") => ssh_tool_check_command(args.collect()),
        Some("ssh-image") => ssh_image_command(args.collect()),
        Some("ssh-ram") => ssh_ram_command(args.collect()),
        Some("ram-status") => ram_status_command(args.collect()),
        Some("wireguard-config") => wireguard_config_command(args.collect()),
        Some("update-check") | Some("check-update") | Some("update") => {
            update_check_command(args.collect())
        }
        Some("case-export") => case_export_command(args.collect()),
        Some("case-import") => case_import_command(args.collect()),
        Some("case-verify") => case_verify_command(args.collect()),
        Some("preflight") => preflight_command(args.collect()),
        Some("mounts") | Some("mount-list") => mount_list_command(),
        Some("mount-cleanup") => mount_cleanup_command(args.collect()),
        Some("completion") => completion_command(args.collect()),
        Some("--version") | Some("-V") | Some("version") => {
            println!("amele {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some("ui") => server::run_native(),
        Some("ui-browser") => server::run_browser(),
        Some("help") | Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        None => default_command(),
        Some(other) => Err(format!(
            "{} {}",
            t_cli("Bilinmeyen komut:", "Unknown command:"),
            other
        )),
    };

    if let Err(err) = result {
        report_fatal_error(&err);
        eprintln!("{err}\n");
        if should_print_help_on_error(&err) {
            print_help();
        }
        std::process::exit(determine_exit_code(&err));
    }
}

/// Hata mesajına ve durumuna göre standart POSIX çıkış kodu üretir.
fn determine_exit_code(err: &str) -> i32 {
    let lower = err.to_lowercase();
    if lower.starts_with("kullanim:")
        || lower.starts_with("usage:")
        || lower.contains("bilinmeyen")
        || lower.contains("unknown")
        || lower.contains("geçersiz")
        || lower.contains("invalid")
    {
        2
    } else if lower.contains("root")
        || lower.contains("yetki")
        || lower.contains("sudo")
        || lower.contains("permission denied")
        || lower.contains("access denied")
    {
        126
    } else if lower.contains("bulunamadı")
        || lower.contains("not found")
        || lower.contains("yüklü değil")
        || lower.contains("not installed")
    {
        127
    } else {
        1
    }
}

/// Sadece kullanıcı komutu yanlış yazdığında genel yardım basar.
fn should_print_help_on_error(err: &str) -> bool {
    err.starts_with("Kullanim:")
        || err.starts_with("Bilinmeyen")
        || err.starts_with("Usage:")
        || err.starts_with("Unknown")
}

#[cfg(target_os = "windows")]
fn default_command() -> Result<(), String> {
    server::run_native()
}

/// Windows dışındaki sistemlerde argüman verilmezse sadece yardım metnini gösterir.
#[cfg(not(target_os = "windows"))]
fn default_command() -> Result<(), String> {
    print_help();
    Ok(())
}

#[cfg(target_os = "windows")]
fn install_error_reporting() {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|loc| format!("{}:{}", loc.file(), loc.line()))
            .unwrap_or_else(|| "unknown location".to_string());
        windows_error::report(&format!(
            "Unexpected Amele startup crash:\n\n{info}\n\nLocation: {location}"
        ));
    }));
}

#[cfg(not(target_os = "windows"))]
fn install_error_reporting() {}

#[cfg(target_os = "windows")]
fn report_fatal_error(message: &str) {
    windows_error::report(message);
}

#[cfg(not(target_os = "windows"))]
fn report_fatal_error(_message: &str) {}

/// Windows başlangıç hatalarını log dosyasına ve mesaj kutusuna yazdırır.
#[cfg(target_os = "windows")]
mod windows_error {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use windows_sys::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};

    pub fn report(message: &str) {
        let log_path = log_path();
        if let Some(path) = &log_path {
            write_log(path, message);
        }

        let mut body = format!("Amele Forensic Tool could not start.\n\n{message}");
        if let Some(path) = log_path {
            body.push_str(&format!("\n\nLog file:\n{}", path.display()));
        }
        show_message(&body);
    }

    fn write_log(path: &PathBuf, message: &str) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|value| value.as_secs())
                .unwrap_or_default();
            let _ = writeln!(file, "[{ts}] {message}");
        }
    }

    fn log_path() -> Option<PathBuf> {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("TEMP").map(PathBuf::from))
            .map(|base| base.join("Amele").join("amele.log"))
    }

    fn show_message(message: &str) {
        let title = wide_null("Amele Forensic Tool");
        let body = wide_null(message);
        unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                body.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
    }

    fn wide_null(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }
}

const AMELE_ASCII_LOGO: &str = r#"          ⣠⣧⡀
         ⣰⠃⡏⢳⡀
        ⡴⠁ ⡇ ⠳⡀
       ⡼⠁ ⡼⠹⡄ ⠹⡄
     ⢀⡜⠁⢀⡜⠁ ⠘⣆ ⠙⣆
    ⢀⡞ ⢀⣀⣙⡦⠦⣞⣁⣀ ⠘⣆
   ⢠⠎⠑⣤⣏⣉⣉⠑⡖⢉⣉⣉⣳⡔⠉⢆
  ⢠⠿⣄⡰⠋⠳⣤⣤⣤⢧⣤⣤⡴⠋⠳⣀⡼⢧
 ⣰⠋ ⡼⠛⢦⡞      ⠘⣦⠞⠻⡄⠈⢳⡀
⣰⣇⣀⣼⣁⣠⠞        ⠘⢦⣀⣹⣄⣀⣷⡀"#;

fn is_silent_or_helper_command(cmd: Option<&str>, raw_args: &[String]) -> bool {
    if raw_args
        .iter()
        .any(|a| a == "--json" || a == "--quiet" || a == "-q" || a == "--no-logo")
    {
        return true;
    }
    match cmd {
        Some("--version")
        | Some("-V")
        | Some("version")
        | Some("completion")
        | Some("settings-default")
        | Some("disk-list-helper")
        | Some("image-helper")
        | Some("ram-helper")
        | Some("avml-install-helper")
        | Some("winpmem-install-helper")
        | Some("mount-helper")
        | Some("disk-size")
        | Some("remote-tool-check")
        | Some("ui")
        | Some("ui-browser") => true,
        Some("update-check") | Some("check-update") | Some("update") => {
            raw_args.iter().any(|a| a == "--json")
        }
        _ => false,
    }
}

/// Profil gerektirmeyen yardım, yapılandırma ve yönetim komutlarını belirler.
fn is_profile_exempt_command(cmd: Option<&str>, raw_args: &[String]) -> bool {
    if raw_args.iter().any(|a| {
        a == "--help"
            || a == "-h"
            || a == "help"
            || a == "--version"
            || a == "-V"
            || a == "--status"
            || a == "status"
            || a == "--list"
            || a == "list"
            || a == "install"
            || a == "download"
    }) {
        return true;
    }
    match cmd {
        None
        | Some("help")
        | Some("--help")
        | Some("-h")
        | Some("--version")
        | Some("-V")
        | Some("version")
        | Some("completion")
        | Some("lang")
        | Some("--lang")
        | Some("profile")
        | Some("profiles")
        | Some("profile-list")
        | Some("profile-create")
        | Some("profile-use")
        | Some("profile-select")
        | Some("profile-logout")
        | Some("profile-online-sync")
        | Some("online-sync")
        | Some("update")
        | Some("update-check")
        | Some("check-update")
        | Some("ui")
        | Some("ui-browser")
        | Some("settings-default")
        | Some("disk-list")
        | Some("ram-status")
        | Some("mount")
        | Some("mounts")
        | Some("mount-list")
        | Some("mount-cleanup")
        | Some("disk-list-helper")
        | Some("image-helper")
        | Some("ram-helper")
        | Some("avml-install-helper")
        | Some("winpmem-install-helper")
        | Some("mount-helper")
        | Some("disk-size")
        | Some("remote-tool-check")
        | Some("ssh-tool-check") => true,
        _ => false,
    }
}

/// Komut çalıştırılmadan önce aktif bir profilin seçilmesini veya oluşturulmasını sağlar.
fn prompt_and_ensure_active_profile() -> Result<(), String> {
    use std::io::{self, Write};

    if amele::profile::active_profile().is_some() {
        return Ok(());
    }

    let store = amele::profile::load_profile_store().unwrap_or_default();
    if store.profiles.is_empty() {
        // İlk çalıştırma: Kayıtlı profil yok
        println!(
            "{}",
            t_cli(
                "⚠️  Aktif bir analist profili bulunamadi!\nAdli islemlerin ve delillerin guvenle kayit altina alinabilmesi icin profil olusturmalisiniz.\n",
                "⚠️  No active analyst profile found!\nYou must create an analyst profile for forensic integrity and auditing.\n"
            )
        );

        print!("{}", t_cli("Adiniz Soyadiniz: ", "Full Name: "));
        let _ = io::stdout().flush();
        let mut name_buf = String::new();
        if io::stdin().read_line(&mut name_buf).is_err() || name_buf.trim().is_empty() {
            return Err(t_cli(
                "Hata: Profil olusturulmadi. CLI komutlarini kullanmak icin profil zorunludur.\nProfil olusturmak icin: amele profile create \"Ad Soyad\" <kullanici_adi> [--direct]",
                "Error: Profile not created. An active profile is required to use CLI commands.\nTo create: amele profile create \"Full Name\" <username> [--direct]",
            ));
        }
        let full_name = name_buf.trim();

        print!(
            "{}",
            t_cli("Kullanici Adi (bosluksuz): ", "Username (no spaces): ")
        );
        let _ = io::stdout().flush();
        let mut user_buf = String::new();
        if io::stdin().read_line(&mut user_buf).is_err() || user_buf.trim().is_empty() {
            return Err(t_cli(
                "Hata: Gecersiz kullanici adi. Profil olusturulamadi.",
                "Error: Invalid username. Profile could not be created.",
            ));
        }
        let username = user_buf.trim();

        print!(
            "{}",
            t_cli(
                "Dil [tr/en] (varsayilan: tr): ",
                "Language [tr/en] (default: en): "
            )
        );
        let _ = io::stdout().flush();
        let mut lang_buf = String::new();
        let _ = io::stdin().read_line(&mut lang_buf);
        let lang = lang_buf.trim();
        let lang = if lang.eq_ignore_ascii_case("en") {
            "en"
        } else {
            "tr"
        };

        let prof = amele::profile::create_profile(full_name, username, lang, "dark", true)
            .map_err(|e| {
                format!(
                    "{}: {}",
                    t_cli("Profil olusturulamadi", "Could not create profile"),
                    e
                )
            })?;

        println!(
            "\n{} @{}\n",
            t_cli(
                "✓ Profil olusturuldu ve varsayilan olarak secildi:",
                "✓ Profile created and selected as default:"
            ),
            prof.username
        );
        return Ok(());
    }

    // Kayıtlı profil var fakat hiçbiri aktif değil
    println!(
        "{}",
        t_cli(
            "⚠️  Aktif bir analist profili secilmemis!\nLutfen devam etmek icin bir profil secin:\n",
            "⚠️  No active analyst profile selected!\nPlease select an analyst profile to continue:\n"
        )
    );

    for (idx, p) in store.profiles.iter().enumerate() {
        println!("  [{}] {} (@{})", idx + 1, p.full_name, p.username);
    }
    println!();

    print!(
        "{}",
        t_cli(
            "Profil No veya Kullanici Adi: ",
            "Profile Number or Username: "
        )
    );
    let _ = io::stdout().flush();
    let mut choice_buf = String::new();
    if io::stdin().read_line(&mut choice_buf).is_err() || choice_buf.trim().is_empty() {
        return Err(t_cli(
            "Hata: Profil secilmedi. Komut iptal edildi.\nProfil secmek icin: amele profile use <kullanici_adi> [--direct]",
            "Error: Profile not selected. Command aborted.\nTo select: amele profile use <username> [--direct]",
        ));
    }
    let choice = choice_buf.trim();

    let selected_user = if let Ok(num) = choice.parse::<usize>() {
        if num >= 1 && num <= store.profiles.len() {
            store.profiles[num - 1].username.clone()
        } else {
            return Err(t_cli(
                "Gecersiz profil numarasi.",
                "Invalid profile number.",
            ));
        }
    } else if let Some(p) = store
        .profiles
        .iter()
        .find(|p| p.username.eq_ignore_ascii_case(choice))
    {
        p.username.clone()
    } else {
        return Err(format!(
            "{}: '{}'",
            t_cli("Profil bulunamadi", "Profile not found"),
            choice
        ));
    };

    let prof = amele::profile::select_profile(&selected_user, true).map_err(|e| {
        format!(
            "{}: {}",
            t_cli("Profil secilemedi", "Could not select profile"),
            e
        )
    })?;

    println!(
        "\n{} @{}\n",
        t_cli("✓ Aktif profil secildi:", "✓ Active profile selected:"),
        prof.username
    );

    Ok(())
}

fn linux_cli_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele Linux Edinim ve Analiz Komutlari

KULLANIM:
  amele linux <disk|ram> [secenekler]

DISK ISLEMLERI:
  amele linux disk --list                                    Yerel diskleri listele
  amele linux disk <kaynak> <vaka> [disk_adi] [raw|aff4]     Yerel disk imaji al (sudo gerektirir)
  amele linux disk analyze <imaj> [mount_klasoru]            Disk imajini yapisal olarak analiz et
  amele linux disk --agent <ip> <port> --list [token]        Uzak agent disklerini listele
  amele linux disk --agent <ip> <port> <id> <cikti> [token]  Uzak agent uzerinden imaj al
  amele linux disk --ssh <ip> <port> --list [user] [pass]    SSH ile diskleri listele
  amele linux disk --ssh <ip> <port> <id> <vaka> [user]      SSH ile agent'siz imaj al

RAM ISLEMLERI (AVML):
  amele linux ram --status                                   AVML ve bellek durumunu goster
  amele linux ram install                                    AVML aracini otomatik indir ve kur (sudo)
  amele linux ram <vaka> [arac_yolu] [raw|aff4]              Yerel canli RAM imaji al (sudo gerektirir)
  amele linux ram analyze <ram> [windows|linux] [symbols]    RAM imaji ozet analizi yap
  amele linux ram strings <ram>                              RAM imajinda IOC / dizgi aramasi yap
  amele linux ram carve <ram> <cikti_klasoru>                RAM imajindan dosya cikar (carving)
  amele linux ram processes <ram> [windows|linux] [symbols]  RAM icinden calisan surecleri listele
  amele linux ram --agent <ip> <port> <vaka> [token]         Uzak agent'tan RAM imaji al
  amele linux ram --ssh <ip> <port> <vaka> [user] [pass]     SSH ile canli RAM dokumu al"#,
                r#"Amele Linux Acquisition & Analysis Commands

USAGE:
  amele linux <disk|ram> [options]

DISK OPERATIONS:
  amele linux disk --list                                    List local disks
  amele linux disk <source> <case> [disk_name] [raw|aff4]    Acquire local disk image (requires sudo)
  amele linux disk analyze <image> [mount_dir]               Analyze disk image structurally
  amele linux disk --agent <ip> <port> --list [token]        List remote agent disks
  amele linux disk --agent <ip> <port> <id> <out_dir> [token] Acquire remote image
  amele linux disk --ssh <ip> <port> --list [user] [pass]    List disks via SSH
  amele linux disk --ssh <ip> <port> <id> <case> [user]      Acquire image via SSH

RAM OPERATIONS (AVML):
  amele linux ram --status                                   Show AVML and memory status
  amele linux ram install                                    Auto-download and install AVML (sudo)
  amele linux ram <case> [tool_path] [raw|aff4]              Acquire local live RAM (requires sudo)
  amele linux ram analyze <ram> [windows|linux] [symbols]    RAM image summary analysis
  amele linux ram strings <ram>                              Search IOCs / strings in RAM image
  amele linux ram carve <ram> <output_dir>                   Carve files from RAM image
  amele linux ram processes <ram> [windows|linux] [symbols]  List processes from RAM dump
  amele linux ram --agent <ip> <port> <case> [token]         Acquire RAM via remote agent
  amele linux ram --ssh <ip> <port> <case> [user] [pass]     Acquire RAM via SSH"#
            )
        );
        return Ok(());
    }
    match args[0].as_str() {
        "disk" => {
            let mut sub_args = args[1..].to_vec();
            if sub_args.first().map(|s| s.as_str()) == Some("acquire") {
                sub_args.remove(0);
            }
            if sub_args.is_empty() || sub_args.iter().any(|a| a == "--list" || a == "list") {
                return disk_list_command_with_args(sub_args);
            }
            if sub_args.first().map(|s| s.as_str()) == Some("analyze") {
                return image_analyze_command(sub_args[1..].to_vec());
            }
            if let Some(pos) = sub_args
                .iter()
                .position(|a| a == "--agent" || a == "--remote")
            {
                sub_args.remove(pos);
                if sub_args.iter().any(|a| a == "--list" || a == "list") {
                    let rem: Vec<String> = sub_args
                        .into_iter()
                        .filter(|a| a != "--list" && a != "list")
                        .collect();
                    return remote_disks_command(rem);
                }
                return remote_image_command(sub_args);
            }
            if let Some(pos) = sub_args.iter().position(|a| a == "--ssh") {
                sub_args.remove(pos);
                if sub_args.iter().any(|a| a == "--list" || a == "list") {
                    let rem: Vec<String> = sub_args
                        .into_iter()
                        .filter(|a| a != "--list" && a != "list")
                        .collect();
                    return ssh_disks_command(rem);
                }
                return ssh_image_command(sub_args);
            }
            local_image_command(sub_args)
        }
        "ram" => {
            let mut sub_args = args[1..].to_vec();
            if sub_args.first().map(|s| s.as_str()) == Some("acquire") {
                sub_args.remove(0);
            }
            if sub_args.is_empty() || sub_args.iter().any(|a| a == "--status" || a == "status") {
                return linux_ram_status_command(sub_args);
            }
            if sub_args.first().map(|s| s.as_str()) == Some("install")
                || sub_args.first().map(|s| s.as_str()) == Some("download")
            {
                return linux_ram_install_command();
            }
            if sub_args.first().map(|s| s.as_str()) == Some("analyze")
                || sub_args.first().map(|s| s.as_str()) == Some("summary")
            {
                let mut rem = sub_args[1..].to_vec();
                if rem.len() == 1 {
                    rem.push("linux".to_string());
                }
                return ram_summary_command(rem);
            }
            if sub_args.first().map(|s| s.as_str()) == Some("strings") {
                return ram_strings_command(sub_args[1..].to_vec());
            }
            if sub_args.first().map(|s| s.as_str()) == Some("carve") {
                return ram_carve_command(sub_args[1..].to_vec());
            }
            if sub_args.first().map(|s| s.as_str()) == Some("processes") {
                let mut rem = sub_args[1..].to_vec();
                if rem.len() == 1 {
                    rem.push("linux".to_string());
                }
                return ram_processes_command(rem);
            }
            if let Some(pos) = sub_args
                .iter()
                .position(|a| a == "--agent" || a == "--remote")
            {
                sub_args.remove(pos);
                return remote_ram_command(sub_args);
            }
            if let Some(pos) = sub_args.iter().position(|a| a == "--ssh") {
                sub_args.remove(pos);
                return ssh_ram_command(sub_args);
            }
            let mut prep = vec!["avml".to_string()];
            prep.extend(sub_args);
            local_ram_command(prep)
        }
        other => Err(format!(
            "{} linux {}",
            t_cli("Bilinmeyen alt komut:", "Unknown subcommand:"),
            other
        )),
    }
}

fn windows_cli_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele Windows Edinim ve Analiz Komutlari

KULLANIM:
  amele windows <disk|ram> [secenekler]

DISK ISLEMLERI:
  amele windows disk --list                                    Yerel diskleri listele
  amele windows disk <kaynak> <vaka> [disk_adi] [raw|aff4]     Yerel disk imaji al
  amele windows disk analyze <imaj> [mount_klasoru]            Disk imajini yapisal olarak analiz et
  amele windows disk --agent <ip> <port> --list [token]        Uzak agent disklerini listele
  amele windows disk --agent <ip> <port> <id> <cikti> [token]  Uzak agent uzerinden imaj al
  amele windows disk --ssh <ip> <port> --list [user] [pass]    SSH ile diskleri listele
  amele windows disk --ssh <ip> <port> <id> <vaka> [user]      SSH ile agent'siz imaj al

RAM ISLEMLERI (WinPMEM):
  amele windows ram --status                                   WinPMEM durumunu goster
  amele windows ram install                                    WinPMEM aracini indir
  amele windows ram <vaka> [arac_yolu] [raw|aff4]              Yerel canli RAM imaji al
  amele windows ram analyze <ram> [windows|linux] [symbols]    RAM imaji ozet analizi yap
  amele windows ram strings <ram>                              RAM imajinda IOC / dizgi aramasi yap
  amele windows ram carve <ram> <cikti_klasoru>                RAM imajindan dosya cikar (carving)
  amele windows ram processes <ram> [windows|linux] [symbols]  RAM icinden calisan surecleri listele
  amele windows ram --agent <ip> <port> <vaka> [token]         Uzak agent'tan RAM imaji al
  amele windows ram --ssh <ip> <port> <vaka> [user] [pass]     SSH ile canli RAM dokumu al"#,
                r#"Amele Windows Acquisition & Analysis Commands

USAGE:
  amele windows <disk|ram> [options]

DISK OPERATIONS:
  amele windows disk --list                                    List local disks
  amele windows disk <source> <case> [disk_name] [raw|aff4]    Acquire local disk image
  amele windows disk analyze <image> [mount_dir]               Analyze disk image structurally
  amele windows disk --agent <ip> <port> --list [token]        List remote agent disks
  amele windows disk --agent <ip> <port> <id> <out_dir> [token] Acquire remote image
  amele windows disk --ssh <ip> <port> --list [user] [pass]    List disks via SSH
  amele windows disk --ssh <ip> <port> <id> <case> [user]      Acquire image via SSH

RAM OPERATIONS (WinPMEM):
  amele windows ram --status                                   Show WinPMEM status
  amele windows ram install                                    Download WinPMEM tool
  amele windows ram <case> [tool_path] [raw|aff4]              Acquire local live RAM
  amele windows ram analyze <ram> [windows|linux] [symbols]    RAM image summary analysis
  amele windows ram strings <ram>                              Search IOCs / strings in RAM image
  amele windows ram carve <ram> <output_dir>                   Carve files from RAM image
  amele windows ram processes <ram> [windows|linux] [symbols]  List processes from RAM dump
  amele windows ram --agent <ip> <port> <case> [token]         Acquire RAM via remote agent
  amele windows ram --ssh <ip> <port> <case> [user] [pass]     Acquire RAM via SSH"#
            )
        );
        return Ok(());
    }
    match args[0].as_str() {
        "disk" => {
            let mut sub_args = args[1..].to_vec();
            if sub_args.first().map(|s| s.as_str()) == Some("acquire") {
                sub_args.remove(0);
            }
            if sub_args.is_empty() || sub_args.iter().any(|a| a == "--list" || a == "list") {
                return disk_list_command_with_args(sub_args);
            }
            if sub_args.first().map(|s| s.as_str()) == Some("analyze") {
                return image_analyze_command(sub_args[1..].to_vec());
            }
            if let Some(pos) = sub_args
                .iter()
                .position(|a| a == "--agent" || a == "--remote")
            {
                sub_args.remove(pos);
                if sub_args.iter().any(|a| a == "--list" || a == "list") {
                    let rem: Vec<String> = sub_args
                        .into_iter()
                        .filter(|a| a != "--list" && a != "list")
                        .collect();
                    return remote_disks_command(rem);
                }
                return remote_image_command(sub_args);
            }
            if let Some(pos) = sub_args.iter().position(|a| a == "--ssh") {
                sub_args.remove(pos);
                if sub_args.iter().any(|a| a == "--list" || a == "list") {
                    let rem: Vec<String> = sub_args
                        .into_iter()
                        .filter(|a| a != "--list" && a != "list")
                        .collect();
                    return ssh_disks_command(rem);
                }
                return ssh_image_command(sub_args);
            }
            local_image_command(sub_args)
        }
        "ram" => {
            let mut sub_args = args[1..].to_vec();
            if sub_args.first().map(|s| s.as_str()) == Some("acquire") {
                sub_args.remove(0);
            }
            if sub_args.is_empty() || sub_args.iter().any(|a| a == "--status" || a == "status") {
                return windows_ram_status_command(sub_args);
            }
            if sub_args.first().map(|s| s.as_str()) == Some("install")
                || sub_args.first().map(|s| s.as_str()) == Some("download")
            {
                return windows_ram_install_command();
            }
            if sub_args.first().map(|s| s.as_str()) == Some("analyze")
                || sub_args.first().map(|s| s.as_str()) == Some("summary")
            {
                let mut rem = sub_args[1..].to_vec();
                if rem.len() == 1 {
                    rem.push("windows".to_string());
                }
                return ram_summary_command(rem);
            }
            if sub_args.first().map(|s| s.as_str()) == Some("strings") {
                return ram_strings_command(sub_args[1..].to_vec());
            }
            if sub_args.first().map(|s| s.as_str()) == Some("carve") {
                return ram_carve_command(sub_args[1..].to_vec());
            }
            if sub_args.first().map(|s| s.as_str()) == Some("processes") {
                let mut rem = sub_args[1..].to_vec();
                if rem.len() == 1 {
                    rem.push("windows".to_string());
                }
                return ram_processes_command(rem);
            }
            if let Some(pos) = sub_args
                .iter()
                .position(|a| a == "--agent" || a == "--remote")
            {
                sub_args.remove(pos);
                return remote_ram_command(sub_args);
            }
            if let Some(pos) = sub_args.iter().position(|a| a == "--ssh") {
                sub_args.remove(pos);
                return ssh_ram_command(sub_args);
            }
            let mut prep = vec!["winpmem".to_string()];
            prep.extend(sub_args);
            local_ram_command(prep)
        }
        other => Err(format!(
            "{} windows {}",
            t_cli("Bilinmeyen alt komut:", "Unknown subcommand:"),
            other
        )),
    }
}

fn android_cli_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele Android Adli Bilisim Komutlari

KULLANIM:
  amele android <alt-komut> [argumanlar]

CIHAZ VE TESHIS:
  devices                         Bagli Android cihazlari listele
  status                          ADB durumunu denetle
  install                         ADB'yi sistem paket yoneticisiyle kur
  profile <seri_no>               Cihaz detay profilini yazdir
  capabilities <seri_no>          Edinim uyumluluk raporu
  lemon <seri_no>                 Lemon fiziksel RAM on kontrolu

VERI EDINIMI:
  logical <seri_no> <vaka> [mod]  Mantiksal edinim (quick|full|root|volatile)
  filesystem <seri_no> <vaka>     Dosya sistemi /data imaji (--root)
  ram <seri_no> <vaka> [mod]      RAM dokumu (volatile|root|physical)

UZAK BAGLANTI:
  connect <host> [port]           Uzak ADB baglantisi (tcp|mesh)
  disconnect <seri_no>            Uzak ADB baglantisini kes
  analysis <vaka>                 Vaka cikti analiz ozeti"#,
                r#"Amele Android Mobile Forensics Commands

USAGE:
  amele android <subcommand> [args]

DEVICE & DIAGNOSTICS:
  devices                         List connected Android devices
  status                          Check ADB daemon status
  install                         Install ADB via package manager
  profile <serial>                Print device details and encryption info
  capabilities <serial>           Acquisition compatibility report
  lemon <serial>                  Lemon physical RAM preflight

ACQUISITION:
  logical <serial> <case> [prof]  Logical acquisition (quick|full|root|volatile)
  filesystem <serial> <case>      Filesystem /data image (--root)
  ram <serial> <case> [mode]      Live RAM dump (volatile|root|physical)

REMOTE CONNECTION:
  connect <host> [port]           Connect remote ADB (tcp|mesh)
  disconnect <serial>             Disconnect remote ADB
  analysis <case>                 Case output analysis summary"#
            )
        );
        return Ok(());
    }
    let sub = args[0].as_str();
    let sub_args = args[1..].to_vec();
    match sub {
        "devices" | "list" => android_devices_command(),
        "status" | "adb-status" => android_adb_status_command(),
        "install" | "adb-install" => android_adb_install_command(),
        "profile" => android_profile_command(sub_args),
        "logical" => android_logical_command(sub_args),
        "filesystem" => android_filesystem_command(sub_args),
        "ram" => android_ram_command(sub_args),
        "capabilities" => android_capabilities_command(sub_args),
        "lemon" | "lemon-preflight" => android_lemon_preflight_command(sub_args),
        "connect" | "remote-connect" => android_remote_connect_command(sub_args),
        "disconnect" | "remote-disconnect" => android_remote_disconnect_command(sub_args),
        "analysis" | "case-analysis" => android_case_analysis_command(sub_args),
        other => Err(format!(
            "{} android {}",
            t_cli(
                "Bilinmeyen Android alt komutu:",
                "Unknown Android subcommand:"
            ),
            other
        )),
    }
}

fn ios_cli_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele iOS Adli Bilisim Komutlari

KULLANIM:
  amele ios <profile|normalize> <backup_klasoru> [vaka]

KOMUTLAR:
  profile <backup_klasoru>            iOS backup metadata bilgilerini yazdir
  normalize <backup_klasoru> <vaka>   iOS backup'i vaka klasorune normalize et"#,
                r#"Amele iOS Forensics Commands

USAGE:
  amele ios <profile|normalize> <backup_dir> [case]

COMMANDS:
  profile <backup_dir>                Inspect iOS backup metadata
  normalize <backup_dir> <case>       Normalize iOS backup into case folder"#
            )
        );
        return Ok(());
    }
    let sub = args[0].as_str();
    let sub_args = args[1..].to_vec();
    match sub {
        "profile" | "backup-profile" => ios_backup_profile_command(sub_args),
        "normalize" | "backup-normalize" => ios_backup_normalize_command(sub_args),
        other => Err(format!(
            "{} ios {}",
            t_cli("Bilinmeyen iOS alt komutu:", "Unknown iOS subcommand:"),
            other
        )),
    }
}

fn docker_cli_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele Docker Adli Bilisim Komutlari

KULLANIM:
  amele docker <status|list|logs|acquire> [--agent <ip> <port>] [secenekler]

KOMUTLAR:
  status [kok_dizin]              Docker daemon durumunu ve riskleri denetle
  list [kok_dizin]                Konteynerleri ve secret risklerini listele
  logs <id> [tail] [kok_dizin]    Konteyner loglarini oku
  acquire <id> <vaka> [kok_dizin] UpperDir drift, config ve loglari vakaya al

UZAK AGENT SECENEGI:
  --agent <ip> <port> <komut> [token]  Uzak agent uzerinden Docker incelemesi yap"#,
                r#"Amele Docker Forensics Commands

USAGE:
  amele docker <status|list|logs|acquire> [--agent <ip> <port>] [options]

COMMANDS:
  status [root]                   Inspect Docker daemon status and risks
  list [root]                     List containers and secret risks
  logs <id> [tail] [root]         Inspect container logs
  acquire <id> <case> [root]      Acquire UpperDir drift, configs and logs

REMOTE AGENT OPTION:
  --agent <ip> <port> <cmd> [token] Run Docker commands on remote agent"#
            )
        );
        return Ok(());
    }
    let mut args = args;
    let agent_pos = args.iter().position(|a| a == "--agent" || a == "--remote");
    if let Some(pos) = agent_pos {
        if pos + 2 < args.len() {
            let ip = args.remove(pos + 1);
            let port = args.remove(pos + 1);
            args.remove(pos);
            let sub = if args.is_empty() {
                "status"
            } else {
                args[0].as_str()
            };
            let sub_args = if args.is_empty() {
                vec![]
            } else {
                args[1..].to_vec()
            };
            return match sub {
                "status" => {
                    let mut p = vec![ip, port];
                    p.extend(sub_args);
                    docker_remote_status_command(p)
                }
                "list" | "containers" => {
                    let mut p = vec![ip, port];
                    p.extend(sub_args);
                    docker_remote_list_command(p)
                }
                "logs" => {
                    if sub_args.is_empty() {
                        return Err(t_cli(
                            "Kullanim: amele docker --agent <ip> <port> logs <id> [tail] [token]",
                            "Usage: amele docker --agent <ip> <port> logs <id> [tail] [token]",
                        ));
                    }
                    let id = sub_args[0].clone();
                    let mut p = vec![ip, port, id];
                    p.extend(sub_args[1..].to_vec());
                    docker_remote_logs_command(p)
                }
                "acquire" => {
                    if sub_args.is_empty() {
                        return Err(t_cli(
                            "Kullanim: amele docker --agent <ip> <port> acquire <id> [vaka] [token]",
                            "Usage: amele docker --agent <ip> <port> acquire <id> [case] [token]",
                        ));
                    }
                    let id = sub_args[0].clone();
                    let mut p = vec![ip, port, id];
                    p.extend(sub_args[1..].to_vec());
                    docker_remote_acquire_command(p)
                }
                other => Err(format!(
                    "{} docker {}",
                    t_cli(
                        "Bilinmeyen Docker alt komutu:",
                        "Unknown Docker subcommand:"
                    ),
                    other
                )),
            };
        }
    }

    let sub = args[0].as_str();
    let sub_args = args[1..].to_vec();
    match sub {
        "status" => docker_status_command(sub_args),
        "list" | "containers" => docker_list_command(sub_args),
        "logs" => docker_logs_command(sub_args),
        "acquire" => docker_acquire_command(sub_args),
        other => Err(format!(
            "{} docker {}",
            t_cli(
                "Bilinmeyen Docker alt komutu:",
                "Unknown Docker subcommand:"
            ),
            other
        )),
    }
}

fn profile_cli_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele Profil Yonetimi Komutlari

KULLANIM:
  amele profile <alt-komut> [secenekler]

KOMUTLAR:
  list                            Yerel ve aktif profilleri listele
  create <isim> <kullanici> [tr|en] [dark|light] [--direct]  Yeni profil olustur
  use <kullanici> [--direct]      Aktif profili sec
  logout                          Otomatik profil oturumunu kapat
  sync                            Online lisans ve rolleri senkronize et"#,
                r#"Amele Profile Management Commands

USAGE:
  amele profile <subcommand> [options]

COMMANDS:
  list                            List local and active profiles
  create <name> <user> [tr|en] [dark|light] [--direct] Create new profile
  use <user> [--direct]           Select active profile
  logout                          Disable automatic login
  sync                            Synchronize online licenses and roles"#
            )
        );
        return Ok(());
    }
    let sub = args[0].as_str();
    let sub_args = args[1..].to_vec();
    match sub {
        "list" => profile_list_command(),
        "create" => profile_create_command(sub_args),
        "use" | "select" => profile_use_command(sub_args),
        "logout" => profile_logout_command(),
        "sync" | "online-sync" => profile_online_sync_command(),
        other => Err(format!(
            "{} profile {}",
            t_cli(
                "Bilinmeyen profil alt komutu:",
                "Unknown profile subcommand:"
            ),
            other
        )),
    }
}

fn case_cli_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele Vaka Yonetimi Komutlari

KULLANIM:
  amele case <alt-komut> [argumanlar]

KOMUTLAR:
  list                            Mevcut vakalari listele
  create <vaka>                   Yeni vaka deposu olustur
  info <vaka>                     Vaka detaylarini ve delil istatistiklerini goster
  export <vaka> [dosya]           Vakayi .amelecase paketine aktar
  import <dosya>                  .amelecase paketini iceri aktar
  verify <dosya>                  .amelecase paket butunlugunu dogrula"#,
                r#"Amele Case Management Commands

USAGE:
  amele case <subcommand> [args]

COMMANDS:
  list                            List existing cases
  create <case>                   Create new case vault
  info <case>                     Show case details and evidence statistics
  export <case> [file]            Export case to .amelecase package
  import <file>                   Import .amelecase package
  verify <file>                   Verify .amelecase package integrity"#
            )
        );
        return Ok(());
    }
    let sub = args[0].as_str();
    let sub_args = args[1..].to_vec();
    match sub {
        "list" => case_list_command(sub_args),
        "create" | "new" => case_create_command(sub_args),
        "info" | "show" => case_info_command(sub_args),
        "export" => case_export_command(sub_args),
        "import" => case_import_command(sub_args),
        "verify" => case_verify_command(sub_args),
        other => Err(format!(
            "{} case {}",
            t_cli("Bilinmeyen vaka alt komutu:", "Unknown case subcommand:"),
            other
        )),
    }
}

fn mount_cli_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele Imaj Baglama (Mount) Komutlari

KULLANIM:
  amele mount <alt-komut|imaj_dosyasi> [secenekler]

KOMUTLAR:
  list                            Aktif bagli adli imajlari listele
  cleanup [vaka]                  Bagli imajlari guvenle coz (unmount)
  <imaj_dosyasi> [baglama_noktasi] Adli disk imajini salt-okunur bagla (sudo gerektirir)"#,
                r#"Amele Image Mount Commands

USAGE:
  amele mount <subcommand|image_file> [options]

COMMANDS:
  list                            List active mounted images
  cleanup [case]                  Safely unmount forensic images (alias: unmount)
  <image_file> [mount_point]      Mount forensic disk image read-only (requires sudo)"#
            )
        );
        return Ok(());
    }
    let sub = args[0].as_str();
    let sub_args = args[1..].to_vec();
    match sub {
        "list" | "mounts" => mount_list_command(),
        "cleanup" | "unmount" | "umount" => mount_cleanup_command(sub_args),
        _ => mount_file_command(args),
    }
}

/// Kullanıcıya desteklenen teknik CLI komutlarını gösterir.
fn print_help() {
    if is_cli_english() {
        println!(
            r#"Amele Forensic Tool CLI (v0.0.19)

USAGE:
  amele <command> [subcommand] [options]
  amele [options]

ACQUISITION & ANALYSIS:
  linux <disk|ram>        Linux forensic disk & live RAM acquisition (local, --agent, --ssh)
  windows <disk|ram>      Windows forensic disk & live RAM acquisition (local, --agent, --ssh)
  ram [subcommand]        Live RAM status, installer, acquisition & analysis
  disk [subcommand]       Disk drives listing, acquisition & image analysis
  android <subcommand>    Android mobile forensics (logical, filesystem, RAM, diagnostics)
  ios <subcommand>        iOS backup metadata analysis & case normalization
  docker <subcommand>     Docker container forensics (drift, configs, logs)

MANAGEMENT & EVIDENCE:
  profile <subcommand>    Manage local analyst profiles & online synchronization
  case <subcommand>       Case management (list, create, info, export, import, verify)
  mount <subcommand>      Mount/unmount forensic disk images (requires sudo)
  hash <file> [algo]      Calculate cryptographic hash (md5, sha1, sha256, sha512)
  verify <image> <sha256> Verify forensic image SHA-256 checksum
  wireguard <file>        Generate secure WireGuard VPN configuration
  update [--json]         Check for software updates
  completion <shell>      Generate shell autocompletions (bash, zsh, fish)

USER INTERFACE:
  ui                      Launch native desktop application
  ui-browser              Launch developer browser UI for debugging

GLOBAL OPTIONS:
  -h, --help              Show help information
  -V, --version           Show version information
  -q, --quiet, --no-logo  Suppress ASCII logo banner
  -v, --verbose           Enable verbose / debug logs
  --lang <tr|en>          Switch CLI language (Turkish or English)
  --profile <username>    Run using specific analyst profile
  --json                  Output structured results as JSON

Run 'amele <command> --help' for detailed sub-command usage."#
        );
    } else {
        println!(
            r#"Amele Forensic Tool CLI (v0.0.19)

KULLANIM:
  amele <komut> [alt-komut] [secenekler]
  amele [secenekler]

ADLI EDINIM VE ANALIZ KOMUTLARI:
  linux <disk|ram>        Linux adli disk ve canli RAM edinimi (yerel, --agent, --ssh)
  windows <disk|ram>      Windows adli disk ve canli RAM edinimi (yerel, --agent, --ssh)
  ram [alt-komut]         Canli RAM durumu, yukleyici, edinim ve analiz komutlari
  disk [alt-komut]        Disk surucu listeleme, imaj alma ve analiz komutlari
  android <alt-komut>     Android mobil edinim (mantiksal, dosya sistemi, RAM, teshis)
  ios <alt-komut>         iOS yedek analizi ve vaka normalizasyonu
  docker <alt-komut>      Docker konteyner adli bilisimi (drift, log, config)

YONETIM VE DELIL ISLEMLERI:
  profile <alt-komut>     Yerel ve online analist profillerini yonet
  case <alt-komut>        Vaka yonetimi (listeleme, olusturma, detay, .amelecase paketleme)
  mount <alt-komut>       Adli disk imaji baglama (mount) ve temizleme (sudo)
  hash <dosya> [algo]     Dosya ozeti hesapla (md5, sha1, sha256, sha512)
  verify <imaj> <sha256>  Imaj SHA-256 hash dogrulamasi yap
  wireguard <dosya>       Guvenli WireGuard VPN yapilandirmasi uret
  update [--json]         Yazilim guncelleme kontrolu
  completion <kabuk>      Kabuk otomatik tamamlama uret (bash, zsh, fish)

ARAYUZ:
  ui                      Masaustu yerel penceresini ac
  ui-browser              Tarayici gelistirici/debug modunda ac

GENEL SECENEKLER:
  -h, --help              Yardim bilgisini goster
  -V, --version           Surum bilgisini goster
  -q, --quiet, --no-logo  ASCII logo basligini gizle
  -v, --verbose           Ayrintili (debug) gunlukleri konsola yaz
  --lang <tr|en>          CLI dilini sec (Turkce veya Ingilizce)
  --profile <kullanici>   Belirtilen analist profili ile calis
  --json                  Yapilandirilmis JSON ciktisi uret

Detayli kullanim icin: amele <komut> --help"#
        );
    }
}

/// Varsayılan uygulama ayarlarını JSON olarak stdout'a yazar.
fn print_default_settings() -> Result<(), String> {
    let settings = AppSettings::default();
    println!(
        "{}",
        serde_json::to_string_pretty(&settings).map_err(|err| err.to_string())?
    );
    Ok(())
}

/// Yerel profilleri JSON olarak listeler.
fn profile_list_command() -> Result<(), String> {
    let state = amele::profile::bootstrap_profiles().map_err(|err| err.to_string())?;
    print_json(&json!({
        "profiles": state.profiles,
        "active_profile": state.active_profile,
        "base_dir": state.base_dir,
    }))
}

/// CLI üzerinden yeni profil oluşturur.
fn profile_create_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let open_directly = remove_direct_flag(&mut args);
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: profile-create <isim_soyisim> <kullanici> [tr|en] [dark|light] [--direct]",
            "Usage: profile-create <full_name> <username> [tr|en] [dark|light] [--direct]",
        ));
    }
    let language = args.get(2).map(String::as_str).unwrap_or("tr");
    let theme = args.get(3).map(String::as_str).unwrap_or("dark");
    let profile =
        amele::profile::create_profile(&args[0], &args[1], language, theme, open_directly)
            .map_err(|err| err.to_string())?;
    print_json(&profile)
}

/// CLI üzerinden mevcut profili seçer.
fn profile_use_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let open_directly = remove_direct_flag(&mut args);
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: profile-use <kullanici> [--direct]",
            "Usage: profile-use <username> [--direct]",
        ));
    }
    let profile =
        amele::profile::select_profile(&args[0], open_directly).map_err(|err| err.to_string())?;
    print_json(&profile)
}

/// Aktif/otomatik profil seçimini kapatır.
fn profile_logout_command() -> Result<(), String> {
    amele::profile::logout_profile().map_err(|err| err.to_string())?;
    print_json(&json!({ "ok": true }))
}

/// CLI üzerinden aktif profilin online bilgilerini yeniler.
fn profile_online_sync_command() -> Result<(), String> {
    let profile = amele::profile::sync_active_online_profile()
        .map_err(|err| crate_diagnostic(err.to_string()))?;
    print_json(&json!({
        "profile": profile,
        "access": amele::profile::mobile_tools_access(),
    }))
}

/// Verilen dosya için seçilen hash algoritmasını çalıştırır.
fn hash_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: hash <dosya> [algoritma]",
            "Usage: hash <file> [algorithm]",
        ));
    }
    let path = PathBuf::from(&args[0]);
    let algorithm = args
        .get(1)
        .and_then(|value| HashAlgorithm::parse(value))
        .unwrap_or(HashAlgorithm::Sha256);
    let value = hash::calculate_file_hash(&path, algorithm).map_err(|err| err.to_string())?;
    println!("{}  {}", algorithm.name(), value);
    Ok(())
}

/// Yerel disk veya dosya kaynağını vaka klasörüne imaj olarak yazar.
fn local_image_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    let selected_format = extract_output_format(&mut args)?;
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: local-image <kaynak> <vaka> [disk_adı] [raw|aff4]",
            "Usage: local-image <source> <case> [disk_name] [raw|aff4]",
        ));
    }
    let source = PathBuf::from(&args[0]);

    let is_block_dev = source.starts_with("/dev/") || source.to_string_lossy().starts_with(r"\\.\");
    if is_block_dev {
        ensure_root_or_elevate(&t_cli(
            "Yerel Disk İmajı Edinimi",
            "Local Disk Image Acquisition",
        ))?;
    }

    let vault = cli_case_vault(&args[1])?;
    let disk_name = args.get(2).map(String::as_str).unwrap_or_else(|| {
        source
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("disk")
    });
    let raw_target = vault.outputs_dir.join(format!(
        "{}_{}.img",
        cli_safe_stem(disk_name),
        cli_timestamp()
    ));
    let plan = output_format::plan_output(&raw_target, selected_format);
    let task = disk::DiskAcquisitionTask::new(&source, &plan.working_path);
    let result = disk::run_disk_acquisition(&task, |done, total| {
        print_progress("imaj", done, total);
    })
    .map_err(|err| crate_diagnostic(err.to_string()))?;
    let finalized = output_format::finalize_output(
        &plan,
        "disk",
        disk_name,
        &vault.case_name,
        result.sha256.clone(),
    )?;
    if json_output {
        print_json(&json!({
            "case": vault.case_name,
            "target_path": finalized.target_path,
            "bytes_copied": result.bytes_copied,
            "total_bytes": result.total_bytes,
            "sha256": finalized.sha256,
            "raw_sha256": finalized.raw_sha256,
            "output_format": finalized.format.as_str(),
        }))
    } else {
        println!("============================================================");
        println!(
            "       {}",
            t_cli(
                "Disk İmajı Edinimi Başarıyla Tamamlandı",
                "Disk Image Acquisition Completed Successfully"
            )
        );
        println!("============================================================");
        println!(
            "  {:<20}: {}",
            t_cli("Vaka Adı", "Case Name"),
            vault.case_name
        );
        println!(
            "  {:<20}: {}",
            t_cli("Kaynak Aygıt", "Source Device"),
            source.display()
        );
        println!(
            "  {:<20}: {}",
            t_cli("Hedef Dosya", "Target File"),
            finalized.target_path.display()
        );
        println!(
            "  {:<20}: {}",
            t_cli("Kopyalanan Boyut", "Copied Size"),
            format_bytes(result.bytes_copied)
        );
        println!(
            "  {:<20}: {}",
            t_cli("İmaj Formatı", "Image Format"),
            finalized.format.as_str().to_uppercase()
        );
        println!("  {:<20}: {}", "SHA-256", finalized.sha256);
        if let Some(raw_h) = &finalized.raw_sha256 {
            if finalized.format.as_str() != "raw" {
                println!("  {:<20}: {}", t_cli("Ham SHA-256", "Raw SHA-256"), raw_h);
            }
        }
        println!("============================================================");
        println!(
            "[✓] {}",
            t_cli(
                "Disk imajı vaka deposuna kaydedildi ve doğrulandı.",
                "Disk image saved to case vault and verified."
            )
        );
        Ok(())
    }
}

/// AVML veya WinPMEM ile yerel RAM imajı alır.
fn local_ram_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    let selected_format = extract_output_format(&mut args)?;
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: local-ram <avml|winpmem> <vaka> [arac_yolu] [raw|aff4]",
            "Usage: local-ram <avml|winpmem> <case> [tool_path] [raw|aff4]",
        ));
    }
    let tool = args[0].to_ascii_lowercase();

    ensure_root_or_elevate(&t_cli("Canlı RAM Edinimi", "Live RAM Acquisition"))?;

    #[cfg(target_os = "linux")]
    if tool == "avml" && ram::find_avml(None).is_none() {
        println!(
            "{}",
            t_cli(
                "[*] AVML aracı sistemde bulunamadı. Otomatik olarak indirilip kuruluyor...",
                "[*] AVML tool not found on system. Downloading and installing automatically..."
            )
        );
        linux_ram_install_command()?;
    }

    let vault = cli_case_vault(&args[1])?;
    let raw_target = vault.ram_dir.join(format!("ram_{}.raw", cli_timestamp()));
    let plan = output_format::plan_output(&raw_target, selected_format);
    let candidate = args.get(2).map(Path::new).filter(|path| path.exists());
    let token = ram::CancellationToken::default();
    let result = match tool.as_str() {
        "avml" => ram::acquire_with_avml(&plan.working_path, candidate, &token, |done, total| {
            print_progress("ram", done, total);
        }),
        "winpmem" => {
            ram::acquire_with_winpmem(&plan.working_path, candidate, &token, |done, total| {
                print_progress("ram", done, total);
            })
        }
        _ => {
            return Err(t_cli(
                "Araç 'avml' veya 'winpmem' olmalıdır.",
                "Tool must be avml or winpmem.",
            ));
        }
    }
    .map_err(|err| crate_diagnostic(err.to_string()))?;
    let finalized = output_format::finalize_output(&plan, "ram", &tool, &vault.case_name, None)?;
    if json_output {
        print_json(&json!({
            "case": vault.case_name,
            "target_path": finalized.target_path,
            "bytes_written": result.bytes_written,
            "sha256": finalized.sha256,
            "raw_sha256": finalized.raw_sha256,
            "output_format": finalized.format.as_str(),
        }))
    } else {
        println!("============================================================");
        println!(
            "       {}",
            t_cli(
                "Canlı RAM Edinimi Başarıyla Tamamlandı",
                "Live RAM Acquisition Completed Successfully"
            )
        );
        println!("============================================================");
        println!(
            "  {:<20}: {}",
            t_cli("Vaka Adı", "Case Name"),
            vault.case_name
        );
        println!(
            "  {:<20}: {}",
            t_cli("Kullanılan Araç", "Tool Used"),
            tool.to_uppercase()
        );
        println!(
            "  {:<20}: {}",
            t_cli("Hedef Dosya", "Target File"),
            finalized.target_path.display()
        );
        println!(
            "  {:<20}: {}",
            t_cli("Yazılan Boyut", "Written Size"),
            format_bytes(result.bytes_written)
        );
        println!(
            "  {:<20}: {}",
            t_cli("İmaj Formatı", "Image Format"),
            finalized.format.as_str().to_uppercase()
        );
        println!("  {:<20}: {}", "SHA-256", finalized.sha256);
        if let Some(raw_h) = &finalized.raw_sha256 {
            if finalized.format.as_str() != "raw" {
                println!("  {:<20}: {}", t_cli("Ham SHA-256", "Raw SHA-256"), raw_h);
            }
        }
        println!("============================================================");
        println!(
            "[✓] {}",
            t_cli(
                "Canlı RAM imajı vaka deposuna kaydedildi ve doğrulandı.",
                "Live RAM image saved to case vault and verified."
            )
        );
        Ok(())
    }
}

/// Uzak agent üzerinde RAM edinimini başlatır ve sonucu vaka klasörüne indirir.
fn remote_ram_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    let selected_format = extract_output_format(&mut args)?;
    if args.len() < 3 {
        return Err(t_cli(
            "Kullanim: remote-ram <ip> <port> <vaka> [token] [raw|aff4]",
            "Usage: remote-ram <ip> <port> <case> [token] [raw|aff4]",
        ));
    }
    let ip = &args[0];
    let port = parse_port(&args[1])?;
    let vault = cli_case_vault(&args[2])?;
    let token = args.get(3).cloned();
    let target = vault
        .ram_dir
        .join(format!("{}_ram_{}.raw", cli_safe_stem(ip), cli_timestamp()));
    let plan = output_format::plan_output(&target, selected_format);
    let remote_file = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("remote_ram.raw")
        .to_string();

    let mut connection = RemoteConnection::connect(ip, port, token)
        .map_err(|err| crate_diagnostic(err.to_string()))?;
    let job_id = format!("cli-{}", std::process::id());
    let remote_result = connection
        .start_remote_ram(
            &remote_file,
            Some(&job_id),
            selected_format,
            |done, total| {
                print_progress("remote-ram", done, total);
            },
        )
        .map_err(|err| crate_diagnostic(err.to_string()))?;
    let download = connection
        .download_ram_file(
            &remote_file,
            &plan.working_path,
            Some(&job_id),
            |done, total| {
                print_progress("download", done, total);
            },
        )
        .map_err(|err| crate_diagnostic(err.to_string()))?;
    let finalized = output_format::finalize_output(
        &plan,
        "ram",
        ip,
        &vault.case_name,
        download.sha256.or(remote_result.sha256),
    )?;
    if json_output {
        print_json(&json!({
            "case": vault.case_name,
            "remote_job_id": remote_result.job_id,
            "target_path": finalized.target_path,
            "bytes_transferred": download.bytes_transferred,
            "remote_bytes": remote_result.total_size,
            "sha256": finalized.sha256,
            "raw_sha256": finalized.raw_sha256,
            "output_format": finalized.format.as_str(),
        }))
    } else {
        println!("============================================================");
        println!(
            "       {}",
            t_cli(
                "Uzak RAM Edinimi Tamamlandı",
                "Remote RAM Acquisition Completed"
            )
        );
        println!("============================================================");
        println!(
            "  {:<20}: {}",
            t_cli("Vaka Adı", "Case Name"),
            vault.case_name
        );
        println!(
            "  {:<20}: {}:{}",
            t_cli("Uzak Agent", "Remote Agent"),
            ip,
            port
        );
        println!(
            "  {:<20}: {}",
            t_cli("İndirilen Dosya", "Downloaded File"),
            finalized.target_path.display()
        );
        println!(
            "  {:<20}: {}",
            t_cli("Aktarılan Boyut", "Transferred Size"),
            format_bytes(download.bytes_transferred)
        );
        println!("  {:<20}: {}", "SHA-256", finalized.sha256);
        println!("============================================================");
        println!(
            "[✓] {}",
            t_cli(
                "Uzak RAM imajı başarıyla vaka deposuna aktarıldı.",
                "Remote RAM image transferred to case vault successfully."
            )
        );
        Ok(())
    }
}

/// Disk imajını mount olmadan yapısal olarak analiz eder.
fn image_analyze_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: image-analyze <imaj> [mount_klasoru]",
            "Usage: image-analyze <image> [mount_dir]",
        ));
    }
    let image_path = PathBuf::from(&args[0]);
    let mount_dir = args.get(1).map(PathBuf::from);
    let report = disk_analysis::analyze_disk_image(&image_path, mount_dir.as_deref())
        .map_err(|err| crate_diagnostic(err.to_string()))?;

    if json_output {
        print_json(&report)
    } else {
        let is_en = is_cli_english();
        println!("============================================================");
        println!(
            "       {}",
            if is_en {
                "Forensic Disk Image Analysis Report"
            } else {
                "Adli Disk İmajı Analiz Raporu"
            }
        );
        println!("============================================================");
        println!(
            "  {:<20}: {}",
            if is_en {
                "Image File"
            } else {
                "İmaj Dosyası"
            },
            report.image_path.display()
        );
        println!(
            "  {:<20}: {}",
            if is_en { "File Size" } else { "Dosya Boyutu" },
            format_bytes(report.size)
        );
        println!(
            "  {:<20}: {} {}",
            if is_en {
                "Sector Size"
            } else {
                "Sektör Boyutu"
            },
            report.sector_size,
            if is_en { "bytes" } else { "bayt" }
        );
        println!(
            "  {:<20}: {}",
            if is_en {
                "Partition Scheme"
            } else {
                "Bölüntü Şeması"
            },
            report.partition_scheme
        );
        println!(
            "  {:<20}: {}",
            if is_en {
                "Image Format"
            } else {
                "İmaj Formatı"
            },
            report.image_type
        );

        if !report.partitions.is_empty() {
            println!(
                "\n  --- {} ---",
                if is_en { "Partitions" } else { "Bölüntüler" }
            );
            for p in &report.partitions {
                println!(
                    "    [{}] {} ({}) - {} | LBA: {}",
                    p.index,
                    p.type_name,
                    p.scheme,
                    format_bytes(p.size),
                    p.start_lba
                );
            }
        }

        if !report.filesystems.is_empty() {
            println!(
                "\n  --- {} ---",
                if is_en {
                    "Detected Filesystems"
                } else {
                    "Tespit Edilen Dosya Sistemleri"
                }
            );
            for fs in &report.filesystems {
                println!(
                    "    • {} @ offset {} ({}%)",
                    fs.fs_type, fs.offset, fs.confidence
                );
            }
        }

        if !report.warnings.is_empty() {
            println!(
                "\n  --- {} ---",
                if is_en { "Warnings" } else { "Uyarılar" }
            );
            for w in &report.warnings {
                println!("    [!] {}", w);
            }
        }

        println!("============================================================");
        Ok(())
    }
}

/// RAM imajı için özet analiz üretir.
fn ram_summary_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: ram-summary <ram> [windows|linux] [symbols]",
            "Usage: ram-summary <ram> [windows|linux] [symbols]",
        ));
    }
    let path = PathBuf::from(&args[0]);
    let os_type = args.get(1).map(String::as_str).unwrap_or("linux");
    let symbols = args.get(2).map(PathBuf::from);
    let summary = ram_analysis::analyze_ram_summary_logged_with_symbol_dir(
        &path,
        Some(os_type),
        symbols.as_deref(),
        None,
    )
    .map_err(|err| crate_diagnostic(err.to_string()))?;

    if json_output {
        print_json(&summary)
    } else {
        let is_en = is_cli_english();
        println!("============================================================");
        println!(
            "       {}",
            if is_en {
                "Live RAM Image Analysis Summary"
            } else {
                "Canlı RAM İmajı Analiz Özeti"
            }
        );
        println!("============================================================");
        println!(
            "  {:<22}: {}",
            if is_en { "File Name" } else { "Dosya Adı" },
            summary.file_name
        );
        println!(
            "  {:<22}: {}",
            if is_en { "File Size" } else { "Dosya Boyutu" },
            format_bytes(summary.size)
        );
        println!(
            "  {:<22}: {}",
            if is_en { "Dump Type" } else { "İmaj Türü" },
            summary.dump_type
        );
        println!(
            "  {:<22}: {:.2} / 8.00",
            if is_en {
                "Entropy Sample"
            } else {
                "Entropi Örneği"
            },
            summary.entropy_sample
        );
        println!(
            "  {:<22}: {}",
            if is_en {
                "String Match Count"
            } else {
                "Dizgi Eşleşme Sayısı"
            },
            summary.string_match_count
        );
        println!(
            "  {:<22}: {}",
            if is_en {
                "Processes Detected"
            } else {
                "Tespit Edilen Süreçler"
            },
            summary.process_count
        );

        if !summary.category_counts.is_empty() {
            println!(
                "\n  --- {} ---",
                if is_en {
                    "IOC / String Matches"
                } else {
                    "IOC / Dizgi Kategorileri"
                }
            );
            for c in &summary.category_counts {
                println!("    • {}: {}", c.category, c.count);
            }
        }

        if !summary.warnings.is_empty() {
            println!(
                "\n  --- {} ---",
                if is_en { "Warnings" } else { "Uyarılar" }
            );
            for w in &summary.warnings {
                println!("    [!] {}", w);
            }
        }
        println!("============================================================");
        Ok(())
    }
}

/// RAM imajında IOC/dizgi taraması yapar.
fn ram_strings_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: ram-strings <ram>",
            "Usage: ram-strings <ram>",
        ));
    }
    let matches = ram_analysis::analyze_ram_strings(Path::new(&args[0]))
        .map_err(|err| crate_diagnostic(err.to_string()))?;

    if json_output {
        print_json(&matches)
    } else {
        let is_en = is_cli_english();
        println!("============================================================");
        println!(
            "       {}",
            if is_en {
                "RAM String & IOC Matches"
            } else {
                "RAM Dizgi ve IOC Eşleşmeleri"
            }
        );
        println!("============================================================");
        if matches.is_empty() {
            println!(
                "  {}",
                if is_en {
                    "No matching indicators found."
                } else {
                    "Eşleşen gösterge bulunamadı."
                }
            );
        } else {
            for m in matches.iter().take(50) {
                println!("  [{}] 0x{:08X}: {}", m.category, m.offset, m.value);
            }
            if matches.len() > 50 {
                println!(
                    "  ... ({} {})",
                    matches.len() - 50,
                    if is_en {
                        "more matches truncated"
                    } else {
                        "fazla eşleşme gösterilmedi"
                    }
                );
            }
        }
        println!("============================================================");
        println!(
            "  {} {}",
            if is_en {
                "Total matches:"
            } else {
                "Toplam eşleşme:"
            },
            matches.len()
        );
        println!("============================================================");
        Ok(())
    }
}

/// RAM içinden sınırlı dosya carving yapar.
fn ram_carve_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: ram-carve <ram> <cikti_klasoru>",
            "Usage: ram-carve <ram> <output_dir>",
        ));
    }
    let files = ram_analysis::carve_files(Path::new(&args[0]), Path::new(&args[1]))
        .map_err(|err| crate_diagnostic(err.to_string()))?;

    if json_output {
        print_json(&files)
    } else {
        let is_en = is_cli_english();
        println!("============================================================");
        println!(
            "       {}",
            if is_en {
                "RAM File Carving Results"
            } else {
                "RAM Dosya Çıkarma (Carving) Sonuçları"
            }
        );
        println!("============================================================");
        if files.is_empty() {
            println!(
                "  {}",
                if is_en {
                    "No recoverable files found."
                } else {
                    "Kurtarılabilir dosya bulunamadı."
                }
            );
        } else {
            for f in &files {
                println!(
                    "  • [{}] {} ({})",
                    f.mime_type,
                    f.file_path,
                    format_bytes(f.size)
                );
            }
        }
        println!("============================================================");
        println!(
            "  {} {}",
            if is_en {
                "Total carved files:"
            } else {
                "Toplam çıkarılan dosya:"
            },
            files.len()
        );
        println!("============================================================");
        Ok(())
    }
}

/// Volatility3 ile proses listesini çıkarmaya çalışır.
fn ram_processes_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: ram-processes <ram> [windows|linux] [symbols]",
            "Usage: ram-processes <ram> [windows|linux] [symbols]",
        ));
    }
    let os_type = args.get(1).map(String::as_str).unwrap_or("linux");
    let symbols = args.get(2).map(PathBuf::from);
    let processes = amele::volatility::get_processes_logged_with_symbol_dir(
        Path::new(&args[0]),
        os_type,
        symbols.as_deref(),
        None,
    )
    .map_err(crate_diagnostic)?;

    if json_output {
        print_json(&processes)
    } else {
        let is_en = is_cli_english();
        println!("============================================================");
        println!(
            "       {}",
            if is_en {
                "RAM Extracted Processes"
            } else {
                "RAM Çıkarılan Süreçler"
            }
        );
        println!("============================================================");
        println!(
            "  {:<8} {:<8} {:<28} {:<10}",
            "PID",
            "PPID",
            if is_en { "Image Name" } else { "Süreç Adı" },
            "Offset"
        );
        println!("------------------------------------------------------------");
        for p in &processes {
            println!(
                "  {:<8} {:<8} {:<28} {:<10}",
                p.pid, p.ppid, p.name, p.offset
            );
        }
        println!("============================================================");
        println!(
            "  {} {}",
            if is_en {
                "Total processes:"
            } else {
                "Toplam süreç:"
            },
            processes.len()
        );
        println!("============================================================");
        Ok(())
    }
}

/// ADB kurulum durumunu JSON olarak yazar.
fn android_adb_status_command() -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    print_json(&android::adb_status())
}

/// ADB'yi uygun paket yöneticisiyle kurmayı dener.
fn android_adb_install_command() -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    let result = android::install_adb()
        .map_err(|err| crate_diagnostic(android::explain_android_error(err)))?;
    print_json(&result)
}

/// ADB ile bağlı Android cihazlarını listeler.
fn android_devices_command() -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    let devices = android::list_devices()
        .map_err(|err| crate_diagnostic(android::explain_android_error(err)))?;
    print_json(&devices)
}

fn require_cli_mobile_tools_access() -> Result<(), String> {
    amele::profile::require_mobile_tools_access().map_err(|err| crate_diagnostic(err.to_string()))
}

/// Android cihaz profilini çıkarır.
fn android_profile_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.is_empty() {
        return Err("Kullanim: android-profile <serial>".to_string());
    }
    let profile = android::detect_device_profile(&args[0])
        .map_err(|err| crate_diagnostic(android::explain_android_error(err)))?;
    print_json(&profile)
}

/// Android mantıksal edinimi vaka klasörüne yazar.
fn android_logical_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.len() < 2 {
        return Err("Kullanim: android-logical <serial> <vaka> [quick|full|root]".to_string());
    }
    let profile = args
        .get(2)
        .map(|value| android::AndroidAcquisitionProfile::from_id(value))
        .unwrap_or(android::AndroidAcquisitionProfile::FullLogical);
    let vault = cli_case_vault(&args[1])?;
    let output_dir = vault
        .android_dir
        .join(format!("logical_{}", cli_timestamp()));
    let result = android::orchestrated_acquisition(
        &args[0],
        &output_dir,
        profile,
        |done, total, category| print_step_progress("android-logical", done, total, category),
        || false,
    )
    .map_err(|err| crate_diagnostic(android::explain_android_error(err)))?;
    print_json(&result)
}

/// Android dosya sistemi edinimini vaka klasörüne yazar.
fn android_filesystem_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.len() < 2 {
        return Err("Kullanim: android-filesystem <serial> <vaka> [--root]".to_string());
    }
    let has_root = args.iter().any(|arg| arg == "--root" || arg == "root");
    let vault = cli_case_vault(&args[1])?;
    let output_dir = vault
        .android_dir
        .join(format!("filesystem_{}", cli_timestamp()));
    let result = android::orchestrated_filesystem_acquisition(
        &args[0],
        &output_dir,
        has_root,
        |done, total, category| print_step_progress("android-filesystem", done, total, category),
        || false,
    )
    .map_err(|err| crate_diagnostic(android::explain_android_error(err)))?;
    print_json(&result)
}

/// Android uçucu veri/RAM edinimini vaka klasörüne yazar.
fn android_ram_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.len() < 2 {
        return Err(
            "Kullanim: android-ram <serial> <vaka> [volatile|root|physical] [--root]".to_string(),
        );
    }
    let mode = args
        .get(2)
        .filter(|value| !value.starts_with("--"))
        .map(|value| android::AndroidRamMode::from_id(value))
        .unwrap_or(android::AndroidRamMode::VolatileData);
    let has_root = args.iter().any(|arg| arg == "--root" || arg == "root");
    let vault = cli_case_vault(&args[1])?;
    let output_dir = vault.android_dir.join(format!("ram_{}", cli_timestamp()));
    let result = android::orchestrated_ram_acquisition(
        &args[0],
        &output_dir,
        has_root,
        mode,
        |done, total, category| print_step_progress("android-ram", done, total, category),
        || false,
    )
    .map_err(|err| crate_diagnostic(android::explain_android_error(err)))?;
    print_json(&result)
}

/// Android cihazın hangi edinim modlarına uygun olduğunu raporlar.
fn android_capabilities_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.is_empty() {
        return Err("Kullanim: android-capabilities <serial>".to_string());
    }
    let profile = android::detect_device_profile(&args[0])
        .map_err(|err| crate_diagnostic(android::explain_android_error(err)))?;
    let report = android::build_android_capability_report(&args[0], &profile);
    print_json(&json!({
        "profile": profile,
        "capabilities": report,
    }))
}

/// Lemon fiziksel RAM aracı için cihaz ön kontrolünü çalıştırır.
fn android_lemon_preflight_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.is_empty() {
        return Err("Kullanim: android-lemon-preflight <serial>".to_string());
    }
    let report = android::lemon_preflight(&args[0]);
    print_json(&report)
}

/// TCP/IP ADB veya MESH relay endpoint'ine bağlanır.
fn android_remote_connect_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.is_empty() {
        return Err(
            "Kullanim: android-remote-connect <host> [port] [tcp|mesh] [etiket]".to_string(),
        );
    }
    let host = args[0].clone();
    let port = args
        .get(1)
        .map(|value| parse_port(value))
        .transpose()?
        .unwrap_or(5555);
    let kind = match args.get(2).map(String::as_str) {
        Some("mesh") | Some("mesh_relay") => android::RemoteEndpointKind::MeshRelay,
        _ => android::RemoteEndpointKind::TcpAdb,
    };
    let label = args
        .get(3)
        .cloned()
        .unwrap_or_else(|| format!("{}:{}", host, port));
    let endpoint = android::RemoteAndroidEndpoint {
        label,
        host,
        port,
        kind,
    };
    let result = android::connect_remote_endpoint(&endpoint);
    print_json(&json!({
        "endpoint": endpoint,
        "result": result,
    }))
}

/// Uzak Android ADB bağlantısını keser.
fn android_remote_disconnect_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.is_empty() {
        return Err("Kullanim: android-remote-disconnect <serial>".to_string());
    }
    let result = android::disconnect_remote_endpoint(&args[0]);
    print_json(&result)
}

/// Seçilen vakanın Android çıktılarından analiz özeti üretir.
fn android_case_analysis_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.is_empty() {
        return Err("Kullanim: android-case-analysis <vaka>".to_string());
    }
    let vault = cli_case_vault(&args[0])?;
    let report = android_analysis::analyze_android_case(&vault.case_name, &vault.android_dir);
    print_json(&report)
}

/// iOS backup profil bilgisini JSON olarak yazar.
fn ios_backup_profile_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.is_empty() {
        return Err("Kullanim: ios-backup-profile <backup_klasoru>".to_string());
    }
    let info = ios::inspect_backup(&args[0]).map_err(|err| crate_diagnostic(err.to_string()))?;
    print_json(&info)
}

/// iOS backup klasörünü vaka ios klasörüne Backup2FS düzeninde normalize eder.
fn ios_backup_normalize_command(args: Vec<String>) -> Result<(), String> {
    require_cli_mobile_tools_access()?;
    if args.len() < 2 {
        return Err("Kullanim: ios-backup-normalize <backup_klasoru> <vaka>".to_string());
    }
    let backup_path = PathBuf::from(&args[0]);
    let vault = cli_case_vault(&args[1])?;
    let backup_name = backup_path
        .file_name()
        .and_then(|value| value.to_str())
        .map(cli_safe_stem)
        .unwrap_or_else(|| "ios_backup".to_string());
    let output_dir = vault
        .ios_dir
        .join(format!("{}_{}", backup_name, cli_timestamp()));
    let result = ios::normalize_backup(
        &backup_path,
        &output_dir,
        &[
            HashAlgorithm::Md5,
            HashAlgorithm::Sha1,
            HashAlgorithm::Sha256,
        ],
        |done, total, step| print_step_progress("ios-backup", done as u32, total as u32, step),
        |line| eprintln!("ios-backup: {line}"),
        || false,
        || false,
    )
    .map_err(|err| crate_diagnostic(err.to_string()))?;
    print_json(&result)
}

/// Docker sistem durumunu veya bağlanmış dizini denetler.
fn docker_status_command(args: Vec<String>) -> Result<(), String> {
    let custom_root = args.first().map(Path::new);
    let status = docker::check_docker_status(custom_root);
    print_json(&status)
}

/// Docker konteynerlerini listeler ve güvenlik/kaçış risk analizini gösterir.
fn docker_list_command(args: Vec<String>) -> Result<(), String> {
    let custom_root = args.first().map(Path::new);
    let containers = docker::list_containers(custom_root).map_err(|err| err.to_string())?;
    print_json(&containers)
}

/// Belirtilen konteynerin log kayıtlarını ekrana basar.
fn docker_logs_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: docker-logs <konteyner_id> [tail] [kok_dizin]",
            "Usage: docker-logs <container_id> [tail] [custom_root]",
        ));
    }
    let cid = &args[0];
    let tail = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(200);
    let custom_root = args.get(2).map(Path::new);
    let logs = docker::get_container_logs(cid, tail, custom_root).map_err(|err| err.to_string())?;
    print_json(&logs)
}

/// Yerel konteyner delillerini (Overlay2 diff, konfigürasyon, loglar) vakaya toplar.
fn docker_acquire_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: docker-acquire <konteyner_id> [vaka_adi] [kok_dizin]",
            "Usage: docker-acquire <container_id> [case_name] [custom_root]",
        ));
    }
    let cid = args[0].clone();
    let case_name = args.get(1).cloned();
    let custom_root = args.get(2).cloned();

    let req = DockerAcquisitionRequest {
        container_id: cid,
        acquire_diff: true,
        acquire_logs: true,
        acquire_config: true,
        case_name,
        custom_docker_root: custom_root,
    };

    let base_dir = amele::api::default_case_base_dir();
    let result = docker::acquire_container_evidence(&req, base_dir, |msg, done, total| {
        print_progress("docker", done, total);
        eprintln!("{msg}");
    })
    .map_err(|err| err.to_string())?;

    print_json(&result)
}

/// Uzak agent'tan Docker daemon durumunu çeker.
fn docker_remote_status_command(args: Vec<String>) -> Result<(), String> {
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: docker-remote-status <ip> <port> [token]",
            "Usage: docker-remote-status <ip> <port> [token]",
        ));
    }
    let ip = &args[0];
    let port = parse_port(&args[1])?;
    let token = args.get(2).cloned();
    let mut conn = RemoteConnection::connect(ip, port, token).map_err(|err| err.to_string())?;
    let status = conn.docker_status().map_err(|err| err.to_string())?;
    print_json(&status)
}

/// Uzak agent üzerindeki Docker konteynerlerini listeler.
fn docker_remote_list_command(args: Vec<String>) -> Result<(), String> {
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: docker-remote-list <ip> <port> [token]",
            "Usage: docker-remote-list <ip> <port> [token]",
        ));
    }
    let ip = &args[0];
    let port = parse_port(&args[1])?;
    let token = args.get(2).cloned();
    let mut conn = RemoteConnection::connect(ip, port, token).map_err(|err| err.to_string())?;
    let containers = conn
        .list_docker_containers()
        .map_err(|err| err.to_string())?;
    print_json(&containers)
}

/// Uzak agent üzerindeki konteynerin loglarını çeker.
fn docker_remote_logs_command(args: Vec<String>) -> Result<(), String> {
    if args.len() < 3 {
        return Err(t_cli(
            "Kullanim: docker-remote-logs <ip> <port> <konteyner_id> [tail] [token]",
            "Usage: docker-remote-logs <ip> <port> <container_id> [tail] [token]",
        ));
    }
    let ip = &args[0];
    let port = parse_port(&args[1])?;
    let cid = &args[2];
    let tail = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(200);
    let token = args.get(4).cloned();
    let mut conn = RemoteConnection::connect(ip, port, token).map_err(|err| err.to_string())?;
    let logs = conn
        .get_docker_container_logs(cid, tail)
        .map_err(|err| err.to_string())?;
    print_json(&logs)
}

/// Uzak agent üzerindeki konteyner delillerini vaka klasörüne aktarır.
fn docker_remote_acquire_command(args: Vec<String>) -> Result<(), String> {
    if args.len() < 3 {
        return Err(t_cli(
            "Kullanim: docker-remote-acquire <ip> <port> <konteyner_id> [vaka_adi] [token]",
            "Usage: docker-remote-acquire <ip> <port> <container_id> [case_name] [token]",
        ));
    }
    let ip = &args[0];
    let port = parse_port(&args[1])?;
    let cid = &args[2];
    let case_name = args.get(3).cloned().unwrap_or_else(|| {
        format!(
            "VAKA_DOCKER_REMOTE_{}",
            Local::now().format("%Y%m%d_%H%M%S")
        )
    });
    let token = args.get(4).cloned();

    let vault = cli_case_vault(&case_name)?;
    let short_id = if cid.len() >= 12 { &cid[..12] } else { cid };
    let target_dir = vault.docker_dir.join(format!("remote_{}", short_id));
    fs::create_dir_all(&target_dir).map_err(|err| err.to_string())?;
    let target_tar = target_dir.join("docker_evidence.tar.gz");

    let mut conn = RemoteConnection::connect(ip, port, token).map_err(|err| err.to_string())?;
    let result = conn
        .acquire_remote_docker(cid, true, true, true, &target_tar, None, |done, total| {
            print_progress("docker-remote", done, total);
        })
        .map_err(|err| err.to_string())?;

    let meta_path = target_dir.join("docker_metadata.json");
    let _ = fs::write(
        &meta_path,
        json!({
            "edinim_zamani": Local::now().to_rfc3339(),
            "konteyner_id": cid,
            "isim": format!("remote_{}", short_id),
            "sha256": result.sha256,
            "boyut": result.bytes_transferred,
        })
        .to_string(),
    );

    let manifest_path = target_dir.join("manifest.csv");
    let manifest_content = format!(
        "Dosya_Adi,Boyut_Byte,SHA256\ndocker_evidence.tar.gz,{},{}\n",
        result.bytes_transferred,
        result.sha256.as_deref().unwrap_or("HATA")
    );
    let _ = fs::write(&manifest_path, manifest_content);

    print_json(&result)
}

/// Yetkili helper sürecinde diskleri listeleyip sonucu dosyaya yazar.
fn disk_list_helper_command(args: Vec<String>) -> Result<(), String> {
    let Some(output) = args.first() else {
        return Err("Kullanim: disk-list-helper <json-cikti>".to_string());
    };
    let payload = match disk::list_disks() {
        Ok(disks) => json!({ "ok": true, "disks": disks }),
        Err(err) => json!({ "ok": false, "error": err.to_string() }),
    };
    fs::write(
        output,
        serde_json::to_vec_pretty(&payload).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

/// Yetkili imaj alma helper'ının JSON istek alanlarını taşır.
#[derive(Deserialize)]
struct ImageHelperRequest {
    source: PathBuf,
    target: PathBuf,
    owner_uid: Option<u32>,
    owner_gid: Option<u32>,
}

/// Root/admin yetkisiyle disk imajı alır ve ilerlemeyi/result dosyalarını günceller.
fn image_helper_command(args: Vec<String>) -> Result<(), String> {
    if !(3..=4).contains(&args.len()) {
        return Err(
            "Kullanim: image-helper <request-json> <result-json> <progress-json> [control-json]"
                .to_string(),
        );
    }
    let request_path = PathBuf::from(&args[0]);
    let result_path = PathBuf::from(&args[1]);
    let progress_path = PathBuf::from(&args[2]);
    let control_path = args.get(3).map(PathBuf::from);
    let request: ImageHelperRequest =
        serde_json::from_slice(&fs::read(&request_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let task = disk::DiskAcquisitionTask::new(&request.source, &request.target);
    let result = disk::run_disk_acquisition_with_control(
        &task,
        |done, total| {
            let _ = write_json_file(
                &progress_path,
                &json!({
                    "done": done,
                    "total": total,
                    "message": "Imaj alma suruyor",
                }),
            );
        },
        || image_helper_control(control_path.as_deref()),
    );

    let payload = match result {
        Ok(result) => {
            restore_helper_output_owner(&result.target, request.owner_uid, request.owner_gid);
            json!({
                "ok": true,
                "target_path": result.target,
                "bytes_copied": result.bytes_copied,
                "total_bytes": result.total_bytes,
                "sha256": result.sha256,
            })
        }
        Err(err) => json!({
            "ok": false,
            "error": err.to_string(),
        }),
    };
    write_json_file(&result_path, &payload)
}

/// Helper root olarak çalıştıysa çıkan dosyaları asıl kullanıcıya geri verir.
fn restore_helper_output_owner(target: &Path, owner_uid: Option<u32>, owner_gid: Option<u32>) {
    let (Some(owner_uid), Some(owner_gid)) = (owner_uid, owner_gid) else {
        return;
    };
    for path in [target.to_path_buf(), sha256_sidecar_path(target)] {
        if path.exists() {
            let _ = Command::new("chown")
                .arg(format!("{owner_uid}:{owner_gid}"))
                .arg(path)
                .output();
        }
    }
}

/// İmaj dosyasının yanında oluşturulan SHA256 sidecar yolunu hesaplar.
fn sha256_sidecar_path(target: &Path) -> PathBuf {
    target.with_extension(format!(
        "{}sha256",
        target
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| format!("{extension}."))
            .unwrap_or_default()
    ))
}

/// UI'dan gelen pause/resume/stop kontrol dosyasını disk edinim durumuna çevirir.
fn image_helper_control(control_path: Option<&Path>) -> disk::DiskAcquisitionControl {
    let Some(control_path) = control_path else {
        return disk::DiskAcquisitionControl::Continue;
    };
    let Some(value) = fs::read(control_path)
        .ok()
        .and_then(|payload| serde_json::from_slice::<Value>(&payload).ok())
    else {
        return disk::DiskAcquisitionControl::Continue;
    };
    match value
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "cancelled" | "cancel" | "stop" => disk::DiskAcquisitionControl::Cancel,
        "paused" | "pause" => disk::DiskAcquisitionControl::Pause,
        _ => disk::DiskAcquisitionControl::Continue,
    }
}

/// Yetkili RAM helper'ının çalıştıracağı araç ve çıktı bilgilerini taşır.
#[derive(Deserialize)]
struct RamHelperRequest {
    output_file: PathBuf,
    tool: String,
    tool_path: Option<PathBuf>,
    owner_uid: Option<u32>,
    owner_gid: Option<u32>,
}

/// Root/admin yetkisiyle AVML veya WinPMEM çalıştırıp RAM çıktısını üretir.
fn ram_helper_command(args: Vec<String>) -> Result<(), String> {
    if args.len() != 4 {
        return Err(
            "Kullanim: ram-helper <request-json> <result-json> <progress-json> <control-json>"
                .to_string(),
        );
    }
    let request_path = PathBuf::from(&args[0]);
    let result_path = PathBuf::from(&args[1]);
    let progress_path = PathBuf::from(&args[2]);
    let control_path = PathBuf::from(&args[3]);
    let request: RamHelperRequest =
        serde_json::from_slice(&fs::read(&request_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let token = ram::CancellationToken::default();
    let watcher_stop = Arc::new(AtomicBool::new(false));
    let watcher = {
        let token = token.clone();
        let watcher_stop = watcher_stop.clone();
        thread::spawn(move || {
            while !watcher_stop.load(Ordering::SeqCst) {
                apply_ram_helper_control(&token, &control_path);
                thread::sleep(Duration::from_millis(200));
            }
        })
    };

    let candidate = request.tool_path.as_deref();
    let result = match request.tool.as_str() {
        "avml" => ram::acquire_with_avml(&request.output_file, candidate, &token, |done, total| {
            let _ = write_json_file(
                &progress_path,
                &json!({
                    "done": done,
                    "total": total,
                    "message": "RAM edinimi suruyor",
                }),
            );
        }),
        "winpmem" => {
            ram::acquire_with_winpmem(&request.output_file, candidate, &token, |done, total| {
                let _ = write_json_file(
                    &progress_path,
                    &json!({
                        "done": done,
                        "total": total,
                        "message": "RAM edinimi suruyor",
                    }),
                );
            })
        }
        _ => Err(amele::error::AmeleError::new(
            amele::error::HataKodu::Genel,
            "Desteklenmeyen RAM araci",
        )),
    };

    watcher_stop.store(true, Ordering::SeqCst);
    let _ = watcher.join();

    let payload = match result {
        Ok(result) => {
            restore_helper_output_owner(&result.output_file, request.owner_uid, request.owner_gid);
            json!({
                "ok": true,
                "target_path": result.output_file,
                "bytes_written": result.bytes_written,
            })
        }
        Err(err) => json!({
            "ok": false,
            "error": err.to_string(),
        }),
    };
    write_json_file(&result_path, &payload)
}

/// RAM helper kontrol dosyasındaki pause/resume/stop durumunu token'a uygular.
fn apply_ram_helper_control(token: &ram::CancellationToken, control_path: &Path) {
    let Some(value) = fs::read(control_path)
        .ok()
        .and_then(|payload| serde_json::from_slice::<Value>(&payload).ok())
    else {
        return;
    };
    match value
        .get("state")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "cancelled" | "cancel" | "stop" => token.cancel(),
        "paused" | "pause" => token.pause(),
        "running" | "resume" => token.resume(),
        _ => {}
    }
}

fn avml_install_helper_command(args: Vec<String>) -> Result<(), String> {
    if args.len() != 2 {
        return Err("Kullanim: avml-install-helper <kaynak> <result-json>".to_string());
    }
    let source = PathBuf::from(&args[0]);
    let result_path = PathBuf::from(&args[1]);
    let payload = match install_avml_binary(&source) {
        Ok(value) => value,
        Err(err) => json!({
            "ok": false,
            "error": err,
        }),
    };
    write_json_file(&result_path, &payload)
}

#[cfg(target_os = "linux")]
fn install_avml_binary(source: &Path) -> Result<Value, String> {
    use std::os::unix::fs::PermissionsExt;

    if !source.is_file() {
        return Err("Downloaded AVML binary not found".to_string());
    }

    let target = Path::new("/usr/bin/avml");
    let temp = Path::new("/usr/bin/.amele-avml.tmp");
    fs::copy(source, temp).map_err(|err| format!("AVML /usr/bin altina kopyalanamadi: {err}"))?;
    let mut permissions = fs::metadata(temp)
        .map_err(|err| err.to_string())?
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(temp, permissions).map_err(|err| err.to_string())?;
    let _ = Command::new("chown").arg("root:root").arg(temp).status();
    fs::rename(temp, target)
        .map_err(|err| format!("AVML /usr/bin/avml olarak kurulamadi: {err}"))?;

    let version = Command::new(target)
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    Ok(json!({
        "ok": true,
        "path": target,
        "version": version,
        "message": "AVML /usr/bin/avml olarak kuruldu",
    }))
}

#[cfg(not(target_os = "linux"))]
fn install_avml_binary(_source: &Path) -> Result<Value, String> {
    Err("AVML installation is only supported on Linux".to_string())
}

fn winpmem_install_helper_command(args: Vec<String>) -> Result<(), String> {
    if args.len() != 2 {
        return Err("Kullanim: winpmem-install-helper <kaynak> <result-json>".to_string());
    }
    let source = PathBuf::from(&args[0]);
    let result_path = PathBuf::from(&args[1]);
    let payload = match install_winpmem_binary(&source) {
        Ok(value) => value,
        Err(err) => json!({
            "ok": false,
            "error": err,
        }),
    };
    write_json_file(&result_path, &payload)
}

#[cfg(windows)]
fn install_winpmem_binary(source: &Path) -> Result<Value, String> {
    if !source.is_file() {
        return Err("Downloaded WinPMEM binary not found".to_string());
    }

    let target_dir = Path::new(r"C:\Tools");
    fs::create_dir_all(target_dir).map_err(|err| format!("C:\\Tools olusturulamadi: {err}"))?;
    let target = target_dir.join(ram::WINPMEM_NAME);
    let temp = target_dir.join(".amele-winpmem.tmp");
    fs::copy(source, &temp)
        .map_err(|err| format!("WinPMEM C:\\Tools altina kopyalanamadi: {err}"))?;
    if target.exists() {
        fs::remove_file(&target)
            .map_err(|err| format!("Eski WinPMEM dosyasi kaldirilamadi: {err}"))?;
    }
    fs::rename(&temp, &target)
        .map_err(|err| format!("WinPMEM C:\\Tools altina kurulamadi: {err}"))?;

    Ok(json!({
        "ok": true,
        "path": target,
        "message": "WinPMEM C:\\Tools altina kuruldu",
    }))
}

#[cfg(not(windows))]
fn install_winpmem_binary(_source: &Path) -> Result<Value, String> {
    Err("WinPMEM installation is only supported on Windows".to_string())
}

#[derive(Deserialize)]
struct MountHelperRequest {
    action: String,
    image_path: Option<PathBuf>,
    mount_dir: PathBuf,
    loop_device: Option<PathBuf>,
}

fn mount_helper_command(args: Vec<String>) -> Result<(), String> {
    if args.len() != 2 {
        return Err("Kullanim: mount-helper <request-json> <result-json>".to_string());
    }
    let request_path = PathBuf::from(&args[0]);
    let result_path = PathBuf::from(&args[1]);
    let request: MountHelperRequest =
        serde_json::from_slice(&fs::read(&request_path).map_err(|err| err.to_string())?)
            .map_err(|err| err.to_string())?;

    let result = match request.action.as_str() {
        "mount" => {
            let image_path = request
                .image_path
                .as_deref()
                .ok_or_else(|| "image_path is required".to_string());
            image_path.and_then(|image_path| mount_image_readonly(image_path, &request.mount_dir))
        }
        "unmount" => unmount_image(&request.mount_dir, request.loop_device.as_deref())
            .map(|_| json!({ "ok": true, "mount_dir": request.mount_dir })),
        _ => Err("action must be mount or unmount".to_string()),
    };

    let payload = match result {
        Ok(value) => value,
        Err(err) => json!({ "ok": false, "error": err }),
    };
    write_json_file(&result_path, &payload)
}

#[cfg(target_os = "linux")]
fn mount_image_readonly(image_path: &Path, mount_dir: &Path) -> Result<Value, String> {
    let direct = Command::new("mount")
        .arg("-o")
        .arg("ro,loop")
        .arg(image_path)
        .arg(mount_dir)
        .output()
        .map_err(|err| err.to_string())?;
    if direct.status.success() {
        return Ok(json!({
            "ok": true,
            "mount_dir": mount_dir,
            "loop_device": Value::Null,
        }));
    }

    let direct_error = command_error_message(
        &direct,
        "mount failed; image may contain a partition table or root privileges may be required",
    );
    mount_partitioned_image(image_path, mount_dir)
        .map_err(|err| format!("{direct_error}\npartition scan failed: {err}"))
}

#[cfg(windows)]
fn mount_image_readonly(image_path: &Path, _mount_dir: &Path) -> Result<Value, String> {
    let mount_dir = windows_mount_image_readonly(image_path)?;
    Ok(json!({
        "ok": true,
        "mount_dir": mount_dir,
        "loop_device": Value::Null,
    }))
}

#[cfg(not(any(target_os = "linux", windows)))]
fn mount_image_readonly(_image_path: &Path, _mount_dir: &Path) -> Result<Value, String> {
    Err("image mount helper is not supported on this platform".to_string())
}

#[cfg(windows)]
fn windows_mount_image_readonly(image_path: &Path) -> Result<PathBuf, String> {
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(
            "$ErrorActionPreference='Stop'; \
             $image = $args[0]; \
             Mount-DiskImage -ImagePath $image -Access ReadOnly | Out-Null; \
             Start-Sleep -Milliseconds 500; \
             $diskImage = Get-DiskImage -ImagePath $image; \
             $disk = $diskImage | Get-Disk -ErrorAction Stop; \
             $partition = $disk | Get-Partition | Where-Object { $_.Type -ne 'Reserved' } | Select-Object -First 1; \
             $volume = $partition | Get-Volume -ErrorAction SilentlyContinue; \
             if ($volume -and $volume.DriveLetter) { \
               Write-Output ($volume.DriveLetter + ':\\'); \
               exit 0; \
             }; \
             $accessPath = $partition.AccessPaths | Where-Object { $_ -like '*:\\*' -or $_ -like '\\\\?\\Volume*' } | Select-Object -First 1; \
             if ($accessPath) { \
               Write-Output $accessPath; \
               exit 0; \
             }; \
             Dismount-DiskImage -ImagePath $image -ErrorAction SilentlyContinue; \
             throw 'Mounted image has no drive letter. Windows supports ISO/VHD/VHDX here; raw DD/IMG needs a forensic image driver.'",
        )
        .arg(image_path)
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return Err(if stderr.is_empty() {
            if stdout.is_empty() {
                "Windows image mount failed".to_string()
            } else {
                stdout
            }
        } else {
            stderr
        });
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .last()
        .map(PathBuf::from)
        .ok_or_else(|| {
            "Windows mount succeeded but did not return a readable mount path.".to_string()
        })
}

#[cfg(target_os = "linux")]
fn mount_partitioned_image(image_path: &Path, mount_dir: &Path) -> Result<Value, String> {
    let setup = Command::new("losetup")
        .arg("--find")
        .arg("--partscan")
        .arg("--read-only")
        .arg("--show")
        .arg(image_path)
        .output()
        .map_err(|err| err.to_string())?;
    if !setup.status.success() {
        return Err(command_error_message(
            &setup,
            "losetup failed; root privileges may be required",
        ));
    }

    let loop_device = PathBuf::from(String::from_utf8_lossy(&setup.stdout).trim());
    if loop_device.as_os_str().is_empty() {
        return Err("losetup did not return a loop device".to_string());
    }
    thread::sleep(Duration::from_millis(250));

    let mut last_error = String::new();
    for candidate in loop_mount_candidates(&loop_device) {
        let output = Command::new("mount")
            .arg("-o")
            .arg("ro")
            .arg(&candidate)
            .arg(mount_dir)
            .output()
            .map_err(|err| err.to_string())?;
        if output.status.success() {
            return Ok(json!({
                "ok": true,
                "mount_dir": mount_dir,
                "loop_device": loop_device,
            }));
        }
        last_error = format!(
            "{}: {}",
            candidate.display(),
            command_error_message(&output, "mount failed")
        );
    }

    let _ = Command::new("losetup").arg("-d").arg(&loop_device).output();
    Err(if last_error.is_empty() {
        "no mountable filesystem partition was found in the image".to_string()
    } else {
        last_error
    })
}

#[cfg(target_os = "linux")]
fn loop_mount_candidates(loop_device: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(output) = Command::new("lsblk")
        .arg("-rnpo")
        .arg("PATH,TYPE")
        .arg(loop_device)
        .output()
        && output.status.success()
    {
        for line in String::from_utf8_lossy(&output.stdout).lines() {
            let mut parts = line.split_whitespace();
            let Some(path) = parts.next() else {
                continue;
            };
            let Some(kind) = parts.next() else {
                continue;
            };
            if kind == "part" {
                candidates.push(PathBuf::from(path));
            }
        }
    }

    if candidates.is_empty()
        && let Some(name) = loop_device.file_name().and_then(|value| value.to_str())
    {
        let sys_block = Path::new("/sys/block").join(name);
        if let Ok(entries) = fs::read_dir(sys_block) {
            for entry in entries.flatten() {
                let partition_name = entry.file_name();
                let partition_name = partition_name.to_string_lossy();
                if partition_name.starts_with(name) && partition_name != name {
                    candidates.push(Path::new("/dev").join(partition_name.as_ref()));
                }
            }
        }
    }

    candidates.push(loop_device.to_path_buf());
    candidates
}

#[cfg(target_os = "linux")]
fn unmount_image(mount_dir: &Path, loop_device: Option<&Path>) -> Result<(), String> {
    let output = Command::new("umount")
        .arg(mount_dir)
        .output()
        .map_err(|err| err.to_string())?;
    if !output.status.success() {
        return Err(command_error_message(&output, "unmount failed"));
    }
    if let Some(loop_device) = loop_device {
        let output = Command::new("losetup")
            .arg("-d")
            .arg(loop_device)
            .output()
            .map_err(|err| err.to_string())?;
        if !output.status.success() {
            return Err(command_error_message(&output, "loop device detach failed"));
        }
    }
    Ok(())
}

#[cfg(windows)]
fn unmount_image(_mount_dir: &Path, _loop_device: Option<&Path>) -> Result<(), String> {
    Err(
        "Windows mount helper unmount requires the image path and is handled by the UI process"
            .to_string(),
    )
}

#[cfg(not(any(target_os = "linux", windows)))]
fn unmount_image(_mount_dir: &Path, _loop_device: Option<&Path>) -> Result<(), String> {
    Err("image unmount helper is not supported on this platform".to_string())
}

fn command_error_message(output: &std::process::Output, fallback: &str) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        fallback.to_string()
    } else {
        stderr
    }
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

/// CLI komutları için varsayılan vaka klasörünü oluşturur.
fn cli_case_vault(case_name: &str) -> Result<EvidenceVault, String> {
    let clean = amele::api::sanitize_case_name(case_name);
    EvidenceVault::create(amele::api::default_case_base_dir(), clean)
        .map_err(|err| crate_diagnostic(err.to_string()))
}

/// Dosya adlarında güvenli kısa parça üretir.
fn cli_safe_stem(value: &str) -> String {
    let clean = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string();
    if clean.is_empty() {
        "output".to_string()
    } else {
        clean
    }
}

/// CLI dosya adları için ortak zaman damgası üretir.
fn cli_timestamp() -> String {
    Local::now().format("%Y%m%d_%H%M%S").to_string()
}

/// CLI argümanlarından raw/aff4 format seçimini ayıklar.
fn extract_output_format(args: &mut Vec<String>) -> Result<AcquisitionOutputFormat, String> {
    let mut selected = None;
    args.retain(|arg| {
        let parsed = if let Some(format) = arg.strip_prefix("--format=") {
            Some(format)
        } else if arg == "--aff4" {
            Some("aff4")
        } else if arg == "--raw" {
            Some("raw")
        } else if matches!(arg.as_str(), "raw" | "dd" | "img" | "aff4") {
            Some(arg.as_str())
        } else {
            None
        };

        if let Some(fmt) = parsed {
            selected = Some(fmt.to_string());
            false
        } else {
            true
        }
    });
    AcquisitionOutputFormat::parse(selected.as_deref())
}

/// Global bayrakları (örn: --quiet, -q, --verbose, -v) komut listesinden ayıklar.
fn extract_flag(args: &mut Vec<String>, flags: &[&str]) -> bool {
    if let Some(pos) = args.iter().position(|a| flags.iter().any(|f| a == f)) {
        args.remove(pos);
        true
    } else {
        false
    }
}

/// Global --profile argümanını komut listesinden ayırır.
fn extract_global_profile(args: &mut Vec<String>) -> Option<String> {
    if let Some(pos) = args
        .iter()
        .position(|a| a == "--profile" || a.starts_with("--profile="))
    {
        let arg = args.remove(pos);
        if let Some(val) = arg.strip_prefix("--profile=") {
            Some(val.to_string())
        } else if pos < args.len() {
            Some(args.remove(pos))
        } else {
            None
        }
    } else {
        None
    }
}

/// Global --lang / --en / --tr bayraklarını ayıklar.
fn extract_global_lang_flag(args: &mut Vec<String>) -> Option<String> {
    if let Some(pos) = args.iter().position(|a| a.starts_with("--lang=")) {
        let val = args.remove(pos);
        let lang = val.trim_start_matches("--lang=").to_lowercase();
        return Some(lang);
    }
    if let Some(pos) = args.iter().position(|a| a == "--lang" || a == "-l") {
        if pos + 1 < args.len() {
            args.remove(pos);
            let lang = args.remove(pos);
            return Some(lang.to_lowercase());
        }
    }
    if let Some(pos) = args.iter().position(|a| a == "--en") {
        args.remove(pos);
        return Some("en".to_string());
    }
    if let Some(pos) = args.iter().position(|a| a == "--tr") {
        args.remove(pos);
        return Some("tr".to_string());
    }
    None
}

/// CLI ve profil dilini kalıcı olarak değiştirir.
fn lang_set_command(target: String) -> Result<(), String> {
    let normalized = match target.to_lowercase().as_str() {
        "en" | "english" => "en",
        "tr" | "turkish" | "turkce" | "türkçe" => "tr",
        other => {
            return Err(format!(
                "{}: '{}' ({})",
                t_cli("Geçersiz dil seçimi", "Invalid language choice"),
                other,
                t_cli("Geçerli: 'tr', 'en'", "Valid: 'tr', 'en'")
            ));
        }
    };

    set_cli_english(normalized == "en");

    // Aktif profil varsa dil tercihini güncelle
    if let Some(prof) = amele::profile::active_profile() {
        let _ = amele::profile::update_active_preferences(normalized, &prof.theme);
    }

    // Uygulama ayarlarını güncelle
    let mut settings = amele::settings::AppSettings::load(amele::settings::default_settings_path())
        .unwrap_or_default();
    settings.dil = normalized.to_string();
    let _ = settings.save(amele::settings::default_settings_path());

    println!(
        "{}",
        t_cli(
            "CLI dili Türkçe (tr) olarak ayarlandı.",
            "CLI language set to English (en)."
        )
    );
    Ok(())
}

/// Profil komutlarında otomatik açılış bayrağını ayıklar.
fn remove_direct_flag(args: &mut Vec<String>) -> bool {
    let original_len = args.len();
    args.retain(|arg| arg != "--direct" && arg != "--open-directly");
    args.len() != original_len
}

/// JSON çıktıyı stdout'a pretty formatta yazar.
fn print_json<T: Serialize>(value: &T) -> Result<(), String> {
    println!(
        "{}",
        serde_json::to_string_pretty(value).map_err(|err| err.to_string())?
    );
    Ok(())
}

/// Byte bazlı ilerlemeyi stderr'e yüzde olarak yazar.
fn print_progress(label: &str, done: u64, total: u64) {
    if total == 0 {
        eprintln!("{label}: {done} byte");
    } else {
        let percent = done.saturating_mul(100).checked_div(total).unwrap_or(0);
        eprintln!("{label}: {percent}% [{done}/{total}]");
    }
}

/// Adım bazlı ilerlemeyi stderr'e yazar.
fn print_step_progress(label: &str, done: u32, total: u32, step: &str) {
    if total == 0 {
        eprintln!("{label}: {step}");
    } else {
        let shown = if done == 0 { 1 } else { done.min(total) };
        eprintln!("{label}: {}/{} {}", shown, total, step);
    }
}

/// CLI hatalarını uygulamanın zengin hata açıklamasıyla döndürür.
fn crate_diagnostic(message: String) -> String {
    amele::diagnostics::error_with_advice(&message)
}

fn disk_size_command(args: Vec<String>) -> Result<(), String> {
    let Some(path) = args.first() else {
        return Err("Kullanim: disk-size <cihaz|dosya>".to_string());
    };
    let size = disk::disk_size(path).map_err(|err| err.to_string())?;
    println!("{size}");
    Ok(())
}

fn verify_command(args: Vec<String>) -> Result<(), String> {
    if args.len() != 2 {
        return Err(t_cli(
            "Kullanim: verify <imaj> <sha256>",
            "Usage: verify <image> <sha256>",
        ));
    }
    let ok = disk::verify_image(&args[0], &args[1]).map_err(|err| err.to_string())?;
    println!("{}", if ok { "OK" } else { "FAIL" });
    Ok(())
}

fn remote_disks_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: remote-disks <ip> <port> [token]",
            "Usage: remote-disks <ip> <port> [token]",
        ));
    }
    let port = parse_port(&args[1])?;
    let token = args.get(2).cloned();
    let mut connection =
        RemoteConnection::connect(&args[0], port, token).map_err(|err| err.to_string())?;
    let disks = connection.list_disks().map_err(|err| err.to_string())?;

    if json_output {
        return print_json(&disks);
    }

    let is_en = is_cli_english();
    println!(
        "=========================================================================================="
    );
    println!(
        "       {} ({}:{})",
        if is_en {
            "Remote Agent Disks"
        } else {
            "Uzak Agent Diskleri"
        },
        &args[0],
        port
    );
    println!(
        "=========================================================================================="
    );
    println!(
        "  {:<16} {:<32} {:<20}",
        if is_en { "Disk ID" } else { "Disk Kimliği" },
        if is_en { "Name / Path" } else { "Adı / Yolu" },
        if is_en { "Size" } else { "Boyut" }
    );
    println!(
        "------------------------------------------------------------------------------------------"
    );
    for d in &disks {
        println!("  {:<16} {:<32} {:<20}", d.id, d.ad, format_bytes(d.boyut));
    }
    println!(
        "=========================================================================================="
    );
    println!(
        "  {} {}",
        if is_en {
            "Total disks:"
        } else {
            "Toplam disk:"
        },
        disks.len()
    );
    println!(
        "=========================================================================================="
    );
    Ok(())
}

fn remote_image_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    let selected_format = extract_output_format(&mut args)?;
    if args.len() < 4 {
        return Err(t_cli(
            "Kullanim: remote-image <ip> <port> <disk_id> <cikti_klasoru> [token] [raw|aff4]",
            "Usage: remote-image <ip> <port> <disk_id> <out_dir> [token] [raw|aff4]",
        ));
    }
    let port = parse_port(&args[1])?;
    let token = args.get(4).cloned();
    let mut connection =
        RemoteConnection::connect(&args[0], port, token).map_err(|err| err.to_string())?;
    let result = connection
        .acquire_image(
            &args[2],
            None,
            &args[3],
            None,
            selected_format,
            |done: u64, total: u64| {
                if let Some(percent) = done.saturating_mul(100).checked_div(total) {
                    eprintln!("{}%", percent);
                }
            },
        )
        .map_err(|err| err.to_string())?;
    let plan = output_format::OutputPlan {
        format: selected_format,
        working_path: result.target_path.clone(),
        final_path: result.target_path.with_extension("aff4"),
    };
    let finalized =
        output_format::finalize_output(&plan, "disk", &args[2], "", result.sha256.clone())?;

    if json_output {
        print_json(&json!({
            "remote_job_id": result.job_id,
            "target_path": finalized.target_path,
            "bytes_transferred": result.bytes_transferred,
            "sha256": finalized.sha256,
            "raw_sha256": finalized.raw_sha256,
            "output_format": finalized.format.as_str(),
            "md5": result.md5,
            "message": result.message,
        }))
    } else {
        println!("============================================================");
        println!(
            "       {}",
            t_cli(
                "Uzak Disk İmajı Edinimi Tamamlandı",
                "Remote Disk Image Acquisition Completed"
            )
        );
        println!("============================================================");
        println!(
            "  {:<20}: {}:{}",
            t_cli("Uzak Agent", "Remote Agent"),
            &args[0],
            port
        );
        println!("  {:<20}: {}", t_cli("Disk Kimliği", "Disk ID"), &args[2]);
        println!(
            "  {:<20}: {}",
            t_cli("Hedef Dosya", "Target File"),
            finalized.target_path.display()
        );
        println!(
            "  {:<20}: {}",
            t_cli("Aktarılan Boyut", "Transferred Size"),
            format_bytes(result.bytes_transferred)
        );
        println!(
            "  {:<20}: {}",
            t_cli("İmaj Formatı", "Image Format"),
            finalized.format.as_str().to_uppercase()
        );
        println!("  {:<20}: {}", "SHA-256", finalized.sha256);
        println!("============================================================");
        println!(
            "[✓] {}",
            t_cli(
                "Uzak disk imajı başarıyla indirildi.",
                "Remote disk image downloaded successfully."
            )
        );
        Ok(())
    }
}

fn remote_tool_check_command(args: Vec<String>) -> Result<(), String> {
    if args.len() < 3 {
        return Err("Kullanim: remote-tool-check <ip> <port> <winpmem|avml> [token]".to_string());
    }
    let port = parse_port(&args[1])?;
    let token = args.get(3).cloned();
    let mut connection =
        RemoteConnection::connect(&args[0], port, token).map_err(|err| err.to_string())?;
    let status = match args[2].as_str() {
        "winpmem" => connection.check_winpmem(),
        "avml" => connection.check_avml(),
        other => return Err(format!("Bilinmeyen arac: {other}")),
    }
    .map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&status).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn ssh_disks_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: ssh-disks <ip> <user> [port] [password] [key_path]",
            "Usage: ssh-disks <ip> <user> [port] [password] [key_path]",
        ));
    }
    let ip = args[0].clone();
    let user = args[1].clone();
    let port = args
        .get(2)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(22);
    let password = args.get(3).cloned();
    let key_path = args.get(4).cloned();

    let params = SshConnectionParams {
        ip: ip.clone(),
        port,
        user: user.clone(),
        password,
        key_path,
    };
    let mut conn = SshConnection::connect(&params).map_err(|err| err.to_string())?;
    let disks = conn.list_disks().map_err(|err| err.to_string())?;

    if json_output {
        return print_json(&disks);
    }

    let is_en = is_cli_english();
    println!(
        "=========================================================================================="
    );
    println!(
        "       {} (ssh://{}@{}:{})",
        if is_en { "SSH Disks" } else { "SSH Diskleri" },
        user,
        ip,
        port
    );
    println!(
        "=========================================================================================="
    );
    println!(
        "  {:<16} {:<32} {:<20}",
        if is_en { "Disk ID" } else { "Disk Kimliği" },
        if is_en { "Name / Path" } else { "Adı / Yolu" },
        if is_en { "Size" } else { "Boyut" }
    );
    println!(
        "------------------------------------------------------------------------------------------"
    );
    for d in &disks {
        println!("  {:<16} {:<32} {:<20}", d.id, d.ad, format_bytes(d.boyut));
    }
    println!(
        "=========================================================================================="
    );
    println!(
        "  {} {}",
        if is_en {
            "Total disks:"
        } else {
            "Toplam disk:"
        },
        disks.len()
    );
    println!(
        "=========================================================================================="
    );
    Ok(())
}

fn ssh_tool_check_command(args: Vec<String>) -> Result<(), String> {
    if args.len() < 2 {
        return Err(t_cli(
            "Kullanim: ssh-tool-check <ip> <user> [port] [password] [key_path]",
            "Usage: ssh-tool-check <ip> <user> [port] [password] [key_path]",
        ));
    }
    let ip = args[0].clone();
    let user = args[1].clone();
    let port = args
        .get(2)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(22);
    let password = args.get(3).cloned();
    let key_path = args.get(4).cloned();

    let params = SshConnectionParams {
        ip,
        port,
        user,
        password,
        key_path,
    };
    let mut conn = SshConnection::connect(&params).map_err(|err| err.to_string())?;
    let status = conn.check_ram_tools().map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&status).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn ssh_image_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    let selected_format = extract_output_format(&mut args)?;
    if args.len() < 4 {
        return Err(t_cli(
            "Kullanim: ssh-image <ip> <user> <disk_path> <cikti_klasoru> [case_name] [port] [password] [key_path] [raw|aff4]",
            "Usage: ssh-image <ip> <user> <disk_path> <out_dir> [case_name] [port] [password] [key_path] [raw|aff4]",
        ));
    }
    let ip = args[0].clone();
    let user = args[1].clone();
    let disk_path = args[2].clone();
    let out_dir = PathBuf::from(&args[3]);
    let case_name = args.get(4).cloned();
    let port = args
        .get(5)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(22);
    let password = args.get(6).cloned();
    let key_path = args.get(7).cloned();

    let params = SshConnectionParams {
        ip: ip.clone(),
        port,
        user: user.clone(),
        password,
        key_path,
    };
    let mut conn = SshConnection::connect(&params).map_err(|err| err.to_string())?;
    let result = conn
        .acquire_disk(
            &disk_path,
            &out_dir,
            case_name.as_deref(),
            selected_format,
            |done: u64, _total: u64| {
                eprintln!("{done} bytes transferred");
            },
        )
        .map_err(|err| err.to_string())?;

    if json_output {
        print_json(&json!({
            "target_path": result.target_path,
            "bytes_transferred": result.bytes_transferred,
            "sha256": result.sha256,
            "md5": result.md5,
            "message": result.message,
        }))
    } else {
        println!("============================================================");
        println!(
            "       {}",
            t_cli(
                "SSH Disk İmajı Edinimi Tamamlandı",
                "SSH Disk Image Acquisition Completed"
            )
        );
        println!("============================================================");
        println!(
            "  {:<20}: ssh://{}@{}:{}",
            t_cli("Hedef Sunucu", "Target Server"),
            user,
            ip,
            port
        );
        println!(
            "  {:<20}: {}",
            t_cli("Kaynak Disk", "Source Disk"),
            disk_path
        );
        println!(
            "  {:<20}: {}",
            t_cli("Hedef Dosya", "Target File"),
            result.target_path.display()
        );
        println!(
            "  {:<20}: {}",
            t_cli("Aktarılan Boyut", "Transferred Size"),
            format_bytes(result.bytes_transferred)
        );
        if let Some(h) = &result.sha256 {
            println!("  {:<20}: {}", "SHA-256", h);
        }
        println!("============================================================");
        println!(
            "[✓] {}",
            t_cli(
                "SSH ile disk imajı başarıyla alındı.",
                "Disk image acquired via SSH successfully."
            )
        );
        Ok(())
    }
}

fn ssh_ram_command(args: Vec<String>) -> Result<(), String> {
    let mut args = args;
    let json_output = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    let selected_format = extract_output_format(&mut args)?;
    if args.len() < 3 {
        return Err(t_cli(
            "Kullanim: ssh-ram <ip> <user> <cikti_klasoru> [case_name] [port] [password] [key_path] [raw|aff4]",
            "Usage: ssh-ram <ip> <user> <out_dir> [case_name] [port] [password] [key_path] [raw|aff4]",
        ));
    }
    let ip = args[0].clone();
    let user = args[1].clone();
    let out_dir = PathBuf::from(&args[2]);
    let case_name = args.get(3).cloned();
    let port = args
        .get(4)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(22);
    let password = args.get(5).cloned();
    let key_path = args.get(6).cloned();

    let params = SshConnectionParams {
        ip: ip.clone(),
        port,
        user: user.clone(),
        password,
        key_path,
    };
    let mut conn = SshConnection::connect(&params).map_err(|err| err.to_string())?;
    let result = conn
        .acquire_ram(
            &out_dir,
            case_name.as_deref(),
            selected_format,
            |done: u64, _total: u64| {
                eprintln!("{done} bytes transferred");
            },
        )
        .map_err(|err| err.to_string())?;

    if json_output {
        print_json(&json!({
            "target_path": result.target_path,
            "bytes_transferred": result.bytes_transferred,
            "sha256": result.sha256,
            "md5": result.md5,
            "message": result.message,
        }))
    } else {
        println!("============================================================");
        println!(
            "       {}",
            t_cli(
                "SSH Canlı RAM Edinimi Tamamlandı",
                "SSH Live RAM Acquisition Completed"
            )
        );
        println!("============================================================");
        println!(
            "  {:<20}: ssh://{}@{}:{}",
            t_cli("Hedef Sunucu", "Target Server"),
            user,
            ip,
            port
        );
        println!(
            "  {:<20}: {}",
            t_cli("Hedef Dosya", "Target File"),
            result.target_path.display()
        );
        println!(
            "  {:<20}: {}",
            t_cli("Aktarılan Boyut", "Transferred Size"),
            format_bytes(result.bytes_transferred)
        );
        if let Some(h) = &result.sha256 {
            println!("  {:<20}: {}", "SHA-256", h);
        }
        println!("============================================================");
        println!(
            "[✓] {}",
            t_cli(
                "SSH ile canlı RAM dökümü başarıyla alındı.",
                "Live RAM dump acquired via SSH successfully."
            )
        );
        Ok(())
    }
}

/// Bayt boyutunu okunabilir formata dönüştürür (GB, MB, KB).
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * KB;
    const GB: u64 = 1024 * MB;
    const TB: u64 = 1024 * GB;
    let b_label = t_cli("bayt", "bytes");

    if bytes >= TB {
        format!("{:.2} TB ({} {})", bytes as f64 / TB as f64, bytes, b_label)
    } else if bytes >= GB {
        format!("{:.2} GB ({} {})", bytes as f64 / GB as f64, bytes, b_label)
    } else if bytes >= MB {
        format!("{:.2} MB ({} {})", bytes as f64 / MB as f64, bytes, b_label)
    } else if bytes >= KB {
        format!("{:.2} KB ({} {})", bytes as f64 / KB as f64, bytes, b_label)
    } else {
        format!("{} {}", bytes, b_label)
    }
}

/// Kök/Yönetici yetkisi gerektiren komutlarda yetkiyi doğrular; Linux'ta terminalden sudo parolası ister.
fn ensure_root_or_elevate(op_name: &str) -> Result<(), String> {
    #[cfg(unix)]
    {
        if ram::is_root_or_admin() {
            return Ok(());
        }

        let sudo_exists = Command::new("which")
            .arg("sudo")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !sudo_exists {
            return Err(t_cli(
                &format!(
                    "'{op_name}' işlemi root yetkisi gerektirir ve sistemde 'sudo' bulunamadı. Lütfen root olarak çalıştırın."
                ),
                &format!(
                    "'{op_name}' operation requires root privileges and 'sudo' was not found. Please run as root."
                ),
            ));
        }

        eprintln!(
            "{}",
            t_cli(
                &format!(
                    "[*] '{op_name}' için root (yönetici) yetkisi gerekiyor. Sudo parolası isteniyor..."
                ),
                &format!(
                    "[*] Root privileges required for '{op_name}'. Requesting sudo password..."
                )
            )
        );

        let current_exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let raw_args: Vec<String> = std::env::args().skip(1).collect();

        let mut cmd = Command::new("sudo");
        cmd.arg("-E")
            .arg(current_exe)
            .args(&raw_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let status = cmd.status().map_err(|err| {
            format!(
                "{}: {err}",
                t_cli("sudo çalıştırılamadı", "Failed to run sudo")
            )
        })?;
        if status.success() {
            std::process::exit(0);
        } else {
            let code = status.code().unwrap_or(1);
            std::process::exit(code);
        }
    }

    #[cfg(windows)]
    {
        if ram::is_root_or_admin() {
            return Ok(());
        }
        return Err(t_cli(
            &format!(
                "'{op_name}' işlemi için Yönetici (Administrator) yetkileri gereklidir. Lütfen terminali Yönetici olarak çalıştırın."
            ),
            &format!(
                "Administrator privileges are required for '{op_name}'. Please run terminal as Administrator."
            ),
        ));
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = op_name;
        Ok(())
    }
}

/// Linux canlı RAM edinim durumunu (AVML, /proc/kcore, root yetkisi) gösterir.
fn linux_ram_status_command(args: Vec<String>) -> Result<(), String> {
    if args.iter().any(|a| a == "--json") {
        let status = serde_json::json!({
            "platform": "linux",
            "tool": "avml",
            "avml": ram::avml_status(None),
            "kcore_accessible": Path::new("/proc/kcore").exists(),
            "root_privilege": ram::is_root_or_admin(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&status).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    let status = ram::avml_status(None);
    let is_root = ram::is_root_or_admin();
    let kcore_exists = Path::new("/proc/kcore").exists();
    let ram_display = format_bytes(status.ram_size);

    println!("============================================================");
    println!(
        "       {}",
        t_cli(
            "Linux Canlı RAM Edinim Durumu (AVML)",
            "Linux Live RAM Acquisition Status (AVML)"
        )
    );
    println!("============================================================");
    if let Some(path) = &status.tool_path {
        println!(
            "  {:<22}: [✓] {} ({})",
            t_cli("AVML Durumu", "AVML Status"),
            t_cli("Kurulu ve Hazır", "Installed & Ready"),
            path.display()
        );
    } else {
        println!(
            "  {:<22}: {}",
            t_cli("AVML Durumu", "AVML Status"),
            t_cli(
                "[✗] Kurulu Değil (Kurmak için: 'amele linux ram install')",
                "[✗] Not Installed (To install: 'amele linux ram install')"
            )
        );
    }

    if is_root {
        println!(
            "  {:<22}: [✓] {}",
            t_cli("Yetki Durumu", "Privilege Status"),
            t_cli(
                "Root (Yönetici) Yetkisi Mevcut",
                "Root Privileges Available"
            )
        );
    } else {
        println!(
            "  {:<22}: [!] {}",
            t_cli("Yetki Durumu", "Privilege Status"),
            t_cli(
                "Root Gerekli (Komut otomatik sudo isteyecektir)",
                "Root Required (Command will prompt for sudo)"
            )
        );
    }

    println!(
        "  {:<22}: {}",
        "/proc/kcore",
        if kcore_exists {
            t_cli(
                "[✓] Erişilebilir (Yedek kaynak)",
                "[✓] Accessible (Fallback source)",
            )
        } else {
            t_cli("[✗] Erişilemez", "[✗] Not Accessible")
        }
    );

    println!(
        "  {:<22}: {}",
        t_cli("Fiziksel RAM Boyutu", "Physical RAM Size"),
        ram_display
    );
    println!(
        "  {:<22}: RAW, LiME, AFF4",
        t_cli("Desteklenen Formatlar", "Supported Formats")
    );
    println!("============================================================");

    Ok(())
}

/// Windows canlı RAM edinim durumunu (WinPMEM, yönetici yetkisi) gösterir.
fn windows_ram_status_command(args: Vec<String>) -> Result<(), String> {
    if args.iter().any(|a| a == "--json") {
        let status = serde_json::json!({
            "platform": "windows",
            "tool": "winpmem",
            "winpmem": ram::winpmem_status(None),
            "admin_privilege": ram::is_root_or_admin(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&status).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    let status = ram::winpmem_status(None);
    let is_admin = ram::is_root_or_admin();
    let ram_display = format_bytes(status.ram_size);

    println!("============================================================");
    println!(
        "       {}",
        t_cli(
            "Windows Canlı RAM Edinim Durumu (WinPMEM)",
            "Windows Live RAM Acquisition Status (WinPMEM)"
        )
    );
    println!("============================================================");
    if let Some(path) = &status.tool_path {
        println!(
            "  {:<22}: [✓] {} ({})",
            t_cli("WinPMEM Durumu", "WinPMEM Status"),
            t_cli("Kurulu ve Hazır", "Installed & Ready"),
            path.display()
        );
    } else {
        println!(
            "  {:<22}: {}",
            t_cli("WinPMEM Durumu", "WinPMEM Status"),
            t_cli(
                "[✗] Kurulu Değil (WinPMEM sürücüsü gerekli)",
                "[✗] Not Installed (WinPMEM driver required)"
            )
        );
    }

    if is_admin {
        println!(
            "  {:<22}: [✓] {}",
            t_cli("Yetki Durumu", "Privilege Status"),
            t_cli(
                "Yönetici (Administrator) Yetkisi Mevcut",
                "Administrator Privileges Available"
            )
        );
    } else {
        println!(
            "  {:<22}: [!] {}",
            t_cli("Yetki Durumu", "Privilege Status"),
            t_cli(
                "Yönetici Yetkisi Gerekli",
                "Administrator Privileges Required"
            )
        );
    }

    println!(
        "  {:<22}: {}",
        t_cli("Fiziksel RAM Boyutu", "Physical RAM Size"),
        ram_display
    );
    println!(
        "  {:<22}: RAW, AFF4",
        t_cli("Desteklenen Formatlar", "Supported Formats")
    );
    println!("============================================================");

    Ok(())
}

/// Linux için AVML aracını GitHub release'den otomatik indirip /usr/bin/avml altına kurar.
fn linux_ram_install_command() -> Result<(), String> {
    ensure_root_or_elevate(&t_cli("AVML Kurulumu", "AVML Installation"))?;

    let asset_name = match std::env::consts::ARCH {
        "x86_64" => "avml",
        "aarch64" => "avml-aarch64",
        other => {
            return Err(format!(
                "{}: {other}",
                t_cli("Desteklenmeyen mimari", "Unsupported architecture")
            ));
        }
    };

    let url = format!("https://github.com/microsoft/avml/releases/latest/download/{asset_name}");
    let temp_download =
        amele::settings::secure_runtime_dir().join(format!("avml-{}.tmp", std::process::id()));

    println!(
        "{}",
        t_cli(
            &format!("[*] AVML GitHub release üzerinden indiriliyor: {url}"),
            &format!("[*] Downloading AVML from GitHub release: {url}")
        )
    );

    let output = Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--silent")
        .arg("--show-error")
        .arg("-o")
        .arg(&temp_download)
        .arg(&url)
        .output();

    match output {
        Ok(out) if out.status.success() => {}
        Ok(out) => {
            let _ = fs::remove_file(&temp_download);
            return Err(format!(
                "{}: {}",
                t_cli("AVML indirilemedi", "Failed to download AVML"),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Err(e) => {
            let _ = fs::remove_file(&temp_download);
            return Err(format!(
                "{}: {e}",
                t_cli("curl çalıştırılamadı", "Failed to run curl")
            ));
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&temp_download) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&temp_download, perms);
        }
    }

    let target = Path::new("/usr/bin/avml");
    if let Err(err) = fs::copy(&temp_download, target) {
        let _ = fs::remove_file(&temp_download);
        return Err(format!(
            "{}: {err}",
            t_cli(
                "AVML /usr/bin/avml konumuna kopyalanamadı",
                "Failed to copy AVML to /usr/bin/avml"
            )
        ));
    }
    let _ = fs::remove_file(&temp_download);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(target) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(target, perms);
        }
    }

    let version_output = Command::new(target)
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "AVML".to_string());

    println!(
        "{}",
        t_cli(
            &format!(
                "[✓] AVML başarıyla kuruldu: /usr/bin/avml ({})",
                version_output.trim()
            ),
            &format!(
                "[✓] AVML successfully installed: /usr/bin/avml ({})",
                version_output.trim()
            )
        )
    );

    Ok(())
}

/// Windows için WinPMEM aracını indirir.
fn windows_ram_install_command() -> Result<(), String> {
    #[cfg(not(windows))]
    {
        return Err(t_cli(
            "WinPMEM sadece Windows üzerinde kurulabilir.",
            "WinPMEM can only be installed on Windows.",
        ));
    }

    #[cfg(windows)]
    {
        println!(
            "{}",
            t_cli(
                "[*] WinPMEM indirme URL'si: https://amele.noirlang.tr/go-winpmem_amd64_1.0-rc2_signed.exe",
                "[*] WinPMEM download URL: https://amele.noirlang.tr/go-winpmem_amd64_1.0-rc2_signed.exe"
            )
        );
        let target = amele::settings::secure_runtime_dir().join("winpmem.exe");
        crate::api::download_file_to_path(
            "https://amele.noirlang.tr/go-winpmem_amd64_1.0-rc2_signed.exe",
            &target,
            "WinPMEM download failed",
        )?;
        println!(
            "{}",
            t_cli(
                &format!("[✓] WinPMEM indirildi: {}", target.display()),
                &format!("[✓] WinPMEM downloaded: {}", target.display())
            )
        );
        Ok(())
    }
}

/// Disk listesini temiz terminal tablosu veya JSON olarak gösterir.
fn disk_list_command_with_args(args: Vec<String>) -> Result<(), String> {
    let disks = disk::list_disks().map_err(|err| err.to_string())?;
    if args.iter().any(|a| a == "--json") {
        return print_json(&disks);
    }

    if disks.is_empty() {
        println!(
            "{}",
            t_cli(
                "Hiçbir yerel disk sürücüsü tespit edilemedi.",
                "No local disk drives detected."
            )
        );
        return Ok(());
    }

    let is_en = is_cli_english();
    let title = if is_en {
        "Local Disk Drives & Partitions"
    } else {
        "Yerel Disk Sürücüleri ve Bölüntüler"
    };

    println!(
        "=========================================================================================="
    );
    println!("       {title}");
    println!(
        "=========================================================================================="
    );
    println!(
        "  {:<28} {:<16} {:<16} {:<14} {:<10}",
        if is_en {
            "Device / Path"
        } else {
            "Aygıt / Yol"
        },
        if is_en { "Total Size" } else { "Toplam Boyut" },
        if is_en {
            "Used Size"
        } else {
            "Kullanılan Boyut"
        },
        if is_en { "Access" } else { "Erişim" },
        if is_en { "Status" } else { "Durum" }
    );
    println!(
        "------------------------------------------------------------------------------------------"
    );

    for d in &disks {
        let dev_str = d.device.display().to_string();
        let total_str = format_bytes(d.total_size);
        let used_str = if d.used_size > 0 {
            format_bytes(d.used_size)
        } else {
            "-".to_string()
        };
        let access_str = if d.accessible {
            if is_en {
                "[✓] Accessible"
            } else {
                "[✓] Okunabilir"
            }
        } else {
            if is_en {
                "[!] Root req."
            } else {
                "[!] Root Gerekli"
            }
        };
        let status_str = if d.accessible {
            if is_en { "Ready" } else { "Hazır" }
        } else {
            if is_en { "Restricted" } else { "Kısıtlı" }
        };

        println!(
            "  {:<28} {:<16} {:<16} {:<14} {:<10}",
            dev_str, total_str, used_str, access_str, status_str
        );
    }

    println!(
        "=========================================================================================="
    );
    println!(
        "  {} {}",
        if is_en {
            "Total devices found:"
        } else {
            "Toplam bulunan aygıt:"
        },
        disks.len()
    );
    println!(
        "=========================================================================================="
    );

    Ok(())
}

/// Vaka depolarını listeler.
fn case_list_command(args: Vec<String>) -> Result<(), String> {
    let base_dir = amele::api::default_case_base_dir();
    if !base_dir.is_dir() {
        println!(
            "{}",
            t_cli(
                "Henüz oluşturulmuş vaka bulunmuyor.",
                "No cases created yet."
            )
        );
        return Ok(());
    }
    let mut cases = Vec::new();
    if let Ok(entries) = fs::read_dir(&base_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !name.starts_with('.') {
                        cases.push((name.to_string(), path));
                    }
                }
            }
        }
    }
    if args.iter().any(|a| a == "--json") {
        let json_list: Vec<_> = cases
            .iter()
            .map(|(n, p)| json!({ "name": n, "path": p }))
            .collect();
        return print_json(&json_list);
    }
    if cases.is_empty() {
        println!(
            "{}",
            t_cli(
                "Henüz oluşturulmuş vaka bulunmuyor.",
                "No cases created yet."
            )
        );
        return Ok(());
    }
    let is_en = is_cli_english();
    println!("============================================================");
    println!(
        "       {}",
        if is_en {
            "Forensic Cases"
        } else {
            "Adli Vakalar"
        }
    );
    println!("============================================================");
    for (name, path) in &cases {
        println!("  • {:<20} ({})", name, path.display());
    }
    println!("============================================================");
    println!(
        "  {} {}",
        if is_en {
            "Total cases:"
        } else {
            "Toplam vaka:"
        },
        cases.len()
    );
    println!("============================================================");
    Ok(())
}

/// Yeni bir vaka deposu oluşturur.
fn case_create_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: case create <vaka_adi>",
            "Usage: case create <case_name>",
        ));
    }
    let case_name = amele::api::sanitize_case_name(&args[0]);
    if case_name.is_empty() {
        return Err(t_cli("Geçersiz vaka adı", "Invalid case name"));
    }
    let base_dir = amele::api::default_case_base_dir();
    let vault =
        amele::evidence::EvidenceVault::create(&base_dir, &case_name).map_err(|e| e.to_string())?;
    println!(
        "{}",
        t_cli(
            &format!(
                "[✓] Vaka başarıyla oluşturuldu: {} ({})",
                vault.case_name,
                vault.case_dir.display()
            ),
            &format!(
                "[✓] Case created successfully: {} ({})",
                vault.case_name,
                vault.case_dir.display()
            )
        )
    );
    Ok(())
}

/// Vaka deposu detaylarını ve delil istatistiklerini gösterir.
fn case_info_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: case info <vaka_adi>",
            "Usage: case info <case_name>",
        ));
    }
    let case_name = amele::api::sanitize_case_name(&args[0]);
    if case_name.is_empty() {
        return Err(t_cli("Geçersiz vaka adı", "Invalid case name"));
    }
    let base_dir = amele::api::default_case_base_dir();
    let case_dir = base_dir.join(&case_name);
    if !case_dir.is_dir() {
        return Err(format!(
            "{} '{}'",
            t_cli("Vaka klasörü bulunamadı:", "Case directory not found:"),
            case_dir.display()
        ));
    }

    let is_en = is_cli_english();
    println!("============================================================");
    println!(
        "       {} : {}",
        if is_en {
            "Case Details"
        } else {
            "Vaka Detayları"
        },
        case_name
    );
    println!("============================================================");
    println!(
        "  {:<20}: {}",
        if is_en { "Directory" } else { "Klasör" },
        case_dir.display()
    );

    let evidence_dir = case_dir.join("deliller");
    if evidence_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&evidence_dir) {
            let files: Vec<_> = entries.flatten().filter(|e| e.path().is_file()).collect();
            println!(
                "  {:<20}: {}",
                if is_en {
                    "Evidence Files"
                } else {
                    "Delil Dosyaları"
                },
                files.len()
            );
            for f in files.iter().take(10) {
                let size = f.metadata().map(|m| m.len()).unwrap_or(0);
                println!(
                    "    • {} ({})",
                    f.file_name().to_string_lossy(),
                    format_bytes(size)
                );
            }
            if files.len() > 10 {
                println!(
                    "    ... (+{} {})",
                    files.len() - 10,
                    if is_en { "more" } else { "daha" }
                );
            }
        }
    }

    let logs_dir = case_dir.join("gunlukler");
    if logs_dir.is_dir() {
        if let Ok(entries) = fs::read_dir(&logs_dir) {
            let count = entries.flatten().count();
            println!(
                "  {:<20}: {} {}",
                if is_en { "Logs" } else { "Günlükler" },
                count,
                if is_en { "files" } else { "dosya" }
            );
        }
    }
    println!("============================================================");
    Ok(())
}

/// Kabuk otomatik tamamlama (shell autocompletion) scripti üretir.
fn completion_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!(
            "{}",
            t_cli(
                r#"Amele Kabuk Otomatik Tamamlama Üreticisi

KULLANIM:
  amele completion <bash|zsh|fish>

ÖRNEKLER:
  # Bash:
  source <(amele completion bash)

  # Zsh:
  source <(amele completion zsh)

  # Fish:
  amele completion fish > ~/.config/fish/completions/amele.fish"#,
                r#"Amele Shell Autocompletion Generator

USAGE:
  amele completion <bash|zsh|fish>

EXAMPLES:
  # Bash:
  source <(amele completion bash)

  # Zsh:
  source <(amele completion zsh)

  # Fish:
  amele completion fish > ~/.config/fish/completions/amele.fish"#
            )
        );
        return Ok(());
    }

    match args[0].to_lowercase().as_str() {
        "bash" => {
            print!("{}", amele::completion::generate_bash_completion());
            Ok(())
        }
        "zsh" => {
            print!("{}", amele::completion::generate_zsh_completion());
            Ok(())
        }
        "fish" => {
            print!("{}", amele::completion::generate_fish_completion());
            Ok(())
        }
        other => Err(format!(
            "{} '{}'. {}",
            t_cli("Desteklenmeyen kabuk:", "Unsupported shell:"),
            other,
            t_cli(
                "Desteklenenler: bash, zsh, fish",
                "Supported shells: bash, zsh, fish"
            )
        )),
    }
}

/// Disk imajını salt-okunur bağlar (mount).
fn mount_file_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanim: mount <imaj_dosyasi> [baglama_noktasi]",
            "Usage: mount <image_file> [mount_point]",
        ));
    }
    let image_path = PathBuf::from(&args[0]);
    if !image_path.is_file() {
        return Err(t_cli(
            &format!("İmaj dosyası bulunamadı: {}", image_path.display()),
            &format!("Image file not found: {}", image_path.display()),
        ));
    }

    ensure_root_or_elevate(&t_cli("İmaj Bağlama", "Image Mount"))?;

    let mount_dir = if let Some(p) = args.get(1) {
        PathBuf::from(p)
    } else {
        let ts = cli_timestamp();
        let stem = cli_safe_stem(
            image_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("mount"),
        );
        amele::settings::secure_runtime_dir().join(format!("amele-mount-{stem}-{ts}"))
    };

    fs::create_dir_all(&mount_dir).map_err(|e| e.to_string())?;

    #[cfg(target_os = "linux")]
    {
        println!(
            "{}",
            t_cli(
                "[*] Disk imajı salt-okunur bağlanıyor...",
                "[*] Mounting disk image read-only..."
            )
        );
        let loop_dev = amele::api::linux_mount_partitioned_image(&image_path, &mount_dir)?;
        let loop_str = loop_dev
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{}",
            t_cli(
                &format!(
                    "[✓] İmaj başarıyla bağlandı!\n  Bağlama Noktası: {}\n  Loop Aygıtı: {}",
                    mount_dir.display(),
                    loop_str
                ),
                &format!(
                    "[✓] Image mounted successfully!\n  Mount Point: {}\n  Loop Device: {}",
                    mount_dir.display(),
                    loop_str
                )
            )
        );
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(t_cli(
            "Bu işletim sisteminde doğrudan CLI mount desteklenmiyor. Lütfen GUI üzerinden bağlayın.",
            "Direct CLI mount is not supported on this operating system. Please mount via GUI.",
        ))
    }
}

fn ram_status_command(args: Vec<String>) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        windows_ram_status_command(args)
    }
    #[cfg(not(target_os = "windows"))]
    {
        linux_ram_status_command(args)
    }
}

fn wireguard_config_command(args: Vec<String>) -> Result<(), String> {
    let Some(path) = args.first() else {
        return Err("Kullanim: wireguard-config <dosya>".to_string());
    };
    let written = wireguard::create_config(path, &WireGuardConfig::default())
        .map_err(|err| err.to_string())?;
    println!("{}", written.display());
    Ok(())
}

fn parse_port(value: &str) -> Result<u16, String> {
    value
        .parse::<u16>()
        .map_err(|_| {
            t_cli(
                "Port 1 ile 65535 arasında olmalıdır.",
                "Port must be between 1 and 65535.",
            )
        })
        .and_then(|port| {
            if port == 0 {
                Err(t_cli(
                    "Port 1 ile 65535 arasında olmalıdır.",
                    "Port must be between 1 and 65535.",
                ))
            } else {
                Ok(port)
            }
        })
}

fn update_check_command(args: Vec<String>) -> Result<(), String> {
    let json_output = args.iter().any(|arg| arg == "--json");
    let response = api::update::update_check_endpoint();
    if response.status != 200 {
        let err_msg = String::from_utf8_lossy(&response.body);
        return Err(format!(
            "{}: {err_msg}",
            t_cli("Güncelleme kontrolü başarısız oldu", "Update check failed")
        ));
    }
    let data: serde_json::Value = serde_json::from_slice(&response.body).map_err(|e| {
        format!(
            "{}: {e}",
            t_cli("Yanıt çözümlenemedi", "Failed to parse response")
        )
    })?;

    if json_output {
        println!("{}", serde_json::to_string_pretty(&data).unwrap());
        return Ok(());
    }

    let is_en = is_cli_english();
    let current = data
        .get("current_version")
        .and_then(|v| v.as_str())
        .unwrap_or("v0.0.0");
    let latest_tag = data
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or(if is_en { "unknown" } else { "bilinmiyor" });
    let release_name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(latest_tag);
    let html_url = data.get("html_url").and_then(|v| v.as_str()).unwrap_or("");

    let target = data.get("update_target");
    let pkg_label = target
        .and_then(|t| t.get("package_label"))
        .and_then(|v| v.as_str())
        .unwrap_or(if is_en { "Unknown" } else { "Bilinmeyen" });
    let detected_by = target
        .and_then(|t| t.get("detected_by"))
        .and_then(|v| v.as_str())
        .unwrap_or(if is_en { "system" } else { "sistem" });
    let install_cmd = target
        .and_then(|t| t.get("install_command"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let asset = data.get("platform_asset");
    let asset_name = asset
        .and_then(|a| a.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or(if is_en { "not found" } else { "bulunamadı" });
    let asset_url = asset
        .and_then(|a| a.get("download_url"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let asset_size = asset
        .and_then(|a| a.get("size"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let clean_current = current.trim_start_matches('v');
    let clean_latest = latest_tag.trim_start_matches('v');
    let has_update = clean_latest != clean_current && !clean_latest.is_empty();

    println!("============================================================");
    println!(
        "       {}",
        if is_en {
            "Amele Forensic Tool Update Check"
        } else {
            "Amele Forensic Tool Güncelleme Kontrolü"
        }
    );
    println!("============================================================");
    println!(
        "  {:<20}: v{}",
        if is_en {
            "Current Version"
        } else {
            "Mevcut Sürüm"
        },
        clean_current
    );
    println!(
        "  {:<20}: {}",
        if is_en {
            "Latest Version"
        } else {
            "Son Sürüm"
        },
        if latest_tag.starts_with('v') {
            latest_tag.to_string()
        } else {
            format!("v{}", latest_tag)
        }
    );
    println!(
        "  {:<20}: {} ({}: {})",
        if is_en {
            "Package Type"
        } else {
            "Paket Türü"
        },
        pkg_label,
        if is_en { "Detected by" } else { "Algılama" },
        detected_by
    );

    if has_update {
        println!(
            "\n  [!] {} ({})",
            if is_en {
                "NEW VERSION AVAILABLE!"
            } else {
                "YENİ SÜRÜM MEVCUT!"
            },
            release_name
        );
        println!(
            "  {:<20}: {}",
            if is_en {
                "Download File"
            } else {
                "İndirilecek Dosya"
            },
            asset_name
        );
        if asset_size > 0 {
            println!(
                "  {:<20}: {:.2} MB",
                if is_en { "File Size" } else { "Dosya Boyutu" },
                asset_size as f64 / 1_048_576.0
            );
        }
        if !asset_url.is_empty() {
            println!(
                "  {:<20}: {}",
                if is_en {
                    "Download Link"
                } else {
                    "İndirme Bağlantısı"
                },
                asset_url
            );
        }
        if !html_url.is_empty() {
            println!(
                "  {:<20}: {}",
                if is_en {
                    "Release Notes"
                } else {
                    "Sürüm Notları"
                },
                html_url
            );
        }
        if !install_cmd.is_empty() {
            println!(
                "\n  {}:\n    {}",
                if is_en {
                    "Recommended Install Command"
                } else {
                    "Önerilen Kurulum Komutu"
                },
                install_cmd
            );
        }
    } else {
        println!(
            "\n  [✓] {}",
            if is_en {
                "Your system is up to date. You are using the latest version."
            } else {
                "Sisteminiz güncel. En son sürümü kullanıyorsunuz."
            }
        );
    }
    println!("============================================================");

    Ok(())
}

fn preflight_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanım: amele preflight <kaynak_yol> <kaynak_tipi> [hedef_yol]\nKaynak tipleri: disk, ram, android, ios",
            "Usage: amele preflight <source_path> <source_type> [target_path]\nSource types: disk, ram, android, ios",
        ));
    }
    let is_en = is_cli_english();
    let source_path = &args[0];
    let source_type = args.get(1).map(|s| s.as_str()).unwrap_or("disk");
    let target_path = args
        .get(2)
        .map(|s| std::path::PathBuf::from(s))
        .unwrap_or_else(|| amele::api::default_case_base_dir());

    let result = amele::storage_guard::preflight_check(source_path, source_type, &target_path);
    println!("============================================================");
    println!(
        "       {}",
        if is_en {
            "Storage Preflight Space Check"
        } else {
            "Disk Alanı Ön Kontrolü"
        }
    );
    println!("============================================================");
    println!(
        "  {:<16}: {}",
        if is_en { "Source" } else { "Kaynak" },
        source_path
    );
    println!(
        "  {:<16}: {}",
        if is_en { "Source Type" } else { "Kaynak Tipi" },
        source_type
    );
    println!(
        "  {:<16}: {:.2} GB",
        if is_en {
            "Source Size"
        } else {
            "Kaynak Boyutu"
        },
        result.source_bytes as f64 / 1_073_741_824.0
    );
    println!(
        "  {:<16}: {:.2} GB",
        if is_en {
            "Available Space"
        } else {
            "Boş Alan"
        },
        result.available_bytes as f64 / 1_073_741_824.0
    );
    if result.is_sufficient {
        println!(
            "\n  [✓] {}",
            if is_en {
                "Sufficient storage space available."
            } else {
                "Yeterli disk alanı mevcut."
            }
        );
    } else {
        println!(
            "\n  [!] {}",
            if is_en {
                "INSUFFICIENT DISK SPACE!"
            } else {
                "YETERSİZ DİSK ALANI!"
            }
        );
        println!(
            "  {:<16}: {:.2} GB",
            if is_en { "Shortage" } else { "Eksik" },
            result.shortage_bytes as f64 / 1_073_741_824.0
        );
        if let Some(msg) = &result.warning_message {
            println!("  {:<16}: {}", if is_en { "Warning" } else { "Uyarı" }, msg);
        }
    }
    println!("============================================================");
    Ok(())
}

fn mount_list_command() -> Result<(), String> {
    let mounts = amele::mount_tracker::list_active_mounts();
    if mounts.is_empty() {
        println!(
            "{}",
            t_cli(
                "Aktif bağlı imaj bulunmuyor.",
                "No active mounted images found."
            )
        );
        return Ok(());
    }
    let is_en = is_cli_english();
    println!("============================================================");
    println!(
        "       {}",
        if is_en {
            "Active Mounted Images"
        } else {
            "Aktif Bağlı İmajlar"
        }
    );
    println!("============================================================");
    for m in &mounts {
        println!("  ID          : {}", m.mount_id);
        println!(
            "  {:<12}: {}",
            if is_en { "Case" } else { "Vaka" },
            m.case_name
        );
        println!(
            "  {:<12}: {}",
            if is_en { "Image" } else { "İmaj" },
            m.image_path.display()
        );
        println!(
            "  {:<12}: {}",
            if is_en { "Mount Point" } else { "Bağlama" },
            m.mount_point.display()
        );
        println!(
            "  {:<12}: {}",
            if is_en { "Date" } else { "Tarih" },
            m.mounted_at
        );
        println!("  ---");
    }
    println!("============================================================");
    println!(
        "  {} {}",
        if is_en { "Total mounts:" } else { "Toplam:" },
        mounts.len()
    );
    println!("============================================================");
    Ok(())
}

fn mount_cleanup_command(args: Vec<String>) -> Result<(), String> {
    let is_en = is_cli_english();
    let case_filter = args.first().map(|s| s.as_str());
    let cleaned = if let Some(case) = case_filter {
        println!(
            "{}",
            t_cli(
                &format!("Vaka '{case}' için mount temizliği yapılıyor..."),
                &format!("Cleaning up mounts for case '{case}'...")
            )
        );
        amele::mount_tracker::cleanup_case_mounts(case)
    } else {
        println!(
            "{}",
            t_cli(
                "Tüm aktif mount'lar temizleniyor...",
                "Cleaning up all active mounts..."
            )
        );
        amele::mount_tracker::cleanup_all_mounts()
    };
    if cleaned.is_empty() {
        println!(
            "{}",
            t_cli(
                "Temizlenecek mount bulunamadı.",
                "No mounts found to clean up."
            )
        );
    } else {
        for id in &cleaned {
            println!(
                "  [✓] {}: {}",
                if is_en { "Cleaned" } else { "Temizlendi" },
                id
            );
        }
        println!(
            "{}",
            t_cli(
                &format!("Toplam {} mount temizlendi.", cleaned.len()),
                &format!("Total {} mounts cleaned up.", cleaned.len())
            )
        );
    }
    Ok(())
}

fn case_export_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanım: amele case-export <vaka_adi> [hedef_dosya]",
            "Usage: amele case-export <case_name> [target_file]",
        ));
    }
    let is_en = is_cli_english();
    let case_name = &args[0];
    let base_dir = amele::api::default_case_base_dir();
    let vault =
        amele::evidence::EvidenceVault::create(&base_dir, case_name).map_err(|e| e.to_string())?;

    let output_path = if let Some(p) = args.get(1) {
        std::path::PathBuf::from(p)
    } else {
        base_dir.clone()
    };

    println!(
        "{}",
        t_cli(
            &format!("Vaka dışa aktarılıyor: {case_name}"),
            &format!("Exporting case: {case_name}")
        )
    );
    let package_path =
        amele::case_package::export_case(&vault, &output_path).map_err(|e| e.to_string())?;
    println!(
        "[✓] {}: {}",
        if is_en {
            "Package created"
        } else {
            "Paket oluşturuldu"
        },
        package_path.display()
    );

    let hash_path = package_path.with_extension(format!(
        "{}sha256",
        package_path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{ext}."))
            .unwrap_or_default()
    ));
    if hash_path.is_file() {
        println!(
            "[✓] {}: {}",
            if is_en {
                "Checksum file"
            } else {
                "Hash dosyası"
            },
            hash_path.display()
        );
    }
    Ok(())
}

fn case_import_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanım: amele case-import <amelecase_dosyasi>",
            "Usage: amele case-import <amelecase_file>",
        ));
    }
    let package_path = std::path::Path::new(&args[0]);
    if !package_path.is_file() {
        return Err(t_cli(
            &format!("Dosya bulunamadı: {}", package_path.display()),
            &format!("File not found: {}", package_path.display()),
        ));
    }

    let is_en = is_cli_english();
    let base_dir = amele::api::default_case_base_dir();
    println!(
        "{}",
        t_cli(
            &format!("Vaka içe aktarılıyor: {}", package_path.display()),
            &format!("Importing case: {}", package_path.display())
        )
    );

    let result =
        amele::case_package::import_case(package_path, &base_dir).map_err(|e| e.to_string())?;

    println!(
        "[✓] {:<20}: {}",
        if is_en {
            "Case imported"
        } else {
            "Vaka aktarıldı"
        },
        result.case_name
    );
    println!(
        "  {:<20}: {}",
        if is_en {
            "Target directory"
        } else {
            "Hedef klasör"
        },
        result.case_dir.display()
    );
    println!(
        "  {:<20}: {}",
        if is_en {
            "Extracted files"
        } else {
            "Dosya sayısı"
        },
        result.files_extracted
    );
    println!(
        "  {:<20}: {}",
        if is_en {
            "Integrity verified"
        } else {
            "Bütünlük doğrulandı"
        },
        if result.integrity_verified {
            if is_en { "Yes [✓]" } else { "Evet [✓]" }
        } else {
            if is_en { "No [✗]" } else { "Hayır [✗]" }
        }
    );
    for warn in &result.warnings {
        println!("  [!] {}", warn);
    }
    Ok(())
}

fn case_verify_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err(t_cli(
            "Kullanım: amele case-verify <amelecase_dosyasi>",
            "Usage: amele case-verify <amelecase_file>",
        ));
    }
    let package_path = std::path::Path::new(&args[0]);
    if !package_path.is_file() {
        return Err(t_cli(
            &format!("Dosya bulunamadı: {}", package_path.display()),
            &format!("File not found: {}", package_path.display()),
        ));
    }

    println!(
        "{}",
        t_cli(
            &format!("Paket bütünlüğü doğrulanıyor: {}", package_path.display()),
            &format!("Verifying package integrity: {}", package_path.display())
        )
    );
    let verified = amele::case_package::verify_package(package_path).map_err(|e| e.to_string())?;

    if verified {
        println!(
            "[✓] {}",
            t_cli(
                "Paket bütünlüğü doğrulandı. SHA-256 hash eşleşiyor.",
                "Package integrity verified. SHA-256 checksum matches."
            )
        );
    } else {
        println!(
            "[✗] {}",
            t_cli(
                "UYARI: Paket bütünlüğü doğrulanamadı! SHA-256 hash eşleşmiyor.",
                "WARNING: Package integrity check failed! SHA-256 checksum does not match."
            )
        );
    }
    Ok(())
}
