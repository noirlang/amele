//! ntfs $bitmap ayrıştırıcısı.
//! vbr ve mft record 6'yı okuyup hangi cluster'ların dolu veya boş olduğunu haritalandırır.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// dosya sistemindeki ardışık dolu veya boş sektör aralığı
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseRange {
    /// disk/bölüm başından itibaren mutlak bayt ofseti
    pub offset: u64,
    /// aralığın bayt uzunluğu
    pub length: u64,
    /// true ise dolu (okunacak), false ise boş (atlanacak)
    pub allocated: bool,
}

/// ntfs bölümünü inceleyip dolu/boş cluster aralıklarını çıkarır
pub fn parse_ntfs_bitmap(
    file: &mut File,
    part_offset: u64,
    part_size: u64,
) -> Option<Vec<SparseRange>> {
    // 1. vbr sektörünü oku (512 bayt)
    file.seek(SeekFrom::Start(part_offset)).ok()?;
    let mut vbr = [0_u8; 512];
    file.read_exact(&mut vbr).ok()?;

    // ntfs oem imzasını kontrol et ("NTFS    ")
    if &vbr[3..11] != b"NTFS    " {
        return None;
    }

    let bytes_per_sector = u16::from_le_bytes([vbr[11], vbr[12]]) as u64;
    let sectors_per_cluster = vbr[13] as u64;
    if bytes_per_sector == 0 || sectors_per_cluster == 0 {
        return None;
    }
    let cluster_size = bytes_per_sector * sectors_per_cluster;
    let total_sectors = u64::from_le_bytes(vbr[40..48].try_into().ok()?);
    let volume_clusters = if total_sectors > 0 {
        total_sectors / sectors_per_cluster
    } else {
        part_size / cluster_size
    };
    if volume_clusters == 0 {
        return None;
    }

    // mft başlangıç cluster numarası
    let mft_lcn = u64::from_le_bytes(vbr[48..56].try_into().ok()?);

    // mft kayıt boyutu (genelde 1024 bayt)
    let mft_clusters_per_record = vbr[64] as i8;
    let mft_record_size: u64 = if mft_clusters_per_record > 0 {
        mft_clusters_per_record as u64 * cluster_size
    } else {
        1_u64 << (-mft_clusters_per_record as u32)
    };
    if mft_record_size == 0 || mft_record_size > 65536 {
        return None;
    }

    // 2. mft record 6 ($bitmap) ofsetine git
    // record 0: $mft, record 6: $bitmap
    let bitmap_record_offset = part_offset + (mft_lcn * cluster_size) + (6 * mft_record_size);
    file.seek(SeekFrom::Start(bitmap_record_offset)).ok()?;
    let mut record_buf = vec![0_u8; mft_record_size as usize];
    file.read_exact(&mut record_buf).ok()?;

    // mft başlık imzasını kontrol et ("FILE" veya "BAAD")
    if &record_buf[0..4] != b"FILE" {
        return None;
    }

    // fixup array uygula (ntfs sektör bütünlüğü için)
    apply_mft_fixup(&mut record_buf, bytes_per_sector as usize);

    // 3. mft niteliklerini (attributes) tara, $data (0x80) niteliğini bul
    let first_attr_offset = u16::from_le_bytes([record_buf[20], record_buf[21]]) as usize;
    let bitmap_bytes = extract_bitmap_data(
        file,
        &record_buf,
        first_attr_offset,
        part_offset,
        cluster_size,
        volume_clusters,
    )?;

    // 4. bitmap bitlerini ardışık sparse range'lere dönüştür
    let ranges = bitmap_to_ranges(&bitmap_bytes, part_offset, cluster_size, volume_clusters);
    if ranges.is_empty() {
        None
    } else {
        Some(ranges)
    }
}

/// mft kaydındaki fixup dizisini uygular
fn apply_mft_fixup(buf: &mut [u8], sector_size: usize) {
    if buf.len() < 48 || sector_size == 0 {
        return;
    }
    let fixup_offset = u16::from_le_bytes([buf[4], buf[5]]) as usize;
    let fixup_count = u16::from_le_bytes([buf[6], buf[7]]) as usize;

    if fixup_count <= 1 || fixup_offset + (fixup_count * 2) > buf.len() {
        return;
    }

    // her sektörün son 2 baytını fixup dizisindeki değerlerle güncelle
    for i in 1..fixup_count {
        let sector_end = i * sector_size;
        if sector_end <= buf.len() && sector_end >= 2 {
            let val_idx = fixup_offset + (i * 2);
            buf[sector_end - 2] = buf[val_idx];
            buf[sector_end - 1] = buf[val_idx + 1];
        }
    }
}

/// $bitmap kaydındaki $data niteliğini bulup bitmap baytlarını çıkarır
fn extract_bitmap_data(
    file: &mut File,
    record: &[u8],
    mut offset: usize,
    part_offset: u64,
    cluster_size: u64,
    volume_clusters: u64,
) -> Option<Vec<u8>> {
    let needed_bytes = ((volume_clusters + 7) / 8) as usize;

    while offset + 8 <= record.len() {
        let attr_type = u32::from_le_bytes(record[offset..offset + 4].try_into().ok()?);
        if attr_type == 0xFFFF_FFFF {
            break; // niteliklerin sonu
        }

        let attr_len = u32::from_le_bytes(record[offset + 4..offset + 8].try_into().ok()?) as usize;
        if attr_len == 0 || offset + attr_len > record.len() {
            break;
        }

        // $data niteliği (0x80)
        if attr_type == 0x80 {
            let non_resident = record[offset + 8];
            let name_len = record[offset + 9] as usize;

            // $bitmap'in ana verisi isimsiz (unnamed) $data niteliğindedir
            if name_len == 0 {
                if non_resident == 0 {
                    // resident veri (küçük disklerde nadiren doğrudan mft içinde olur)
                    if offset + 24 <= record.len() {
                        let data_len =
                            u32::from_le_bytes(record[offset + 16..offset + 20].try_into().ok()?)
                                as usize;
                        let data_offset =
                            u16::from_le_bytes([record[offset + 20], record[offset + 21]]) as usize;
                        let start = offset + data_offset;
                        let end = (start + data_len).min(record.len());
                        if start < record.len() {
                            return Some(record[start..end].to_vec());
                        }
                    }
                } else {
                    // non-resident veri (runlist üzerinden okunur)
                    if offset + 64 <= record.len() {
                        let runlist_offset =
                            u16::from_le_bytes([record[offset + 32], record[offset + 33]]) as usize;
                        let real_size =
                            u64::from_le_bytes(record[offset + 48..offset + 56].try_into().ok()?)
                                as usize;

                        let runlist_start = offset + runlist_offset;
                        if runlist_start < record.len() {
                            let runs = parse_runlist(&record[runlist_start..offset + attr_len])?;
                            return read_runs_data(
                                file,
                                &runs,
                                part_offset,
                                cluster_size,
                                real_size.min(needed_bytes),
                            );
                        }
                    }
                }
            }
        }

        offset += attr_len;
    }

    None
}

/// ntfs data run (lcn aralığı) yapısı
#[derive(Debug)]
struct DataRun {
    lcn: Option<u64>, // None ise seyrek (sparse) run
    length_clusters: u64,
}

/// data runlist baytlarını çözer
fn parse_runlist(mut data: &[u8]) -> Option<Vec<DataRun>> {
    let mut runs = Vec::new();
    let mut current_lcn: i64 = 0;

    while !data.is_empty() {
        let header = data[0];
        if header == 0 {
            break; // runlist bitti
        }
        data = &data[1..];

        let len_size = (header & 0x0F) as usize;
        let offset_size = ((header >> 4) & 0x0F) as usize;

        if data.len() < len_size + offset_size {
            return None;
        }

        // küme uzunluğu (pozitif sayı)
        let mut length_clusters: u64 = 0;
        for i in 0..len_size {
            length_clusters |= (data[i] as u64) << (i * 8);
        }
        data = &data[len_size..];

        // küme ofseti (işaretli tam sayı - signed delta)
        let lcn = if offset_size == 0 {
            None // sparse run
        } else {
            let mut offset_val: i64 = 0;
            for i in 0..offset_size {
                offset_val |= (data[i] as i64) << (i * 8);
            }
            // en yüksek bit 1 ise işaret uzatması yap (negative sign extend)
            if (data[offset_size - 1] & 0x80) != 0 {
                for i in offset_size..8 {
                    offset_val |= 0xFF_i64 << (i * 8);
                }
            }
            current_lcn += offset_val;
            data = &data[offset_size..];
            if current_lcn >= 0 {
                Some(current_lcn as u64)
            } else {
                None
            }
        };

        runs.push(DataRun {
            lcn,
            length_clusters,
        });
    }

    Some(runs)
}

/// runlist ile gösterilen disk bloklarından bitmap baytlarını okur
fn read_runs_data(
    file: &mut File,
    runs: &[DataRun],
    part_offset: u64,
    cluster_size: u64,
    max_bytes: usize,
) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(max_bytes);

    for run in runs {
        if out.len() >= max_bytes {
            break;
        }
        let run_bytes = (run.length_clusters * cluster_size) as usize;
        let to_read = run_bytes.min(max_bytes - out.len());

        match run.lcn {
            Some(lcn) => {
                let abs_offset = part_offset + (lcn * cluster_size);
                if file.seek(SeekFrom::Start(abs_offset)).is_ok() {
                    let mut chunk = vec![0_u8; to_read];
                    if file.read_exact(&mut chunk).is_ok() {
                        out.extend_from_slice(&chunk);
                    } else {
                        // okuma hatasında boş sayma, sıfırla doldur
                        out.resize(out.len() + to_read, 0);
                    }
                } else {
                    out.resize(out.len() + to_read, 0);
                }
            }
            None => {
                // seyrek küme (sparse run)
                out.resize(out.len() + to_read, 0);
            }
        }
    }

    Some(out)
}

/// bitmap baytlarını ardışık dolu/boş sektör aralıklarına (SparseRange) çevirir
fn bitmap_to_ranges(
    bitmap: &[u8],
    part_offset: u64,
    cluster_size: u64,
    total_clusters: u64,
) -> Vec<SparseRange> {
    let mut ranges: Vec<SparseRange> = Vec::new();
    let mut current_state: Option<bool> = None;
    let mut current_start_cluster: u64 = 0;
    let mut cluster_count: u64 = 0;

    for cluster_idx in 0..total_clusters {
        let byte_idx = (cluster_idx / 8) as usize;
        let bit_idx = (cluster_idx % 8) as u8;

        let is_allocated = if byte_idx < bitmap.len() {
            ((bitmap[byte_idx] >> bit_idx) & 1) != 0
        } else {
            // bitmap sınırını aştıysa güvenli olarak dolu kabul et
            true
        };

        match current_state {
            Some(state) if state == is_allocated => {
                cluster_count += 1;
            }
            Some(state) => {
                ranges.push(SparseRange {
                    offset: part_offset + (current_start_cluster * cluster_size),
                    length: cluster_count * cluster_size,
                    allocated: state,
                });
                current_state = Some(is_allocated);
                current_start_cluster = cluster_idx;
                cluster_count = 1;
            }
            None => {
                current_state = Some(is_allocated);
                current_start_cluster = cluster_idx;
                cluster_count = 1;
            }
        }
    }

    if let Some(state) = current_state {
        if cluster_count > 0 {
            ranges.push(SparseRange {
                offset: part_offset + (current_start_cluster * cluster_size),
                length: cluster_count * cluster_size,
                allocated: state,
            });
        }
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmap_to_ranges_basic() {
        // 0b00001111: ilk 4 cluster dolu, sonraki 4 cluster boş
        let bitmap = vec![0b0000_1111];
        let ranges = bitmap_to_ranges(&bitmap, 0, 4096, 8);
        assert_eq!(ranges.len(), 2);
        assert_eq!(
            ranges[0],
            SparseRange {
                offset: 0,
                length: 4 * 4096,
                allocated: true,
            }
        );
        assert_eq!(
            ranges[1],
            SparseRange {
                offset: 4 * 4096,
                length: 4 * 4096,
                allocated: false,
            }
        );
    }

    #[test]
    fn test_parse_ntfs_bitmap_synthetic() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let mut vbr = [0_u8; 512];
        vbr[3..11].copy_from_slice(b"NTFS    ");
        vbr[11..13].copy_from_slice(&512_u16.to_le_bytes());
        vbr[13] = 8;
        vbr[40..48].copy_from_slice(&1024_u64.to_le_bytes());
        vbr[48..56].copy_from_slice(&4_u64.to_le_bytes());
        vbr[64] = -10_i8 as u8;
        tmp.write_all(&vbr).unwrap();

        let padding = vec![0_u8; 64 * 1024 - 512];
        tmp.write_all(&padding).unwrap();

        let mut file = std::fs::File::open(tmp.path()).unwrap();
        assert!(parse_ntfs_bitmap(&mut file, 0, 64 * 1024).is_none());
    }
}
