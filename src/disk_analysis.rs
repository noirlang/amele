use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const SECTOR_SIZE: u64 = 512;
const MAX_FS_CANDIDATES: usize = 32;
const MAX_LARGEST_FILES: usize = 20;
const MAX_TREE_ENTRIES: usize = 5_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskImageAnalysis {
    pub image_path: PathBuf,
    pub file_name: String,
    pub size: u64,
    pub extension: String,
    pub image_type: String,
    pub sector_size: u64,
    pub partition_scheme: String,
    pub partitions: Vec<PartitionInfo>,
    pub filesystems: Vec<FileSystemCandidate>,
    pub mounted: Option<MountedImageAnalysis>,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionInfo {
    pub index: u32,
    pub scheme: String,
    pub bootable: bool,
    pub type_code: String,
    pub type_name: String,
    pub start_lba: u64,
    pub sectors: u64,
    pub size: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemCandidate {
    pub offset: u64,
    pub source: String,
    pub fs_type: String,
    pub confidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountedImageAnalysis {
    pub mount_dir: PathBuf,
    pub file_count: usize,
    pub directory_count: usize,
    pub total_visible_bytes: u64,
    pub top_extensions: Vec<ExtensionCount>,
    pub largest_files: Vec<MountedFileEntry>,
    pub scanned_entries: usize,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionCount {
    pub extension: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountedFileEntry {
    pub path: String,
    pub size: u64,
}

pub fn analyze_disk_image(
    image_path: &Path,
    mounted_dir: Option<&Path>,
) -> io::Result<DiskImageAnalysis> {
    let metadata = fs::metadata(image_path)?;
    let mut file = File::open(image_path)?;
    let mut first_sector = [0_u8; SECTOR_SIZE as usize];
    let read = file.read(&mut first_sector)?;
    let first_sector = &first_sector[..read];

    let partitions = parse_partitions(&mut file, first_sector, metadata.len())?;
    let partition_scheme = if partitions.iter().any(|part| part.scheme == "GPT") {
        "GPT"
    } else if partitions.iter().any(|part| part.scheme == "MBR") {
        "MBR"
    } else {
        "Yok / raw volume"
    }
    .to_string();

    let mut filesystems = Vec::new();
    if let Some(candidate) = detect_filesystem_at(&mut file, 0, "image-start")? {
        filesystems.push(candidate);
    }
    for part in &partitions {
        if filesystems.len() >= MAX_FS_CANDIDATES {
            break;
        }
        if part.start_lba == 0 || part.sectors == 0 {
            continue;
        }
        let offset = part.start_lba.saturating_mul(SECTOR_SIZE);
        if let Some(candidate) =
            detect_filesystem_at(&mut file, offset, &format!("partition-{}", part.index))?
        {
            filesystems.push(candidate);
        }
    }

    let image_type = detect_image_type(first_sector, &filesystems);
    let mounted = mounted_dir.and_then(|dir| analyze_mount_dir(dir).ok());
    let warnings = disk_warnings(metadata.len(), &partitions, &filesystems, mounted.as_ref());
    let recommendations = disk_recommendations(&partitions, &filesystems, mounted.as_ref());

    Ok(DiskImageAnalysis {
        image_path: image_path.to_path_buf(),
        file_name: image_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string(),
        size: metadata.len(),
        extension: image_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase(),
        image_type,
        sector_size: SECTOR_SIZE,
        partition_scheme,
        partitions,
        filesystems,
        mounted,
        warnings,
        recommendations,
    })
}

fn parse_partitions(
    file: &mut File,
    first_sector: &[u8],
    image_size: u64,
) -> io::Result<Vec<PartitionInfo>> {
    let mut partitions = parse_mbr_partitions(first_sector, image_size);
    let protective_gpt = partitions
        .iter()
        .any(|part| part.type_code.eq_ignore_ascii_case("0xEE"));
    if protective_gpt {
        let gpt = parse_gpt_partitions(file, image_size)?;
        if !gpt.is_empty() {
            partitions = gpt;
        }
    }
    Ok(partitions)
}

fn parse_mbr_partitions(first_sector: &[u8], image_size: u64) -> Vec<PartitionInfo> {
    if first_sector.len() < 512 || first_sector[510] != 0x55 || first_sector[511] != 0xAA {
        return Vec::new();
    }

    let mut partitions = Vec::new();
    for index in 0..4 {
        let base = 446 + index * 16;
        let entry = &first_sector[base..base + 16];
        let type_code = entry[4];
        let start_lba = u32::from_le_bytes([entry[8], entry[9], entry[10], entry[11]]) as u64;
        let sectors = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as u64;
        if type_code == 0 || sectors == 0 {
            continue;
        }
        let size = sectors.saturating_mul(SECTOR_SIZE);
        let available = image_size.saturating_sub(start_lba.saturating_mul(SECTOR_SIZE));
        partitions.push(PartitionInfo {
            index: (index + 1) as u32,
            scheme: "MBR".to_string(),
            bootable: entry[0] == 0x80,
            type_code: format!("0x{type_code:02X}"),
            type_name: mbr_type_name(type_code).to_string(),
            start_lba,
            sectors,
            size: size.min(available),
            name: String::new(),
        });
    }
    partitions
}

fn parse_gpt_partitions(file: &mut File, image_size: u64) -> io::Result<Vec<PartitionInfo>> {
    let mut header = [0_u8; 512];
    file.seek(SeekFrom::Start(SECTOR_SIZE))?;
    file.read_exact(&mut header)?;
    if &header[0..8] != b"EFI PART" {
        return Ok(Vec::new());
    }

    let entries_lba = u64::from_le_bytes(header[72..80].try_into().unwrap_or([0; 8]));
    let entry_count = u32::from_le_bytes(header[80..84].try_into().unwrap_or([0; 4])) as usize;
    let entry_size = u32::from_le_bytes(header[84..88].try_into().unwrap_or([0; 4])) as usize;
    if entries_lba == 0 || entry_count == 0 || entry_size < 128 || entry_size > 4096 {
        return Ok(Vec::new());
    }

    let mut partitions = Vec::new();
    let max_entries = entry_count.min(256);
    file.seek(SeekFrom::Start(entries_lba.saturating_mul(SECTOR_SIZE)))?;
    for index in 0..max_entries {
        let mut entry = vec![0_u8; entry_size];
        file.read_exact(&mut entry)?;
        if entry[0..16].iter().all(|byte| *byte == 0) {
            continue;
        }
        let first_lba = u64::from_le_bytes(entry[32..40].try_into().unwrap_or([0; 8]));
        let last_lba = u64::from_le_bytes(entry[40..48].try_into().unwrap_or([0; 8]));
        if first_lba == 0 || last_lba < first_lba {
            continue;
        }
        let sectors = last_lba - first_lba + 1;
        let size = sectors.saturating_mul(SECTOR_SIZE);
        let available = image_size.saturating_sub(first_lba.saturating_mul(SECTOR_SIZE));
        partitions.push(PartitionInfo {
            index: (index + 1) as u32,
            scheme: "GPT".to_string(),
            bootable: false,
            type_code: guid_to_string(&entry[0..16]),
            type_name: gpt_type_name(&entry[0..16]).to_string(),
            start_lba: first_lba,
            sectors,
            size: size.min(available),
            name: utf16le_name(&entry[56..entry_size.min(128)]),
        });
    }
    Ok(partitions)
}

fn detect_filesystem_at(
    file: &mut File,
    offset: u64,
    source: &str,
) -> io::Result<Option<FileSystemCandidate>> {
    let mut boot = vec![0_u8; 36 * 1024];
    file.seek(SeekFrom::Start(offset))?;
    let read = file.read(&mut boot)?;
    if read < 16 {
        return Ok(None);
    }
    let boot = &boot[..read];

    let fs_type = if boot.len() > 11 && &boot[3..11] == b"NTFS    " {
        Some("NTFS")
    } else if boot.len() > 11 && &boot[3..11] == b"EXFAT   " {
        Some("exFAT")
    } else if boot.len() > 90 && &boot[82..87] == b"FAT32" {
        Some("FAT32")
    } else if boot.len() > 60 && (&boot[54..59] == b"FAT16" || &boot[54..59] == b"FAT12") {
        Some("FAT")
    } else if boot.len() > 1082 && boot[1080] == 0x53 && boot[1081] == 0xEF {
        Some("ext2/3/4")
    } else if boot.len() > 0x8006 && &boot[0x8001..0x8006] == b"CD001" {
        Some("ISO9660")
    } else {
        None
    };

    Ok(fs_type.map(|fs_type| FileSystemCandidate {
        offset,
        source: source.to_string(),
        fs_type: fs_type.to_string(),
        confidence: "signature".to_string(),
    }))
}

fn analyze_mount_dir(mount_dir: &Path) -> io::Result<MountedImageAnalysis> {
    let mut state = MountScanState::default();
    scan_mount_dir(mount_dir, mount_dir, 0, &mut state)?;
    let mut top_extensions: Vec<ExtensionCount> = state
        .extensions
        .into_iter()
        .map(|(extension, count)| ExtensionCount { extension, count })
        .collect();
    top_extensions.sort_by(|left, right| right.count.cmp(&left.count));
    top_extensions.truncate(12);
    state
        .largest_files
        .sort_by(|left, right| right.size.cmp(&left.size));
    state.largest_files.truncate(MAX_LARGEST_FILES);

    Ok(MountedImageAnalysis {
        mount_dir: mount_dir.to_path_buf(),
        file_count: state.file_count,
        directory_count: state.directory_count,
        total_visible_bytes: state.total_visible_bytes,
        top_extensions,
        largest_files: state.largest_files,
        scanned_entries: state.scanned_entries,
        truncated: state.truncated,
    })
}

#[derive(Default)]
struct MountScanState {
    file_count: usize,
    directory_count: usize,
    total_visible_bytes: u64,
    scanned_entries: usize,
    truncated: bool,
    extensions: BTreeMap<String, usize>,
    largest_files: Vec<MountedFileEntry>,
}

fn scan_mount_dir(
    root: &Path,
    dir: &Path,
    depth: usize,
    state: &mut MountScanState,
) -> io::Result<()> {
    if depth > 5 || state.scanned_entries >= MAX_TREE_ENTRIES {
        state.truncated = true;
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(()),
    };
    for entry in entries.flatten() {
        if state.scanned_entries >= MAX_TREE_ENTRIES {
            state.truncated = true;
            break;
        }
        state.scanned_entries += 1;
        let path = entry.path();
        let Ok(meta) = entry.metadata() else {
            continue;
        };
        if meta.is_dir() {
            state.directory_count += 1;
            scan_mount_dir(root, &path, depth + 1, state)?;
            continue;
        }
        if !meta.is_file() {
            continue;
        }
        state.file_count += 1;
        state.total_visible_bytes = state.total_visible_bytes.saturating_add(meta.len());
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .filter(|ext| !ext.is_empty())
            .unwrap_or_else(|| "(no extension)".to_string());
        *state.extensions.entry(extension).or_default() += 1;
        let rel = path.strip_prefix(root).unwrap_or(&path).to_string_lossy();
        state.largest_files.push(MountedFileEntry {
            path: rel.into_owned(),
            size: meta.len(),
        });
    }
    Ok(())
}

fn detect_image_type(first_sector: &[u8], filesystems: &[FileSystemCandidate]) -> String {
    if first_sector.len() > 6 && &first_sector[0..6] == b"EVF\t\r\n" {
        "EWF/E01".to_string()
    } else if first_sector.len() > 512
        && first_sector.len() > 0x8006
        && &first_sector[0x8001..0x8006] == b"CD001"
    {
        "ISO9660 image".to_string()
    } else if !filesystems.is_empty() {
        "Raw disk/volume image".to_string()
    } else {
        "Unknown or unsupported image".to_string()
    }
}

fn disk_warnings(
    size: u64,
    partitions: &[PartitionInfo],
    filesystems: &[FileSystemCandidate],
    mounted: Option<&MountedImageAnalysis>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if size < SECTOR_SIZE {
        warnings.push("Imaj 512 bayttan kucuk; tamamlanmamis veya gecersiz olabilir.".to_string());
    }
    if partitions.is_empty() && filesystems.is_empty() {
        warnings.push("Bolum tablosu veya dosya sistemi imzasi bulunamadi.".to_string());
    }
    if let Some(mounted) = mounted
        && mounted.truncated
    {
        warnings.push("Bagli klasor ozeti sinirlandirildi; cok fazla girdi var.".to_string());
    }
    warnings
}

fn disk_recommendations(
    partitions: &[PartitionInfo],
    filesystems: &[FileSystemCandidate],
    mounted: Option<&MountedImageAnalysis>,
) -> Vec<String> {
    let mut recommendations = Vec::new();
    if !partitions.is_empty() {
        recommendations.push("Bolum baslangic LBA ve boyutlarini rapora ekleyin.".to_string());
    }
    if !filesystems.is_empty() {
        recommendations
            .push("Dosya sistemi imzalariyla mount edilen bolumu karsilastirin.".to_string());
    }
    if mounted.is_none() {
        recommendations.push("Icerik incelemesi icin imaji salt-okunur baglayin.".to_string());
    }
    recommendations
}

fn mbr_type_name(code: u8) -> &'static str {
    match code {
        0x01 | 0x04 | 0x06 | 0x0E => "FAT",
        0x07 => "NTFS/exFAT/HPFS",
        0x0B | 0x0C => "FAT32",
        0x82 => "Linux swap",
        0x83 => "Linux filesystem",
        0x8E => "Linux LVM",
        0xA5 => "FreeBSD",
        0xAF => "Apple HFS/HFS+",
        0xEE => "GPT protective MBR",
        0xEF => "EFI System",
        _ => "Unknown",
    }
}

fn gpt_type_name(guid: &[u8]) -> &'static str {
    let guid = guid_to_string(guid);
    match guid.as_str() {
        "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7" => "Microsoft Basic Data",
        "E3C9E316-0B5C-4DB8-817D-F92DF00215AE" => "Microsoft Reserved",
        "DE94BBA4-06D1-4D40-A16A-BFD50179D6AC" => "Windows Recovery",
        "0FC63DAF-8483-4772-8E79-3D69D8477DE4" => "Linux filesystem",
        "0657FD6D-A4AB-43C4-84E5-0933C84B4F4F" => "Linux swap",
        "E6D6D379-F507-44C2-A23C-238F2A3DF928" => "Linux LVM",
        "C12A7328-F81F-11D2-BA4B-00A0C93EC93B" => "EFI System",
        "48465300-0000-11AA-AA11-00306543ECAC" => "Apple HFS/HFS+",
        _ => "Unknown",
    }
}

fn guid_to_string(bytes: &[u8]) -> String {
    if bytes.len() < 16 {
        return String::new();
    }
    format!(
        "{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_le_bytes([bytes[4], bytes[5]]),
        u16::from_le_bytes([bytes[6], bytes[7]]),
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

fn utf16le_name(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .take_while(|unit| *unit != 0)
        .collect();
    String::from_utf16_lossy(&units).trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn detects_mbr_partition() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("disk.img");
        let mut image = vec![0_u8; 4096];
        image[446] = 0x80;
        image[450] = 0x83;
        image[454..458].copy_from_slice(&1_u32.to_le_bytes());
        image[458..462].copy_from_slice(&7_u32.to_le_bytes());
        image[510] = 0x55;
        image[511] = 0xAA;
        File::create(&path).unwrap().write_all(&image).unwrap();

        let report = analyze_disk_image(&path, None).unwrap();
        assert_eq!(report.partition_scheme, "MBR");
        assert_eq!(report.partitions.len(), 1);
        assert_eq!(report.partitions[0].type_name, "Linux filesystem");
    }
}
