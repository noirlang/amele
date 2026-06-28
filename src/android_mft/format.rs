//! Android MFT paket formatındaki alan, kayıt ve başlık yapılarını tanımlar.
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_HEADER_SIZE: usize = 32;
const RECORD_HEADER_SIZE: usize = 17;
const FIELD_HEADER_SIZE: usize = 6;
pub(super) const MAGIC: u32 = 0x4D465401;
pub(super) const VERSION: u32 = 1;
pub(super) const MAX_TEXT_INPUT: u64 = 16 * 1024 * 1024;
pub(super) const MAX_RECORDS_PER_SOURCE: usize = 2_000;

#[derive(Debug, Clone)]
/// Üretilen MFT/JSON/rapor dosyasının boyut, hash ve kayıt sayısı özetidir.
pub struct MftBundleInfo {
    pub file_name: String,
    pub size: u64,
    pub sha256: String,
    pub record_count: usize,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
/// Android delil kayıtlarının MFT içindeki tür kodlarını tanımlar.
pub(super) enum RecordType {
    Contact = 0x01,
    Call = 0x02,
    Sms = 0x03,
    Media = 0x05,
    ProcInfo = 0x07,
    Network = 0x08,
    LogEntry = 0x09,
    MemoryDump = 0x0D,
    MemInfo = 0x0E,
    Notification = 0x11,
    Telemetry = 0x12,
    UsageStat = 0x13,
    Account = 0x14,
    Location = 0x15,
    Wifi = 0x16,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
/// MFT alanının string, sayı, binary veya bool olarak nasıl kodlandığını belirtir.
pub(super) enum FieldType {
    String = 0x01,
    Int64 = 0x02,
    Binary = 0x03,
    Bool = 0x04,
}

#[derive(Debug, Clone)]
/// Tek bir MFT kaydı içindeki alan kimliği, tipi ve ham verisini taşır.
pub(super) struct Field {
    pub(super) id: u8,
    pub(super) field_type: FieldType,
    pub(super) data: Vec<u8>,
}

impl Field {
    /// String değeri MFT alanına dönüştürür.
    pub(super) fn string(id: u8, value: impl AsRef<str>) -> Self {
        Self {
            id,
            field_type: FieldType::String,
            data: value.as_ref().as_bytes().to_vec(),
        }
    }

    /// i64 değeri little-endian MFT alanına dönüştürür.
    pub(super) fn int64(id: u8, value: i64) -> Self {
        Self {
            id,
            field_type: FieldType::Int64,
            data: value.to_le_bytes().to_vec(),
        }
    }

    /// Bool değeri tek byte MFT alanına dönüştürür.
    pub(super) fn bool(id: u8, value: bool) -> Self {
        Self {
            id,
            field_type: FieldType::Bool,
            data: vec![u8::from(value)],
        }
    }

    /// Alanın disk üzerinde kaplayacağı toplam byte sayısını hesaplar.
    fn encoded_len(&self) -> usize {
        FIELD_HEADER_SIZE + self.data.len()
    }

    /// Alan header ve verisini binary MFT çıktısına yazar.
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.id, self.field_type as u8])?;
        writer.write_all(&(self.data.len() as u32).to_le_bytes())?;
        writer.write_all(&self.data)
    }
}

#[derive(Debug, Clone)]
/// Tek bir MFT kaydının türünü, zamanını ve alanlarını taşır.
pub(super) struct Record {
    pub(super) record_type: RecordType,
    pub(super) timestamp_ns: i64,
    pub(super) fields: Vec<Field>,
}

impl Record {
    /// Yeni kayıt oluşturur ve zaman damgasını anlık saatten alır.
    pub(super) fn new(record_type: RecordType, fields: Vec<Field>) -> Self {
        Self {
            record_type,
            timestamp_ns: now_ns(),
            fields,
        }
    }

    /// Kayıt header ve alanları dahil toplam byte uzunluğunu hesaplar.
    fn encoded_len(&self) -> usize {
        RECORD_HEADER_SIZE + self.fields.iter().map(Field::encoded_len).sum::<usize>()
    }

    /// Kayıt header ve alanlarını binary MFT çıktısına yazar.
    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let total_len = self.encoded_len();
        writer.write_all(&[self.record_type as u8])?;
        writer.write_all(&(total_len as u32).to_le_bytes())?;
        writer.write_all(&self.timestamp_ns.to_le_bytes())?;
        writer.write_all(&(self.fields.len() as u32).to_le_bytes())?;
        for field in &self.fields {
            field.write_to(writer)?;
        }
        Ok(())
    }
}

/// Dosya header yazıldıktan sonra kayıtları ardışık olarak yazan yardımcıdır.
pub(super) struct RecordWriter<W: Write> {
    writer: W,
}

impl<W: Write> RecordWriter<W> {
    /// Yeni MFT dosyası için header yazar ve writer döndürür.
    pub(super) fn new(mut writer: W, serial: &str) -> io::Result<Self> {
        write_file_header(&mut writer, serial)?;
        Ok(Self { writer })
    }

    /// Tek bir MFT kaydını dosyaya ekler.
    pub(super) fn write_record(&mut self, record: &Record) -> io::Result<()> {
        record.write_to(&mut self.writer)
    }
}

/// MFT dosya header alanlarını magic, version, zaman ve serial ile doldurur.
fn write_file_header<W: Write>(writer: &mut W, serial: &str) -> io::Result<()> {
    let mut header = [0_u8; FILE_HEADER_SIZE];
    header[0..4].copy_from_slice(&MAGIC.to_le_bytes());
    header[4..8].copy_from_slice(&VERSION.to_le_bytes());
    header[8..16].copy_from_slice(&now_ns().to_le_bytes());
    let serial = serial.as_bytes();
    let serial_len = serial.len().min(12);
    header[16..20].copy_from_slice(&(serial_len as u32).to_le_bytes());
    header[20..20 + serial_len].copy_from_slice(&serial[..serial_len]);
    writer.write_all(&header)
}

/// UNIX epoch nanosecond zaman damgası üretir.
pub(super) fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as i64)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_header_matches_mft_v1_layout() {
        let mut buf = Vec::new();
        write_file_header(&mut buf, "123456789abcdef").unwrap();

        assert_eq!(buf.len(), FILE_HEADER_SIZE);
        assert_eq!(&buf[0..4], &MAGIC.to_le_bytes());
        assert_eq!(&buf[4..8], &VERSION.to_le_bytes());
        assert_eq!(u32::from_le_bytes(buf[16..20].try_into().unwrap()), 12);
        assert_eq!(&buf[20..32], b"123456789abc");
    }

    #[test]
    fn record_header_uses_total_encoded_length() {
        let record = Record::new(
            RecordType::Telemetry,
            vec![
                Field::string(0x01, "ro.product.model"),
                Field::string(0x02, "Pixel"),
            ],
        );
        let mut buf = Vec::new();
        record.write_to(&mut buf).unwrap();

        assert_eq!(buf[0], RecordType::Telemetry as u8);
        assert_eq!(
            u32::from_le_bytes(buf[1..5].try_into().unwrap()) as usize,
            buf.len()
        );
        assert_eq!(u32::from_le_bytes(buf[13..17].try_into().unwrap()), 2);
    }
}
