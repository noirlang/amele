//! Android edinim çıktıları için ortak manifest üretir.
use super::capability::AndroidCapabilityReport;
use super::logical::AcquisitionItem;
use super::session::AndroidSession;
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
/// Manifest içinde tek bir dosya, klasör veya başarısız adımı temsil eder.
pub struct AndroidManifestArtifact {
    pub category: String,
    pub file_name: String,
    pub path: Option<PathBuf>,
    pub size: u64,
    pub sha256: Option<String>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
/// Android ediniminin oturum, kabiliyet ve çıktı özetini tek JSON içinde toplar.
pub struct AndroidAcquisitionManifest {
    pub schema_version: u32,
    pub acquisition_type: String,
    pub generated_at: String,
    pub session: AndroidSession,
    pub capabilities: AndroidCapabilityReport,
    pub artifacts: Vec<AndroidManifestArtifact>,
    pub total_bytes: u64,
    pub acquisition_sha256: Option<String>,
    pub errors: Vec<String>,
}

/// Mantıksal edinim adımlarından ortak Android manifest modelini üretir.
pub fn manifest_from_logical_items(
    acquisition_type: &str,
    session: &AndroidSession,
    capabilities: &AndroidCapabilityReport,
    output_dir: &Path,
    items: &[AcquisitionItem],
    total_bytes: u64,
    acquisition_sha256: Option<String>,
    errors: &[String],
) -> AndroidAcquisitionManifest {
    let artifacts = items
        .iter()
        .map(|item| AndroidManifestArtifact {
            category: item.category.clone(),
            file_name: item.file_name.clone(),
            path: Some(output_dir.join(&item.file_name)),
            size: item.size,
            sha256: None,
            success: item.success,
            error: item.error.clone(),
        })
        .collect();

    AndroidAcquisitionManifest {
        schema_version: 1,
        acquisition_type: acquisition_type.to_string(),
        generated_at: chrono::Local::now().to_rfc3339(),
        session: session.clone(),
        capabilities: capabilities.clone(),
        artifacts,
        total_bytes,
        acquisition_sha256,
        errors: errors.to_vec(),
    }
}

/// Tek dosyalı filesystem veya RAM edinimleri için ortak manifest modelini üretir.
pub fn manifest_from_single_artifact(
    acquisition_type: &str,
    session: &AndroidSession,
    capabilities: &AndroidCapabilityReport,
    category: &str,
    output_file: &Path,
    size: u64,
    sha256: &str,
) -> AndroidAcquisitionManifest {
    let file_name = output_file
        .file_name()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "artifact".to_string());

    AndroidAcquisitionManifest {
        schema_version: 1,
        acquisition_type: acquisition_type.to_string(),
        generated_at: chrono::Local::now().to_rfc3339(),
        session: session.clone(),
        capabilities: capabilities.clone(),
        artifacts: vec![AndroidManifestArtifact {
            category: category.to_string(),
            file_name,
            path: Some(output_file.to_path_buf()),
            size,
            sha256: Some(sha256.to_string()),
            success: true,
            error: None,
        }],
        total_bytes: size,
        acquisition_sha256: Some(sha256.to_string()),
        errors: Vec::new(),
    }
}

/// Ortak Android manifestini yazar ve manifest JSON'unun SHA-256 değerini döndürür.
pub fn write_android_manifest(
    output_dir: &Path,
    manifest: &AndroidAcquisitionManifest,
) -> Result<String, String> {
    use sha2::{Digest, Sha256};

    let content = serde_json::to_string_pretty(manifest)
        .map_err(|err| format!("Android manifest olusturulamadi: {err}"))?;
    let path = output_dir.join("android_manifest.json");
    std::fs::write(&path, &content).map_err(|err| format!("Android manifest yazilamadi: {err}"))?;

    let hash = crate::hash::to_hex(&Sha256::digest(content.as_bytes()));
    let sidecar = output_dir.join("android_manifest.json.sha256");
    let _ = std::fs::write(&sidecar, format!("{hash}  android_manifest.json\n"));
    Ok(hash)
}
