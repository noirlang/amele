use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const FILE_HEADER_SIZE: usize = 32;
const RECORD_HEADER_SIZE: usize = 17;
const FIELD_HEADER_SIZE: usize = 6;
const MAGIC: u32 = 0x4D465401;
const VERSION: u32 = 1;
const MAX_TEXT_INPUT: u64 = 16 * 1024 * 1024;
const MAX_RECORDS_PER_SOURCE: usize = 2_000;

#[derive(Debug, Clone)]
pub struct MftBundleInfo {
    pub file_name: String,
    pub size: u64,
    pub sha256: String,
    pub record_count: usize,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
enum RecordType {
    Contact = 0x01,
    Call = 0x02,
    Sms = 0x03,
    Media = 0x05,
    ProcInfo = 0x07,
    Network = 0x08,
    LogEntry = 0x09,
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
enum FieldType {
    String = 0x01,
    Int64 = 0x02,
    Binary = 0x03,
    Bool = 0x04,
}

#[derive(Debug, Clone)]
struct Field {
    id: u8,
    field_type: FieldType,
    data: Vec<u8>,
}

impl Field {
    fn string(id: u8, value: impl AsRef<str>) -> Self {
        Self {
            id,
            field_type: FieldType::String,
            data: value.as_ref().as_bytes().to_vec(),
        }
    }

    fn int64(id: u8, value: i64) -> Self {
        Self {
            id,
            field_type: FieldType::Int64,
            data: value.to_le_bytes().to_vec(),
        }
    }

    fn bool(id: u8, value: bool) -> Self {
        Self {
            id,
            field_type: FieldType::Bool,
            data: vec![u8::from(value)],
        }
    }

    fn encoded_len(&self) -> usize {
        FIELD_HEADER_SIZE + self.data.len()
    }

    fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(&[self.id, self.field_type as u8])?;
        writer.write_all(&(self.data.len() as u32).to_le_bytes())?;
        writer.write_all(&self.data)
    }
}

#[derive(Debug, Clone)]
struct Record {
    record_type: RecordType,
    timestamp_ns: i64,
    fields: Vec<Field>,
}

impl Record {
    fn new(record_type: RecordType, fields: Vec<Field>) -> Self {
        Self {
            record_type,
            timestamp_ns: now_ns(),
            fields,
        }
    }

    fn encoded_len(&self) -> usize {
        RECORD_HEADER_SIZE + self.fields.iter().map(Field::encoded_len).sum::<usize>()
    }

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

struct RecordWriter<W: Write> {
    writer: W,
}

impl<W: Write> RecordWriter<W> {
    fn new(mut writer: W, serial: &str) -> io::Result<Self> {
        write_file_header(&mut writer, serial)?;
        Ok(Self { writer })
    }

    fn write_record(&mut self, record: &Record) -> io::Result<()> {
        record.write_to(&mut self.writer)
    }
}

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
                Field::string(0x01, "worm.mft.status"),
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

fn build_logical_records(dir: &Path) -> Vec<Record> {
    let mut records = Vec::new();

    if let Some(content) = read_text_file(dir.join("device_info.txt")) {
        records.extend(parse_getprop_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("dumpsys_usagestats.txt")) {
        records.extend(parse_usage_stat_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("dumpsys_account.txt")) {
        records.extend(parse_account_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("dumpsys_wifi.txt")) {
        records.extend(parse_wifi_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("dumpsys_location.txt")) {
        records.extend(parse_location_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("dumpsys_meminfo.txt")) {
        records.push(Record::new(
            RecordType::MemInfo,
            vec![Field::string(0x07, trim_for_record(&content, 32 * 1024))],
        ));
    }
    if let Some(content) = read_text_file(dir.join("dumpsys_notification.txt")) {
        records.extend(parse_notification_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("processes.txt")) {
        records.extend(parse_process_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("network_info.txt")) {
        records.extend(parse_network_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("logcat.txt")) {
        records.extend(parse_logcat_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("content_sms.txt")) {
        records.extend(parse_sms_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("content_calls.txt")) {
        records.extend(parse_call_records(&content));
    }
    if let Some(content) = read_text_file(dir.join("content_contacts.txt")) {
        records.extend(parse_contact_records(&content));
    }
    for file in [
        "content_media_images.txt",
        "content_media_videos.txt",
        "content_media_audio.txt",
        "content_media_files.txt",
    ] {
        if let Some(content) = read_text_file(dir.join(file)) {
            records.extend(parse_media_records(&content));
        }
    }

    records
}

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

fn now_ns() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as i64)
        .unwrap_or_default()
}

fn read_text_file(path: impl AsRef<Path>) -> Option<String> {
    let path = path.as_ref();
    let mut file = File::open(path).ok()?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_TEXT_INPUT)
        .read_to_end(&mut bytes)
        .ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|err| format!("Hash icin dosya acilamadi: {err}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| format!("Hash icin dosya okunamadi: {err}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(crate::hash::to_hex(&hasher.finalize()))
}

fn parse_getprop_records(content: &str) -> Vec<Record> {
    let prefixes = [
        "ro.product",
        "ro.build",
        "ro.hardware",
        "ro.serialno",
        "gsm.version",
        "ro.crypto",
        "ro.secure",
    ];
    content
        .lines()
        .filter_map(parse_getprop_line)
        .filter(|(key, value)| {
            !value.is_empty() && prefixes.iter().any(|prefix| key.starts_with(prefix))
        })
        .map(|(key, value)| {
            Record::new(
                RecordType::Telemetry,
                vec![Field::string(0x01, key), Field::string(0x02, value)],
            )
        })
        .collect()
}

fn parse_getprop_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('[') || !trimmed.contains("]: [") {
        return None;
    }
    let (key, value) = trimmed.split_once("]: [")?;
    Some((
        key.trim_start_matches('[').to_string(),
        value.trim_end_matches(']').to_string(),
    ))
}

fn parse_usage_stat_records(content: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut seen = HashSet::new();
    for line in content.lines().map(str::trim) {
        if records.len() >= MAX_RECORDS_PER_SOURCE {
            break;
        }
        if !(line.contains("package=") || line.contains("packageName=")) {
            continue;
        }
        let package = extract_value(line, "package")
            .or_else(|| extract_value(line, "packageName"))
            .unwrap_or_default();
        if package.is_empty() || !seen.insert(package.clone()) {
            continue;
        }
        let mut fields = vec![Field::string(0x01, package)];
        if let Some(last) =
            extract_value(line, "lastTime").or_else(|| extract_value(line, "lastTimeUsed"))
        {
            if let Some(value) = parse_i64_prefix(&last) {
                fields.push(Field::int64(0x02, value));
            }
        }
        if let Some(total) = extract_value(line, "totalTime")
            .or_else(|| extract_value(line, "totalTimeInForeground"))
        {
            if let Some(value) = parse_duration_to_ms(&total) {
                fields.push(Field::int64(0x03, value));
            }
        }
        records.push(Record::new(RecordType::UsageStat, fields));
    }
    records
}

fn parse_account_records(content: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut in_accounts = false;
    let mut seen = HashSet::new();

    for line in content.lines().map(str::trim) {
        if records.len() >= MAX_RECORDS_PER_SOURCE {
            break;
        }
        if line.starts_with("Accounts:") {
            in_accounts = true;
            continue;
        }
        if !in_accounts {
            continue;
        }
        if !(line.starts_with("Account {name=") || line.starts_with("Account {")) {
            if line.is_empty() || !line.starts_with("Account") {
                in_accounts = false;
            }
            continue;
        }
        let fields = parse_braced_fields(line);
        let name = fields.get("name").cloned().unwrap_or_default();
        let account_type = fields.get("type").cloned().unwrap_or_default();
        let key = format!("{name}\0{account_type}");
        if (name.is_empty() && account_type.is_empty()) || !seen.insert(key) {
            continue;
        }
        records.push(Record::new(
            RecordType::Account,
            vec![Field::string(0x01, name), Field::string(0x02, account_type)],
        ));
    }

    records
}

fn parse_wifi_records(content: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut seen = HashSet::new();
    let mut in_configured = false;

    for raw in content.lines() {
        if records.len() >= MAX_RECORDS_PER_SOURCE {
            break;
        }
        let line = raw.trim();
        if line.starts_with("Configured networks:") || line.contains("NetworkList:") {
            in_configured = true;
            continue;
        }
        if line.is_empty() {
            in_configured = false;
            continue;
        }

        let candidate = if in_configured && line.contains("SSID:") {
            parse_configured_wifi_line(line)
        } else if line.contains("SSID:") && line.contains("BSSID:") {
            parse_connection_wifi_line(line)
        } else {
            None
        };

        let Some((ssid, bssid, security)) = candidate else {
            continue;
        };
        let key = format!("{ssid}\0{bssid}");
        if (ssid.is_empty() && bssid.is_empty()) || !seen.insert(key) {
            continue;
        }
        let mut fields = Vec::new();
        push_str_field(&mut fields, 0x01, &ssid);
        push_str_field(&mut fields, 0x02, &bssid);
        push_str_field(&mut fields, 0x04, &security);
        records.push(Record::new(RecordType::Wifi, fields));
    }

    records
}

fn parse_location_records(content: &str) -> Vec<Record> {
    let mut records = Vec::new();
    let mut seen = HashSet::new();
    let mut in_last_known = false;
    let mut in_requests = false;

    for raw in content.lines() {
        if records.len() >= MAX_RECORDS_PER_SOURCE {
            break;
        }
        let line = raw.trim();
        if line.starts_with("Last Known Locations:") {
            in_last_known = true;
            in_requests = false;
            continue;
        }
        if line.contains("Location Requests:") || line.contains("Active Records:") {
            in_last_known = false;
            in_requests = true;
            continue;
        }
        if line.is_empty() {
            in_last_known = false;
            in_requests = false;
            continue;
        }

        let mut fields = Vec::new();
        if in_last_known {
            if let Some((provider, lat, lon)) = parse_last_known_location(line) {
                fields.push(Field::string(0x01, lat));
                fields.push(Field::string(0x02, lon));
                push_str_field(&mut fields, 0x03, &provider);
            }
        } else if in_requests {
            if let Some(app) = extract_package_name(line) {
                push_str_field(&mut fields, 0x05, &app);
                if line.contains("network") {
                    fields.push(Field::string(0x03, "network"));
                } else if line.contains("gps") {
                    fields.push(Field::string(0x03, "gps"));
                }
            }
        }

        if !fields.is_empty() {
            let key = fields
                .iter()
                .map(|field| String::from_utf8_lossy(&field.data).into_owned())
                .collect::<Vec<_>>()
                .join("\0");
            if seen.insert(key) {
                records.push(Record::new(RecordType::Location, fields));
            }
        }
    }

    records
}

fn parse_notification_records(content: &str) -> Vec<Record> {
    content
        .lines()
        .filter(|line| line.contains("NotificationRecord") || line.contains("pkg="))
        .take(MAX_RECORDS_PER_SOURCE)
        .filter_map(|line| {
            let package = extract_value(line, "pkg").or_else(|| extract_package_name(line));
            let package = package?;
            let mut fields = vec![Field::string(0x01, package)];
            if let Some(channel) = extract_value(line, "channel") {
                fields.push(Field::string(0x07, channel));
            }
            if let Some(tag) = extract_value(line, "tag") {
                fields.push(Field::string(0x05, tag));
            }
            Some(Record::new(RecordType::Notification, fields))
        })
        .collect()
}

fn parse_process_records(content: &str) -> Vec<Record> {
    let mut records = Vec::new();
    for line in content.lines().skip(1) {
        if records.len() >= MAX_RECORDS_PER_SOURCE {
            break;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let pid_index = parts.iter().position(|part| part.parse::<i64>().is_ok());
        let Some(pid_index) = pid_index else {
            continue;
        };
        let mut fields = Vec::new();
        if let Ok(pid) = parts[pid_index].parse::<i64>() {
            fields.push(Field::int64(0x01, pid));
        }
        if let Some(ppid) = parts
            .get(pid_index + 1)
            .and_then(|value| value.parse::<i64>().ok())
        {
            fields.push(Field::int64(0x04, ppid));
        }
        if let Some(name) = parts.last() {
            fields.push(Field::string(0x02, *name));
            fields.push(Field::string(0x09, *name));
        }
        records.push(Record::new(RecordType::ProcInfo, fields));
    }
    records
}

fn parse_network_records(content: &str) -> Vec<Record> {
    content
        .lines()
        .filter(|line| line.contains("tcp") || line.contains("udp"))
        .take(MAX_RECORDS_PER_SOURCE)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 4 {
                return None;
            }
            let mut fields = vec![Field::string(0x04, parts[0])];
            push_str_field(&mut fields, 0x01, parts.get(3).copied().unwrap_or_default());
            push_str_field(&mut fields, 0x02, parts.get(4).copied().unwrap_or_default());
            push_str_field(&mut fields, 0x03, parts.get(5).copied().unwrap_or_default());
            Some(Record::new(RecordType::Network, fields))
        })
        .collect()
}

fn parse_logcat_records(content: &str) -> Vec<Record> {
    content
        .lines()
        .take(MAX_RECORDS_PER_SOURCE)
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 6 {
                return None;
            }
            let pid = parts.get(2).and_then(|value| value.parse::<i64>().ok())?;
            let tid = parts
                .get(3)
                .and_then(|value| value.parse::<i64>().ok())
                .unwrap_or(0);
            let priority = parts.get(4).copied().unwrap_or_default();
            let rest = parts[5..].join(" ");
            let (tag, message) = rest
                .split_once(':')
                .map(|(tag, message)| (tag.trim(), message.trim()))
                .unwrap_or(("", rest.trim()));
            Some(Record::new(
                RecordType::LogEntry,
                vec![
                    Field::int64(0x01, pid),
                    Field::int64(0x02, tid),
                    Field::string(0x03, priority),
                    Field::string(0x04, tag),
                    Field::string(0x05, message),
                ],
            ))
        })
        .collect()
}

fn parse_sms_records(content: &str) -> Vec<Record> {
    parse_content_rows(content)
        .into_iter()
        .take(MAX_RECORDS_PER_SOURCE)
        .filter_map(|row| {
            let mut fields = Vec::new();
            push_row_str(&mut fields, 0x01, &row, &["address", "number"]);
            push_row_str(&mut fields, 0x02, &row, &["body", "text"]);
            push_row_int(&mut fields, 0x03, &row, &["type"]);
            push_row_int(&mut fields, 0x04, &row, &["date"]);
            push_row_int(&mut fields, 0x05, &row, &["thread_id"]);
            if let Some(read) = row.get("read").and_then(|value| parse_i64_prefix(value)) {
                fields.push(Field::bool(0x06, read != 0));
            }
            push_row_str(&mut fields, 0x07, &row, &["service_center"]);
            (!fields.is_empty()).then(|| Record::new(RecordType::Sms, fields))
        })
        .collect()
}

fn parse_call_records(content: &str) -> Vec<Record> {
    parse_content_rows(content)
        .into_iter()
        .take(MAX_RECORDS_PER_SOURCE)
        .filter_map(|row| {
            let mut fields = Vec::new();
            push_row_str(&mut fields, 0x01, &row, &["number", "phone_number"]);
            push_row_int(&mut fields, 0x02, &row, &["type"]);
            push_row_int(&mut fields, 0x03, &row, &["duration"]);
            push_row_int(&mut fields, 0x04, &row, &["date"]);
            push_row_str(&mut fields, 0x05, &row, &["name", "cached_name"]);
            push_row_str(
                &mut fields,
                0x06,
                &row,
                &["subscription_component_name", "account_id"],
            );
            (!fields.is_empty()).then(|| Record::new(RecordType::Call, fields))
        })
        .collect()
}

fn parse_contact_records(content: &str) -> Vec<Record> {
    parse_content_rows(content)
        .into_iter()
        .take(MAX_RECORDS_PER_SOURCE)
        .filter_map(|row| {
            let mut fields = Vec::new();
            push_row_str(&mut fields, 0x01, &row, &["_id", "contact_id"]);
            push_row_str(&mut fields, 0x02, &row, &["display_name", "name"]);
            push_row_str(&mut fields, 0x03, &row, &["data1", "number", "phone"]);
            push_row_str(&mut fields, 0x04, &row, &["email", "data4"]);
            push_row_str(&mut fields, 0x05, &row, &["mimetype", "type"]);
            push_row_str(&mut fields, 0x06, &row, &["raw_contact_id"]);
            (!fields.is_empty()).then(|| Record::new(RecordType::Contact, fields))
        })
        .collect()
}

fn parse_media_records(content: &str) -> Vec<Record> {
    parse_content_rows(content)
        .into_iter()
        .take(MAX_RECORDS_PER_SOURCE)
        .filter_map(|row| {
            let mut fields = Vec::new();
            push_row_str(
                &mut fields,
                0x01,
                &row,
                &["_data", "relative_path", "document_id"],
            );
            push_row_int(&mut fields, 0x02, &row, &["_size", "size"]);
            push_row_int(&mut fields, 0x03, &row, &["date_added", "date_modified"]);
            push_row_str(&mut fields, 0x04, &row, &["mime_type"]);
            push_row_str(&mut fields, 0x05, &row, &["_display_name", "title"]);
            push_row_int(&mut fields, 0x07, &row, &["width"]);
            push_row_int(&mut fields, 0x08, &row, &["height"]);
            push_row_int(&mut fields, 0x09, &row, &["duration"]);
            (!fields.is_empty()).then(|| Record::new(RecordType::Media, fields))
        })
        .collect()
}

fn parse_content_rows(content: &str) -> Vec<HashMap<String, String>> {
    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let row_start = line.find("Row:")?;
            let payload = &line[row_start + 4..];
            let payload = payload.trim_start();
            let payload = payload
                .split_once(' ')
                .map(|(_, rest)| rest)
                .unwrap_or(payload);
            let separator = if payload.contains(", ") { ", " } else { " " };
            let mut fields = HashMap::new();
            for part in payload.split(separator) {
                let Some((key, value)) = part.split_once('=') else {
                    continue;
                };
                let value = value.trim().trim_matches('"');
                if !value.is_empty() && value != "null" {
                    fields.insert(key.trim().to_string(), value.to_string());
                }
            }
            (!fields.is_empty()).then_some(fields)
        })
        .collect()
}

fn parse_braced_fields(line: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    let Some(start) = line.find('{') else {
        return fields;
    };
    let Some(end) = line[start + 1..].find('}') else {
        return fields;
    };
    let content = &line[start + 1..start + 1 + end];
    for part in content.split(',') {
        let Some((key, value)) = part.trim().split_once('=') else {
            continue;
        };
        let value = value.trim();
        if !value.is_empty() && value != "null" {
            fields.insert(key.trim().to_string(), value.to_string());
        }
    }
    fields
}

fn parse_configured_wifi_line(line: &str) -> Option<(String, String, String)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    let mut ssid = String::new();
    let mut bssid = String::new();
    let mut security = String::new();
    for (index, part) in parts.iter().enumerate() {
        if *part == "SSID:" {
            ssid = parts
                .get(index + 1)
                .map(|value| value.trim_matches('"').to_string())
                .unwrap_or_default();
        } else if *part == "BSSID:" {
            bssid = parts
                .get(index + 1)
                .filter(|value| **value != "null" && **value != "any")
                .map(|value| (*value).to_string())
                .unwrap_or_default();
        } else if part.contains("WPA") || part.contains("WEP") || part.contains("SAE") {
            security = (*part).to_string();
        }
    }
    normalize_wifi_tuple(ssid, bssid, security)
}

fn parse_connection_wifi_line(line: &str) -> Option<(String, String, String)> {
    let mut ssid = String::new();
    let mut bssid = String::new();
    for part in line.split(',') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("SSID:") {
            ssid = value.trim().trim_matches('"').to_string();
        } else if let Some(value) = part.strip_prefix("BSSID:") {
            let value = value.trim();
            if value != "null" && value != "any" {
                bssid = value.to_string();
            }
        }
    }
    normalize_wifi_tuple(ssid, bssid, String::new())
}

fn normalize_wifi_tuple(
    mut ssid: String,
    bssid: String,
    security: String,
) -> Option<(String, String, String)> {
    if ssid == "<unknown ssid>" {
        ssid.clear();
    }
    if ssid.is_empty() && bssid.is_empty() {
        None
    } else {
        Some((ssid, bssid, security))
    }
}

fn parse_last_known_location(line: &str) -> Option<(String, String, String)> {
    let (provider, rest) = line.split_once(": Location[")?;
    let mut parts = rest.split_whitespace();
    let _provider_name = parts.next()?;
    let coords = parts.next()?;
    let (lat, lon) = coords.split_once(',')?;
    if lat.parse::<f64>().is_err() || lon.parse::<f64>().is_err() {
        return None;
    }
    Some((
        provider.trim().to_string(),
        lat.to_string(),
        lon.to_string(),
    ))
}

fn extract_package_name(line: &str) -> Option<String> {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '.'))
        .find(|word| {
            (word.starts_with("com.") || word.starts_with("org.") || word.starts_with("net."))
                && word.contains('.')
        })
        .map(ToOwned::to_owned)
}

fn extract_value(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    if let Some(rest) = rest.strip_prefix('"') {
        return rest.split_once('"').map(|(value, _)| value.to_string());
    }
    let value = rest
        .split(|ch: char| ch.is_whitespace() || ch == ',' || ch == ']')
        .next()
        .unwrap_or_default()
        .trim_matches('"')
        .to_string();
    (!value.is_empty() && value != "null").then_some(value)
}

fn parse_i64_prefix(value: &str) -> Option<i64> {
    let digits: String = value
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '-')
        .collect();
    digits.parse().ok()
}

fn parse_duration_to_ms(value: &str) -> Option<i64> {
    if let Some(number) = parse_i64_prefix(value) {
        return Some(number);
    }
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() >= 2 && parts.iter().all(|part| part.parse::<i64>().is_ok()) {
        let mut total = 0_i64;
        for part in parts {
            total = total * 60 + part.parse::<i64>().ok()?;
        }
        return Some(total * 1000);
    }
    None
}

fn push_row_str(fields: &mut Vec<Field>, id: u8, row: &HashMap<String, String>, keys: &[&str]) {
    if let Some(value) = keys.iter().find_map(|key| row.get(*key)) {
        push_str_field(fields, id, value);
    }
}

fn push_row_int(fields: &mut Vec<Field>, id: u8, row: &HashMap<String, String>, keys: &[&str]) {
    if let Some(value) = keys
        .iter()
        .find_map(|key| row.get(*key).and_then(|value| parse_i64_prefix(value)))
    {
        fields.push(Field::int64(id, value));
    }
}

fn push_str_field(fields: &mut Vec<Field>, id: u8, value: &str) {
    if !value.trim().is_empty() {
        fields.push(Field::string(id, value.trim()));
    }
}

fn trim_for_record(value: &str, max_len: usize) -> String {
    if value.len() <= max_len {
        return value.to_string();
    }
    let mut out = value[..max_len].to_string();
    out.push_str("\n[truncated]");
    out
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

    #[test]
    fn account_parser_reads_dumpsys_account_records() {
        let input = "\
Accounts: 2
  Account {name=user@example.com, type=com.google}
  Account {name=15551234567, type=com.whatsapp}
";
        let records = parse_account_records(input);
        assert_eq!(records.len(), 2);
        assert!(
            records[0]
                .fields
                .iter()
                .any(|f| f.data == b"user@example.com")
        );
    }

    #[test]
    fn content_row_parser_keeps_android_query_fields() {
        let rows = parse_content_rows(
            "Row: 0 _id=1, address=5551234, body=hello, date=1710000000000, read=1",
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("address").unwrap(), "5551234");
        assert_eq!(rows[0].get("body").unwrap(), "hello");
    }
}
