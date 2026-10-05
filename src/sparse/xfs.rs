//! xfs tahsis grubu (allocation group - agf) ve bnobt boş alan ayrıştırıcısı.
//! xfsb superblock ve agf başlıklarını okuyarak boş blok aralıklarını çıkarır.

use super::ntfs::SparseRange;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// xfs bölümündeki agf ve bnobt ağaçlarını okuyup boş aralıkları haritalandırır
pub fn parse_xfs_bitmap(
    file: &mut File,
    part_offset: u64,
    part_size: u64,
) -> Option<Vec<SparseRange>> {
    // 1. xfs superblock'unu oku (bölüm başlangıcı, 512 bayt)
    file.seek(SeekFrom::Start(part_offset)).ok()?;
    let mut sb = [0_u8; 512];
    file.read_exact(&mut sb).ok()?;

    // xfs sihirli sayısı ("XFSB")
    if &sb[0..4] != b"XFSB" {
        return None;
    }

    // xfs alanları big-endian (ağ sırası) saklanır
    let block_size = u32::from_be_bytes(sb[4..8].try_into().ok()?) as u64;
    let dblocks = u64::from_be_bytes(sb[8..16].try_into().ok()?);
    let agblocks = u32::from_be_bytes(sb[84..88].try_into().ok()?) as u64;
    let agcount = u32::from_be_bytes(sb[88..92].try_into().ok()?) as u64;

    if block_size < 512 || block_size > 65536 || agblocks == 0 || agcount == 0 {
        return None;
    }

    let total_size = if dblocks > 0 {
        dblocks * block_size
    } else {
        part_size
    };
    if total_size == 0 {
        return None;
    }

    let mut ranges: Vec<SparseRange> = Vec::new();
    let ag_bytes = agblocks * block_size;

    // her ag (allocation group) için agf başlığını ve bnobt ağacını incele
    for ag_idx in 0..agcount {
        let ag_start_offset = part_offset + (ag_idx * ag_bytes);
        if ag_start_offset >= part_offset + total_size {
            break;
        }

        // agf başlığı sektör 1'de (ofset 512 bayt) yer alır
        let agf_offset = ag_start_offset + 512;
        if file.seek(SeekFrom::Start(agf_offset)).is_err() {
            // okunamazsa grubu dolu say
            ranges.push(SparseRange {
                offset: ag_start_offset,
                length: ag_bytes.min(part_offset + total_size - ag_start_offset),
                allocated: true,
            });
            continue;
        }

        let mut agf = [0_u8; 512];
        if file.read_exact(&mut agf).is_err() || &agf[0..4] != b"XAGF" {
            ranges.push(SparseRange {
                offset: ag_start_offset,
                length: ag_bytes.min(part_offset + total_size - ag_start_offset),
                allocated: true,
            });
            continue;
        }

        let free_blks = u32::from_be_bytes(agf[20..24].try_into().unwrap_or([0; 4])) as u64;
        let bnobt_root = u32::from_be_bytes(agf[28..32].try_into().unwrap_or([0; 4])) as u64;

        if free_blks == 0 {
            // agf tamamen dolu
            ranges.push(SparseRange {
                offset: ag_start_offset,
                length: ag_bytes.min(part_offset + total_size - ag_start_offset),
                allocated: true,
            });
            continue;
        }

        // bnobt (boş blok numarasına göre btree) kök bloğunu oku
        let root_offset = ag_start_offset + (bnobt_root * block_size);
        let mut btree_node = vec![0_u8; block_size as usize];

        let btree_read = if file.seek(SeekFrom::Start(root_offset)).is_ok() {
            file.read_exact(&mut btree_node).is_ok()
        } else {
            false
        };

        if !btree_read {
            ranges.push(SparseRange {
                offset: ag_start_offset,
                length: ag_bytes.min(part_offset + total_size - ag_start_offset),
                allocated: true,
            });
            continue;
        }

        let magic = &btree_node[0..4];
        let is_v4 = magic == b"ABTB";
        let is_v5 = magic == b"AB3B";

        if !is_v4 && !is_v5 {
            // btree tanınamadı, güvenli modda dolu say
            ranges.push(SparseRange {
                offset: ag_start_offset,
                length: ag_bytes.min(part_offset + total_size - ag_start_offset),
                allocated: true,
            });
            continue;
        }

        let level = u16::from_be_bytes(btree_node[4..6].try_into().unwrap_or([0; 2]));
        let numrecs = u16::from_be_bytes(btree_node[6..8].try_into().unwrap_or([0; 2])) as usize;

        // yaprak düğüm (level 0) ise kayıtları doğrudan oku
        if level == 0 {
            let header_size = if is_v5 { 56 } else { 16 };
            let mut free_extents: Vec<(u64, u64)> = Vec::new(); // (start_block, count)

            let mut rec_offset = header_size;
            for _ in 0..numrecs {
                if rec_offset + 8 > btree_node.len() {
                    break;
                }
                let start_b = u32::from_be_bytes(
                    btree_node[rec_offset..rec_offset + 4]
                        .try_into()
                        .unwrap_or([0; 4]),
                ) as u64;
                let count_b = u32::from_be_bytes(
                    btree_node[rec_offset + 4..rec_offset + 8]
                        .try_into()
                        .unwrap_or([0; 4]),
                ) as u64;
                free_extents.push((start_b, count_b));
                rec_offset += 8;
            }

            // ag içindeki blokları dolu/boş olarak aralıklara dönüştür
            let this_ag_blocks = agblocks.min((total_size - (ag_idx * ag_bytes)) / block_size);
            convert_ag_extents_to_ranges(
                &mut ranges,
                ag_start_offset,
                block_size,
                this_ag_blocks,
                &free_extents,
            );
        } else {
            // çok seviyeli derin btree'lerde risk almamak için tüm ag'yi dolu kabul et
            ranges.push(SparseRange {
                offset: ag_start_offset,
                length: ag_bytes.min(part_offset + total_size - ag_start_offset),
                allocated: true,
            });
        }
    }

    if ranges.is_empty() {
        None
    } else {
        Some(merge_adjacent_ranges(ranges))
    }
}

/// ag içindeki boş blok aralıklarını genel sparse range listesine ekler
fn convert_ag_extents_to_ranges(
    ranges: &mut Vec<SparseRange>,
    ag_start_offset: u64,
    block_size: u64,
    ag_total_blocks: u64,
    free_extents: &[(u64, u64)],
) {
    let mut current_block = 0_u64;

    for &(free_start, free_count) in free_extents {
        if free_start > current_block {
            // aradaki bloklar dolu
            let alloc_count = free_start - current_block;
            ranges.push(SparseRange {
                offset: ag_start_offset + (current_block * block_size),
                length: alloc_count * block_size,
                allocated: true,
            });
        }

        // boş bloklar
        ranges.push(SparseRange {
            offset: ag_start_offset + (free_start * block_size),
            length: free_count * block_size,
            allocated: false,
        });

        current_block = free_start + free_count;
    }

    if current_block < ag_total_blocks {
        // kalan son bloklar dolu
        let rem_count = ag_total_blocks - current_block;
        ranges.push(SparseRange {
            offset: ag_start_offset + (current_block * block_size),
            length: rem_count * block_size,
            allocated: true,
        });
    }
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

    #[test]
    fn test_merge_adjacent_ranges() {
        let input = vec![
            SparseRange {
                offset: 0,
                length: 4096,
                allocated: true,
            },
            SparseRange {
                offset: 4096,
                length: 8192,
                allocated: true,
            },
            SparseRange {
                offset: 12288,
                length: 4096,
                allocated: false,
            },
        ];
        let merged = merge_adjacent_ranges(input);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0].length, 12288);
        assert_eq!(merged[0].allocated, true);
        assert_eq!(merged[1].allocated, false);
    }

    #[test]
    fn test_parse_xfs_bitmap_synthetic() {
        use std::io::Write;
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        let mut sb = [0_u8; 512];
        sb[0..4].copy_from_slice(b"XFSB");
        sb[4..8].copy_from_slice(&4096_u32.to_be_bytes());
        sb[8..16].copy_from_slice(&100_u64.to_be_bytes());
        sb[84..88].copy_from_slice(&100_u32.to_be_bytes());
        sb[88..92].copy_from_slice(&1_u32.to_be_bytes());
        tmp.write_all(&sb).unwrap();

        let padding = vec![0_u8; 64 * 1024 - 512];
        tmp.write_all(&padding).unwrap();

        let mut file = std::fs::File::open(tmp.path()).unwrap();
        let res = parse_xfs_bitmap(&mut file, 0, 64 * 1024);
        assert!(res.is_some());
        let ranges = res.unwrap();
        assert_eq!(ranges.len(), 1);
        assert!(ranges[0].allocated);
    }
}
