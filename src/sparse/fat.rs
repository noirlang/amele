//! fat16 / fat32 dosya sistemi blok tahsis ayrıştırıcısı.
//! vbr ve fat tablosunu okuyarak dosya sistemi küme (cluster) bazlı dolu/boş haritasını çıkarır.

use super::ntfs::SparseRange;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// fat16 veya fat32 bölümündeki fat tablosunu inceleyerek seyrek aralıkları üretir
pub fn parse_fat_bitmap(
    file: &mut File,
    part_offset: u64,
    part_size: u64,
) -> Option<Vec<SparseRange>> {
    // 1. vbr sektörünü oku (512 bayt)
    file.seek(SeekFrom::Start(part_offset)).ok()?;
    let mut vbr = [0_u8; 512];
    file.read_exact(&mut vbr).ok()?;

    // standart önyükleme imzası kontrolü (0x55, 0xAA)
    if vbr[510] != 0x55 || vbr[511] != 0xAA {
        return None;
    }

    let bytes_per_sector = u16::from_le_bytes([vbr[11], vbr[12]]) as u64;
    if ![512, 1024, 2048, 4096].contains(&bytes_per_sector) {
        return None;
    }

    let sectors_per_cluster = vbr[13] as u64;
    if sectors_per_cluster == 0 || !sectors_per_cluster.is_power_of_two() {
        return None;
    }
    let cluster_size = bytes_per_sector * sectors_per_cluster;

    let reserved_sectors = u16::from_le_bytes([vbr[14], vbr[15]]) as u64;
    let number_of_fats = vbr[16] as u64;
    if reserved_sectors == 0 || number_of_fats == 0 {
        return None;
    }

    let root_entries_16 = u16::from_le_bytes([vbr[17], vbr[18]]) as u64;
    let total_sectors_16 = u16::from_le_bytes([vbr[19], vbr[20]]) as u64;
    let fat_size_16 = u16::from_le_bytes([vbr[21], vbr[22]]) as u64;
    let total_sectors_32 = u32::from_le_bytes(vbr[32..36].try_into().ok()?) as u64;

    let total_sectors = if total_sectors_16 != 0 {
        total_sectors_16
    } else {
        total_sectors_32
    };
    if total_sectors == 0 {
        return None;
    }

    // fat32 veya fat16 türünü ve fat boyutunu belirle
    let (is_fat32, fat_size_sectors, root_dir_sectors) = if fat_size_16 == 0 {
        // fat32
        let fat_size_32 = u32::from_le_bytes(vbr[36..40].try_into().ok()?) as u64;
        if fat_size_32 == 0 {
            return None;
        }
        (true, fat_size_32, 0_u64)
    } else {
        // fat16 (veya fat12)
        let root_sectors = ((root_entries_16 * 32) + (bytes_per_sector - 1)) / bytes_per_sector;
        (false, fat_size_16, root_sectors)
    };

    let metadata_sectors =
        reserved_sectors + (number_of_fats * fat_size_sectors) + root_dir_sectors;
    if metadata_sectors >= total_sectors {
        return None;
    }

    let data_sectors = total_sectors - metadata_sectors;
    let total_clusters = data_sectors / sectors_per_cluster;
    if total_clusters == 0 {
        return None;
    }

    // fat12 desteğini atla (adli bilişimde disk imajı için geçerli değildir)
    if !is_fat32 && total_clusters < 4085 {
        return None;
    }

    let mut ranges = Vec::new();

    // 2. meta veri bölgesi (vbr, ayrılmış sektörler, fat kopyaları, kök dizin) - adli bütünlük için dolu say
    let metadata_bytes = metadata_sectors * bytes_per_sector;
    ranges.push(SparseRange {
        offset: part_offset,
        length: metadata_bytes,
        allocated: true,
    });

    let fat_table_offset = part_offset + (reserved_sectors * bytes_per_sector);
    let data_area_offset = part_offset + metadata_bytes;

    // 3. fat tablosunu oku ve kümelerin dolu/boş durumunu haritalandır
    file.seek(SeekFrom::Start(fat_table_offset)).ok()?;

    let mut current_state: Option<bool> = None;
    let mut current_cluster_start = 0_u64;
    let mut current_cluster_count = 0_u64;

    if is_fat32 {
        // fat32: her küme girdisi 4 bayttır (28 bit kullanılır)
        // ilk 2 küme (0 ve 1) rezerve olduğu için 2. kümeden başlanır
        let entry_size = 4;
        let mut cluster_index = 0_u64; // 0..total_clusters (fiziksel küme 2 + cluster_index)

        // ilk 2 rezerve girişi atla (8 bayt)
        file.seek(SeekFrom::Start(fat_table_offset + (2 * entry_size as u64)))
            .ok()?;

        let mut buf = [0_u8; 65536];
        let mut clusters_left = total_clusters;

        while clusters_left > 0 {
            let max_clusters_in_buf = buf.len() / entry_size;
            let clusters_to_read =
                std::cmp::min(clusters_left, max_clusters_in_buf as u64) as usize;
            let bytes_to_read = clusters_to_read * entry_size;

            file.read_exact(&mut buf[..bytes_to_read]).ok()?;
            clusters_left -= clusters_to_read as u64;

            for chunk in buf[..bytes_to_read].chunks_exact(entry_size) {
                let entry_val = u32::from_le_bytes(chunk.try_into().unwrap()) & 0x0FFF_FFFF;
                // fat32'de 0x00000000 boş kümedir, sıfırdan farklı her değer doludur
                let is_allocated = entry_val != 0;

                match current_state {
                    Some(state) if state == is_allocated => {
                        current_cluster_count += 1;
                    }
                    Some(state) => {
                        let offset = data_area_offset + (current_cluster_start * cluster_size);
                        let length = current_cluster_count * cluster_size;
                        ranges.push(SparseRange {
                            offset,
                            length,
                            allocated: state,
                        });
                        current_state = Some(is_allocated);
                        current_cluster_start = cluster_index;
                        current_cluster_count = 1;
                    }
                    None => {
                        current_state = Some(is_allocated);
                        current_cluster_start = cluster_index;
                        current_cluster_count = 1;
                    }
                }
                cluster_index += 1;
            }
        }
    } else {
        // fat16: her küme girdisi 2 bayttır
        let entry_size = 2;
        let mut cluster_index = 0_u64;

        // ilk 2 rezerve girişi atla (4 bayt)
        file.seek(SeekFrom::Start(fat_table_offset + (2 * entry_size as u64)))
            .ok()?;

        let mut buf = [0_u8; 65536];
        let mut clusters_left = total_clusters;

        while clusters_left > 0 {
            let max_clusters_in_buf = buf.len() / entry_size;
            let clusters_to_read =
                std::cmp::min(clusters_left, max_clusters_in_buf as u64) as usize;
            let bytes_to_read = clusters_to_read * entry_size;

            file.read_exact(&mut buf[..bytes_to_read]).ok()?;
            clusters_left -= clusters_to_read as u64;

            for chunk in buf[..bytes_to_read].chunks_exact(entry_size) {
                let entry_val = u16::from_le_bytes(chunk.try_into().unwrap());
                // fat16'da 0x0000 boş kümedir
                let is_allocated = entry_val != 0;

                match current_state {
                    Some(state) if state == is_allocated => {
                        current_cluster_count += 1;
                    }
                    Some(state) => {
                        let offset = data_area_offset + (current_cluster_start * cluster_size);
                        let length = current_cluster_count * cluster_size;
                        ranges.push(SparseRange {
                            offset,
                            length,
                            allocated: state,
                        });
                        current_state = Some(is_allocated);
                        current_cluster_start = cluster_index;
                        current_cluster_count = 1;
                    }
                    None => {
                        current_state = Some(is_allocated);
                        current_cluster_start = cluster_index;
                        current_cluster_count = 1;
                    }
                }
                cluster_index += 1;
            }
        }
    }

    if let Some(state) = current_state {
        if current_cluster_count > 0 {
            let offset = data_area_offset + (current_cluster_start * cluster_size);
            let length = current_cluster_count * cluster_size;
            ranges.push(SparseRange {
                offset,
                length,
                allocated: state,
            });
        }
    }

    // 4. bölüm sonu hizalama boşluğu varsa dolu say
    let data_end = data_area_offset + (total_clusters * cluster_size);
    let part_end = part_offset + part_size;
    if part_end > data_end {
        ranges.push(SparseRange {
            offset: data_end,
            length: part_end - data_end,
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
    fn test_parse_fat32_synthetic() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        // 512 bayt VBR
        let mut vbr = [0_u8; 512];
        vbr[11..13].copy_from_slice(&512_u16.to_le_bytes()); // bytes_per_sector: 512
        vbr[13] = 1; // sectors_per_cluster: 1 sektor (512 bayt)
        vbr[14..16].copy_from_slice(&32_u16.to_le_bytes()); // reserved_sectors: 32 sektor
        vbr[16] = 2; // number_of_fats: 2
        vbr[32..36].copy_from_slice(&100_u32.to_le_bytes()); // total_sectors_32: 100 sektor
        vbr[36..40].copy_from_slice(&2_u32.to_le_bytes()); // fat_size_32: 2 sektor
        vbr[510] = 0x55;
        vbr[511] = 0xAA;
        file.write_all(&vbr).unwrap();

        // Ayrılmış sektörlerin kalanını (31 sektör * 512 = 15872 bayt) yaz
        let res_padding = vec![0_u8; 31 * 512];
        file.write_all(&res_padding).unwrap();

        // FAT 1 (2 sektör = 1024 bayt)
        // metadata_sectors = 32 + (2 * 2) = 36 sektor.
        // data_sectors = 100 - 36 = 64 sektor = 64 kume.
        // Cluster 0 ve 1 rezerve (8 bayt)
        let mut fat = vec![0_u8; 1024];
        fat[0..4].copy_from_slice(&0x0FFF_FFF8_u32.to_le_bytes()); // cluster 0 media
        fat[4..8].copy_from_slice(&0x0FFF_FFFF_u32.to_le_bytes()); // cluster 1 eoc

        // Cluster 2: Dolu (örn. root dir)
        fat[8..12].copy_from_slice(&0x0FFF_FFFF_u32.to_le_bytes());
        // Cluster 3: Dolu
        fat[12..16].copy_from_slice(&0x0000_0004_u32.to_le_bytes());
        // Cluster 4..10: Boş (0)
        // Cluster 11: Dolu
        fat[44..48].copy_from_slice(&0x0FFF_FFFF_u32.to_le_bytes());

        file.write_all(&fat).unwrap();

        // FAT 2 (2 sektör = 1024 bayt)
        file.write_all(&fat).unwrap();

        // Data alanı (64 sektör * 512 = 32768 bayt)
        let data = vec![0_u8; 64 * 512];
        file.write_all(&data).unwrap();
        file.flush().unwrap();

        let total_size = 100 * 512;
        let mut f = file.reopen().unwrap();
        let ranges = parse_fat_bitmap(&mut f, 0, total_size).unwrap();

        // Metadata: 36 sektor = 18432 bayt (allocated: true)
        // Cluster 2..3 (2 kume = 1024 bayt) dolu -> metadata ile birlesir: 18432 + 1024 = 19456 bayt (allocated: true)
        // Cluster 4..10 (7 kume = 3584 bayt) bos (allocated: false)
        // Cluster 11 (1 kume = 512 bayt) dolu (allocated: true)
        // Cluster 12..65 (54 kume = 27648 bayt) bos (allocated: false)
        assert_eq!(ranges[0].offset, 0);
        assert_eq!(ranges[0].length, 19456);
        assert!(ranges[0].allocated);

        assert_eq!(ranges[1].offset, 19456);
        assert_eq!(ranges[1].length, 3584);
        assert!(!ranges[1].allocated);

        assert_eq!(ranges[2].offset, 19456 + 3584);
        assert_eq!(ranges[2].length, 512);
        assert!(ranges[2].allocated);

        assert_eq!(ranges[3].offset, 19456 + 3584 + 512);
        assert_eq!(ranges[3].length, 27648);
        assert!(!ranges[3].allocated);
    }

    #[test]
    fn test_parse_fat16_synthetic() {
        let mut file = tempfile::NamedTempFile::new().unwrap();

        // 512 bayt VBR
        let mut vbr = [0_u8; 512];
        vbr[11..13].copy_from_slice(&512_u16.to_le_bytes()); // bytes_per_sector: 512
        vbr[13] = 1; // sectors_per_cluster: 1 sektor
        vbr[14..16].copy_from_slice(&1_u16.to_le_bytes()); // reserved_sectors: 1
        vbr[16] = 2; // number_of_fats: 2
        vbr[17..19].copy_from_slice(&512_u16.to_le_bytes()); // root_entries_16: 512 girdi (32 sektor)
        let total_secs = 1 + (2 * 32) + 32 + 5000_u32; // 5097 sektor
        vbr[19..21].copy_from_slice(&(total_secs as u16).to_le_bytes()); // total_sectors_16
        vbr[21..22].copy_from_slice(&32_u8.to_le_bytes()); // fat_size_16: 32 sektor
        vbr[510] = 0x55;
        vbr[511] = 0xAA;
        file.write_all(&vbr).unwrap();

        // FAT 1 (32 sektor = 16384 bayt)
        // Her girdi 2 bayt. Ilk 2 kume rezerve (4 bayt)
        let mut fat = vec![0_u8; 32 * 512];
        fat[0..2].copy_from_slice(&0xFFF8_u16.to_le_bytes());
        fat[2..4].copy_from_slice(&0xFFFF_u16.to_le_bytes());
        // Cluster 2: Dolu
        fat[4..6].copy_from_slice(&0xFFFF_u16.to_le_bytes());
        // Diger kumeler bos (0)
        file.write_all(&fat).unwrap();

        // FAT 2
        file.write_all(&fat).unwrap();

        // Root dir (32 sektor)
        let root_dir = vec![0_u8; 32 * 512];
        file.write_all(&root_dir).unwrap();

        // Data alani (5000 sektor * 512 bayt)
        let data = vec![0_u8; 5000 * 512];
        file.write_all(&data).unwrap();
        file.flush().unwrap();

        let total_size = total_secs as u64 * 512;
        let mut f = file.reopen().unwrap();
        let ranges = parse_fat_bitmap(&mut f, 0, total_size).unwrap();

        assert!(!ranges.is_empty());
        assert!(ranges[0].allocated);
        assert!(!ranges[1].allocated);
    }
}
