import { explainErrorMessage } from "./errors.js";

// Backend'e hata log satırı gönderen tek yönlü fire-and-forget fonksiyon.
// Hata oluşursa sessizce geçilir — kendisi hata üretmemelidir.
function _reportToDevLog(level, scope, message) {
  try {
    fetch("/api/developer-log", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ level, scope, message }),
    }).catch(() => {});
  } catch {
    // Sessizce yut
  }
}

export function createApiRequest({ backendAvailable }) {
  return async function apiRequest(path, options = {}) {
    const headers = new Headers(options.headers || {});
    if (options.body && !headers.has("content-type")) {
      headers.set("content-type", "application/json");
    }

    const t0 = typeof performance !== "undefined" ? performance.now() : Date.now();

    let response;
    try {
      response = await fetch(path, { ...options, headers });
    } catch (error) {
      const msg = formatBackendConnectionError(path, error, backendAvailable);
      // Bağlantı hatasını backend'e raporla (dev log için)
      _reportToDevLog("ERROR", `ui:api:fetch`, `BAGLANTI HATASI ${path} — ${error?.message || error}`);
      throw new Error(msg);
    }

    const elapsed = ((typeof performance !== "undefined" ? performance.now() : Date.now()) - t0).toFixed(1);
    const text = await response.text();

    let data = {};
    if (text) {
      try {
        data = JSON.parse(text);
      } catch {
        const msg = formatInvalidResponseError(path, response, text);
        _reportToDevLog(
          "ERROR",
          `ui:api:json`,
          `GECERSIZ JSON ${path} HTTP=${response.status} [${elapsed}ms] — ${String(text || "").slice(0, 200)}`
        );
        throw new Error(msg);
      }
    }

    if (!response.ok) {
      const msg = formatApiError(path, response, data);
      const level = response.status >= 500 ? "ERROR" : "WARN";
      _reportToDevLog(
        level,
        `ui:api:${response.status}`,
        `API HATA ${options.method || "GET"} ${path} HTTP=${response.status} [${elapsed}ms] — ${data.error || response.statusText || "bilinmeyen hata"} | kod=${data.code || "?"} | öneri=${data.suggestion || "?"}`
      );
      throw new Error(msg);
    }

    // Başarılı ama yavaş istekleri WARN olarak logla (>2000ms)
    if (parseFloat(elapsed) > 2000) {
      _reportToDevLog(
        "WARN",
        `ui:api:slow`,
        `YAVAS YANIT ${path} [${elapsed}ms]`
      );
    }

    return data;
  };
}

function formatBackendConnectionError(path, error, backendAvailable) {
  const advice = explainErrorMessage(error?.message || error || "fetch failed");
  return [
    "Backend bağlantısı kurulamadı.",
    `İstek: ${path}`,
    `Ayrıntı: ${error?.message || error || "fetch failed"}`,
    `Kod: ${advice.code}`,
    `Neden: ${advice.detail}`,
    backendAvailable
      ? "Çözüm: Uygulama backend süreci kapanmış olabilir; Amele'u yeniden başlatın."
      : "Çözüm: Bu işlem sadece masaüstü uygulama modunda çalışır."
  ].join("\n");
}

function formatInvalidResponseError(path, response, text) {
  const body = String(text || "").trim().slice(0, 900) || "(boş yanıt)";
  const advice = explainErrorMessage(body);
  return [
    "Backend geçersiz yanıt döndürdü.",
    `HTTP: ${response.status} ${response.statusText || ""}`.trim(),
    `İstek: ${path}`,
    `Yanıt: ${body}`,
    `Kod: ${advice.code}`,
    `Neden: ${advice.detail}`,
    "Çözüm: Uygulama dosyaları eksik olabilir veya endpoint beklenmeyen HTML/metin döndürmüş olabilir."
  ].join("\n");
}

function formatApiError(path, response, data) {
  const rawMessage = data.error || response.statusText || "İşlem başarısız.";
  const advice = data.detail || data.suggestion ? null : explainErrorMessage(rawMessage);
  const lines = [
    rawMessage,
    `HTTP: ${response.status} ${response.statusText || ""}`.trim(),
    `İstek: ${path}`
  ];
  if (data.code || advice?.code) lines.push(`Kod: ${data.code || advice.code}`);
  if (data.detail && data.detail !== data.error) lines.push(`Neden: ${data.detail}`);
  if (!data.detail && advice?.detail) lines.push(`Neden: ${advice.detail}`);
  if (data.suggestion) lines.push(`Çözüm: ${data.suggestion}`);
  if (!data.suggestion && advice?.suggestion) lines.push(`Çözüm: ${advice.suggestion}`);
  return lines.filter(Boolean).join("\n");
}
