use crate::android;
use crate::disk;
use crate::disk::{DiskAcquisitionControl, DiskAcquisitionTask};
use crate::evidence::EvidenceVault;
use crate::hash::{self, HashAlgorithm};
use crate::ram;
use crate::remote::RemoteConnection;
use crate::report::{self, ReportFormat, ReportInfo};
use crate::settings::AppSettings;
use crate::wireguard::{self, WireGuardConfig, WireGuardManager};
use chrono::Local;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;

const DEV_UI_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/ui");
#[cfg(windows)]
const WINPMEM_DOWNLOAD_URL: &str = "https://github.com/Velocidex/WinPmem/releases/download/v4.0.rc1/go-winpmem_amd64_1.0-rc2_signed.exe";
static NEXT_ACQUISITION_JOB_ID: AtomicU64 = AtomicU64::new(1);
static ACQUISITION_JOBS: OnceLock<Mutex<HashMap<String, AcquisitionJob>>> = OnceLock::new();
static CURRENT_EVIDENCE_CASE: OnceLock<Mutex<Option<EvidenceCaseState>>> = OnceLock::new();
static CURRENT_IMAGE_MOUNT: OnceLock<Mutex<Option<ImageMountState>>> = OnceLock::new();
static WIREGUARD_MANAGER: OnceLock<Mutex<WireGuardManager>> = OnceLock::new();

#[derive(Clone)]
struct AcquisitionJob {
    status: String,
    done: u64,
    total: u64,
    message: String,
    result: Option<Value>,
    error: Option<String>,
    control: ram::CancellationToken,
}

#[derive(Clone)]
struct EvidenceCaseState {
    base_dir: PathBuf,
    case_name: String,
}

#[derive(Clone)]
struct ImageMountState {
    #[cfg(windows)]
    image_path: PathBuf,
    mount_dir: PathBuf,
    #[cfg(target_os = "linux")]
    loop_device: Option<PathBuf>,
}

pub fn run_native() -> Result<(), String> {
    crate::native_window::prepare_environment();
    let url = start_background()?;
    let native_url = format!("{url}?native=1");
    println!("Worm native UI: {native_url}");
    crate::native_window::run(&native_url)
}

pub fn run_browser() -> Result<(), String> {
    let url = start_background()?;
    println!("Worm UI backend: {url}");
    open_window(&url);
    loop {
        thread::park();
    }
}

fn start_background() -> Result<String, String> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|err| err.to_string())?;
    let addr = listener.local_addr().map_err(|err| err.to_string())?;
    let url = format!("http://{addr}/");

    thread::Builder::new()
        .name("worm-ui-server".to_string())
        .spawn(move || serve(listener))
        .map_err(|err| err.to_string())?;

    Ok(url)
}

fn serve(listener: TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    if let Err(err) = handle_stream(stream) {
                        eprintln!("UI request failed: {err}");
                    }
                });
            }
            Err(err) => eprintln!("UI connection failed: {err}"),
        }
    }
}

fn open_window(url: &str) {
    let browsers: &[(&str, &[&str])] = &[
        (
            "chromium",
            &["--new-window", "--no-first-run", "--app", url],
        ),
        (
            "google-chrome",
            &["--new-window", "--no-first-run", "--app", url],
        ),
        (
            "google-chrome-stable",
            &["--new-window", "--no-first-run", "--app", url],
        ),
        (
            "brave-browser",
            &["--new-window", "--no-first-run", "--app", url],
        ),
        ("firefox", &[url]),
        ("xdg-open", &[url]),
    ];

    for (program, args) in browsers {
        if Command::new(program)
            .args(*args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
        {
            return;
        }
    }

    eprintln!("Browser could not be opened automatically. Use: {url}");
}

fn handle_stream(stream: TcpStream) -> Result<(), String> {
    let peer = stream.peer_addr().ok();
    if peer.map(|addr| !addr.ip().is_loopback()).unwrap_or(true) {
        return Err("non-loopback request rejected".to_string());
    }

    let mut reader = BufReader::new(stream.try_clone().map_err(|err| err.to_string())?);
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|err| err.to_string())?;
    if request_line.trim().is_empty() {
        return Ok(());
    }

    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let raw_path = parts.next().unwrap_or("/").to_string();
    let mut content_length = 0_usize;

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(|err| err.to_string())?;
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse::<usize>().unwrap_or(0);
        }
    }

    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|err| err.to_string())?;
    }

    let response = route_request(&method, &raw_path, &body);
    write_response(stream, response)
}

fn route_request(method: &str, raw_path: &str, body: &[u8]) -> Response {
    if method == "OPTIONS" {
        return Response::empty(204);
    }

    let path = raw_path.split('?').next().unwrap_or("/");
    if path.starts_with("/api/") {
        return route_api(method, path, body);
    }

    if method != "GET" && method != "HEAD" {
        return json_error(405, "method not allowed");
    }

    serve_static(path, method == "HEAD")
}

fn route_api(method: &str, path: &str, body: &[u8]) -> Response {
    match (method, path) {
        ("GET", "/api/health") => json_ok(json!({
            "ok": true,
            "version": env!("CARGO_PKG_VERSION"),
        })),
        ("GET", "/api/settings-default") => match serde_json::to_value(AppSettings::default()) {
            Ok(value) => json_ok(value),
            Err(err) => json_error(500, err.to_string()),
        },
        ("GET", "/api/disk-list") => disk_list_endpoint(),
        ("GET", "/api/android-adb-status") => match serde_json::to_value(android::adb_status()) {
            Ok(value) => json_ok(value),
            Err(err) => json_error(500, err.to_string()),
        },
        ("GET", "/api/android-devices") => match android::list_devices() {
            Ok(devices) => json_ok(json!({ "devices": devices })),
            Err(err) => json_error(500, err),
        },
        ("GET", "/api/ram-status") => json_ok(json!({
            "avml": ram::avml_status(None),
            "winpmem": ram::winpmem_status(None),
        })),
        ("POST", "/api/avml-install") => avml_install_endpoint(),
        ("POST", "/api/winpmem-install") => winpmem_install_endpoint(),
        ("POST", "/api/acquisition-control") => acquisition_control_endpoint(body),
        ("POST", "/api/acquisition-status") => acquisition_status_endpoint(body),
        ("POST", "/api/connect") => connect_endpoint(body),
        ("POST", "/api/hash") => hash_endpoint(body),
        ("POST", "/api/local-image") => local_image_endpoint(body),
        ("POST", "/api/local-ram") => local_ram_endpoint(body),
        ("POST", "/api/remote-disks") => remote_disks_endpoint(body),
        ("POST", "/api/remote-image") => remote_image_endpoint(body),
        ("POST", "/api/remote-ram") => remote_ram_endpoint(body),
        ("POST", "/api/remote-tool-check") => remote_tool_check_endpoint(body),
        ("POST", "/api/evidence-create") => evidence_create_endpoint(body),
        ("POST", "/api/evidence-add-note") => evidence_add_note_endpoint(body),
        ("POST", "/api/evidence-list-files") => evidence_list_files_endpoint(body),
        ("GET", "/api/evidence-cases") => evidence_cases_endpoint(),
        ("GET", "/api/evidence-summary") => evidence_summary_endpoint(),
        ("POST", "/api/report-create") => report_create_endpoint(body),
        ("POST", "/api/image-mount-readonly") => image_mount_readonly_endpoint(body),
        ("POST", "/api/image-unmount") => image_unmount_endpoint(),
        ("POST", "/api/wireguard-config") => wireguard_config_endpoint(body),
        ("POST", "/api/wireguard-start") => wireguard_start_endpoint(body),
        ("POST", "/api/wireguard-stop") => wireguard_stop_endpoint(),
        ("GET", "/api/wireguard-status") => wireguard_status_endpoint(),
        ("GET", "/api/update-check") => update_check_endpoint(),
        ("POST", "/api/update-download") => update_download_endpoint(body),
        ("POST", "/api/update-install") => update_install_endpoint(body),
        ("POST", "/api/pick-file") => pick_path_endpoint(false),
        ("POST", "/api/pick-folder") => pick_path_endpoint(true),
        _ => json_error(404, "api endpoint not found"),
    }
}

fn serve_static(path: &str, head_only: bool) -> Response {
    let path = if path == "/" { "/index.html" } else { path };
    let Ok(decoded) = percent_decode(path) else {
        return json_error(400, "invalid path encoding");
    };
    let relative = decoded.trim_start_matches('/');
    if relative.split('/').any(|part| part == "..") {
        return json_error(403, "path traversal rejected");
    }

    let mut file_path = ui_root();
    file_path.push(relative);
    if file_path.is_dir() {
        file_path.push("index.html");
    }

    match fs::read(&file_path) {
        Ok(body) => Response {
            status: 200,
            content_type: mime_for(&file_path).to_string(),
            body: if head_only { Vec::new() } else { body },
        },
        Err(_) => Response {
            status: 404,
            content_type: "text/html; charset=utf-8".to_string(),
            body: b"Not found".to_vec(),
        },
    }
}

fn connect_endpoint(body: &[u8]) -> Response {
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

fn disk_list_endpoint() -> Response {
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

fn hash_endpoint(body: &[u8]) -> Response {
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

fn local_image_endpoint(body: &[u8]) -> Response {
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

fn remote_image_endpoint(body: &[u8]) -> Response {
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

fn local_ram_endpoint(body: &[u8]) -> Response {
    let request: LocalRamRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    if request.output.trim().is_empty() {
        return json_error(400, "output is required");
    }

    let tool = request.tool.as_deref().unwrap_or_default();
    if !matches!(tool, "avml" | "winpmem") {
        return json_error(400, "tool must be avml or winpmem");
    }

    let (job_id, control) = create_acquisition_job("Yerel RAM edinimi başlatıldı");
    let thread_job_id = job_id.clone();
    thread::spawn(move || run_local_ram_job(thread_job_id, request, control));

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

fn remote_ram_endpoint(body: &[u8]) -> Response {
    let request: RemoteRamRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    if request.ip.trim().is_empty() {
        return json_error(400, "ip is required");
    }
    if request.port == 0 {
        return json_error(400, "port is required");
    }
    if request.output.trim().is_empty() {
        return json_error(400, "output is required");
    }

    let (job_id, _control) = create_acquisition_job("Uzak RAM edinimi başlatıldı");
    let thread_job_id = job_id.clone();
    thread::spawn(move || run_remote_ram_job(thread_job_id, request));

    json_ok(json!({
        "job_id": job_id,
        "status": "running",
    }))
}

fn evidence_create_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct EvidenceCreateRequest {
        case_name: String,
    }

    let request: EvidenceCreateRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    let case_name = sanitize_case_name(&request.case_name);
    if case_name.is_empty() {
        return json_error(400, "case_name is required");
    }
    let base_dir = default_case_base_dir();

    match EvidenceVault::create(&base_dir, &case_name) {
        Ok(vault) => {
            let summary = match vault.summary() {
                Ok(summary) => summary,
                Err(err) => return json_error(500, err.to_string()),
            };
            set_current_evidence_case(base_dir, case_name);
            json_ok(json!({
                "case_name": summary.case_name,
                "case_dir": summary.case_dir,
                "base_dir": default_case_base_dir(),
                "output_dir": vault.outputs_dir,
                "output_count": summary.output_count,
                "hash_count": summary.hash_count,
                "report_count": summary.report_count,
            }))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

fn evidence_add_note_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct EvidenceNoteRequest {
        note: String,
        case_name: Option<String>,
    }

    let request: EvidenceNoteRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    if request.note.trim().is_empty() {
        return json_error(400, "note is required");
    }

    let vault = match report_evidence_vault(request.case_name.as_deref()) {
        Ok(vault) => vault,
        Err(response) => return response,
    };
    match vault.add_note(request.note.trim()) {
        Ok(path) => json_ok(json!({ "path": path })),
        Err(err) => json_error(500, err.to_string()),
    }
}

fn evidence_list_files_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct EvidenceListRequest {
        subdir: Option<String>,
    }

    let request: EvidenceListRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let vault = match current_evidence_vault() {
        Ok(vault) => vault,
        Err(response) => return response,
    };
    let subdir = evidence_subdir(request.subdir.as_deref().unwrap_or_default());

    match vault.list_files(subdir) {
        Ok(files) => {
            let files: Vec<Value> = files.into_iter().map(file_entry_json).collect();
            json_ok(json!({ "subdir": subdir, "files": files }))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

fn evidence_summary_endpoint() -> Response {
    let vault = match current_evidence_vault() {
        Ok(vault) => vault,
        Err(response) => return response,
    };
    match vault.summary() {
        Ok(summary) => json_ok(json!({
            "case_name": summary.case_name,
            "case_dir": summary.case_dir,
            "output_count": summary.output_count,
            "hash_count": summary.hash_count,
            "report_count": summary.report_count,
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

fn evidence_cases_endpoint() -> Response {
    let base_dir = default_case_base_dir();
    if let Err(err) = fs::create_dir_all(&base_dir) {
        return json_error(500, err.to_string());
    }

    let mut cases = Vec::new();
    let entries = match fs::read_dir(&base_dir) {
        Ok(entries) => entries,
        Err(err) => return json_error(500, err.to_string()),
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let case_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        if case_name.is_empty() {
            continue;
        }
        cases.push(case_listing_json(&case_name, &path));
    }
    cases.sort_by(|left, right| {
        left["case_name"]
            .as_str()
            .unwrap_or_default()
            .cmp(right["case_name"].as_str().unwrap_or_default())
    });

    let current = current_evidence_case()
        .lock()
        .ok()
        .and_then(|state| state.clone())
        .map(|state| {
            let case_dir = state.base_dir.join(&state.case_name);
            json!({
                "case_name": state.case_name,
                "case_dir": case_dir,
                "base_dir": state.base_dir,
                "output_dir": case_dir.join("ciktilar"),
            })
        });

    json_ok(json!({
        "base_dir": base_dir,
        "cases": cases,
        "current_case": current,
    }))
}

fn report_create_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct ReportCreateRequest {
        case_name: Option<String>,
        title: Option<String>,
        description: Option<String>,
        source: Option<String>,
        hash_sha256: Option<String>,
        format: Option<String>,
    }

    let request: ReportCreateRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    let vault = match report_evidence_vault(request.case_name.as_deref()) {
        Ok(vault) => vault,
        Err(response) => return response,
    };
    let format = match report_format(request.format.as_deref().unwrap_or("txt")) {
        Some(format) => format,
        None => return json_error(400, "format must be txt or json"),
    };
    let title = request
        .title
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Forensic Technical Report");
    let description = request
        .description
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    let source = request
        .source
        .as_deref()
        .map(str::trim)
        .unwrap_or("Worm Forensic Tool");
    let hash_sha256 = request
        .hash_sha256
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();
    let info = ReportInfo {
        title: title.to_string(),
        description: description.to_string(),
        creator: std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "worm".to_string()),
        source: source.to_string(),
        hash_sha256: hash_sha256.to_string(),
        date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    let target = vault
        .reports_dir
        .join(report::new_report_file_name(&vault.case_name, format));

    match report::create_report(&info, format, &target, Some(&vault)) {
        Ok(path) => json_ok(json!({ "path": path })),
        Err(err) => json_error(500, err.to_string()),
    }
}

fn image_mount_readonly_endpoint(body: &[u8]) -> Response {
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

fn image_unmount_endpoint() -> Response {
    match image_unmount_current() {
        Ok(Some(mount_dir)) => json_ok(json!({ "mount_dir": mount_dir })),
        Ok(None) => json_ok(json!({ "mount_dir": Value::Null })),
        Err(err) => json_error(500, err),
    }
}

fn wireguard_config_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct WireGuardConfigRequest {
        config_file: String,
        private_key: Option<String>,
        public_key: Option<String>,
        endpoint: String,
        allowed_ips: Option<String>,
        address: Option<String>,
        dns: Option<String>,
        keepalive: Option<u16>,
    }

    let request: WireGuardConfigRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    if request.config_file.trim().is_empty() {
        return json_error(400, "config_file is required");
    }
    if request.endpoint.trim().is_empty() {
        return json_error(400, "endpoint is required");
    }
    let config = WireGuardConfig {
        private_key: request.private_key.as_deref().unwrap_or_default().trim(),
        public_key: request.public_key.as_deref().unwrap_or_default().trim(),
        endpoint: request.endpoint.trim(),
        allowed_ips: request.allowed_ips.as_deref().unwrap_or("0.0.0.0/0").trim(),
        address: request.address.as_deref().unwrap_or("10.0.0.2/24").trim(),
        dns: request.dns.as_deref().unwrap_or("1.1.1.1").trim(),
        keepalive: request.keepalive.unwrap_or(25),
    };

    match wireguard::create_config(request.config_file.trim(), &config) {
        Ok(path) => json_ok(json!({ "path": path })),
        Err(err) => json_error(500, err.to_string()),
    }
}

fn wireguard_start_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct WireGuardStartRequest {
        config_file: String,
    }

    let request: WireGuardStartRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };
    if request.config_file.trim().is_empty() {
        return json_error(400, "config_file is required");
    }
    let manager = wireguard_manager();
    let mut guard = match manager.lock() {
        Ok(guard) => guard,
        Err(_) => return json_error(500, "wireguard manager lock failed"),
    };
    match guard.start(request.config_file.trim()) {
        Ok(()) => json_ok(json!({
            "active": guard.is_active(),
            "config_file": guard.config_file,
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

fn wireguard_stop_endpoint() -> Response {
    let manager = wireguard_manager();
    let mut guard = match manager.lock() {
        Ok(guard) => guard,
        Err(_) => return json_error(500, "wireguard manager lock failed"),
    };
    match guard.stop() {
        Ok(()) => json_ok(json!({ "active": guard.is_active() })),
        Err(err) => json_error(500, err.to_string()),
    }
}

fn wireguard_status_endpoint() -> Response {
    let manager = wireguard_manager();
    let guard = match manager.lock() {
        Ok(guard) => guard,
        Err(_) => return json_error(500, "wireguard manager lock failed"),
    };
    json_ok(json!({
        "interface_name": guard.interface_name,
        "config_file": guard.config_file,
        "active": guard.active,
    }))
}

fn update_check_endpoint() -> Response {
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

fn update_download_endpoint(body: &[u8]) -> Response {
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

fn update_install_endpoint(body: &[u8]) -> Response {
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

fn avml_install_endpoint() -> Response {
    #[cfg(not(target_os = "linux"))]
    {
        return json_error(400, "AVML installation is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        let Some(asset_name) = avml_release_asset_name() else {
            return json_error(
                400,
                format!(
                    "AVML binary is not available for this architecture: {}",
                    std::env::consts::ARCH
                ),
            );
        };
        let url =
            format!("https://github.com/microsoft/avml/releases/latest/download/{asset_name}");
        let download_dir = std::env::temp_dir().join("worm-avml-install");
        if let Err(err) = fs::create_dir_all(&download_dir) {
            return json_error(500, err.to_string());
        }
        let download_path = download_dir.join(format!("{asset_name}.download"));

        if let Err(err) = download_file_to_path(&url, &download_path, "AVML download failed") {
            let _ = fs::remove_file(&download_path);
            return json_error(500, err);
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(&download_path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o755);
                let _ = fs::set_permissions(&download_path, permissions);
            }
        }

        let sha256 = match sha256_file(&download_path) {
            Ok(value) => value,
            Err(err) => {
                let _ = fs::remove_file(&download_path);
                return json_error(500, err);
            }
        };
        let stem = helper_file_stem("worm-avml-install");
        let result_path = download_dir.join(format!("{stem}-result.json"));
        let args = vec![
            "avml-install-helper".to_string(),
            download_path.to_string_lossy().into_owned(),
            result_path.to_string_lossy().into_owned(),
        ];

        let run_result = run_elevated_helper_wait(&args);
        let helper_result = read_helper_json(&result_path).ok();
        cleanup_helper_files(&[&download_path, &result_path]);
        if let Err(err) = run_result {
            let message = helper_result
                .as_ref()
                .and_then(|value| value.get("error"))
                .and_then(Value::as_str)
                .unwrap_or(&err)
                .to_string();
            return json_error(500, message);
        }

        let helper_result = match helper_result {
            Some(value) => value,
            None => return json_error(500, "AVML install helper did not return a result"),
        };
        if helper_result.get("ok").and_then(Value::as_bool) != Some(true) {
            return json_error(
                500,
                helper_result
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("AVML installation failed")
                    .to_string(),
            );
        }

        json_ok(json!({
            "asset": asset_name,
            "download_url": url,
            "sha256": sha256,
            "path": helper_result.get("path").cloned().unwrap_or(Value::Null),
            "version": helper_result.get("version").cloned().unwrap_or(Value::Null),
            "message": helper_result.get("message").cloned().unwrap_or(Value::String("AVML installed".to_string())),
            "status": ram::avml_status(None),
        }))
    }
}

fn winpmem_install_endpoint() -> Response {
    #[cfg(not(windows))]
    {
        return json_error(400, "WinPMEM installation is only supported on Windows");
    }

    #[cfg(windows)]
    {
        if std::env::consts::ARCH != "x86_64" {
            return json_error(
                400,
                format!(
                    "WinPMEM binary is not available for this architecture: {}",
                    std::env::consts::ARCH
                ),
            );
        }

        let download_dir = std::env::temp_dir().join("worm-winpmem-install");
        if let Err(err) = fs::create_dir_all(&download_dir) {
            return json_error(500, err.to_string());
        }
        let download_path = download_dir.join(ram::WINPMEM_NAME);
        if let Err(err) = download_file_to_path(
            WINPMEM_DOWNLOAD_URL,
            &download_path,
            "WinPMEM download failed",
        ) {
            let _ = fs::remove_file(&download_path);
            return json_error(500, err);
        }

        let sha256 = match sha256_file(&download_path) {
            Ok(value) => value,
            Err(err) => {
                let _ = fs::remove_file(&download_path);
                return json_error(500, err);
            }
        };
        let stem = helper_file_stem("worm-winpmem-install");
        let result_path = download_dir.join(format!("{stem}-result.json"));
        let args = vec![
            "winpmem-install-helper".to_string(),
            download_path.to_string_lossy().into_owned(),
            result_path.to_string_lossy().into_owned(),
        ];

        let run_result = run_elevated_helper_wait(&args);
        let helper_result = read_helper_json(&result_path).ok();
        cleanup_helper_files(&[&download_path, &result_path]);
        if let Err(err) = run_result {
            let message = helper_result
                .as_ref()
                .and_then(|value| value.get("error"))
                .and_then(Value::as_str)
                .unwrap_or(&err)
                .to_string();
            return json_error(500, message);
        }

        let helper_result = match helper_result {
            Some(value) => value,
            None => return json_error(500, "WinPMEM install helper did not return a result"),
        };
        if helper_result.get("ok").and_then(Value::as_bool) != Some(true) {
            return json_error(
                500,
                helper_result
                    .get("error")
                    .and_then(Value::as_str)
                    .unwrap_or("WinPMEM installation failed")
                    .to_string(),
            );
        }

        json_ok(json!({
            "asset": ram::WINPMEM_NAME,
            "download_url": WINPMEM_DOWNLOAD_URL,
            "sha256": sha256,
            "path": helper_result.get("path").cloned().unwrap_or(Value::Null),
            "message": helper_result.get("message").cloned().unwrap_or(Value::String("WinPMEM installed".to_string())),
            "status": ram::winpmem_status(None),
        }))
    }
}

fn avml_release_asset_name() -> Option<&'static str> {
    match std::env::consts::ARCH {
        "x86_64" => Some("avml"),
        "aarch64" => Some("avml-aarch64"),
        _ => None,
    }
}

fn current_evidence_case() -> &'static Mutex<Option<EvidenceCaseState>> {
    CURRENT_EVIDENCE_CASE.get_or_init(|| Mutex::new(None))
}

fn current_image_mount() -> &'static Mutex<Option<ImageMountState>> {
    CURRENT_IMAGE_MOUNT.get_or_init(|| Mutex::new(None))
}

fn wireguard_manager() -> &'static Mutex<WireGuardManager> {
    WIREGUARD_MANAGER.get_or_init(|| Mutex::new(WireGuardManager::new()))
}

fn current_evidence_vault() -> Result<EvidenceVault, Response> {
    let state = current_evidence_case()
        .lock()
        .ok()
        .and_then(|current| current.clone())
        .ok_or_else(|| json_error(400, "case is not created"))?;
    EvidenceVault::create(&state.base_dir, &state.case_name)
        .map_err(|err| json_error(500, err.to_string()))
}

fn report_evidence_vault(case_name: Option<&str>) -> Result<EvidenceVault, Response> {
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

    let vault = EvidenceVault::create(&base_dir, &case_name)
        .map_err(|err| json_error(500, err.to_string()))?;
    set_current_evidence_case(base_dir, case_name);
    Ok(vault)
}

fn set_current_evidence_case(base_dir: PathBuf, case_name: String) {
    if let Ok(mut current) = current_evidence_case().lock() {
        *current = Some(EvidenceCaseState {
            base_dir,
            case_name,
        });
    }
}

fn sanitize_case_name(value: &str) -> String {
    let sanitized = sanitize_file_stem(value);
    if sanitized.is_empty() {
        String::new()
    } else {
        sanitized
    }
}

fn default_case_base_dir() -> PathBuf {
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

fn default_case_name() -> String {
    format!("Case_{}", Local::now().format("%Y%m%d_%H%M%S"))
}

#[cfg(test)]
fn test_case_base_dir() -> &'static Mutex<Option<PathBuf>> {
    static TEST_CASE_BASE_DIR: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    TEST_CASE_BASE_DIR.get_or_init(|| Mutex::new(None))
}

fn default_download_dir() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Downloads")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn evidence_subdir(value: &str) -> &'static str {
    match value {
        "gunlukler" | "logs" => "gunlukler",
        "raporlar" | "reports" => "raporlar",
        "hash" => "hash",
        "notlar" | "notes" => "notlar",
        "disk_imajlari" | "ram" | "ciktilar" | "outputs" | "images" => "ciktilar",
        _ => "ciktilar",
    }
}

fn case_listing_json(case_name: &str, case_dir: &Path) -> Value {
    json!({
        "case_name": case_name,
        "case_dir": case_dir,
        "output_dir": case_dir.join("ciktilar"),
        "output_count": count_directory_entries(&case_dir.join("ciktilar")),
        "hash_count": count_directory_entries(&case_dir.join("hash")),
        "report_count": count_directory_entries(&case_dir.join("raporlar")),
    })
}

fn count_directory_entries(path: &Path) -> usize {
    fs::read_dir(path)
        .map(|entries| entries.flatten().count())
        .unwrap_or_default()
}

fn elevated_disk_list() -> Result<Vec<disk::DiskInfo>, String> {
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

fn run_elevated_helper_wait(args: &[String]) -> Result<(), String> {
    let mut child = spawn_elevated_helper(args)?;
    let status = child.wait().map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("yetki yükseltme iptal edildi veya başarısız oldu".to_string())
    }
}

#[cfg(target_os = "linux")]
fn spawn_elevated_helper(args: &[String]) -> Result<std::process::Child, String> {
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

#[cfg(target_os = "linux")]
fn elevated_helper_executable() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("APPIMAGE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    std::env::current_exe().map_err(|err| err.to_string())
}

#[cfg(windows)]
fn spawn_elevated_helper(args: &[String]) -> Result<std::process::Child, String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(
            "$ErrorActionPreference='Stop'; \
             $exe=$args[0]; \
             $argList=@(); \
             for ($i=1; $i -lt $args.Count; $i++) { $argList += $args[$i] }; \
             $process=Start-Process -FilePath $exe -ArgumentList $argList -Verb RunAs -Wait -PassThru; \
             exit $process.ExitCode",
        )
        .arg(exe)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("UAC baslatilamadi: {err}"))
}

#[cfg(not(any(target_os = "linux", windows)))]
fn spawn_elevated_helper(_args: &[String]) -> Result<std::process::Child, String> {
    Err("yetki yükseltme bu platformda desteklenmiyor".to_string())
}

#[cfg(not(any(target_os = "linux", windows)))]
fn run_elevated_disk_list_helper(_output_path: &Path) -> Result<(), String> {
    Err("yetki yükseltmeli disk listeleme bu platformda desteklenmiyor".to_string())
}

fn file_entry_json(path: PathBuf) -> Value {
    let metadata = fs::metadata(&path).ok();
    json!({
        "name": path.file_name().and_then(|name| name.to_str()).unwrap_or_default(),
        "path": path,
        "is_dir": metadata.as_ref().map(|meta| meta.is_dir()).unwrap_or(false),
        "size": metadata.as_ref().map(|meta| meta.len()).unwrap_or_default(),
    })
}

fn report_format(value: &str) -> Option<ReportFormat> {
    match value.trim().to_ascii_lowercase().as_str() {
        "txt" => Some(ReportFormat::Txt),
        "json" => Some(ReportFormat::Json),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn linux_mount_image_readonly(
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

#[cfg(target_os = "linux")]
fn elevated_linux_mount_image_readonly(
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
fn elevated_linux_unmount_image(
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
fn linux_mount_partitioned_image(
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
fn linux_loop_mount_candidates(loop_device: &Path) -> Vec<PathBuf> {
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

fn command_error_message(output: &std::process::Output, fallback: &str) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        fallback.to_string()
    } else {
        stderr
    }
}

fn image_unmount_current() -> Result<Option<PathBuf>, String> {
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

fn sanitize_download_name(value: &str) -> String {
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

fn download_file_to_path(url: &str, target: &Path, fallback: &str) -> Result<(), String> {
    #[cfg(windows)]
    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(
            "$ProgressPreference='SilentlyContinue'; \
             Invoke-WebRequest -Uri $args[0] -OutFile $args[1] -UseBasicParsing",
        )
        .arg(url)
        .arg(target)
        .output();

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

fn sha256_file(path: &Path) -> Result<String, String> {
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
    Ok(format!("{:x}", hasher.finalize()))
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

#[derive(Deserialize)]
struct LocalImageRequest {
    source: String,
    output: String,
    case_name: Option<String>,
}

#[derive(Deserialize)]
struct RemoteImageRequest {
    ip: String,
    port: u16,
    token: Option<String>,
    disk_id: String,
    output: String,
    case_name: Option<String>,
}

#[derive(Deserialize)]
struct LocalRamRequest {
    output: String,
    tool: Option<String>,
    tool_path: Option<String>,
}

#[derive(Deserialize)]
struct RemoteRamRequest {
    ip: String,
    port: u16,
    token: Option<String>,
    output: String,
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
    let target = acquisition_target_path(&request.source, &output.to_string_lossy(), "local");
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
        Ok(result) => finish_acquisition_job_with_message(
            &job_id,
            json!({
                "message": "Imaj alma tamamlandi",
                "target_path": result.target,
                "bytes_copied": result.bytes_copied,
                "total_bytes": result.total_bytes,
                "sha256": result.sha256,
            }),
            "Imaj alma tamamlandi",
        ),
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
                &output,
                Some(&remote_job_id),
                |done, total| update_acquisition_progress(&job_id, done, total),
            ) {
                Ok(result) => finish_acquisition_job_with_message(
                    &job_id,
                    json!({
                        "message": result.message,
                        "remote_job_id": result.job_id,
                        "target_path": result.target_path,
                        "bytes_transferred": result.bytes_transferred,
                        "sha256": result.sha256,
                        "md5": result.md5,
                    }),
                    "Imaj alma tamamlandi",
                ),
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

fn run_local_ram_job(job_id: String, request: LocalRamRequest, control: ram::CancellationToken) {
    let tool = request.tool.as_deref().unwrap_or_default();
    if local_ram_requires_elevation(tool) {
        run_elevated_local_ram_job(&job_id, &request, &control);
        return;
    }

    let output = PathBuf::from(&request.output);
    let candidate = request
        .tool_path
        .as_deref()
        .map(Path::new)
        .filter(|path| path.exists());

    let result = match tool {
        "avml" => ram::acquire_with_avml(&output, candidate, &control, |done, total| {
            update_acquisition_progress_message(&job_id, done, total, "RAM edinimi sürüyor");
        }),
        "winpmem" => ram::acquire_with_winpmem(&output, candidate, &control, |done, total| {
            update_acquisition_progress_message(&job_id, done, total, "RAM edinimi sürüyor");
        }),
        _ => Err(crate::error::WormError::new(
            crate::error::HataKodu::Genel,
            "Desteklenmeyen RAM araci",
        )),
    };

    match result {
        Ok(result) => finish_acquisition_job_with_message(
            &job_id,
            json!({
                "message": "RAM edinimi tamamlandi",
                "target_path": result.output_file,
                "bytes_written": result.bytes_written,
            }),
            "RAM edinimi tamamlandi",
        ),
        Err(err) => {
            let message = err.to_string();
            if local_ram_error_can_retry_elevated(&message) {
                run_elevated_local_ram_job(&job_id, &request, &control);
            } else {
                fail_acquisition_job_with_message(&job_id, message, "RAM edinimi basarisiz")
            }
        }
    }
}

fn run_remote_ram_job(job_id: String, request: RemoteRamRequest) {
    let remote_file = ram_remote_file_name(&request.output);
    let target_path = PathBuf::from(&request.output);

    match RemoteConnection::connect(&request.ip, request.port, request.token.clone()) {
        Ok(mut connection) => {
            let remote_job_id = job_id.clone();
            match connection.start_remote_ram(&remote_file, Some(&remote_job_id), |done, total| {
                update_acquisition_progress_message(&job_id, done, total, "RAM edinimi sürüyor");
            }) {
                Ok(ram_result) => {
                    update_acquisition_message(&job_id, "RAM dosyası indiriliyor");
                    match connection.download_ram_file(
                        &remote_file,
                        &target_path,
                        Some(&remote_job_id),
                        |done, total| {
                            update_acquisition_progress_message(
                                &job_id,
                                done,
                                total,
                                "RAM dosyası indiriliyor",
                            );
                        },
                    ) {
                        Ok(download) => finish_acquisition_job_with_message(
                            &job_id,
                            json!({
                                "message": download.message,
                                "remote_job_id": ram_result.job_id,
                                "target_path": download.target_path,
                                "bytes_transferred": download.bytes_transferred,
                                "remote_bytes": ram_result.total_size,
                                "sha256": download.sha256.or(ram_result.sha256),
                            }),
                            "RAM edinimi tamamlandi",
                        ),
                        Err(err) => fail_acquisition_job_with_message(
                            &job_id,
                            err.to_string(),
                            "RAM dosyası indirilemedi",
                        ),
                    }
                }
                Err(err) => fail_acquisition_job_with_message(
                    &job_id,
                    err.to_string(),
                    "RAM edinimi basarisiz",
                ),
            }
        }
        Err(err) => {
            fail_acquisition_job_with_message(&job_id, err.to_string(), "RAM edinimi basarisiz")
        }
    }
}

fn acquisition_status_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct StatusRequest {
        job_id: String,
    }

    let request: StatusRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    match get_acquisition_job(&request.job_id) {
        Some(job) => json_ok(json!({
            "job_id": request.job_id,
            "status": job.status,
            "done": job.done,
            "total": job.total,
            "message": job.message,
            "result": job.result,
            "error": job.error,
        })),
        None => json_error(404, "acquisition job not found"),
    }
}

fn acquisition_control_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct ControlRequest {
        ip: Option<String>,
        port: Option<u16>,
        token: Option<String>,
        job_id: String,
        action: String,
    }

    let request: ControlRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    if request.job_id.trim().is_empty() {
        return json_error(400, "job_id is required");
    }
    if !matches!(request.action.as_str(), "pause" | "resume" | "stop") {
        return json_error(400, "action must be pause, resume, or stop");
    }

    let ip = request.ip.unwrap_or_default();
    let remote_control = !ip.trim().is_empty();

    if remote_control {
        let port = request.port.unwrap_or_default();
        if port == 0 {
            return json_error(400, "port is required");
        }
        match RemoteConnection::connect(&ip, port, request.token) {
            Ok(connection) => match connection.control_job(&request.job_id, &request.action) {
                Ok(message) => {
                    apply_local_acquisition_control(&request.job_id, &request.action);
                    json_ok(json!({
                        "job_id": request.job_id,
                        "action": request.action,
                        "message": message,
                    }))
                }
                Err(err) => json_error(500, err.to_string()),
            },
            Err(err) => json_error(500, err.to_string()),
        }
    } else {
        match apply_local_acquisition_control(&request.job_id, &request.action) {
            Some(message) => json_ok(json!({
                "job_id": request.job_id,
                "action": request.action,
                "message": message,
            })),
            None => json_error(404, "acquisition job not found"),
        }
    }
}

fn acquisition_jobs() -> &'static Mutex<HashMap<String, AcquisitionJob>> {
    ACQUISITION_JOBS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn create_acquisition_job(message: &str) -> (String, ram::CancellationToken) {
    let id = NEXT_ACQUISITION_JOB_ID.fetch_add(1, Ordering::SeqCst);
    let job_id = format!("acq-{id}");
    let control = ram::CancellationToken::default();
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

fn update_acquisition_progress(job_id: &str, done: u64, total: u64) {
    update_acquisition_progress_message(job_id, done, total, "Imaj alma sürüyor");
}

fn update_acquisition_progress_message(job_id: &str, done: u64, total: u64, label: &str) {
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

fn update_acquisition_message(job_id: &str, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.message = message.to_string();
    }
}

fn finish_acquisition_job_with_message(job_id: &str, result: Value, message: &str) {
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

fn fail_acquisition_job_with_message(job_id: &str, error: String, message: &str) {
    if let Ok(mut jobs) = acquisition_jobs().lock()
        && let Some(job) = jobs.get_mut(job_id)
    {
        job.status = "failed".to_string();
        job.message = message.to_string();
        job.error = Some(error);
    }
}

fn apply_local_acquisition_control(job_id: &str, action: &str) -> Option<String> {
    let mut jobs = acquisition_jobs().lock().ok()?;
    let job = jobs.get_mut(job_id)?;
    match action {
        "pause" => {
            job.control.pause();
            job.status = "paused".to_string();
            job.message = "Duraklatma komutu uygulandı".to_string();
            Some("Duraklatma komutu uygulandi".to_string())
        }
        "resume" => {
            job.control.resume();
            job.status = "running".to_string();
            job.message = "Devam komutu uygulandı".to_string();
            Some("Devam komutu uygulandi".to_string())
        }
        "stop" => {
            job.control.cancel();
            disk::cancel_disk_acquisition();
            job.message = "Durdurma komutu uygulandı".to_string();
            Some("Durdurma komutu uygulandi".to_string())
        }
        _ => None,
    }
}

fn get_acquisition_job(job_id: &str) -> Option<AcquisitionJob> {
    acquisition_jobs()
        .lock()
        .ok()
        .and_then(|jobs| jobs.get(job_id).cloned())
}

fn progress_percent(done: u64, total: u64) -> u64 {
    done.saturating_mul(100)
        .checked_div(total)
        .unwrap_or(0)
        .min(100)
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

fn local_ram_requires_elevation(tool: &str) -> bool {
    #[cfg(target_os = "linux")]
    {
        tool == "avml" && !process_is_root()
    }

    #[cfg(windows)]
    {
        tool == "winpmem" && !ram::is_root_or_admin()
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = tool;
        false
    }
}

fn local_ram_error_can_retry_elevated(message: &str) -> bool {
    if !(cfg!(target_os = "linux") || cfg!(windows)) {
        return false;
    }
    let message = message.to_ascii_lowercase();
    message.contains("root")
        || message.contains("administrator")
        || message.contains("permission denied")
        || message.contains("access is denied")
        || message.contains("erişim engellendi")
        || message.contains("yetkisiz")
        || message.contains("os error 13")
}

fn run_elevated_local_ram_job(
    job_id: &str,
    request: &LocalRamRequest,
    control: &ram::CancellationToken,
) {
    update_acquisition_message(job_id, "Yetki bekleniyor");
    let stem = helper_file_stem("worm-ram-helper");
    let request_path = std::env::temp_dir().join(format!("{stem}-request.json"));
    let result_path = std::env::temp_dir().join(format!("{stem}-result.json"));
    let progress_path = std::env::temp_dir().join(format!("{stem}-progress.json"));
    let control_path = std::env::temp_dir().join(format!("{stem}-control.json"));

    let request_json = json!({
        "output_file": &request.output,
        "tool": request.tool.as_deref().unwrap_or_default(),
        "tool_path": &request.tool_path,
        "owner_uid": helper_owner_uid(),
        "owner_gid": helper_owner_gid(),
    });
    if let Err(err) = write_json_file(&request_path, &request_json) {
        fail_acquisition_job_with_message(job_id, err, "RAM edinimi basarisiz");
        return;
    }
    if let Err(err) = write_helper_control_state(&control_path, "running") {
        cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
        fail_acquisition_job_with_message(job_id, err, "RAM edinimi basarisiz");
        return;
    }

    let args = vec![
        "ram-helper".to_string(),
        request_path.to_string_lossy().into_owned(),
        result_path.to_string_lossy().into_owned(),
        progress_path.to_string_lossy().into_owned(),
        control_path.to_string_lossy().into_owned(),
    ];
    let mut child = match spawn_elevated_helper(&args) {
        Ok(child) => child,
        Err(err) => {
            cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
            fail_acquisition_job_with_message(job_id, err, "RAM edinimi basarisiz");
            return;
        }
    };

    loop {
        if control.is_cancelled() {
            let _ = write_helper_control_state(&control_path, "cancelled");
            update_acquisition_message(job_id, "RAM edinimi iptal ediliyor");
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
                "RAM edinimi iptal edildi".to_string(),
                "RAM edinimi basarisiz",
            );
            return;
        }
        if control.is_paused() {
            let _ = write_helper_control_state(&control_path, "paused");
            update_acquisition_message(job_id, "RAM edinimi duraklatildi");
        } else {
            let _ = write_helper_control_state(&control_path, "running");
        }

        if let Some((done, total, message)) = read_helper_progress(&progress_path) {
            update_acquisition_progress_message(job_id, done, total, &message);
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let error = read_helper_error(&result_path).unwrap_or_else(|| {
                        "yetki yükseltme iptal edildi veya başarısız oldu".to_string()
                    });
                    cleanup_helper_files(&[
                        &request_path,
                        &result_path,
                        &progress_path,
                        &control_path,
                    ]);
                    fail_acquisition_job_with_message(job_id, error, "RAM edinimi basarisiz");
                    return;
                }
                break;
            }
            Ok(None) => thread::sleep(std::time::Duration::from_millis(500)),
            Err(err) => {
                cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
                fail_acquisition_job_with_message(job_id, err.to_string(), "RAM edinimi basarisiz");
                return;
            }
        }
    }

    let result = match read_helper_json(&result_path) {
        Ok(result) => result,
        Err(err) => {
            cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);
            fail_acquisition_job_with_message(job_id, err, "RAM edinimi basarisiz");
            return;
        }
    };
    cleanup_helper_files(&[&request_path, &result_path, &progress_path, &control_path]);

    if result.get("ok").and_then(Value::as_bool) == Some(true) {
        finish_acquisition_job_with_message(
            job_id,
            json!({
                "message": "RAM edinimi tamamlandi",
                "target_path": result.get("target_path").cloned().unwrap_or(Value::Null),
                "bytes_written": result.get("bytes_written").cloned().unwrap_or(Value::Null),
            }),
            "RAM edinimi tamamlandi",
        );
    } else {
        fail_acquisition_job_with_message(
            job_id,
            result
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("Yetkili RAM edinimi basarisiz")
                .to_string(),
            "RAM edinimi basarisiz",
        );
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

        if let Some((done, total, message)) = read_helper_progress(&progress_path) {
            update_acquisition_progress_message(job_id, done, total, &message);
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    let error = read_helper_error(&result_path).unwrap_or_else(|| {
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

fn helper_file_stem(prefix: &str) -> String {
    format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        Local::now().format("%Y%m%d%H%M%S%3f")
    )
}

fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

fn write_helper_control_state(path: &Path, state: &str) -> Result<(), String> {
    write_json_file(path, &json!({ "state": state }))
}

#[cfg(unix)]
fn helper_owner_uid() -> Option<u32> {
    Some(unsafe { libc::geteuid() })
}

#[cfg(not(unix))]
fn helper_owner_uid() -> Option<u32> {
    None
}

#[cfg(unix)]
fn helper_owner_gid() -> Option<u32> {
    Some(unsafe { libc::getegid() })
}

#[cfg(not(unix))]
fn helper_owner_gid() -> Option<u32> {
    None
}

fn read_helper_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

fn read_helper_error(path: &Path) -> Option<String> {
    read_helper_json(path).ok().and_then(|value| {
        value
            .get("error")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

fn read_helper_progress(path: &Path) -> Option<(u64, u64, String)> {
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

fn cleanup_helper_files(paths: &[&Path]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

#[cfg(target_os = "linux")]
fn process_is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(not(target_os = "linux"))]
fn process_is_root() -> bool {
    false
}

fn acquisition_target_path(source: &str, output: &str, prefix: &str) -> PathBuf {
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
    let file_name = format!(
        "{}_{}_{}.img",
        prefix,
        sanitize_file_stem(source_name),
        Local::now().format("%Y%m%d_%H%M%S")
    );
    output.join(file_name)
}

fn ram_remote_file_name(output: &str) -> String {
    Path::new(output)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| "memory_dump.raw".to_string())
}

fn sanitize_file_stem(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    sanitized.trim_matches('_').to_string()
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

fn remote_disks_endpoint(body: &[u8]) -> Response {
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

fn remote_tool_check_endpoint(body: &[u8]) -> Response {
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

#[derive(Deserialize)]
struct RemoteRequest {
    ip: String,
    port: u16,
    token: Option<String>,
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

fn pick_path_endpoint(directory: bool) -> Response {
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

fn write_response(mut stream: TcpStream, response: Response) -> Result<(), String> {
    let reason = match response.status {
        200 => "OK",
        204 => "No Content",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        499 => "Client Closed Request",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let headers = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: http://127.0.0.1\r\nAccess-Control-Allow-Headers: content-type\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nConnection: close\r\n\r\n",
        response.status,
        reason,
        response.content_type,
        response.body.len(),
    );
    stream
        .write_all(headers.as_bytes())
        .and_then(|_| stream.write_all(&response.body))
        .map_err(|err| err.to_string())
}

struct Response {
    status: u16,
    content_type: String,
    body: Vec<u8>,
}

impl Response {
    fn empty(status: u16) -> Self {
        Self {
            status,
            content_type: "text/plain; charset=utf-8".to_string(),
            body: Vec::new(),
        }
    }
}

fn json_ok(value: Value) -> Response {
    Response {
        status: 200,
        content_type: "application/json; charset=utf-8".to_string(),
        body: serde_json::to_vec(&value).unwrap_or_else(|_| b"{}".to_vec()),
    }
}

fn json_error(status: u16, message: impl Into<String>) -> Response {
    Response {
        status,
        content_type: "application/json; charset=utf-8".to_string(),
        body: serde_json::to_vec(&json!({
            "ok": false,
            "error": message.into(),
        }))
        .unwrap_or_else(|_| b"{\"ok\":false}".to_vec()),
    }
}

fn ui_root() -> PathBuf {
    if let Some(path) = std::env::var_os("WORM_UI_ROOT") {
        let path = PathBuf::from(path);
        if path.join("index.html").exists() {
            return path;
        }
    }

    if let Ok(exe) = std::env::current_exe()
        && let Some(bin_dir) = exe.parent()
        && let Some(prefix) = bin_dir.parent()
    {
        let packaged = prefix.join("share").join("worm").join("ui");
        if packaged.join("index.html").exists() {
            return packaged;
        }
    }

    PathBuf::from(DEV_UI_ROOT)
}

fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "css" => "text/css; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "svg" => "image/svg+xml",
        "ttf" => "font/ttf",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}

fn percent_decode(input: &str) -> Result<String, ()> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' if index + 2 < bytes.len() => {
                let high = hex_value(bytes[index + 1]).ok_or(())?;
                let low = hex_value(bytes[index + 2]).ok_or(())?;
                out.push((high << 4) | low);
                index += 3;
            }
            b'+' => {
                out.push(b' ');
                index += 1;
            }
            byte => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(out).map_err(|_| ())
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response_json(response: Response) -> Value {
        assert_eq!(response.status, 200);
        serde_json::from_slice(&response.body).unwrap()
    }

    #[test]
    fn evidence_note_and_report_endpoints_write_files() {
        let dir = tempfile::tempdir().unwrap();
        *test_case_base_dir().lock().unwrap() = Some(dir.path().to_path_buf());
        *current_evidence_case().lock().unwrap() = None;

        let create = response_json(evidence_create_endpoint(
            json!({
                "case_name": "case_endpoint",
            })
            .to_string()
            .as_bytes(),
        ));
        assert!(Path::new(create["case_dir"].as_str().unwrap()).is_dir());

        let note = response_json(evidence_add_note_endpoint(
            json!({ "note": "endpoint note" }).to_string().as_bytes(),
        ));
        assert!(Path::new(note["path"].as_str().unwrap()).is_file());

        let report = response_json(report_create_endpoint(
            json!({
                "title": "Endpoint Report",
                "description": "Created by endpoint smoke test.",
                "format": "json",
            })
            .to_string()
            .as_bytes(),
        ));
        assert!(Path::new(report["path"].as_str().unwrap()).is_file());

        *current_evidence_case().lock().unwrap() = None;
        *test_case_base_dir().lock().unwrap() = None;
    }

    #[test]
    fn directory_tree_json_returns_limited_children() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("a")).unwrap();
        fs::write(dir.path().join("a").join("b.txt"), b"hello").unwrap();

        let tree = directory_tree_json(dir.path(), 2, 10);
        assert!(tree["is_dir"].as_bool().unwrap());
        assert_eq!(tree["children"].as_array().unwrap().len(), 1);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn disk_list_requests_elevation_for_linux_user_session() {
        if process_is_root() {
            return;
        }

        let disks = vec![disk::DiskInfo {
            device: PathBuf::from("/dev/sda"),
            total_size: 1024,
            used_size: 1024,
            accessible: true,
        }];

        assert!(should_request_elevated_disk_list(&disks));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn local_ram_requests_elevation_for_linux_user_session() {
        if process_is_root() {
            return;
        }

        assert!(local_ram_requires_elevation("avml"));
        assert!(!local_ram_requires_elevation("winpmem"));
    }
}
