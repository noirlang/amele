pub mod android;
pub mod evidence;
pub mod ram;
pub mod system;
pub mod wireguard;

use crate::server::{Response, json_error, json_ok};
use chrono::Local;
use serde_json::{Value, json};
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::AtomicU64;
use std::sync::{Mutex, OnceLock};
use std::thread;

pub static NEXT_ACQUISITION_JOB_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub struct AcquisitionJob {
    pub status: String,
    pub done: u64,
    pub total: u64,
    pub message: String,
    pub result: Option<Value>,
    pub error: Option<String>,
    pub control: crate::ram::CancellationToken,
}

#[derive(Clone)]
pub struct EvidenceCaseState {
    pub base_dir: PathBuf,
    pub case_name: String,
}

#[derive(Clone)]
pub struct ImageMountState {
    #[cfg(windows)]
    pub image_path: PathBuf,
    pub mount_dir: PathBuf,
    #[cfg(target_os = "linux")]
    pub loop_device: Option<PathBuf>,
}

pub fn acquisition_jobs() -> &'static Mutex<HashMap<String, AcquisitionJob>> {
    static ACQUISITION_JOBS: OnceLock<Mutex<HashMap<String, AcquisitionJob>>> = OnceLock::new();
    ACQUISITION_JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn current_evidence_case() -> &'static Mutex<Option<EvidenceCaseState>> {
    static CURRENT_EVIDENCE_CASE: OnceLock<Mutex<Option<EvidenceCaseState>>> = OnceLock::new();
    CURRENT_EVIDENCE_CASE.get_or_init(|| Mutex::new(None))
}

pub fn current_image_mount() -> &'static Mutex<Option<ImageMountState>> {
    static CURRENT_IMAGE_MOUNT: OnceLock<Mutex<Option<ImageMountState>>> = OnceLock::new();
    CURRENT_IMAGE_MOUNT.get_or_init(|| Mutex::new(None))
}

pub fn wireguard_manager() -> &'static Mutex<crate::wireguard::WireGuardManager> {
    static WIREGUARD_MANAGER: OnceLock<Mutex<crate::wireguard::WireGuardManager>> = OnceLock::new();
    WIREGUARD_MANAGER.get_or_init(|| Mutex::new(crate::wireguard::WireGuardManager::new()))
}

pub fn route_api(method: &str, path: &str, body: &[u8]) -> Response {
    match (method, path) {
        ("GET", "/api/health") => json_ok(serde_json::json!({
            "ok": true,
            "version": env!("CARGO_PKG_VERSION"),
        })),
        ("GET", "/api/settings-default") => {
            match serde_json::to_value(crate::settings::AppSettings::default()) {
                Ok(value) => json_ok(value),
                Err(err) => json_error(500, err.to_string()),
            }
        }
        ("GET", "/api/disk-list") => system::disk_list_endpoint(),
        ("GET", "/api/android-adb-status") => {
            match serde_json::to_value(crate::android::adb_status()) {
                Ok(value) => json_ok(value),
                Err(err) => json_error(500, err.to_string()),
            }
        }
        ("GET", "/api/android-devices") => match crate::android::list_devices() {
            Ok(devices) => json_ok(serde_json::json!({ "devices": devices })),
            Err(err) => json_error(500, err),
        },
        ("POST", "/api/android-logical-image") => android::android_logical_image_endpoint(body),
        ("POST", "/api/android-filesystem-image") => {
            android::android_filesystem_image_endpoint(body)
        }
        ("POST", "/api/android-ram-image") => android::android_ram_image_endpoint(body),
        ("GET", "/api/ram-status") => json_ok(serde_json::json!({
            "avml": crate::ram::avml_status(None),
            "winpmem": crate::ram::winpmem_status(None),
        })),
        ("POST", "/api/avml-install") => ram::avml_install_endpoint(),
        ("POST", "/api/winpmem-install") => ram::winpmem_install_endpoint(),
        ("POST", "/api/acquisition-control") => ram::acquisition_control_endpoint(body),
        ("POST", "/api/acquisition-status") => ram::acquisition_status_endpoint(body),
        ("POST", "/api/connect") => system::connect_endpoint(body),
        ("POST", "/api/hash") => system::hash_endpoint(body),
        ("POST", "/api/local-image") => system::local_image_endpoint(body),
        ("POST", "/api/local-ram") => ram::local_ram_endpoint(body),
        ("POST", "/api/remote-disks") => system::remote_disks_endpoint(body),
        ("POST", "/api/remote-image") => system::remote_image_endpoint(body),
        ("POST", "/api/remote-ram") => ram::remote_ram_endpoint(body),
        ("POST", "/api/remote-tool-check") => system::remote_tool_check_endpoint(body),
        ("POST", "/api/evidence-create") => evidence::evidence_create_endpoint(body),
        ("POST", "/api/evidence-add-note") => evidence::evidence_add_note_endpoint(body),
        ("POST", "/api/evidence-list-files") => evidence::evidence_list_files_endpoint(body),
        ("GET", "/api/evidence-cases") => evidence::evidence_cases_endpoint(),
        ("GET", "/api/evidence-summary") => evidence::evidence_summary_endpoint(),
        ("POST", "/api/report-create") => evidence::report_create_endpoint(body),
        ("POST", "/api/image-mount-readonly") => system::image_mount_readonly_endpoint(body),
        ("POST", "/api/image-unmount") => system::image_unmount_endpoint(),
        ("POST", "/api/image-browse") => system::image_browse_endpoint(body),
        ("POST", "/api/image-read-file") => system::image_read_file_endpoint(body),
        ("POST", "/api/ram-analyze-strings") => ram::ram_analyze_strings_endpoint(body),
        ("POST", "/api/ram-carve-files") => ram::ram_carve_files_endpoint(body),
        ("POST", "/api/ram-list-processes") => ram::ram_list_processes_endpoint(body),
        ("POST", "/api/ram-process-details") => ram::ram_process_details_endpoint(body),
        ("POST", "/api/ram-process-search") => ram::ram_process_search_endpoint(body),
        ("POST", "/api/ram-read-carved") => ram::ram_read_carved_endpoint(body),
        ("POST", "/api/wireguard-config") => wireguard::wireguard_config_endpoint(body),
        ("POST", "/api/wireguard-start") => wireguard::wireguard_start_endpoint(body),
        ("POST", "/api/wireguard-stop") => wireguard::wireguard_stop_endpoint(),
        ("GET", "/api/wireguard-status") => wireguard::wireguard_status_endpoint(),
        ("GET", "/api/update-check") => system::update_check_endpoint(),
        ("POST", "/api/update-download") => system::update_download_endpoint(body),
        ("POST", "/api/update-install") => system::update_install_endpoint(body),
        ("POST", "/api/open-url") => system::open_url_endpoint(body),
        ("POST", "/api/pick-file") => system::pick_path_endpoint(false),
        ("POST", "/api/pick-folder") => system::pick_path_endpoint(true),
        _ => json_error(404, "api endpoint not found"),
    }
}

pub fn create_acquisition_job(message: &str) -> (String, crate::ram::CancellationToken) {
    let id = NEXT_ACQUISITION_JOB_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let job_id = format!("acq-{id}");
    let control = crate::ram::CancellationToken::default();
    let job = AcquisitionJob {
        status: "running".to_string(),
        done: 0,
        total: 0,
        message: message.to_string(),
        result: None,
        error: None,
        control: control.clone(),
    };
    if let Ok(mut jobs) = acquisition_jobs().lock() {
        jobs.insert(job_id.clone(), job);
    }
    (job_id, control)
}

pub fn update_acquisition_progress(job_id: &str, done: u64, total: u64) {
    update_acquisition_progress_message(job_id, done, total, "Imaj alma sürüyor");
}

pub fn update_acquisition_progress_message(job_id: &str, done: u64, total: u64, label: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.status = "running".to_string();
        job.done = done;
        job.total = total;
        job.message = if total > 0 {
            format!("{label}: {}%", progress_percent(done, total))
        } else {
            label.to_string()
        };
    }
}

pub fn update_acquisition_message(job_id: &str, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.message = message.to_string();
    }
}

pub fn finish_acquisition_job_with_message(job_id: &str, result: Value, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.status = "completed".to_string();
        if job.total == 0 {
            job.total = 1;
            job.done = 1;
        } else {
            job.done = job.total;
        }
        job.message = message.to_string();
        job.result = Some(result);
        job.error = None;
    }
}

pub fn fail_acquisition_job_with_message(job_id: &str, error: String, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.status = "failed".to_string();
        job.message = message.to_string();
        job.error = Some(error);
    }
}

fn progress_percent(done: u64, total: u64) -> u64 {
    if total == 0 { 0 } else { done * 100 / total }
}

pub fn default_case_base_dir() -> PathBuf {
    #[cfg(test)]
    if let Some(path) = test_case_base_dir()
        .lock()
        .ok()
        .and_then(|current| current.clone())
    {
        return path;
    }

    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Worm")
        .join("Vakalar")
}

pub fn default_case_name() -> String {
    format!("Case_{}", Local::now().format("%Y%m%d_%H%M%S"))
}

pub fn sanitize_case_name(value: &str) -> String {
    let sanitized = sanitize_file_stem(value);
    if sanitized.is_empty() {
        String::new()
    } else {
        sanitized
    }
}

pub fn sanitize_file_stem(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    sanitized.trim_matches('_').to_string()
}

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

pub fn set_current_evidence_case(base_dir: PathBuf, case_name: String) {
    if let Ok(mut current) = current_evidence_case().lock() {
        *current = Some(EvidenceCaseState {
            base_dir,
            case_name,
        });
    }
}

pub fn evidence_vault_for_output(
    case_name: Option<&str>,
) -> Result<crate::evidence::EvidenceVault, String> {
    let explicit_case = case_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_case_name)
        .filter(|value| !value.is_empty());

    let (base_dir, case_name) = if let Some(case_name) = explicit_case {
        (default_case_base_dir(), case_name)
    } else if let Some(state) = current_evidence_case()
        .lock()
        .ok()
        .and_then(|current| current.clone())
    {
        (state.base_dir, state.case_name)
    } else {
        (default_case_base_dir(), default_case_name())
    };

    let vault = crate::evidence::EvidenceVault::create(&base_dir, &case_name)
        .map_err(|err| err.to_string())?;
    set_current_evidence_case(base_dir, case_name);
    Ok(vault)
}

pub fn current_evidence_vault() -> Result<crate::evidence::EvidenceVault, Response> {
    let state = current_evidence_case()
        .lock()
        .ok()
        .and_then(|current| current.clone())
        .ok_or_else(|| json_error(400, "case is not created"))?;
    crate::evidence::EvidenceVault::create(&state.base_dir, &state.case_name)
        .map_err(|err| json_error(500, err.to_string()))
}

pub fn report_evidence_vault(
    case_name: Option<&str>,
) -> Result<crate::evidence::EvidenceVault, Response> {
    let explicit_case = case_name
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(sanitize_case_name)
        .filter(|value| !value.is_empty());

    let (base_dir, case_name) = if let Some(case_name) = explicit_case {
        (default_case_base_dir(), case_name)
    } else if let Some(state) = current_evidence_case()
        .lock()
        .ok()
        .and_then(|current| current.clone())
    {
        (state.base_dir, state.case_name)
    } else {
        (default_case_base_dir(), default_case_name())
    };

    let vault = crate::evidence::EvidenceVault::create(&base_dir, &case_name)
        .map_err(|err| json_error(500, err.to_string()))?;
    set_current_evidence_case(base_dir, case_name);
    Ok(vault)
}

pub fn evidence_subdir(value: &str) -> &'static str {
    match value {
        "gunlukler" | "logs" => "gunlukler",
        "raporlar" | "reports" => "raporlar",
        "hash" => "hash",
        "notlar" | "notes" => "notlar",
        "disk_imajlari" | "ciktilar" | "outputs" | "images" => "ciktilar",
        "ram" => "ram",
        "android" => "android",
        _ => "ciktilar",
    }
}

#[cfg(test)]
pub fn test_case_base_dir() -> &'static Mutex<Option<PathBuf>> {
    static TEST_CASE_BASE_DIR: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    TEST_CASE_BASE_DIR.get_or_init(|| Mutex::new(None))
}

// ---------------------------------------------------------
// Shared Helper Functions (used by ram.rs, system.rs, etc)
// ---------------------------------------------------------

pub fn command_error_message(output: &std::process::Output, fallback: &str) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        fallback.to_string()
    } else {
        stderr
    }
}

pub fn process_is_root() -> bool {
    #[cfg(target_os = "linux")]
    {
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(windows)]
    {
        crate::ram::is_root_or_admin()
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        false
    }
}

pub fn spawn_elevated_helper(args: &[String]) -> Result<Child, String> {
    #[cfg(target_os = "linux")]
    {
        let exe = elevated_helper_executable()?;
        Command::new("pkexec")
            .arg(exe)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("pkexec baslatilamadi: {err}"))
    }

    #[cfg(windows)]
    {
        let exe = std::env::current_exe().map_err(|err| err.to_string())?;
        let exe_str = exe.to_string_lossy().to_string();

        let quoted_args: Vec<String> = args
            .iter()
            .map(|a| {
                let escaped = a.replace('\'', "''");
                format!("'{escaped}'")
            })
            .collect();
        let arg_list = quoted_args.join(",");

        let ps_command = format!(
            "$ErrorActionPreference='Stop'; \
             $process = Start-Process -FilePath '{}' -ArgumentList {} -Verb RunAs -Wait -PassThru; \
             exit $process.ExitCode",
            exe_str.replace('\'', "''"),
            arg_list,
        );

        Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(&ps_command)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("UAC baslatilamadi: {err}"))
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = args;
        Err("yetki yükseltme bu platformda desteklenmiyor".to_string())
    }
}

#[cfg(target_os = "linux")]
pub fn elevated_helper_executable() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("APPIMAGE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }
    std::env::current_exe().map_err(|err| err.to_string())
}

pub fn run_elevated_helper_wait(args: &[String]) -> Result<(), String> {
    let mut child = spawn_elevated_helper(args)?;
    let status = child.wait().map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("yetki yükseltme iptal edildi veya başarısız oldu".to_string())
    }
}

pub fn download_file_to_path(url: &str, target: &Path, fallback: &str) -> Result<(), String> {
    #[cfg(windows)]
    let output = {
        let target_str = target.to_string_lossy();
        let ps_command = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
             $ProgressPreference = 'SilentlyContinue'; \
             Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
            url.replace('\'', "''"),
            target_str.replace('\'', "''"),
        );
        Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(&ps_command)
            .output()
    };

    #[cfg(not(windows))]
    let output = Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--silent")
        .arg("--show-error")
        .arg("-o")
        .arg(target)
        .arg(url)
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(command_error_message(&output, fallback)),
        Err(err) => Err(err.to_string()),
    }
}

pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|err| err.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 64];
    loop {
        let read = file.read(&mut buffer).map_err(|err| err.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(crate::hash::to_hex(&hasher.finalize()))
}

pub fn helper_file_stem(prefix: &str) -> String {
    format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        Local::now().format("%Y%m%d%H%M%S%3f")
    )
}

pub fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

pub fn write_helper_control_state(path: &Path, state: &str) -> Result<(), String> {
    write_json_file(path, &json!({ "state": state }))
}

pub fn read_helper_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

pub fn read_helper_error(path: &Path) -> Option<String> {
    read_helper_json(path).ok().and_then(|value| {
        value
            .get("error")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

pub fn read_helper_progress(path: &Path) -> Option<(u64, u64, String)> {
    let value = read_helper_json(path).ok()?;
    let done = value
        .get("done")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let total = value
        .get("total")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("Imaj alma sürüyor")
        .to_string();
    Some((done, total, message))
}

pub fn cleanup_helper_files(paths: &[&Path]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

#[cfg(unix)]
pub fn helper_owner_uid() -> Option<u32> {
    Some(unsafe { libc::geteuid() })
}

#[cfg(not(unix))]
pub fn helper_owner_uid() -> Option<u32> {
    None
}

#[cfg(unix)]
pub fn helper_owner_gid() -> Option<u32> {
    Some(unsafe { libc::getegid() })
}

#[cfg(not(unix))]
pub fn helper_owner_gid() -> Option<u32> {
    None
}

pub fn elevated_disk_list() -> Result<Vec<crate::disk::DiskInfo>, String> {
    let output_path = std::env::temp_dir().join(format!(
        "worm-disk-list-{}-{}.json",
        std::process::id(),
        Local::now().format("%Y%m%d%H%M%S%3f")
    ));

    let run_result = run_elevated_disk_list_helper(&output_path);
    if let Err(err) = run_result {
        let _ = fs::remove_file(&output_path);
        return Err(err);
    }

    let content = fs::read_to_string(&output_path).map_err(|err| err.to_string())?;
    let _ = fs::remove_file(&output_path);
    let value: Value = serde_json::from_str(&content).map_err(|err| err.to_string())?;
    if value.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(value
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("elevated disk list failed")
            .to_string());
    }
    serde_json::from_value(
        value
            .get("disks")
            .cloned()
            .unwrap_or(Value::Array(Vec::new())),
    )
    .map_err(|err| err.to_string())
}

#[cfg(target_os = "linux")]
fn run_elevated_disk_list_helper(output_path: &Path) -> Result<(), String> {
    run_elevated_helper_wait(&[
        "disk-list-helper".to_string(),
        output_path.to_string_lossy().into_owned(),
    ])
}

#[cfg(windows)]
fn run_elevated_disk_list_helper(output_path: &Path) -> Result<(), String> {
    run_elevated_helper_wait(&[
        "disk-list-helper".to_string(),
        output_path.to_string_lossy().into_owned(),
    ])
}

#[cfg(not(any(target_os = "linux", windows)))]
fn run_elevated_disk_list_helper(_output_path: &Path) -> Result<(), String> {
    Err("yetki yükseltmeli disk listeleme bu platformda desteklenmiyor".to_string())
}

// ---------------------------------------------------------
// Image Mounting Helper Functions
// ---------------------------------------------------------

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
    if result.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(format!(
            "{initial_error}\nyetkili mount başarısız: {}",
            result
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("bilinmeyen hata")
        ));
    }

    Ok(result
        .get("loop_device")
        .and_then(Value::as_str)
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
    if result.get("ok").and_then(Value::as_bool) == Some(true) {
        Ok(())
    } else {
        Err(result
            .get("error")
            .and_then(Value::as_str)
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
