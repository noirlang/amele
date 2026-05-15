use crate::error::{HataKodu, WormError, WormResult};
use digest::Digest;
use md5::Md5;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub const HASH_BUFFER_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl HashAlgorithm {
    pub fn name(self) -> &'static str {
        match self {
            HashAlgorithm::Md5 => "MD5",
            HashAlgorithm::Sha1 => "SHA1",
            HashAlgorithm::Sha256 => "SHA256",
            HashAlgorithm::Sha512 => "SHA512",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "md5" => Some(Self::Md5),
            "sha1" => Some(Self::Sha1),
            "sha256" => Some(Self::Sha256),
            "sha512" => Some(Self::Sha512),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HashResult {
    pub algorithm: HashAlgorithm,
    pub value: String,
}

enum HashState {
    Md5(Md5),
    Sha1(Sha1),
    Sha256(Sha256),
    Sha512(Sha512),
}

impl HashState {
    fn new(algorithm: HashAlgorithm) -> Self {
        match algorithm {
            HashAlgorithm::Md5 => Self::Md5(Md5::new()),
            HashAlgorithm::Sha1 => Self::Sha1(Sha1::new()),
            HashAlgorithm::Sha256 => Self::Sha256(Sha256::new()),
            HashAlgorithm::Sha512 => Self::Sha512(Sha512::new()),
        }
    }

    fn update(&mut self, data: &[u8]) {
        match self {
            HashState::Md5(ctx) => ctx.update(data),
            HashState::Sha1(ctx) => ctx.update(data),
            HashState::Sha256(ctx) => ctx.update(data),
            HashState::Sha512(ctx) => ctx.update(data),
        }
    }

    fn finalize(self) -> String {
        match self {
            HashState::Md5(ctx) => to_hex(&ctx.finalize()),
            HashState::Sha1(ctx) => to_hex(&ctx.finalize()),
            HashState::Sha256(ctx) => to_hex(&ctx.finalize()),
            HashState::Sha512(ctx) => to_hex(&ctx.finalize()),
        }
    }
}

pub fn calculate_file_hash(path: impl AsRef<Path>, algorithm: HashAlgorithm) -> WormResult<String> {
    let results = calculate_multiple(path, &[algorithm])?;
    results
        .into_iter()
        .next()
        .map(|result| result.value)
        .ok_or_else(|| WormError::new(HataKodu::Genel, "Hash sonucu uretilemedi"))
}

pub fn calculate_multiple(
    path: impl AsRef<Path>,
    algorithms: &[HashAlgorithm],
) -> WormResult<Vec<HashResult>> {
    if algorithms.is_empty() {
        return Err(WormError::new(
            HataKodu::Genel,
            "En az bir hash algoritmasi gerekli",
        ));
    }

    let mut file = File::open(path.as_ref())
        .map_err(|err| WormError::io(HataKodu::DosyaAcilamadi, "Hash dosyasi acilamadi", err))?;
    let mut states: Vec<(HashAlgorithm, HashState)> = algorithms
        .iter()
        .copied()
        .map(|algorithm| (algorithm, HashState::new(algorithm)))
        .collect();
    let mut buffer = vec![0_u8; HASH_BUFFER_SIZE];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|err| WormError::io(HataKodu::DosyaOkuma, "Hash dosyasi okunamadi", err))?;
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

pub fn compare_hash(left: &str, right: &str) -> bool {
    left.eq_ignore_ascii_case(right)
}

pub(crate) fn write_sha256_sidecar(target: &Path, hash: &str) -> WormResult<()> {
    let sidecar = target.with_extension(format!(
        "{}sha256",
        target
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{ext}."))
            .unwrap_or_default()
    ));
    let mut file = File::create(&sidecar)
        .map_err(|err| WormError::io(HataKodu::DosyaYazma, "Hash dosyasi olusturulamadi", err))?;
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    writeln!(file, "{hash}  {name}")
        .map_err(|err| WormError::io(HataKodu::DosyaYazma, "Hash dosyasi yazilamadi", err))
}

pub(crate) fn to_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
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
    }
}
