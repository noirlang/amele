//! dosya hash hesaplama api rotası.

use serde::Deserialize;
use serde_json::Value;
use std::path::Path;

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
        return json_error(400, "istek govdesi bos olamaz (path parametresi gereklidir)");
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
        return json_error(400, format!("Hedef yol bir klasor, dosya degil: {}", target_path_str));
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
        assert_eq!(algs.len(), 4);
        assert!(algs.contains(&HashAlgorithm::Md5));
        assert!(algs.contains(&HashAlgorithm::Sha1));
        assert!(algs.contains(&HashAlgorithm::Sha256));
        assert!(algs.contains(&HashAlgorithm::Sha512));

        let algs_empty = parse_algorithms(Some(vec![])).unwrap();
        assert_eq!(algs_empty.len(), 4);
    }

    #[test]
    fn test_parse_algorithms_custom_and_deduplicated() {
        let input = vec![
            "sha256".to_string(),
            "MD5".to_string(),
            "sha256".to_string(),
            "  Sha1  ".to_string(),
        ];
        let algs = parse_algorithms(Some(input)).unwrap();
        assert_eq!(algs, vec![
            HashAlgorithm::Sha256,
            HashAlgorithm::Md5,
            HashAlgorithm::Sha1,
        ]);
    }

    #[test]
    fn test_parse_algorithms_unsupported() {
        let input = vec!["blake3".to_string()];
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
}
