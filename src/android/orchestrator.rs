use super::{
    AcquisitionItem, AndroidAcquisitionProfile, AndroidDeviceProfile, LogicalAcquisitionResult,
    detect_device_profile, logical_acquisition_with_profile, logical_steps_for_profile,
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

fn write_device_profile(
    output_dir: &Path,
    device_profile: &AndroidDeviceProfile,
) -> Result<(), String> {
    let path = output_dir.join("device_profile.json");
    let content = serde_json::to_vec_pretty(device_profile).map_err(|err| err.to_string())?;
    std::fs::write(&path, content).map_err(|err| format!("Cihaz profili yazilamadi: {err}"))
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
