// ============================================================================
// # ÇALIŞMA PLATFORMU VE İŞLETİM SİSTEMİ TESPİTİ (ui/core/platform.js)
// ============================================================================
//
// Bu modül, tarayıcı ortamından veya gömülü WebView'den işletim sistemini
// (Windows, Linux, Android, macOS) tespit eder; platforma özgü arayüz
// optimizasyonlarını ve dosya yolu ayrıştırıcılarını (`/` vs `\`) etkinleştirir.
// ============================================================================

export function detectPlatform() {
  const override = typeof window !== "undefined" && window.location ? new URLSearchParams(window.location.search).get("platform") : null;
  if (["windows", "linux", "android", "mac"].includes(override || "")) return override;
  const ua = typeof navigator !== "undefined" && navigator.userAgent ? navigator.userAgent : "";
  const plat = typeof navigator !== "undefined" && navigator.platform ? navigator.platform : "";
  const text = `${ua} ${plat}`.toLowerCase();
  if (text.includes("android")) return "android";
  if (text.includes("win")) return "windows";
  if (text.includes("linux")) return "linux";
  if (text.includes("mac")) return "mac";
  return "unknown";
}

export function platformLabel(platform, unknownLabel = "Unknown") {
  if (platform === "windows") return "Windows";
  if (platform === "linux") return "Linux";
  if (platform === "android") return "Android";
  if (platform === "mac") return "macOS";
  return unknownLabel;
}
