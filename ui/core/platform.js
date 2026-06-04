export function detectPlatform() {
  const override = new URLSearchParams(window.location.search).get("platform");
  if (["windows", "linux", "android", "mac"].includes(override || "")) return override;
  const text = `${navigator.userAgent} ${navigator.platform}`.toLowerCase();
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
