//! delil kasası ve dosya listesi api rotaları.

use crate::api::{
    current_evidence_case, current_evidence_vault, default_case_base_dir, evidence_subdir,
    report_evidence_vault, sanitize_case_name, set_current_evidence_case,
};
use crate::evidence::{EvidenceVault, relative_case_path};
use crate::report::{self, ReportFormat, ReportInfo};
use crate::server::{Response, json_error, json_ok, json_serialize};
use chrono::Local;
use serde::Deserialize;
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};

/// Yeni vaka klasörü oluşturur ve aktif vakayı günceller.
pub fn evidence_create_endpoint(body: &[u8]) -> Response {
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
            set_current_evidence_case(base_dir, case_name.clone());
            let _ = crate::profile::record_active_profile_activity(
                "case",
                "create",
                Some(&case_name),
                Some(&format!("Vaka oluşturuldu: {}", case_name)),
            );
            json_serialize(&summary)
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Aktif veya seçili vakaya metin notu ekler.
pub fn evidence_add_note_endpoint(body: &[u8]) -> Response {
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
        Ok(path) => {
            let _ = crate::profile::record_active_profile_activity(
                "case",
                "add_note",
                Some(&vault.case_name),
                Some("Vakaya adli not eklendi"),
            );
            json_ok(json!({ "path": path }))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Aktif vaka alt klasöründeki dosyaları listeler.
pub fn evidence_list_files_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct EvidenceListRequest {
        subdir: Option<String>,
    }

    let request: EvidenceListRequest = if body.is_empty() {
        EvidenceListRequest { subdir: None }
    } else {
        match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(err) => return json_error(400, err.to_string()),
        }
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

/// Aktif vaka için dosya sayılarını döndürür.
pub fn evidence_summary_endpoint() -> Response {
    let vault = match current_evidence_vault() {
        Ok(vault) => vault,
        Err(response) => return response,
    };
    match vault.summary() {
        Ok(summary) => json_ok(json!({
            "case_name": summary.case_name,
            "case_dir": summary.case_dir,
            "created_by": summary.created_by,
            "created_by_name": summary.created_by_name,
            "output_count": summary.output_count,
            "android_count": summary.android_count,
            "ios_count": summary.ios_count,
            "docker_count": summary.docker_count,
            "hash_count": summary.hash_count,
            "report_count": summary.report_count,
            "manifest_path": summary.manifest_path,
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Seçili veya aktif vaka için bütünlük manifesti üretir.
pub fn evidence_manifest_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct EvidenceManifestRequest {
        case_name: Option<String>,
    }

    let request: EvidenceManifestRequest = if body.is_empty() {
        EvidenceManifestRequest { case_name: None }
    } else {
        match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(err) => return json_error(400, err.to_string()),
        }
    };

    let vault = match report_evidence_vault(request.case_name.as_deref()) {
        Ok(vault) => vault,
        Err(response) => return response,
    };

    match vault.write_case_manifest() {
        Ok(path) => {
            let manifest = fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str::<Value>(&content).ok());
            json_ok(json!({
                "path": path,
                "manifest": manifest,
            }))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Android ve iOS edinim manifestlerinden vaka geçmişini üretir.
pub fn acquisition_history_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct AcquisitionHistoryRequest {
        case_name: Option<String>,
    }

    let request: AcquisitionHistoryRequest = if body.is_empty() {
        AcquisitionHistoryRequest { case_name: None }
    } else {
        match serde_json::from_slice(body) {
            Ok(request) => request,
            Err(err) => return json_error(400, err.to_string()),
        }
    };

    let vault = match report_evidence_vault(request.case_name.as_deref()) {
        Ok(vault) => vault,
        Err(response) => return response,
    };
    let history = acquisition_history_for_vault(&vault);
    json_ok(json!({
        "case_name": &vault.case_name,
        "case_dir": &vault.case_dir,
        "history": history,
    }))
}

/// Varsayılan vaka klasöründeki tüm vakaları listeler.
pub fn evidence_cases_endpoint() -> Response {
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
                "ram_dir": case_dir.join("ram"),
                "android_dir": case_dir.join("android"),
                "ios_dir": case_dir.join("ios"),
                "docker_dir": case_dir.join("docker"),
            })
        });

    json_ok(json!({
        "base_dir": base_dir,
        "cases": cases,
        "current_case": current,
    }))
}

/// Seçili vaka için TXT veya JSON rapor oluşturur.
pub fn report_create_endpoint(body: &[u8]) -> Response {
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
        .unwrap_or("Amele Forensic Tool (https://amele.noirlang.tr)");
    let hash_sha256 = request
        .hash_sha256
        .as_deref()
        .map(str::trim)
        .unwrap_or_default();

    let creator = crate::profile::active_profile()
        .map(|p| {
            if !p.full_name.trim().is_empty() {
                format!("{} ({})", p.full_name, p.username)
            } else {
                p.username
            }
        })
        .or_else(|| std::env::var("USER").ok())
        .or_else(|| std::env::var("USERNAME").ok())
        .unwrap_or_else(|| "amele".to_string());

    let info = ReportInfo {
        title: title.to_string(),
        description: description.to_string(),
        creator,
        source: source.to_string(),
        hash_sha256: hash_sha256.to_string(),
        date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    };
    let target = vault
        .reports_dir
        .join(report::new_report_file_name(&vault.case_name, format));

    match report::create_report(&info, format, &target, Some(&vault)) {
        Ok(path) => {
            let _ = crate::profile::record_active_profile_activity(
                "report",
                "create",
                Some(&vault.case_name),
                Some(&format!("Adli rapor oluşturuldu: {}", target.display())),
            );
            let manifest_path = vault.write_case_manifest().ok();
            json_ok(json!({
                "path": path,
                "manifest_path": manifest_path,
            }))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

struct CaseArtifactsData {
    output_count: usize,
    ram_count: usize,
    android_count: usize,
    ios_count: usize,
    docker_count: usize,
    hash_count: usize,
    report_count: usize,
    artifacts: Value,
}

/// Vaka içindeki edinim türlerini platform ve türe göre gruplayıp sayar.
fn case_artifacts_summary(case_dir: &Path) -> CaseArtifactsData {
    let mut linux_disk = 0usize;
    let mut windows_disk = 0usize;
    #[allow(unused_mut)]
    let mut other_disk = 0usize;
    let mut linux_ram = 0usize;
    let mut windows_ram = 0usize;
    #[allow(unused_mut)]
    let mut other_ram = 0usize;

    let mut output_count = 0usize;
    if let Ok(entries) = fs::read_dir(case_dir.join("ciktilar")) {
        for entry in entries.flatten() {
            output_count += 1;
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if name.ends_with(".sha256") || name.ends_with(".txt") || name.ends_with(".log") {
                    continue;
                }
                if name.contains("linux")
                    || name.starts_with("sd")
                    || name.starts_with("nvme")
                    || name.starts_with("mmc")
                    || name.contains("loop")
                {
                    linux_disk += 1;
                } else if name.contains("win") || name.contains("physicaldrive") {
                    windows_disk += 1;
                } else {
                    #[cfg(unix)]
                    {
                        linux_disk += 1;
                    }
                    #[cfg(windows)]
                    {
                        windows_disk += 1;
                    }
                    #[cfg(not(any(unix, windows)))]
                    {
                        other_disk += 1;
                    }
                }
            }
        }
    }

    let mut ram_count = 0usize;
    if let Ok(entries) = fs::read_dir(case_dir.join("ram")) {
        for entry in entries.flatten() {
            ram_count += 1;
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if name.ends_with(".sha256") || name.ends_with(".txt") || name.ends_with(".log") {
                    continue;
                }
                if name.contains("win") || name.contains("pmem") {
                    windows_ram += 1;
                } else if name.contains("linux") || name.contains("avml") || name.contains("lime") {
                    linux_ram += 1;
                } else {
                    #[cfg(unix)]
                    {
                        linux_ram += 1;
                    }
                    #[cfg(windows)]
                    {
                        windows_ram += 1;
                    }
                    #[cfg(not(any(unix, windows)))]
                    {
                        other_ram += 1;
                    }
                }
            }
        }
    }

    let android_count = count_directory_entries(&case_dir.join("android"));
    let ios_count = count_directory_entries(&case_dir.join("ios"));
    let docker_count = count_directory_entries(&case_dir.join("docker"));
    let hash_count = count_directory_entries(&case_dir.join("hash"));
    let report_count = count_directory_entries(&case_dir.join("raporlar"));

    let artifacts = json!({
        "linux_disk": linux_disk,
        "windows_disk": windows_disk,
        "other_disk": other_disk,
        "linux_ram": linux_ram,
        "windows_ram": windows_ram,
        "other_ram": other_ram,
        "android": android_count,
        "ios": ios_count,
        "docker": docker_count,
        "hash": hash_count,
        "report": report_count,
    });

    CaseArtifactsData {
        output_count,
        ram_count,
        android_count,
        ios_count,
        docker_count,
        hash_count,
        report_count,
        artifacts,
    }
}

/// Tek vaka klasörünü API listeleme JSON'una dönüştürür.
fn case_listing_json(case_name: &str, case_dir: &Path) -> Value {
    let metadata = crate::evidence::read_case_metadata(case_dir);
    let data = case_artifacts_summary(case_dir);
    json!({
        "case_name": case_name,
        "case_dir": case_dir,
        "output_dir": case_dir.join("ciktilar"),
        "ram_dir": case_dir.join("ram"),
        "android_dir": case_dir.join("android"),
        "ios_dir": case_dir.join("ios"),
        "docker_dir": case_dir.join("docker"),
        "created_by": metadata.created_by,
        "created_by_name": metadata.created_by_name,
        "created_at": metadata.created_at,
        "output_count": data.output_count,
        "ram_count": data.ram_count,
        "android_count": data.android_count,
        "ios_count": data.ios_count,
        "docker_count": data.docker_count,
        "hash_count": data.hash_count,
        "report_count": data.report_count,
        "manifest_path": case_dir.join("case_manifest.json"),
        "artifacts": data.artifacts,
    })
}

/// Klasördeki doğrudan girdi sayısını döndürür.
fn count_directory_entries(path: &Path) -> usize {
    fs::read_dir(path)
        .map(|entries| entries.flatten().count())
        .unwrap_or_default()
}

/// Dosya/klasör yolunu arayüzün beklediği JSON formata çevirir.
fn file_entry_json(path: PathBuf) -> Value {
    let metadata = fs::metadata(&path).ok();
    json!({
        "name": path.file_name().and_then(|name| name.to_str()).unwrap_or_default(),
        "path": path,
        "is_dir": metadata.as_ref().map(|meta| meta.is_dir()).unwrap_or(false),
        "size": metadata.as_ref().map(|meta| meta.len()).unwrap_or_default(),
    })
}

/// Rapor formatı stringini enum değerine çevirir.
fn report_format(value: &str) -> Option<ReportFormat> {
    match value.trim().to_ascii_lowercase().as_str() {
        "txt" => Some(ReportFormat::Txt),
        "json" => Some(ReportFormat::Json),
        _ => None,
    }
}

fn acquisition_history_for_vault(vault: &EvidenceVault) -> Vec<Value> {
    let mut items = Vec::new();
    collect_android_history(&vault.android_dir, &mut items);
    collect_ios_history(&vault.ios_dir, &mut items);
    collect_docker_history(&vault.docker_dir, &mut items);
    items.sort_by(|left, right| {
        right["sort_key"]
            .as_str()
            .unwrap_or_default()
            .cmp(left["sort_key"].as_str().unwrap_or_default())
    });
    items
}

fn collect_android_history(android_dir: &Path, items: &mut Vec<Value>) {
    let mut manifests = Vec::new();
    collect_named_files_recursive(android_dir, "android_manifest.json", &mut manifests);
    for manifest_path in manifests {
        let Some(item) = android_history_item(android_dir, &manifest_path) else {
            continue;
        };
        items.push(item);
    }
}

fn collect_ios_history(ios_dir: &Path, items: &mut Vec<Value>) {
    let mut manifests = Vec::new();
    collect_named_files_recursive(ios_dir, "ios_manifest.json", &mut manifests);
    for manifest_path in manifests {
        let Some(item) = ios_history_item(ios_dir, &manifest_path) else {
            continue;
        };
        items.push(item);
    }
}

fn collect_docker_history(docker_dir: &Path, items: &mut Vec<Value>) {
    let mut manifests = Vec::new();
    collect_named_files_recursive(docker_dir, "docker_metadata.json", &mut manifests);
    for meta_path in manifests {
        let Some(item) = docker_history_item(docker_dir, &meta_path) else {
            continue;
        };
        items.push(item);
    }
}

fn docker_history_item(docker_dir: &Path, meta_path: &Path) -> Option<Value> {
    let content = fs::read_to_string(meta_path).ok()?;
    let meta: Value = serde_json::from_str(&content).ok()?;
    let container_id = meta
        .get("konteyner_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let name = meta
        .get("isim")
        .and_then(|v| v.as_str())
        .unwrap_or("container");
    let time_str = meta
        .get("edinim_zamani")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let parent_dir = meta_path.parent()?;
    let rel_dir = relative_case_path(docker_dir, parent_dir);
    let short_id = if container_id.len() >= 12 {
        &container_id[..12]
    } else {
        container_id
    };
    let image = meta
        .get("config_v2")
        .and_then(|c| c.get("Config"))
        .and_then(|c| c.get("Image"))
        .and_then(|i| i.as_str())
        .unwrap_or("-");
    let bytes = meta.get("boyut").and_then(|v| v.as_u64()).unwrap_or(0);

    Some(json!({
        "id": format!("docker_{}", short_id),
        "platform": "docker",
        "title": format!("Docker: {} ({})", name, short_id),
        "subtitle": format!("İmaj: {}", image),
        "generated_at": time_str,
        "timestamp": time_str,
        "sort_key": time_str,
        "status": "completed",
        "total_bytes": bytes,
        "folder": parent_dir.to_string_lossy().to_string(),
        "relative_folder": rel_dir,
        "output_dir": parent_dir.to_string_lossy().to_string(),
        "relative_output": rel_dir,
        "manifest_path": meta_path.to_string_lossy().to_string(),
        "summary": {
            "container_id": container_id,
            "container_name": name,
            "image": image,
        }
    }))
}

fn android_history_item(android_dir: &Path, manifest_path: &Path) -> Option<Value> {
    let content = fs::read_to_string(manifest_path).ok()?;
    let manifest: Value = serde_json::from_str(&content).ok()?;
    let session = manifest.get("session").unwrap_or(&Value::Null);
    let profile = session.get("device_profile").unwrap_or(&Value::Null);
    let artifacts = manifest
        .get("artifacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let errors = manifest
        .get("errors")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or_default();
    let failed = artifacts
        .iter()
        .filter(|artifact| artifact.get("success").and_then(Value::as_bool) == Some(false))
        .count();
    let success = artifacts.len().saturating_sub(failed);
    let output_dir = manifest_path.parent().unwrap_or(android_dir);
    let relative_output = relative_path_string(android_dir, output_dir);
    let generated_at = json_string(&manifest, "generated_at")
        .or_else(|| json_string(session, "created_at"))
        .unwrap_or_else(|| file_modified_string(manifest_path));

    Some(json!({
        "id": format!("android:{relative_output}"),
        "platform": "android",
        "kind": json_string(&manifest, "acquisition_type").unwrap_or_else(|| "android".to_string()),
        "title": android_history_title(&manifest),
        "subtitle": android_device_label(session, profile),
        "generated_at": generated_at,
        "sort_key": file_modified_string(manifest_path),
        "output_dir": output_dir,
        "relative_output": relative_output,
        "manifest_path": manifest_path,
        "total_bytes": manifest.get("total_bytes").and_then(Value::as_u64).unwrap_or_default(),
        "success_count": success,
        "error_count": errors + failed,
        "status": if errors + failed == 0 { "completed" } else { "warnings" },
        "serial": json_string(session, "serial"),
        "model": json_string(profile, "model"),
        "manifest_sha256": json_string(&manifest, "acquisition_sha256"),
    }))
}

fn ios_history_item(ios_dir: &Path, manifest_path: &Path) -> Option<Value> {
    let content = fs::read_to_string(manifest_path).ok()?;
    let manifest: Value = serde_json::from_str(&content).ok()?;
    let device = manifest.get("device").unwrap_or(&Value::Null);
    let backup = manifest.get("backup").unwrap_or(&Value::Null);
    let summary = manifest.get("summary").unwrap_or(&Value::Null);
    let output_dir = manifest_path.parent().unwrap_or(ios_dir);
    let relative_output = relative_path_string(ios_dir, output_dir);
    let generated_at =
        json_string(&manifest, "created_at").unwrap_or_else(|| file_modified_string(manifest_path));
    let errors = json_usize(summary, "errors");
    let missing = json_usize(summary, "missing");

    Some(json!({
        "id": format!("ios:{relative_output}"),
        "platform": "ios",
        "kind": "ios_backup_normalize",
        "title": "iOS backup normalizasyonu",
        "subtitle": ios_device_label(device),
        "generated_at": generated_at,
        "sort_key": file_modified_string(manifest_path),
        "output_dir": output_dir,
        "relative_output": relative_output,
        "manifest_path": manifest_path,
        "total_bytes": summary.get("total_bytes").and_then(Value::as_u64).unwrap_or_default(),
        "total_entries": summary.get("total_entries").and_then(Value::as_u64).unwrap_or_default(),
        "files_copied": summary.get("files_copied").and_then(Value::as_u64).unwrap_or_default(),
        "success_count": summary.get("files_copied").and_then(Value::as_u64).unwrap_or_default(),
        "error_count": errors + missing,
        "status": if errors + missing == 0 { "completed" } else { "warnings" },
        "serial": json_string(device, "serial_number"),
        "model": json_string(device, "model"),
        "ios_version": json_string(device, "ios_version"),
        "encrypted": backup.get("encrypted").and_then(Value::as_bool).unwrap_or(false),
    }))
}

fn collect_named_files_recursive(dir: &Path, file_name: &str, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_named_files_recursive(&path, file_name, files);
        } else if path.file_name().and_then(|name| name.to_str()) == Some(file_name) {
            files.push(path);
        }
    }
}

fn android_history_title(manifest: &Value) -> String {
    match json_string(manifest, "acquisition_type")
        .unwrap_or_default()
        .as_str()
    {
        "android_logical" => "Android mantiksal edinim".to_string(),
        "android_filesystem" => "Android dosya sistemi edinimi".to_string(),
        "android_ram" => "Android RAM edinimi".to_string(),
        value if !value.is_empty() => value.replace('_', " "),
        _ => "Android edinimi".to_string(),
    }
}

fn android_device_label(session: &Value, profile: &Value) -> String {
    let model = json_string(profile, "model")
        .or_else(|| json_string(profile, "product"))
        .unwrap_or_else(|| "Android cihaz".to_string());
    let serial = json_string(session, "serial").unwrap_or_else(|| "-".to_string());
    format!("{model} | {serial}")
}

fn ios_device_label(device: &Value) -> String {
    let model = json_string(device, "model")
        .or_else(|| json_string(device, "product_type"))
        .unwrap_or_else(|| "iOS cihaz".to_string());
    let version = json_string(device, "ios_version")
        .map(|value| format!("iOS {value}"))
        .unwrap_or_else(|| "iOS".to_string());
    format!("{model} | {version}")
}

fn relative_path_string(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn file_modified_string(path: &Path) -> String {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .ok()
        .map(|time| {
            chrono::DateTime::<Local>::from(time)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_default()
}

fn json_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|item| !item.trim().is_empty())
}

fn json_usize(value: &Value, key: &str) -> usize {
    value
        .get(key)
        .and_then(Value::as_u64)
        .and_then(|value| value.try_into().ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_format() {
        assert_eq!(report_format("txt"), Some(ReportFormat::Txt));
        assert_eq!(report_format("TXT"), Some(ReportFormat::Txt));
        assert_eq!(report_format("json"), Some(ReportFormat::Json));
        assert_eq!(report_format("  Json  "), Some(ReportFormat::Json));
        assert_eq!(report_format("pdf"), None);
        assert_eq!(report_format(""), None);
    }

    #[test]
    fn test_evidence_create_endpoint_validation() {
        let resp = evidence_create_endpoint(b"");
        assert_eq!(resp.status, 400);

        let resp = evidence_create_endpoint(b"invalid");
        assert_eq!(resp.status, 400);

        let resp = evidence_create_endpoint(br#"{"case_name": ""}"#);
        assert_eq!(resp.status, 400);

        let resp = evidence_create_endpoint(br#"{"case_name": "   "}"#);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_evidence_add_note_endpoint_validation() {
        let resp = evidence_add_note_endpoint(b"");
        assert_eq!(resp.status, 400);

        let resp = evidence_add_note_endpoint(br#"{"note": ""}"#);
        assert_eq!(resp.status, 400);

        let resp = evidence_add_note_endpoint(br#"{"note": "   "}"#);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_report_create_endpoint_validation() {
        let resp = report_create_endpoint(b"not json");
        assert_eq!(resp.status, 400);

        let resp = report_create_endpoint(br#"{"format": "html"}"#);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_evidence_cases_endpoint_returns_ok() {
        let resp = evidence_cases_endpoint();
        assert_eq!(resp.status, 200);
        let val: Value = serde_json::from_slice(&resp.body).unwrap();
        assert!(val.get("base_dir").is_some());
        assert!(val.get("cases").is_some());
    }

    #[test]
    fn test_case_artifacts_summary() {
        let dir = tempfile::tempdir().unwrap();
        let case_dir = dir.path().join("test_case");
        fs::create_dir_all(case_dir.join("ciktilar")).unwrap();
        fs::create_dir_all(case_dir.join("ram")).unwrap();
        fs::create_dir_all(case_dir.join("docker")).unwrap();

        fs::write(case_dir.join("ciktilar").join("sda.dd"), b"disk data").unwrap();
        fs::write(case_dir.join("ram").join("lime.dump"), b"ram data").unwrap();
        fs::write(case_dir.join("docker").join("docker_metadata.json"), b"{}").unwrap();

        let data = case_artifacts_summary(&case_dir);
        assert_eq!(data.output_count, 1);
        assert_eq!(data.ram_count, 1);
        assert_eq!(data.docker_count, 1);
        assert_eq!(data.artifacts["linux_disk"], 1);
        assert_eq!(data.artifacts["linux_ram"], 1);
    }

    #[test]
    fn builds_acquisition_history_from_android_and_ios_manifests() {
        let dir = tempfile::tempdir().unwrap();
        let vault = EvidenceVault::create(dir.path(), "history_case").unwrap();

        let android_run = vault.android_dir.join("logical_phone_20260727");
        fs::create_dir_all(&android_run).unwrap();
        fs::write(
            android_run.join("android_manifest.json"),
            serde_json::to_string_pretty(&json!({
                "acquisition_type": "android_logical",
                "generated_at": "2026-07-27T10:00:00+03:00",
                "session": {
                    "serial": "R5C123",
                    "created_at": "2026-07-27T10:00:00+03:00",
                    "device_profile": {
                        "model": "Pixel 8",
                        "product": "pixel"
                    }
                },
                "artifacts": [
                    { "success": true },
                    { "success": false }
                ],
                "total_bytes": 100,
                "errors": ["one failed"]
            }))
            .unwrap(),
        )
        .unwrap();

        let ios_run = vault.ios_dir.join("iphone_20260727");
        fs::create_dir_all(&ios_run).unwrap();
        fs::write(
            ios_run.join("ios_manifest.json"),
            serde_json::to_string_pretty(&json!({
                "created_at": "2026-07-27 11:00:00",
                "device": {
                    "model": "iPhone 15",
                    "ios_version": "18.5"
                },
                "backup": { "encrypted": false },
                "summary": {
                    "total_entries": 10,
                    "files_copied": 9,
                    "missing": 1,
                    "errors": 0,
                    "total_bytes": 200
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let docker_run = vault.docker_dir.join("web_nginx_a1b2c3d4e5f6");
        fs::create_dir_all(&docker_run).unwrap();
        fs::write(
            docker_run.join("docker_metadata.json"),
            serde_json::to_string_pretty(&json!({
                "edinim_zamani": "2026-07-27T12:00:00+03:00",
                "konteyner_id": "a1b2c3d4e5f67890abcdef123456",
                "isim": "web_nginx",
                "config_v2": {
                    "Config": { "Image": "nginx:alpine" }
                }
            }))
            .unwrap(),
        )
        .unwrap();

        let history = acquisition_history_for_vault(&vault);
        assert_eq!(history.len(), 3);
        assert!(history.iter().any(|item| item["platform"] == "android"));
        assert!(history.iter().any(|item| item["platform"] == "ios"));
        assert!(history.iter().any(|item| item["platform"] == "docker"));
    }
}
