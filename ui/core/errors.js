import { escapeHtml } from "./utils.js";

const KNOWN_ERROR_RULES = [
  {
    code: "ELEVATION_FAILED",
    patterns: ["uac", "runas", "pkexec", "sudo", "askpass", "polkit", "yetki yükseltme", "yetki yukseltme", "root yetkisi", "administrator privileges", "requires administrator"],
    detail: "İşlem root/administrator yetkisi gerektiriyor ancak yetki onayı tamamlanamadı.",
    suggestion: "Linux'ta sudo/pkexec parola penceresini onaylayın; pencere açılmıyorsa polkit agent veya zenity/kdialog/ssh-askpass kurun. Windows'ta UAC penceresini onaylayın veya Worm'u yönetici olarak başlatın."
  },
  {
    code: "PERMISSION_DENIED",
    patterns: ["permission denied", "access denied", "erişim engellendi", "erisim engellendi", "os error 13", "yetki"],
    detail: "İşlem için gerekli sistem yetkisi alınamadı.",
    suggestion: "Linux'ta sudo/pkexec onayını verin. Windows'ta UAC penceresini onaylayın veya uygulamayı yönetici olarak açın."
  },
  {
    code: "INSUFFICIENT_SPACE",
    patterns: ["no space left", "insufficient space", "yetersiz boş", "yetersiz bos", "os error 28", "partial_size", "write block failed"],
    detail: "Çıktı veya geçici dosyalar için yeterli boş alan yok.",
    suggestion: "Vaka klasörünü daha geniş bir diske taşıyın. RAM imajında /tmp tmpfs ise disk üzerindeki bir klasörü kullanın."
  },
  {
    code: "IMAGE_MOUNT_FAILED",
    patterns: ["wrong fs type", "bad superblock", "mount failed", "losetup failed", "mounted image has no drive letter", "raw dd/img", "diskimage"],
    detail: "Disk imajı doğrudan salt-okunur bağlanamadı.",
    suggestion: "İmajın tamamlandığını ve hashinin doğru olduğunu kontrol edin. Windows raw DD/IMG için native klasör gezintisi forensic sürücü gerektirebilir."
  },
  {
    code: "VOLATILITY_SYMBOLS",
    patterns: ["symbol_table_name", "layer_name", "unsatisfied requirement", "translation layer", "kernel banner", "volatility3"],
    detail: "Volatility3 RAM imajı için gerekli profil/sembol bilgisini çözemedi.",
    suggestion: "Doğru OS profilini seçin. Linux imajlarında kernel banner ile uyumlu sembol dosyasını .symbols klasörüne koyun."
  },
  {
    code: "ANDROID_ADB_MISSING",
    patterns: ["adb bulunamadı", "adb bulunamadi", "android debug bridge bulunamadı", "android debug bridge bulunamadi", "platform tools", "failed to execute adb"],
    detail: "Android Debug Bridge bulunamadı veya çalıştırılamadı.",
    suggestion: "Android Platform Tools kurulu olmalı ve adb komutu PATH içinde görünmelidir. ADB kontrolünü tekrar çalıştırın."
  },
  {
    code: "ADB_DEVICE",
    patterns: ["unauthorized", "offline", "no devices", "no device found", "device not found", "cihaz bulunamadı", "cihaz bulunamadi", "more than one device", "more than one emulator"],
    detail: "Android cihaz ADB üzerinden hazır değil.",
    suggestion: "USB hata ayıklamayı açın, cihazdaki RSA onayını kabul edin ve tek hedef cihaz bağlıyken listelemeyi tekrar çalıştırın."
  },
  {
    code: "TOOL_MISSING",
    patterns: ["avml", "winpmem", "not installed", "command not found", "araç bulunamadı", "arac bulunamadi"],
    detail: "Gerekli edinim aracı bulunamadı veya çalıştırılamadı.",
    suggestion: "Araç kontrolünü tekrar çalıştırın ve varsa indir/kur butonuyla aracı Worm'un beklediği konuma kurun."
  },
  {
    code: "CONNECTION_FAILED",
    patterns: ["connection refused", "connection reset", "timed out", "timeout", "bağlantı", "baglanti", "network", "firewall"],
    detail: "Agent veya backend bağlantısı kurulamadı ya da işlem zaman aşımına uğradı.",
    suggestion: "IP, port, token, firewall ve VPN durumunu kontrol edin. Agent'ın hedef makinede açık olduğundan emin olun."
  },
  {
    code: "FILE_NOT_FOUND",
    patterns: ["not found", "no such file", "bulunamadı", "bulunamadi", "file not found"],
    detail: "Gerekli dosya veya klasör bulunamadı.",
    suggestion: "Seçilen yolun var olduğunu, dosyanın taşınmadığını ve uygulamanın bu yola erişebildiğini kontrol edin."
  },
  {
    code: "FUSE_APPIMAGE",
    patterns: ["fuse", "cannot mount appimage", "appimage", "extract-and-run"],
    detail: "AppImage çalıştırmak için FUSE kurulumu veya AppImage mount desteği yok.",
    suggestion: "FUSE paketini kurun veya uygulamayı APPIMAGE_EXTRACT_AND_RUN=1 ile çalıştırmayı deneyin."
  },
  {
    code: "WEBVIEW_RUNTIME",
    patterns: ["webview2", "webkitnetworkprocess", "webkit", "vcruntime", "msvcp", "glibc"],
    detail: "Native pencere için gerekli sistem runtime bileşeni eksik veya uyumsuz.",
    suggestion: "Windows'ta WebView2/Visual C++ runtime, Linux'ta WebKitGTK ve uyumlu paket sürümünü kontrol edin."
  },
  {
    code: "INVALID_DATA",
    patterns: ["json", "parse", "invalid", "geçersiz", "gecersiz", "bozuk"],
    detail: "Beklenen veri formatı okunamadı.",
    suggestion: "Agent ve uygulama sürümlerinin aynı olduğundan emin olun; dosya bozuksa edinimi tekrar alın."
  }
];

export function explainErrorMessage(message) {
  const text = String(message || "").trim();
  const lower = text.toLocaleLowerCase("tr-TR");
  return KNOWN_ERROR_RULES.find((rule) => rule.patterns.some((pattern) => lower.includes(pattern))) || {
    code: "UNEXPECTED_ERROR",
    detail: "İşlem beklenmeyen bir hata ile durdu.",
    suggestion: "Ayrıntı mesajını kontrol edin; tekrar denemeden önce yetki, disk alanı, seçilen yol ve bağlantı durumunu doğrulayın."
  };
}

export function normalizeErrorMessage(message, fallback = "İşlem tamamlanamadı.") {
  const text = String(message || fallback).trim() || fallback;
  if (hasStructuredAdvice(text)) return text;
  const advice = explainErrorMessage(text);
  return [
    text,
    `Kod: ${advice.code}`,
    `Neden: ${advice.detail}`,
    `Çözüm: ${advice.suggestion}`
  ].join("\n");
}

export function errorBoxHtml(title, message) {
  const body = normalizeErrorMessage(message);
  return `
    <div class="error-detail-box">
      <strong>${escapeHtml(title || "Hata")}</strong>
      <pre>${escapeHtml(body)}</pre>
    </div>
  `;
}

function hasStructuredAdvice(text) {
  return /(^|\n)(Kod|Code|Neden|Reason|Çözüm|Cozum|Suggestion):/i.test(text);
}
