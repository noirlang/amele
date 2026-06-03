use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const ANDROID_EVENT_LIMIT: usize = 30;
const REPORT_PREVIEW_LIMIT: usize = 4_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidCaseAnalysis {
    pub case_name: String,
    pub android_dir: PathBuf,
    pub files: Vec<AndroidAnalysisFile>,
    pub device_profile: Option<Value>,
    pub record_count: usize,
    pub record_types: Vec<AndroidRecordTypeCount>,
    pub timeline_event_count: usize,
    pub high_severity_count: usize,
    pub recent_events: Vec<Value>,
    pub correlation_count: usize,
    pub volatile_sections: Vec<String>,
    pub bugreport_present: bool,
    pub report_preview: String,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidAnalysisFile {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidRecordTypeCount {
    pub record_type: String,
    pub count: usize,
}

pub fn analyze_android_case(case_name: &str, android_dir: &Path) -> AndroidCaseAnalysis {
    let files = list_files(android_dir);
    let evidence = read_json(android_dir.join("evidence.json"));
    let timeline = read_json(android_dir.join("timeline.json"));
    let correlations = read_json(android_dir.join("correlations.json"));
    let device_profile = read_json(android_dir.join("device_profile.json"));
    let report_preview = read_text_preview(android_dir.join("mobile_report.txt"));
    let volatile_sections = volatile_sections(android_dir.join("android_volatile_data.txt"));
    let bugreport_present = android_dir.join("bugreport.zip").is_file()
        || files
            .iter()
            .any(|file| file.name.ends_with("bugreport.zip"));

    let (record_count, record_types) = evidence_record_summary(evidence.as_ref());
    let (timeline_event_count, high_severity_count, recent_events) =
        timeline_summary(timeline.as_ref());
    let correlation_count = correlations
        .as_ref()
        .and_then(|value| value.get("record_count"))
        .and_then(Value::as_u64)
        .unwrap_or_default() as usize;
    let warnings = android_warnings(
        android_dir,
        record_count,
        timeline_event_count,
        bugreport_present,
        files.len(),
    );
    let recommendations = android_recommendations(
        record_count,
        timeline_event_count,
        correlation_count,
        bugreport_present,
        !volatile_sections.is_empty(),
    );

    AndroidCaseAnalysis {
        case_name: case_name.to_string(),
        android_dir: android_dir.to_path_buf(),
        files,
        device_profile,
        record_count,
        record_types,
        timeline_event_count,
        high_severity_count,
        recent_events,
        correlation_count,
        volatile_sections,
        bugreport_present,
        report_preview,
        warnings,
        recommendations,
    }
}

fn list_files(dir: &Path) -> Vec<AndroidAnalysisFile> {
    let mut files = Vec::new();
    collect_files(dir, dir, &mut files);
    files.sort_by(|left, right| left.name.cmp(&right.name));
    files
}

fn collect_files(root: &Path, dir: &Path, files: &mut Vec<AndroidAnalysisFile>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            collect_files(root, &path, files);
        } else if meta.is_file() {
            let name = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            files.push(AndroidAnalysisFile {
                name,
                path,
                size: meta.len(),
            });
        }
    }
}

fn read_json(path: PathBuf) -> Option<Value> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn read_text_preview(path: PathBuf) -> String {
    let Ok(content) = fs::read_to_string(path) else {
        return String::new();
    };
    content.chars().take(REPORT_PREVIEW_LIMIT).collect()
}

fn evidence_record_summary(evidence: Option<&Value>) -> (usize, Vec<AndroidRecordTypeCount>) {
    let Some(records) = evidence
        .and_then(|value| value.get("records"))
        .and_then(Value::as_array)
    else {
        return (0, Vec::new());
    };
    let mut counts = BTreeMap::new();
    for record in records {
        let record_type = record
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        *counts.entry(record_type).or_insert(0_usize) += 1;
    }
    let mut counts = counts
        .into_iter()
        .map(|(record_type, count)| AndroidRecordTypeCount { record_type, count })
        .collect::<Vec<_>>();
    counts.sort_by(|left, right| right.count.cmp(&left.count));
    (records.len(), counts)
}

fn timeline_summary(timeline: Option<&Value>) -> (usize, usize, Vec<Value>) {
    let Some(events) = timeline
        .and_then(|value| value.get("events"))
        .and_then(Value::as_array)
    else {
        return (0, 0, Vec::new());
    };
    let high = events
        .iter()
        .filter(|event| {
            event
                .get("severity")
                .and_then(Value::as_u64)
                .unwrap_or_default()
                >= 3
        })
        .count();
    (
        events.len(),
        high,
        events.iter().take(ANDROID_EVENT_LIMIT).cloned().collect(),
    )
}

fn volatile_sections(path: PathBuf) -> Vec<String> {
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(|line| line.strip_prefix("## "))
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn android_warnings(
    android_dir: &Path,
    record_count: usize,
    timeline_count: usize,
    bugreport_present: bool,
    file_count: usize,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if !android_dir.is_dir() {
        warnings.push("Android vaka klasoru bulunamadi.".to_string());
    }
    if file_count == 0 {
        warnings.push("Android analiz edilecek cikti bulamadi.".to_string());
    }
    if record_count == 0 {
        warnings.push("Yapisal evidence.json kaydi yok veya bos.".to_string());
    }
    if timeline_count == 0 {
        warnings.push("Timeline olayi yok; logical analiz tamamlanmamis olabilir.".to_string());
    }
    if !bugreport_present {
        warnings.push("Bugreport bulunamadi; sistem durum analizi sinirli kalabilir.".to_string());
    }
    warnings
}

fn android_recommendations(
    record_count: usize,
    timeline_count: usize,
    correlation_count: usize,
    bugreport_present: bool,
    has_volatile: bool,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    if record_count > 0 {
        recommendations
            .push("Kayit turu dagilimini rapordaki bulgu basliklariyla eslestirin.".to_string());
    }
    if timeline_count > 0 {
        recommendations.push("Yuksek onemli timeline olaylarini manuel dogrulayin.".to_string());
    }
    if correlation_count > 0 {
        recommendations
            .push("Arama/mesaj korelasyonlarini kisi kayitlariyla karsilastirin.".to_string());
    }
    if bugreport_present {
        recommendations.push(
            "Bugreport icindeki dumpsys, dumpstate ve logcat bolumlerini inceleyin.".to_string(),
        );
    }
    if has_volatile {
        recommendations.push(
            "Ucusucu veri bolumlerini RAM/proses bulgulariyla birlikte raporlayin.".to_string(),
        );
    }
    recommendations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_android_evidence_records() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("evidence.json"),
            r#"{"records":[{"type":"Sms"},{"type":"Sms"},{"type":"Call"}]}"#,
        )
        .unwrap();
        let report = analyze_android_case("case1", dir.path());
        assert_eq!(report.record_count, 3);
        assert_eq!(report.record_types[0].record_type, "Sms");
        assert_eq!(report.record_types[0].count, 2);
    }
}
