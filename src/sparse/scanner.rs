//! mbr ve gpt bölümlerini tarayarak tüm disk için birleşik seyrek harita oluşturan modül.

use super::exfat::parse_exfat_bitmap;
use super::ext4::parse_ext4_bitmap;
use super::fat::parse_fat_bitmap;
use super::ntfs::{SparseRange, parse_ntfs_bitmap};
use super::xfs::parse_xfs_bitmap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

const SECTOR_SIZE: u64 = 512;

/// disk üzerindeki bölüm girdisi
#[derive(Debug, Clone)]
pub struct DetectedPartition {
    pub start_offset: u64,
    pub length: u64,
}

/// tüm disk veya bölüm için eksiksiz seyrek alan haritası üretir
pub fn build_disk_sparse_map(file: &mut File, total_disk_size: u64) -> Vec<SparseRange> {
    build_disk_sparse_map_ext(file, total_disk_size).0
}

/// tüm disk veya bölüm için eksiksiz seyrek alan haritası, desteklenen bölüm sayısı ve tespit edilen dosya sistemlerini üretir
pub fn build_disk_sparse_map_ext(
    file: &mut File,
    total_disk_size: u64,
) -> (Vec<SparseRange>, usize, Vec<String>) {
    if total_disk_size == 0 {
        return (Vec::new(), 0, Vec::new());
    }

    // 1. diskteki bölümleri tespit et
    let partitions = detect_partitions(file, total_disk_size);

    if partitions.is_empty() {
        // bölüm tablosu yoksa doğrudan disk başlangıcında dosya sistemi ara
        if let Some((fs_name, ranges)) = identify_and_scan_volume(file, 0, total_disk_size) {
            return (ranges, 1, vec![fs_name.to_string()]);
        }

        // linux çekirdek seviyesi seek_hole / seek_data dene (sadece gerçek delikler varsa)
        #[cfg(unix)]
        if let Some(ranges) = scan_kernel_holes(file, total_disk_size) {
            let has_hole = ranges.iter().any(|r| !r.allocated);
            if has_hole {
                return (ranges, 1, vec!["sparse-hole".to_string()]);
            }
        }

        // fallback: hiçbir harita çıkarılamadıysa tüm diski dolu say
        return (
            vec![SparseRange {
                offset: 0,
                length: total_disk_size,
                allocated: true,
            }],
            0,
            Vec::new(),
        );
    }

    let mut full_ranges: Vec<SparseRange> = Vec::new();
    let mut current_offset: u64 = 0;
    let mut supported_volumes_found = 0usize;
    let mut detected_filesystems = Vec::new();

    for part in partitions {
        // bölümden önceki alan (mbr, gpt tabloları veya boşluklar)
        if part.start_offset > current_offset {
            let gap_len = part.start_offset - current_offset;
            // adli bütünlük için bölüm öncesi başlıkları/tabloları mutlaka dolu (okunacak) say
            full_ranges.push(SparseRange {
                offset: current_offset,
                length: gap_len,
                allocated: true,
            });
        }

        // bölümün içindeki dosya sistemini tara
        let part_ranges = if let Some((fs_name, ranges)) =
            identify_and_scan_volume(file, part.start_offset, part.length)
        {
            supported_volumes_found += 1;
            detected_filesystems.push(fs_name.to_string());
            ranges
        } else {
            // dosya sistemi desteklenmiyorsa veya şifreliyse bölümün tamamını güvenle oku
            vec![SparseRange {
                offset: part.start_offset,
                length: part.length,
                allocated: true,
            }]
        };

        full_ranges.extend(part_ranges);
        current_offset = part.start_offset + part.length;
    }

    // son bölümden sonraki disk sonu alanı (gpt ikincil başlığı vb.)
    if current_offset < total_disk_size {
        full_ranges.push(SparseRange {
            offset: current_offset,
            length: total_disk_size - current_offset,
            allocated: true,
        });
    }

    (
        merge_ranges(full_ranges),
        supported_volumes_found,
        detected_filesystems,
    )
}

/// tek bir bölüm veya raw volume içindeki dosya sistemi bitmap'ini inceler ve dosya sistemi adıyla birlikte döner
pub fn identify_and_scan_volume(
    file: &mut File,
    part_offset: u64,
    part_size: u64,
) -> Option<(&'static str, Vec<SparseRange>)> {
    // 1. ntfs dene
    if let Some(ranges) = parse_ntfs_bitmap(file, part_offset, part_size) {
        return Some(("NTFS", ranges));
    }

    // 2. ext4 dene
    if let Some(ranges) = parse_ext4_bitmap(file, part_offset, part_size) {
        return Some(("ext4", ranges));
    }

    // 3. xfs dene
    if let Some(ranges) = parse_xfs_bitmap(file, part_offset, part_size) {
        return Some(("XFS", ranges));
    }

    // 4. exfat dene
    if let Some(ranges) = parse_exfat_bitmap(file, part_offset, part_size) {
        return Some(("exFAT", ranges));
    }

    // 5. fat (fat16/fat32) dene
    if let Some(ranges) = parse_fat_bitmap(file, part_offset, part_size) {
        return Some(("FAT", ranges));
    }

    None
}

/// tek bir bölüm veya raw volume içindeki dosya sistemi bitmap'ini inceler
pub fn scan_single_volume(
    file: &mut File,
    part_offset: u64,
    part_size: u64,
) -> Option<Vec<SparseRange>> {
    identify_and_scan_volume(file, part_offset, part_size).map(|(_, ranges)| ranges)
}

/// gpt ve mbr bölüm tablolarını okur
fn detect_partitions(file: &mut File, total_size: u64) -> Vec<DetectedPartition> {
    let mut partitions = Vec::new();

    // sektör 0 oku (mbr veya protective mbr)
    if file.seek(SeekFrom::Start(0)).is_err() {
        return partitions;
    }
    let mut mbr = [0_u8; 512];
    if file.read_exact(&mut mbr).is_err() || mbr[510] != 0x55 || mbr[511] != 0xAA {
        return partitions;
    }

    // 1. gpt kontrolü (sektör 1 = lba 1)
    if file.seek(SeekFrom::Start(SECTOR_SIZE)).is_ok() {
        let mut gpt_hdr = [0_u8; 512];
        if file.read_exact(&mut gpt_hdr).is_ok() && &gpt_hdr[0..8] == b"EFI PART" {
            let part_entry_lba = u64::from_le_bytes(gpt_hdr[72..80].try_into().unwrap_or([0; 8]));
            let num_entries =
                u32::from_le_bytes(gpt_hdr[80..84].try_into().unwrap_or([0; 4])) as usize;
            let entry_size =
                u32::from_le_bytes(gpt_hdr[84..88].try_into().unwrap_or([0; 4])) as usize;

            if part_entry_lba > 0 && num_entries > 0 && entry_size >= 128 {
                let table_offset = part_entry_lba * SECTOR_SIZE;
                if file.seek(SeekFrom::Start(table_offset)).is_ok() {
                    let mut entry_buf = vec![0_u8; entry_size];
                    for _ in 0..num_entries.min(128) {
                        if file.read_exact(&mut entry_buf).is_err() {
                            break;
                        }
                        // ilk 16 bayt partition type guid (hepsi sıfırsa kullanılmayan girdi)
                        if entry_buf[0..16].iter().all(|&b| b == 0) {
                            continue;
                        }
                        let first_lba =
                            u64::from_le_bytes(entry_buf[32..40].try_into().unwrap_or([0; 8]));
                        let last_lba =
                            u64::from_le_bytes(entry_buf[40..48].try_into().unwrap_or([0; 8]));

                        if last_lba >= first_lba && first_lba > 0 {
                            let start_offset = first_lba * SECTOR_SIZE;
                            let length = (last_lba - first_lba + 1) * SECTOR_SIZE;
                            if start_offset + length <= total_size {
                                partitions.push(DetectedPartition {
                                    start_offset,
                                    length,
                                });
                            }
                        }
                    }
                }
            }

            if !partitions.is_empty() {
                partitions.sort_by_key(|p| p.start_offset);
                return partitions;
            }
        }
    }

    // 2. klasik mbr bölüm tablosu kontrolü
    for i in 0..4 {
        let entry_offset = 446 + (i * 16);
        let entry = &mbr[entry_offset..entry_offset + 16];
        let p_type = entry[4];
        let start_lba = u32::from_le_bytes(entry[8..12].try_into().unwrap_or([0; 4])) as u64;
        let sector_count = u32::from_le_bytes(entry[12..16].try_into().unwrap_or([0; 4])) as u64;

        // protective mbr (0xEE) değilse ve geçerli sektör sayısı varsa
        if p_type != 0 && p_type != 0xEE && start_lba > 0 && sector_count > 0 {
            let start_offset = start_lba * SECTOR_SIZE;
            let length = sector_count * SECTOR_SIZE;
            if start_offset + length <= total_size {
                partitions.push(DetectedPartition {
                    start_offset,
                    length,
                });
            }
        }
    }

    partitions.sort_by_key(|p| p.start_offset);
    partitions
}

#[cfg(unix)]
/// linux kernel seek_hole / seek_data desteği ile seyrek aralıkları tespit eder
fn scan_kernel_holes(file: &File, total_size: u64) -> Option<Vec<SparseRange>> {
    use std::os::unix::io::AsRawFd;

    let fd = file.as_raw_fd();
    let mut ranges = Vec::new();
    let mut current_offset: libc::off64_t = 0;

    // SEEK_DATA = 3, SEEK_HOLE = 4 (linux çekirdeği)
    const SEEK_DATA: libc::c_int = 3;
    const SEEK_HOLE: libc::c_int = 4;

    while (current_offset as u64) < total_size {
        let data_offset = unsafe { libc::lseek64(fd, current_offset, SEEK_DATA) };
        if data_offset < 0 {
            // veri bulunamadı, kalan alan delik (hole)
            if (current_offset as u64) < total_size {
                ranges.push(SparseRange {
                    offset: current_offset as u64,
                    length: total_size - (current_offset as u64),
                    allocated: false,
                });
            }
            break;
        }

        if data_offset > current_offset {
            // aradaki alan boş (hole)
            ranges.push(SparseRange {
                offset: current_offset as u64,
                length: (data_offset - current_offset) as u64,
                allocated: false,
            });
        }

        let hole_offset = unsafe { libc::lseek64(fd, data_offset, SEEK_HOLE) };
        let end_of_data = if hole_offset < 0 || (hole_offset as u64) > total_size {
            total_size as libc::off64_t
        } else {
            hole_offset
        };

        if end_of_data > data_offset {
            ranges.push(SparseRange {
                offset: data_offset as u64,
                length: (end_of_data - data_offset) as u64,
                allocated: true,
            });
        }

        current_offset = end_of_data;
    }

    if ranges.is_empty() {
        None
    } else {
        Some(merge_ranges(ranges))
    }
}

/// komşu aynı durumdaki aralıkları birleştirir
fn merge_ranges(ranges: Vec<SparseRange>) -> Vec<SparseRange> {
    let mut merged = Vec::new();
    for r in ranges {
        if r.length == 0 {
            continue;
        }
        if let Some(last) = merged.last_mut() {
            let last_range: &mut SparseRange = last;
            if last_range.allocated == r.allocated
                && last_range.offset + last_range.length == r.offset
            {
                last_range.length += r.length;
                continue;
            }
        }
        merged.push(r);
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_build_disk_sparse_map_raw_fallback() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&[0x11; 4096]).unwrap();
        let mut file = File::open(tmp.path()).unwrap();
        let map = build_disk_sparse_map(&mut file, 4096);
        assert_eq!(map.len(), 1);
        assert_eq!(map[0].length, 4096);
        assert!(map[0].allocated);
    }

    #[test]
    fn test_scan_single_volume_exfat_detected() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let mut vbr = [0_u8; 512];
        vbr[0..3].copy_from_slice(&[0xEB, 0x76, 0x90]);
        vbr[3..11].copy_from_slice(b"EXFAT   ");
        vbr[108] = 9;
        vbr[109] = 1;
        vbr[88..92].copy_from_slice(&10_u32.to_le_bytes());
        vbr[92..96].copy_from_slice(&16_u32.to_le_bytes());
        vbr[96..100].copy_from_slice(&2_u32.to_le_bytes());
        vbr[510] = 0x55;
        vbr[511] = 0xAA;
        tmp.write_all(&vbr).unwrap();

        let padding = vec![0_u8; 5120 - 512];
        tmp.write_all(&padding).unwrap();

        let mut root_dir = vec![0_u8; 1024];
        root_dir[0] = 0x81;
        root_dir[1] = 0x00;
        root_dir[20..24].copy_from_slice(&3_u32.to_le_bytes());
        root_dir[24..32].copy_from_slice(&2_u64.to_le_bytes());
        tmp.write_all(&root_dir).unwrap();

        let mut bitmap = vec![0_u8; 1024];
        bitmap[0] = 0xFF;
        bitmap[1] = 0x00;
        tmp.write_all(&bitmap).unwrap();

        let rem = vec![0_u8; 14 * 1024];
        tmp.write_all(&rem).unwrap();
        tmp.flush().unwrap();

        let mut file = File::open(tmp.path()).unwrap();
        let total = 5120 + 16 * 1024;
        let ranges = scan_single_volume(&mut file, 0, total);
        assert!(ranges.is_some());
        let r = ranges.unwrap();
        assert!(!r.is_empty());
    }

    #[test]
    fn test_scan_single_volume_fat32_detected() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let mut vbr = [0_u8; 512];
        vbr[11..13].copy_from_slice(&512_u16.to_le_bytes());
        vbr[13] = 1;
        vbr[14..16].copy_from_slice(&32_u16.to_le_bytes());
        vbr[16] = 2;
        vbr[32..36].copy_from_slice(&100_u32.to_le_bytes());
        vbr[36..40].copy_from_slice(&2_u32.to_le_bytes());
        vbr[510] = 0x55;
        vbr[511] = 0xAA;
        tmp.write_all(&vbr).unwrap();

        let res_padding = vec![0_u8; 31 * 512];
        tmp.write_all(&res_padding).unwrap();

        let mut fat = vec![0_u8; 1024];
        fat[0..4].copy_from_slice(&0x0FFF_FFF8_u32.to_le_bytes());
        fat[4..8].copy_from_slice(&0x0FFF_FFFF_u32.to_le_bytes());
        fat[8..12].copy_from_slice(&0x0FFF_FFFF_u32.to_le_bytes());
        tmp.write_all(&fat).unwrap();
        tmp.write_all(&fat).unwrap();

        let data = vec![0_u8; 64 * 512];
        tmp.write_all(&data).unwrap();
        tmp.flush().unwrap();

        let mut file = File::open(tmp.path()).unwrap();
        let ranges = scan_single_volume(&mut file, 0, 100 * 512);
        assert!(ranges.is_some());
    }
}
