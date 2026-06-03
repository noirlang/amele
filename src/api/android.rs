use crate::android;
use crate::android_analysis;
use crate::api::{
    create_acquisition_job, evidence_vault_for_output, fail_acquisition_job_with_message,
    finish_acquisition_job_with_message, report_evidence_vault,
    update_acquisition_progress_message,
};
use crate::ram;
use crate::server::{Response, json_error, json_ok};
use serde::Deserialize;
use serde_json::json;
use std::thread;

pub fn android_case_analysis_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AndroidAnalysisRequest {
        case_name: Option<String>,
    }

    let request: AndroidAnalysisRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let vault = match report_evidence_vault(request.case_name.as_deref()) {
        Ok(vault) => vault,
        Err(response) => return response,
    };
    let report = android_analysis::analyze_android_case(&vault.case_name, &vault.android_dir);
    json_ok(serde_json::to_value(report).unwrap_or(serde_json::Value::Null))
}

pub fn android_device_profile_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AndroidProfileRequest {
        serial: String,
    }

    let request: AndroidProfileRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let serial = request.serial.trim();
    if serial.is_empty() {
        return json_error(400, "serial is required");
    }

    match android::detect_device_profile(serial) {
        Ok(profile) => json_ok(json!({ "profile": profile })),
        Err(err) => json_error(500, android::explain_android_error(err)),
    }
}

pub fn android_profile_acquisition_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AndroidProfileAcquisitionRequest {
        serial: String,
        case_name: Option<String>,
        profile: Option<String>,
    }

    let request: AndroidProfileAcquisitionRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let serial = request.serial.trim().to_string();
    if serial.is_empty() {
        return json_error(400, "serial is required");
    }
    let profile = request
        .profile
        .as_deref()
        .map(android::AndroidAcquisitionProfile::from_id)
        .unwrap_or(android::AndroidAcquisitionProfile::FullLogical);

    let (job_id, control) = create_acquisition_job("Android profil edinimi baslatildi");
    let thread_job_id = job_id.clone();
    thread::spawn(move || {
        run_android_profile_acquisition_job(
            thread_job_id,
            serial,
            request.case_name,
            profile,
            control,
        )
    });

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

fn run_android_profile_acquisition_job(
    job_id: String,
    serial: String,
    case_name: Option<String>,
    profile: android::AndroidAcquisitionProfile,
    control: ram::CancellationToken,
) {
    let vault = match evidence_vault_for_output(case_name.as_deref()) {
        Ok(vault) => vault,
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err, "Android profil edinimi basarisiz");
            return;
        }
    };

    let android_dir = vault.case_dir.join("android");
    if let Err(err) = std::fs::create_dir_all(&android_dir) {
        fail_acquisition_job_with_message(
            &job_id,
            err.to_string(),
            "Android profil edinimi basarisiz",
        );
        return;
    }

    match android::orchestrated_acquisition(
        &serial,
        &android_dir,
        profile,
        |done, total, category| {
            update_acquisition_progress_message(
                &job_id,
                done as u64,
                total as u64,
                &format!("Toplaniyor: {category}"),
            );
        },
        || android_job_should_stop(&control),
    ) {
        Ok(result) => {
            let success_count = result.items.iter().filter(|i| i.success).count();
            let total_count = result.items.len();
            finish_acquisition_job_with_message(
                &job_id,
                json!({
                    "message": format!("Android profil edinimi tamamlandi ({success_count}/{total_count} adim basarili)"),
                    "profile": result.profile,
                    "device_profile": result.device_profile,
                    "output_dir": result.output_dir,
                    "total_bytes": result.total_bytes,
                    "sha256": result.sha256,
                    "items": result.items,
                    "errors": result.errors,
                }),
                "Android profil edinimi tamamlandi",
            );
        }
        Err(err) => {
            fail_android_job(&job_id, err, "Android profil edinimi basarisiz");
        }
    }
}

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
        || android_job_should_stop(&control),
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
            fail_android_job(&job_id, err, "Android imaj alma basarisiz");
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

    match android::orchestrated_filesystem_acquisition(
        &serial,
        &android_dir,
        has_root,
        |done, total, category| {
            update_acquisition_progress_message(&job_id, done as u64, total as u64, category);
        },
        || android_job_should_stop(&control),
    ) {
        Ok(result) => {
            finish_acquisition_job_with_message(
                &job_id,
                json!({
                    "message": "Android dosya sistemi imajı başarıyla tamamlandı",
                    "device_profile": result.device_profile,
                    "output_file": result.output_file,
                    "total_bytes": result.total_bytes,
                    "sha256": result.sha256,
                }),
                "Android dosya sistemi imajı başarıyla tamamlandı",
            );
        }
        Err(err) => {
            fail_android_job(&job_id, err, "Android dosya sistemi imaj alma basarisiz");
        }
    }
}

pub fn android_ram_image_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AndroidRamRequest {
        serial: String,
        case_name: Option<String>,
        has_root: Option<bool>,
        mode: Option<String>,
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
    let mode = request
        .mode
        .as_deref()
        .map(android::AndroidRamMode::from_id)
        .unwrap_or(android::AndroidRamMode::VolatileData);
    thread::spawn(move || {
        run_android_ram_job(
            thread_job_id,
            serial,
            request.case_name,
            has_root,
            mode,
            control,
        )
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
    mode: android::AndroidRamMode,
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

    match android::orchestrated_ram_acquisition(
        &serial,
        &android_dir,
        has_root,
        mode,
        |done, total, category| {
            update_acquisition_progress_message(&job_id, done as u64, total as u64, category);
        },
        || android_job_should_stop(&control),
    ) {
        Ok(result) => {
            finish_acquisition_job_with_message(
                &job_id,
                json!({
                    "message": "Android RAM imajı başarıyla tamamlandı",
                    "device_profile": result.device_profile,
                    "output_file": result.output_file,
                    "total_bytes": result.total_bytes,
                    "sha256": result.sha256,
                    "mode": result.mode,
                }),
                "Android RAM imajı başarıyla tamamlandı",
            );
        }
        Err(err) => {
            fail_android_job(&job_id, err, "Android RAM imaj alma basarisiz");
        }
    }
}

fn fail_android_job(job_id: &str, err: String, title: &str) {
    fail_acquisition_job_with_message(job_id, android::explain_android_error(err), title);
}

fn android_job_should_stop(control: &ram::CancellationToken) -> bool {
    while control.is_paused() {
        if control.is_cancelled() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    control.is_cancelled()
}
