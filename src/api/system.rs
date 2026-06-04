use chrono::Local;
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

use crate::disk;
use crate::disk::{DiskAcquisitionControl, DiskAcquisitionTask};
use crate::disk_analysis;
use crate::evidence::EvidenceVault;
use crate::hash;
use crate::hash::HashAlgorithm;
use crate::ram;
use crate::remote::RemoteConnection;
use crate::server::{Response, json_error, json_ok};

use super::{
    ImageMountState,
    cleanup_helper_files,
    create_acquisition_job,
    current_image_mount,
    default_case_base_dir,
    elevated_disk_list,
    fail_acquisition_job_with_message,
    finish_acquisition_job_with_message,
    helper_file_stem,
    helper_owner_gid,
    helper_owner_uid,
    image_unmount_current,
    // Shared helpers
    process_is_root,
    read_helper_json,
    sanitize_case_name,
    sanitize_file_stem,
    set_current_evidence_case,
    sha256_file,
    spawn_elevated_helper,
    update_acquisition_message,
    update_acquisition_progress,
    update_acquisition_progress_message,
    write_helper_control_state,
    write_json_file,
};

#[cfg(target_os = "linux")]
use super::linux_mount_image_readonly;

#[derive(Deserialize)]
pub struct LocalImageRequest {
    pub source: String,
    pub disk_name: Option<String>,
    pub output: String,
    pub case_name: Option<String>,
}

#[derive(Deserialize)]
pub struct RemoteImageRequest {
    pub ip: String,
    pub port: u16,
    pub token: Option<String>,
    pub disk_id: String,
    pub disk_name: Option<String>,
    pub output: String,
    pub case_name: Option<String>,
}

#[derive(Deserialize)]
pub struct RemoteRequest {
    pub ip: String,
    pub port: u16,
    pub token: Option<String>,
}

pub fn connect_endpoint(body: &[u8]) -> Response {
    let request = match parse_remote_request(body) {
        Ok(request) => request,
        Err(response) => return response,
    };

    match RemoteConnection::connect(&request.ip, request.port, request.token) {
        Ok(connection) => json_ok(json!({
            "connected": true,
            "host": connection.host(),
            "port": connection.port(),
            "server_name": connection.server_name,
            "server_version": connection.server_version,
            "features": connection.features,
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

pub fn disk_list_endpoint() -> Response {
    match disk::list_disks() {
        Ok(disks) => {
            if should_request_elevated_disk_list(&disks) {
                match elevated_disk_list() {
                    Ok(elevated_disks) if !elevated_disks.is_empty() => {
                        json_ok(json!({ "disks": elevated_disks, "elevated": true }))
                    }
                    Ok(_) => json_ok(json!({ "disks": disks, "elevated": true })),
                    Err(err) => json_ok(json!({
                        "disks": disks,
                        "elevated": false,
                        "elevation_error": err,
                    })),
                }
            } else {
                json_ok(json!({ "disks": disks, "elevated": false }))
            }
        }
        Err(err) => match elevated_disk_list() {
            Ok(disks) => json_ok(json!({ "disks": disks, "elevated": true })),
            Err(elevation_err) => {
                json_error(500, format!("{}; elevation failed: {elevation_err}", err))
            }
        },
    }
}

fn should_request_elevated_disk_list(disks: &[disk::DiskInfo]) -> bool {
    #[cfg(target_os = "linux")]
    {
        if !process_is_root() {
            return true;
        }
    }

    if !(cfg!(target_os = "linux") || cfg!(windows)) {
        return false;
    }
    disks.is_empty() || disks.iter().any(|disk| !disk.accessible)
}

pub fn hash_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct HashRequest {
        path: String,
        algorithms: Option<Vec<String>>,
    }

    let request: HashRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    let algorithms = match parse_algorithms(request.algorithms) {
        Ok(algorithms) => algorithms,
        Err(message) => return json_error(400, message),
    };

    match hash::calculate_multiple(&request.path, &algorithms) {
        Ok(results) => {
            let mut value = serde_json::Map::new();
            for result in results {
                value.insert(
                    result.algorithm.name().to_ascii_lowercase(),
                    Value::String(result.value),
                );
            }
            json_ok(Value::Object(value))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

fn parse_algorithms(values: Option<Vec<String>>) -> Result<Vec<HashAlgorithm>, String> {
    let values = values.unwrap_or_else(|| {
        vec![
            "md5".to_string(),
            "sha1".to_string(),
            "sha256".to_string(),
            "sha512".to_string(),
        ]
    });

    let mut algorithms = Vec::new();
    for value in values {
        let algorithm = HashAlgorithm::parse(&value)
            .ok_or_else(|| format!("unsupported hash algorithm: {value}"))?;
        algorithms.push(algorithm);
    }
    Ok(algorithms)
}

pub fn local_image_endpoint(body: &[u8]) -> Response {
    let request: LocalImageRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    if request.source.trim().is_empty() {
        return json_error(400, "source is required");
    }
    if request.output.trim().is_empty()
        && request
            .case_name
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        return json_error(400, "output is required");
    }

    let (job_id, control) = create_acquisition_job("Yerel imaj alma başlatıldı");
    let thread_job_id = job_id.clone();
    thread::spawn(move || run_local_image_job(thread_job_id, request, control));

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

fn run_local_image_job(
    job_id: String,
    request: LocalImageRequest,
    control: ram::CancellationToken,
) {
    let output = match image_output_dir(&request.output, request.case_name.as_deref()) {
        Ok(output) => output,
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err, "Imaj alma basarisiz");
            return;
        }
    };
    let target = acquisition_target_path(
        &request.source,
        request.disk_name.as_deref(),
        &output.to_string_lossy(),
        None,
    );
    let task = DiskAcquisitionTask::new(&request.source, &target);

    if local_image_source_requires_elevation(&task.source) {
        run_elevated_local_image_job(&job_id, &task, &control);
        return;
    }

    match disk::run_disk_acquisition_with_control(
        &task,
        |done, total| {
            update_acquisition_progress_message(&job_id, done, total, "İmaj alma sürüyor");
        },
        || {
            if control.is_cancelled() {
                DiskAcquisitionControl::Cancel
            } else if control.is_paused() {
                DiskAcquisitionControl::Pause
            } else {
                DiskAcquisitionControl::Continue
            }
        },
    ) {
        Ok(result) => {
            finish_acquisition_job_with_message(
                &job_id,
                json!({
                    "message": "Imaj alma tamamlandi",
                    "target_path": result.target,
                    "bytes_copied": result.bytes_copied,
                    "total_bytes": result.total_bytes,
                    "sha256": result.sha256,
                }),
                "Imaj alma tamamlandi",
            );
        }
        Err(err) => {
            let message = err.to_string();
            if local_image_error_can_retry_elevated(&message) {
                run_elevated_local_image_job(&job_id, &task, &control);
            } else {
                fail_acquisition_job_with_message(&job_id, message, "Imaj alma basarisiz")
            }
        }
    }
}

fn run_elevated_local_image_job(
    job_id: &str,
    task: &DiskAcquisitionTask,
    control: &ram::CancellationToken,
) {
    update_acquisition_message(job_id, "Yetki bekleniyor");
    let stem = helper_file_stem("worm-image-helper");
    let request_path = std::env::temp_dir().join(format!("{stem}-request.json"));
    let result_path = std::env::temp_dir().join(format!("{stem}-result.json"));
    let progress_path = std::env::temp_dir().join(format!("{stem}-progress.json"));
    let control_path = std::env::temp_dir().join(format!("{stem}-control.json"));

    let request = json!({
        "source": task.source,
        "target": task.target,
        "owner_uid": helper_owner_uid(),
        "owner_gid": helper_owner_gid(),
    });
    if let Err(err) = write_json_file(&request_path, &request) {
        fail_acquisition_job_with_message(job_id, err, "Imaj alma basarisiz");
        return;
    }
    if let Err(err) = write_helper_control_state(&control_path, "running") {
        cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
        fail_acquisition_job_with_message(job_id, err, "Imaj alma basarisiz");
        return;
    }

    let args = vec![
        "image-helper".to_string(),
        request_path.to_string_lossy().into_owned(),
        result_path.to_string_lossy().into_owned(),
        progress_path.to_string_lossy().into_owned(),
        control_path.to_string_lossy().into_owned(),
    ];
    let mut child = match spawn_elevated_helper(&args) {
        Ok(child) => child,
        Err(err) => {
            cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
            fail_acquisition_job_with_message(job_id, err, "Imaj alma basarisiz");
            return;
        }
    };

    loop {
        if control.is_cancelled() {
            let _ = write_helper_control_state(&control_path, "cancelled");
            update_acquisition_message(job_id, "Imaj alma iptal ediliyor");
            let mut exited = false;
            for _ in 0..30 {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        exited = true;
                        break;
                    }
                    Ok(None) => thread::sleep(std::time::Duration::from_millis(100)),
                    Err(_) => break,
                }
            }
            if !exited {
                let _ = child.kill();
                let _ = child.wait();
            }
            cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
            fail_acquisition_job_with_message(
                job_id,
                "Imaj alma iptal edildi".to_string(),
                "Imaj alma basarisiz",
            );
            return;
        }
        if control.is_paused() {
            let _ = write_helper_control_state(&control_path, "paused");
            update_acquisition_message(job_id, "Imaj alma duraklatildi");
        } else {
            let _ = write_helper_control_state(&control_path, "running");
        }

        if let Some((done, total, message)) = super::read_helper_progress(&progress_path) {
            update_acquisition_progress_message(job_id, done, total, &message);
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let error = super::read_helper_error(&result_path).unwrap_or_else(|| {
                        "yetki yükseltme iptal edildi veya başarısız oldu".to_string()
                    });
                    cleanup_helper_files(&[
                        &request_path,
                        &result_path,
                        &progress_path,
                        &control_path,
                    ]);
                    fail_acquisition_job_with_message(job_id, error, "Imaj alma basarisiz");
                    return;
                }
                break;
            }
            Ok(None) => thread::sleep(std::time::Duration::from_millis(500)),
            Err(err) => {
                cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
                fail_acquisition_job_with_message(job_id, err.to_string(), "Imaj alma basarisiz");
                return;
            }
        }
    }

    let result = match read_helper_json(&result_path) {
        Ok(result) => result,
        Err(err) => {
            cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
            fail_acquisition_job_with_message(job_id, err, "Imaj alma basarisiz");
            return;
        }
    };
    cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);

    if result.get("ok").and_then(Value::as_bool) == Some(true) {
        finish_acquisition_job_with_message(
            job_id,
            json!({
                "message": "Imaj alma tamamlandi",
                "target_path": result.get("target_path").cloned().unwrap_or(Value::Null),
                "bytes_copied": result.get("bytes_copied").cloned().unwrap_or(Value::Null),
                "total_bytes": result.get("total_bytes").cloned().unwrap_or(Value::Null),
                "sha256": result.get("sha256").cloned().unwrap_or(Value::Null),
            }),
            "Imaj alma tamamlandi",
        );
    } else {
        fail_acquisition_job_with_message(
            job_id,
            result
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("Yetkili imaj alma basarisiz")
                .to_string(),
            "Imaj alma basarisiz",
        );
    }
}

pub fn remote_disks_endpoint(body: &[u8]) -> Response {
    let request = match parse_remote_request(body) {
        Ok(request) => request,
        Err(response) => return response,
    };

    match RemoteConnection::connect(&request.ip, request.port, request.token) {
        Ok(mut connection) => match connection.list_disks() {
            Ok(disks) => json_ok(json!({
                "server_name": connection.server_name,
                "server_version": connection.server_version,
                "features": connection.features,
                "disks": disks,
            })),
            Err(err) => json_error(500, err.to_string()),
        },
        Err(err) => json_error(500, err.to_string()),
    }
}

pub fn remote_image_endpoint(body: &[u8]) -> Response {
    let request: RemoteImageRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    if request.ip.trim().is_empty() {
        return json_error(400, "ip is required");
    }
    if request.port == 0 {
        return json_error(400, "port is required");
    }
    if request.disk_id.trim().is_empty() {
        return json_error(400, "disk_id is required");
    }
    if request.output.trim().is_empty()
        && request
            .case_name
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        return json_error(400, "output is required");
    }

    let (job_id, _control) = create_acquisition_job("Uzak imaj alma başlatıldı");
    let thread_job_id = job_id.clone();
    thread::spawn(move || run_remote_image_job(thread_job_id, request));

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

fn run_remote_image_job(job_id: String, request: RemoteImageRequest) {
    match RemoteConnection::connect(&request.ip, request.port, request.token) {
        Ok(mut connection) => {
            let remote_job_id = job_id.clone();
            let output = match image_output_dir(&request.output, request.case_name.as_deref()) {
                Ok(output) => output,
                Err(err) => {
                    fail_acquisition_job_with_message(&job_id, err, "Imaj alma basarisiz");
                    return;
                }
            };
            match connection.acquire_image(
                &request.disk_id,
                request.disk_name.as_deref(),
                &output,
                Some(&remote_job_id),
                |done, total| update_acquisition_progress(&job_id, done, total),
            ) {
                Ok(result) => {
                    let sha256 = match finalize_image_sha256(
                        &job_id,
                        &result.target_path,
                        result.sha256.clone(),
                    ) {
                        Ok(value) => value,
                        Err(err) => {
                            fail_acquisition_job_with_message(
                                &job_id,
                                err,
                                "Imaj hash olusturulamadi",
                            );
                            return;
                        }
                    };
                    finish_acquisition_job_with_message(
                        &job_id,
                        json!({
                            "message": result.message,
                            "remote_job_id": result.job_id,
                            "target_path": result.target_path,
                            "bytes_transferred": result.bytes_transferred,
                            "sha256": sha256,
                            "md5": result.md5,
                        }),
                        "Imaj alma tamamlandi",
                    );
                }
                Err(err) => fail_acquisition_job_with_message(
                    &job_id,
                    err.to_string(),
                    "Imaj alma basarisiz",
                ),
            }
        }
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err.to_string(), "Imaj alma basarisiz")
        }
    }
}

fn finalize_image_sha256(
    job_id: &str,
    target_path: &Path,
    existing_sha256: Option<String>,
) -> Result<String, String> {
    update_acquisition_message(job_id, "Imaj SHA256 olusturuluyor");
    let sha256 = match existing_sha256.filter(|value| !value.trim().is_empty()) {
        Some(value) => value,
        None => sha256_file(target_path)?,
    };
    hash::write_sha256_sidecar(target_path, &sha256).map_err(|err| err.to_string())?;
    Ok(sha256)
}

pub fn remote_tool_check_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct ToolRequest {
        ip: String,
        port: u16,
        token: Option<String>,
        tool: String,
    }

    let request: ToolRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    match RemoteConnection::connect(&request.ip, request.port, request.token) {
        Ok(mut connection) => {
            let status = match request.tool.as_str() {
                "winpmem" => connection.check_winpmem(),
                "avml" => connection.check_avml(),
                _ => return json_error(400, "tool must be winpmem or avml"),
            };
            match status {
                Ok(status) => json_ok(json!({ "status": status })),
                Err(err) => json_error(500, err.to_string()),
            }
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

fn parse_remote_request(body: &[u8]) -> Result<RemoteRequest, Response> {
    let request: RemoteRequest =
        serde_json::from_slice(body).map_err(|err| json_error(400, err.to_string()))?;
    if request.ip.trim().is_empty() {
        return Err(json_error(400, "ip is required"));
    }
    if request.port == 0 {
        return Err(json_error(400, "port is required"));
    }
    Ok(request)
}

pub fn image_mount_readonly_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct ImageMountRequest {
        path: String,
    }

    let request: ImageMountRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let image_path = PathBuf::from(request.path.trim());
    if image_path.as_os_str().is_empty() {
        return json_error(400, "path is required");
    }
    if !image_path.exists() {
        return json_error(404, "image file not found");
    }

    #[cfg(target_os = "linux")]
    {
        let _ = image_unmount_current();
        let mount_dir = std::env::temp_dir().join(format!(
            "worm-image-mount-{}",
            Local::now().format("%Y%m%d%H%M%S")
        ));
        if let Err(err) = fs::create_dir_all(&mount_dir) {
            return json_error(500, err.to_string());
        }

        match linux_mount_image_readonly(&image_path, &mount_dir) {
            Ok(loop_device) => {
                let tree = directory_tree_json(&mount_dir, 3, 400);
                let state = ImageMountState {
                    image_path: image_path.clone(),
                    mount_dir: mount_dir.clone(),
                    loop_device,
                };
                if let Ok(mut current) = current_image_mount().lock() {
                    *current = Some(state);
                }
                json_ok(json!({
                    "image_path": image_path,
                    "mount_dir": mount_dir,
                    "tree": tree,
                }))
            }
            Err(err) => {
                let _ = fs::remove_dir_all(&mount_dir);
                json_error(500, err)
            }
        }
    }

    #[cfg(windows)]
    {
        let _ = image_unmount_current();
        let output = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(
                "$ErrorActionPreference='Stop'; \
                 $image = $args[0]; \
                 $disk = Mount-DiskImage -ImagePath $image -Access ReadOnly -PassThru; \
                 Start-Sleep -Milliseconds 500; \
                 $volume = $disk | Get-Volume | Select-Object -First 1; \
                 if (-not $volume -or -not $volume.DriveLetter) { \
                   Dismount-DiskImage -ImagePath $image -ErrorAction SilentlyContinue; \
                   throw 'Mounted image has no drive letter. Windows supports ISO/VHD/VHDX here; raw DD/IMG needs a forensic image driver.' \
                 }; \
                 Write-Output ($volume.DriveLetter + ':\\')",
            )
            .arg(&image_path)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let mount_dir = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
                let tree = directory_tree_json(&mount_dir, 3, 400);
                let state = ImageMountState {
                    image_path: image_path.clone(),
                    mount_dir: mount_dir.clone(),
                };
                if let Ok(mut current) = current_image_mount().lock() {
                    *current = Some(state);
                }
                json_ok(json!({
                    "image_path": image_path,
                    "mount_dir": mount_dir,
                    "tree": tree,
                }))
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                json_error(
                    500,
                    if stderr.is_empty() {
                        "Windows image mount failed".to_string()
                    } else {
                        stderr
                    },
                )
            }
            Err(err) => json_error(500, err.to_string()),
        }
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        json_error(
            400,
            "read-only image mount is not supported on this platform",
        )
    }
}

pub fn image_unmount_endpoint() -> Response {
    match image_unmount_current() {
        Ok(Some(mount_dir)) => json_ok(json!({ "mount_dir": mount_dir })),
        Ok(None) => json_ok(json!({ "mount_dir": Value::Null })),
        Err(err) => json_error(500, err),
    }
}

pub fn image_analyze_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AnalyzeRequest {
        path: Option<String>,
    }

    let request: AnalyzeRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => return json_error(400, err.to_string()),
    };

    let current_mount = current_image_mount()
        .lock()
        .ok()
        .and_then(|state| state.clone());
    let image_path = request
        .path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| current_mount.as_ref().map(|state| state.image_path.clone()));

    let Some(image_path) = image_path else {
        return json_error(400, "İmaj yolu gerekli / Image path required");
    };
    if !image_path.exists() {
        return json_error(404, "İmaj dosyası bulunamadı / Image file not found");
    }

    let mount_dir = current_mount
        .as_ref()
        .filter(|state| state.image_path == image_path)
        .map(|state| state.mount_dir.as_path());

    match disk_analysis::analyze_disk_image(&image_path, mount_dir) {
        Ok(report) => json_ok(serde_json::to_value(report).unwrap_or(Value::Null)),
        Err(err) => json_error(500, err.to_string()),
    }
}

pub fn image_browse_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct BrowseRequest {
        path: Option<String>,
    }

    let request: BrowseRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => return json_error(400, err.to_string()),
    };

    let mount_dir = match current_image_mount().lock() {
        Ok(current) => match &*current {
            Some(state) => state.mount_dir.clone(),
            None => {
                return json_error(400, "Aktif bir imaj bağlantısı yok / No active image mount");
            }
        },
        Err(_) => return json_error(500, "Mutex lock hatası / Mutex lock error"),
    };

    let target_path = if let Some(sub) = request.path {
        let sub = sub.trim().replace("..", "");
        let clean_sub = sub.trim_start_matches('/');
        mount_dir.join(clean_sub)
    } else {
        mount_dir.clone()
    };

    if !target_path.starts_with(&mount_dir) {
        return json_error(403, "Yetkisiz erişim / Access denied");
    }

    if !target_path.exists() {
        return json_error(404, "Dizin bulunamadı / Directory not found");
    }

    let mut files = Vec::new();
    match fs::read_dir(&target_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let meta = entry.metadata().ok();
                let is_dir = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                let name = entry.file_name().to_string_lossy().into_owned();
                let rel_path = target_path
                    .join(&name)
                    .strip_prefix(&mount_dir)
                    .unwrap_or(&Path::new(""))
                    .to_string_lossy()
                    .into_owned();

                files.push(json!({
                    "name": name,
                    "relative_path": rel_path,
                    "is_dir": is_dir,
                    "size": size,
                }));
            }
            json_ok(json!({ "files": files }))
        }
        Err(err) => json_error(
            500,
            format!("Dizin okunamadı / Directory read failed: {}", err),
        ),
    }
}

pub fn image_read_file_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct ReadRequest {
        path: String,
    }

    let request: ReadRequest = match serde_json::from_slice(body) {
        Ok(req) => req,
        Err(err) => return json_error(400, err.to_string()),
    };

    let mount_dir = match current_image_mount().lock() {
        Ok(current) => match &*current {
            Some(state) => state.mount_dir.clone(),
            None => {
                return json_error(400, "Aktif bir imaj bağlantısı yok / No active image mount");
            }
        },
        Err(_) => return json_error(500, "Mutex lock hatası / Mutex lock error"),
    };

    let sub = request.path.trim().replace("..", "");
    let clean_sub = sub.trim_start_matches('/');
    let target_path = mount_dir.join(clean_sub);

    if !target_path.starts_with(&mount_dir) {
        return json_error(403, "Yetkisiz erişim / Access denied");
    }

    if !target_path.is_file() {
        return json_error(404, "Dosya bulunamadı / File not found");
    }

    let ext = target_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    let size = match fs::metadata(&target_path) {
        Ok(meta) => meta.len(),
        Err(err) => return json_error(500, err.to_string()),
    };

    if ["png", "jpg", "jpeg", "gif", "bmp", "webp"].contains(&ext.as_str()) {
        if size > 15 * 1024 * 1024 {
            return json_error(
                400,
                "Resim boyutu önizleme için çok büyük / Image size too large for preview",
            );
        }
        match fs::read(&target_path) {
            Ok(bytes) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
                let mime = match ext.as_str() {
                    "png" => "image/png",
                    "jpg" | "jpeg" => "image/jpeg",
                    "gif" => "image/gif",
                    "webp" => "image/webp",
                    _ => "image/png",
                };
                return json_ok(json!({
                    "type": "image",
                    "mime": mime,
                    "content": format!("data:{};base64,{}", mime, encoded),
                    "size": size,
                }));
            }
            Err(err) => return json_error(500, err.to_string()),
        }
    }

    let is_text_ext = [
        "txt", "log", "json", "xml", "plist", "html", "css", "js", "sh", "prop", "rc", "conf",
        "ini",
    ]
    .contains(&ext.as_str())
        || size < 200_000;

    match fs::File::open(&target_path) {
        Ok(mut f) => {
            let mut buf = vec![0_u8; 16384.min(size as usize)];
            let read = f.read(&mut buf).unwrap_or(0);
            let content_bytes = &buf[..read];

            if is_text_ext {
                if let Ok(text) = std::str::from_utf8(content_bytes) {
                    return json_ok(json!({
                        "type": "text",
                        "content": text,
                        "size": size,
                        "truncated": size > 16384,
                    }));
                }
            }

            let mut hex_lines = Vec::new();
            for chunk in content_bytes.chunks(16) {
                let offset = (hex_lines.len() * 16) as u64;
                let hex_parts: Vec<String> = chunk.iter().map(|b| format!("{:02X}", b)).collect();
                let hex_str = hex_parts.join(" ");
                let ascii_str: String = chunk
                    .iter()
                    .map(|&b| {
                        if b.is_ascii_graphic() || b == b' ' {
                            b as char
                        } else {
                            '.'
                        }
                    })
                    .collect();
                hex_lines.push(format!("{:08X}  {:48}  |{}|", offset, hex_str, ascii_str));
            }
            json_ok(json!({
                "type": "hex",
                "content": hex_lines.join("\n"),
                "size": size,
                "truncated": size > 16384,
            }))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

pub fn update_check_endpoint() -> Response {
    let output = Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--silent")
        .arg("--show-error")
        .arg("https://api.github.com/repos/noirlang/worm/releases/latest")
        .output();
    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return json_error(
                500,
                if stderr.is_empty() {
                    "release check failed".to_string()
                } else {
                    stderr
                },
            );
        }
        Err(err) => return json_error(500, err.to_string()),
    };

    let release: Value = match serde_json::from_slice(&output.stdout) {
        Ok(release) => release,
        Err(err) => return json_error(500, err.to_string()),
    };
    let assets = release
        .get("assets")
        .and_then(Value::as_array)
        .map(|assets| {
            assets
                .iter()
                .map(|asset| {
                    json!({
                        "name": asset.get("name").and_then(Value::as_str).unwrap_or_default(),
                        "download_url": asset.get("browser_download_url").and_then(Value::as_str).unwrap_or_default(),
                        "size": asset.get("size").and_then(Value::as_u64).unwrap_or_default(),
                        "digest": asset.get("digest").and_then(Value::as_str).unwrap_or_default(),
                    })
                })
                .collect::<Vec<Value>>()
        })
        .unwrap_or_default();
    let platform_asset = preferred_update_asset(&assets);

    json_ok(json!({
        "current_version": env!("CARGO_PKG_VERSION"),
        "tag_name": release.get("tag_name").and_then(Value::as_str).unwrap_or_default(),
        "name": release.get("name").and_then(Value::as_str).unwrap_or_default(),
        "html_url": release.get("html_url").and_then(Value::as_str).unwrap_or_default(),
        "body": release.get("body").and_then(Value::as_str).unwrap_or_default(),
        "assets": assets,
        "platform_asset": platform_asset,
    }))
}

pub fn update_download_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct UpdateDownloadRequest {
        url: String,
        name: Option<String>,
        output_dir: Option<String>,
        expected_sha256: Option<String>,
    }

    let request: UpdateDownloadRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let url = request.url.trim();
    if url.is_empty() {
        return json_error(400, "url is required");
    }
    let output_dir = request
        .output_dir
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(default_download_dir);
    if let Err(err) = fs::create_dir_all(&output_dir) {
        return json_error(500, err.to_string());
    }
    let name = request
        .name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_download_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "worm-update.bin".to_string());
    let target = output_dir.join(name);
    let output = Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--silent")
        .arg("--show-error")
        .arg("-o")
        .arg(&target)
        .arg(url)
        .output();

    match output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let _ = fs::remove_file(&target);
            return json_error(
                500,
                if stderr.is_empty() {
                    "download failed".to_string()
                } else {
                    stderr
                },
            );
        }
        Err(err) => return json_error(500, err.to_string()),
    }

    let sha256 = match sha256_file(&target) {
        Ok(value) => value,
        Err(err) => return json_error(500, err),
    };
    if let Some(expected) = request.expected_sha256 {
        let expected = expected
            .trim()
            .strip_prefix("sha256:")
            .unwrap_or_else(|| expected.trim())
            .to_ascii_lowercase();
        if !expected.is_empty() && expected != sha256 {
            let _ = fs::remove_file(&target);
            return json_error(500, "downloaded file sha256 mismatch");
        }
    }
    let size = fs::metadata(&target)
        .map(|meta| meta.len())
        .unwrap_or_default();

    json_ok(json!({
        "path": target,
        "size": size,
        "sha256": sha256,
    }))
}

pub fn update_install_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct UpdateInstallRequest {
        path: String,
    }

    let request: UpdateInstallRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let path = PathBuf::from(request.path.trim());
    if path.as_os_str().is_empty() {
        return json_error(400, "path is required");
    }
    if !path.is_file() {
        return json_error(404, "update package not found");
    }

    match launch_update_installer(&path) {
        Ok(message) => json_ok(json!({ "path": path, "message": message })),
        Err(err) => json_error(500, err),
    }
}

pub fn open_url_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct OpenUrlRequest {
        url: String,
    }

    let request: OpenUrlRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    let url = match validate_external_url(&request.url) {
        Ok(url) => url,
        Err(err) => return json_error(400, err),
    };

    match open_external_url(&url) {
        Ok(()) => json_ok(json!({ "opened": true })),
        Err(err) => json_error(500, err),
    }
}

pub fn pick_path_endpoint(directory: bool) -> Response {
    match pick_path(directory) {
        Ok(Some(path)) => json_ok(json!({ "path": path })),
        Ok(None) => json_error(499, "selection cancelled"),
        Err(err) => json_error(500, err),
    }
}

fn pick_path(directory: bool) -> Result<Option<String>, String> {
    let candidates: &[(&str, &[&str])] = if directory {
        &[
            ("zenity", &["--file-selection", "--directory"]),
            ("kdialog", &["--getexistingdirectory"]),
        ]
    } else {
        &[
            ("zenity", &["--file-selection"]),
            ("kdialog", &["--getopenfilename"]),
        ]
    };

    let mut last_error = String::new();
    for (program, args) in candidates {
        match Command::new(program).args(*args).output() {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if path.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(path));
            }
            Ok(output) => {
                if output.status.code() == Some(1) {
                    return Ok(None);
                }
                last_error = String::from_utf8_lossy(&output.stderr).trim().to_string();
            }
            Err(err) => last_error = err.to_string(),
        }
    }

    Err(if last_error.is_empty() {
        "no file picker command found".to_string()
    } else {
        last_error
    })
}

fn open_external_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let openers: &[(&str, &[&str])] = &[("xdg-open", &[url]), ("gio", &["open", url])];
        for (program, args) in openers {
            if Command::new(program)
                .args(*args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }
        Err("external link opener could not be started".to_string())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("rundll32")
            .arg("url.dll,FileProtocolHandler")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|err| format!("external link opener could not be started: {err}"))
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|err| format!("external link opener could not be started: {err}"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = url;
        Err("external links are not supported on this platform".to_string())
    }
}

fn validate_external_url(value: &str) -> Result<String, String> {
    let url = value.trim();
    if url.is_empty() {
        return Err("url is required".to_string());
    }
    if url.chars().any(char::is_control) {
        return Err("url contains invalid characters".to_string());
    }

    let lower = url.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:")
    {
        Ok(url.to_string())
    } else {
        Err("only http, https and mailto links can be opened".to_string())
    }
}

fn directory_tree_json(root: &Path, max_depth: usize, max_entries: usize) -> Value {
    let mut used = 0_usize;
    directory_tree_json_inner(root, root, 0, max_depth, max_entries, &mut used)
}

fn directory_tree_json_inner(
    root: &Path,
    path: &Path,
    depth: usize,
    max_depth: usize,
    max_entries: usize,
    used: &mut usize,
) -> Value {
    *used += 1;
    let metadata = fs::metadata(path).ok();
    let is_dir = metadata.as_ref().map(|meta| meta.is_dir()).unwrap_or(false);
    let mut node = serde_json::Map::new();
    let display_name = if path == root {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("/")
            .to_string()
    } else {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string()
    };
    node.insert("name".to_string(), Value::String(display_name));
    node.insert(
        "path".to_string(),
        Value::String(path.to_string_lossy().to_string()),
    );
    node.insert("is_dir".to_string(), Value::Bool(is_dir));
    node.insert(
        "size".to_string(),
        Value::Number(metadata.map(|meta| meta.len()).unwrap_or_default().into()),
    );

    if is_dir && depth < max_depth && *used < max_entries {
        let mut children = Vec::new();
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten().take(max_entries.saturating_sub(*used)) {
                if *used >= max_entries {
                    break;
                }
                children.push(directory_tree_json_inner(
                    root,
                    &entry.path(),
                    depth + 1,
                    max_depth,
                    max_entries,
                    used,
                ));
            }
        }
        node.insert("children".to_string(), Value::Array(children));
    }

    Value::Object(node)
}

fn preferred_update_asset(assets: &[Value]) -> Value {
    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &["windows", ".msi", ".exe"]
    } else if cfg!(target_os = "linux") {
        &["linux", "appimage", ".tar.gz"]
    } else {
        &[]
    };

    assets
        .iter()
        .find(|asset| {
            let name = asset
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_lowercase();
            candidates
                .iter()
                .any(|candidate| name.contains(&candidate.to_ascii_lowercase()))
        })
        .cloned()
        .or_else(|| assets.first().cloned())
        .unwrap_or(Value::Null)
}

pub fn sanitize_download_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn launch_update_installer(path: &Path) -> Result<String, String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    #[cfg(windows)]
    {
        let mut command = if extension == "msi" {
            let mut command = Command::new("msiexec");
            command.arg("/i").arg(path).arg("/passive");
            command
        } else {
            Command::new(path)
        };
        command
            .spawn()
            .map_err(|err| format!("installer could not be started: {err}"))?;
        return Ok("installer started".to_string());
    }

    #[cfg(unix)]
    {
        if extension == "appimage" {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(path)
                .map_err(|err| err.to_string())?
                .permissions();
            permissions.set_mode(permissions.mode() | 0o755);
            fs::set_permissions(path, permissions).map_err(|err| err.to_string())?;
        }
        Command::new(path)
            .spawn()
            .map_err(|err| format!("installer could not be started: {err}"))?;
        Ok("installer started".to_string())
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = extension;
        Err("automatic update install is not supported on this platform".to_string())
    }
}

fn default_download_dir() -> PathBuf {
    super::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Downloads")
}

fn image_output_dir(output: &str, case_name: Option<&str>) -> Result<PathBuf, String> {
    let case_name = case_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_case_name)
        .filter(|value| !value.is_empty());

    if let Some(case_name) = case_name {
        let base_dir = default_case_base_dir();
        let vault = EvidenceVault::create(&base_dir, &case_name).map_err(|err| err.to_string())?;
        set_current_evidence_case(base_dir, case_name);
        return Ok(vault.outputs_dir);
    }

    let output = output.trim();
    if output.is_empty() {
        Err("output is required".to_string())
    } else {
        Ok(PathBuf::from(output))
    }
}

fn acquisition_target_path(
    source: &str,
    disk_name: Option<&str>,
    output: &str,
    remote_ip: Option<&str>,
) -> PathBuf {
    let output = PathBuf::from(output);
    if output
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| matches!(extension, "dd" | "img" | "raw" | "001"))
        .unwrap_or(false)
    {
        return output;
    }

    let source_name = source
        .rsplit(['/', '\\'])
        .find(|part| !part.is_empty())
        .unwrap_or("disk");
    let file_name = canonical_image_file_name(remote_ip, source_name, disk_name);
    output.join(file_name)
}

fn canonical_image_file_name(
    remote_ip: Option<&str>,
    disk_id: &str,
    disk_name: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    if let Some(ip) = remote_ip
        .map(sanitize_file_stem)
        .filter(|value| !value.is_empty())
    {
        parts.push(ip);
    }

    let disk_id = sanitize_file_stem(disk_id);
    parts.push(if disk_id.is_empty() {
        "disk".to_string()
    } else {
        disk_id
    });

    if let Some(name) = disk_name
        .map(sanitize_file_stem)
        .filter(|value| !value.is_empty())
        && parts.last().map(|last| last != &name).unwrap_or(true)
    {
        parts.push(name);
    }

    format!(
        "{}_{}.img",
        parts.join("_"),
        Local::now().format("%Y%m%d_%H%M%S")
    )
}

fn local_image_source_requires_elevation(source: &Path) -> bool {
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::FileTypeExt;
        fs::metadata(source)
            .map(|metadata| metadata.file_type().is_block_device() && !process_is_root())
            .unwrap_or(false)
    }

    #[cfg(windows)]
    {
        return source
            .to_string_lossy()
            .to_ascii_lowercase()
            .starts_with(r"\\.\physicaldrive");
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = source;
        false
    }
}

fn local_image_error_can_retry_elevated(message: &str) -> bool {
    if !(cfg!(target_os = "linux") || cfg!(windows)) {
        return false;
    }
    let message = message.to_ascii_lowercase();
    message.contains("permission denied")
        || message.contains("access is denied")
        || message.contains("erişim engellendi")
        || message.contains("os error 13")
}
