export const ACTIVE_CASE_STORAGE_KEY = "amele-active-case";

export function getPersistedCaseName(username = "") {
  try {
    if (username) {
      const perUser = localStorage.getItem(`${ACTIVE_CASE_STORAGE_KEY}:${username}`)?.trim();
      if (perUser) return perUser;
    }
    return localStorage.getItem(ACTIVE_CASE_STORAGE_KEY)?.trim() || "";
  } catch (_) {
    return "";
  }
}

export function persistCaseName(caseName, username = "") {
  const normalized = typeof caseName === "string" ? caseName.trim() : "";
  try {
    if (normalized) {
      localStorage.setItem(ACTIVE_CASE_STORAGE_KEY, normalized);
      if (username) {
        localStorage.setItem(`${ACTIVE_CASE_STORAGE_KEY}:${username}`, normalized);
      }
    } else {
      localStorage.removeItem(ACTIVE_CASE_STORAGE_KEY);
      if (username) {
        localStorage.removeItem(`${ACTIVE_CASE_STORAGE_KEY}:${username}`);
      }
    }
  } catch (_) {}

  // Backend ayarlar.json dosyasına da kaydet
  if (typeof window !== "undefined" && typeof window.fetch === "function") {
    try {
      fetch("/api/evidence-select", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ case_name: normalized }),
      }).catch(() => {});
    } catch (_) {}
  }
}
