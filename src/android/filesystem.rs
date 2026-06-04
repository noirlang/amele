use super::adb::run_adb_command;
use serde::Serialize;
use std::process::Command;

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
