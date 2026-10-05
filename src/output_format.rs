//! raw dd formatında veya adli paket formatında dosya çıktısı üretme.

use crate::error::{AmeleError, AmeleResult, HataKodu};
use crate::hash::{self, HashAlgorithm};
use chrono::Local;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Desteklenen edinim çıktı formatlarıdır.
pub enum AcquisitionOutputFormat {
    Raw,
    Aff4,
}

#[derive(Debug, Clone)]
/// Edinim sırasında yazılacak geçici/final hedef yollarını taşır.
pub struct OutputPlan {
    pub format: AcquisitionOutputFormat,
    pub working_path: PathBuf,
    pub final_path: PathBuf,
}

/// finalize uzun fazlarinin arayuzdeki etiketleri.
pub const PHASE_HASH: &str = "SHA-256 hesaplanıyor";
pub const PHASE_PACK: &str = "AFF4 paketleniyor";

#[derive(Debug, Clone)]
/// Final format dönüşümünden sonra API/CLI'ye dönecek bilgidir.
pub struct FinalizedOutput {
    pub target_path: PathBuf,
    pub sha256: String,
    /// hash edinim sirasinda hazir geldiyse (disk) None, RAM'de finalize sirasinda hesaplanir
    pub blake3: Option<String>,
    pub raw_sha256: Option<String>,
    pub format: AcquisitionOutputFormat,
}

impl AcquisitionOutputFormat {
    /// Kullanıcı/API değerini çıktı formatına çevirir.
    pub fn parse(value: Option<&str>) -> Result<Self, String> {
        match value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("raw")
            .to_ascii_lowercase()
            .as_str()
        {
            "raw" | "dd" | "img" | "raw_sparse" | "raw-sparse" | "raw_full" | "raw-full" => {
                Ok(Self::Raw)
            }
            "aff4" | "aff4_sparse" | "aff4-sparse" | "aff4_full" | "aff4-full" => Ok(Self::Aff4),
            other => Err(format!("output_format raw veya aff4 olmalıdır: {other}")),
        }
    }

    /// Format dizgisi içinde açıkça seyrek veya tam edinim tercihi belirtilmişse döner.
    pub fn parse_sparse_preference(value: Option<&str>) -> Option<bool> {
        let val = value?.trim().to_ascii_lowercase();
        if val.ends_with("_sparse") || val.ends_with("-sparse") || val.contains("sparse") {
            Some(true)
        } else if val.ends_with("_full") || val.ends_with("-full") || val.contains("full") {
            Some(false)
        } else {
            None
        }
    }

    /// JSON ve UI için kısa format adını döndürür.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Raw => "raw",
            Self::Aff4 => "aff4",
        }
    }
}

/// İstenen final format için çalışma ve final yollarını üretir.
pub fn plan_output(target: impl AsRef<Path>, format: AcquisitionOutputFormat) -> OutputPlan {
    let target = target.as_ref();
    match format {
        AcquisitionOutputFormat::Raw => OutputPlan {
            format,
            working_path: target.to_path_buf(),
            final_path: target.to_path_buf(),
        },
        AcquisitionOutputFormat::Aff4 => {
            let final_path = target.with_extension("aff4");
            let working_path = final_path.with_extension("aff4.raw");
            OutputPlan {
                format,
                working_path,
                final_path,
            }
        }
    }
}

/// Edinim çıktısını seçilen final formata tamamlar.
pub fn finalize_output(
    plan: &OutputPlan,
    artifact_kind: &str,
    source_label: &str,
    case_name: &str,
    existing_raw_sha256: Option<String>,
) -> Result<FinalizedOutput, String> {
    let mut noop = |_done: u64, _total: u64, _phase: &str| {};
    finalize_output_with_progress(
        plan,
        artifact_kind,
        source_label,
        case_name,
        existing_raw_sha256,
        &mut noop,
    )
}

/// Edinim çıktısını final formata tamamlar, uzun hash/paket fazlarını raporlar.
/// on_progress(done, total, phase) imaj sonu takilma hissini onler.
pub fn finalize_output_with_progress(
    plan: &OutputPlan,
    artifact_kind: &str,
    source_label: &str,
    case_name: &str,
    existing_raw_sha256: Option<String>,
    on_progress: &mut dyn FnMut(u64, u64, &'static str),
) -> Result<FinalizedOutput, String> {
    match plan.format {
        AcquisitionOutputFormat::Raw => {
            let (sha256, blake3) = raw_hashes(plan, existing_raw_sha256, on_progress)?;
            hash::write_sha256_sidecar(&plan.working_path, &sha256)
                .map_err(|err| err.to_string())?;
            if let Some(value) = &blake3 {
                hash::write_blake3_sidecar(&plan.working_path, value)
                    .map_err(|err| err.to_string())?;
            }
            Ok(FinalizedOutput {
                target_path: plan.working_path.clone(),
                sha256,
                blake3,
                raw_sha256: None,
                format: plan.format,
            })
        }
        AcquisitionOutputFormat::Aff4 => {
            let (raw_sha256, _) = raw_hashes(plan, existing_raw_sha256, on_progress)?;
            on_progress(0, 0, PHASE_PACK);
            package_aff4(plan, artifact_kind, source_label, case_name, &raw_sha256)
                .map_err(|err| err.to_string())?;
            let aff4_sha256 = hash::calculate_file_hash(&plan.final_path, HashAlgorithm::Sha256)
                .map_err(|err| err.to_string())?;
            hash::write_sha256_sidecar(&plan.final_path, &aff4_sha256)
                .map_err(|err| err.to_string())?;
            // paketlenen aff4 icin blake3 mmap + rayon ile hizli, ayri sidecar yaziyoruz
            let aff4_blake3 = hash::calculate_file_hash(&plan.final_path, HashAlgorithm::Blake3)
                .map_err(|err| err.to_string())?;
            hash::write_blake3_sidecar(&plan.final_path, &aff4_blake3)
                .map_err(|err| err.to_string())?;
            let _ = fs::remove_file(&plan.working_path);
            let _ = fs::remove_file(plan.working_path.with_extension("aff4.raw.sha256"));
            let _ = fs::remove_file(plan.working_path.with_extension("aff4.raw.b3sum"));
            Ok(FinalizedOutput {
                target_path: plan.final_path.clone(),
                sha256: aff4_sha256,
                blake3: Some(aff4_blake3),
                raw_sha256: Some(raw_sha256),
                format: plan.format,
            })
        }
    }
}

/// Ham dosyanin SHA-256 (ve gerekirse BLAKE3) degerlerini uretir.
/// Hash edinim sirasinda hesaplanmissa (disk) tekrar okumaz. RAM gibi dis aracin yazdigi
/// dosyalarda ise dosyayi tek geciste okuyup ikisini birden hesaplar.
fn raw_hashes(
    plan: &OutputPlan,
    existing_raw_sha256: Option<String>,
    on_progress: &mut dyn FnMut(u64, u64, &'static str),
) -> Result<(String, Option<String>), String> {
    if let Some(value) = existing_raw_sha256.filter(|value| !value.trim().is_empty()) {
        return Ok((value, None));
    }
    on_progress(0, 0, PHASE_HASH);
    let results = hash::calculate_multiple_with_progress(
        &plan.working_path,
        &[HashAlgorithm::Sha256, HashAlgorithm::Blake3],
        &mut |done, total| on_progress(done, total, PHASE_HASH),
    )
    .map_err(|err| err.to_string())?;
    let find = |algorithm: HashAlgorithm| {
        results
            .iter()
            .find(|result| result.algorithm == algorithm)
            .map(|result| result.value.clone())
    };
    let sha256 = find(HashAlgorithm::Sha256).ok_or("SHA-256 sonucu uretilemedi")?;
    Ok((sha256, find(HashAlgorithm::Blake3)))
}

/// Basit AFF4 kanıt paketini manifest ve veri girdileriyle oluşturur.
fn package_aff4(
    plan: &OutputPlan,
    artifact_kind: &str,
    source_label: &str,
    case_name: &str,
    raw_sha256: &str,
) -> AmeleResult<()> {
    if let Some(parent) = plan.final_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            AmeleError::io(
                HataKodu::DosyaYazma,
                "AFF4 hedef klasörü oluşturulamadı",
                err,
            )
        })?;
    }

    let raw_file = File::open(&plan.working_path)
        .map_err(|err| AmeleError::io(HataKodu::DosyaOkuma, "RAW veri açılamadı", err))?;
    let raw_size = raw_file
        .metadata()
        .map_err(|err| AmeleError::io(HataKodu::DosyaOkuma, "RAW metadata okunamadı", err))?
        .len();
    let aff4_file = File::create(&plan.final_path)
        .map_err(|err| AmeleError::io(HataKodu::DosyaYazma, "AFF4 paketi oluşturulamadı", err))?;
    let mut builder = tar::Builder::new(aff4_file);
    let data_name = plan
        .working_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("evidence.raw")
        .to_string();

    let manifest = json!({
        "format": "aff4",
        "container": "tar-aff4-evidence-package",
        "created_at": Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        "artifact_kind": artifact_kind,
        "source": source_label,
        "case_name": case_name,
        "operator": crate::profile::active_profile(),
        "data_file": data_name,
        "raw_size": raw_size,
        "raw_sha256": raw_sha256,
    });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    let mut manifest_header = tar::Header::new_gnu();
    manifest_header.set_size(manifest_bytes.len() as u64);
    manifest_header.set_mode(0o644);
    manifest_header.set_cksum();
    builder
        .append_data(
            &mut manifest_header,
            "manifest.json",
            manifest_bytes.as_slice(),
        )
        .map_err(|err| AmeleError::io(HataKodu::DosyaYazma, "AFF4 manifest yazılamadı", err))?;

    let mut data_header = tar::Header::new_gnu();
    data_header.set_size(raw_size);
    data_header.set_mode(0o644);
    data_header.set_cksum();
    builder
        .append_data(&mut data_header, data_name, raw_file)
        .map_err(|err| AmeleError::io(HataKodu::DosyaYazma, "AFF4 veri yazılamadı", err))?;
    builder
        .finish()
        .map_err(|err| AmeleError::io(HataKodu::DosyaYazma, "AFF4 paket kapatılamadı", err))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_parsing() {
        assert_eq!(
            AcquisitionOutputFormat::parse(None).unwrap(),
            AcquisitionOutputFormat::Raw
        );
        assert_eq!(
            AcquisitionOutputFormat::parse(Some("raw")).unwrap(),
            AcquisitionOutputFormat::Raw
        );
        assert_eq!(
            AcquisitionOutputFormat::parse(Some("dd")).unwrap(),
            AcquisitionOutputFormat::Raw
        );
        assert_eq!(
            AcquisitionOutputFormat::parse(Some("aff4")).unwrap(),
            AcquisitionOutputFormat::Aff4
        );
        assert_eq!(
            AcquisitionOutputFormat::parse(Some("raw_sparse")).unwrap(),
            AcquisitionOutputFormat::Raw
        );
        assert_eq!(
            AcquisitionOutputFormat::parse(Some("raw_full")).unwrap(),
            AcquisitionOutputFormat::Raw
        );
        assert_eq!(
            AcquisitionOutputFormat::parse(Some("aff4_sparse")).unwrap(),
            AcquisitionOutputFormat::Aff4
        );
        assert_eq!(
            AcquisitionOutputFormat::parse(Some("aff4_full")).unwrap(),
            AcquisitionOutputFormat::Aff4
        );
        assert_eq!(
            AcquisitionOutputFormat::parse_sparse_preference(Some("raw_sparse")),
            Some(true)
        );
        assert_eq!(
            AcquisitionOutputFormat::parse_sparse_preference(Some("raw_full")),
            Some(false)
        );
        assert_eq!(
            AcquisitionOutputFormat::parse_sparse_preference(Some("aff4_sparse")),
            Some(true)
        );
        assert_eq!(
            AcquisitionOutputFormat::parse_sparse_preference(Some("aff4_full")),
            Some(false)
        );
        assert_eq!(
            AcquisitionOutputFormat::parse_sparse_preference(Some("raw")),
            None
        );
        assert!(AcquisitionOutputFormat::parse(Some("invalid")).is_err());
    }

    #[test]
    fn test_plan_output() {
        let p = Path::new("/tmp/test.raw");
        let plan_raw = plan_output(p, AcquisitionOutputFormat::Raw);
        assert_eq!(plan_raw.final_path, PathBuf::from("/tmp/test.raw"));
        assert_eq!(plan_raw.working_path, PathBuf::from("/tmp/test.raw"));

        let plan_aff4 = plan_output(p, AcquisitionOutputFormat::Aff4);
        assert_eq!(plan_aff4.final_path, PathBuf::from("/tmp/test.aff4"));
        assert_eq!(plan_aff4.working_path, PathBuf::from("/tmp/test.aff4.raw"));
    }

    #[test]
    fn test_finalize_output_raw_and_aff4() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("mem.raw");
        fs::write(&target, b"raw forensic data").unwrap();

        let plan = plan_output(&target, AcquisitionOutputFormat::Raw);
        let fin = finalize_output(&plan, "ram", "mem", "case1", None).unwrap();
        assert_eq!(fin.target_path, target);
        assert!(!fin.sha256.is_empty());
        assert!(dir.path().join("mem.raw.sha256").exists());
        // ram imajinda blake3 de hesaplanip .b3sum yazilmali
        assert!(dir.path().join("mem.raw.b3sum").exists());
        assert!(fin.blake3.is_some());

        // Test Aff4
        let aff4_plan = plan_output(dir.path().join("dump"), AcquisitionOutputFormat::Aff4);
        fs::write(&aff4_plan.working_path, b"some memory").unwrap();
        let fin_aff4 = finalize_output(&aff4_plan, "ram", "mem", "case1", None).unwrap();
        assert!(fin_aff4.target_path.exists());
        assert_eq!(fin_aff4.format, AcquisitionOutputFormat::Aff4);
        assert!(fin_aff4.blake3.is_some());
        assert!(dir.path().join("dump.aff4.b3sum").exists());
        // gecici ham dosyanin sidecar'lari temizlenmis olmali
        assert!(!dir.path().join("dump.aff4.raw.b3sum").exists());
    }

    #[test]
    fn test_ram_raw_blake3_matches_independent_hash() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("ram.raw");
        // 8 MB'tan buyuk veri: birden fazla okuma bloguna yayilsin
        let data: Vec<u8> = (0..(9 * 1024 * 1024)).map(|i| (i % 251) as u8).collect();
        fs::write(&target, &data).unwrap();

        let plan = plan_output(&target, AcquisitionOutputFormat::Raw);
        let fin = finalize_output(&plan, "ram", "mem", "case1", None).unwrap();

        assert_eq!(
            fin.blake3.as_deref(),
            Some(blake3::hash(&data).to_hex().as_str())
        );
        // otomatik dogrulama .b3sum'u bulup gecmeli
        assert!(crate::disk::verify_image_auto(&target).unwrap());
    }

    #[test]
    fn test_disk_existing_sha256_skips_rehash() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("disk.img");
        fs::write(&target, b"disk data").unwrap();

        // disk edinimi hash'i zaten hesapladiysa finalize dosyayi tekrar okumaz, blake3 de uretmez
        let plan = plan_output(&target, AcquisitionOutputFormat::Raw);
        let fin = finalize_output(&plan, "disk", "d", "case1", Some("abc123".to_string())).unwrap();
        assert_eq!(fin.sha256, "abc123");
        assert!(fin.blake3.is_none());
        assert!(!dir.path().join("disk.img.b3sum").exists());
    }
}
