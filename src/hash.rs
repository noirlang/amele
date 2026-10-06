//! md5, sha1, sha256 hesaplayan yer. kopyalama yaparken eşzamanlı hesaplıyo.

use crate::error::{AmeleError, AmeleResult, HataKodu};
use digest::Digest;
use md5::Md5;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

pub const HASH_BUFFER_SIZE: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
/// Desteklenen dosya bütünlük algoritmalarını temsil eder.
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
    Blake3,
}

impl HashAlgorithm {
    /// Hash algoritmasının raporda gösterilecek adını döndürür.
    pub fn name(self) -> &'static str {
        match self {
            HashAlgorithm::Md5 => "MD5",
            HashAlgorithm::Sha1 => "SHA1",
            HashAlgorithm::Sha256 => "SHA256",
            HashAlgorithm::Sha512 => "SHA512",
            HashAlgorithm::Blake3 => "BLAKE3",
        }
    }

    /// Kullanıcı/API metnini hash algoritmasına çevirir.
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "md5" => Some(Self::Md5),
            "sha1" => Some(Self::Sha1),
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            "blake3" | "b3" => Some(Self::Blake3),
            _ => None,
        }
    }
}

/// Aktif sistemde BLAKE3 için çalışan SIMD komut setini raporlar (AVX-512, AVX2, NEON vb.).
pub fn blake3_simd_instruction_set() -> &'static str {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if std::is_x86_feature_detected!("avx512f") {
            "AVX-512"
        } else if std::is_x86_feature_detected!("avx2") {
            "AVX2"
        } else if std::is_x86_feature_detected!("sse4.1") {
            "SSE4.1"
        } else if std::is_x86_feature_detected!("sse2") {
            "SSE2"
        } else {
            "Scalar"
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        "NEON"
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
    {
        "Portable"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// Hesaplanan hash algoritması ve değerini birlikte taşır.
pub struct HashResult {
    pub algorithm: HashAlgorithm,
    pub value: String,
}

/// Çalışan hash context türünü tek enum altında saklar.
enum HashState {
    Md5(Md5),
    Sha1(Sha1),
    Sha256(Sha256),
    Sha512(Sha512),
    Blake3(blake3::Hasher),
}

impl HashState {
    /// Seçilen algoritmaya uygun hash context oluşturur.
    fn new(algorithm: HashAlgorithm) -> Self {
        match algorithm {
            HashAlgorithm::Md5 => Self::Md5(Md5::new()),
            HashAlgorithm::Sha1 => Self::Sha1(Sha1::new()),
            HashAlgorithm::Sha256 => Self::Sha256(Sha256::new()),
            HashAlgorithm::Sha512 => Self::Sha512(Sha512::new()),
            HashAlgorithm::Blake3 => Self::Blake3(blake3::Hasher::new()),
        }
    }

    /// Okunan dosya parçasını ilgili hash contextine ekler.
    fn update(&mut self, data: &[u8]) {
        match self {
            HashState::Md5(ctx) => ctx.update(data),
            HashState::Sha1(ctx) => ctx.update(data),
            HashState::Sha256(ctx) => ctx.update(data),
            HashState::Sha512(ctx) => ctx.update(data),
            HashState::Blake3(ctx) => {
                // Büyük tamponlarda Rayon ile çok çekirdekli SIMD hesaplama
                if data.len() >= 64 * 1024 {
                    ctx.update_rayon(data);
                } else {
                    ctx.update(data);
                }
            }
        }
    }

    /// Hash contextini tamamlayıp hex string üretir.
    fn finalize(self) -> String {
        match self {
            HashState::Md5(ctx) => to_hex(&ctx.finalize()),
            HashState::Sha1(ctx) => to_hex(&ctx.finalize()),
            HashState::Sha256(ctx) => to_hex(&ctx.finalize()),
            HashState::Sha512(ctx) => to_hex(&ctx.finalize()),
            HashState::Blake3(ctx) => to_hex(ctx.finalize().as_bytes()),
        }
    }
}

/// Tek algoritma için dosya hashini hesaplar.
pub fn calculate_file_hash(
    path: impl AsRef<Path>,
    algorithm: HashAlgorithm,
) -> AmeleResult<String> {
    if algorithm == HashAlgorithm::Blake3 {
        // Hızlı disklerde (NVMe/SSD) CPU darboğazını önlemek için mmap + Rayon çok çekirdekli SIMD
        if let Ok(hasher) = blake3::Hasher::new().update_mmap_rayon(path.as_ref()) {
            return Ok(to_hex(hasher.finalize().as_bytes()));
        }
    }
    let results = calculate_multiple(path, &[algorithm])?;
    results
        .into_iter()
        .next()
        .map(|result| result.value)
        .ok_or_else(|| AmeleError::new(HataKodu::Genel, "Hash sonucu uretilemedi"))
}

/// Tek algoritma için dosya hashini okunan bayt ilerlemesiyle hesaplar.
/// GB'lik imajlarda arayuzun kilitlenmis gibi durmamasi icin kullanilir.
pub fn calculate_file_hash_with_progress(
    path: impl AsRef<Path>,
    algorithm: HashAlgorithm,
    on_progress: &mut dyn FnMut(u64, u64),
) -> AmeleResult<String> {
    let file = File::open(path.as_ref())
        .map_err(|err| AmeleError::io(HataKodu::DosyaAcilamadi, "Hash dosyasi acilamadi", err))?;
    let total = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    let mut state = HashState::new(algorithm);
    let mut buffer = vec![0_u8; HASH_BUFFER_SIZE];
    let mut done = 0_u64;
    let mut last_report = Instant::now();
    // file degiskeni asagida okuma icin tekrar kullaniliyor.
    let mut file = file;
    on_progress(0, total);
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| AmeleError::io(HataKodu::DosyaOkuma, "Hash dosyasi okunamadi", err))?;
        if read == 0 {
            break;
        }
        state.update(&buffer[..read]);
        done += read as u64;
        if last_report.elapsed() >= Duration::from_millis(250) || done >= total {
            last_report = Instant::now();
            on_progress(done, total);
        }
    }
    on_progress(total, total);
    Ok(state.finalize())
}

/// Dosyayı tek geçişte okuyup birden fazla algoritmayı ilerleme bildirerek hesaplar.
/// RAM imajı gibi dış araçların yazdığı dosyalarda (kopyalama sırasında hash alamıyoruz)
/// dosyayı iki kere okumamak için kullanılır.
pub fn calculate_multiple_with_progress(
    path: impl AsRef<Path>,
    algorithms: &[HashAlgorithm],
    on_progress: &mut dyn FnMut(u64, u64),
) -> AmeleResult<Vec<HashResult>> {
    if algorithms.is_empty() {
        return Err(AmeleError::new(
            HataKodu::Genel,
            "En az bir hash algoritmasi gerekli",
        ));
    }

    let mut file = File::open(path.as_ref())
        .map_err(|err| AmeleError::io(HataKodu::DosyaAcilamadi, "Hash dosyasi acilamadi", err))?;
    let total = file.metadata().map(|meta| meta.len()).unwrap_or(0);
    let mut states: Vec<(HashAlgorithm, HashState)> = algorithms
        .iter()
        .copied()
        .map(|algorithm| (algorithm, HashState::new(algorithm)))
        .collect();
    let mut buffer = vec![0_u8; HASH_BUFFER_SIZE];
    let mut done = 0_u64;
    let mut last_report = Instant::now();
    on_progress(0, total);

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| AmeleError::io(HataKodu::DosyaOkuma, "Hash dosyasi okunamadi", err))?;
        if read == 0 {
            break;
        }
        for (_, state) in &mut states {
            state.update(&buffer[..read]);
        }
        done += read as u64;
        if last_report.elapsed() >= Duration::from_millis(250) || done >= total {
            last_report = Instant::now();
            on_progress(done, total);
        }
    }
    on_progress(total, total);

    Ok(states
        .into_iter()
        .map(|(algorithm, state)| HashResult {
            algorithm,
            value: state.finalize(),
        })
        .collect())
}

/// Dosyayı bir kez okuyarak birden fazla hash algoritmasını aynı anda hesaplar.
pub fn calculate_multiple(
    path: impl AsRef<Path>,
    algorithms: &[HashAlgorithm],
) -> AmeleResult<Vec<HashResult>> {
    if algorithms.is_empty() {
        return Err(AmeleError::new(
            HataKodu::Genel,
            "En az bir hash algoritmasi gerekli",
        ));
    }

    let mut file = File::open(path.as_ref())
        .map_err(|err| AmeleError::io(HataKodu::DosyaAcilamadi, "Hash dosyasi acilamadi", err))?;
    let mut states: Vec<(HashAlgorithm, HashState)> = algorithms
        .iter()
        .copied()
        .map(|algorithm| (algorithm, HashState::new(algorithm)))
        .collect();
    let mut buffer = vec![0_u8; HASH_BUFFER_SIZE];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| AmeleError::io(HataKodu::DosyaOkuma, "Hash dosyasi okunamadi", err))?;
        if read == 0 {
            break;
        }
        for (_, state) in &mut states {
            state.update(&buffer[..read]);
        }
    }

    Ok(states
        .into_iter()
        .map(|(algorithm, state)| HashResult {
            algorithm,
            value: state.finalize(),
        })
        .collect())
}

/// İki hash değerini büyük/küçük harf duyarsız karşılaştırır.
pub fn compare_hash(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

/// Hedef dosyanın yanına SHA-256 sidecar dosyası yazar.
pub fn write_sha256_sidecar(target: &Path, hash: &str) -> AmeleResult<()> {
    write_hash_sidecar(target, HashAlgorithm::Sha256, hash)
}

/// Hedef dosyanın yanına BLAKE3 sidecar dosyası yazar (.b3sum).
pub fn write_blake3_sidecar(target: &Path, hash: &str) -> AmeleResult<()> {
    write_hash_sidecar(target, HashAlgorithm::Blake3, hash)
}

/// Algoritmaya uygun uzantıyla sidecar hash dosyası yazar.
pub fn write_hash_sidecar(target: &Path, algorithm: HashAlgorithm, hash: &str) -> AmeleResult<()> {
    let ext_str = match algorithm {
        HashAlgorithm::Sha256 => "sha256",
        HashAlgorithm::Blake3 => "b3sum",
        HashAlgorithm::Sha512 => "sha512",
        HashAlgorithm::Sha1 => "sha1",
        HashAlgorithm::Md5 => "md5",
    };
    let sidecar = target.with_extension(format!(
        "{}{ext_str}",
        target
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{ext}."))
            .unwrap_or_default()
    ));
    if let Some(parent) = sidecar.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut file = File::create(&sidecar)
        .map_err(|err| AmeleError::io(HataKodu::DosyaYazma, "Hash dosyasi olusturulamadi", err))?;
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    writeln!(file, "{hash}  {name}")
        .map_err(|err| AmeleError::io(HataKodu::DosyaYazma, "Hash dosyasi yazilamadi", err))
}

/// İmaj dosyasının yanındaki sidecar hash dosyasını otomatik olarak arayıp beklenen hash ve algoritmayı bulur.
pub fn detect_sidecar_hash(target: &Path) -> AmeleResult<Option<(String, HashAlgorithm)>> {
    let candidates = [
        ("b3sum", HashAlgorithm::Blake3),
        ("blake3", HashAlgorithm::Blake3),
        ("sha256", HashAlgorithm::Sha256),
        ("sha512", HashAlgorithm::Sha512),
        ("sha1", HashAlgorithm::Sha1),
        ("md5", HashAlgorithm::Md5),
    ];

    for (ext, alg) in candidates {
        let sidecar = target.with_extension(format!(
            "{}{ext}",
            target
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!("{e}."))
                .unwrap_or_default()
        ));
        if sidecar.exists() {
            if let Ok(content) = std::fs::read_to_string(&sidecar) {
                if let Some(first_word) = content.split_whitespace().next() {
                    let cleaned = first_word.trim().to_string();
                    if !cleaned.is_empty() {
                        return Ok(Some((cleaned, alg)));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Byte dizisini küçük harf hex stringe çevirir.
pub(crate) fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

/// delil dosyasi icin rfc 3161 tsr (time stamp response) yan dosya yolunu belirler
pub fn timestamp_sidecar_path(target: &Path) -> std::path::PathBuf {
    target.with_extension(format!(
        "{}tsr",
        target
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{ext}."))
            .unwrap_or_default()
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampSealResult {
    pub path: std::path::PathBuf,
    pub sha256: String,
    pub tsa_url: String,
    pub timestamped_at: String,
    pub timestamp_response_path: std::path::PathBuf,
    pub protocol: &'static str,
}

/// delilin sha256 ozetini cikarip rfc 3161 tsa sunucusundan guvenilir zaman damgasi alir ve tsr olarak kaydeder
pub fn seal_file_timestamp(target: &Path, tsa_url: &str) -> AmeleResult<TimestampSealResult> {
    if !target.is_file() {
        return Err(AmeleError::new(
            HataKodu::Dosya,
            format!("hedef dosya bulunamadi veya gecersiz: {}", target.display()),
        ));
    }
    let sha256 = calculate_file_hash(target, HashAlgorithm::Sha256)?;
    let timestamp =
        tsp_http_client::request_timestamp_for_digest(tsa_url, &sha256).map_err(|err| {
            AmeleError::new(HataKodu::Ag, format!("tsa zaman damgasi alinamadi: {err}"))
        })?;
    let timestamped_at = timestamp
        .datetime()
        .map_err(|err| {
            AmeleError::new(
                HataKodu::IcerikGecersiz,
                format!("tsa yaniti gecersiz: {err}"),
            )
        })?
        .to_rfc3339();
    let sidecar = timestamp_sidecar_path(target);
    std::fs::write(&sidecar, timestamp.as_der_encoded()).map_err(|err| {
        AmeleError::io(
            HataKodu::DosyaYazma,
            format!("tsa yaniti kaydedilemedi: {err}"),
            err,
        )
    })?;

    let _ = crate::profile::record_active_profile_activity(
        "hash",
        "timestamp",
        None,
        Some(&format!("dosya: {}, tsa: {}", target.display(), tsa_url)),
    );

    Ok(TimestampSealResult {
        path: target.to_path_buf(),
        sha256,
        tsa_url: tsa_url.to_string(),
        timestamped_at,
        timestamp_response_path: sidecar,
        protocol: "RFC 3161",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn calculates_known_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.bin");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"abc").unwrap();

        let results = calculate_multiple(
            &path,
            &[
                HashAlgorithm::Md5,
                HashAlgorithm::Sha1,
                HashAlgorithm::Sha256,
                HashAlgorithm::Sha512,
                HashAlgorithm::Blake3,
            ],
        )
        .unwrap();

        assert_eq!(results[0].value, "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(results[1].value, "a9993e364706816aba3e25717850c26c9cd0d89d");
        assert_eq!(
            results[2].value,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(results[3].value.len(), 128);
        assert_eq!(
            results[4].value,
            "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
        );
    }

    #[test]
    fn test_hash_algorithm_parse_and_name() {
        assert_eq!(HashAlgorithm::parse("md5"), Some(HashAlgorithm::Md5));
        assert_eq!(HashAlgorithm::parse("SHA256"), Some(HashAlgorithm::Sha256));
        assert_eq!(HashAlgorithm::parse("blake3"), Some(HashAlgorithm::Blake3));
        assert_eq!(HashAlgorithm::parse("B3"), Some(HashAlgorithm::Blake3));
        assert_eq!(HashAlgorithm::parse("unknown"), None);

        assert_eq!(HashAlgorithm::Sha256.name(), "SHA256");
        assert_eq!(HashAlgorithm::Blake3.name(), "BLAKE3");
    }

    #[test]
    fn test_calculate_file_hash_and_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sample.txt");
        std::fs::write(&path, b"test").unwrap();

        let hash = calculate_file_hash(&path, HashAlgorithm::Sha256).unwrap();
        assert_eq!(
            hash,
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );

        write_sha256_sidecar(&path, &hash).unwrap();
        let sidecar = dir.path().join("sample.txt.sha256");
        assert!(sidecar.exists());
        let content = std::fs::read_to_string(&sidecar).unwrap();
        assert!(content.starts_with(&hash));
    }

    #[test]
    fn test_blake3_simd_and_sidecar_detection() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("image.dd");
        std::fs::write(&path, b"abc").unwrap();

        let simd = blake3_simd_instruction_set();
        assert!(!simd.is_empty());

        let b3 = calculate_file_hash(&path, HashAlgorithm::Blake3).unwrap();
        assert_eq!(
            b3,
            "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
        );

        let mut progress_done = 0_u64;
        let b3_prog =
            calculate_file_hash_with_progress(&path, HashAlgorithm::Blake3, &mut |done, _| {
                progress_done = done;
            })
            .unwrap();
        assert_eq!(b3, b3_prog);
        assert_eq!(progress_done, 3);

        write_blake3_sidecar(&path, &b3).unwrap();
        let (detected_hash, detected_alg) = detect_sidecar_hash(&path).unwrap().unwrap();
        assert_eq!(detected_hash, b3);
        assert_eq!(detected_alg, HashAlgorithm::Blake3);
    }

    #[test]
    fn test_to_hex() {
        assert_eq!(to_hex(&[0x00, 0xab, 0xff]), "00abff");
    }
}
