//! Android edinim adımlarını profil, dosya sistemi ve RAM akışları için koordine eder.
use super::capability::write_android_capability_report;
use super::manifest::{
    manifest_from_logical_items, manifest_from_single_artifact, write_android_manifest,
};
use super::session::write_android_session;
use super::{
    AcquisitionItem, AndroidAcquisitionProfile, AndroidCapabilityReport, AndroidDeviceProfile,
    AndroidRamAcquisitionResult, AndroidRamMode, AndroidSession, FilesystemAcquisitionResult,
    LogicalAcquisitionResult, build_android_capability_report, build_android_session,
    detect_device_profile, filesystem_acquisition, logical_acquisition_with_profile,
    logical_steps_for_profile, ram_acquisition_with_mode,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
/// Profil destekli Android mantıksal ediniminin cihaz profiliyle birleştirilmiş sonucudur.
pub struct AndroidOrchestratedAcquisitionResult {
    pub profile: AndroidAcquisitionProfile,
    pub device_profile: AndroidDeviceProfile,
    pub session: AndroidSession,
    pub capabilities: AndroidCapabilityReport,
    pub output_dir: PathBuf,
    pub items: Vec<AcquisitionItem>,
    pub total_bytes: u64,
    pub sha256: Option<String>,
    pub manifest_sha256: Option<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
/// Android dosya sistemi edinimini cihaz profiliyle birlikte döndüren sonuç modelidir.
pub struct AndroidOrchestratedFilesystemResult {
    pub device_profile: AndroidDeviceProfile,
    pub session: AndroidSession,
    pub capabilities: AndroidCapabilityReport,
    pub output_file: PathBuf,
    pub total_bytes: u64,
    pub sha256: String,
    pub manifest_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
/// Android RAM/uçucu veri edinimini cihaz profiliyle birlikte döndüren sonuç modelidir.
pub struct AndroidOrchestratedRamResult {
    pub device_profile: AndroidDeviceProfile,
    pub session: AndroidSession,
    pub capabilities: AndroidCapabilityReport,
    pub output_file: PathBuf,
    pub total_bytes: u64,
    pub sha256: String,
    pub manifest_sha256: Option<String>,
    pub mode: AndroidRamMode,
}

/// Cihaz profilini yazar, seçilen mantıksal profili çalıştırır ve sonucu birleştirir.
pub fn orchestrated_acquisition<F, C>(
    serial: &str,
    output_dir: &Path,
    profile: AndroidAcquisitionProfile,
    mut progress: F,
    cancelled: C,
) -> Result<AndroidOrchestratedAcquisitionResult, String>
where
    F: FnMut(u32, u32, &str),
    C: Fn() -> bool,
{
    std::fs::create_dir_all(output_dir)
        .map_err(|err| format!("Cikti dizini olusturulamadi: {err}"))?;

    let total = logical_steps_for_profile(profile).len() as u32 + 3;
    progress(0, total, "device_profile");

    let device_profile = detect_device_profile(serial)?;
    let (session, capabilities) =
        prepare_android_context(serial, output_dir, device_profile.clone())?;
    if profile == AndroidAcquisitionProfile::RootLogical && !device_profile.is_rooted {
        return Err("Root profili secildi ama cihazda root yetkisi dogrulanamadi".to_string());
    }

    if cancelled() {
        return Err("Kullanici tarafindan iptal edildi".to_string());
    }

    let logical = logical_acquisition_with_profile(
        serial,
        output_dir,
        profile,
        |done, _logical_total, category| progress(done + 1, total, category),
        cancelled,
    )?;

    let manifest_sha256 =
        write_logical_android_manifest(output_dir, &session, &capabilities, &logical).ok();

    Ok(from_logical_result(
        profile,
        device_profile,
        session,
        capabilities,
        manifest_sha256,
        logical,
    ))
}

/// Android dosya sistemi edinimini profil kaydıyla birlikte yürütür.
pub fn orchestrated_filesystem_acquisition<F, C>(
    serial: &str,
    output_dir: &Path,
    has_root: bool,
    mut progress: F,
    cancelled: C,
) -> Result<AndroidOrchestratedFilesystemResult, String>
where
    F: FnMut(u32, u32, &str),
    C: Fn() -> bool,
{
    std::fs::create_dir_all(output_dir)
        .map_err(|err| format!("Cikti dizini olusturulamadi: {err}"))?;
    progress(0, 4, "device_profile");
    let device_profile = detect_device_profile(serial)?;
    let (session, capabilities) =
        prepare_android_context(serial, output_dir, device_profile.clone())?;
    if cancelled() {
        return Err("Kullanici tarafindan iptal edildi".to_string());
    }

    let result = filesystem_acquisition(
        serial,
        output_dir,
        has_root,
        |done, total, category| progress(done + 1, total + 1, category),
        cancelled,
    )?;
    let manifest_sha256 = write_single_artifact_android_manifest(
        output_dir,
        &session,
        &capabilities,
        "android_filesystem",
        "filesystem",
        &result.output_file,
        result.total_bytes,
        &result.sha256,
    )
    .ok();
    Ok(from_filesystem_result(
        device_profile,
        session,
        capabilities,
        manifest_sha256,
        result,
    ))
}

/// Android RAM/uçucu veri edinimini profil kaydıyla birlikte yürütür.
pub fn orchestrated_ram_acquisition<F, C>(
    serial: &str,
    output_dir: &Path,
    has_root: bool,
    mode: AndroidRamMode,
    mut progress: F,
    cancelled: C,
) -> Result<AndroidOrchestratedRamResult, String>
where
    F: FnMut(u32, u32, &str),
    C: Fn() -> bool,
{
    std::fs::create_dir_all(output_dir)
        .map_err(|err| format!("Cikti dizini olusturulamadi: {err}"))?;
    progress(0, 4, "device_profile");
    let device_profile = detect_device_profile(serial)?;
    let (session, capabilities) =
        prepare_android_context(serial, output_dir, device_profile.clone())?;
    if cancelled() {
        return Err("Kullanici tarafindan iptal edildi".to_string());
    }

    let result = ram_acquisition_with_mode(
        serial,
        output_dir,
        has_root,
        mode,
        |done, total, category| progress(done + 1, total + 1, category),
        cancelled,
    )?;
    let manifest_sha256 = write_single_artifact_android_manifest(
        output_dir,
        &session,
        &capabilities,
        "android_memory",
        "memory",
        &result.output_file,
        result.total_bytes,
        &result.sha256,
    )
    .ok();
    Ok(from_ram_result(
        device_profile,
        session,
        capabilities,
        manifest_sha256,
        result,
    ))
}

/// Cihaz profili, oturum ve kabiliyet raporlarını edinim klasörüne yazar.
fn prepare_android_context(
    serial: &str,
    output_dir: &Path,
    device_profile: AndroidDeviceProfile,
) -> Result<(AndroidSession, AndroidCapabilityReport), String> {
    write_device_profile(output_dir, &device_profile)?;
    let session = build_android_session(serial, device_profile.clone());
    let capabilities = build_android_capability_report(serial, &device_profile);
    write_android_session(output_dir, &session)?;
    write_android_capability_report(output_dir, &capabilities)?;
    Ok((session, capabilities))
}

/// Algılanan cihaz profilini edinim klasörüne JSON olarak yazar.
fn write_device_profile(
    output_dir: &Path,
    device_profile: &AndroidDeviceProfile,
) -> Result<(), String> {
    let path = output_dir.join("device_profile.json");
    let content = serde_json::to_vec_pretty(device_profile).map_err(|err| err.to_string())?;
    std::fs::write(&path, content).map_err(|err| format!("Cihaz profili yazilamadi: {err}"))
}

/// Mantıksal edinim için ortak Android manifestini yazar.
fn write_logical_android_manifest(
    output_dir: &Path,
    session: &AndroidSession,
    capabilities: &AndroidCapabilityReport,
    logical: &LogicalAcquisitionResult,
) -> Result<String, String> {
    let manifest = manifest_from_logical_items(
        "android_logical",
        session,
        capabilities,
        output_dir,
        &logical.items,
        logical.total_bytes,
        logical.sha256.clone(),
        &logical.errors,
    );
    write_android_manifest(output_dir, &manifest)
}

/// Tek dosyalı Android edinimleri için ortak Android manifestini yazar.
fn write_single_artifact_android_manifest(
    output_dir: &Path,
    session: &AndroidSession,
    capabilities: &AndroidCapabilityReport,
    acquisition_type: &str,
    category: &str,
    output_file: &Path,
    size: u64,
    sha256: &str,
) -> Result<String, String> {
    let manifest = manifest_from_single_artifact(
        acquisition_type,
        session,
        capabilities,
        category,
        output_file,
        size,
        sha256,
    );
    write_android_manifest(output_dir, &manifest)
}

/// Dosya sistemi edinim sonucunu orkestrasyon modeline çevirir.
fn from_filesystem_result(
    device_profile: AndroidDeviceProfile,
    session: AndroidSession,
    capabilities: AndroidCapabilityReport,
    manifest_sha256: Option<String>,
    result: FilesystemAcquisitionResult,
) -> AndroidOrchestratedFilesystemResult {
    AndroidOrchestratedFilesystemResult {
        device_profile,
        session,
        capabilities,
        output_file: result.output_file,
        total_bytes: result.total_bytes,
        sha256: result.sha256,
        manifest_sha256,
    }
}

/// RAM edinim sonucunu orkestrasyon modeline çevirir.
fn from_ram_result(
    device_profile: AndroidDeviceProfile,
    session: AndroidSession,
    capabilities: AndroidCapabilityReport,
    manifest_sha256: Option<String>,
    result: AndroidRamAcquisitionResult,
) -> AndroidOrchestratedRamResult {
    AndroidOrchestratedRamResult {
        device_profile,
        session,
        capabilities,
        output_file: result.output_file,
        total_bytes: result.total_bytes,
        sha256: result.sha256,
        manifest_sha256,
        mode: result.mode,
    }
}

/// Mantıksal edinim sonucunu orkestrasyon modeline çevirir.
fn from_logical_result(
    profile: AndroidAcquisitionProfile,
    device_profile: AndroidDeviceProfile,
    session: AndroidSession,
    capabilities: AndroidCapabilityReport,
    manifest_sha256: Option<String>,
    logical: LogicalAcquisitionResult,
) -> AndroidOrchestratedAcquisitionResult {
    AndroidOrchestratedAcquisitionResult {
        profile,
        device_profile,
        session,
        capabilities,
        output_dir: logical.output_dir,
        items: logical.items,
        total_bytes: logical.total_bytes,
        sha256: logical.sha256,
        manifest_sha256,
        errors: logical.errors,
    }
}
