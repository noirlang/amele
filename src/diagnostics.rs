use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct ErrorAdvice {
    pub code: &'static str,
    pub detail: &'static str,
    pub suggestion: &'static str,
}

pub fn classify_error(message: &str) -> ErrorAdvice {
    let lower = message.to_lowercase();

    if lower.contains("webview2") {
        return ErrorAdvice {
            code: "WEBVIEW2_RUNTIME",
            detail: "Windows native pencere WebView2 runtime olmadan acilamaz.",
            suggestion: "Microsoft Edge WebView2 Evergreen Runtime kurun, sonra Worm'u tekrar baslatin.",
        };
    }

    if lower.contains("vcruntime") || lower.contains("msvcp") {
        return ErrorAdvice {
            code: "WINDOWS_RUNTIME_DLL",
            detail: "Windows Visual C++ runtime DLL dosyasi sistemde bulunamadi.",
            suggestion: "Guncel Worm paketini kullanin. Eski paketlerde Microsoft Visual C++ Redistributable kurulumu gerekebilir.",
        };
    }

    if lower.contains("webkitnetworkprocess") || lower.contains("webkit helper") {
        return ErrorAdvice {
            code: "WEBKIT_HELPER_MISSING",
            detail: "Linux WebKit alt sureci bulunamadigi icin native pencere baslatilamiyor.",
            suggestion: "Dağıtım WebKitGTK paketlerini kurun veya WebKitNetworkProcess iceren AppImage/.deb/.rpm paketini kullanin.",
        };
    }

    if lower.contains("gtk")
        && (lower.contains("display") || lower.contains("cannot open") || lower.contains("ekran"))
    {
        return ErrorAdvice {
            code: "DISPLAY_UNAVAILABLE",
            detail: "GTK ekrana baglanamadi; grafik oturum veya DISPLAY/WAYLAND ayari uygun degil.",
            suggestion: "Uygulamayi masaustu oturumunda baslatin. SSH/terminal uzerindeyseniz X11/Wayland erisimini kontrol edin.",
        };
    }

    if lower.contains("permission denied")
        || lower.contains("access denied")
        || lower.contains("erisim engellendi")
        || lower.contains("erişim engellendi")
        || lower.contains("os error 13")
    {
        return ErrorAdvice {
            code: "PERMISSION_DENIED",
            detail: "Islem icin gereken disk, RAM veya sistem dosyasi yetkisi yok.",
            suggestion: "Linux'ta pkexec/sudo yetkilendirmesini onaylayin. Windows'ta uygulamayi yonetici olarak baslatin veya UAC iznini verin.",
        };
    }

    if lower.contains("no space left")
        || lower.contains("yetersiz bos")
        || lower.contains("yetersiz boş")
        || lower.contains("insufficient space")
        || lower.contains("os error 28")
    {
        return ErrorAdvice {
            code: "INSUFFICIENT_SPACE",
            detail: "Cikti veya gecici dosyalar icin yeterli bos alan yok.",
            suggestion: "Vaka klasorunu daha genis bir diske tasiyin. Linux'ta /tmp tmpfs ise RAM dump sirasinda /var/tmp veya disk uzerindeki bir klasor kullanin.",
        };
    }

    if lower.contains("unable to create memory snapshot")
        || lower.contains("/proc/kcore")
        || lower.contains("write block failed")
    {
        return ErrorAdvice {
            code: "RAM_SNAPSHOT_FAILED",
            detail: "RAM imaji uretilirken cekirdek bellegi veya gecici snapshot yazimi basarisiz oldu.",
            suggestion: "Root yetkisini, kernel lockdown durumunu, /tmp kapasitesini ve cikti diskindeki bos alani kontrol edin.",
        };
    }

    if lower.contains("wrong fs type")
        || lower.contains("bad superblock")
        || lower.contains("mount failed")
        || lower.contains("losetup failed")
    {
        return ErrorAdvice {
            code: "IMAGE_MOUNT_FAILED",
            detail: "Disk imaji salt-okunur baglanamadi; imaj bolum tablosu, dosya sistemi veya yetki sorunu icerebilir.",
            suggestion: "Imajin tamamlandigini/hash dogrulamasini kontrol edin. Linux'ta mount/losetup icin root yetkisi ve gerekli dosya sistemi paketleri gerekir.",
        };
    }

    if lower.contains("connection refused")
        || lower.contains("timed out")
        || lower.contains("timeout")
        || lower.contains("baglanti")
        || lower.contains("bağlantı")
        || lower.contains("connection reset")
    {
        return ErrorAdvice {
            code: "CONNECTION_FAILED",
            detail: "Agent veya yerel backend ile ag baglantisi kurulamadi ya da islem zaman asimina ugradi.",
            suggestion: "IP/port/token bilgisini, firewall kurallarini, agent'in calistigini ve ayni ag/VPN erisimini kontrol edin.",
        };
    }

    if lower.contains("address already in use") || lower.contains("only one usage") {
        return ErrorAdvice {
            code: "LOCAL_PORT_BUSY",
            detail: "Yerel UI backend portu acilamadi.",
            suggestion: "Arka planda kalmis Worm sureclerini kapatin ve uygulamayi tekrar baslatin.",
        };
    }

    if lower.contains("not found")
        || lower.contains("no such file")
        || lower.contains("bulunamadi")
        || lower.contains("bulunamadı")
    {
        return ErrorAdvice {
            code: "FILE_NOT_FOUND",
            detail: "Gerekli dosya, arac veya paketlenen UI varligi bulunamadi.",
            suggestion: "Secilen yolu ve kurulum paketini kontrol edin. Paket icinden calisiyorsaniz uygulamayi yeniden kurun.",
        };
    }

    if lower.contains("json") || lower.contains("parse") || lower.contains("invalid") {
        return ErrorAdvice {
            code: "INVALID_DATA",
            detail: "Beklenen veri formati okunamadi veya bozuk geldi.",
            suggestion: "Islemi tekrar deneyin. Agent ve uygulama surumlerinin ayni oldugundan emin olun.",
        };
    }

    ErrorAdvice {
        code: "UNEXPECTED_ERROR",
        detail: "Islem beklenmeyen bir hata ile durdu.",
        suggestion: "Ayrinti mesajini ve log dosyasini kontrol edin; ayni adimi tekrar calistirmadan once yetki, disk alani ve secilen yolları dogrulayin.",
    }
}

pub fn startup_error(stage: &str, message: &str) -> String {
    let advice = classify_error(message);
    format!(
        "{stage}\n\nAyrinti: {message}\nKod: {}\nNeden: {}\nCozum: {}",
        advice.code, advice.detail, advice.suggestion
    )
}

pub fn ui_assets_missing(root: &Path) -> String {
    startup_error(
        "Arayuz dosyalari bulunamadi.",
        &format!(
            "index.html beklenen klasorde yok: {}",
            root.join("index.html").display()
        ),
    )
}

pub fn panic_payload(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(value) = payload.downcast_ref::<&str>() {
        return (*value).to_string();
    }
    if let Some(value) = payload.downcast_ref::<String>() {
        return value.clone();
    }
    "panic payload could not be decoded".to_string()
}

#[cfg(test)]
mod tests {
    use super::classify_error;

    #[test]
    fn classifies_permission_errors() {
        assert_eq!(
            classify_error("Disk access error: Access denied (os error 13)").code,
            "PERMISSION_DENIED"
        );
    }

    #[test]
    fn classifies_webview2_errors() {
        assert_eq!(
            classify_error("WebView2 view could not be created").code,
            "WEBVIEW2_RUNTIME"
        );
    }
}
