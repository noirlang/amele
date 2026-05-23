use crate::android;
use crate::api::{
    create_acquisition_job, evidence_vault_for_output, fail_acquisition_job_with_message,
    finish_acquisition_job_with_message, update_acquisition_progress_message,
};
use crate::ram;
use crate::server::{Response, json_error, json_ok};
use serde::Deserialize;
use serde_json::json;
use std::thread;

pub fn android_logical_image_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AndroidLogicalRequest {
        serial: String,
        case_name: Option<String>,
    }

    let request: AndroidLogicalRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let serial = request.serial.trim().to_string();
    if serial.is_empty() {
        return json_error(400, "serial is required");
    }

    let (job_id, control) = create_acquisition_job("Android mantiksal imaj alma baslatildi");
    let thread_job_id = job_id.clone();
    thread::spawn(move || {
        run_android_logical_job(thread_job_id, serial, request.case_name, control)
    });

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

fn run_android_logical_job(
    job_id: String,
    serial: String,
    case_name: Option<String>,
    control: ram::CancellationToken,
) {
    let vault = match evidence_vault_for_output(case_name.as_deref()) {
        Ok(vault) => vault,
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err, "Android imaj alma basarisiz");
            return;
        }
    };

    let android_dir = vault.case_dir.join("android");
    if let Err(err) = std::fs::create_dir_all(&android_dir) {
        fail_acquisition_job_with_message(&job_id, err.to_string(), "Android imaj alma basarisiz");
        return;
    }

    match android::logical_acquisition(
        &serial,
        &android_dir,
        |done, total, category| {
            update_acquisition_progress_message(
                &job_id,
                done as u64,
                total as u64,
                &format!("Toplaniyor: {category}"),
            );
        },
        || control.is_cancelled(),
    ) {
        Ok(result) => {
            let success_count = result.items.iter().filter(|i| i.success).count();
            let total_count = result.items.len();
            finish_acquisition_job_with_message(
                &job_id,
                json!({
                    "message": format!("Android mantiksal imaj tamamlandi ({success_count}/{total_count} adim basarili)"),
                    "output_dir": result.output_dir,
                    "total_bytes": result.total_bytes,
                    "sha256": result.sha256,
                    "items": result.items,
                    "errors": result.errors,
                }),
                "Android mantiksal imaj tamamlandi",
            );
        }
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err, "Android imaj alma basarisiz");
        }
    }
}

pub fn android_filesystem_image_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AndroidFilesystemRequest {
        serial: String,
        case_name: Option<String>,
        has_root: Option<bool>,
    }

    let request: AndroidFilesystemRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let serial = request.serial.trim().to_string();
    if serial.is_empty() {
        return json_error(400, "serial is required");
    }

    let (job_id, control) = create_acquisition_job("Android dosya sistemi imaj alma baslatildi");
    let thread_job_id = job_id.clone();
    let has_root = request.has_root.unwrap_or(false);
    thread::spawn(move || {
        run_android_filesystem_job(thread_job_id, serial, request.case_name, has_root, control)
    });

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

fn run_android_filesystem_job(
    job_id: String,
    serial: String,
    case_name: Option<String>,
    has_root: bool,
    control: ram::CancellationToken,
) {
    let vault = match evidence_vault_for_output(case_name.as_deref()) {
        Ok(vault) => vault,
        Err(err) => {
            fail_acquisition_job_with_message(
                &job_id,
                err,
                "Android dosya sistemi imaj alma basarisiz",
            );
            return;
        }
    };

    let android_dir = vault.case_dir.join("android");
    if let Err(err) = std::fs::create_dir_all(&android_dir) {
        fail_acquisition_job_with_message(
            &job_id,
            err.to_string(),
            "Android dosya sistemi imaj alma basarisiz",
        );
        return;
    }

    match android::filesystem_acquisition(
        &serial,
        &android_dir,
        has_root,
        |done, total, category| {
            update_acquisition_progress_message(&job_id, done as u64, total as u64, category);
        },
        || control.is_cancelled(),
    ) {
        Ok(result) => {
            finish_acquisition_job_with_message(
                &job_id,
                json!({
                    "message": "Android dosya sistemi imajı başarıyla tamamlandı",
                    "output_file": result.output_file,
                    "total_bytes": result.total_bytes,
                    "sha256": result.sha256,
                }),
                "Android dosya sistemi imajı başarıyla tamamlandı",
            );
        }
        Err(err) => {
            fail_acquisition_job_with_message(
                &job_id,
                err,
                "Android dosya sistemi imaj alma basarisiz",
            );
        }
    }
}

pub fn android_ram_image_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AndroidRamRequest {
        serial: String,
        case_name: Option<String>,
        has_root: Option<bool>,
    }

    let request: AndroidRamRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let serial = request.serial.trim().to_string();
    if serial.is_empty() {
        return json_error(400, "serial is required");
    }

    let (job_id, control) = create_acquisition_job("Android RAM imaj alma baslatildi");
    let thread_job_id = job_id.clone();
    let has_root = request.has_root.unwrap_or(false);
    thread::spawn(move || {
        run_android_ram_job(thread_job_id, serial, request.case_name, has_root, control)
    });

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

fn run_android_ram_job(
    job_id: String,
    serial: String,
    case_name: Option<String>,
    has_root: bool,
    control: ram::CancellationToken,
) {
    let vault = match evidence_vault_for_output(case_name.as_deref()) {
        Ok(vault) => vault,
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err, "Android RAM imaj alma basarisiz");
            return;
        }
    };

    let android_dir = vault.case_dir.join("android");
    if let Err(err) = std::fs::create_dir_all(&android_dir) {
        fail_acquisition_job_with_message(
            &job_id,
            err.to_string(),
            "Android RAM imaj alma basarisiz",
        );
        return;
    }

    match android::ram_acquisition(
        &serial,
        &android_dir,
        has_root,
        |done, total, category| {
            update_acquisition_progress_message(&job_id, done as u64, total as u64, category);
        },
        || control.is_cancelled(),
    ) {
        Ok(result) => {
            finish_acquisition_job_with_message(
                &job_id,
                json!({
                    "message": "Android RAM imajı başarıyla tamamlandı",
                    "output_file": result.output_file,
                    "total_bytes": result.total_bytes,
                    "sha256": result.sha256,
                }),
                "Android RAM imajı başarıyla tamamlandı",
            );
        }
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err, "Android RAM imaj alma basarisiz");
        }
    }
}
