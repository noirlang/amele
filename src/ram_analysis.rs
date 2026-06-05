use regex::bytes::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

const RAM_SAMPLE_LIMIT: usize = 16 * 1024 * 1024;
const RAM_MATCH_SAMPLE_LIMIT: usize = 80;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamStringMatch {
    pub category: String,
    pub value: String,
    pub offset: u64,
    pub context: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CarvedFile {
    pub file_name: String,
    pub file_path: String,
    pub offset: u64,
    pub size: u64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveProcessInfo {
    pub pid: String,
    pub name: String,
    pub dump_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct RamCategoryCount {
    pub category: String,
    pub count: usize,
}

pub fn analyze_ram_summary(file_path: &Path, os_type: Option<&str>) -> io::Result<RamAnalysisSummary> {
    let metadata = fs::metadata(file_path)?;
    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();

    let os = os_type.unwrap_or("windows");
    let label = if os == "linux" { "Linux Memory (Volatility3)" } else { "Windows Memory (Volatility3)" };
    
    let procs = match crate::volatility::get_processes(file_path, os) {
        Ok(plist) => plist.into_iter().map(|p| ActiveProcessInfo {
            pid: p.pid.to_string(),
            name: format!("{} ({})", p.name, p.offset),
            dump_size: 0,
        }).collect(),
        Err(err) => {
            warnings.push(format!("Volatility3 error: {}", err));
            Vec::new()
        }
    };
    let largest_processes = procs;
    let dump_type = label.to_string();

    let entropy_sample = sample_entropy(file_path)?;
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

/// Analyze a memory dump (or process dump) for volatile string patterns.
pub fn analyze_ram_strings(file_path: &Path) -> io::Result<Vec<RamStringMatch>> {
    let mut file = File::open(file_path)?;

    // Define standard forensic regexes on bytes
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
    let chunk_size = 1024 * 1024; // 1 MB chunks
    let overlap = 1024; // 1 KB overlap
    let mut buffer = vec![0_u8; chunk_size];
    let mut offset = 0_u64;

    // Maintain unique matches per category to avoid cluttering (limit to 250 each)
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

                    // Extract context (up to 20 bytes before and after)
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

fn ram_warnings(
    size: u64,
    entropy: f64,
    matches: &[RamStringMatch],
) -> Vec<String> {
    let mut warnings = Vec::new();
    if size < 16 * 1024 * 1024 {
        warnings.push("Bellek dosyası çok küçük; tam bir RAM imajı olmayabilir.".to_string());
    }
    if entropy > 7.7 && matches.is_empty() {
        warnings.push("Yüksek entropi ve az dizgi bulundu; bellek şifrelenmiş veya sıkıştırılmış olabilir.".to_string());
    }
    warnings
}

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

/// Carves files out of a raw RAM dump based on binary magic headers and footers.
pub fn carve_files(file_path: &Path, output_dir: &Path) -> io::Result<Vec<CarvedFile>> {
    let mut file = File::open(file_path)?;
    let metadata = file.metadata()?;
    let file_size = metadata.len();

    // Dynamic output subdirectory for carved assets
    let carved_dir = output_dir.join("carved");
    fs::create_dir_all(&carved_dir)?;

    // Define file signatures
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
            max_size: 5 * 1024 * 1024, // 5MB max PNG
        },
        Signature {
            ext: "jpg",
            mime: "image/jpeg",
            header: &[0xFF, 0xD8, 0xFF],
            footer: Some(&[0xFF, 0xD9]),
            max_size: 5 * 1024 * 1024, // 5MB max JPG
        },
        Signature {
            ext: "pdf",
            mime: "application/pdf",
            header: &[0x25, 0x50, 0x44, 0x46],             // %PDF
            footer: Some(&[0x25, 0x25, 0x45, 0x4F, 0x46]), // %%EOF
            max_size: 15 * 1024 * 1024,                    // 15MB max PDF
        },
        Signature {
            ext: "zip",
            mime: "application/zip",
            header: &[0x50, 0x4B, 0x03, 0x04],       // PK\x03\x04
            footer: Some(&[0x50, 0x4B, 0x05, 0x06]), // End of Central Directory
            max_size: 20 * 1024 * 1024,              // 20MB max ZIP
        },
        Signature {
            ext: "elf",
            mime: "application/octet-stream",
            header: &[0x7F, 0x45, 0x4C, 0x46], // \x7fELF
            footer: None,
            max_size: 2 * 1024 * 1024, // Carve 2MB of ELF
        },
    ];

    let mut carved_files = Vec::new();
    let chunk_size = 4 * 1024 * 1024; // 4 MB chunks
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

                    // Found a header! Let's search for the footer
                    let mut file_len = sig.max_size;

                    if let Some(footer) = sig.footer {
                        // Scan within chunk or seek forward to find the footer
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

                        // If not found in current chunk, and file_size allows, we could search further.
                        // But for speed & memory safety, scanning 4MB window is typically excellent.
                        if !found_footer && sig.ext != "elf" {
                            // Skip carving if footer not found for reliability
                            continue;
                        }
                    }

                    // Perform carving
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

                        // Advance beyond this file to avoid nested matches
                        i += file_len.max(1);
                        break;
                    }
                }
            }
            i += 1;
        }

        // Advance by chunk size minus overlap to ensure we don't miss headers at chunk borders
        offset += (chunk_size - 1024) as u64;

        // Stop carving after 100 files to avoid disk overflow or massive runtimes
        if carved_files.len() >= 100 {
            break;
        }
    }

    Ok(carved_files)
}



/// Search volatile strings within a raw memory image (fallback when not a process tar archive).
pub fn search_raw_memory(
    file_path: &Path,
    query: &str,
) -> io::Result<Vec<RamStringMatch>> {
    let mut file = File::open(file_path)?;
    let mut results = Vec::new();
    let query_lower = query.to_ascii_lowercase();
    let query_bytes = query_lower.as_bytes();

    let chunk_size = 4 * 1024 * 1024; // 4 MB chunks
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
                    value: String::from_utf8_lossy(&current_chunk[pos..pos + query_bytes.len()]).into_owned(),
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
