//! Komut satırı girişini işler ve UI, browser veya helper modlarını başlatır.
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use worm::disk;
use worm::hash::{self, HashAlgorithm};
use worm::ram;
use worm::remote::RemoteConnection;
use worm::server;
use worm::settings::AppSettings;
use worm::wireguard::{self, WireGuardConfig};

/// CLI argümanını okuyup ilgili alt komutu veya UI modunu çalıştırır.
fn main() {
    install_error_reporting();

    let mut args = std::env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("settings-default") => print_default_settings(),
        Some("hash") => hash_command(args.collect()),
        Some("disk-list") => disk_list_command(),
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
        Some("ram-status") => ram_status_command(),
        Some("wireguard-config") => wireguard_config_command(args.collect()),
        Some("ui") => server::run_native(),
        Some("ui-browser") => server::run_browser(),
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        None => default_command(),
        Some(other) => Err(format!("Bilinmeyen komut: {other}")),
    };

    if let Err(err) = result {
        report_fatal_error(&err);
        eprintln!("{err}");
        print_help();
        std::process::exit(2);
    }
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
            "Unexpected Worm startup crash:\n\n{info}\n\nLocation: {location}"
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

        let mut body = format!("Worm Forensic Tool could not start.\n\n{message}");
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
            .map(|base| base.join("Worm").join("worm.log"))
    }

    fn show_message(message: &str) {
        let title = wide_null("Worm Forensic Tool");
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

/// Kullanıcıya desteklenen teknik CLI komutlarını gösterir.
fn print_help() {
    println!(
        "worm teknik CLI\n\n\
         Komutlar:\n\
           settings-default              Varsayilan ayarlari JSON olarak yazdir\n\
           hash <dosya> [algoritma]      md5/sha1/sha256/sha512 hash hesapla\n\
           disk-list                     Yerel diskleri listele\n\
           disk-list-helper <json>        Yetkili disk listeleme yardimci komutu\n\
           image-helper <req> <res> <prg> [ctrl] Yetkili imaj alma yardimci komutu\n\
           ram-helper <req> <res> <prg> <ctrl> Yetkili RAM alma yardimci komutu\n\
           avml-install-helper <kaynak> <res> Yetkili AVML kurulum yardimci komutu\n\
           winpmem-install-helper <kaynak> <res> Yetkili WinPMEM kurulum yardimci komutu\n\
           mount-helper <req> <res>       Yetkili imaj mount yardimci komutu\n\
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

/// Varsayılan uygulama ayarlarını JSON olarak stdout'a yazar.
fn print_default_settings() -> Result<(), String> {
    let settings = AppSettings::default();
    println!(
        "{}",
        serde_json::to_string_pretty(&settings).map_err(|err| err.to_string())?
    );
    Ok(())
}

/// Verilen dosya için seçilen hash algoritmasını çalıştırır.
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

/// Yerel disk listesini JSON olarak üretir.
fn disk_list_command() -> Result<(), String> {
    let disks = disk::list_disks().map_err(|err| err.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&disks).map_err(|err| err.to_string())?
    );
    Ok(())
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
        _ => Err(worm::error::WormError::new(
            worm::error::HataKodu::Genel,
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
    let temp = Path::new("/usr/bin/.worm-avml.tmp");
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
    let temp = target_dir.join(".worm-winpmem.tmp");
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
        .acquire_image(&args[2], None, &args[3], None, |done, total| {
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
