// adli yardımcılar, vaka yönetimi, raporlama, ayarlar ve hakkında sayfası.

export function otherPage({
  t,
  icon,
  state,
  pageTitle,
  pickerField,
  field,
  escapeHtml,
  caseSelectOptions,
  detailPanel,
}) {
  return `
    <section class="page">
      ${pageTitle(t("other.title"), t("other.desc"), "tiles")}
      <div class="other-grid">
        ${simpleCard(t("other.hash.title"), t("other.hash.desc"), "shield", "hash", icon, t, "1")}
        ${simpleCard(t("other.evidence.title"), t("other.evidence.desc"), "scale", "evidence", icon, t, "2")}
        ${simpleCard(t("other.reports.title"), t("other.reports.desc"), "report", "reports", icon, t, "3")}
        ${simpleCard(t("other.history.title"), t("other.history.desc"), "clock", "history", icon, t, "4")}
        ${simpleCard(t("other.logs.title"), t("other.logs.desc"), "clock", "logs", icon, t, "5")}
      </div>
      <div id="other-detail" class="workflow-panel" style="margin-top:16px">${detailPanel(state.activeTab)}</div>
    </section>
  `;
}

function simpleCard(title, desc, iconName, tab, icon, t, shortcutKey = "") {
  return `
    <button class="forensic-card" data-tab="${tab}" ${shortcutKey ? `data-shortcut="${shortcutKey}"` : ""}>
      <span class="card-icon">${icon(iconName)}</span>
      ${shortcutKey ? `<span class="shortcut-key-badge" aria-hidden="true">${shortcutKey}</span>` : ""}
      <h3>${title}</h3>
      <p>${desc}</p>
      <span class="meta">${t("open")}</span>
    </button>
  `;
}

export function detailPanel({
  tab,
  t,
  icon,
  state,
  pickerField,
  field,
  escapeHtml,
  caseSelectOptions,
  hashPanel,
}) {
  if (tab === "evidence") {
    return `
      <p class="section-label">${t("case.management")}</p>
      <div class="side-info" style="margin: 12px 0 16px;">
        <span class="metric-icon">${icon("folder")}</span>
        <span><strong>${t("case.location")}</strong><small data-case-base>${escapeHtml(state.caseBaseDir || "~/Amele/Vakalar")}</small></span>
      </div>
      <p class="field-hint">${t("case.fixedLocation")}</p>
      ${field(t("case.name"), '<input id="case-name" class="input" placeholder="Case_2026_001" />')}
      <div class="button-row" style="margin: 14px 0;">
        <button class="primary-button" data-action="create-case" data-shortcut="C">${icon("folder")} ${t("case.create")}<span class="shortcut-key-badge" aria-hidden="true">C</span></button>
        <button class="secondary-button" data-action="refresh-cases" data-shortcut="R">${icon("refresh")} ${t("case.refresh")}<span class="shortcut-key-badge" aria-hidden="true">R</span></button>
        <button class="secondary-button" data-action="create-manifest" data-shortcut="M">${icon("shield")} ${t("case.manifest.create")}<span class="shortcut-key-badge" aria-hidden="true">M</span></button>
      </div>
      <div class="case-status-container" style="display:flex; flex-direction:column; gap:8px; margin: 12px 0 6px;">
        <div class="status-badge" data-case-status style="display:none"></div>
        <div class="status-badge" data-manifest-status style="display:none"></div>
      </div>
      <div class="section-divider"></div>
      <p class="section-label">${t("case.files")}</p>
      ${field(t("case.folder"), `<select id="case-folder" class="select"><option value="ciktilar">${t("case.outputs")}</option><option value="disk_imajlari">${t("case.diskImages")}</option><option value="ram">${t("case.ram")}</option><option value="android">${t("case.android")}</option><option value="raporlar">${t("case.reports")}</option><option value="hash">${t("case.hash")}</option><option value="notlar">${t("case.notes")}</option><option value="gunlukler">${t("case.logs")}</option></select>`)}
      ${field(t("case.file"), `<select id="case-file-list" class="select"><option>${t("case.listFilesPlaceholder")}</option></select>`)}
      <div class="button-row" style="margin: 14px 0;">
        <button class="secondary-button" data-action="list-files" data-shortcut="L">${icon("search")} ${t("case.listFiles")}<span class="shortcut-key-badge" aria-hidden="true">L</span></button>
      </div>
    `;
  }
  if (tab === "reports") {
    return `
      <p class="section-label">${t("report.createTitle")}</p>
      <p class="field-hint">${t("report.hint")}</p>
      ${field(t("report.case"), `<select id="report-case" class="select" data-case-select data-allow-new-case="1">${caseSelectOptions(state.activeCase?.case_name, { allowNew: true })}</select>`)}
      ${field(t("report.title"), `<input id="report-title" class="input" value="${t("report.defaultTitle")}" />`)}
      ${field(t("report.format"), '<select id="report-format" class="select"><option value="txt">TXT</option><option value="json">JSON</option></select>')}
      ${field(t("report.signHash"), '<label class="checkbox-row" style="display:inline-flex; align-items:center; gap:8px; cursor:pointer;"><input id="report-sign-hash" type="checkbox" checked style="width:18px;height:18px;cursor:pointer;" /><span>' + t("report.signHashDesc") + "</span></label>")}
      <div class="button-row" style="margin: 16px 0;">
        <button class="primary-button" data-action="create-report" data-shortcut="G">${icon("report")} ${t("report.generate")}<span class="shortcut-key-badge" aria-hidden="true">G</span></button>
        <button class="secondary-button" data-action="list-reports" data-shortcut="R">${icon("refresh")} ${t("report.refresh")}<span class="shortcut-key-badge" aria-hidden="true">R</span></button>
      </div>
      <div class="status-badge" data-report-status style="display:none"></div>
      <div class="log-box" data-report-output style="margin-top: 14px;">${t("report.outputWaiting")}</div>
    `;
  }
  if (tab === "history") {
    const historyItems = Array.isArray(state?.acquisitionHistory) ? state.acquisitionHistory : [];
    return `
      <p class="section-label">${t("history.title")}</p>
      <p class="field-hint">${t("history.hint")}</p>
      <div class="side-info" style="margin: 12px 0 16px;">
        <span class="metric-icon">${icon("clock")}</span>
        <span><strong>${t("history.scope")}</strong><small>${escapeHtml(t("history.scopeAll"))}</small></span>
      </div>
      <div class="button-row" style="margin: 14px 0;">
        <button class="secondary-button" data-action="refresh-history" data-shortcut="R">${icon("refresh")} ${t("history.refresh")}<span class="shortcut-key-badge" aria-hidden="true">R</span></button>
      </div>
      <div class="history-list acquisition-history-list" data-history-list style="margin-top: 14px;">
        ${historyItems.length > 0 ? renderHistoryList(historyItems, escapeHtml, icon) : `<div class="log-box">${t("history.loading")}</div>`}
      </div>
    `;
  }
  if (tab === "logs") {
    return `
      <p class="section-label">${t("logs.title")}</p>
      <p class="field-hint">${t("logs.hint")}</p>
      <div class="side-info" style="margin: 12px 0 16px;">
        <span class="metric-icon">${icon("clock")}</span>
        <span><strong>${t("logs.scope")}</strong><small>${escapeHtml(state.activeCase?.case_name ? `${state.activeCase.case_name}/gunlukler` : t("logs.activeCaseOnly"))}</small></span>
      </div>
      <div class="button-row" style="margin: 14px 0;">
        <button class="secondary-button" data-action="refresh-logs" data-shortcut="R">${icon("refresh")} ${t("logs.refresh")}<span class="shortcut-key-badge" aria-hidden="true">R</span></button>
      </div>
      <div class="log-box" data-logs-output style="margin-top: 14px;">${t("logs.outputWaiting")}</div>
    `;
  }
  return hashPanel(pickerField, field, state, t, icon);
}

function renderHistoryList(items, escapeHtml, icon) {
  return items
    .map(
      (item) => `
    <div class="acquisition-history-item">
      <span class="metric-icon">${icon("clock")}</span>
      <div>
        <div class="history-title-row">
          <strong>${escapeHtml(item.mode || item.title || item.type || "Adli Edinim")}</strong>
          <small>${escapeHtml(item.timestamp || item.created_at || "")}</small>
        </div>
        <small class="path-text">${escapeHtml(item.path || item.output || item.case_name || "")}</small>
      </div>
    </div>
  `
    )
    .join("");
}

export function hashPanel(pickerField, field, state, t, icon) {
  const method = state?.hashMethod || "sha256";
  const path = state?.hashTargetInput || "";
  const result = state?.hashResult || null;

  return `
    <p class="section-label">${t("hash.title")}</p>
    <p class="field-hint">${t("hash.hint")}</p>
    ${pickerField(
      t("hash.targetFile"),
      "hash-target-path",
      path || "/path/to/evidence.raw",
      "file"
    )}
    ${field(
      t("hash.algorithm"),
      `<select id="hash-algorithm" class="select">
        <option value="sha256" ${method === "sha256" ? "selected" : ""}>SHA-256 (Önerilen / Recommended)</option>
        <option value="md5" ${method === "md5" ? "selected" : ""}>MD5</option>
        <option value="both" ${method === "both" ? "selected" : ""}>SHA-256 + MD5</option>
      </select>`
    )}
    <div class="button-row" style="margin: 16px 0;">
      <button class="primary-button" data-action="run-hash" data-shortcut="H">${icon("shield")} ${t("hash.calculate")}<span class="shortcut-key-badge" aria-hidden="true">H</span></button>
    </div>
    <div class="status-badge" data-hash-status style="display:none"></div>
    <div class="log-box" data-hash-output style="margin-top: 14px;">
      ${result ? renderHashResult(result, t) : t("hash.outputWaiting")}
    </div>
  `;
}

function renderHashResult(res, t) {
  let out = `<strong>${t("hash.file") || "Dosya"}:</strong> ${res.path}<br/><strong>${t("hash.size") || "Boyut"}:</strong> ${res.file_size_formatted || res.file_size + " B"}<br/>`;
  if (res.sha256)
    out += `<strong>SHA-256:</strong> <code style="word-break:break-all">${res.sha256}</code><br/>`;
  if (res.md5)
    out += `<strong>MD5:</strong> <code style="word-break:break-all">${res.md5}</code><br/>`;
  return out;
}

export function settingsPage({
  t,
  icon,
  state,
  platformLabel,
  APP_VERSION,
  pageTitle,
  escapeHtml,
}) {
  const isDark = state.theme !== "light";
  const esc = escapeHtml || ((s) => s);
  const isTr = (state.language || "en") === "tr";

  return `
    <section class="page settings-page">
      ${pageTitle ? pageTitle(t("settings.title"), "", "settings", icon) : ""}
      <div class="settings-stack">
        <!-- 1. Yatay Kart: Tercihler (Başlık / Kicker yok) -->
        <article class="settings-card settings-card-horizontal">
          <div class="settings-card-body">
            <!-- Karanlık Tema Satırı (Gündüz / Gece Efektli Toggle) -->
            <div class="settings-row">
              <div class="settings-row-label">
                <span class="settings-row-icon">${isDark ? icon("moon") : icon("sun")}</span>
                <span class="settings-row-title">${t("settings.darkTheme") || "Karanlık Tema"}</span>
              </div>
              <div class="settings-row-control">
                <button type="button" class="day-night-toggle ${isDark ? "is-dark" : "is-light"}" data-action="theme-toggle" data-shortcut="D" role="switch" aria-checked="${isDark ? "true" : "false"}" aria-label="${t("settings.darkTheme") || "Karanlık Tema"}">
                  <span class="shortcut-key-badge" aria-hidden="true">D</span>
                  <span class="dn-track">
                    <!-- Gece Yıldızları (Gece Modunda Görünür) -->
                    <span class="dn-stars">
                      <span class="dn-star dn-star-1"></span>
                      <span class="dn-star dn-star-2"></span>
                      <span class="dn-star dn-star-3"></span>
                      <span class="dn-star dn-star-4"></span>
                    </span>
                    <!-- Gündüz Bulutları (Gündüz Modunda Görünür) -->
                    <span class="dn-clouds">
                      <span class="dn-cloud dn-cloud-1"></span>
                      <span class="dn-cloud dn-cloud-2"></span>
                    </span>
                    <!-- Güneş / Ay Düğmesi (Sun / Moon Knob) -->
                    <span class="dn-knob">
                      <span class="dn-crater dn-crater-1"></span>
                      <span class="dn-crater dn-crater-2"></span>
                      <span class="dn-crater dn-crater-3"></span>
                    </span>
                  </span>
                </button>
              </div>
            </div>

            <!-- Dil Satırı -->
            <div class="settings-row">
              <div class="settings-row-label">
                <span class="settings-row-icon">${icon("globe")}</span>
                <span class="settings-row-title">${t("settings.language") || "Dil"}</span>
              </div>
              <div class="settings-row-control">
                <div class="flag-switch-group" role="radiogroup" aria-label="${t("settings.language")}">
                  <button type="button" class="flag-btn ${state.language === "tr" ? "is-active" : ""}" data-action="set-language" data-lang="tr" data-shortcut="T" aria-label="Türkçe" title="Türkçe">
                    <img src="./assets/flags/tr.svg" alt="Türkçe" class="flag-circle-img" draggable="false" />
                    <span class="shortcut-key-badge" aria-hidden="true">T</span>
                  </button>
                  <button type="button" class="flag-btn ${state.language === "en" ? "is-active" : ""}" data-action="set-language" data-lang="en" data-shortcut="E" aria-label="English" title="English">
                    <img src="./assets/flags/gb.svg" alt="English" class="flag-circle-img" draggable="false" />
                    <span class="shortcut-key-badge" aria-hidden="true">E</span>
                  </button>
                </div>
              </div>
            </div>

            <!-- Algılanan Sistem Satırı -->
            <div class="settings-row">
              <div class="settings-row-label">
                <span class="settings-row-icon">${icon("monitor")}</span>
                <span class="settings-row-title">${t("settings.detectedSystem") || "Algılanan Sistem"}</span>
              </div>
              <div class="settings-row-control">
                <span class="platform-badge">${icon(state.platform === "windows" ? "windows" : state.platform === "linux" ? "linux" : "monitor")} ${platformLabel(state.platform)}</span>
              </div>
            </div>

            ${
              state.platform === "linux"
                ? `
            <!-- Linux Render Motoru Satırı -->
            <div class="settings-row">
              <div class="settings-row-label">
                <span class="settings-row-icon engine-icon ${state.renderingEngine === "webkit" ? "is-webkit" : "is-chromium"}">${icon(state.renderingEngine === "webkit" ? "gauge" : "zap")}</span>
                <span class="settings-row-title">${t("settings.renderingEngine") || "Arayüz Motoru"}</span>
              </div>
              <div class="settings-row-control">
                <span class="engine-badge ${state.renderingEngine === "webkit" ? "is-webkit" : "is-chromium"}">
                  ${icon(state.renderingEngine === "webkit" ? "alert-circle" : "check")}
                  ${state.renderingEngine === "webkit" ? t("settings.engineWebKit") || "WebKitGTK" : t("settings.engineChromium") || "Chromium"}
                </span>
              </div>
            </div>
            ${
              state.renderingEngine === "webkit"
                ? `
            <div class="settings-engine-banner is-warning">
              <span class="settings-engine-banner-icon">${icon("alert-circle")}</span>
              <div class="settings-engine-banner-text">
                <strong>${t("settings.engineRecommendation") || "Öneri:"}</strong>
                <span>${t("settings.engineWebKitRecommend") || "Daha akıcı bir deneyim için sisteminize Chromium tabanlı bir tarayıcı kurmanızı öneririz."}</span>
              </div>
            </div>
            `
                : `
            <div class="settings-engine-banner is-success">
              <span class="settings-engine-banner-icon">${icon("check")}</span>
              <div class="settings-engine-banner-text">
                <span>${t("settings.engineChromiumDesc") || "Uygulamadan en iyi şekilde verim alıyorsunuz."}</span>
              </div>
            </div>
            `
            }
            `
                : ""
            }
          </div>
        </article>

        <!-- 2. Yatay Kart: Uygulama Bilgileri (Ortalanmış) -->
        <article class="settings-card settings-card-horizontal settings-card-centered">
          <h3 class="settings-centered-title">${t("settings.aboutApp") || "Uygulama Bilgileri"}</h3>

          <div class="settings-centered-info">
            <div class="settings-info-item">
              <span class="settings-info-label">${t("settings.versionNumber") || "Sürüm Numarası"}</span>
              <span class="settings-info-value version-highlight">${APP_VERSION}</span>
            </div>
            <div class="settings-info-divider"></div>
            <div class="settings-info-item">
              <span class="settings-info-label">${t("settings.license") || "Lisans"}</span>
              <span class="settings-info-value">GPL-3.0</span>
            </div>
          </div>

          <div class="settings-update-row-centered">
            <button type="button" class="settings-minimal-btn" data-action="check-update" data-shortcut="U">
              <span class="btn-icon">${icon("refresh")}</span>
              <span>${t("settings.checkUpdate") || "Güncellemeleri Denetle"}</span>
              <span class="shortcut-key-badge" aria-hidden="true">U</span>
            </button>
          </div>
          <div class="settings-update-status-row text-center">
            <span class="settings-update-status-text" data-update-status>${state.updateAvailable ? `<span class="status-icon-update">${icon("rocket")}</span> <span>${isTr ? "Yeni sürüm:" : "New version:"} <b>${esc(state.updateAvailable.latestTag)}</b></span>` : ""}</span>
          </div>

          <div class="settings-update-result text-center" data-update-result style="${state.updateAvailable ? "display: block;" : "display: none;"}">
            <button class="secondary-button" data-action="download-update" data-shortcut="I" style="${state.updateAvailable ? "display: inline-flex;" : "display: none;"} margin: 8px auto 0;">${icon("download")} ${t("settings.downloadInstall") || (isTr ? "İndir ve Kur" : "Download & Install")}<span class="shortcut-key-badge" aria-hidden="true">I</span></button>
          </div>
        </article>

        <!-- 3. En Altta Ayrı Div: Copyleft & noirLang Linki -->
        <div class="settings-copyright-bar">
          <span>Copyleft © 2026 <a href="https://noirlang.tr" target="_blank" rel="noopener noreferrer" class="noirlang-link">noirLang</a></span>
        </div>
      </div>
    </section>
  `;
}

export function getContributorFallbackPhoto(c, assetPath = "assets") {
  const id = (c?.id || "").toLowerCase();
  const name = (c?.name || "").toLowerCase();
  const photo = (c?.photo || "").toLowerCase();

  if (id.includes("melih") || name.includes("melih") || photo.includes("melih")) {
    return `${assetPath}/contributors/melih-emik.jpg`;
  }
  if (
    id.includes("guner") ||
    id.includes("m-ali") ||
    id.includes("kafkaskrtl") ||
    name.includes("muhammet") ||
    photo.includes("muhammet")
  ) {
    return `${assetPath}/contributors/muhammet-ali-guner.jpg`;
  }
  if (
    id.includes("toretto") ||
    id.includes("abdulhalim") ||
    id.includes("altuntas") ||
    name.includes("abdulhalim") ||
    photo.includes("abdulhalim")
  ) {
    return `${assetPath}/contributors/abdulhalim.jpg`;
  }
  if (id.includes("yusuf") || id.includes("tuncel") || name.includes("yusuf") || photo.includes("yusuf")) {
    return `${assetPath}/contributors/yusuf-tuncel.jpg`;
  }
  return `${assetPath}/contributors/melih-emik.jpg`;
}

export function renderContributors(contributors, t, icon, assetPath) {
  if (Array.isArray(contributors) && contributors.length > 0) {
    return contributors.map(c => {
      const roleText = c.roleKey ? t(c.roleKey) : (c.role || "Developer");
      const fallbackPhoto = getContributorFallbackPhoto(c, assetPath);
      const avatarSrc = c.photo ? (c.photo.startsWith("http") ? c.photo : `${assetPath}/contributors/${c.photo}`) : fallbackPhoto;
      return `
        <article class="contributor-card">
          <img class="avatar" src="${avatarSrc}" alt="${c.name}" draggable="false" onerror="this.onerror=null; this.src='${fallbackPhoto}'" />
          <h3>${c.name}</h3>
          <p>${roleText}</p>
          <div class="social-row" aria-label="${c.name} bağlantıları">
            ${(c.links || []).map(([label, url]) => socialLink(label, url, icon)).join("")}
          </div>
        </article>
      `;
    }).join("");
  }

  return `
    <div class="contributors-error-card" style="grid-column: 1 / -1; text-align: center; padding: 28px 20px; border: 1px dashed var(--border-color, #333); border-radius: 12px; background: rgba(255,255,255,0.02);">
      <div style="font-size: 28px; margin-bottom: 8px;">⚠️</div>
      <p style="margin: 0 0 14px; color: var(--text-muted, #aaa); font-size: 14px; line-height: 1.5;">
        ${t("about.contributorsFetchError")}
      </p>
      <a href="https://github.com/amele-next/amele-next/issues" target="_blank" rel="noopener" class="btn btn-secondary btn-sm" style="display: inline-flex; align-items: center; gap: 8px; text-decoration: none; padding: 8px 16px; border-radius: 8px; font-size: 13px;">
        ${icon("github")} <span>${t("about.reportIssue")}</span>
      </a>
    </div>
  `;
}

export function aboutPage({ t, icon, APP_VERSION, assetPath, theme, state }) {
  const logoFile = theme === "light" ? "logo-siyah.png" : "logo.png";
  const orbitShortcuts = { windows: "W", linux: "L", docker: "D", android: "A", ios: "I", other: "O" };
  return `
    <section class="page">
      <div class="about-hero">
        <div class="about-hero-left">
          <div class="about-hero-brand">
            <h1 class="about-hero-title">
              <span>Amele</span>
              <span>Forensic</span>
              <span>Tool</span>
            </h1>
            <div class="about-hero-version-row">
              <span class="about-hero-version">${APP_VERSION.startsWith("v") ? APP_VERSION : `v${APP_VERSION}`}</span>
              <button class="about-version-check-btn" data-action="about-check-update" data-shortcut="U" aria-label="${t("settings.checkUpdate") || "Güncellemeleri kontrol et"}" title="${t("settings.checkUpdate") || "Güncellemeleri kontrol et"}">
                <span class="about-version-check-icon">${icon("refresh")}</span>
                <span class="shortcut-key-badge" aria-hidden="true">U</span>
              </button>
            </div>
          </div>
        </div>

        <div class="about-hero-center">
          <img class="about-hero-logo" src="${assetPath}/logo/${logoFile}" alt="Amele logo" draggable="false" />
        </div>

        <div class="about-hero-right">
          <div class="about-radial-widget">
            <a class="about-radial-logo" href="https://noirlang.tr" target="_blank" rel="noopener noreferrer" aria-label="Noirlang">
              <img src="${assetPath}/logo/sirket.png" alt="Noirlang" draggable="false" />
            </a>
            <div class="about-radial-orbit">
              ${["windows", "linux", "docker", "android", "ios", "other"]
                .map(
                  (route, index) => `
                <div class="about-radial-position" style="--angle: ${index * 60}deg">
                  <div class="about-radial-upright">
                    <button type="button" class="about-radial-button" data-route="${route}" data-shortcut="${orbitShortcuts[route] || ""}" title="${t(`nav.${route}`)}" aria-label="${t(`nav.${route}`)}">
                      <span class="about-radial-icon" aria-hidden="true">${icon(route === "other" ? "tiles" : route)}</span>
                      <span class="shortcut-key-badge" aria-hidden="true">${orbitShortcuts[route] || ""}</span>
                    </button>
                  </div>
                </div>
              `
                )
                .join("")}
            </div>
          </div>
        </div>
      </div>

      <h2 class="section-heading">${t("about.maintainers")}</h2>
      <div class="contributor-grid">
        ${renderContributors(state?.contributors, t, icon, assetPath)}
      </div>

    </section>
  `;
}

export function socialLink(label, url, icon) {
  const l = (label || "").toLowerCase();
  let key = "link";
  if (l.includes("linkedin")) key = "linkedin";
  else if (l.includes("github")) key = "github";
  else if (l.includes("website") || l.includes("site") || l.includes("web")) key = "website";
  else if (l.includes("mail") || l.includes("email") || l.includes("posta")) key = "mail";
  else if (l.includes("gpg") || l.includes("pgp") || l.includes("key")) key = "key";
  else if (l.includes("twitter") || l.includes("x") || l.includes("terminal")) key = "terminal";

  const href = key === "mail" && !url.startsWith("mailto:") ? `mailto:${url}` : url;
  return `<a class="social-button" href="${href}" target="_blank" rel="noopener noreferrer" aria-label="${label}">${icon(key)}</a>`;
}
