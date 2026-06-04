use super::{
    AcquisitionItem, AndroidAcquisitionProfile, AndroidDeviceProfile, AndroidRamAcquisitionResult,
    AndroidRamMode, FilesystemAcquisitionResult, LogicalAcquisitionResult, detect_device_profile,
    filesystem_acquisition, logical_acquisition_with_profile, logical_steps_for_profile,
    ram_acquisition_with_mode,
};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize)]
pub struct AndroidOrchestratedAcquisitionResult {
    pub profile: AndroidAcquisitionProfile,
    pub device_profile: AndroidDeviceProfile,
    pub output_dir: PathBuf,
    pub items: Vec<AcquisitionItem>,
    pub total_bytes: u64,
    pub sha256: Option<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AndroidOrchestratedFilesystemResult {
    pub device_profile: AndroidDeviceProfile,
    pub output_file: PathBuf,
    pub total_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AndroidOrchestratedRamResult {
    pub device_profile: AndroidDeviceProfile,
    pub output_file: PathBuf,
    pub total_bytes: u64,
    pub sha256: String,
    pub mode: AndroidRamMode,
}

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
    if profile == AndroidAcquisitionProfile::RootLogical && !device_profile.is_rooted {
        return Err("Root profili secildi ama cihazda root yetkisi dogrulanamadi".to_string());
    }

    write_device_profile(output_dir, &device_profile)?;

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

    Ok(from_logical_result(profile, device_profile, logical))
}

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
    write_device_profile(output_dir, &device_profile)?;
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
    Ok(from_filesystem_result(device_profile, result))
}

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
    write_device_profile(output_dir, &device_profile)?;
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
    Ok(from_ram_result(device_profile, result))
}

fn write_device_profile(
    output_dir: &Path,
    device_profile: &AndroidDeviceProfile,
) -> Result<(), String> {
    let path = output_dir.join("device_profile.json");
    let content = serde_json::to_vec_pretty(device_profile).map_err(|err| err.to_string())?;
    std::fs::write(&path, content).map_err(|err| format!("Cihaz profili yazilamadi: {err}"))
}

fn from_filesystem_result(
    device_profile: AndroidDeviceProfile,
    result: FilesystemAcquisitionResult,
) -> AndroidOrchestratedFilesystemResult {
    AndroidOrchestratedFilesystemResult {
        device_profile,
        output_file: result.output_file,
        total_bytes: result.total_bytes,
        sha256: result.sha256,
    }
}

fn from_ram_result(
    device_profile: AndroidDeviceProfile,
    result: AndroidRamAcquisitionResult,
) -> AndroidOrchestratedRamResult {
    AndroidOrchestratedRamResult {
        device_profile,
        output_file: result.output_file,
        total_bytes: result.total_bytes,
        sha256: result.sha256,
        mode: result.mode,
    }
}

fn from_logical_result(
    profile: AndroidAcquisitionProfile,
    device_profile: AndroidDeviceProfile,
    logical: LogicalAcquisitionResult,
) -> AndroidOrchestratedAcquisitionResult {
    AndroidOrchestratedAcquisitionResult {
        profile,
        device_profile,
        output_dir: logical.output_dir,
        items: logical.items,
        total_bytes: logical.total_bytes,
        sha256: logical.sha256,
        errors: logical.errors,
    }
}
