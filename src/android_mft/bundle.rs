//! dosya listesi ve metadata kayıtlarını arşivleyen paketleyici.

use super::format::{Field, MftBundleInfo, Record, RecordType, RecordWriter};
use super::outputs::sha256_file;
use super::parsers::build_logical_records;
use std::fs::{self, File};
use std::path::Path;

/// Mantıksal Android kayıtlarından binary evidence.mft paketi ve hash dosyası üretir.
pub fn write_logical_mft_bundle(serial: &str, dir: &Path) -> Result<MftBundleInfo, String> {
    let _ = fs::create_dir_all(dir);
    let file_name = "evidence.mft";
    let output_path = dir.join(file_name);
    let file = File::create(&output_path).map_err(|err| format!("MFT olusturulamadi: {err}"))?;
    let mut writer =
        RecordWriter::new(file, serial).map_err(|err| format!("MFT header yazilamadi: {err}"))?;

    let mut record_count = 0_usize;
    for record in build_logical_records(dir) {
        writer
            .write_record(&record)
            .map_err(|err| format!("MFT record yazilamadi: {err}"))?;
        record_count += 1;
    }

    if record_count == 0 {
        let record = Record::new(
            RecordType::Telemetry,
            vec![
                Field::string(0x01, "amele.mft.status"),
                Field::string(0x02, "no structured Android records were extracted"),
            ],
        );
        writer
            .write_record(&record)
            .map_err(|err| format!("MFT durum record'u yazilamadi: {err}"))?;
        record_count = 1;
    }

    drop(writer);
    let size = fs::metadata(&output_path)
        .map_err(|err| format!("MFT metadata okunamadi: {err}"))?
        .len();
    let sha256 = sha256_file(&output_path)?;
    let sidecar = dir.join("evidence.mft.sha256");
    fs::write(&sidecar, format!("{sha256}  {file_name}\n"))
        .map_err(|err| format!("MFT hash dosyasi yazilamadi: {err}"))?;

    Ok(MftBundleInfo {
        file_name: file_name.to_string(),
        size,
        sha256,
        record_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_logical_mft_bundle_empty_dir() {
        let temp_dir = std::env::temp_dir().join(format!("amele_mft_bundle_test_{}", std::process::id()));
        let res = write_logical_mft_bundle("test_serial", &temp_dir);
        assert!(res.is_ok());
        let info = res.unwrap();
        assert_eq!(info.record_count, 1);
        assert!(info.size > 0);
        assert!(!info.sha256.is_empty());
        assert!(temp_dir.join("evidence.mft").is_file());
        assert!(temp_dir.join("evidence.mft.sha256").is_file());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
