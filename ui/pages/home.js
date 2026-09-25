// Ana Dashboard: Yapay Zeka Adli Ajanı, Yatay Haber Menüsü ve Adli Araçlar.

import { escapeHtml } from "../core/utils.js";
import { DEFAULT_AGENTS } from "../core/agent.js";

export function homePage({ t, icon, assetPath, theme, state }) {
  const isEn = state?.language === "en";

  // 1. Karşılama başlığı (saate göre: sabah, gün, akşam, gece)
  const hour = new Date().getHours();
  const greetingKey = hour < 5
    ? "agent.greeting.night"
    : hour < 12
      ? "agent.greeting.morning"
      : hour < 18
        ? "agent.greeting.afternoon"
        : hour < 23
          ? "agent.greeting.evening"
          : "agent.greeting.night";
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
  // kurulu ajan yoksa gonder butonu ve metin alani kilitlensin
  const hasInstalledAgent = agents.some((a) => a.installed === true);
  const selectedInstalled = selectedAgent ? selectedAgent.installed === true : false;
  const isInputDisabled = isGenerating || !hasInstalledAgent || !selectedInstalled;

  // Sohbet geçmişi HTML
  const chatHistoryHtml = messages.length > 0 || isGenerating
    ? `
      <div class="agent-chat-history">
        ${messages.map((msg) => renderChatMessage(msg, state, t, escapeHtml, icon)).join("")}
        ${
          isGenerating
            ? `
              <div class="agent-msg assistant">
                <div class="agent-msg-header">
                  <span class="agent-msg-author"><span class="agent-msg-avatar">${getAgentOfficialSvg(selectedAgentId)}</span> ${escapeHtml(selectedAgent?.name || "Agent")}</span>
                </div>
                <div class="agent-msg-body">
                  <div class="loading-row"><span class="spinner" aria-hidden="true"></span><span>${t("copilot.generating")}</span></div>
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
          <span class="news-h-author"><span class="inline-ico">${icon ? icon("user") : ""}</span> @${escapeHtml(author)}</span>
          <span class="news-h-datetime"><span class="inline-ico">${icon ? icon("clock") : ""}</span> ${dateFormatted}</span>
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
    ? renderElevationModal(elevationModal, t, escapeHtml, icon)
    : "";

  return `
    <section class="page page-home">
      <!-- Yapay Zeka Ajan Bölümü (en üstte, ortalı) -->
      <div class="agent-container">
        <!-- Karşılama Çubuğu (Preview ve geri bildirim kaldırıldı, yalın karşılama) -->
        <div class="agent-greeting-bar">
          <div class="agent-greeting-left">
            <span class="agent-mascot" aria-hidden="true">
              <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="m12 3-1.9 5.8a2 2 0 0 1-1.3 1.3L3 12l5.8 1.9a2 2 0 0 1 1.3 1.3L12 21l1.9-5.8a2 2 0 0 1 1.3-1.3L21 12l-5.8-1.9a2 2 0 0 1-1.3-1.3L12 3z"/>
                <path d="M5 3v4"/>
                <path d="M3 5h4"/>
                <path d="M19 17v4"/>
                <path d="M17 19h4"/>
              </svg>
            </span>
            <h1 class="agent-greeting-title">${escapeHtml(greetingText)}</h1>
          </div>
        </div>

        <!-- Sohbet Geçmişi & Komut Çalıştırma Sonuçları (yazma kutusu en altta) -->
        ${chatHistoryHtml}

        <!-- Ajan Giriş Kutusu -->
        <div class="agent-box">
          <div class="agent-input-wrapper">
            <textarea
              id="agent-prompt-input"
              class="agent-textarea"
              placeholder="${escapeHtml(
                !hasInstalledAgent
                  ? (t("copilot.noAgentInstalled") || "Kurulu yapay zeka ajanı yok. En az bir ajanı kurun.")
                  : (!selectedInstalled
                    ? (t("copilot.agentNotInstalled") || "Seçilen ajan kurulu değil.")
                    : t("copilot.placeholder"))
              )}"
              rows="2"
              data-agent-input
              ${isInputDisabled ? "disabled" : ""}
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
                  aria-haspopup="listbox"
                >
                  <span class="agent-dropdown-icon">${getAgentOfficialSvg(selectedAgentId)}</span>
                  <span class="agent-dropdown-label">${escapeHtml(selectedAgent?.name || "Agent")}</span>
                  <span class="agent-dropdown-chevron">
                    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                      <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                  </span>
                </button>
                <div class="agent-dropdown-menu" id="agent-dropdown-menu" role="listbox">
                  <div class="agent-dropdown-header">${t("copilot.selectAgent") || "Yapay Zeka Ajanı"}</div>
                  ${agents.map((a) => {
                    const isSel = a.id === selectedAgentId;
                    const isInst = a.installed !== false;
                    return `
                      <div
                        class="agent-dropdown-item ${isSel ? "active" : ""} ${!isInst ? "disabled" : ""}"
                        data-agent-action="select-agent-item"
                        data-agent-id="${a.id}"
                        role="option"
                        aria-selected="${isSel}"
                      >
                        <span class="agent-item-icon">${getAgentOfficialSvg(a.id)}</span>
                        <span class="agent-item-name">${escapeHtml(a.name)}</span>
                        ${!isInst ? `<span class="agent-item-badge">Kurulu Değil</span>` : ""}
                        ${isSel ? `
                          <span class="agent-item-check">
                            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                              <polyline points="20 6 9 17 4 12"></polyline>
                            </svg>
                          </span>
                        ` : ""}
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
                  aria-haspopup="listbox"
                >
                  <span class="agent-dropdown-label">${escapeHtml(selectedModel?.name || selectedModelId || t("copilot.selectModel"))}</span>
                  <span class="agent-dropdown-chevron">
                    <svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                      <polyline points="6 9 12 15 18 9"></polyline>
                    </svg>
                  </span>
                </button>
                <div class="agent-dropdown-menu" id="model-dropdown-menu" role="listbox">
                  <div class="agent-dropdown-header">${t("copilot.selectModel") || "Model Seçimi"}</div>
                  ${models.map((m) => {
                    const isSel = m.id === selectedModelId;
                    return `
                      <div
                        class="agent-dropdown-item ${isSel ? "active" : ""}"
                        data-agent-action="select-model-item"
                        data-model-id="${m.id}"
                        title="${escapeHtml(m.description || "")}"
                        role="option"
                        aria-selected="${isSel}"
                      >
                        <span class="agent-item-name">${escapeHtml(m.name)}</span>
                        ${isSel ? `
                          <span class="agent-item-check">
                            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
                              <polyline points="20 6 9 17 4 12"></polyline>
                            </svg>
                          </span>
                        ` : ""}
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
                title="${escapeHtml(
                  !hasInstalledAgent
                    ? (t("copilot.noAgentInstalled") || "Kurulu yapay zeka ajanı yok")
                    : (!selectedInstalled
                      ? (t("copilot.agentNotInstalled") || "Seçilen ajan kurulu değil")
                      : t("copilot.send"))
                )}"
                ${isInputDisabled ? "disabled" : ""}
              >
                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                  <path d="M12 4l-6.5 6.5 1.41 1.41L11 7.83V20h2V7.83l4.09 4.08 1.41-1.41L12 4z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Yatay Haber Menüsü (Fotoğrafsız, ajanın altında ortada) -->
      <div class="home-news-horizontal-section">
        <div class="news-horizontal-header">
          <span class="news-header-icon">${icon ? icon("globe") : ""}</span>
          <h3>News / Haberler</h3>
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

/// Ajanlara ait resmi SVG ve vektör logoları
function getAgentOfficialSvg(agentId) {
  const id = String(agentId || "agy").toLowerCase();
  switch (id) {
    case "agy":
    case "antigravity":
      // Google Antigravity (AGY) – sistemdeki resmi Antigravity uygulama logosu
      return `<img src="./assets/icons/antigravity.png" width="18" height="18" alt="AGY" style="border-radius:4px;display:block;object-fit:contain;" />`;

    case "claude":
      // Anthropic Claude Code – resmi temiz Claude vektör logosu, turuncu (#D97757)
      return `<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" xmlns="http://www.w3.org/2000/svg" style="color:#D97757">
        <path fill="currentColor" d="m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z"/>
      </svg>`;

    case "codex":
    case "openai":
      // OpenAI Codex – resmi OpenAI spiral vektör logosu, yeşil (#10a37f)
      return `<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" xmlns="http://www.w3.org/2000/svg" style="color:#10a37f">
        <path d="M22.2819 9.8211a5.9847 5.9847 0 0 0-.5157-4.9108 6.0462 6.0462 0 0 0-6.5098-2.9A6.0651 6.0651 0 0 0 4.9807 4.1818a5.9847 5.9847 0 0 0-3.9977 2.9 6.0462 6.0462 0 0 0 .7427 7.0966 5.98 5.98 0 0 0 .511 4.9107 6.051 6.051 0 0 0 6.5146 2.9001A5.9847 5.9847 0 0 0 13.2599 24a6.0557 6.0557 0 0 0 5.7718-4.2058 5.9894 5.9894 0 0 0 3.9977-2.9001 6.0557 6.0557 0 0 0-.7475-7.0729zm-9.022 12.6081a4.4755 4.4755 0 0 1-2.8764-1.0408l.1419-.0804 4.7783-2.7582a.7948.7948 0 0 0 .3927-.6813v-6.7369l2.02 1.1686a.071.071 0 0 1 .038.052v5.5826a4.504 4.504 0 0 1-4.4945 4.4944zm-9.6607-4.1254a4.4708 4.4708 0 0 1-.5346-3.0137l.142.0852 4.783 2.7582a.7712.7712 0 0 0 .7806 0l5.8428-3.3685v2.3324a.0804.0804 0 0 1-.0332.0615L9.74 19.9502a4.4992 4.4992 0 0 1-6.1408-1.6464zM2.3408 7.8956a4.485 4.485 0 0 1 2.3655-1.9728V11.6a.7664.7664 0 0 0 .3879.6765l5.8144 3.3543-2.0201 1.1685a.0757.0757 0 0 1-.071 0l-4.8303-2.7865A4.504 4.504 0 0 1 2.3408 7.872zm16.5963 3.8558L13.1038 8.364 15.1192 7.2a.0757.0757 0 0 1 .071 0l4.8303 2.7913a4.4944 4.4944 0 0 1-.6765 8.1042v-5.6772a.79.79 0 0 0-.407-.667zm2.0107-3.0231l-.142-.0852-4.7735-2.7818a.7759.7759 0 0 0-.7854 0L9.409 9.2297V6.8974a.0662.0662 0 0 1 .0284-.0615l4.8303-2.7866a4.4992 4.4992 0 0 1 6.6802 4.66zM8.3065 12.863l-2.02-1.1638a.0804.0804 0 0 1-.038-.0567V6.0742a4.4992 4.4992 0 0 1 7.3757-3.4537l-.142.0805L8.704 5.459a.7948.7948 0 0 0-.3927.6813zm1.0976-2.3654l2.602-1.4998 2.6069 1.4998v2.9994l-2.5974 1.4997-2.6067-1.4997Z"/>
      </svg>`;

    case "pi":
      // Pi – earendil-works resmi geometrik üç renk piksel logosu
      return `<svg viewBox="165 165 470 470" width="18" height="18" xmlns="http://www.w3.org/2000/svg">
        <path fill="#F09082" d="M165.29 165.29H517.36V400H400V282.65H165.29Z"/>
        <path fill="#4D9ABF" d="M165.29 282.65H282.65V400H400V517.36H282.65V634.72H165.29Z"/>
        <path fill="#F1BE58" d="M517.36 400H634.72V634.72H517.36Z"/>
      </svg>`;

    case "opencode":
      // OpenCode – resmi opencode.ai kare marka logosu
      return `<svg viewBox="0 0 300 300" width="18" height="18" fill="none" xmlns="http://www.w3.org/2000/svg">
        <g transform="translate(30, 0)">
          <path d="M180 240H60V120H180V240Z" fill="#CFCECD"/>
          <path d="M180 60H60V240H180V60ZM240 300H0V0H240V300Z" fill="white"/>
        </g>
      </svg>`;

    default:
      return `<svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" style="color:#60a5fa">
        <path d="M12 2C12 7.52 7.52 12 2 12C7.52 12 12 16.48 12 22C12 16.48 16.48 12 22 12C16.48 12 12 7.52 12 2Z"/>
      </svg>`;
  }
}

function getAgentDisplayName(agentId) {
  const id = String(agentId || "").toLowerCase();
  switch (id) {
    case "agy":
    case "antigravity":
      return "Antigravity (AGY)";
    case "claude":
      return "Claude Code";
    case "codex":
      return "Codex";
    case "pi":
      return "Pi";
    case "opencode":
      return "OpenCode";
    default:
      return "Amele Ajanı";
  }
}

function renderChatMessage(msg, state, t, escapeHtml, iconFn) {
  const isUser = msg.role === "user";
  const authorName = isUser
    ? (state?.activeProfile?.fullName || state?.activeProfile?.username || "melihemik")
    : (msg.agent ? (msg.model ? `${getAgentDisplayName(msg.agent)} (${msg.model})` : getAgentDisplayName(msg.agent)) : "Amele Ajanı");

  const formattedContent = formatSimpleMarkdown(msg.content);

  // avatar: kullanıcıda kişi ikonu, ajanda seçili ajan logosu
  const rawAgentId = String(msg.agent || msg.agentId || state?.agent?.selectedAgent || state?.copilot?.selectedAgent || "agy").toLowerCase();
  const userAvatar = iconFn
    ? iconFn("user")
    : `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="4"/><path d="M4 21c1.8-4 4.4-6 8-6s6.2 2 8 6"/></svg>`;
  const agentAvatar = getAgentOfficialSvg(rawAgentId);
  const avatarHtml = isUser ? userAvatar : agentAvatar;

  // Önerilen CLI komutu varsa kart olarak çiz
  const commandCardHtml = msg.suggested_command
    ? renderCommandCard(msg.suggested_command, msg.id, msg.execResult, state, t, escapeHtml, iconFn)
    : "";

  const isEn = state?.language === "en";
  const relaunchBtnHtml = (!isUser && msg.prompt)
    ? `
      <div class="agent-msg-actions">
        <button
          type="button"
          class="agent-relaunch-btn"
          data-agent-action="relaunch-terminal"
          data-prompt="${escapeHtml(msg.prompt)}"
          data-agent-id="${escapeHtml(rawAgentId)}"
          data-model-id="${escapeHtml(msg.model || "")}"
        >
          <span class="inline-ico">
            <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="4 17 10 11 4 5"></polyline>
              <line x1="12" y1="19" x2="20" y2="19"></line>
            </svg>
          </span>
          <span>${isEn ? "Open in Terminal" : "Terminalde Yeniden Aç"}</span>
        </button>
      </div>
    `
    : "";

  return `
    <div class="agent-msg ${isUser ? "user" : "assistant"}" data-msg-id="${msg.id}">
      <div class="agent-msg-header">
        <span class="agent-msg-author"><span class="agent-msg-avatar">${avatarHtml}</span> ${escapeHtml(authorName)}</span>
        <span class="agent-msg-time">${new Date(msg.timestamp || Date.now()).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</span>
      </div>
      <div class="agent-msg-body">
        ${formattedContent}
      </div>
      ${relaunchBtnHtml}
      ${commandCardHtml}
    </div>
  `;
}

function renderCommandCard(cmd, messageId, execResult, state, t, escapeHtml, iconFn) {
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
        <span class="agent-badge ${badgeClass}"><span class="inline-ico"><svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg></span> ${badgeLabel}</span>
        <button type="button" class="agent-copy-btn" data-agent-action="copy-cmd" data-cmd="${escapeHtml(cmd)}">
          <span class="inline-ico"><svg viewBox="0 0 24 24" width="12" height="12" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="12" height="12" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/></svg></span> ${t("copilot.copy")}
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
          <span class="inline-ico"><svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor"><path d="M8 5v14l11-7Z"/></svg></span> ${t("copilot.runCommand")}
        </button>
      </div>
      ${execHtml}
    </div>
  `;
}

function renderElevationModal(modal, t, escapeHtml, iconFn) {
  const isLinux = modal.os === "linux";

  if (isLinux) {
    return `
      <div class="elevation-modal-overlay" id="sudo-elevation-modal">
        <div class="elevation-modal-card">
          <div class="elevation-modal-icon linux"><svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/></svg></div>
          <h3 class="elevation-modal-title">${t("elevation.linuxTitle")}</h3>
          <p class="elevation-modal-desc">${escapeHtml(modal.reason || t("elevation.linuxDesc"))}</p>
          <div class="elevation-modal-cmd">
            <code>${escapeHtml(modal.command)}</code>
          </div>
          <div class="elevation-modal-form">
            <label for="sudo-password-input" class="elevation-modal-label">
              <span class="inline-ico">
                <svg viewBox="0 0 24 24" width="13" height="13" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/>
                  <path d="M7 11V7a5 5 0 0 1 10 0v4"/>
                </svg>
              </span>
              <span>${t("elevation.passwordLabel") || "Sudo Parolası"}</span>
            </label>
            <div class="elevation-input-box">
              <input
                type="password"
                id="sudo-password-input"
                class="elevation-password-input"
                placeholder="${t("elevation.passwordPlaceholder") || "Sistem sudo parolanızı girin..."}"
                autocomplete="current-password"
                data-elevation-input="password"
              />
              <button
                type="button"
                class="elevation-pwd-toggle"
                data-elevation-action="toggle-pwd"
                title="Parolayı göster/gizle"
              >
                <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
                  <circle cx="12" cy="12" r="3"/>
                </svg>
              </button>
            </div>
            <small class="elevation-hint">${t("elevation.passwordHint") || "Parola yalnızca bu komut için tek seferlik kullanılır, kaydedilmez."}</small>
          </div>
          <div class="elevation-modal-actions">
            <button type="button" class="btn btn-secondary" data-elevation-action="cancel">
              ${t("common.cancel")}
            </button>
            <button
              type="button"
              class="btn btn-danger"
              data-elevation-action="confirm-linux"
              data-cmd="${escapeHtml(modal.command)}"
              data-msg-id="${modal.messageId || ""}"
            >
              ${t("elevation.authenticateAndRun")}
            </button>
          </div>
        </div>
      </div>
    `;
  }

  // Windows Yönetici İzni Modalı
  return `
    <div class="elevation-modal-overlay" id="windows-elevation-modal">
      <div class="elevation-modal-card">
        <div class="elevation-modal-icon windows"><svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3 5 6v6c0 4.5 3 7.5 7 9 4-1.5 7-4.5 7-9V6l-7-3Z"/><path d="m9 12 2 2 4-5"/></svg></div>
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

/**
 * Sol kenarda dikey listelenen vaka paneli.
 * Ana sayfanın ortalı 880px tasarımını kesinlikle bozmaz.
 */
export function renderCaseSidebar(state, t, icon, esc) {
  const cases = Array.isArray(state?.cases) ? state.cases : [];
  const activeCaseName = state?.activeCase?.case_name || state?.pendingCaseName || "";

  const casesHtml = cases.length > 0
    ? cases.map((c) => {
        const name = c.case_name || "";
        const isActive = name === activeCaseName;
        const a = c.artifacts || {};

        // Platform ve araç rozetleri
        const badges = [];

        // Linux Disk
        const linuxDiskCount = a.linux_disk || 0;
        if (linuxDiskCount > 0) {
          badges.push(`
            <span class="case-badge badge-linux" title="Linux Disk İmajı: ${linuxDiskCount}">
              <span class="badge-ico">${icon("linux")}</span>
              <span class="badge-ico">${icon("disk")}</span>
              <span class="badge-num">${linuxDiskCount}</span>
            </span>
          `);
        } else if ((c.output_count || 0) > 0 && !(a.windows_disk > 0)) {
          badges.push(`
            <span class="case-badge badge-linux" title="Disk İmajı: ${c.output_count}">
              <span class="badge-ico">${icon("linux")}</span>
              <span class="badge-ico">${icon("disk")}</span>
              <span class="badge-num">${c.output_count}</span>
            </span>
          `);
        }

        // Windows Disk
        const winDiskCount = a.windows_disk || 0;
        if (winDiskCount > 0) {
          badges.push(`
            <span class="case-badge badge-windows" title="Windows Disk İmajı: ${winDiskCount}">
              <span class="badge-ico">${icon("windows")}</span>
              <span class="badge-ico">${icon("disk")}</span>
              <span class="badge-num">${winDiskCount}</span>
            </span>
          `);
        }

        // Linux RAM
        const linuxRamCount = a.linux_ram || 0;
        if (linuxRamCount > 0) {
          badges.push(`
            <span class="case-badge badge-ram" title="Linux RAM Dökümü: ${linuxRamCount}">
              <span class="badge-ico">${icon("linux")}</span>
              <span class="badge-ico">${icon("chip")}</span>
              <span class="badge-num">${linuxRamCount}</span>
            </span>
          `);
        } else if ((c.ram_count || 0) > 0 && !(a.windows_ram > 0)) {
          badges.push(`
            <span class="case-badge badge-ram" title="RAM Dökümü: ${c.ram_count}">
              <span class="badge-ico">${icon("chip")}</span>
              <span class="badge-num">${c.ram_count}</span>
            </span>
          `);
        }

        // Windows RAM
        const winRamCount = a.windows_ram || 0;
        if (winRamCount > 0) {
          badges.push(`
            <span class="case-badge badge-windows" title="Windows RAM Dökümü: ${winRamCount}">
              <span class="badge-ico">${icon("windows")}</span>
              <span class="badge-ico">${icon("chip")}</span>
              <span class="badge-num">${winRamCount}</span>
            </span>
          `);
        }

        // Android
        const androidCount = a.android || c.android_count || 0;
        if (androidCount > 0) {
          badges.push(`
            <span class="case-badge badge-android" title="Android Edinim: ${androidCount}">
              <span class="badge-ico">${icon("android")}</span>
              <span class="badge-num">${androidCount}</span>
            </span>
          `);
        }

        // iOS
        const iosCount = a.ios || c.ios_count || 0;
        if (iosCount > 0) {
          badges.push(`
            <span class="case-badge badge-ios" title="iOS Yedek: ${iosCount}">
              <span class="badge-ico">${icon("ios")}</span>
              <span class="badge-num">${iosCount}</span>
            </span>
          `);
        }

        // Docker
        const dockerCount = a.docker || c.docker_count || 0;
        if (dockerCount > 0) {
          badges.push(`
            <span class="case-badge badge-docker" title="Docker Konteyner: ${dockerCount}">
              <span class="badge-ico">${icon("docker")}</span>
              <span class="badge-num">${dockerCount}</span>
            </span>
          `);
        }

        // Hash
        const hashCount = a.hash || c.hash_count || 0;
        if (hashCount > 0) {
          badges.push(`
            <span class="case-badge badge-hash" title="Hash Kayıtları: ${hashCount}">
              <span class="badge-ico">${icon("check")}</span>
              <span class="badge-num">${hashCount}</span>
            </span>
          `);
        }

        // Rapor
        const reportCount = a.report || c.report_count || 0;
        if (reportCount > 0) {
          badges.push(`
            <span class="case-badge badge-report" title="Raporlar: ${reportCount}">
              <span class="badge-ico">${icon("report")}</span>
              <span class="badge-num">${reportCount}</span>
            </span>
          `);
        }

        const dateStr = c.created_at ? c.created_at.slice(0, 10) : "";

        return `
          <div
            class="case-sidebar-card ${isActive ? "active" : ""}"
            data-action="select-case"
            data-case-name="${esc(name)}"
            title="${esc(name)}"
            role="button"
            tabindex="0"
          >
            <div class="case-sidebar-card-top">
              <span class="case-sidebar-dot"></span>
              <span class="case-sidebar-card-name">${esc(name)}</span>
              ${isActive ? `<span class="case-sidebar-default-tag">${t("case.active") || "Varsayılan"}</span>` : ""}
            </div>
            <div class="case-sidebar-badges">
              ${badges.length > 0 ? badges.join("") : `<span class="case-badge-empty">Henüz edinim yok</span>`}
            </div>
            ${dateStr ? `<div class="case-sidebar-date">${esc(dateStr)}</div>` : ""}
          </div>
        `;
      }).join("")
    : `
      <div class="case-sidebar-empty">
        <span class="case-sidebar-empty-icon">${icon("folder")}</span>
        <p>${t("case.noCases") || "Kayıtlı vaka yok"}</p>
        <button type="button" class="case-sidebar-create-btn" data-action="new-case-prompt">
          ${icon("folder")} ${t("case.create") || "Yeni Vaka Oluştur"}
        </button>
      </div>
    `;

  return `
    <button
      type="button"
      class="home-case-sidebar-toggle"
      data-action="toggle-case-sidebar"
      title="${t("case.sidebarTitle") || "Vakalar"}"
      aria-label="Vaka Menüsü"
    >
      <span class="toggle-ico">${icon("folder")}</span>
      <span class="toggle-txt">${t("case.sidebarTitle") || "Vakalar"}</span>
      <span class="toggle-badge">${cases.length}</span>
    </button>

    <div class="home-case-sidebar-backdrop" data-action="close-case-sidebar"></div>

    <aside class="home-case-sidebar" id="home-case-sidebar">
      <div class="case-sidebar-header">
        <div class="case-sidebar-header-left">
          <span class="case-sidebar-header-icon">${icon("folder")}</span>
          <span class="case-sidebar-header-title">${t("case.sidebarTitle") || "Vakalar"}</span>
          <span class="case-sidebar-count-badge">${cases.length}</span>
        </div>
        <div class="case-sidebar-header-right">
          <button
            type="button"
            class="case-sidebar-header-btn"
            data-action="refresh-cases"
            title="${t("case.refresh") || "Yenile"}"
          >
            ${icon("refresh")}
          </button>
          <button
            type="button"
            class="case-sidebar-header-btn case-sidebar-close-btn"
            data-action="close-case-sidebar"
            title="Kapat"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>

      <div class="case-sidebar-list" id="case-sidebar-list">
        ${casesHtml}
      </div>

      <div class="case-sidebar-footer">
        <button
          type="button"
          class="case-sidebar-new-btn"
          data-action="new-case-prompt"
        >
          <span class="plus-ico">+</span>
          <span>${t("case.newCase") || "Yeni Vaka Oluştur"}</span>
        </button>
      </div>
    </aside>
  `;
}

