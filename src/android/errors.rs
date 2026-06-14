pub fn explain_android_error(error: impl AsRef<str>) -> String {
    let raw = error.as_ref().trim();
    if raw.is_empty() {
        return "Android islemi basarisiz oldu. Ayrinti alinamadi.".to_string();
    }

    let lower = raw.to_ascii_lowercase();
    let guidance = if contains_any(&lower, &["adb bulunamadi", "no such file", "not found"])
        && lower.contains("adb")
    {
        Some("ADB bulunamadi. Android Platform Tools kurulu ve PATH icinde olmalidir.")
    } else if lower.contains("unauthorized") {
        Some(
            "Cihaz ADB icin yetkilendirilmemis. Telefonda USB hata ayiklama onayini kabul edin ve tekrar deneyin.",
        )
    } else if contains_any(
        &lower,
        &[
            "no devices/emulators found",
            "no devices",
            "no device found",
            "device not found",
            "cihaz bulunamadi",
            "cihaz bulunamadı",
        ],
    ) {
        Some(
            "ADB cihazi bulamadi. USB kablosunu, USB hata ayiklamayi ve cihaz secimini kontrol edin.",
        )
    } else if lower.contains("offline") {
        Some(
            "Cihaz offline gorunuyor. USB baglantisini yenileyin, ADB yetkisini iptal edip yeniden onaylayin.",
        )
    } else if lower.contains("more than one device") || lower.contains("more than one emulator") {
        Some(
            "Birden fazla Android hedefi var. Listeden tek cihaz secin veya diger cihazlari ayirin.",
        )
    } else if contains_any(&lower, &["permission denied", "operation not permitted"]) {
        Some(
            "Cihaz bu islemi engelledi. Bu adim root yetkisi, ozel izin veya desteklenen Android surumu gerektirebilir.",
        )
    } else if contains_any(&lower, &["read-only file system", "not permitted"]) {
        Some(
            "Dosya sistemi yazma/okuma izni vermedi. Root modu ve hedef yol izinlerini kontrol edin.",
        )
    } else if lower.contains("closed") || lower.contains("connection reset") {
        Some(
            "ADB baglantisi islem sirasinda koptu. Kabloyu, cihaz ekran kilidini ve ADB servisini kontrol edin.",
        )
    } else if lower.contains("timeout") || lower.contains("zaman asimina") {
        Some(
            "ADB komutu zaman asimina ugradi. Cihaz yanit vermiyor veya islem beklenenden uzun suruyor.",
        )
    } else if contains_any(
        &lower,
        &[
            "su: not found",
            "inaccessible or not found",
            "adbd cannot run as root",
        ],
    ) {
        Some(
            "Root erisimi kullanilamiyor. Root gerektiren Android edinimleri bu cihazda calismayabilir.",
        )
    } else if lower.contains("bugreport") && contains_any(&lower, &["failed", "error"]) {
        Some(
            "Bug report alinamadi. Cihaz Android surumu, depolama alani veya ADB yetkisi nedeniyle islemi reddediyor olabilir.",
        )
    } else {
        None
    };

    match guidance {
        Some(message) if raw.contains(message) => raw.to_string(),
        Some(message) => format!("{message} Ayrinti: {raw}"),
        None => format!("Android islemi basarisiz oldu. Ayrinti: {raw}"),
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explains_unauthorized_devices() {
        let message = explain_android_error("device unauthorized");
        assert!(message.contains("USB hata ayiklama"));
    }

    #[test]
    fn explains_missing_adb() {
        let message = explain_android_error("ADB bulunamadi");
        assert!(message.contains("Platform Tools"));
    }

    #[test]
    fn explains_missing_device() {
        let message = explain_android_error("Cihaz bulunamadı");
        assert!(message.contains("ADB cihazi bulamadi"));
    }

    #[test]
    fn explains_permission_denied() {
        let message = explain_android_error("permission denied");
        assert!(message.contains("root"));
    }
}
