/**
 * developer.js — Worm Forensic Tool Developer Mode
 *
 * Ana logoya 5 kez tıklayınca açılan gizli geliştirici paneli.
 * Backend log akışı, sistem bilgisi, iş durumu ve tarayıcı console override'ı içerir.
 * Windows + Linux uyumlu olarak tasarlanmıştır.
 */

// ─── Constants ────────────────────────────────────────────────────────────────

const DEV_CLICK_TARGET   = 5;
const DEV_CLICK_TIMEOUT  = 3000; // ms — art arda 5 tıklama için pencere
const POLL_INTERVAL      = 1500; // ms — backend log polling
const MAX_FRONTEND_LOGS  = 3000;
const MAX_DISPLAY_LOGS   = 500;

// ─── State ────────────────────────────────────────────────────────────────────

let devOpen          = false;
let clickCount       = 0;
let clickTimer       = null;
let pollTimer        = null;
let lastLogSeq       = 0;
let allLogs          = [];       // { seq, timestamp, level, scope, message, source }
let filterLevel      = "all";
let filterText       = "";
let pinToBottom      = true;
let frontendLogBuf   = [];       // console override buffer

// ─── Public API ───────────────────────────────────────────────────────────────

/**
 * Developer modunu başlatır. Logoya tıklama sayacını ve console override'ı kurar.
 * @param {Object} opts
 * @param {Function} opts.apiRequest  — app.js'deki apiRequest fonksiyonu
 * @param {Function} opts.backendReady — backend durumu kontrolü
 */
export function initDeveloperMode({ apiRequest, backendReady }) {
  _installConsoleOverride(apiRequest, backendReady);
  _installBrandClickCounter(apiRequest, backendReady);
  _logBrowserEnv(apiRequest, backendReady);
}

/**
 * Bir frontend log satırı ekler ve backend'e gönderir.
 */
export function devLog(level, scope, message, apiRequest, backendReady) {
  const entry = _makeFrontendEntry(level, scope, message);
  _appendLog(entry);
  _sendToBackend(level, scope, message, apiRequest, backendReady);
  _refreshIfOpen();
}

// ─── Click Counter ────────────────────────────────────────────────────────────

function _installBrandClickCounter(apiRequest, backendReady) {
  // index.html'deki logo butonuna id="brand-logo" eklenmiş olmalı
  document.addEventListener("click", (e) => {
    const logo = e.target.closest("#brand-logo");
    if (!logo) return;

    clickCount++;
    if (clickTimer) clearTimeout(clickTimer);

    // Görsel geri bildirim — logo titreşimi
    logo.classList.add("dev-click-pulse");
    setTimeout(() => logo.classList.remove("dev-click-pulse"), 300);

    if (clickCount >= DEV_CLICK_TARGET) {
      clickCount = 0;
      clearTimeout(clickTimer);
      toggleDevPanel(apiRequest, backendReady);
      return;
    }

    // Kalan tıklama sayısını kısa süre göster
    _showClickHint(logo, DEV_CLICK_TARGET - clickCount);

    clickTimer = setTimeout(() => {
      clickCount = 0;
    }, DEV_CLICK_TIMEOUT);
  });
}

function _showClickHint(logo, remaining) {
  let hint = document.getElementById("dev-click-hint");
  if (!hint) {
    hint = document.createElement("div");
    hint.id = "dev-click-hint";
    hint.className = "dev-click-hint";
    document.body.appendChild(hint);
  }
  hint.textContent = `🐛 ${remaining}`;
  hint.classList.add("visible");
  clearTimeout(hint._timer);
  hint._timer = setTimeout(() => hint.classList.remove("visible"), 800);
}

// ─── Panel Toggle ─────────────────────────────────────────────────────────────

function toggleDevPanel(apiRequest, backendReady) {
  if (devOpen) {
    closeDevPanel();
  } else {
    openDevPanel(apiRequest, backendReady);
  }
}

function openDevPanel(apiRequest, backendReady) {
  devOpen = true;

  // Zaten varsa kaldır
  document.getElementById("dev-overlay")?.remove();

  const overlay = document.createElement("div");
  overlay.id = "dev-overlay";
  overlay.className = "dev-overlay";
  overlay.innerHTML = _buildPanelHtml();
  document.body.appendChild(overlay);

  // requestAnimationFrame ile open class ekle (CSS animasyon için)
  requestAnimationFrame(() => {
    requestAnimationFrame(() => overlay.classList.add("open"));
  });

  _bindPanelEvents(overlay, apiRequest, backendReady);
  _startPolling(apiRequest, backendReady);
  _refreshPanel();
}

function closeDevPanel() {
  devOpen = false;
  _stopPolling();

  const overlay = document.getElementById("dev-overlay");
  if (overlay) {
    overlay.classList.remove("open");
    overlay.addEventListener("transitionend", () => overlay.remove(), { once: true });
  }
}

// ─── HTML Builder ─────────────────────────────────────────────────────────────

function _buildPanelHtml() {
  return `
    <div class="dev-panel" role="dialog" aria-label="Developer Panel" id="dev-panel">
      <div class="dev-header">
        <div class="dev-title-row">
          <span class="dev-badge">🐛 DEV</span>
          <span class="dev-title">Worm Developer Console</span>
          <span class="dev-subtitle" id="dev-log-count">— loglar yükleniyor</span>
        </div>
        <div class="dev-header-actions">
          <button class="dev-btn dev-btn-sm" id="dev-clear-btn" title="Logları temizle">🗑 Temizle</button>
          <button class="dev-btn dev-btn-sm" id="dev-export-btn" title="Logları dışa aktar">💾 Dışa Aktar</button>
          <button class="dev-btn dev-btn-sm" id="dev-copy-btn" title="Panoya kopyala">📋 Kopyala</button>
          <button class="dev-btn dev-btn-sm dev-btn-danger" id="dev-close-btn" title="Kapat">✕</button>
        </div>
      </div>

      <div class="dev-toolbar">
        <div class="dev-filter-row">
          <label class="dev-label">Seviye:</label>
          <select class="dev-select" id="dev-filter-level">
            <option value="all">Tümü</option>
            <option value="error">ERROR</option>
            <option value="warn">WARN</option>
            <option value="info">INFO</option>
            <option value="debug">DEBUG</option>
            <option value="ui">UI</option>
          </select>
          <label class="dev-label">Ara:</label>
          <input class="dev-input" id="dev-filter-text" type="text" placeholder="scope / mesaj..." />
          <label class="dev-label dev-pin-label">
            <input type="checkbox" id="dev-pin" ${pinToBottom ? "checked" : ""} />
            Aşağıya pin
          </label>
        </div>
      </div>

      <div class="dev-tabs">
        <button class="dev-tab active" data-dev-tab="logs">📋 Loglar</button>
        <button class="dev-tab" data-dev-tab="system">🖥 Sistem</button>
        <button class="dev-tab" data-dev-tab="jobs">⚙ İşler</button>
      </div>

      <div class="dev-body">
        <div class="dev-tab-content active" id="dev-tab-logs">
          <div class="dev-log-area" id="dev-log-area"></div>
        </div>
        <div class="dev-tab-content" id="dev-tab-system">
          <div class="dev-info-area" id="dev-system-area">
            <span class="dev-loading">Sistem bilgisi yükleniyor...</span>
          </div>
        </div>
        <div class="dev-tab-content" id="dev-tab-jobs">
          <div class="dev-info-area" id="dev-jobs-area">
            <span class="dev-loading">İş bilgisi yükleniyor...</span>
          </div>
        </div>
      </div>

      <div class="dev-statusbar">
        <span id="dev-status-left">Toplam: <b id="dev-total-count">0</b> satır</span>
        <span id="dev-status-right">Son güncelleme: <b id="dev-last-update">—</b></span>
      </div>
    </div>
  `;
}

// ─── Event Bindings ───────────────────────────────────────────────────────────

function _bindPanelEvents(overlay, apiRequest, backendReady) {
  // Kapatma
  document.getElementById("dev-close-btn")?.addEventListener("click", closeDevPanel);
  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) closeDevPanel();
  });

  // Klavye ESC
  const keyHandler = (e) => {
    if (e.key === "Escape" && devOpen) {
      closeDevPanel();
      document.removeEventListener("keydown", keyHandler);
    }
  };
  document.addEventListener("keydown", keyHandler);

  // Filtreler
  document.getElementById("dev-filter-level")?.addEventListener("change", (e) => {
    filterLevel = e.target.value;
    _refreshPanel();
  });

  document.getElementById("dev-filter-text")?.addEventListener("input", (e) => {
    filterText = e.target.value.toLowerCase();
    _refreshPanel();
  });

  document.getElementById("dev-pin")?.addEventListener("change", (e) => {
    pinToBottom = e.target.checked;
    if (pinToBottom) _scrollToBottom();
  });

  // Temizle
  document.getElementById("dev-clear-btn")?.addEventListener("click", () => {
    allLogs = [];
    lastLogSeq = 0;
    frontendLogBuf = [];
    _refreshPanel();
  });

  // Dışa aktar
  document.getElementById("dev-export-btn")?.addEventListener("click", () => {
    _exportLogs();
  });

  // Panoya kopyala
  document.getElementById("dev-copy-btn")?.addEventListener("click", () => {
    const visible = _filteredLogs();
    const text = visible.map(_formatLogLine).join("\n");
    navigator.clipboard?.writeText(text).then(() => {
      const btn = document.getElementById("dev-copy-btn");
      if (btn) { btn.textContent = "✓ Kopyalandı!"; setTimeout(() => { btn.textContent = "📋 Kopyala"; }, 1500); }
    }).catch(() => {});
  });

  // Sekmeler
  document.querySelectorAll("[data-dev-tab]").forEach(tab => {
    tab.addEventListener("click", () => {
      document.querySelectorAll(".dev-tab").forEach(t => t.classList.remove("active"));
      document.querySelectorAll(".dev-tab-content").forEach(c => c.classList.remove("active"));
      tab.classList.add("active");
      document.getElementById(`dev-tab-${tab.dataset.devTab}`)?.classList.add("active");
    });
  });
}

// ─── Polling ──────────────────────────────────────────────────────────────────

function _startPolling(apiRequest, backendReady) {
  _stopPolling();
  _fetchLogs(apiRequest, backendReady);
  pollTimer = setInterval(() => _fetchLogs(apiRequest, backendReady), POLL_INTERVAL);
}

function _stopPolling() {
  if (pollTimer) { clearInterval(pollTimer); pollTimer = null; }
}

async function _fetchLogs(apiRequest, backendReady) {
  if (!devOpen) return;

  try {
    if (backendReady()) {
      const data = await apiRequest("/api/developer-logs");

      // Backend logları ekle (seq ile dedupe)
      if (Array.isArray(data.logs)) {
        const newEntries = data.logs
          .filter(e => e.seq > lastLogSeq)
          .map(e => ({
            seq: e.seq,
            timestamp: e.timestamp,
            level: (e.level || "INFO").toUpperCase(),
            scope: e.scope || "backend",
            message: e.message,
            thread: e.thread || "",
            source: "backend",
          }));

        if (newEntries.length > 0) {
          lastLogSeq = newEntries[newEntries.length - 1].seq;
          allLogs.push(...newEntries);
          _trimLogs();
        }
      }

      // Sistem bilgisi güncelle
      if (data.system) {
        _renderSystemInfo(data.system);
      }

      // İş bilgisi güncelle
      if (data.jobs) {
        _renderJobs(data.jobs);
      }

      _updateStatusBar(data.logs?.length);
    }
    _refreshPanel();
  } catch (err) {
    const errEntry = _makeFrontendEntry("ERROR", "dev-poll", `Backend log fetch hatası: ${err.message}`);
    _appendLog(errEntry);
    _refreshPanel();
  }
}

// ─── Log Rendering ────────────────────────────────────────────────────────────

function _refreshPanel() {
  if (!devOpen) return;
  const area = document.getElementById("dev-log-area");
  if (!area) return;

  const visible = _filteredLogs().slice(-MAX_DISPLAY_LOGS);

  if (visible.length === 0) {
    area.innerHTML = `<div class="dev-empty">Gösterilecek log yok. Filtre ayarlarını kontrol edin.</div>`;
    return;
  }

  const wasAtBottom = area.scrollHeight - area.scrollTop - area.clientHeight < 40;

  const fragment = document.createDocumentFragment();
  visible.forEach(entry => {
    const row = document.createElement("div");
    row.className = `dev-log-row dev-level-${(entry.level || "info").toLowerCase()}`;
    row.innerHTML = `
      <span class="dev-log-ts">${_escHtml(entry.timestamp?.slice(11, 23) || "")}</span>
      <span class="dev-log-level">${_escHtml(entry.level || "")}</span>
      <span class="dev-log-src ${entry.source === "ui" ? "dev-src-ui" : ""}">${_escHtml(entry.scope || "")}</span>
      <span class="dev-log-msg">${_escHtml(entry.message || "")}</span>
      ${entry.thread ? `<span class="dev-log-thread">${_escHtml(entry.thread)}</span>` : ""}
    `;
    fragment.appendChild(row);
  });

  area.innerHTML = "";
  area.appendChild(fragment);

  const countEl = document.getElementById("dev-log-count");
  if (countEl) countEl.textContent = `— ${visible.length} / ${allLogs.length} satır`;

  const totalEl = document.getElementById("dev-total-count");
  if (totalEl) totalEl.textContent = allLogs.length;

  if (pinToBottom && (wasAtBottom || visible.length < 30)) {
    _scrollToBottom();
  }
}

function _refreshIfOpen() {
  if (devOpen) _refreshPanel();
}

function _scrollToBottom() {
  const area = document.getElementById("dev-log-area");
  if (area) area.scrollTop = area.scrollHeight;
}

function _filteredLogs() {
  return allLogs.filter(entry => {
    if (filterLevel !== "all") {
      const lvl = (entry.level || "info").toLowerCase();
      const src = (entry.source || "");
      if (filterLevel === "ui" && src !== "ui") return false;
      if (filterLevel !== "ui" && lvl !== filterLevel) return false;
    }
    if (filterText) {
      const haystack = `${entry.scope} ${entry.message}`.toLowerCase();
      if (!haystack.includes(filterText)) return false;
    }
    return true;
  });
}

function _formatLogLine(entry) {
  const ts   = entry.timestamp || "";
  const lvl  = (entry.level || "").padEnd(5);
  const src  = (entry.scope || "").padEnd(20);
  const thr  = entry.thread ? `[${entry.thread}]` : "";
  return `${ts} | ${lvl} | ${src} | ${thr} ${entry.message}`;
}

// ─── System Info ──────────────────────────────────────────────────────────────

function _renderSystemInfo(system) {
  const area = document.getElementById("dev-system-area");
  if (!area) return;

  const rows = [
    ["Uygulama Versiyonu", system.version],
    ["İşletim Sistemi", `${system.os} / ${system.family} / ${system.arch}`],
    ["PID", system.pid],
    ["Yürütülebilir", system.exe],
    ["Çalışma Dizini", system.cwd],
    ["UI Kök", system.ui_root],
    ["Yükseltilmiş (root)", system.is_elevated ? "✅ Evet" : "❌ Hayır"],
    ["Log Dosyası", system.runtime_log_file || "—"],
    ["User Agent", navigator.userAgent],
    ["Platform (JS)", navigator.platform || "—"],
    ["Ekran Çözünürlüğü", `${screen.width}×${screen.height} (${window.devicePixelRatio}x)`],
    ["Pencere Boyutu", `${window.innerWidth}×${window.innerHeight}`],
    ["Bellek (JS heap)", _jsHeapInfo()],
    ["Dil", navigator.language],
    ["Online", navigator.onLine ? "✅" : "❌"],
  ];

  const envRows = Array.isArray(system.env)
    ? system.env.map(e => [e.key, e.value || "(yok)"])
    : [];

  area.innerHTML = `
    <table class="dev-table">
      <thead><tr><th>Alan</th><th>Değer</th></tr></thead>
      <tbody>
        ${rows.map(([k, v]) => `<tr><td class="dev-td-key">${_escHtml(k)}</td><td class="dev-td-val">${_escHtml(String(v ?? "—"))}</td></tr>`).join("")}
      </tbody>
    </table>
    ${envRows.length > 0 ? `
      <div class="dev-section-title" style="margin-top:12px">Ortam Değişkenleri</div>
      <table class="dev-table">
        <thead><tr><th>Değişken</th><th>Değer</th></tr></thead>
        <tbody>
          ${envRows.map(([k, v]) => `<tr><td class="dev-td-key">${_escHtml(k)}</td><td class="dev-td-val dev-env-val">${_escHtml(String(v))}</td></tr>`).join("")}
        </tbody>
      </table>
    ` : ""}
  `;
}

function _jsHeapInfo() {
  try {
    const mem = typeof performance !== "undefined" ? performance?.memory : null;
    if (!mem) return "—";
    const mb = (bytes) => `${(bytes / 1024 / 1024).toFixed(1)} MB`;
    return `kullanılan ${mb(mem.usedJSHeapSize)} / toplam ${mb(mem.totalJSHeapSize)} / limit ${mb(mem.jsHeapSizeLimit)}`;
  } catch {
    return "—";
  }
}

// ─── Jobs ─────────────────────────────────────────────────────────────────────

function _renderJobs(jobs) {
  const area = document.getElementById("dev-jobs-area");
  if (!area) return;

  if (!Array.isArray(jobs) || jobs.length === 0) {
    area.innerHTML = `<div class="dev-empty">Aktif veya son iş yok.</div>`;
    return;
  }

  area.innerHTML = `
    <table class="dev-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Durum</th>
          <th>İlerleme</th>
          <th>Mesaj</th>
          <th>Hata</th>
          <th>Log</th>
        </tr>
      </thead>
      <tbody>
        ${jobs.map(job => `
          <tr>
            <td class="dev-td-key dev-monospace">${_escHtml(String(job.id || "").slice(0, 12))}…</td>
            <td><span class="dev-status-pill dev-status-${(job.status || "").toLowerCase()}">${_escHtml(job.status || "?")}</span></td>
            <td>${job.total > 0 ? `${job.done}/${job.total} (${Math.round((job.done/job.total)*100)}%)` : job.done > 0 ? `${job.done}` : "—"}</td>
            <td>${_escHtml(job.message || "")}</td>
            <td>${job.error ? `<span class="dev-error-text">${_escHtml(job.error)}</span>` : "—"}</td>
            <td>${job.log_count ?? "—"}</td>
          </tr>
        `).join("")}
      </tbody>
    </table>
  `;
}

// ─── Status Bar ───────────────────────────────────────────────────────────────

function _updateStatusBar(backendCount) {
  const el = document.getElementById("dev-last-update");
  if (el) el.textContent = new Date().toLocaleTimeString();
}

// ─── Console Override ─────────────────────────────────────────────────────────

function _installConsoleOverride(apiRequest, backendReady) {
  const levels = { error: "ERROR", warn: "WARN", log: "INFO", debug: "DEBUG", info: "INFO" };

  Object.entries(levels).forEach(([method, level]) => {
    const original = console[method].bind(console);
    console[method] = (...args) => {
      original(...args);
      try {
        const message = args.map(a => {
          if (a instanceof Error) return `${a.name}: ${a.message}\n${a.stack || ""}`;
          if (typeof a === "object") {
            try { return JSON.stringify(a); } catch { return String(a); }
          }
          return String(a);
        }).join(" ");

        const entry = _makeFrontendEntry(level, `console.${method}`, message);
        _appendLog(entry);

        // ERROR ve WARN'ları backend'e de gönder
        if (level === "ERROR" || level === "WARN") {
          _sendToBackend(level, `ui:console.${method}`, message, apiRequest, backendReady);
        }

        _refreshIfOpen();
      } catch {
        // console override içinde hata olursa sessizce devam et
      }
    };
  });

  // Yakalanmayan hatalar
  window.addEventListener("error", (e) => {
    const message = `Yakalanmayan hata: ${e.message} @ ${e.filename}:${e.lineno}:${e.colno}`;
    const entry = _makeFrontendEntry("ERROR", "window.onerror", message);
    _appendLog(entry);
    _sendToBackend("ERROR", "ui:window.onerror", message, apiRequest, backendReady);
    _refreshIfOpen();
  });

  window.addEventListener("unhandledrejection", (e) => {
    const reason = e.reason instanceof Error
      ? `${e.reason.name}: ${e.reason.message}`
      : String(e.reason);
    const message = `Yakalanmayan Promise reddi: ${reason}`;
    const entry = _makeFrontendEntry("ERROR", "window.unhandledrejection", message);
    _appendLog(entry);
    _sendToBackend("ERROR", "ui:unhandledrejection", message, apiRequest, backendReady);
    _refreshIfOpen();
  });
}

// ─── Browser Environment Dump ─────────────────────────────────────────────────

function _logBrowserEnv(apiRequest, backendReady) {
  const safeNav = typeof navigator !== "undefined" ? navigator : {};
  const safeScreen = typeof screen !== "undefined" ? screen : {};
  const safeLocation = typeof location !== "undefined" ? location : { protocol: "?", origin: "?", search: "" };

  const entries = [
    `User-Agent: ${safeNav.userAgent || "unknown"}`,
    `Platform (JS): ${safeNav.platform || "unknown"}`,
    `Language: ${safeNav.language || "unknown"}`,
    `Ekran: ${safeScreen.width || "?"}×${safeScreen.height || "?"} @ ${typeof window !== "undefined" ? window.devicePixelRatio || 1 : 1}x`,
    `Pencere: ${typeof window !== "undefined" ? window.innerWidth || "?" : "?"}×${typeof window !== "undefined" ? window.innerHeight || "?" : "?"}`,
    `Protokol: ${safeLocation.protocol}`,
    `Origin: ${safeLocation.origin || "?"}`,
    `Online: ${safeNav.onLine ?? "unknown"}`,
    `Cookie enabled: ${safeNav.cookieEnabled ?? "unknown"}`,
    `URL Params: ${safeLocation.search || ""}`,
  ];

  entries.forEach(msg => {
    const entry = _makeFrontendEntry("INFO", "ui:env", msg);
    _appendLog(entry);
  });

  _sendToBackend("INFO", "ui:startup", entries.join(" | "), apiRequest, backendReady);
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

let _uiFrontendSeq = 100000; // UI logları backend seq ile çakışmasın diye büyük sayıdan başla

function _makeFrontendEntry(level, scope, message) {
  return {
    seq: _uiFrontendSeq++,
    timestamp: new Date().toISOString().replace("T", " ").slice(0, 23),
    level: level.toUpperCase(),
    scope,
    message,
    source: "ui",
    thread: "",
  };
}

function _appendLog(entry) {
  allLogs.push(entry);
  frontendLogBuf.push(entry);
  _trimLogs();
}

function _trimLogs() {
  if (allLogs.length > MAX_FRONTEND_LOGS) {
    allLogs = allLogs.slice(allLogs.length - MAX_FRONTEND_LOGS);
  }
}

async function _sendToBackend(level, scope, message, apiRequest, backendReady) {
  if (!backendReady || !backendReady()) return;
  try {
    await apiRequest("/api/developer-log", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ level, scope, message }),
    });
  } catch {
    // Sessizce yut — log gönderimi kritik değil
  }
}

function _exportLogs() {
  const lines = allLogs.map(_formatLogLine).join("\n");
  const blob = new Blob([lines], { type: "text/plain; charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `worm-dev-log-${new Date().toISOString().slice(0, 19).replace(/[T:]/g, "-")}.txt`;
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 5000);
}

function _escHtml(str) {
  return String(str ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
