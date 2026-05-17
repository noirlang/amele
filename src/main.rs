use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use worm::disk;
use worm::hash::{self, HashAlgorithm};
use worm::ram;
use worm::remote::RemoteConnection;
use worm::settings::AppSettings;
use worm::ui_server;
use worm::wireguard::{self, WireGuardConfig};

fn main() {
    let mut args = std::env::args().skip(1);
    let result = match args.next().as_deref() {
        Some("settings-default") => print_default_settings(),
        Some("hash") => hash_command(args.collect()),
        Some("disk-list") => disk_list_command(),
        Some("disk-list-helper") => disk_list_helper_command(args.collect()),
        Some("image-helper") => image_helper_command(args.collect()),
        Some("mount-helper") => mount_helper_command(args.collect()),
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
        "worm teknik CLI\n\n\
         Komutlar:\n\
           settings-default              Varsayilan ayarlari JSON olarak yazdir\n\
           hash <dosya> [algoritma]      md5/sha1/sha256/sha512 hash hesapla\n\
           disk-list                     Yerel diskleri listele\n\
           disk-list-helper <json>        Yetkili disk listeleme yardimci komutu\n\
           image-helper <req> <res> <prg> [ctrl] Yetkili imaj alma yardimci komutu\n\
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

#[derive(Deserialize)]
struct ImageHelperRequest {
    source: PathBuf,
    target: PathBuf,
    owner_uid: Option<u32>,
    owner_gid: Option<u32>,
}

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
