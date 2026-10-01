//! android manifest xml dosyalarını ayrıştıran yer.

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

    let _ = std::fs::create_dir_all(output_dir);
    let content = serde_json::to_string_pretty(manifest)
        .map_err(|err| format!("Android manifest olusturulamadi: {err}"))?;
    let path = output_dir.join("android_manifest.json");
    std::fs::write(&path, &content).map_err(|err| format!("Android manifest yazilamadi: {err}"))?;

    let hash = crate::hash::to_hex(&Sha256::digest(content.as_bytes()));
    let sidecar = output_dir.join("android_manifest.json.sha256");
    let _ = std::fs::write(&sidecar, format!("{hash}  android_manifest.json\n"));
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::android::session::{AndroidTransport, AndroidTransportKind};

    fn dummy_session() -> AndroidSession {
        AndroidSession {
            serial: "test_device_1".to_string(),
            created_at: "now".to_string(),
            adb_state: Some("device".to_string()),
            connected: true,
            transport: AndroidTransport {
                kind: AndroidTransportKind::Usb,
                label: "USB".to_string(),
            },
            device_profile: crate::android::profile::AndroidDeviceProfile {
                serial: "test_device_1".to_string(),
                product: Some("pixel8".to_string()),
                model: Some("Pixel 8".to_string()),
                device: Some("shiba".to_string()),
                abi: Some("arm64-v8a".to_string()),
                api_level: Some(34),
                build: Some("UQ1A.240205.004".to_string()),
                fingerprint: None,
                security_patch: Some("2024-02-05".to_string()),
                selinux: Some("Enforcing".to_string()),
                encryption: Some("file".to_string()),
                kernel_version: None,
                is_rooted: false,
                adb_root: false,
                su_available: false,
            },
        }
    }

    fn dummy_capabilities() -> AndroidCapabilityReport {
        let check = crate::android::capability::AndroidCapabilityCheck {
            level: crate::android::capability::AndroidCapabilityLevel::Supported,
            available: true,
            reason: "ok".to_string(),
            recommendation: None,
        };
        AndroidCapabilityReport {
            serial: "test_device_1".to_string(),
            generated_at: "now".to_string(),
            adb_authorized: check.clone(),
            logical_acquisition: check.clone(),
            shared_storage: check.clone(),
            bugreport: check.clone(),
            adb_backup: check.clone(),
            filesystem_non_root: check.clone(),
            filesystem_root: check.clone(),
            volatile_memory: check.clone(),
            process_memory_root: check.clone(),
            physical_memory_probe: check.clone(),
            lemon_physical_memory: check.clone(),
            remote_mesh_transport: check,
        }
    }

    #[test]
    fn test_manifest_creation_and_writing() {
        let temp_dir =
            std::env::temp_dir().join(format!("amele_manifest_test_{}", std::process::id()));
        let session = dummy_session();
        let capabilities = dummy_capabilities();

        let artifact_path = temp_dir.join("userdata.img");
        let manifest = manifest_from_single_artifact(
            "filesystem",
            &session,
            &capabilities,
            "filesystem_image",
            &artifact_path,
            1024,
            "dummy_sha256_hash",
        );

        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.total_bytes, 1024);
        assert_eq!(manifest.artifacts.len(), 1);
        assert_eq!(manifest.artifacts[0].file_name, "userdata.img");

        let hash_res = write_android_manifest(&temp_dir, &manifest);
        assert!(hash_res.is_ok());

        let manifest_file = temp_dir.join("android_manifest.json");
        let sidecar_file = temp_dir.join("android_manifest.json.sha256");
        assert!(manifest_file.is_file());
        assert!(sidecar_file.is_file());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
