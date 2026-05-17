use serde_json::json;
use std::fs;
use std::path::PathBuf;
use worm_rewrite_rust::disk;
use worm_rewrite_rust::hash::{self, HashAlgorithm};
use worm_rewrite_rust::ram;
use worm_rewrite_rust::remote::RemoteConnection;
use worm_rewrite_rust::settings::AppSettings;
use worm_rewrite_rust::ui_server;
use worm_rewrite_rust::wireguard::{self, WireGuardConfig};

fn main() {
    let mut args = std::env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("settings-default") => print_default_settings(),
        Some("hash") => hash_command(args.collect()),
        Some("disk-list") => disk_list_command(),
        Some("disk-list-helper") => disk_list_helper_command(args.collect()),
        Some("disk-size") => disk_size_command(args.collect()),
        Some("verify") => verify_command(args.collect()),
        Some("remote-disks") => remote_disks_command(args.collect()),
        Some("remote-image") => remote_image_command(args.collect()),
        Some("remote-tool-check") => remote_tool_check_command(args.collect()),
        Some("ram-status") => ram_status_command(),
        Some("wireguard-config") => wireguard_config_command(args.collect()),
        Some("ui") => ui_server::run_native(),
        Some("ui-browser") => ui_server::run_browser(),
        Some("--help") | Some("-h") | None => {
            print_help();
            Ok(())
        }
        Some(other) => Err(format!("Bilinmeyen komut: {other}")),
    };

    if let Err(err) = result {
        eprintln!("{err}");
        print_help();
        std::process::exit(2);
    }
}

fn print_help() {
    println!(
        "worm-rewrite-rust teknik CLI\n\n\
         Komutlar:\n\
           settings-default              Varsayilan ayarlari JSON olarak yazdir\n\
           hash <dosya> [algoritma]      md5/sha1/sha256/sha512 hash hesapla\n\
           disk-list                     Yerel diskleri listele\n\
           disk-list-helper <json>        Yetkili disk listeleme yardimci komutu\n\
           disk-size <cihaz|dosya>       Disk veya dosya boyutu al\n\
           verify <imaj> <sha256>        SHA256 imaj dogrulama yap\n\n\
           remote-disks <ip> <port> [token]\n\
           remote-image <ip> <port> <disk_id> <cikti_klasoru> [token]\n\
           remote-tool-check <ip> <port> <winpmem|avml> [token]\n\
           ram-status                    Yerel AVML/WinPMEM durumunu yazdir\n\
           wireguard-config <dosya>      Varsayilan WireGuard config uret\n\n\
           ui                            Rust backend'e bagli native uygulama penceresini ac\n\
           ui-browser                    Debug icin tarayici penceresiyle ac\n\n\
         Not: ui komutu yerel HTTP backend baslatir ve GTK/WebKit penceresini buraya baglar."
    );
}

fn print_default_settings() -> Result<(), String> {
    let settings = AppSettings::default();
    println!(
        "{}",
        serde_json::to_string_pretty(&settings).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn hash_command(args: Vec<String>) -> Result<(), String> {
    if args.is_empty() {
        return Err("Kullanim: hash <dosya> [algoritma]".to_string());
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

fn disk_list_command() -> Result<(), String> {
    let disks = disk::list_disks().map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&disks).map_err(|err| err.to_string())?
    );
    Ok(())
}

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
        return Err("Kullanim: verify <imaj> <sha256>".to_string());
    }
    let ok = disk::verify_image(&args[0], &args[1]).map_err(|err| err.to_string())?;
    println!("{}", if ok { "OK" } else { "FAIL" });
    Ok(())
}

fn remote_disks_command(args: Vec<String>) -> Result<(), String> {
    if args.len() < 2 {
        return Err("Kullanim: remote-disks <ip> <port> [token]".to_string());
    }
    let port = parse_port(&args[1])?;
    let token = args.get(2).cloned();
    let mut connection =
        RemoteConnection::connect(&args[0], port, token).map_err(|err| err.to_string())?;
    let disks = connection.list_disks().map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&disks).map_err(|err| err.to_string())?
    );
    Ok(())
}

fn remote_image_command(args: Vec<String>) -> Result<(), String> {
    if args.len() < 4 {
        return Err(
            "Kullanim: remote-image <ip> <port> <disk_id> <cikti_klasoru> [token]".to_string(),
        );
    }
    let port = parse_port(&args[1])?;
    let token = args.get(4).cloned();
    let mut connection =
        RemoteConnection::connect(&args[0], port, token).map_err(|err| err.to_string())?;
    let result = connection
        .acquire_image(&args[2], &args[3], None, |done, total| {
            if let Some(percent) = done.saturating_mul(100).checked_div(total) {
                eprintln!("{}%", percent);
            }
        })
        .map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&result).map_err(|err| err.to_string())?
    );
    Ok(())
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

fn ram_status_command() -> Result<(), String> {
    let status = serde_json::json!({
        "avml": ram::avml_status(None),
        "winpmem": ram::winpmem_status(None),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&status).map_err(|err| err.to_string())?
    );
    Ok(())
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
        .map_err(|_| "Port 1 ile 65535 arasinda olmali".to_string())
        .and_then(|port| {
            if port == 0 {
                Err("Port 1 ile 65535 arasinda olmali".to_string())
            } else {
                Ok(port)
            }
        })
}
