//! Android MFT kayıtlarını dosyaya yazan paket oluşturma mantığını içerir.
use super::format::{Field, MftBundleInfo, Record, RecordType, RecordWriter};
use super::outputs::sha256_file;
use super::parsers::build_logical_records;
use std::fs::{self, File};
use std::path::Path;

/// Mantıksal Android kayıtlarından binary evidence.mft paketi ve hash dosyası üretir.
pub fn write_logical_mft_bundle(serial: &str, dir: &Path) -> Result<MftBundleInfo, String> {
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
