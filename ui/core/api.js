export function createApiRequest({ backendAvailable }) {
  return async function apiRequest(path, options = {}) {
    const headers = new Headers(options.headers || {});
    if (options.body && !headers.has("content-type")) {
      headers.set("content-type", "application/json");
    }
    let response;
    try {
      response = await fetch(path, { ...options, headers });
    } catch (error) {
      throw new Error(formatBackendConnectionError(path, error, backendAvailable));
    }
    const text = await response.text();
    let data = {};
    if (text) {
      try {
        data = JSON.parse(text);
      } catch {
        throw new Error(formatInvalidResponseError(path, response, text));
      }
    }
    if (!response.ok) {
      throw new Error(formatApiError(path, response, data));
    }
    return data;
  };
}

function formatBackendConnectionError(path, error, backendAvailable) {
  return [
    "Backend bağlantısı kurulamadı.",
    `İstek: ${path}`,
    `Ayrıntı: ${error?.message || error || "fetch failed"}`,
    backendAvailable
      ? "Çözüm: Uygulama backend süreci kapanmış olabilir; Worm'u yeniden başlatın."
      : "Çözüm: Bu işlem sadece masaüstü uygulama modunda çalışır."
  ].join("\n");
}

function formatInvalidResponseError(path, response, text) {
  const body = String(text || "").trim().slice(0, 900) || "(boş yanıt)";
  return [
    "Backend geçersiz yanıt döndürdü.",
    `HTTP: ${response.status} ${response.statusText || ""}`.trim(),
    `İstek: ${path}`,
    `Yanıt: ${body}`,
    "Çözüm: Uygulama dosyaları eksik olabilir veya endpoint beklenmeyen HTML/metin döndürmüş olabilir."
  ].join("\n");
}

function formatApiError(path, response, data) {
  const lines = [
    data.error || response.statusText || "İşlem başarısız.",
    `HTTP: ${response.status} ${response.statusText || ""}`.trim(),
    `İstek: ${path}`
  ];
  if (data.code) lines.push(`Kod: ${data.code}`);
  if (data.detail && data.detail !== data.error) lines.push(`Neden: ${data.detail}`);
  if (data.suggestion) lines.push(`Çözüm: ${data.suggestion}`);
  return lines.filter(Boolean).join("\n");
}
