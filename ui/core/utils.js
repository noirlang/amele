// boyut formatlama, tarih saat çevirme gibi ufak tefek yardımcı fonksiyonlar.

export function timestampForFileName(date = new Date()) {
  const pad = (value) => String(value).padStart(2, "0");
  return `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}_${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`;
}

export function sanitizeFileStem(value) {
  return String(value || "")
    .trim()
    .replace(/[<>:"/\\|?*\x00-\x1F\s]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

export function canonicalRamFileName(remoteIp = "", date = new Date(), platform = "") {
  const ip = sanitizeFileStem(remoteIp);
  const plat = String(platform || "")
    .toLowerCase()
    .includes("win")
    ? "win"
    : String(platform || "")
          .toLowerCase()
          .includes("lin")
      ? "linux"
      : "";
  const parts = [ip, plat, "ram"].filter(Boolean);
  return `${parts.join("_")}_${timestampForFileName(date)}.raw`;
}

export function formatBytes(bytes) {
  const value = Number(bytes || 0);
  if (!Number.isFinite(value) || value <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let size = value;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return `${size.toFixed(size >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

export function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

export function compactLogLine(message) {
  return String(message || "")
    .replace(/\s+/g, " ")
    .trim();
}
