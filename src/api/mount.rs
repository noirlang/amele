use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

use super::{
    cleanup_helper_files, command_error_message, current_image_mount, helper_file_stem,
    process_is_root, read_helper_json, run_elevated_helper_wait, write_json_file,
};

pub fn image_unmount_current() -> Result<Option<PathBuf>, String> {
    let state = current_image_mount()
        .lock()
        .ok()
        .and_then(|mut current| current.take());
    let Some(state) = state else {
        return Ok(None);
    };

    #[cfg(target_os = "linux")]
    {
        if !process_is_root() {
            elevated_linux_unmount_image(&state.mount_dir, state.loop_device.as_deref())?;
        } else {
            let output = Command::new("umount").arg(&state.mount_dir).output();
            match output {
                Ok(output) if output.status.success() => {}
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    return Err(if stderr.is_empty() {
                        "unmount failed".to_string()
                    } else {
                        stderr
                    });
                }
                Err(err) => return Err(err.to_string()),
            }

            if let Some(loop_device) = &state.loop_device {
                let output = Command::new("losetup").arg("-d").arg(loop_device).output();
                match output {
                    Ok(output) if output.status.success() => {}
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                        return Err(if stderr.is_empty() {
                            "loop device detach failed".to_string()
                        } else {
                            stderr
                        });
                    }
                    Err(err) => return Err(err.to_string()),
                }
            }
        }
    }

    #[cfg(windows)]
    {
        let output = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(
                "$ErrorActionPreference='Stop'; \
                 Dismount-DiskImage -ImagePath $args[0]",
            )
            .arg(&state.image_path)
            .output();
        match output {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(if stderr.is_empty() {
                    "Windows image unmount failed".to_string()
                } else {
                    stderr
                });
            }
            Err(err) => return Err(err.to_string()),
        }
    }

    #[cfg(target_os = "linux")]
    let _ = fs::remove_dir_all(&state.mount_dir);
    Ok(Some(state.mount_dir))
}

#[cfg(target_os = "linux")]
pub fn elevated_linux_mount_image_readonly(
    image_path: &Path,
    mount_dir: &Path,
    initial_error: &str,
) -> Result<Option<PathBuf>, String> {
    let stem = helper_file_stem("worm-mount-helper");
    let request_path = std::env::temp_dir().join(format!("{stem}-request.json"));
    let result_path = std::env::temp_dir().join(format!("{stem}-result.json"));
    write_json_file(
        &request_path,
        &json!({
            "action": "mount",
            "image_path": image_path,
            "mount_dir": mount_dir,
        }),
    )?;

    let args = vec![
        "mount-helper".to_string(),
        request_path.to_string_lossy().into_owned(),
        result_path.to_string_lossy().into_owned(),
    ];
    let run_result = run_elevated_helper_wait(&args);
    if let Err(err) = run_result {
        cleanup_helper_files(&[&request_path, &result_path]);
        return Err(format!("{initial_error}\nyetki yükseltme başarısız: {err}"));
    }

    let result = read_helper_json(&result_path);
    cleanup_helper_files(&[&request_path, &result_path]);
    let result = result?;
    if result.get("ok").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err(format!(
            "{initial_error}\nyetkili mount başarısız: {}",
            result
                .get("error")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("bilinmeyen hata")
        ));
    }

    Ok(result
        .get("loop_device")
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from))
}

#[cfg(target_os = "linux")]
pub fn elevated_linux_unmount_image(
    mount_dir: &Path,
    loop_device: Option<&Path>,
) -> Result<(), String> {
    let stem = helper_file_stem("worm-unmount-helper");
    let request_path = std::env::temp_dir().join(format!("{stem}-request.json"));
    let result_path = std::env::temp_dir().join(format!("{stem}-result.json"));
    write_json_file(
        &request_path,
        &json!({
            "action": "unmount",
            "mount_dir": mount_dir,
            "loop_device": loop_device,
        }),
    )?;

    let args = vec![
        "mount-helper".to_string(),
        request_path.to_string_lossy().into_owned(),
        result_path.to_string_lossy().into_owned(),
    ];
    let run_result = run_elevated_helper_wait(&args);
    if let Err(err) = run_result {
        cleanup_helper_files(&[&request_path, &result_path]);
        return Err(format!("yetki yükseltme başarısız: {err}"));
    }

    let result = read_helper_json(&result_path);
    cleanup_helper_files(&[&request_path, &result_path]);
    let result = result?;
    if result.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err(result
            .get("error")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("yetkili unmount başarısız")
            .to_string())
    }
}

#[cfg(target_os = "linux")]
pub fn linux_mount_partitioned_image(
    image_path: &Path,
    mount_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    let setup_output = Command::new("losetup")
        .arg("--find")
        .arg("--partscan")
        .arg("--read-only")
        .arg("--show")
        .arg(image_path)
        .output()
        .map_err(|err| err.to_string())?;

    if !setup_output.status.success() {
        return Err(command_error_message(
            &setup_output,
            "losetup failed; root privileges may be required",
        ));
    }

    let loop_device = PathBuf::from(String::from_utf8_lossy(&setup_output.stdout).trim());
    if loop_device.as_os_str().is_empty() {
        return Err("losetup did not return a loop device".to_string());
    }

    thread::sleep(std::time::Duration::from_millis(250));

    let candidates = linux_loop_mount_candidates(&loop_device);
    let mut last_error = String::new();
    for candidate in candidates {
        let output = Command::new("mount")
            .arg("-o")
            .arg("ro")
            .arg(&candidate)
            .arg(mount_dir)
            .output();
        match output {
            Ok(output) if output.status.success() => return Ok(Some(loop_device)),
            Ok(output) => {
                last_error = format!(
                    "{}: {}",
                    candidate.display(),
                    command_error_message(&output, "mount failed")
                );
            }
            Err(err) => {
                last_error = format!("{}: {err}", candidate.display());
            }
        }
    }

    let _ = Command::new("losetup").arg("-d").arg(&loop_device).output();
    if last_error.is_empty() {
        Err("no mountable filesystem partition was found in the image".to_string())
    } else {
        Err(last_error)
    }
}

#[cfg(target_os = "linux")]
pub fn linux_loop_mount_candidates(loop_device: &Path) -> Vec<PathBuf> {
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
pub fn linux_mount_image_readonly(
    image_path: &Path,
    mount_dir: &Path,
) -> Result<Option<PathBuf>, String> {
    let direct_output = Command::new("mount")
        .arg("-o")
        .arg("ro,loop")
        .arg(image_path)
        .arg(mount_dir)
        .output();

    match direct_output {
        Ok(output) if output.status.success() => Ok(None),
        Ok(output) => {
            let direct_error = command_error_message(
                &output,
                "mount failed; image may contain a partition table or root privileges may be required",
            );
            if !process_is_root() {
                return elevated_linux_mount_image_readonly(image_path, mount_dir, &direct_error);
            }
            linux_mount_partitioned_image(image_path, mount_dir)
                .map_err(|err| format!("{direct_error}\npartition scan failed: {err}"))
        }
        Err(err) => {
            if !process_is_root() {
                return elevated_linux_mount_image_readonly(
                    image_path,
                    mount_dir,
                    &err.to_string(),
                );
            }
            linux_mount_partitioned_image(image_path, mount_dir)
                .map_err(|scan_err| format!("{err}; partition scan failed: {scan_err}"))
        }
    }
}
