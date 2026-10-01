//! Uygulama ayarlarını okuma ve kaydetme API uç noktaları.

use serde::Deserialize;
use serde_json::json;

use crate::server::{Response, json_error, json_ok};

/// UI ayar kaydetme isteğinde tema ve dil tercihini taşır.
#[derive(Debug, Deserialize, PartialEq, Eq)]
struct SaveSettingsRequest {
    theme: Option<String>,
    language: Option<String>,
}

/// Kalıcı uygulama ayarlarını dosyadan okur.
pub fn settings_get_endpoint() -> Response {
    let path = crate::settings::default_settings_path();
    match crate::settings::AppSettings::load(&path) {
        Ok(settings) => json_ok(json!({
            "settings": settings,
            "path": path,
        })),
        Err(err) => json_error(500, err.to_string()),
    }
}

/// Tema ve dil tercihini kalıcı ayar dosyasına yazar.
pub fn settings_save_endpoint(body: &[u8]) -> Response {
    let request: SaveSettingsRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    if let Some(theme) = request.theme.as_deref() {
        if theme != "dark" && theme != "light" {
            return json_error(400, format!("unsupported theme: {theme}"));
        }
    }

    if let Some(language) = request.language.as_deref() {
        if language != "tr" && language != "en" {
            return json_error(400, format!("unsupported language: {language}"));
        }
    }

    let path = crate::settings::default_settings_path();
    let mut settings = match crate::settings::AppSettings::load(&path) {
        Ok(settings) => settings,
        Err(err) => return json_error(500, err.to_string()),
    };

    if let Some(theme) = request.theme.as_deref() {
        settings.karanlik_tema = theme == "dark";
    }

    if let Some(language) = request.language.as_deref() {
        settings.dil = language.to_string();
    }

    settings.normalize();
    match settings.save(&path) {
        Ok(()) => {
            let theme_str = if settings.karanlik_tema {
                "dark"
            } else {
                "light"
            };
            let _ = crate::profile::update_active_preferences(&settings.dil, theme_str);
            let _ = crate::profile::record_active_profile_activity(
                "settings",
                "update",
                None,
                Some(&format!("theme={theme_str} language={}", settings.dil)),
            );
            crate::logging::runtime_log(
                crate::logging::LogLevel::Info,
                "api:settings",
                format!(
                    "Ayarlar kaydedildi: theme={theme_str} language={} path={}",
                    settings.dil,
                    path.display()
                ),
            );
            json_ok(json!({
                "settings": settings,
                "path": path,
            }))
        }
        Err(err) => json_error(500, err.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SETTINGS_TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn test_settings_get_endpoint() {
        let _guard = SETTINGS_TEST_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        let resp = settings_get_endpoint();
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val.get("settings").is_some());
        assert!(val.get("path").is_some());
    }

    #[test]
    fn test_settings_save_endpoint_validation() {
        let _guard = SETTINGS_TEST_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        // Empty body
        let resp = settings_save_endpoint(b"");
        assert_eq!(resp.status, 400);

        // Invalid JSON
        let resp = settings_save_endpoint(b"not a json");
        assert_eq!(resp.status, 400);

        // Unsupported theme
        let resp = settings_save_endpoint(br#"{"theme":"neon"}"#);
        assert_eq!(resp.status, 400);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(val["error"].as_str().unwrap().contains("unsupported theme"));

        // Unsupported language
        let resp = settings_save_endpoint(br#"{"language":"fr"}"#);
        assert_eq!(resp.status, 400);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert!(
            val["error"]
                .as_str()
                .unwrap()
                .contains("unsupported language")
        );
    }

    #[test]
    fn test_settings_save_endpoint_valid_updates() {
        let _guard = SETTINGS_TEST_MUTEX
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        // Save dark theme and Turkish
        let resp = settings_save_endpoint(br#"{"theme":"dark","language":"tr"}"#);
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert_eq!(val["settings"]["karanlik_tema"], true);
        assert_eq!(val["settings"]["dil"], "tr");

        // Save light theme and English
        let resp = settings_save_endpoint(br#"{"theme":"light","language":"en"}"#);
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).expect("valid json");
        assert_eq!(val["settings"]["karanlik_tema"], false);
        assert_eq!(val["settings"]["dil"], "en");
    }
}
