// Ana Dashboard: Yapay Zeka Adli Ajanı, Yatay Haber Menüsü ve Adli Araçlar.

import { escapeHtml } from "../core/utils.js";
import { DEFAULT_AGENTS } from "../core/agent.js";

export function homePage({ t, icon, assetPath, theme, state }) {
  const isEn = state?.language === "en";

  // 1. Karşılama başlığı
  const hour = new Date().getHours();
  const greetingKey = hour < 12 ? "copilot.greeting.morning" : hour < 18 ? "copilot.greeting.afternoon" : "copilot.greeting.evening";
  const userName = state?.activeProfile?.fullName || state?.activeProfile?.username || "melihemik";
  const greetingText = t(greetingKey, { name: userName });

  // 2. Yapay Zeka Ajan durumu
  const agentState = state?.agent || state?.copilot || {};
  const rawAgents = agentState.agents || [];
  const agents = rawAgents.length > 0 ? rawAgents : DEFAULT_AGENTS;
  const selectedAgentId = agentState.selectedAgent || "agy";
  const selectedAgent = agents.find((a) => a.id === selectedAgentId) || agents[0];
  const models = selectedAgent?.models || [];
  const selectedModelId = agentState.selectedModel || (models[0]?.id || "");
  const selectedModel = models.find((m) => m.id === selectedModelId) || models[0];
  const isGenerating = Boolean(agentState.isGenerating);
  const messages = agentState.messages || [];

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
        <!-- Karşılama Çubuğu (Preview ve geri bildirim kaldırıldı, yalın karşılama) -->
        <div class="agent-greeting-bar">
          <div class="agent-greeting-left">
            <div class="agent-mascot">
              <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
                <path d="M12 2a2 2 0 0 1 2 2c0 .74-.4 1.39-1 1.73V7h1a7 7 0 0 1 7 7h1a1 1 0 0 1 1 1v3a1 1 0 0 1-1 1h-1v1a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-1H2a1 1 0 0 1-1-1v-3a1 1 0 0 1 1-1h1a7 7 0 0 1 7-7h1V5.73c-.6-.34-1-.99-1-1.73a2 2 0 0 1 2-2M7.5 13A2.5 2.5 0 0 0 5 15.5 2.5 2.5 0 0 0 7.5 18a2.5 2.5 0 0 0 2.5-2.5A2.5 2.5 0 0 0 7.5 13m9 0a2.5 2.5 0 0 0-2.5 2.5 2.5 2.5 0 0 0 2.5 2.5 2.5 2.5 0 0 0 2.5-2.5 2.5 2.5 0 0 0-2.5-2.5"/>
              </svg>
            </div>
            <h1 class="agent-greeting-title">${escapeHtml(greetingText)}</h1>
          </div>
        </div>

        <!-- Ajan Giriş Kutusu -->
        <div class="agent-box">
          <div class="agent-input-wrapper">
            <textarea
              id="agent-prompt-input"
              class="agent-textarea"
              placeholder="${escapeHtml(t("copilot.placeholder"))}"
              rows="2"
              data-agent-input
            >${escapeHtml(agentState.promptDraft || "")}</textarea>
          </div>

          <!-- Yalnızca Ajan Seçme, Model Seçme ve Gönderme -->
          <div class="agent-toolbar">
            <div class="agent-toolbar-left">
              <!-- Ajan Seçici: Özel Dropdown Menü (İkon Dahili) -->
              <div class="agent-custom-dropdown" id="agent-custom-dropdown">
                <button
                  type="button"
                  class="agent-dropdown-trigger"
                  data-agent-action="toggle-agent-menu"
                  title="${t("copilot.selectAgent")}"
                >
                  <span class="agent-dropdown-icon">${getAgentOfficialSvg(selectedAgentId)}</span>
                  <span class="agent-dropdown-label">${escapeHtml(selectedAgent?.name || "Agent")}</span>
                  <span class="agent-dropdown-chevron">▾</span>
                </button>
                <div class="agent-dropdown-menu" id="agent-dropdown-menu" style="display: none;">
                  ${agents.map((a) => {
                    const isSel = a.id === selectedAgentId;
                    const isInst = a.installed !== false;
                    return `
                      <div
                        class="agent-dropdown-item ${isSel ? "active" : ""} ${!isInst ? "disabled" : ""}"
                        data-agent-action="select-agent-item"
                        data-agent-id="${a.id}"
                      >
                        <span class="agent-item-icon">${getAgentOfficialSvg(a.id)}</span>
                        <span class="agent-item-name">${escapeHtml(a.name)}</span>
                        ${!isInst ? `<span class="agent-item-badge">Kurulu Değil</span>` : ""}
                      </div>
                    `;
                  }).join("")}
                </div>
              </div>

              <!-- Model Seçici: Yalnızca model varsa gösterilen Özel Dropdown Menü -->
              ${models.length > 0 ? `
              <div class="agent-custom-dropdown" id="model-custom-dropdown">
                <button
                  type="button"
                  class="agent-dropdown-trigger"
                  data-agent-action="toggle-model-menu"
                  title="${t("copilot.selectModel")}"
                >
                  <span class="agent-dropdown-label">${escapeHtml(selectedModel?.name || selectedModelId || t("copilot.selectModel"))}</span>
                  <span class="agent-dropdown-chevron">▾</span>
                </button>
                <div class="agent-dropdown-menu" id="model-dropdown-menu" style="display: none;">
                  ${models.map((m) => {
                    const isSel = m.id === selectedModelId;
                    return `
                      <div
                        class="agent-dropdown-item ${isSel ? "active" : ""}"
                        data-agent-action="select-model-item"
                        data-model-id="${m.id}"
                        title="${escapeHtml(m.description || "")}"
                      >
                        <span class="agent-item-name">${escapeHtml(m.name)}</span>
                      </div>
                    `;
                  }).join("")}
                </div>
              </div>
              ` : ""}
            </div>

            <div class="agent-toolbar-right">
              <!-- Gönder Butonu (Beyaz, yukarı bakan ok) -->
              <button
                type="button"
                class="agent-send-btn"
                data-agent-action="send"
                title="${t("copilot.send")}"
                ${isGenerating ? "disabled" : ""}
              >
                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                  <path d="M12 4l-6.5 6.5 1.41 1.41L11 7.83V20h2V7.83l4.09 4.08 1.41-1.41L12 4z"/>
                </svg>
              </button>
            </div>
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

      <!-- Standart Adli Araç Kartları Grid (Uygulamanın ana araçları) -->
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

/// Ajanlara ait resmi SVG logoları
function getAgentOfficialSvg(agentId) {
  switch (agentId) {
    case "agy":
      // Google Antigravity (AGY) – resmi Antigravity vektör logosu
      return `<svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" xmlns="http://www.w3.org/2000/svg" style="color:#60a5fa">
        <path d="M21.751 22.607c1.34 1.005 3.35.335 1.508-1.508C17.73 15.74 18.904 1 12.037 1 5.17 1 6.342 15.74.815 21.1c-2.01 2.009.167 2.511 1.507 1.506 5.192-3.517 4.857-9.714 9.715-9.714 4.857 0 4.522 6.197 9.714 9.715z"/>
      </svg>`;
    case "claude":
      // Anthropic Claude Code – resmi güneş ışını (sunburst) logosu, turuncu (#D97757)
      return `<svg viewBox="0 0 248 248" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path d="M52.4 162.9L98.8 136.9l.8-2.3-.8-1.3h-2.3l-7.8-.5-26.5-.7-22.9-.9L17 130l-5.6-1.2-5.2-7 .5-3.4 4.7-3.2 6.8.6 14.9 1.1 22.4 1.5 16.2.9 24 2.5h3.8l.5-1.5-1.3-.9-1-.9L74.6 102.7 49.5 86.2l-13.1-9.6-7-4.8-3.6-4.5-1.5-9.9 6.4-7.1 8.7.6 2.2.6 8.8 6.7 18.7 14.5 24.5 18 3.6 3 1.4-1 .2-.7-1.7-2.7-13.2-24-14.1-24.5-6.4-10.2-1.7-6c-.6-2.5-1-4.6-1-7.2L67.8 7.5l4.1-1.3 9.8 1.3 4.1 3.5 6.1 14 9.8 21.9 15.3 29.8 4.5 8.9 2.4 8.2.9 2.5h1.5v-1.4l1.3-16.8 2.3-20.6 2.3-26.5.8-7.4 3.7-9 7.4-4.8 5.7 2.7 4.7 6.7-.6 4.4-2.8 18.2-5.5 28.5-3.6 19.1h2l2.4-2.5 9.7-12.8 16.2-20.3 7.1-8 8.4-8.9 5.3-4.3h10.2l7.4 11.1-3.3 11.5-10.4 13.2-8.7 11.2-12.4 16.6-7.7 13.4.7 1.1 1.9-.2 28-6 15.2-2.7 18.1-3.1 8.1 3.8.9 3.9-3.2 7.9-19.4 4.7-22.7 4.6-33.8 7.9-.4.3.4.7 15.2 1.4 6.5.4h15.9l29.7 2.2 7.8 5.1 4.6 6.3-.8 4.8-12 6-16-3.8-37.6-9-12.9-3.2h-1.8v1.1l10.7 10.5 19.7 17.7 24.6 22.9 1.3 5.7-3.2 4.5-3.3-.5-21.7-16.3-8.4-7.3-18.8-16h-1.3v1.7l4.3 6.4 23.1 34.6 1.1 10.6-1.7 3.4-6 2.1-6.5-1.2-13.6-19-13.9-21.3-11.2-19.1-1.4.9-6.7 71.2-3.1 3.7-7.1 2.7-6-4.5-3.2-7.3 3.2-14.5 3.8-18.9 3.1-15 2.8-18.7 1.7-6.2-.2-.4-1.4.2-14.1 19.3-21.4 29-16.9 18.1-4.1 1.7-7-3.7.6-6.5 4-5.8 23.4-29.8 14.1-18.6 9.1-10.6-.1-1.5-.5 0L46.7 188.5l-11.1 1.4-4.8-4.5.6-7.3 2.3-2.4 18.7-12.9z" fill="#D97757"/>
      </svg>`;
    case "codex":
      // OpenAI / Codex – resmi OpenAI spiral geometrik logosu, yeşil (#10a37f)
      return `<svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" xmlns="http://www.w3.org/2000/svg" style="color:#10a37f">
        <path d="M9.205 8.658v-2.26c0-.19.072-.333.238-.428l4.543-2.616c.619-.357 1.356-.523 2.117-.523 2.854 0 4.662 2.212 4.662 4.566 0 .167 0 .357-.024.547l-4.71-2.759a.797.797 0 00-.856 0l-5.97 3.473zm10.609 8.8V12.06c0-.333-.143-.57-.429-.737l-5.97-3.473 1.95-1.118a.433.433 0 01.476 0l4.543 2.617c1.309.76 2.189 2.378 2.189 3.948 0 1.808-1.07 3.473-2.76 4.163zM7.802 12.703l-1.95-1.142c-.167-.095-.239-.238-.239-.428V5.899c0-2.545 1.95-4.472 4.591-4.472 1 0 1.927.333 2.712.928L8.23 5.067c-.285.166-.428.404-.428.737v6.898zM12 15.128l-2.795-1.57v-3.33L12 8.658l2.795 1.57v3.33L12 15.128zm1.796 7.23c-1 0-1.927-.332-2.712-.927l4.686-2.712c.285-.166.428-.404.428-.737v-6.898l1.974 1.142c.167.095.238.238.238.428v5.233c0 2.545-1.974 4.472-4.614 4.472zm-5.637-5.303l-4.544-2.617c-1.308-.761-2.188-2.378-2.188-3.948A4.482 4.482 0 014.21 6.327v5.423c0 .333.143.571.428.738l5.947 3.449-1.95 1.118a.432.432 0 01-.476 0zm-.262 3.9c-2.688 0-4.662-2.021-4.662-4.519 0-.19.024-.38.047-.57l4.686 2.71c.286.167.571.167.856 0l5.97-3.448v2.26c0 .19-.07.333-.237.428l-4.543 2.616c-.619.357-1.356.523-2.117.523zm5.899 2.83a5.947 5.947 0 005.827-4.756C22.287 18.339 24 15.84 24 13.296c0-1.665-.713-3.282-1.998-4.448.119-.5.19-.999.19-1.498 0-3.401-2.759-5.947-5.946-5.947-.642 0-1.26.095-1.88.31A5.962 5.962 0 0010.205 0a5.947 5.947 0 00-5.827 4.757C1.713 5.447 0 7.945 0 10.49c0 1.666.713 3.283 1.998 4.448-.119.5-.19 1-.19 1.499 0 3.401 2.759 5.946 5.946 5.946.642 0 1.26-.095 1.88-.309a5.96 5.96 0 004.162 1.713z"/>
      </svg>`;
    case "pi":
      // Pi Coding Agent – earendil-works resmi üç renk piksel logosu
      return `<svg viewBox="0 0 800 800" width="16" height="16" xmlns="http://www.w3.org/2000/svg">
        <path fill="#F09082" d="M165.29 165.29H517.36V400H400V282.65H165.29Z"/>
        <path fill="#4D9ABF" d="M165.29 282.65H282.65V400H400V517.36H282.65V634.72H165.29Z"/>
        <path fill="#F1BE58" d="M517.36 400H634.72V634.72H517.36Z"/>
      </svg>`;
    case "opencode":
      // OpenCode – resmi terminal kutusu logosu (beyaz kontur)
      return `<svg viewBox="0 0 512 512" width="16" height="16" fill="none" xmlns="http://www.w3.org/2000/svg">
        <path fill-rule="evenodd" clip-rule="evenodd" d="M384 416H128V96H384V416ZM320 160H192V352H320V160Z" fill="white"/>
        <path d="M320 224V352H192V224H320Z" fill="#5A5858"/>
      </svg>`;
    default:
      return `<svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" style="color:#60a5fa">
        <path d="M12 2C12 7.52 7.52 12 2 12C7.52 12 12 16.48 12 22C12 16.48 16.48 12 22 12C16.48 12 12 7.52 12 2Z"/>
      </svg>`;
  }
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
