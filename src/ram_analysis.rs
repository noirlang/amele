//! RAM imajlarından string, IOC, proses, entropy ve dosya carving analizleri üretir.
use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

const RAM_SAMPLE_LIMIT: usize = 16 * 1024 * 1024;
const RAM_MATCH_SAMPLE_LIMIT: usize = 80;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// RAM içinde bulunan tek bir dizgi/IOC eşleşmesini offset ve bağlamıyla taşır.
pub struct RamStringMatch {
    pub category: String,
    pub value: String,
    pub offset: u64,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// RAM carving sonucunda çıkarılan dosyanın yol ve imza bilgisini taşır.
pub struct CarvedFile {
    pub file_name: String,
    pub file_path: String,
    pub offset: u64,
    pub size: u64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Volatility veya yerel analizden gelen proses özetini temsil eder.
pub struct ActiveProcessInfo {
    pub pid: String,
    pub name: String,
    pub dump_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Linux proses bellek haritasındaki tek bir adres aralığını temsil eder.
pub struct MemoryMapEntry {
    pub start: String,
    pub end: String,
    pub perms: String,
    pub offset: String,
    pub dev: String,
    pub inode: String,
    pub pathname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// RAM analiz ekranında gösterilen genel özet ve uyarıları taşır.
pub struct RamAnalysisSummary {
    pub file_path: String,
    pub file_name: String,
    pub size: u64,
    pub dump_type: String,
    pub entropy_sample: f64,
    pub string_match_count: usize,
    pub category_counts: Vec<RamCategoryCount>,
    pub sample_matches: Vec<RamStringMatch>,
    pub process_count: usize,
    pub largest_processes: Vec<ActiveProcessInfo>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// IOC/dizgi kategorisi başına bulunan eşleşme sayısını belirtir.
pub struct RamCategoryCount {
    pub category: String,
    pub count: usize,
}

/// Varsayılan ayarlarla RAM imajı için özet analiz üretir.
pub fn analyze_ram_summary(
    file_path: &Path,
    os_type: Option<&str>,
) -> io::Result<RamAnalysisSummary> {
    analyze_ram_summary_logged(file_path, os_type, None)
}

/// Canlı konsola log basabilen RAM özet analizini çalıştırır.
pub fn analyze_ram_summary_logged(
    file_path: &Path,
    os_type: Option<&str>,
    log: Option<Arc<dyn Fn(String) + Send + Sync>>,
) -> io::Result<RamAnalysisSummary> {
    analyze_ram_summary_logged_with_symbol_dir(file_path, os_type, None, log)
}

/// Volatility sembol klasörü seçilerek RAM özet analizini yürütür.
pub fn analyze_ram_summary_logged_with_symbol_dir(
    file_path: &Path,
    os_type: Option<&str>,
    symbol_dir: Option<&Path>,
    log: Option<Arc<dyn Fn(String) + Send + Sync>>,
) -> io::Result<RamAnalysisSummary> {
    if let Some(log) = &log {
        log(format!(
            "RAM analiz dosyası okunuyor: {}",
            file_path.display()
        ));
    }
    let metadata = fs::metadata(file_path)?;
    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();

    let os = os_type.unwrap_or("windows");
    let label = if os == "linux" {
        "Linux Memory (Volatility3)"
    } else {
        "Windows Memory (Volatility3)"
    };

    if let Some(log) = &log {
        log(format!("{label} proses listesi çıkarılıyor"));
    }
    let procs = match crate::volatility::get_processes_logged_with_symbol_dir(
        file_path,
        os,
        symbol_dir,
        log.clone(),
    ) {
        Ok(plist) => plist
            .into_iter()
            .map(|p| ActiveProcessInfo {
                pid: p.pid.to_string(),
                name: format!("{} ({})", p.name, p.offset),
                dump_size: 0,
            })
            .collect(),
        Err(err) => {
            if let Some(log) = &log {
                log(format!("Volatility3 proses analizi uyarısı: {err}"));
            }
            if os == "linux" {
                if let Some(log) = &log {
                    log("Linux sembol eşleşmesi için kernel banner adayları aranıyor.".to_string());
                }
                match crate::volatility::scan_linux_banners(file_path, 1, log.clone()) {
                    Ok(banners) if !banners.is_empty() => warnings.push(format!(
                        "Linux kernel banner adayları bulundu: {}",
                        banners.join(" | ")
                    )),
                    Ok(_) => warnings.push(
                        "Linux kernel banner adayı bulunamadı; imaj türü veya temiz edinim kontrol edilmeli."
                            .to_string(),
                    ),
                    Err(banner_err) => warnings.push(format!(
                        "Linux banner taraması da başarısız oldu: {banner_err}"
                    )),
                }
            }
            warnings.push(format!("Volatility3 error: {}", err));
            Vec::new()
        }
    };
    let largest_processes = procs;
    let dump_type = label.to_string();

    if let Some(log) = &log {
        log("Entropy örneği hesaplanıyor".to_string());
    }
    let entropy_sample = sample_entropy(file_path)?;
    if let Some(log) = &log {
        log("IOC/dizgi taraması başlatıldı".to_string());
    }
    let matches = analyze_ram_strings(file_path)?;
    let mut counts = BTreeMap::new();
    for item in &matches {
        *counts.entry(item.category.clone()).or_insert(0_usize) += 1;
    }
    let category_counts = counts
        .into_iter()
        .map(|(category, count)| RamCategoryCount { category, count })
        .collect::<Vec<_>>();

    let process_count = largest_processes.len();
    let sample_matches = matches
        .iter()
        .take(RAM_MATCH_SAMPLE_LIMIT)
        .cloned()
        .collect::<Vec<_>>();

    let mut native_warnings = ram_warnings(metadata.len(), entropy_sample, &matches);
    warnings.append(&mut native_warnings);

    let mut native_recommendations = ram_recommendations(matches.len());
    recommendations.append(&mut native_recommendations);

    if let Some(log) = &log {
        log(format!(
            "RAM analiz özeti hazır: {} proses, {} IOC/dizgi",
            process_count,
            matches.len()
        ));
    }

    Ok(RamAnalysisSummary {
        file_path: file_path.to_string_lossy().into_owned(),
        file_name: file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
        size: metadata.len(),
        dump_type,
        entropy_sample,
        string_match_count: matches.len(),
        category_counts,
        sample_matches,
        process_count,
        largest_processes: largest_processes.into_iter().take(40).collect(),
        warnings,
        recommendations,
    })
}

/// RAM imajını parça parça okuyup e-posta, IP, URL ve token benzeri dizgileri arar.
pub fn analyze_ram_strings(file_path: &Path) -> io::Result<Vec<RamStringMatch>> {
    let mut file = File::open(file_path)?;

    // Standart adli bilişim regexleri byte düzeyinde çalışır.
    let patterns = vec![
        (
            "E-Posta",
            Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,4}").unwrap(),
        ),
        (
            "IPv4 Adresi",
            Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap(),
        ),
        (
            "URL / Web Adresi",
            Regex::new(r"https?://[a-zA-Z0-9./?=&_-]+").unwrap(),
        ),
        (
            "Alan Adı",
            Regex::new(r"\b[a-zA-Z0-9.-]+\.(?:com|net|org|io|tr|dev|app|gov|edu)\b").unwrap(),
        ),
        ("Telefon Numarası", Regex::new(r"\+?[0-9]{9,15}").unwrap()),
        (
            "Mesajlaşma İzleri",
            Regex::new(
                r"(?:chat\.whatsapp\.com|telegram\.me|t\.me|wa\.me|whatsapp\.net|telegram\.org)",
            )
            .unwrap(),
        ),
        (
            "Kimlik Bilgisi Anahtar Kelimesi",
            Regex::new(r"(?i)(password|passwd|pwd|token|secret|apikey|api_key|authorization)")
                .unwrap(),
        ),
        (
            "JWT Benzeri Token",
            Regex::new(r"eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}").unwrap(),
        ),
        (
            "Dosya Yolu",
            Regex::new(r"(?:[A-Za-z]:\\[A-Za-z0-9_ .\\/-]{6,}|/[A-Za-z0-9_./-]{6,})").unwrap(),
        ),
    ];

    let mut results = Vec::new();
    let chunk_size = 1024 * 1024; // 1 MB parça
    let overlap = 1024; // Sınırdaki eşleşmeleri kaçırmamak için 1 KB örtüşme
    let mut buffer = vec![0_u8; chunk_size];
    let mut offset = 0_u64;

    // Arayüzü boğmamak için kategori başına en fazla 250 eşleşme tutulur.
    let mut category_counts = std::collections::HashMap::new();

    loop {
        file.seek(SeekFrom::Start(offset))?;
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let current_chunk = &buffer[..bytes_read];

        for (category, regex) in &patterns {
            let count = category_counts.entry(category.to_string()).or_insert(0);
            if *count >= 250 {
                continue;
            }

            for mat in regex.find_iter(current_chunk) {
                if *count >= 250 {
                    break;
                }

                let value_bytes = mat.as_bytes();
                if let Ok(value_str) = std::str::from_utf8(value_bytes) {
                    let value_str = value_str.trim().to_string();
                    if value_str.is_empty() || value_str.len() < 4 {
                        continue;
                    }

                    // Bulgunun çevresinden kısa bağlam alınır.
                    let match_start = mat.start();
                    let match_end = mat.end();

                    let context_start = match_start.saturating_sub(20);
                    let context_end = (match_end + 20).min(bytes_read);

                    let context_bytes = &current_chunk[context_start..context_end];
                    let context_str = String::from_utf8_lossy(context_bytes)
                        .chars()
                        .map(|c| {
                            if c.is_ascii_graphic() || c == ' ' {
                                c
                            } else {
                                '.'
                            }
                        })
                        .collect::<String>();

                    results.push(RamStringMatch {
                        category: category.to_string(),
                        value: value_str,
                        offset: offset + match_start as u64,
                        context: context_str,
                    });

                    *count += 1;
                }
            }
        }

        if bytes_read < chunk_size {
            break;
        }
        offset += (chunk_size - overlap) as u64;
    }

    Ok(results)
}

/// Dosyanın ilk bölümünden Shannon entropy örneği hesaplar.
fn sample_entropy(path: &Path) -> io::Result<f64> {
    let mut file = File::open(path)?;
    let mut counts = [0_u64; 256];
    let mut total = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    while total < RAM_SAMPLE_LIMIT as u64 {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = RAM_SAMPLE_LIMIT.saturating_sub(total as usize);
        let usable = read.min(remaining);
        for byte in &buffer[..usable] {
            counts[*byte as usize] += 1;
        }
        total += usable as u64;
    }
    if total == 0 {
        return Ok(0.0);
    }
    let total_f = total as f64;
    let entropy = counts
        .iter()
        .filter(|count| **count > 0)
        .map(|count| {
            let p = *count as f64 / total_f;
            -p * p.log2()
        })
        .sum();
    Ok(entropy)
}

/// RAM imajı boyutu, entropy ve IOC sonucuna göre kullanıcı uyarıları üretir.
fn ram_warnings(size: u64, entropy: f64, matches: &[RamStringMatch]) -> Vec<String> {
    let mut warnings = Vec::new();
    if size < 16 * 1024 * 1024 {
        warnings.push("Bellek dosyası çok küçük; tam bir RAM imajı olmayabilir.".to_string());
    }
    if entropy > 7.7 && matches.is_empty() {
        warnings.push(
            "Yüksek entropi ve az dizgi bulundu; bellek şifrelenmiş veya sıkıştırılmış olabilir."
                .to_string(),
        );
    }
    warnings
}

/// RAM analizinden sonra arayüzde gösterilecek takip önerilerini üretir.
fn ram_recommendations(match_count: usize) -> Vec<String> {
    let mut recommendations = Vec::new();
    recommendations.push(
        "Bulunan ham RAM imajı üzerinde Volatility3 ile süreçleri listeleme ve analiz araçlarını çalıştırın.".to_string()
    );
    if match_count > 0 {
        recommendations
            .push("IOC dizgilerini kategori ve offset bilgisiyle rapora taşıyın.".to_string());
    }
    recommendations
}

/// Magic header/footer imzalarına göre RAM içinden sınırlı dosya carving yapar.
pub fn carve_files(file_path: &Path, output_dir: &Path) -> io::Result<Vec<CarvedFile>> {
    let mut file = File::open(file_path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    // Çıkarılan dosyalar ayrı carved klasörüne yazılır.
    let carved_dir = output_dir.join("carved");
    fs::create_dir_all(&carved_dir)?;

    // Desteklenen dosya imzaları ve güvenli maksimum boyutları tanımlanır.
    struct Signature {
        ext: &'static str,
        mime: &'static str,
        header: &'static [u8],
        footer: Option<&'static [u8]>,
        max_size: usize,
    }

    let signatures = vec![
        Signature {
            ext: "png",
            mime: "image/png",
            header: &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            footer: Some(&[0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82]),
            max_size: 5 * 1024 * 1024, // PNG için 5 MB üst sınır
        },
        Signature {
            ext: "jpg",
            mime: "image/jpeg",
            header: &[0xFF, 0xD8, 0xFF],
            footer: Some(&[0xFF, 0xD9]),
            max_size: 5 * 1024 * 1024, // JPG için 5 MB üst sınır
        },
        Signature {
            ext: "pdf",
            mime: "application/pdf",
            header: &[0x25, 0x50, 0x44, 0x46], // %PDF imzası
            footer: Some(&[0x25, 0x25, 0x45, 0x4F, 0x46]), // %%EOF kapanışı
            max_size: 15 * 1024 * 1024,        // PDF için 15 MB üst sınır
        },
        Signature {
            ext: "zip",
            mime: "application/zip",
            header: &[0x50, 0x4B, 0x03, 0x04], // ZIP başlangıç imzası
            footer: Some(&[0x50, 0x4B, 0x05, 0x06]), // Central directory kapanışı
            max_size: 20 * 1024 * 1024,        // ZIP için 20 MB üst sınır
        },
        Signature {
            ext: "elf",
            mime: "application/octet-stream",
            header: &[0x7F, 0x45, 0x4C, 0x46], // ELF başlangıç imzası
            footer: None,
            max_size: 2 * 1024 * 1024, // ELF için hızlı ön izleme sınırı
        },
    ];

    let mut carved_files = Vec::new();
    let chunk_size = 4 * 1024 * 1024; // 4 MB parça
    let mut buffer = vec![0_u8; chunk_size];
    let mut offset = 0_u64;

    let mut carve_count = 0;

    while offset < file_size {
        file.seek(SeekFrom::Start(offset))?;
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let chunk = &buffer[..bytes_read];
        let mut i = 0;

        while i < bytes_read.saturating_sub(8) {
            for sig in &signatures {
                if chunk[i..].starts_with(sig.header) {
                    let absolute_header_offset = offset + i as u64;

                    // Header bulunduğunda güvenilir dosya sonu için footer aranır.
                    let mut file_len = sig.max_size;

                    if let Some(footer) = sig.footer {
                        // Önce mevcut parça içinde footer taranır.
                        let mut found_footer = false;
                        let scan_start = i + sig.header.len();
                        let scan_end = (i + sig.max_size).min(bytes_read);

                        for j in scan_start..scan_end.saturating_sub(footer.len()) {
                            if chunk[j..].starts_with(footer) {
                                file_len = j + footer.len() - i;
                                found_footer = true;
                                break;
                            }
                        }

                        // Footer yoksa yanlış pozitifleri azaltmak için carving atlanır.
                        if !found_footer && sig.ext != "elf" {
                            // Skip carving if footer not found for reliability
                            continue;
                        }
                    }

                    // İmza aralığı güvenliyse dosya carved klasörüne yazılır.
                    if absolute_header_offset + file_len as u64 <= file_size {
                        let mut carved_data = vec![0_u8; file_len];
                        file.seek(SeekFrom::Start(absolute_header_offset))?;
                        file.read_exact(&mut carved_data)?;

                        carve_count += 1;
                        let name = format!("carved_{offset}_{}.{}", carve_count, sig.ext);
                        let path = carved_dir.join(&name);
                        fs::write(&path, &carved_data)?;

                        carved_files.push(CarvedFile {
                            file_name: name,
                            file_path: path.to_string_lossy().into_owned(),
                            offset: absolute_header_offset,
                            size: file_len as u64,
                            mime_type: sig.mime.to_string(),
                        });

                        // Aynı dosya içinde iç içe imza yakalamamak için ileri sarılır.
                        i += file_len.max(1);
                        break;
                    }
                }
            }
            i += 1;
        }

        // Parça sınırındaki imzaları kaçırmamak için küçük örtüşmeyle ilerlenir.
        offset += (chunk_size - 1024) as u64;

        // Disk taşmasını ve aşırı süreyi önlemek için 100 dosyada durulur.
        if carved_files.len() >= 100 {
            break;
        }
    }

    Ok(carved_files)
}

/// Volatility çalışmadığında ham RAM içinde kullanıcı sorgusunu byte düzeyinde arar.
pub fn search_raw_memory(file_path: &Path, query: &str) -> io::Result<Vec<RamStringMatch>> {
    let mut file = File::open(file_path)?;
    let mut results = Vec::new();
    let query_lower = query.to_ascii_lowercase();
    let query_bytes = query_lower.as_bytes();

    let chunk_size = 4 * 1024 * 1024; // 4 MB parça
    let overlap = query_bytes.len().saturating_sub(1);
    let mut buffer = vec![0_u8; chunk_size];
    let mut offset = 0_u64;

    loop {
        file.seek(SeekFrom::Start(offset))?;
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }

        let current_chunk = &buffer[..bytes_read];
        let mut pos = 0;

        while pos < current_chunk.len().saturating_sub(query_bytes.len()) {
            let window = &current_chunk[pos..pos + query_bytes.len()];
            if window.eq_ignore_ascii_case(query_bytes) {
                let context_start = pos.saturating_sub(20);
                let context_end = (pos + query_bytes.len() + 20).min(bytes_read);
                let context_bytes = &current_chunk[context_start..context_end];

                let context_str = String::from_utf8_lossy(context_bytes)
                    .chars()
                    .map(|c| {
                        if c.is_ascii_graphic() || c == ' ' {
                            c
                        } else {
                            '.'
                        }
                    })
                    .collect::<String>();

                results.push(RamStringMatch {
                    category: "Raw Match".to_string(),
                    value: String::from_utf8_lossy(&current_chunk[pos..pos + query_bytes.len()])
                        .into_owned(),
                    offset: offset + pos as u64,
                    context: context_str,
                });

                if results.len() >= 300 {
                    return Ok(results);
                }
            }
            pos += 1;
        }

        if bytes_read < chunk_size {
            break;
        }
        offset += (chunk_size - overlap) as u64;
    }

    Ok(results)
}
