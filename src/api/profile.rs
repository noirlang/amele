//! profil kaydetme ve online senkron api rotaları.

use serde::Deserialize;
use serde_json::json;

use crate::server::{Response, json_error, json_ok};

#[derive(Deserialize)]
/// Profil oluşturma isteğinin gövdesidir.
struct CreateProfileRequest {
    full_name: String,
    username: String,
    language: Option<String>,
    theme: Option<String>,
    open_directly: Option<bool>,
}

#[derive(Deserialize)]
/// Profil seçme isteğinin gövdesidir.
struct SelectProfileRequest {
    username: String,
    open_directly: Option<bool>,
}

#[derive(Deserialize)]
/// Online profil bağlama isteğinin gövdesidir.
struct OnlineProfileLoginRequest {
    identifier: String,
    password: String,
    language: Option<String>,
    theme: Option<String>,
    open_directly: Option<bool>,
}

/// Profil listesini ve açılış kararını döndürür.
pub fn profiles_get_endpoint() -> Response {
    match crate::profile::bootstrap_profiles() {
        Ok(state) => json_ok(json!({
            "profiles": state.profiles,
            "active_profile": state.active_profile,
            "should_prompt": state.should_prompt,
            "base_dir": state.base_dir,
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Yeni profil oluşturur, aktif eder ve profil klasörlerini hazırlar.
pub fn profile_create_endpoint(body: &[u8]) -> Response {
    let request: CreateProfileRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    match crate::profile::create_profile(
        &request.full_name,
        &request.username,
        request.language.as_deref().unwrap_or("tr"),
        request.theme.as_deref().unwrap_or("dark"),
        request.open_directly.unwrap_or(false),
    ) {
        Ok(profile) => json_ok(json!({
            "profile": profile,
            "access": crate::profile::mobile_tools_access(),
            "settings_path": crate::settings::default_settings_path(),
            "case_base_dir": crate::api::default_case_base_dir(),
        })),
        Err(err) => json_error(400, err.to_string()),
    }
}

/// Var olan profili aktif eder.
pub fn profile_select_endpoint(body: &[u8]) -> Response {
    let request: SelectProfileRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    match crate::profile::select_profile(&request.username, request.open_directly.unwrap_or(false))
    {
        Ok(profile) => json_ok(json!({
            "profile": profile,
            "access": crate::profile::mobile_tools_access(),
            "settings_path": crate::settings::default_settings_path(),
            "case_base_dir": crate::api::default_case_base_dir(),
        })),
        Err(err) => {
            let status = match err.code {
                crate::HataKodu::IcerikGecersiz => 404,
                crate::HataKodu::YetkisizErisim | crate::HataKodu::TokenGecersiz => 401,
                _ => 500,
            };
            json_error(status, err.to_string())
        }
    }
}

/// Online site hesabını yerel profile bağlar ve aktif profili günceller.
pub fn profile_online_login_endpoint(body: &[u8]) -> Response {
    let request: OnlineProfileLoginRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    match crate::profile::link_online_profile(
        &request.identifier,
        &request.password,
        request.language.as_deref().unwrap_or("tr"),
        request.theme.as_deref().unwrap_or("dark"),
        request.open_directly.unwrap_or(false),
    ) {
        Ok(profile) => json_ok(json!({
            "profile": profile,
            "access": crate::profile::mobile_tools_access(),
            "settings_path": crate::settings::default_settings_path(),
            "case_base_dir": crate::api::default_case_base_dir(),
        })),
        Err(err) => json_error(401, err.to_string()),
    }
}

/// Aktif online profil bilgilerini site API'sinden yeniler.
pub fn profile_online_sync_endpoint() -> Response {
    match crate::profile::sync_active_online_profile() {
        Ok(profile) => {
            let status = profile
                .online
                .as_ref()
                .and_then(|o| o.status.clone())
                .unwrap_or_else(|| "offline".to_string());
            let is_offline = status == "offline";
            let is_expired = status == "session_expired";
            json_ok(json!({
                "ok": status == "online",
                "status": status,
                "offline": is_offline || is_expired,
                "session_expired": is_expired,
                "profile": profile,
                "access": crate::profile::mobile_tools_access(),
            }))
        }
        Err(err) => json_error(400, err.to_string()),
    }
}

/// Aktif profilden online hesap bağlantısını kaldırır.
pub fn profile_online_logout_endpoint() -> Response {
    match crate::profile::disconnect_active_online_profile() {
        Ok(profile) => json_ok(json!({
            "profile": profile,
            "access": crate::profile::mobile_tools_access(),
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Android/iOS araçları için online üyelik durumunu döndürür.
pub fn profile_mobile_access_endpoint() -> Response {
    match crate::profile::require_mobile_tools_access() {
        Ok(()) => json_ok(json!({
            "access": {
                "allowed": true,
                "profile": crate::profile::active_profile(),
            }
        })),
        Err(err) => json_ok(json!({
            "access": {
                "allowed": false,
                "reason": err.to_string(),
                "profile": crate::profile::active_profile(),
            }
        })),
    }
}

/// Mobil araç API'leri için üyelik kilidini uygular.
pub fn require_mobile_tools_response() -> Option<Response> {
    match crate::profile::require_mobile_tools_access() {
        Ok(()) => None,
        Err(err) => Some(json_error(403, err.to_string())),
    }
}

/// Aktif profilden çıkar ve sonraki açılışta profil seçimini gösterir.
pub fn profile_logout_endpoint() -> Response {
    match crate::profile::logout_profile() {
        Ok(()) => json_ok(json!({ "ok": true })),
        Err(err) => json_error(500, err.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct OnlineReportSubmitRequest {
    pub title: String,
    pub description: String,
    #[serde(default, rename = "imageUrls", alias = "image_urls")]
    pub image_urls: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct OnlineReportUploadImageRequest {
    #[serde(rename = "imageBase64", alias = "image_base64")]
    pub image_base64: String,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default, rename = "contentType", alias = "content_type")]
    pub content_type: Option<String>,
}

/// amele.noirlang.tr online portalına hata/öneri raporu gönderir.
pub fn profile_report_submit_endpoint(body: &[u8]) -> Response {
    let request: OnlineReportSubmitRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    match crate::profile::submit_online_report(
        &request.title,
        &request.description,
        &request.image_urls,
    ) {
        Ok(result) => json_ok(json!({ "ok": true, "result": result })),
        Err(err) => {
            let status = match err.code {
                crate::HataKodu::YetkisizErisim | crate::HataKodu::TokenGecersiz => 401,
                crate::HataKodu::IcerikGecersiz => 400,
                _ => 500,
            };
            json_error(status, err.to_string())
        }
    }
}

/// amele.noirlang.tr online portalına ekran görüntüsü yükler.
pub fn profile_report_upload_image_endpoint(body: &[u8]) -> Response {
    let request: OnlineReportUploadImageRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    let base64_str = if let Some(idx) = request.image_base64.find(',') {
        &request.image_base64[idx + 1..]
    } else {
        &request.image_base64
    };

    use base64::Engine;
    let image_bytes = match base64::engine::general_purpose::STANDARD.decode(base64_str.trim()) {
        Ok(bytes) => bytes,
        Err(err) => return json_error(400, format!("Geçersiz base64 görsel verisi: {err}")),
    };

    let filename = request
        .filename
        .unwrap_or_else(|| "screenshot.png".to_string());
    let content_type = request
        .content_type
        .unwrap_or_else(|| "image/png".to_string());

    match crate::profile::upload_online_image(&image_bytes, &filename, &content_type) {
        Ok(url) => json_ok(json!({ "ok": true, "url": url })),
        Err(err) => json_error(500, err.to_string()),
    }
}

