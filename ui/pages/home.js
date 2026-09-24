// Ana Dashboard: Yapay Zeka Adli Ajanı, Yatay Haber Menüsü ve Adli Araçlar.

import { escapeHtml } from "../core/utils.js";

export function homePage({ t, icon, assetPath, theme, state }) {
  const isEn = state?.language === "en";

  // 1. Karşılama başlığı
  const hour = new Date().getHours();
  const greetingKey = hour < 12 ? "copilot.greeting.morning" : hour < 18 ? "copilot.greeting.afternoon" : "copilot.greeting.evening";
  const userName = state?.activeProfile?.fullName || state?.activeProfile?.username || "melihemik";
  const greetingText = t(greetingKey, { name: userName });

  // 2. Yapay Zeka Ajan durumu
  const agentState = state?.agent || state?.copilot || {};
  const agents = agentState.agents || [];
  const selectedAgentId = agentState.selectedAgent || "agy";
  const selectedAgent = agents.find((a) => a.id === selectedAgentId) || agents[0];
  const models = selectedAgent?.models || [];
  const selectedModelId = agentState.selectedModel || (models[0]?.id || "");
  const selectedScope = agentState.selectedScope || "all";
  const isGenerating = Boolean(agentState.isGenerating);
  const messages = agentState.messages || [];

  // Ajan seçenekleri HTML
  const agentOptionsHtml = agents.length > 0
    ? agents.map((a) => {
        const isSel = a.id === selectedAgentId;
        const iconPrefix = a.id === "pi" ? "🥧" : a.id === "agy" ? "🌌" : a.id === "claude" ? "🎭" : a.id === "codex" ? "🧠" : a.id === "opencode" ? "🌐" : "🛡️";
        const statusSuffix = a.installed ? "" : " (Kurulu Değil)";
        return `<option value="${a.id}" ${isSel ? "selected" : ""}>${iconPrefix} ${escapeHtml(a.name)}${statusSuffix}</option>`;
      }).join("")
    : `
        <option value="agy">🌌 Antigravity (AGY)</option>
        <option value="pi">🥧 Pi Coding Agent</option>
        <option value="claude">🎭 Claude Code</option>
        <option value="codex">🧠 Codex / OpenAI</option>
        <option value="opencode">🌐 OpenCode</option>
        <option value="amele-expert" selected>🛡️ Amele Adli Uzman</option>
      `;

  // Model seçenekleri HTML
  const modelOptionsHtml = models.length > 0
    ? models.map((m) => {
        const isSel = m.id === selectedModelId;
        return `<option value="${m.id}" ${isSel ? "selected" : ""} title="${escapeHtml(m.description)}">${escapeHtml(m.name)}</option>`;
      }).join("")
    : `<option value="default">${t("copilot.selectModel")}</option>`;

  // Sohbet geçmişi HTML
  const chatHistoryHtml = messages.length > 0 || isGenerating
    ? `
      <div class="agent-chat-history">
        ${messages.map((msg) => renderChatMessage(msg, state, t, escapeHtml)).join("")}
        ${
          isGenerating
            ? `
              <div class="agent-msg assistant">
                <div class="agent-msg-header">
                  <span class="agent-msg-author">🤖 ${selectedAgent?.name || "Agent"}</span>
                  <span class="agent-badge badge-user">${t("copilot.generating")}</span>
                </div>
                <div class="agent-msg-body">
                  <div class="loading-dots"><span>.</span><span>.</span><span>.</span></div>
                </div>
              </div>
            `
            : ""
        }
      </div>
    `
    : "";

  // 3. Yatay Haber Menüsü (Fotoğrafsız)
  const newsList = Array.isArray(state?.news) && state.news.length > 0
    ? state.news.slice(0, 8)
    : [
        {
          id: "default-news-1",
          titleTr: "Amele v0.0.20 Adli Bilişim Güncellemesi Yayınlandı",
          titleEn: "Amele v0.0.20 Digital Forensics Release",
          summaryTr: "Konteyner adli bilişimi, bellek ve disk edinim motoru, bağımsız 120 FPS Chromium motoru sisteme entegre edildi.",
          summaryEn: "Container forensics, memory and disk acquisition engine, and standalone 120 FPS Chromium engine integrated.",
          createdBy: "melihemik",
          createdAt: new Date().toISOString(),
          link: "https://amele.noirlang.tr"
        }
      ];

  const newsCardsHtml = newsList.map((item) => {
    const title = isEn
      ? (item.titleEn || item.title || item.titleTr || "")
      : (item.titleTr || item.title || item.titleEn || "");
    const summary = isEn
      ? (item.summaryEn || item.summary || item.summaryTr || (item.contentEn || item.content ? (item.contentEn || item.content).slice(0, 160) + "..." : ""))
      : (item.summaryTr || item.summary || item.summaryEn || (item.contentTr || item.content ? (item.contentTr || item.content).slice(0, 160) + "..." : ""));
    const author = item.createdBy || item.author || "NOIRLANG Ekibi";
    const dateFormatted = formatNewsDate(item.createdAt || item.date, isEn);

    let targetUrl = item.link || item.url || (item.slug ? `https://amele.noirlang.tr/news/${encodeURIComponent(item.slug)}` : "https://amele.noirlang.tr");
    if (targetUrl.startsWith("/")) targetUrl = `https://amele.noirlang.tr${targetUrl}`;

    return `
      <div class="news-h-card" data-news-link="${targetUrl}">
        <div class="news-h-card-top">
          <span class="news-h-author">✍️ @${escapeHtml(author)}</span>
          <span class="news-h-datetime">🕒 ${dateFormatted}</span>
        </div>
        <h4 class="news-h-title">${escapeHtml(title)}</h4>
        <p class="news-h-subtitle">${escapeHtml(summary)}</p>
        <div class="news-h-card-footer">
          <span class="news-h-readmore">${t("news.readMore")} →</span>
        </div>
      </div>
    `;
  }).join("");

  // 4. Yetki Yükseltme Modalı (Linux Sudo veya Windows Admin)
  const elevationModal = agentState.elevationModal;
  const elevationModalHtml = elevationModal && elevationModal.isOpen
    ? renderElevationModal(elevationModal, t, escapeHtml)
    : "";

  return `
    <section class="page">
      <!-- Yapay Zeka Ajan Bölümü -->
      <div class="agent-container">
        <!-- Karşılama Çubuğu -->
        <div class="agent-greeting-bar">
          <div class="agent-greeting-left">
            <div class="agent-mascot">
              <svg viewBox="0 0 24 24" width="22" height="22" fill="currentColor">
                <path d="M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2M7.5 13A2.5 2.5 0 0 0 5 15.5 2.5 2.5 0 0 0 7.5 18a2.5 2.5 0 0 0 2.5-2.5A2.5 2.5 0 0 0 7.5 13m9 0a2.5 2.5 0 0 0-2.5 2.5 2.5 2.5 0 0 0 2.5 2.5 2.5 2.5 0 0 0 2.5-2.5 2.5 2.5 0 0 0-2.5-2.5"/>
              </svg>
            </div>
            <h1 class="agent-greeting-title">${escapeHtml(greetingText)}</h1>
          </div>
          <div class="agent-greeting-right">
            <span class="agent-badge-preview">Preview</span>
            <span class="agent-separator">·</span>
            <a href="https://github.com/noirlang/amele/issues" target="_blank" rel="noopener noreferrer" class="agent-feedback-link">
              ${t("copilot.giveFeedback")}
            </a>
          </div>
        </div>

        <!-- Ajan Giriş Kutusu -->
        <div class="agent-box">
          <div class="agent-input-wrapper">
            <textarea
              id="agent-prompt-input"
              class="agent-textarea"
              placeholder="${t("copilot.placeholder")}"
              rows="2"
              data-agent-input
            >${escapeHtml(agentState.promptDraft || "")}</textarea>
          </div>

          <div class="agent-toolbar">
            <div class="agent-toolbar-left">
              <!-- Sor butonu -->
              <div class="agent-btn-dropdown">
                <button type="button" class="agent-tool-btn" data-agent-action="toggle-mode">
                  <span>💬 ${t("copilot.modeAsk")}</span>
                  <span class="dropdown-caret">▾</span>
                </button>
              </div>

              <!-- Kapsam -->
              <div class="agent-btn-dropdown">
                <select class="agent-select" data-agent-action="change-scope" title="${t("copilot.scopeTitle")}">
                  <option value="all" ${selectedScope === "all" ? "selected" : ""}>🔖 ${t("copilot.scopeAll")}</option>
                  <option value="ram" ${selectedScope === "ram" ? "selected" : ""}>🧠 ${t("copilot.scopeRam")}</option>
                  <option value="disk" ${selectedScope === "disk" ? "selected" : ""}>🛠️ ${t("copilot.scopeDisk")}</option>
                  <option value="case" ${selectedScope === "case" ? "selected" : ""}>📋 ${t("copilot.scopeCase")}</option>
                  <option value="docker" ${selectedScope === "docker" ? "selected" : ""}>🐳 ${t("copilot.scopeDocker")}</option>
                  <option value="android" ${selectedScope === "android" ? "selected" : ""}>📱 ${t("copilot.scopeAndroid")}</option>
                </select>
              </div>

              <!-- [+] Bağlam Ekleme Butonu -->
              <button type="button" class="agent-icon-btn" data-agent-action="add-context" title="${t("copilot.addContext")}">
                +
              </button>

              <!-- Ajan Seçici Dropdown -->
              <div class="agent-btn-dropdown">
                <select class="agent-select agent-select-highlight" data-agent-action="change-agent" title="${t("copilot.selectAgent")}">
                  ${agentOptionsHtml}
                </select>
              </div>

              <!-- Model Seçici Dropdown (Ajan seçildiğinde dinamik güncellenir) -->
              <div class="agent-btn-dropdown">
                <select class="agent-select agent-model-select" data-agent-action="change-model" title="${t("copilot.selectModel")}">
                  ${modelOptionsHtml}
                </select>
              </div>
            </div>

            <div class="agent-toolbar-right">
              <!-- Auto Dropdown -->
              <div class="agent-btn-dropdown">
                <select class="agent-select agent-pill-select" data-agent-action="change-auto">
                  <option value="auto">🔀 ${t("copilot.auto")}</option>
                  <option value="manual">⚙️ ${t("copilot.manual")}</option>
                </select>
              </div>

              <!-- Optimized for: Balance Dropdown -->
              <div class="agent-btn-dropdown">
                <select class="agent-select agent-pill-select" data-agent-action="change-opt">
                  <option value="balance" ${agentState.selectedOpt === "balance" ? "selected" : ""}>${t("copilot.optBalance")}</option>
                  <option value="speed" ${agentState.selectedOpt === "speed" ? "selected" : ""}>${t("copilot.optSpeed")}</option>
                  <option value="reasoning" ${agentState.selectedOpt === "reasoning" ? "selected" : ""}>${t("copilot.optReasoning")}</option>
                </select>
              </div>

              <!-- Hız / Optimizasyon İkonu -->
              <button type="button" class="agent-icon-btn" data-agent-action="toggle-opt" title="${t("copilot.sparkleTitle")}">
                ✨
              </button>

              <span class="agent-v-divider"></span>

              <!-- Gönder Butonu -->
              <button
                type="button"
                class="agent-send-btn"
                data-agent-action="send"
                title="${t("copilot.send")}"
                ${isGenerating ? "disabled" : ""}
              >
                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                  <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>

        <!-- Hızlı Eylem Çipleri (Action Chips) -->
        <div class="agent-chips-container">
          <div class="agent-chips-row">
            <button type="button" class="agent-chip" data-agent-chip="debug">
              <span class="chip-icon">🐞</span>
              <span>${t("copilot.chipDebug")}</span>
            </button>
            <button type="button" class="agent-chip" data-agent-chip="agent">
              <span class="chip-icon">☁️</span>
              <span>${t("copilot.chipAgent")}</span>
            </button>
            <button type="button" class="agent-chip" data-agent-chip="issue">
              <span class="chip-icon">◌</span>
              <span>${t("copilot.chipIssue")}</span>
            </button>
            <button type="button" class="agent-chip" data-agent-chip="code">
              <span class="chip-icon">📄</span>
              <span>${t("copilot.chipCode")}</span>
              <span class="dropdown-caret">▾</span>
            </button>
            <button type="button" class="agent-chip" data-agent-chip="git">
              <span class="chip-icon">⑂</span>
              <span>${t("copilot.chipGit")}</span>
              <span class="dropdown-caret">▾</span>
            </button>
          </div>
          <div class="agent-chips-row">
            <button type="button" class="agent-chip" data-agent-chip="pr">
              <span class="chip-icon">⇅</span>
              <span>${t("copilot.chipPr")}</span>
              <span class="dropdown-caret">▾</span>
            </button>
          </div>
        </div>

        <!-- Sohbet Geçmişi & Komut Çalıştırma Sonuçları -->
        ${chatHistoryHtml}
      </div>

      <!-- Yatay Haber Menüsü (Fotoğrafsız) -->
      <div class="home-news-horizontal-section">
        <div class="news-horizontal-header">
          <div class="news-horizontal-title">
            <span class="news-header-icon">📢</span>
            <h3>${t("news.latestUpdates")}</h3>
          </div>
          <div class="news-horizontal-nav">
            <button type="button" class="news-h-btn" data-news-h-scroll="left" aria-label="Geri">‹</button>
            <button type="button" class="news-h-btn" data-news-h-scroll="right" aria-label="İleri">›</button>
          </div>
        </div>

        <div class="news-horizontal-scroll-container" id="news-horizontal-track">
          ${newsCardsHtml}
        </div>
      </div>

      <!-- Standart Adli Araç Kartları Grid -->
      <div class="home-grid">
        ${homeTile(t("home.windows.title"), t("home.windows.desc"), "windows", "windows", "var(--text)", icon, state)}
        ${homeTile(t("home.linux.title"), t("home.linux.desc"), "linux", "linux", "var(--text)", icon, state)}
        ${homeTile(t("home.docker.title"), t("home.docker.desc"), "docker", "docker", "var(--text)", icon, state)}
        ${homeTile(t("home.android.title"), t("home.android.desc"), "android", "android", "var(--text)", icon, state)}
        ${homeTile(t("home.ios.title"), t("home.ios.desc"), "ios", "ios", "var(--text)", icon, state)}
        ${homeTile(t("home.remote.title"), t("home.remote.desc"), "key", "remote-acq", "var(--text)", icon, state)}
        ${homeTile(t("home.other.title"), t("home.other.desc"), "tiles", "other", "var(--text)", icon, state)}
      </div>

      <!-- Yetki Yükseltme Modalı -->
      ${elevationModalHtml}
    </section>
  `;
}

function renderChatMessage(msg, state, t, escapeHtml) {
  const isUser = msg.role === "user";
  const authorName = isUser
    ? (state?.activeProfile?.fullName || state?.activeProfile?.username || "melihemik")
    : (msg.agent ? `${msg.agent.toUpperCase()} (${msg.model || "Expert"})` : "Amele Ajanı");

  const formattedContent = formatSimpleMarkdown(msg.content);

  // Önerilen CLI komutu varsa kart olarak çiz
  const commandCardHtml = msg.suggested_command
    ? renderCommandCard(msg.suggested_command, msg.id, msg.execResult, state, t, escapeHtml)
    : "";

  return `
    <div class="agent-msg ${isUser ? "user" : "assistant"}" data-msg-id="${msg.id}">
      <div class="agent-msg-header">
        <span class="agent-msg-author">${isUser ? "👤" : "🤖"} ${escapeHtml(authorName)}</span>
        <span class="agent-msg-time">${new Date(msg.timestamp || Date.now()).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</span>
      </div>
      <div class="agent-msg-body">
        ${formattedContent}
      </div>
      ${commandCardHtml}
    </div>
  `;
}

function renderCommandCard(cmd, messageId, execResult, state, t, escapeHtml) {
  const isLinux = state?.platform === "linux";
  const isRootReq = cmd.includes("sudo") || cmd.includes("ram") || cmd.includes("disk") || cmd.includes("/dev/");

  const badgeClass = isRootReq ? "badge-root" : "badge-user";
  const badgeLabel = isRootReq
    ? (isLinux ? t("copilot.badgeRoot") : t("copilot.badgeAdmin"))
    : t("copilot.badgeUser");

  let execHtml = "";
  if (execResult) {
    const isSuccess = execResult.ok;
    const outputText = (execResult.stdout || "") + (execResult.stderr ? `\n[STDERR]\n${execResult.stderr}` : "");
    execHtml = `
      <div class="agent-exec-output ${isSuccess ? "success" : "error"}">
        <strong>${isSuccess ? t("copilot.exitSuccess") : t("copilot.exitFailed", { code: execResult.exit_code })}</strong>
        <div>${escapeHtml(outputText.trim() || "(Çıktı yok)")}</div>
      </div>
    `;
  }

  return `
    <div class="agent-command-card">
      <div class="agent-command-header">
        <span class="agent-badge ${badgeClass}">🔒 ${badgeLabel}</span>
        <button type="button" class="agent-copy-btn" data-agent-action="copy-cmd" data-cmd="${escapeHtml(cmd)}">
          📋 ${t("copilot.copy")}
        </button>
      </div>
      <pre class="agent-command-code"><code>${escapeHtml(cmd)}</code></pre>
      <div class="agent-command-actions">
        <button
          type="button"
          class="agent-run-btn"
          data-agent-action="run-cmd"
          data-cmd="${escapeHtml(cmd)}"
          data-msg-id="${messageId}"
        >
          ▶ ${t("copilot.runCommand")}
        </button>
      </div>
      ${execHtml}
    </div>
  `;
}

function renderElevationModal(modal, t, escapeHtml) {
  const isLinux = modal.os === "linux";

  if (isLinux) {
    return `
      <div class="elevation-modal-overlay" id="sudo-elevation-modal">
        <div class="elevation-modal-card">
          <div class="elevation-modal-icon linux">🔒</div>
          <h3 class="elevation-modal-title">${t("elevation.linuxTitle")}</h3>
          <p class="elevation-modal-desc">${escapeHtml(modal.reason || t("elevation.linuxDesc"))}</p>
          <div class="elevation-modal-cmd">
            <code>${escapeHtml(modal.command)}</code>
          </div>
          <form class="elevation-modal-form" data-elevation-form="linux" data-cmd="${escapeHtml(modal.command)}" data-msg-id="${modal.messageId || ""}">
            <label for="sudo-pass-input">${t("elevation.passwordLabel")}</label>
            <input
              type="password"
              id="sudo-pass-input"
              placeholder="${t("elevation.passwordPlaceholder")}"
              required
              autofocus
            />
            <div class="elevation-modal-actions">
              <button type="button" class="btn btn-secondary" data-elevation-action="cancel">
                ${t("common.cancel") || "İptal"}
              </button>
              <button type="submit" class="btn btn-danger">
                ${t("elevation.authenticateAndRun")}
              </button>
            </div>
          </form>
        </div>
      </div>
    `;
  }

  // Windows Yönetici İzni Modalı
  return `
    <div class="elevation-modal-overlay" id="windows-elevation-modal">
      <div class="elevation-modal-card">
        <div class="elevation-modal-icon windows">🛡️</div>
        <h3 class="elevation-modal-title">${t("elevation.windowsTitle")}</h3>
        <p class="elevation-modal-desc">${escapeHtml(modal.reason || t("elevation.windowsDesc"))}</p>
        <div class="elevation-modal-cmd">
          <code>${escapeHtml(modal.command)}</code>
        </div>
        <div class="elevation-modal-actions">
          <button type="button" class="btn btn-secondary" data-elevation-action="cancel">
            ${t("elevation.no")}
          </button>
          <button
            type="button"
            class="btn btn-primary"
            data-elevation-action="confirm-windows"
            data-cmd="${escapeHtml(modal.command)}"
            data-msg-id="${modal.messageId || ""}"
          >
            ${t("elevation.yesRunAdmin")}
          </button>
        </div>
      </div>
    </div>
  `;
}

function formatSimpleMarkdown(text) {
  if (!text) return "";

  // 1. Kod blokları
  let escaped = text.replace(/```([a-zA-Z0-9_]*)\n([\s\S]*?)```/g, (_, lang, code) => {
    return `<pre><code>${escapeHtml(code.trim())}</code></pre>`;
  });

  // 2. Satır içi kod
  escaped = escaped.replace(/`([^`]+)`/g, "<code>$1</code>");

  // 3. Kalın yazı
  escaped = escaped.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");

  // 4. Başlıklar
  escaped = escaped.replace(/^### (.*$)/gim, "<h4>$1</h4>");
  escaped = escaped.replace(/^## (.*$)/gim, "<h3>$1</h3>");
  escaped = escaped.replace(/^# (.*$)/gim, "<h2>$1</h2>");

  // 5. Madde işaretleri
  escaped = escaped.replace(/^\s*[-*]\s+(.*$)/gim, "<li>$1</li>");

  // 6. Satır sonları
  escaped = escaped.replace(/\n/g, "<br/>");

  return escaped;
}

function formatNewsDate(isoStr, isEn) {
  if (!isoStr) return "";
  try {
    const d = new Date(isoStr);
    if (isNaN(d.getTime())) return isoStr;
    const day = String(d.getDate()).padStart(2, "0");
    const monthTr = ["Oca", "Şub", "Mar", "Nis", "May", "Haz", "Tem", "Ağu", "Eyl", "Eki", "Kas", "Ara"];
    const monthEn = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    const month = (isEn ? monthEn : monthTr)[d.getMonth()];
    const year = d.getFullYear();
    const hours = String(d.getHours()).padStart(2, "0");
    const mins = String(d.getMinutes()).padStart(2, "0");
    return `${day} ${month} ${year} · ${hours}:${mins}`;
  } catch {
    return isoStr;
  }
}

function homeTile(title, desc, iconName, route, accent, icon, state) {
  const isMobile = route === "android" || route === "ios";
  const allowed = Boolean(state?.mobileToolsAccess?.allowed);
  const statusClass = isMobile ? (allowed ? "is-unlocked" : "is-locked") : "";

  return `
    <button class="action-tile ${statusClass}" data-route="${route}" style="--accent:${accent}">
      <span class="tile-icon">${icon(iconName)}</span>
      <span>
        <h3>${title}</h3>
        <p>${desc}</p>
      </span>
      <span class="tile-arrow">→</span>
    </button>
  `;
}

export function metric(label, value, iconName, accent, icon) {
  return `
    <div class="metric" style="--accent:${accent}">
      <span class="metric-icon">${icon(iconName)}</span>
      <span><small>${label}</small><strong>${value}</strong></span>
    </div>
  `;
}
