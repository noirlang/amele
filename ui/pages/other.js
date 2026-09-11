export function otherPage({ t, icon, state, pageTitle, pickerField, field, escapeHtml, caseSelectOptions, detailPanel }) {
  return `
    <section class="page">
      ${pageTitle(t("other.title"), t("other.desc"), "tiles")}
      <div class="other-grid">
        ${simpleCard(t("other.hash.title"), t("other.hash.desc"), "shield", "hash", icon, t)}
        ${simpleCard(t("other.evidence.title"), t("other.evidence.desc"), "scale", "evidence", icon, t)}
        ${simpleCard(t("other.reports.title"), t("other.reports.desc"), "report", "reports", icon, t)}
        ${simpleCard(t("other.history.title"), t("other.history.desc"), "clock", "history", icon, t)}
        ${simpleCard(t("other.logs.title"), t("other.logs.desc"), "clock", "logs", icon, t)}
      </div>
      <div id="other-detail" class="workflow-panel" style="margin-top:16px">${detailPanel(state.activeTab)}</div>
    </section>
  `;
}

function simpleCard(title, desc, iconName, tab, icon, t) {
  return `
    <button class="forensic-card" data-tab="${tab}">
      <span class="card-icon">${icon(iconName)}</span>
      <h3>${title}</h3>
      <p>${desc}</p>
      <span class="meta">${t("open")}</span>
    </button>
  `;
}

export function detailPanel({ tab, t, icon, state, pickerField, field, escapeHtml, caseSelectOptions, hashPanel }) {
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
        <button class="primary-button" data-action="create-case">${icon("folder")} ${t("case.create")}</button>
        <button class="secondary-button" data-action="refresh-cases">${icon("refresh")} ${t("case.refresh")}</button>
        <button class="secondary-button" data-action="create-manifest">${icon("shield")} ${t("case.manifest.create")}</button>
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
        <button class="secondary-button" data-action="list-files">${icon("search")} ${t("case.listFiles")}</button>
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
        <button class="primary-button" data-action="create-report">${icon("report")} ${t("report.generate")}</button>
        <button class="secondary-button" data-action="list-reports">${icon("refresh")} ${t("report.refresh")}</button>
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
        <button class="secondary-button" data-action="refresh-history">${icon("refresh")} ${t("history.refresh")}</button>
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
        <button class="secondary-button" data-action="refresh-logs">${icon("refresh")} ${t("logs.refresh")}</button>
      </div>
      <div class="log-box" data-logs-output style="margin-top: 14px;">${t("logs.outputWaiting")}</div>
    `;
  }
  return hashPanel(pickerField, field, state, t, icon);
}

function renderHistoryList(items, escapeHtml, icon) {
  return items.map((item) => `
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
  `).join("");
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
      <button class="primary-button" data-action="run-hash">${icon("shield")} ${t("hash.calculate")}</button>
    </div>
    <div class="status-badge" data-hash-status style="display:none"></div>
    <div class="log-box" data-hash-output style="margin-top: 14px;">
      ${result ? renderHashResult(result, t) : t("hash.outputWaiting")}
    </div>
  `;
}

function renderHashResult(res, t) {
  let out = `<strong>${t("hash.file") || "Dosya"}:</strong> ${res.path}<br/><strong>${t("hash.size") || "Boyut"}:</strong> ${res.file_size_formatted || res.file_size + " B"}<br/>`;
  if (res.sha256) out += `<strong>SHA-256:</strong> <code style="word-break:break-all">${res.sha256}</code><br/>`;
  if (res.md5) out += `<strong>MD5:</strong> <code style="word-break:break-all">${res.md5}</code><br/>`;
  return out;
}

export function settingsPage({ t, icon, state, platformLabel, APP_VERSION, pageTitle }) {
  const isDark = state.theme !== "light";

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
                <button type="button" class="day-night-toggle ${isDark ? "is-dark" : "is-light"}" data-action="theme-toggle" role="switch" aria-checked="${isDark ? "true" : "false"}" aria-label="${t("settings.darkTheme") || "Karanlık Tema"}">
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
                  <button type="button" class="flag-btn ${state.language === "tr" ? "is-active" : ""}" data-action="set-language" data-lang="tr" aria-label="Türkçe" title="Türkçe">
                    <img src="./assets/flags/tr.svg" alt="Türkçe" class="flag-circle-img" draggable="false" />
                  </button>
                  <button type="button" class="flag-btn ${state.language === "en" ? "is-active" : ""}" data-action="set-language" data-lang="en" aria-label="English" title="English">
                    <img src="./assets/flags/gb.svg" alt="English" class="flag-circle-img" draggable="false" />
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

            ${state.platform === "linux" ? `
            <!-- Linux Render Motoru Satırı -->
            <div class="settings-row">
              <div class="settings-row-label">
                <span class="settings-row-icon engine-icon ${state.renderingEngine === "webkit" ? "is-webkit" : "is-chromium"}">${icon(state.renderingEngine === "webkit" ? "gauge" : "zap")}</span>
                <span class="settings-row-title">${t("settings.renderingEngine") || "Arayüz Motoru"}</span>
              </div>
              <div class="settings-row-control">
                <span class="engine-badge ${state.renderingEngine === "webkit" ? "is-webkit" : "is-chromium"}">
                  ${icon(state.renderingEngine === "webkit" ? "alert-circle" : "check")}
                  ${state.renderingEngine === "webkit" ? (t("settings.engineWebKit") || "WebKitGTK") : (t("settings.engineChromium") || "Chromium")}
                </span>
              </div>
            </div>
            ${state.renderingEngine === "webkit" ? `
            <div class="settings-engine-banner is-warning">
              <span class="settings-engine-banner-icon">${icon("alert-circle")}</span>
              <div class="settings-engine-banner-text">
                <strong>${t("settings.engineRecommendation") || "Öneri:"}</strong>
                <span>${t("settings.engineWebKitRecommend") || "Daha akıcı bir deneyim için sisteminize Chromium tabanlı bir tarayıcı kurmanızı öneririz."}</span>
              </div>
            </div>
            ` : `
            <div class="settings-engine-banner is-success">
              <span class="settings-engine-banner-icon">${icon("check")}</span>
              <div class="settings-engine-banner-text">
                <span>${t("settings.engineChromiumDesc") || "Uygulamadan en iyi şekilde verim alıyorsunuz."}</span>
              </div>
            </div>
            `}
            ` : ""}
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
            <button type="button" class="settings-minimal-btn" data-action="check-update">
              <span class="btn-icon">${icon("refresh")}</span>
              <span>${t("settings.checkUpdate") || "Güncellemeleri Denetle"}</span>
            </button>
          </div>
          <div class="settings-update-status-row text-center">
            <span class="settings-update-status-text" data-update-status></span>
          </div>

          <div class="settings-update-result text-center" data-update-result style="display: none;">
            <button class="secondary-button" data-action="download-update" style="display: none; margin: 8px auto 0;">${icon("download")} ${t("settings.downloadInstall")}</button>
          </div>
        </article>

        <!-- 3. En Altta Ayrı Div: Copyright & noirLang Linki -->
        <div class="settings-copyright-bar">
          <span>Copyright © 2026 <a href="https://noirlang.tr" target="_blank" rel="noopener noreferrer" class="noirlang-link">noirLang</a></span>
        </div>
      </div>
    </section>
  `;
}


export const KNOWN_CONTRIBUTORS = {
  melihemik: {
    key: "melihemik",
    name: "Melih Emik",
    roleKey: "about.role.lead",
    defaultRole: "BDFL",
    photo: "melih-emik.jpg",
    links: [
      ["GitHub", "https://github.com/melihemik"],
      ["LinkedIn", "https://www.linkedin.com/in/melihemik/"],
      ["Website", "https://melihemik.com.tr"]
    ]
  },
  yetece1: {
    key: "yetece1",
    name: "Yusuf Tuncel",
    roleKey: "about.role.windows",
    defaultRole: "Windows Sorumlusu",
    photo: "yusuf-tuncel.jpg",
    links: [
      ["GitHub", "https://github.com/yetece1"],
      ["LinkedIn", "https://www.linkedin.com/in/yusuf-tuncel/"],
      ["Website", "https://yusuftuncel.tr"]
    ]
  },
  kafkaskrtl: {
    key: "kafkaskrtl",
    name: "Muhammet Ali Güner",
    roleKey: "about.role.linux",
    defaultRole: "Linux Sorumlusu",
    photo: "muhammet-ali-guner.jpg",
    links: [
      ["GitHub", "https://github.com/kafkaskrtl"],
      ["LinkedIn", "https://www.linkedin.com/in/muhammetali-g%C3%BCner/"]
    ]
  },
  abdulhalimaltuntas: {
    key: "abdulhalimaltuntas",
    name: "Abdulhalim Altuntaş",
    roleKey: "about.role.android",
    defaultRole: "Android Sorumlusu",
    photo: "abdulhalim.jpg",
    links: [
      ["GitHub", "https://github.com/abdulhalimaltuntas"],
      ["LinkedIn", "https://www.linkedin.com/in/abdulhalim-altunta%C5%9F-7992672b5/"]
    ]
  }
};

export function renderContributors(contributors, t, icon, assetPath) {
  const list = (contributors && contributors.length > 0) ? contributors : [
    KNOWN_CONTRIBUTORS.melihemik,
    KNOWN_CONTRIBUTORS.yetece1,
    KNOWN_CONTRIBUTORS.kafkaskrtl,
    KNOWN_CONTRIBUTORS.abdulhalimaltuntas
  ];

  return list.map(c => {
    const roleText = c.roleKey ? t(c.roleKey) : (c.role || "Developer");
    const avatarSrc = c.photo ? (c.photo.startsWith("http") ? c.photo : `${assetPath}/contributors/${c.photo}`) : `${assetPath}/contributors/melih-emik.jpg`;
    return `
      <article class="contributor-card">
        <img class="avatar" src="${avatarSrc}" alt="${c.name}" draggable="false" onerror="this.src='${assetPath}/contributors/melih-emik.jpg'" />
        <h3>${c.name}</h3>
        <p>${roleText}</p>
        <div class="social-row" aria-label="${c.name} bağlantıları">
          ${(c.links || []).map(([label, url]) => socialLink(label, url, icon)).join("")}
        </div>
      </article>
    `;
  }).join("");
}

export function aboutPage({ t, icon, APP_VERSION, assetPath, theme, state }) {
  const logoFile = theme === "light" ? "logo-siyah.png" : "logo.png";
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
              <button class="about-version-check-btn" data-action="about-check-update" aria-label="${t("settings.checkUpdate") || "Güncellemeleri kontrol et"}" title="${t("settings.checkUpdate") || "Güncellemeleri kontrol et"}">
                <span class="about-version-check-icon">${icon("refresh")}</span>
              </button>
            </div>
          </div>
        </div>

        <div class="about-hero-center">
          <img class="about-hero-logo" src="${assetPath}/logo/${logoFile}" alt="Amele logo" draggable="false" />
        </div>

        <div class="about-hero-right">
          <div class="about-hero-features">
            <div class="about-feature-row">
              <div class="about-feature-chip" data-route="windows" title="${t("nav.windows")}" aria-label="${t("nav.windows")}">
                <span class="about-feature-icon">${icon("windows")}</span>
                <span class="about-feature-label">${t("nav.windows")}</span>
              </div>
              <div class="about-feature-chip" data-route="linux" title="${t("nav.linux")}" aria-label="${t("nav.linux")}">
                <span class="about-feature-icon">${icon("linux")}</span>
                <span class="about-feature-label">${t("nav.linux")}</span>
              </div>
            </div>
            <div class="about-feature-row">
              <div class="about-feature-chip" data-route="docker" title="${t("nav.docker")}" aria-label="${t("nav.docker")}">
                <span class="about-feature-icon">${icon("docker")}</span>
                <span class="about-feature-label">${t("nav.docker")}</span>
              </div>
              <div class="about-feature-chip" data-route="android" title="${t("nav.android")}" aria-label="${t("nav.android")}">
                <span class="about-feature-icon">${icon("android")}</span>
                <span class="about-feature-label">${t("nav.android")}</span>
              </div>
            </div>
            <div class="about-feature-row">
              <div class="about-feature-chip" data-route="ios" title="${t("nav.ios")}" aria-label="${t("nav.ios")}">
                <span class="about-feature-icon">${icon("ios")}</span>
                <span class="about-feature-label">${t("nav.ios")}</span>
              </div>
              <div class="about-feature-chip" data-route="other" title="${t("nav.other")}" aria-label="${t("nav.other")}">
                <span class="about-feature-icon">${icon("tiles")}</span>
                <span class="about-feature-label">${t("nav.other")}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <h2 class="section-heading">${t("about.maintainers")}</h2>
      <div class="contributor-grid">
        ${renderContributors(state?.contributors, t, icon, assetPath)}
      </div>

      <div class="company-logo-card">
        <img class="company-logo-img" src="${assetPath}/logo/sirket.png" alt="Şirket logosu" draggable="false" />
      </div>
    </section>
  `;
}

function capabilityCard(title, desc, iconName, accent, icon) {
  return `
    <article class="forensic-card" style="--accent:${accent};cursor:default">
      <span class="card-icon">${icon(iconName)}</span>
      <h3>${title}</h3>
      <p>${desc}</p>
    </article>
  `;
}

function socialLink(label, url, icon) {
  const key = label === "LinkedIn" ? "linkedin" : label === "Website" ? "website" : "github";
  return `<a class="social-button" href="${url}" target="_blank" rel="noopener noreferrer" aria-label="${label}">${icon(key)}</a>`;
}
