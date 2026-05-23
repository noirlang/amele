use crate::api;
use crate::server::{self, Response};
use std::fs;

pub fn route_request(method: &str, raw_path: &str, body: &[u8]) -> Response {
    if method == "OPTIONS" {
        return Response::empty(204);
    }

    let path = raw_path.split('?').next().unwrap_or("/");
    if path.starts_with("/api/") {
        return api::route_api(method, path, body);
    }

    if method != "GET" && method != "HEAD" {
        return server::json_error(405, "method not allowed");
    }

    serve_static(path, method == "HEAD")
}

fn serve_static(path: &str, head_only: bool) -> Response {
    let path = if path == "/" { "/index.html" } else { path };
    let Ok(decoded) = server::percent_decode(path) else {
        return server::json_error(400, "invalid path encoding");
    };
    let relative = decoded.trim_start_matches('/');
    if relative.split('/').any(|part| part == "..") {
        return server::json_error(403, "path traversal rejected");
    }

    let mut file_path = server::ui_root();
    file_path.push(relative);
    if file_path.is_dir() {
        file_path.push("index.html");
    }

    match fs::read(&file_path) {
        Ok(body) => Response {
            status: 200,
            content_type: server::mime_for(&file_path).to_string(),
            body: if head_only { Vec::new() } else { body },
        },
        Err(_) => Response {
            status: 404,
            content_type: "text/html; charset=utf-8".to_string(),
            body: b"Not found".to_vec(),
        },
    }
}
