use std::path::PathBuf;
use worm_rewrite_rust::disk;
use worm_rewrite_rust::hash::{self, HashAlgorithm};
use worm_rewrite_rust::settings::AppSettings;

fn main() {
    let mut args = std::env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("settings-default") => print_default_settings(),
        Some("hash") => hash_command(args.collect()),
        Some("disk-list") => disk_list_command(),
        Some("disk-size") => disk_size_command(args.collect()),
        Some("verify") => verify_command(args.collect()),
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
           disk-size <cihaz|dosya>       Disk veya dosya boyutu al\n\
           verify <imaj> <sha256>        SHA256 imaj dogrulama yap\n\n\
         Not: UI bu crate'e daha sonra Tauri tarafindan baglanacak."
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
