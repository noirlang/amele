//! exfat dosya sistemi blok tahsis bitmap (allocation bitmap) ayrıştırıcısı.
//! vbr ve root directory içindeki 0x81 (allocation bitmap) girdisini okuyarak
//! küme (cluster) bazlı dolu/boş haritasını çıkarır.

use super::ntfs::SparseRange;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// exfat bölümündeki vbr ve allocation bitmap'i inceleyerek seyrek aralıkları üretir
pub fn parse_exfat_bitmap(
    file: &mut File,
    part_offset: u64,
    part_size: u64,
) -> Option<Vec<SparseRange>> {
    // 1. vbr sektörünü oku (512 bayt)
    file.seek(SeekFrom::Start(part_offset)).ok()?;
    let mut vbr = [0_u8; 512];
    file.read_exact(&mut vbr).ok()?;

    // standart imza kontrolü (0x55, 0xAA)
    if vbr[510] != 0x55 || vbr[511] != 0xAA {
        return None;
    }

    // exfat oem adı kontrolü ("EXFAT   ")
    if &vbr[3..11] != b"EXFAT   " {
        return None;
    }

    // sektör boyutu (1 << bytes_per_sector_shift), tipik olarak 9 (512) veya 12 (4096)
    let bytes_per_sector_shift = vbr[108];
    if !(9..=12).contains(&bytes_per_sector_shift) {
        return None;
    }
    let bytes_per_sector = 1_u64 << bytes_per_sector_shift;

    // küme (cluster) boyutu: bytes_per_sector << sectors_per_cluster_shift
    let sectors_per_cluster_shift = vbr[109];
    if sectors_per_cluster_shift > (25 - bytes_per_sector_shift) {
        return None;
    }
    let cluster_size = bytes_per_sector << sectors_per_cluster_shift;
    if cluster_size == 0 {
        return None;
    }

    let cluster_heap_offset_sectors = u32::from_le_bytes(vbr[88..92].try_into().ok()?) as u64;
    let cluster_count = u32::from_le_bytes(vbr[92..96].try_into().ok()?) as u64;
    let first_cluster_root = u32::from_le_bytes(vbr[96..100].try_into().ok()?);

    if cluster_count == 0 || first_cluster_root < 2 || cluster_heap_offset_sectors == 0 {
        return None;
    }

    let cluster_heap_byte_offset = part_offset + (cluster_heap_offset_sectors * bytes_per_sector);

    // 2. root directory içindeki 0x81 (allocation bitmap) girdisini ara
    let root_dir_offset =
        cluster_heap_byte_offset + ((first_cluster_root as u64 - 2) * cluster_size);

    file.seek(SeekFrom::Start(root_dir_offset)).ok()?;

    // root dizininde ilk kümedeki girdileri tara (max 512 girdi = 16 kb)
    let max_entries = std::cmp::min(cluster_size / 32, 512) as usize;
    let mut dir_buf = vec![0_u8; max_entries * 32];
    file.read_exact(&mut dir_buf).ok()?;

    let mut bitmap_first_cluster = 0_u32;
    let mut bitmap_data_len = 0_u64;
    let mut found_bitmap = false;

    for entry in dir_buf.chunks_exact(32) {
        let entry_type = entry[0];
        if entry_type == 0x00 {
            // dizin sonu
            break;
        }
        // 0x81: allocation bitmap directory entry
        if entry_type == 0x81 {
            let bitmap_flags = entry[1];
            // bit 0: 0 ise ilk (ana) allocation bitmap
            if (bitmap_flags & 1) == 0 {
                bitmap_first_cluster = u32::from_le_bytes(entry[20..24].try_into().ok()?);
                bitmap_data_len = u64::from_le_bytes(entry[24..32].try_into().ok()?);
                found_bitmap = true;
                break;
            }
        }
    }

    if !found_bitmap || bitmap_first_cluster < 2 || bitmap_data_len == 0 {
        return None;
    }

    let mut ranges = Vec::new();

    // 3. cluster heap öncesi alan (vbr, fat tabloları, meta veriler) - adli bütünlük için mutlaka dolu say
    let heap_start_rel = cluster_heap_offset_sectors * bytes_per_sector;
    if heap_start_rel > 0 {
        ranges.push(SparseRange {
            offset: part_offset,
            length: heap_start_rel,
            allocated: true,
        });
    }

    // 4. allocation bitmap oku ve kümelerin dolu/boş durumunu haritalandır
    let bitmap_byte_offset =
        cluster_heap_byte_offset + ((bitmap_first_cluster as u64 - 2) * cluster_size);
    file.seek(SeekFrom::Start(bitmap_byte_offset)).ok()?;

    let expected_bytes = (cluster_count + 7) / 8;
    let read_bytes_total = std::cmp::min(bitmap_data_len, expected_bytes);

    let mut current_state: Option<bool> = None;
    let mut current_cluster_start = 0_u64;
    let mut current_cluster_count = 0_u64;

    let mut processed_clusters = 0_u64;
    let mut buf = [0_u8; 65536];
    let mut bytes_left = read_bytes_total;

    while bytes_left > 0 && processed_clusters < cluster_count {
        let to_read = std::cmp::min(bytes_left, buf.len() as u64) as usize;
        file.read_exact(&mut buf[..to_read]).ok()?;
        bytes_left -= to_read as u64;

        for &byte in &buf[..to_read] {
            for bit in 0..8 {
                if processed_clusters >= cluster_count {
                    break;
                }
                let is_allocated = (byte & (1 << bit)) != 0;
                match current_state {
                    Some(state) if state == is_allocated => {
                        current_cluster_count += 1;
                    }
                    Some(state) => {
                        let offset =
                            cluster_heap_byte_offset + (current_cluster_start * cluster_size);
                        let length = current_cluster_count * cluster_size;
                        ranges.push(SparseRange {
                            offset,
                            length,
                            allocated: state,
                        });
                        current_state = Some(is_allocated);
                        current_cluster_start = processed_clusters;
                        current_cluster_count = 1;
                    }
                    None => {
                        current_state = Some(is_allocated);
                        current_cluster_start = processed_clusters;
                        current_cluster_count = 1;
                    }
                }
                processed_clusters += 1;
            }
        }
    }

    if let Some(state) = current_state {
        if current_cluster_count > 0 {
            let offset = cluster_heap_byte_offset + (current_cluster_start * cluster_size);
            let length = current_cluster_count * cluster_size;
            ranges.push(SparseRange {
                offset,
                length,
                allocated: state,
            });
        }
    }

    // 5. cluster heap sonrası bölüm sonu boşluğu varsa dolu say
    let heap_end = cluster_heap_byte_offset + (cluster_count * cluster_size);
    let part_end = part_offset + part_size;
    if part_end > heap_end {
        ranges.push(SparseRange {
            offset: heap_end,
            length: part_end - heap_end,
            allocated: true,
        });
    }

    Some(merge_adjacent_ranges(ranges))
}

/// ardışık aynı durumdaki (dolu-dolu veya boş-boş) aralıkları birleştirir
fn merge_adjacent_ranges(ranges: Vec<SparseRange>) -> Vec<SparseRange> {
    let mut merged = Vec::new();
    for r in ranges {
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
    fn test_merge_adjacent_ranges_exfat() {
        let input = vec![
            SparseRange {
                offset: 0,
                length: 100,
                allocated: true,
            },
            SparseRange {
                offset: 100,
                length: 200,
                allocated: true,
            },
            SparseRange {
                offset: 300,
                length: 50,
                allocated: false,
            },
            SparseRange {
                offset: 350,
                length: 50,
                allocated: false,
            },
        ];
        let merged = merge_adjacent_ranges(input);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].offset, 0);
        assert_eq!(merged[0].length, 300);
        assert!(merged[0].allocated);
        assert_eq!(merged[1].offset, 300);
        assert_eq!(merged[1].length, 100);
        assert!(!merged[1].allocated);
    }

    #[test]
    fn test_parse_exfat_bitmap_synthetic() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        // 512 bayt VBR oluştur
        let mut vbr = [0_u8; 512];
        vbr[0..3].copy_from_slice(&[0xEB, 0x76, 0x90]);
        vbr[3..11].copy_from_slice(b"EXFAT   ");
        vbr[108] = 9; // bytes_per_sector_shift: 9 => 512 bayt
        vbr[109] = 1; // sectors_per_cluster_shift: 1 => 2 sektor = 1024 bayt cluster
        vbr[88..92].copy_from_slice(&10_u32.to_le_bytes()); // cluster_heap_offset_sectors: 10 sektor = 5120 bayt
        vbr[92..96].copy_from_slice(&16_u32.to_le_bytes()); // cluster_count: 16 kume
        vbr[96..100].copy_from_slice(&2_u32.to_le_bytes()); // first_cluster_root: 2
        vbr[510] = 0x55;
        vbr[511] = 0xAA;

        file.write_all(&vbr).unwrap();

        // Heap başlangıcına kadar olan kısmı (5120 - 512 = 4608 bayt) doldur
        let padding = vec![0_u8; 5120 - 512];
        file.write_all(&padding).unwrap();

        // Cluster 2: Root Directory (1024 bayt)
        // Root directory içinde 0x81 (allocation bitmap) girdisi oluştur
        let mut root_dir = vec![0_u8; 1024];
        root_dir[0] = 0x81; // EntryType: Allocation Bitmap
        root_dir[1] = 0x00; // Flags: 0 (first bitmap)
        root_dir[20..24].copy_from_slice(&3_u32.to_le_bytes()); // FirstCluster: 3
        root_dir[24..32].copy_from_slice(&2_u64.to_le_bytes()); // DataLength: 2 bayt (16 kume)

        file.write_all(&root_dir).unwrap();

        // Cluster 3: Allocation Bitmap (1024 bayt)
        // 16 kume icin 2 bayt bitmap:
        // Byte 0: 0b00000011 (ilk 2 kume dolu [root dir ve bitmap], sonraki 6 kume bos)
        // Byte 1: 0b11110000 (sonraki 4 kume bos, son 4 kume dolu)
        let mut bitmap_cluster = vec![0_u8; 1024];
        bitmap_cluster[0] = 0b0000_0011;
        bitmap_cluster[1] = 0b1111_0000;
        file.write_all(&bitmap_cluster).unwrap();

        // Kalan kumeleri doldur (14 kume * 1024 = 14336 bayt)
        let rem_data = vec![0_u8; 14 * 1024];
        file.write_all(&rem_data).unwrap();
        file.flush().unwrap();

        let total_size = 5120 + (16 * 1024);
        let mut f = file.reopen().unwrap();
        let ranges = parse_exfat_bitmap(&mut f, 0, total_size).unwrap();

        // Beklenen:
        // 1. Meta bolgesi (0..5120): allocated: true
        // 2. Cluster 0..1 (offset 5120..7168, 2 kume): allocated: true
        //    (Bu bolum meta bolgesiyle bitisik oldugu icin birlestirilir: 0..7168, allocated: true)
        // 3. Bos kumeler: 6 kume (offset 7168..13312) + 4 kume (offset 13312..17408) = 10 kume (10240 bayt), allocated: false
        // 4. Dolu kumeler: 4 kume (offset 17408..21504, 4096 bayt), allocated: true
        assert_eq!(ranges.len(), 3);
        assert_eq!(ranges[0].offset, 0);
        assert_eq!(ranges[0].length, 5120 + 2048);
        assert!(ranges[0].allocated);

        assert_eq!(ranges[1].offset, 7168);
        assert_eq!(ranges[1].length, 10 * 1024);
        assert!(!ranges[1].allocated);

        assert_eq!(ranges[2].offset, 7168 + 10240);
        assert_eq!(ranges[2].length, 4 * 1024);
        assert!(ranges[2].allocated);
    }
}
