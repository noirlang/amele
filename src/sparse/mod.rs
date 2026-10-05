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
pub use scanner::{build_disk_sparse_map, build_disk_sparse_map_ext, identify_and_scan_volume};

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
const ZERO_BUFFER_SIZE: usize = 4 * 1024 * 1024; // 4 mb

/// diskin seyrek haritası ve istatistik özeti
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseMap {
    pub ranges: Vec<SparseRange>,
    pub total_size: u64,
    pub allocated_bytes: u64,
    pub unallocated_bytes: u64,
    pub supported_volumes_found: usize,
    pub detected_filesystems: Vec<String>,
}

impl SparseMap {
    /// disk dosyasını inceleyerek seyrek harita oluşturur
    pub fn from_file(file: &mut File, total_size: u64) -> Self {
        let (ranges, supported_volumes_found, detected_filesystems) =
            scanner::build_disk_sparse_map_ext(file, total_size);
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
            supported_volumes_found,
            detected_filesystems,
        }
    }

    /// diskin veya bölümlerinin desteklenen bir dosya sistemine sahip olup olmadığını belirtir
    pub fn is_supported(&self) -> bool {
        self.supported_volumes_found > 0
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

    if !sparse_map.is_supported() {
        runtime_log(
            LogLevel::Error,
            "sparse",
            format!(
                "Desteklenen dosya sistemi bulunamadi: {}. Seyrek edinim durduruluyor.",
                task.source.display()
            ),
        );
        return Err(AmeleError::new(
            HataKodu::Disk,
            "Diskiniz desteklenmiyor: Akıllı seyrek edinim (dolu olan kadar al) için bu disk veya bölümde desteklenen bir dosya sistemi (NTFS, ext4, XFS, exFAT, FAT) bulunamadı. Lütfen 'Tamamını al (Fiziksel DD)' modunu seçin.",
        ));
    }

    runtime_log(
        LogLevel::Info,
        "sparse",
        format!(
            "Seyrek harita cikarildi: Toplam {} bayt, Dolu: {} bayt, Bos: {} bayt (Tasarruf: {:.1}%), Sistemler: {:?}",
            sparse_map.total_size,
            sparse_map.allocated_bytes,
            sparse_map.unallocated_bytes,
            sparse_map.savings_percentage(),
            sparse_map.detected_filesystems
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
                let mut zero_reported = 0_u64;
                while remaining_zeroes > 0 {
                    match control() {
                        DiskAcquisitionControl::Continue => {}
                        DiskAcquisitionControl::Pause => {
                            thread::sleep(Duration::from_millis(200));
                            continue;
                        }
                        DiskAcquisitionControl::Cancel => {
                            runtime_log(
                                LogLevel::Warn,
                                "sparse",
                                "Seyrek edinim sifir hashleme sirasinda kullanici tarafindan iptal edildi.",
                            );
                            cancelled = true;
                            break;
                        }
                    }

                    let to_hash = remaining_zeroes.min(zero_buffer.len() as u64) as usize;
                    if let Some(ctx) = &mut sha256 {
                        ctx.update(&zero_buffer[..to_hash]);
                    }
                    if let Some(ctx) = &mut blake3 {
                        ctx.update(&zero_buffer[..to_hash]);
                    }
                    remaining_zeroes -= to_hash as u64;
                    processed_bytes += to_hash as u64;
                    zero_reported += to_hash as u64;

                    // her 32 MB'da bir veya aralık tamamlandığında ilerleme bildir ki donma hissi olmasın
                    if zero_reported >= 32 * 1024 * 1024 || remaining_zeroes == 0 {
                        progress(processed_bytes, source_size);
                        zero_reported = 0;
                    }
                }
                if cancelled {
                    break;
                }
            } else {
                processed_bytes += range.length;
                progress(processed_bytes, source_size);
            }

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

    fn create_synthetic_exfat_disk() -> Vec<u8> {
        let mut data = Vec::new();

        // 512 bayt VBR
        let mut vbr = [0_u8; 512];
        vbr[0..3].copy_from_slice(&[0xEB, 0x76, 0x90]);
        vbr[3..11].copy_from_slice(b"EXFAT   ");
        vbr[108] = 9; // 512 bayt sektor
        vbr[109] = 1; // 2 sektor = 1024 bayt cluster
        vbr[88..92].copy_from_slice(&10_u32.to_le_bytes()); // heap offset: 10 sektor = 5120 bayt
        vbr[92..96].copy_from_slice(&16_u32.to_le_bytes()); // 16 cluster
        vbr[96..100].copy_from_slice(&2_u32.to_le_bytes()); // root dir cluster: 2
        vbr[510] = 0x55;
        vbr[511] = 0xAA;
        data.extend_from_slice(&vbr);

        // Padding (5120 - 512 = 4608 bayt)
        data.resize(5120, 0);

        // Cluster 2: Root Directory (1024 bayt)
        let mut root_dir = vec![0_u8; 1024];
        root_dir[0] = 0x81; // Allocation bitmap entry
        root_dir[1] = 0x00;
        root_dir[20..24].copy_from_slice(&3_u32.to_le_bytes()); // FirstCluster: 3
        root_dir[24..32].copy_from_slice(&2_u64.to_le_bytes()); // 2 bytes bitmap
        data.extend_from_slice(&root_dir);

        // Cluster 3: Allocation Bitmap (1024 bayt)
        // Byte 0: 0b0000_0011 (cluster 2 ve 3 dolu, cluster 4..9 bos)
        // Byte 1: 0b1111_0000 (cluster 10..13 bos, cluster 14..17 dolu)
        let mut bitmap = vec![0_u8; 1024];
        bitmap[0] = 0b0000_0011;
        bitmap[1] = 0b1111_0000;
        data.extend_from_slice(&bitmap);

        // Kalan 14 kume (14 * 1024 = 14336 bayt)
        let mut rem_data = vec![0_u8; 14 * 1024];
        // Son 4 kume (14..17) dolu olduğu için onlara test verisi yazalım
        for b in &mut rem_data[10 * 1024..14 * 1024] {
            *b = 0xEE;
        }
        data.extend_from_slice(&rem_data);

        data
    }

    #[test]
    fn test_sparse_acquisition_roundtrip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let src_path = temp_dir.path().join("source.raw");
        let dst_path = temp_dir.path().join("target.raw");

        let source_data = create_synthetic_exfat_disk();
        let total_size = source_data.len() as u64;
        fs::write(&src_path, &source_data).unwrap();

        let task = DiskAcquisitionTask::new(&src_path, &dst_path).with_sparse(true);

        let res = run_sparse_acquisition(
            &task,
            total_size,
            |_, _| {},
            || DiskAcquisitionControl::Continue,
        )
        .unwrap();
        assert_eq!(res.total_bytes, total_size);
        assert!(res.sha256.is_some());

        // Hedef dosya boyutunu ve içeriğini doğrula
        let target_data = fs::read(&dst_path).unwrap();
        assert_eq!(target_data.len(), total_size as usize);
        assert_eq!(
            target_data, source_data,
            "Seyrek olarak kopyalanan hedef veri kaynakla tam uyusmali"
        );
    }

    #[test]
    fn test_sparse_acquisition_unsupported_disk_fails() {
        let temp_dir = tempfile::tempdir().unwrap();
        let src_path = temp_dir.path().join("unsupported.raw");
        let dst_path = temp_dir.path().join("target.raw");

        // Rastgele / tanimsiz dosya sistemi verisi
        let source_data = vec![0x33_u8; 64 * 1024];
        fs::write(&src_path, &source_data).unwrap();

        let task = DiskAcquisitionTask::new(&src_path, &dst_path).with_sparse(true);

        let res = run_sparse_acquisition(
            &task,
            64 * 1024,
            |_, _| {},
            || DiskAcquisitionControl::Continue,
        );

        assert!(
            res.is_err(),
            "Desteklenmeyen disk seyrek modda hata vermelidir"
        );
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains("Diskiniz desteklenmiyor"),
            "Hata mesaji 'Diskiniz desteklenmiyor' icermelidir: {}",
            err_msg
        );
    }
}
