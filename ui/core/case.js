export const ACTIVE_CASE_STORAGE_KEY = "amele-active-case";

export function getPersistedCaseName() {
  try {
    return localStorage.getItem(ACTIVE_CASE_STORAGE_KEY)?.trim() || "";
  } catch (_) {
    return "";
  }
}

export function persistCaseName(caseName) {
  const normalized = typeof caseName === "string" ? caseName.trim() : "";
  try {
    if (normalized) {
      localStorage.setItem(ACTIVE_CASE_STORAGE_KEY, normalized);
    } else {
      localStorage.removeItem(ACTIVE_CASE_STORAGE_KEY);
    }
  } catch (_) {}
}
