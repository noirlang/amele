//! akıllı seyrek alan (sparse / bitmap) atlamalı edinim motoru.
//! ntfs $bitmap, ext4 ve xfs blok tahsis haritalarını okuyarak veri olmayan boş sektörleri
//! fiziksel okuma yapmadan doğrudan atlar ve hedefte seyrek dosya (sparse hole) açar.

pub mod exfat;
pub mod ext4;
pub mod fat;
pub mod ntfs;
pub mod scanner;
pub mod xfs;

pub use ntfs::SparseRange;
pub use scanner::build_disk_sparse_map;

use crate::disk::{
    DiskAcquisitionControl, DiskAcquisitionResult, DiskAcquisitionTask, mark_partial,
};
use crate::error::{AmeleError, AmeleResult, HataKodu};
use crate::hash::{to_hex, write_blake3_sidecar, write_sha256_sidecar};
use crate::logging::{LogLevel, runtime_log};
use digest::Digest;
use sha2::Sha256;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::thread;
use std::time::Duration;

/// sabit sıfır buffer boyutu (hash hesaplamasını ram'de hızlandırmak için)
const ZERO_BUFFER_SIZE: usize = 1024 * 1024; // 1 mb

/// diskin seyrek haritası ve istatistik özeti
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseMap {
    pub ranges: Vec<SparseRange>,
    pub total_size: u64,
    pub allocated_bytes: u64,
    pub unallocated_bytes: u64,
}

impl SparseMap {
    /// disk dosyasını inceleyerek seyrek harita oluşturur
    pub fn from_file(file: &mut File, total_size: u64) -> Self {
        let ranges = build_disk_sparse_map(file, total_size);
        let mut allocated_bytes = 0_u64;
        let mut unallocated_bytes = 0_u64;

        for r in &ranges {
            if r.allocated {
                allocated_bytes += r.length;
            } else {
                unallocated_bytes += r.length;
            }
        }

        Self {
            ranges,
            total_size,
            allocated_bytes,
            unallocated_bytes,
        }
    }

    /// boş alan tasarruf oranını yüzdelik olarak döndürür (0.0 .. 100.0)
    pub fn savings_percentage(&self) -> f64 {
        if self.total_size == 0 {
            0.0
        } else {
            (self.unallocated_bytes as f64 / self.total_size as f64) * 100.0
        }
    }

    /// atlanabilir boş alan olup olmadığını belirtir
    pub fn has_sparse_areas(&self) -> bool {
        self.unallocated_bytes > 0
    }
}

/// akıllı seyrek disk edinimi işlemini çalıştırır
pub fn run_sparse_acquisition<F, C>(
    task: &DiskAcquisitionTask,
    source_size: u64,
    mut progress: F,
    mut control: C,
) -> AmeleResult<DiskAcquisitionResult>
where
    F: FnMut(u64, u64),
    C: FnMut() -> DiskAcquisitionControl,
{
    runtime_log(
        LogLevel::Info,
        "sparse",
        format!(
            "Akilli seyrek edinim baslatiliyor. Kaynak: {}, Hedef: {}",
            task.source.display(),
            task.target.display()
        ),
    );

    let mut source = File::open(&task.source)
        .map_err(|err| AmeleError::io(HataKodu::DiskErisim, "Kaynak disk acilamadi", err))?;

    // 1. diskin dosya sistemi ve bölüm bitmap haritasını çıkar
    let sparse_map = SparseMap::from_file(&mut source, source_size);

    runtime_log(
        LogLevel::Info,
        "sparse",
        format!(
            "Seyrek harita cikarildi: Toplam {} bayt, Dolu: {} bayt, Bos: {} bayt (Tasarruf: {:.1}%)",
            sparse_map.total_size,
            sparse_map.allocated_bytes,
            sparse_map.unallocated_bytes,
            sparse_map.savings_percentage()
        ),
    );

    if let Some(parent) = task.target.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            AmeleError::io(HataKodu::DosyaYazma, "Hedef klasor olusturulamadi", err)
        })?;
    }

    let mut target = File::create(&task.target)
        .map_err(|err| AmeleError::io(HataKodu::DosyaYazma, "Hedef dosya olusturulamadi", err))?;

    // hedef dosya boyutunu baştan diskin tam boyutuna ayarla (sparse file boyutu)
    if let Err(err) = target.set_len(source_size) {
        runtime_log(
            LogLevel::Warn,
            "sparse",
            format!("set_len basarisiz oldu, seek ile genisletilecek: {:?}", err),
        );
    }

    let chunk_size = task.chunk_size.max(4096);
    let mut buffer = vec![0_u8; chunk_size];
    let zero_buffer = vec![0_u8; ZERO_BUFFER_SIZE];

    let mut processed_bytes = 0_u64;
    let mut actually_read_bytes = 0_u64;
    let mut sha256 = task.calculate_hash.then(Sha256::new);
    let mut blake3 = task.calculate_blake3.then(blake3::Hasher::new);
    let mut cancelled = false;

    // 2. haritadaki her aralığı sırayla işle
    for range in &sparse_map.ranges {
        match control() {
            DiskAcquisitionControl::Continue => {}
            DiskAcquisitionControl::Pause => {
                thread::sleep(Duration::from_millis(200));
            }
            DiskAcquisitionControl::Cancel => {
                runtime_log(
                    LogLevel::Warn,
                    "sparse",
                    "Seyrek edinim kullanici tarafindan iptal edildi.",
                );
                cancelled = true;
                break;
            }
        }

        if !range.allocated {
            // BOŞ ALAN: diskten fiziksel okuma yapma, doğrudan hedef dosyada atla!
            let next_offset = range.offset + range.length;
            if let Err(err) = target.seek(SeekFrom::Start(next_offset)) {
                let partial = mark_partial(&task.target)?;
                return Err(AmeleError::io(
                    HataKodu::DosyaYazma,
                    format!(
                        "Hedef dosyada seek yapilamadi, partial={}",
                        partial.display()
                    ),
                    err,
                ));
            }

            // hash bütünlüğü için ram'deki sıfır tamponunu sha256 / blake3'e besle (disk i/o yok)
            if sha256.is_some() || blake3.is_some() {
                let mut remaining_zeroes = range.length;
                while remaining_zeroes > 0 {
                    let to_hash = remaining_zeroes.min(zero_buffer.len() as u64) as usize;
                    if let Some(ctx) = &mut sha256 {
                        ctx.update(&zero_buffer[..to_hash]);
                    }
                    if let Some(ctx) = &mut blake3 {
                        ctx.update(&zero_buffer[..to_hash]);
                    }
                    remaining_zeroes -= to_hash as u64;
                }
            }

            processed_bytes += range.length;
            progress(processed_bytes, source_size);
            continue;
        }

        // DOLU ALAN: diskten oku ve hedefe yaz
        if let Err(err) = source.seek(SeekFrom::Start(range.offset)) {
            let partial = mark_partial(&task.target)?;
            return Err(AmeleError::io(
                HataKodu::DiskOkuma,
                format!(
                    "Kaynak diskte seek yapilamadi, partial={}",
                    partial.display()
                ),
                err,
            ));
        }
        if let Err(err) = target.seek(SeekFrom::Start(range.offset)) {
            let partial = mark_partial(&task.target)?;
            return Err(AmeleError::io(
                HataKodu::DosyaYazma,
                format!(
                    "Hedef dosyada seek yapilamadi, partial={}",
                    partial.display()
                ),
                err,
            ));
        }

        let mut range_read = 0_u64;
        while range_read < range.length {
            match control() {
                DiskAcquisitionControl::Continue => {}
                DiskAcquisitionControl::Pause => {
                    thread::sleep(Duration::from_millis(200));
                    continue;
                }
                DiskAcquisitionControl::Cancel => {
                    cancelled = true;
                    break;
                }
            }

            let to_read = (range.length - range_read).min(buffer.len() as u64) as usize;
            let read = match source.read(&mut buffer[..to_read]) {
                Ok(read) => read,
                Err(err) => {
                    let _ = target.flush();
                    drop(target);
                    let partial = mark_partial(&task.target)?;
                    return Err(AmeleError::io(
                        HataKodu::DiskOkuma,
                        format!("Seyrek edinim okuma hatasi, partial={}", partial.display()),
                        err,
                    ));
                }
            };
            if read == 0 {
                break;
            }

            if let Err(err) = target.write_all(&buffer[..read]) {
                let _ = target.flush();
                drop(target);
                let partial = mark_partial(&task.target)?;
                return Err(AmeleError::io(
                    HataKodu::DosyaYazma,
                    format!("Seyrek edinim yazma hatasi, partial={}", partial.display()),
                    err,
                ));
            }

            if let Some(ctx) = &mut sha256 {
                ctx.update(&buffer[..read]);
            }
            if let Some(ctx) = &mut blake3 {
                ctx.update(&buffer[..read]);
            }

            range_read += read as u64;
            actually_read_bytes += read as u64;
            processed_bytes += read as u64;
            progress(processed_bytes, source_size);
        }

        if cancelled {
            break;
        }
    }

    if let Err(err) = target.flush() {
        drop(target);
        let partial = mark_partial(&task.target)?;
        return Err(AmeleError::io(
            HataKodu::DosyaYazma,
            format!("Hedef dosya flush edilemedi, partial={}", partial.display()),
            err,
        ));
    }
    drop(target);

    if cancelled {
        let partial = mark_partial(&task.target)?;
        return Ok(DiskAcquisitionResult {
            target: task.target.clone(),
            bytes_copied: actually_read_bytes,
            total_bytes: source_size,
            sha256: None,
            blake3: None,
            partial_path: Some(partial),
        });
    }

    let mut hash_value = None;
    let mut blake3_value = None;
    if let Some(ctx) = sha256 {
        let hash = to_hex(&ctx.finalize());
        runtime_log(
            LogLevel::Info,
            "sparse",
            format!("Seyrek edinim SHA-256 tamamlandi: {}", hash),
        );
        let _ = write_sha256_sidecar(&task.target, &hash);
        hash_value = Some(hash);
    }
    if let Some(ctx) = blake3 {
        let hash = to_hex(ctx.finalize().as_bytes());
        runtime_log(
            LogLevel::Info,
            "sparse",
            format!("Seyrek edinim BLAKE3 tamamlandi: {}", hash),
        );
        let _ = write_blake3_sidecar(&task.target, &hash);
        blake3_value = Some(hash);
    }

    runtime_log(
        LogLevel::Info,
        "sparse",
        format!(
            "Akilli seyrek edinim basariyla bitti. Toplam: {} bayt, Fiziksel okunan: {} bayt",
            source_size, actually_read_bytes
        ),
    );

    Ok(DiskAcquisitionResult {
        target: task.target.clone(),
        bytes_copied: source_size,
        total_bytes: source_size,
        sha256: hash_value,
        blake3: blake3_value,
        partial_path: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sparse_acquisition_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let src_path = temp_dir.path().join("source.raw");
        let dst_path = temp_dir.path().join("target.raw");

        // 64 KB test diski oluştur: İlk 16 KB dolu veri, sonraki 32 KB boş (0), son 16 KB dolu veri
        let mut source_data = vec![0_u8; 64 * 1024];
        for b in &mut source_data[0..16 * 1024] {
            *b = 0xAA;
        }
        for b in &mut source_data[48 * 1024..64 * 1024] {
            *b = 0xBB;
        }
        fs::write(&src_path, &source_data).unwrap();

        let task = DiskAcquisitionTask::new(&src_path, &dst_path).with_sparse(true);

        let res = run_sparse_acquisition(
            &task,
            64 * 1024,
            |_, _| {},
            || DiskAcquisitionControl::Continue,
        )
        .unwrap();
        assert_eq!(res.total_bytes, 64 * 1024);
        assert!(res.sha256.is_some());

        // Hedef dosya boyutunu ve içeriğini doğrula
        let target_data = fs::read(&dst_path).unwrap();
        assert_eq!(target_data.len(), 64 * 1024);
        assert_eq!(
            target_data, source_data,
            "Seyrek olarak kopyalanan hedef veri kaynakla tam uyusmali"
        );
    }
}
