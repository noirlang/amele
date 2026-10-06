//! dosya hash hesaplama api rotası.

use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::hash::{self, HashAlgorithm};
use crate::server::{Response, json_error, json_ok};

/// Dosya hash hesaplama API isteğini işler.
pub fn hash_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct HashRequest {
        path: String,
        algorithms: Option<Vec<String>>,
    }

    if body.is_empty() {
        return json_error(
            400,
            "istek govdesi bos olamaz (path parametresi gereklidir)",
        );
    }

    let request: HashRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, format!("Gecersiz istek JSON: {}", err)),
    };

    let target_path_str = request.path.trim();
    if target_path_str.is_empty() {
        return json_error(400, "path parametresi bos olamaz");
    }

    let target_path = Path::new(target_path_str);
    if !target_path.exists() {
        return json_error(404, format!("Hedef dosya bulunamadi: {}", target_path_str));
    }
    if target_path.is_dir() {
        return json_error(
            400,
            format!("Hedef yol bir klasor, dosya degil: {}", target_path_str),
        );
    }

    let algorithms = match parse_algorithms(request.algorithms) {
        Ok(algorithms) => algorithms,
        Err(message) => return json_error(400, message),
    };

    match hash::calculate_multiple(target_path, &algorithms) {
        Ok(results) => {
            let _ = crate::profile::record_active_profile_activity(
                "hash",
                "calculate",
                None,
                Some(&format!("Dosya: {}", target_path_str)),
            );

            let mut value = serde_json::Map::new();
            for result in results {
                value.insert(
                    result.algorithm.name().to_ascii_lowercase(),
                    Value::String(result.value),
                );
            }
            json_ok(Value::Object(value))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

/// SHA-256 özeti için RFC 3161 zaman damgası alır ve DER yanıtını sidecar olarak saklar.
pub fn hash_timestamp_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct TimestampRequest {
        path: String,
        tsa_url: Option<String>,
    }

    let request: TimestampRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, format!("Gecersiz istek JSON: {}", err)),
    };
    let path = request.path.trim();
    if path.is_empty() {
        return json_error(400, "path parametresi bos olamaz");
    }
    let target = Path::new(path);
    if !target.is_file() {
        return json_error(404, format!("Hedef dosya bulunamadi: {}", path));
    }

    let tsa_url = request
        .tsa_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("https://freetsa.org/tsr");
    let sha256 = match hash::calculate_file_hash(target, HashAlgorithm::Sha256) {
        Ok(value) => value,
        Err(err) => return json_error(500, err.to_string()),
    };
    let timestamp = match tsp_http_client::request_timestamp_for_digest(tsa_url, &sha256) {
        Ok(timestamp) => timestamp,
        Err(err) => return json_error(502, format!("TSA zaman damgasi alinamadi: {err}")),
    };
    let timestamped_at = match timestamp.datetime() {
        Ok(value) => value.to_rfc3339(),
        Err(err) => return json_error(502, format!("TSA yaniti gecersiz: {err}")),
    };

    let sidecar = timestamp_sidecar_path(target);
    if let Err(err) = fs::write(&sidecar, timestamp.as_der_encoded()) {
        return json_error(500, format!("TSA yaniti kaydedilemedi: {err}"));
    }

    let _ = crate::profile::record_active_profile_activity(
        "hash",
        "timestamp",
        None,
        Some(&format!("Dosya: {}, TSA: {}", path, tsa_url)),
    );
    json_ok(serde_json::json!({
        "path": path,
        "sha256": sha256,
        "tsa_url": tsa_url,
        "timestamped_at": timestamped_at,
        "timestamp_response_path": sidecar,
        "protocol": "RFC 3161",
    }))
}

fn timestamp_sidecar_path(target: &Path) -> PathBuf {
    target.with_extension(format!(
        "{}tsr",
        target
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{ext}."))
            .unwrap_or_default()
    ))
}

/// API'den gelen hash algoritması stringlerini tekilleştirilmiş enum listesine çevirir.
fn parse_algorithms(values: Option<Vec<String>>) -> Result<Vec<HashAlgorithm>, String> {
    let list = match values {
        Some(list) if !list.is_empty() => list,
        _ => {
            return Ok(vec![
                HashAlgorithm::Md5,
                HashAlgorithm::Sha1,
                HashAlgorithm::Sha256,
                HashAlgorithm::Sha512,
                HashAlgorithm::Blake3,
            ]);
        }
    };

    let mut result = Vec::new();
    for v in list {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            continue;
        }
        let alg = HashAlgorithm::parse(trimmed)
            .ok_or_else(|| format!("desteklenmeyen hash algoritmasi: {trimmed}"))?;
        if !result.contains(&alg) {
            result.push(alg);
        }
    }

    if result.is_empty() {
        return Ok(vec![
            HashAlgorithm::Md5,
            HashAlgorithm::Sha1,
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha512,
            HashAlgorithm::Blake3,
        ]);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_parse_algorithms_default() {
        let algs = parse_algorithms(None).unwrap();
        assert_eq!(algs.len(), 5);
        assert!(algs.contains(&HashAlgorithm::Md5));
        assert!(algs.contains(&HashAlgorithm::Sha1));
        assert!(algs.contains(&HashAlgorithm::Sha256));
        assert!(algs.contains(&HashAlgorithm::Sha512));
        assert!(algs.contains(&HashAlgorithm::Blake3));

        let algs_empty = parse_algorithms(Some(vec![])).unwrap();
        assert_eq!(algs_empty.len(), 5);
    }

    #[test]
    fn test_parse_algorithms_custom_and_deduplicated() {
        let input = vec![
            "sha256".to_string(),
            "MD5".to_string(),
            "sha256".to_string(),
            "  Sha1  ".to_string(),
            "blake3".to_string(),
        ];
        let algs = parse_algorithms(Some(input)).unwrap();
        assert_eq!(
            algs,
            vec![
                HashAlgorithm::Sha256,
                HashAlgorithm::Md5,
                HashAlgorithm::Sha1,
                HashAlgorithm::Blake3,
            ]
        );
    }

    #[test]
    fn test_parse_algorithms_unsupported() {
        let input = vec!["unsupported_algo_xyz".to_string()];
        let err = parse_algorithms(Some(input)).unwrap_err();
        assert!(err.contains("desteklenmeyen"));
    }

    #[test]
    fn test_hash_endpoint_validation() {
        let resp = hash_endpoint(b"");
        assert_eq!(resp.status, 400);

        let resp = hash_endpoint(b"invalid");
        assert_eq!(resp.status, 400);

        let resp = hash_endpoint(br#"{"path": ""}"#);
        assert_eq!(resp.status, 400);

        let resp = hash_endpoint(br#"{"path": "   "}"#);
        assert_eq!(resp.status, 400);

        let resp = hash_endpoint(br#"{"path": "/tmp/nonexistent_file_amele_test_12345"}"#);
        assert_eq!(resp.status, 404);

        let dir = tempfile::tempdir().unwrap();
        let body = serde_json::json!({
            "path": dir.path().to_str().unwrap()
        });
        let resp = hash_endpoint(&serde_json::to_vec(&body).unwrap());
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_hash_endpoint_success() {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(b"amele forensic test content").unwrap();
        file.flush().unwrap();

        let path = file.path().to_str().unwrap();
        let body = serde_json::json!({
            "path": path,
            "algorithms": ["md5", "sha256"]
        });

        let resp = hash_endpoint(&serde_json::to_vec(&body).unwrap());
        assert_eq!(resp.status, 200);

        let val: Value = serde_json::from_slice(&resp.body).unwrap();
        assert!(val.get("md5").is_some());
        assert!(val.get("sha256").is_some());
        assert!(val.get("sha1").is_none());
    }

    #[test]
    fn test_timestamp_sidecar_path() {
        assert_eq!(
            timestamp_sidecar_path(Path::new("/tmp/evidence.raw")),
            PathBuf::from("/tmp/evidence.raw.tsr")
        );
        assert_eq!(
            timestamp_sidecar_path(Path::new("/tmp/evidence")),
            PathBuf::from("/tmp/evidence.tsr")
        );
    }

    #[test]
    fn test_hash_timestamp_endpoint_validation() {
        assert_eq!(hash_timestamp_endpoint(b"invalid").status, 400);
        assert_eq!(hash_timestamp_endpoint(br#"{"path":""}"#).status, 400);
        assert_eq!(
            hash_timestamp_endpoint(br#"{"path":"/tmp/nonexistent_file_amele_tsa_test_12345"}"#)
                .status,
            404
        );
    }
}
