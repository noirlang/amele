//! ext4 blok tahsis haritası (block allocation bitmap) ayrıştırıcısı.
//! superblock ve block group descriptor'ları okuyarak boş blokları atlar.

use super::ntfs::SparseRange;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// ext4 bölümündeki superblock ve blok bitmap'lerini okuyup aralıkları çıkarır
pub fn parse_ext4_bitmap(
    file: &mut File,
    part_offset: u64,
    part_size: u64,
) -> Option<Vec<SparseRange>> {
    // 1. superblock oku (bölüm başından 1024 bayt sonra, 1024 bayt uzunluğunda)
    let sb_offset = part_offset + 1024;
    file.seek(SeekFrom::Start(sb_offset)).ok()?;
    let mut sb = [0_u8; 1024];
    file.read_exact(&mut sb).ok()?;

    // ext2/3/4 sihirli sayısı kontrolü (0xEF53)
    let magic = u16::from_le_bytes([sb[56], sb[57]]);
    if magic != 0xEF53 {
        return None;
    }

    let s_log_block_size = u32::from_le_bytes(sb[24..28].try_into().ok()?);
    if s_log_block_size > 6 {
        // blok boyutu 64 kb'dan büyük olamaz
        return None;
    }
    let block_size = 1024_u64 << s_log_block_size;

    let blocks_count_lo = u32::from_le_bytes(sb[4..8].try_into().ok()?) as u64;
    let blocks_per_group = u32::from_le_bytes(sb[32..36].try_into().ok()?) as u64;
    if block_size == 0 || blocks_per_group == 0 {
        return None;
    }

    // 64-bit özellik kontrolü
    let s_feature_incompat = u32::from_le_bytes(sb[96..100].try_into().ok()?);
    let is_64bit = (s_feature_incompat & 0x0080) != 0;

    let blocks_count_hi = if is_64bit {
        u32::from_le_bytes(sb[336..340].try_into().ok()?) as u64
    } else {
        0
    };
    let total_blocks = (blocks_count_hi << 32) | blocks_count_lo;
    let total_blocks = if total_blocks > 0 {
        total_blocks
    } else {
        part_size / block_size
    };
    if total_blocks == 0 {
        return None;
    }

    let s_desc_size = if is_64bit {
        let size = u16::from_le_bytes([sb[254], sb[255]]) as usize;
        if size < 64 { 64 } else { size }
    } else {
        32
    };

    let group_count = (total_blocks + blocks_per_group - 1) / blocks_per_group;

    // 2. block group descriptor table (gdt) konumu
    // 1024 baytlık bloklarda superblock 1. bloktadır, gdt 2. blokta başlar.
    // diğer blok boyutlarında superblock 0. bloğun içindedir, gdt 1. blokta başlar.
    let gdt_block = if block_size == 1024 { 2 } else { 1 };
    let gdt_offset = part_offset + (gdt_block * block_size);

    file.seek(SeekFrom::Start(gdt_offset)).ok()?;

    let mut ranges: Vec<SparseRange> = Vec::new();
    let mut current_state: Option<bool> = None;
    let mut current_start_block: u64 = 0;
    let mut current_block_count: u64 = 0;

    let mut desc_buf = vec![0_u8; s_desc_size];

    // her blok grubunu sırayla incele
    for group_idx in 0..group_count {
        file.seek(SeekFrom::Start(
            gdt_offset + (group_idx * s_desc_size as u64),
        ))
        .ok()?;
        if file.read_exact(&mut desc_buf).is_err() {
            break;
        }

        let bg_block_bitmap_lo = u32::from_le_bytes(desc_buf[0..4].try_into().ok()?) as u64;
        let bg_flags = u16::from_le_bytes([desc_buf[22], desc_buf[23]]);

        let bg_block_bitmap_hi = if is_64bit && desc_buf.len() >= 36 {
            u32::from_le_bytes(desc_buf[32..36].try_into().ok()?) as u64
        } else {
            0
        };
        let bitmap_block = (bg_block_bitmap_hi << 32) | bg_block_bitmap_lo;

        // EXT4_BG_BLOCK_UNINIT bayrağı (0x0002)
        // uninitialized ise bu gruptaki tüm bloklar boştur
        let is_uninit = (bg_flags & 0x0002) != 0;

        let blocks_in_this_group = if group_idx == group_count - 1 {
            let rem = total_blocks % blocks_per_group;
            if rem == 0 { blocks_per_group } else { rem }
        } else {
            blocks_per_group
        };

        if is_uninit {
            // grubun tamamı boş
            append_block_states(
                &mut ranges,
                &mut current_state,
                &mut current_start_block,
                &mut current_block_count,
                group_idx * blocks_per_group,
                blocks_in_this_group,
                false,
                part_offset,
                block_size,
            );
            continue;
        }

        // bitmap bloğunu oku
        let bitmap_byte_offset = part_offset + (bitmap_block * block_size);
        let bytes_to_read = ((blocks_in_this_group + 7) / 8) as usize;
        let mut bitmap = vec![0_u8; bytes_to_read];

        let read_success = if file.seek(SeekFrom::Start(bitmap_byte_offset)).is_ok() {
            file.read_exact(&mut bitmap).is_ok()
        } else {
            false
        };

        if !read_success {
            // okuyamazsak veri kaybı olmasın diye grubu dolu kabul et
            append_block_states(
                &mut ranges,
                &mut current_state,
                &mut current_start_block,
                &mut current_block_count,
                group_idx * blocks_per_group,
                blocks_in_this_group,
                true,
                part_offset,
                block_size,
            );
            continue;
        }

        // gruptaki her bloğun doluluk durumunu işle
        for b_idx in 0..blocks_in_this_group {
            let byte_i = (b_idx / 8) as usize;
            let bit_i = (b_idx % 8) as u8;
            let is_allocated = if byte_i < bitmap.len() {
                ((bitmap[byte_i] >> bit_i) & 1) != 0
            } else {
                true
            };

            let global_block = (group_idx * blocks_per_group) + b_idx;
            append_single_block_state(
                &mut ranges,
                &mut current_state,
                &mut current_start_block,
                &mut current_block_count,
                global_block,
                is_allocated,
                part_offset,
                block_size,
            );
        }
    }

    if let Some(state) = current_state {
        if current_block_count > 0 {
            ranges.push(SparseRange {
                offset: part_offset + (current_start_block * block_size),
                length: current_block_count * block_size,
                allocated: state,
            });
        }
    }

    if ranges.is_empty() {
        None
    } else {
        Some(ranges)
    }
}

fn append_single_block_state(
    ranges: &mut Vec<SparseRange>,
    current_state: &mut Option<bool>,
    current_start_block: &mut u64,
    current_block_count: &mut u64,
    global_block: u64,
    is_allocated: bool,
    part_offset: u64,
    block_size: u64,
) {
    match *current_state {
        Some(state) if state == is_allocated => {
            *current_block_count += 1;
        }
        Some(state) => {
            ranges.push(SparseRange {
                offset: part_offset + (*current_start_block * block_size),
                length: *current_block_count * block_size,
                allocated: state,
            });
            *current_state = Some(is_allocated);
            *current_start_block = global_block;
            *current_block_count = 1;
        }
        None => {
            *current_state = Some(is_allocated);
            *current_start_block = global_block;
            *current_block_count = 1;
        }
    }
}

fn append_block_states(
    ranges: &mut Vec<SparseRange>,
    current_state: &mut Option<bool>,
    current_start_block: &mut u64,
    current_block_count: &mut u64,
    global_block_start: u64,
    count: u64,
    is_allocated: bool,
    part_offset: u64,
    block_size: u64,
) {
    if count == 0 {
        return;
    }
    match *current_state {
        Some(state) if state == is_allocated => {
            *current_block_count += count;
        }
        Some(state) => {
            ranges.push(SparseRange {
                offset: part_offset + (*current_start_block * block_size),
                length: *current_block_count * block_size,
                allocated: state,
            });
            *current_state = Some(is_allocated);
            *current_start_block = global_block_start;
            *current_block_count = count;
        }
        None => {
            *current_state = Some(is_allocated);
            *current_start_block = global_block_start;
            *current_block_count = count;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_ext4_magic_verification() {
        // 0xEF53 little-endian baytları
        let magic = [0x53, 0xEF];
        assert_eq!(u16::from_le_bytes(magic), 0xEF53);
    }

    #[test]
    fn test_parse_ext4_invalid_file() {
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        tmp.write_all(&[0u8; 2048]).unwrap();
        let mut file = std::fs::File::open(tmp.path()).unwrap();
        assert!(parse_ext4_bitmap(&mut file, 0, 2048).is_none());
    }
}
