// arayüzün ana dosyası. sayfa geçişleri, sekmeler, tema dil falan hepsi burdan yönetiliyo. açılışta sistemi kontrol edip ona göre çiziyo.

import {
  androidModePage,
  androidPage,
  handleAndroidAction,
  syncAndroidDeviceSelection,
} from "./tools/android/index.js";
import { iosPage, handleIosAction, syncIosBackupPathInput } from "./tools/ios/index.js";
import { dockerPage, handleDockerAction } from "./tools/docker/index.js";
import { windowsPage } from "./tools/windows/index.js";
import { linuxPage } from "./tools/linux/index.js";
import { helpPage } from "./pages/help.js";
import { remoteAcqPage } from "./tools/remote-acq/index.js";
import { createApiRequest, fetchNewsAnnouncements } from "./core/api.js";
import { errorBoxHtml } from "./core/errors.js";
import { detectPlatform, platformLabel as platformName } from "./core/platform.js";
import { showToast } from "./core/toast.js";
import { canonicalRamFileName, compactLogLine, escapeHtml, formatBytes } from "./core/utils.js";
import { localText, toolCards, workflows } from "./core/workflows.js";
import { icon, hydrateIcons, fontIcons } from "./icons.js";
import { translate } from "./i18n.js";
import { homePage, metric, renderCaseSidebar } from "./pages/home.js";
import { renderReportSidebar } from "./pages/reportSidebar.js";
import { toolsPage } from "./pages/tools.js";
import { renderRadialNav, renderRadialWheelHtml } from "./core/radialNav.js";
import { initKeyboardShortcuts, syncShortcutBadges } from "./core/shortcuts.js";
import { otherPage, detailPanel, settingsPage, aboutPage, hashPanel } from "./pages/other.js";
import { workflowPage, pickerField, field, pageTitle, casePanel } from "./pages/workflow.js";
import { initDeveloperMode, devLog, toggleDevPanel } from "./developer.js";
import { initJobWidget } from "./core/jobs.js";
import { getPersistedCaseName, persistCaseName } from "./core/case.js";
import {
  initAgent,
  loadAgents,
  handleAgentChange,
  handleModelChange,
  handleScopeChange,
  submitAgentPrompt,
  executeAmeleCommand,
  getQuickChipPrompt,
  getPersistedAgentId,
} from "./core/agent.js";

const APP_VERSION = "v0.1.1";
const assetPath = "./assets";
const backendAvailable = location.protocol === "http:" || location.protocol === "https:";
const urlParams = new URLSearchParams(window.location.search);
const isDevConsole = urlParams.get("route") === "devlogs";
const isNativeWebView = urlParams.get("native") === "1";
const isNativeLinux =
  isNativeWebView && /linux/i.test(`${navigator.platform || ""} ${navigator.userAgent || ""}`);
if (isNativeWebView) document.documentElement.classList.add("native-webview");
if (isNativeLinux) document.documentElement.classList.add("native-linux");

const app = document.querySelector("#app");
const view = document.querySelector("#view");
const profileGate = document.querySelector("#profile-gate");
const preferredLanguage = ["tr", "en"].includes(urlParams.get("lang") || "")
  ? urlParams.get("lang")
  : localStorage.getItem("amele-language") || "en";
const requestedTheme = urlParams.get("theme");
const preferredTheme = ["dark", "light"].includes(requestedTheme || "")
  ? requestedTheme
  : localStorage.getItem("amele-theme") || "dark";
const preferredSidebarCollapsed = localStorage.getItem("amele-sidebar-collapsed") === "1";
if (preferredSidebarCollapsed) app.classList.add("sidebar-collapsed");

function safeJsonParse(value, fallback = null) {
  if (!value) return fallback;
  try {
    return JSON.parse(value) || fallback;
  } catch (_) {
    return fallback;
  }
}

function initialLogMessages(language) {
  return [translate(language, backendAvailable ? "log.appReady" : "log.previewMode")];
}

const state = {
  route: urlParams.get("route") || "home",
  isDevConsole,
  navMenu: { isOpen: false, isClosing: false, activeSubmenu: null },
  theme: preferredTheme,
  language: preferredLanguage,
  sidebarCollapsed: preferredSidebarCollapsed,
  platform: detectPlatform(),
  renderingEngine:
    urlParams.get("engine") ||
    (typeof window !== "undefined" &&
    window.chrome &&
    /Chrome|Chromium/i.test(navigator.userAgent || "")
      ? "chromium"
      : /WebKit/i.test(navigator.userAgent || "")
        ? "webkit"
        : "chromium"),
  news: safeJsonParse(localStorage.getItem("amele_news_cache"))?.items || [],
  activeNewsIndex: 0,
  files: {},
  activeTab: urlParams.get("tab") || "hash",
  approvedSecurityKey: "",
  remoteConnections: {},
  activeAcquisition: null,
  activeCase: null,
  pendingCaseName: "",
  cases: [],
  contributors: null,
  acquisitionHistory: [],
  caseBaseDir: "",
  profiles: [],
  activeProfile: null,
  mobileToolsAccess: { allowed: false, reason: "" },
  profileGateVisible: false,
  profileGateMode: "select",
  wizardStep: 1,
  wizardMode: "create",
  profileDraft: {
    fullName: "",
    username: "",
    usernameManual: false,
    onlineIdentifier: "",
    onlinePassword: "",
    language: preferredLanguage,
    theme: preferredTheme,
    openDirectly: false,
  },
  imageMount: null,
  imageMountLogHTML: "",
  imagePathInput: "",
  ramAnalysisPathInput: "",
  ramOsProfile: "windows",
  ramSymbolDirInput: "",
  latestUpdate: null,
  updateTarget: null,
  android: {
    adbStatus: null,
    devices: [],
    selectedDevice: "",
  },
  ios: {
    backupPath: "",
    profile: null,
    hashAlgorithms: ["md5", "sha1", "sha256"],
    normalizeJob: null,
    normalizeLog: [],
  },
  jobs: {},
  cachedDefaultCaseName: "",
  agent: {
    agents: [],
    selectedAgent: getPersistedAgentId() || "agy",
    selectedModel: "",
    selectedScope: "all",
    selectedMode: "ask",
    selectedOpt: "balance",
    promptDraft: "",
    messages: [],
    isGenerating: false,
    isExecuting: false,
    elevationModal: null,
  },
  reportDraft: {
    title: "",
    description: "",
    images: [],
    submitting: false,
    error: "",
    success: "",
  },
  copilot: null,
  lastLog: initialLogMessages(preferredLanguage),
};
state.copilot = state.agent;

function t(key, vars = {}) {
  return translate(state.language, key, vars);
}

const apiRequest = createApiRequest({ backendAvailable });
const localizeText = (value) => localText(value, state.language);

function boundDetailPanel(tab) {
  return detailPanel({
    tab,
    t,
    icon,
    state,
    pickerField: boundPickerField,
    field,
    escapeHtml,
    caseSelectOptions,
    hashPanel: (picker, f, st, trans, ico, esc) =>
      hashPanel(picker, f, st, trans, ico, esc || escapeHtml),
  });
}

function boundPickerField(label, id, value, type = "file") {
  return pickerField(label, id, value, type, icon, t);
}

const MAIN_ROUTE_ORDER = [
  "home",
  "windows",
  "linux",
  "docker",
  "android",
  "ios",
  "other",
  "help",
  "about",
  "settings",
  "profile",
];

function setRoute(route, direction) {
  if (isMobileToolsRoute(route) && !onlineMobileToolsAllowed()) {
    devLog(
      "WARN",
      "ui:router",
      `Mobile route blocked (locked): ${route}`,
      apiRequest,
      backendReady
    );
    return;
  }
  if (route.startsWith("workflow:")) {
    const workflow = workflows[route.split(":")[1]];
    if (workflow && isLocalWorkflowBlocked(workflow)) {
      devLog(
        "WARN",
        "ui:router",
        `Route blocked (platform mismatch): ${route} — expected ${workflow.platform}, got ${state.platform}`,
        apiRequest,
        backendReady
      );
      showToast(t("platformBlocked", { platform: workflow.platform }), "warning");
      return;
    }
  }

  const prevRoute = state.route || "home";

  if (prevRoute !== route && !state.activeAcquisition) {
    state.lastLog = initialLogMessages(state.language);
  }

  if (direction) {
    state.navDirection = direction;
  } else if (
    (route.startsWith("workflow:") && !prevRoute.startsWith("workflow:")) ||
    (route.startsWith("android:") && !prevRoute.startsWith("android:"))
  ) {
    state.navDirection = "forward";
  } else if (
    (!route.startsWith("workflow:") && prevRoute.startsWith("workflow:")) ||
    (!route.startsWith("android:") && prevRoute.startsWith("android:"))
  ) {
    state.navDirection = "back";
  } else {
    const prevGroup = routeGroup(prevRoute);
    const nextGroup = routeGroup(route);
    const prevIdx = MAIN_ROUTE_ORDER.indexOf(prevGroup);
    const nextIdx = MAIN_ROUTE_ORDER.indexOf(nextGroup);
    if (prevIdx !== -1 && nextIdx !== -1 && prevIdx !== nextIdx) {
      state.navDirection = nextIdx > prevIdx ? "forward" : "back";
    } else if (prevRoute === route) {
      state.navDirection = "refresh";
    } else {
      state.navDirection = "standard";
    }
  }

  devLog(
    "DEBUG",
    "ui:router",
    `Navigate → ${route} (${state.navDirection})`,
    apiRequest,
    backendReady
  );
  state.route = route;
  render();
}

function isLocalWorkflowBlocked(workflow) {
  if (!workflow.mode.startsWith("local")) return false;
  return workflow.platform.toLowerCase() !== state.platform;
}

function setTheme(theme) {
  const nextTheme = theme === "light" ? "light" : "dark";
  state.theme = nextTheme;
  localStorage.setItem("amele-theme", nextTheme);
  app.classList.toggle("theme-light", nextTheme === "light");
  app.classList.toggle("theme-dark", nextTheme !== "light");
  syncBrandLogo();
}

function setLanguage(language) {
  const nextLanguage = language === "en" ? "en" : "tr";
  state.language = nextLanguage;
  localStorage.setItem("amele-language", nextLanguage);
  document.documentElement.lang = nextLanguage;
  document.querySelectorAll("[data-i18n]").forEach((node) => {
    node.textContent = t(node.dataset.i18n);
  });
  syncProfileButton();
}

function syncBrandLogo() {
  const isLight = state.theme === "light";
  const logoPath = isLight ? "./assets/logo/logo-siyah.png" : "./assets/logo/logo.png";
  const brandImg = document.querySelector("#brand-logo-img");
  if (brandImg) {
    brandImg.src = logoPath;
  }
  const aboutLogo = document.querySelector(".about-hero-logo");
  if (aboutLogo) {
    aboutLogo.src = logoPath;
  }
  document.querySelectorAll(".nav-center-logo").forEach((img) => {
    img.src = logoPath;
  });
}

function syncSidebarState() {
  app.classList.toggle("sidebar-collapsed", state.sidebarCollapsed);
  document.documentElement?.classList?.toggle?.("sidebar-collapsed", state.sidebarCollapsed);
  document.querySelectorAll("[data-sidebar-toggle]").forEach((button) => {
    button.setAttribute("aria-expanded", String(!state.sidebarCollapsed));
    button.setAttribute(
      "aria-label",
      state.sidebarCollapsed ? t("sidebar.expand") : t("sidebar.collapse")
    );
  });
}

function setSidebarCollapsed(collapsed) {
  state.sidebarCollapsed = Boolean(collapsed);
  try {
    localStorage.setItem("amele-sidebar-collapsed", state.sidebarCollapsed ? "1" : "0");
  } catch (_) {}
  if (state.activeProfile) {
    state.activeProfile.sidebarCollapsed = state.sidebarCollapsed;
    upsertProfile(state.activeProfile);
  }
  syncSidebarState();
}

function applyPersistedSettings(settings) {
  if (!settings || typeof settings !== "object") return;
  if (!urlParams.get("lang")) setLanguage(settings.dil === "en" ? "en" : "tr");
  if (!urlParams.get("theme")) setTheme(settings.karanlik_tema ? "dark" : "light");
}

async function loadPersistedSettings() {
  if (!backendReady()) return;
  try {
    const result = await apiRequest("/api/settings");
    applyPersistedSettings(result.settings);
    devLog(
      "INFO",
      "ui:settings",
      `Kalıcı ayarlar yüklendi: ${result.path || "(path yok)"}`,
      apiRequest,
      backendReady
    );
  } catch (error) {
    devLog(
      "WARN",
      "ui:settings",
      `Kalıcı ayarlar yüklenemedi: ${error.message}`,
      apiRequest,
      backendReady
    );
  }
}

async function loadUpdateTarget() {
  if (!backendReady()) return;
  try {
    state.updateTarget = await apiRequest("/api/update-target");
    devLog(
      "INFO",
      "ui:update",
      `Güncelleme paket tipi algılandı: ${state.updateTarget.package_label || state.updateTarget.package_kind || "-"}`,
      apiRequest,
      backendReady
    );
  } catch (error) {
    devLog(
      "WARN",
      "ui:update",
      `Güncelleme paket tipi algılanamadı: ${error.message}`,
      apiRequest,
      backendReady
    );
  }
}

async function saveSettingsFromControls() {
  const language =
    document.querySelector("[data-action='language-select']")?.value || state.language;
  setLanguage(language);
  setTheme(state.theme);
  if (!backendReady()) return { path: "localStorage" };

  const result = await apiRequest("/api/settings", {
    method: "POST",
    body: JSON.stringify({
      theme: state.theme,
      language: state.language,
    }),
  });
  applyPersistedSettings(result.settings);
  if (state.activeProfile) {
    state.activeProfile.language = state.language;
    state.activeProfile.theme = state.theme;
    upsertProfile(state.activeProfile);
    syncProfileButton();
  }
  return result;
}

async function persistSettingsFromControls() {
  const result = await saveSettingsFromControls();
  render();
  const savedPath = result?.path ? `<br /><small>${escapeHtml(result.path)}</small>` : "";
  setStatus("[data-settings-status]", `${icon("info")} ${t("settingsSaved")}${savedPath}`);
  showToast(t("settingsSaved"));
  return result;
}

async function loadProfiles() {
  if (!backendReady()) {
    hideProfileGate();
    return;
  }
  try {
    const result = await apiRequest("/api/profiles");
    state.profiles = Array.isArray(result.profiles) ? result.profiles : [];
    state.activeProfile = result.active_profile || null;
    if (state.activeProfile) {
      if (!urlParams.get("lang")) setLanguage(state.activeProfile.language === "en" ? "en" : "tr");
      if (!urlParams.get("theme"))
        setTheme(state.activeProfile.theme === "light" ? "light" : "dark");
      if (typeof state.activeProfile.sidebarCollapsed === "boolean") {
        setSidebarCollapsed(state.activeProfile.sidebarCollapsed);
      }
      state.mobileToolsAccess = cachedMobileToolsAccess(state.activeProfile);
      hideProfileGate();
    } else {
      state.mobileToolsAccess = { allowed: false, reason: "" };
      state.profileGateMode = state.profiles.length ? "select" : "wizard";
      state.wizardStep = 1;
      state.wizardMode = "create";
      showProfileGate();
    }
    syncProfileButton();
  } catch (error) {
    devLog(
      "ERROR",
      "ui:profile",
      `Profil listesi yüklenemedi: ${error.message}`,
      apiRequest,
      backendReady
    );
    state.profileGateMode = "wizard";
    state.wizardStep = 1;
    state.wizardMode = "create";
    showProfileGate(t("profile.loadFailed", { message: error.message }));
  }
}

function showProfileGate(errorMessage = "") {
  state.profileGateVisible = true;
  renderProfileGate(errorMessage);
}

function hideProfileGate() {
  state.profileGateVisible = false;
  if (profileGate) {
    profileGate.hidden = true;
    profileGate.innerHTML = "";
  }
}

function slugifyUsername(fullName) {
  if (!fullName) return "";
  const trMap = {
    ç: "c",
    ğ: "g",
    ı: "i",
    ö: "o",
    ş: "s",
    ü: "u",
    Ç: "c",
    Ğ: "g",
    İ: "i",
    I: "i",
    Ö: "o",
    Ş: "s",
    Ü: "u",
  };
  const normalized = String(fullName).replace(/[çğışüöÇĞİŞÜÖ]/g, (c) => trMap[c] || c);
  return normalized
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 32);
}

function renderProfileGate(errorMessage = "") {
  if (!profileGate) return;
  profileGate.hidden = false;

  if (state.profileGateMode === "select" && state.profiles.length > 0) {
    renderProfileSelectView(errorMessage);
  } else {
    state.profileGateMode = "wizard";
    renderProfileWizardView(errorMessage);
  }
  hydrateIcons(profileGate);
  attachProfileGateEvents();
}

function renderProfileSelectView(errorMessage = "") {
  const count = state.profiles.length;
  const countClass =
    count === 1
      ? "profile-cards-1"
      : count === 2
        ? "profile-cards-2"
        : count === 3
          ? "profile-cards-3"
          : "";
  const cards = state.profiles
    .map(
      (profile) => `
    <button class="profile-select-card" data-action="profile-select" data-username="${escapeHtml(profile.username)}">
      <div class="profile-select-card-avatar">
        ${renderProfileAvatar(profile, "large")}
        ${profile.online ? `<span class="profile-online-badge" title="${escapeHtml(t("profile.onlineConnectedShort"))}">${icon("globe")}</span>` : ""}
      </div>
      <div class="profile-select-card-meta">
        <strong class="profile-select-card-name">${escapeHtml(profile.full_name || profile.username)}</strong>
        <span class="profile-select-card-user">@${escapeHtml(profile.username)}</span>
        ${profile.online ? `<span class="profile-select-card-status">${icon("globe")} ${profile.online.status === "offline" ? "Offline" : profile.online.status === "session_expired" ? "Offline (Oturum Doldu)" : t("profile.onlineAccount")} @${escapeHtml(profile.online.username || "-")}</span>` : ""}
      </div>
    </button>
  `
    )
    .join("");

  profileGate.innerHTML = `
    <section class="profile-select-container">
      <div class="profile-select-top">
        <img src="./assets/logo/${state.theme === "light" ? "logo-siyah.png" : "logo.png"}" alt="Amele" class="profile-select-logo" />
        <h1 class="profile-select-title">${t("profile.selectTitle")}</h1>
        <p class="profile-select-desc">${t("profile.selectDesc")}</p>
      </div>
      ${errorMessage ? `<div class="error-panel">${escapeHtml(errorMessage)}</div>` : ""}
      <div class="profile-cards-grid ${countClass}">${cards}</div>
      <div class="profile-select-bottom">
        <label class="check-row profile-gate-check">
          <input type="checkbox" id="profile-open-directly" />
          <span>${t("profile.openDirectly")}</span>
        </label>
        <div class="profile-gate-actions">
          <button class="primary-button profile-action-btn" data-action="profile-create-start">
            ${icon("user")} ${t("profile.newProfile")}
          </button>
          <button class="secondary-button profile-action-btn" data-action="profile-online-start">
            ${icon("globe")} ${t("profile.onlineConnect")}
          </button>
        </div>
      </div>
    </section>
  `;
}

function renderProfileWizardView(errorMessage = "") {
  const isOnlineMode = state.wizardMode === "online";
  const step = state.wizardStep || 1;
  const draft = state.profileDraft || {};
  const currentLang = draft.language || state.language || "tr";
  const currentTheme = draft.theme || state.theme || "dark";
  const isDark = currentTheme !== "light";

  let bodyHtml = "";

  if (step === 1) {
    // 1. Dil Seçimi: Solda İngilizce, Sağda Türkçe
    bodyHtml = `
      <h1 class="wizard-title">${t("wizard.selectLanguage")}</h1>
      <p class="wizard-subtitle">${t("wizard.selectLanguageDesc")}</p>
      ${errorMessage ? `<div class="error-panel">${escapeHtml(errorMessage)}</div>` : ""}
      <div class="wizard-choice-group wizard-language-group">
        <button type="button" class="wizard-choice-card wizard-lang-card ${currentLang === "en" ? "is-selected" : ""}" data-action="wizard-set-lang" data-lang="en">
          <img src="./assets/flags/gb.svg" alt="English" class="wizard-flag-img" />
          <span class="wizard-choice-label">English</span>
        </button>
        <button type="button" class="wizard-choice-card wizard-lang-card ${currentLang === "tr" ? "is-selected" : ""}" data-action="wizard-set-lang" data-lang="tr">
          <img src="./assets/flags/tr.svg" alt="Türkçe" class="wizard-flag-img" />
          <span class="wizard-choice-label">Türkçe</span>
        </button>
      </div>
      <div class="wizard-bottom">
        <button type="button" class="wizard-btn-next" data-action="wizard-step-next">
          ${t("wizard.next")}
        </button>
        ${
          state.profiles.length
            ? `
          <button type="button" class="wizard-btn-back" data-action="profile-select-back">
            ${t("wizard.back")}
          </button>
        `
            : ""
        }
      </div>
    `;
  } else if (step === 2) {
    // 2. Tema Seçimi: SADECE ayarlardaki animasyonlu day-night-toggle (ekstra kutular yok)
    bodyHtml = `
      <h1 class="wizard-title">${t("wizard.selectTheme")}</h1>
      <p class="wizard-subtitle">${t("wizard.selectThemeDesc")}</p>
      ${errorMessage ? `<div class="error-panel">${escapeHtml(errorMessage)}</div>` : ""}
      <div class="wizard-theme-container">
        <div class="wizard-dn-toggle-wrap">
          <button type="button" class="day-night-toggle ${isDark ? "is-dark" : "is-light"} wizard-day-night" data-action="wizard-theme-toggle" role="switch" aria-checked="${isDark ? "true" : "false"}" aria-label="${t("settings.darkTheme") || "Karanlık Tema"}">
            <span class="dn-track">
              <span class="dn-stars">
                <span class="dn-star dn-star-1"></span>
                <span class="dn-star dn-star-2"></span>
                <span class="dn-star dn-star-3"></span>
                <span class="dn-star dn-star-4"></span>
              </span>
              <span class="dn-clouds">
                <span class="dn-cloud dn-cloud-1"></span>
                <span class="dn-cloud dn-cloud-2"></span>
              </span>
              <span class="dn-knob">
                <span class="dn-crater dn-crater-1"></span>
                <span class="dn-crater dn-crater-2"></span>
                <span class="dn-crater dn-crater-3"></span>
              </span>
            </span>
          </button>
        </div>
      </div>
      <div class="wizard-bottom">
        <button type="button" class="wizard-btn-next" data-action="wizard-step-next">
          ${t("wizard.next")}
        </button>
        <button type="button" class="wizard-btn-back" data-action="wizard-step-prev">
          ${t("wizard.back")}
        </button>
      </div>
    `;
  } else if (step === 3) {
    // 3. SADECE Ad Soyad (Kullanıcı adı ve checkbox yok)
    bodyHtml = `
      <h1 class="wizard-title">${t("wizard.fullNameTitle")}</h1>
      <p class="wizard-subtitle">${t("wizard.fullNameDesc")}</p>
      ${errorMessage ? `<div class="error-panel">${escapeHtml(errorMessage)}</div>` : ""}
      <div class="wizard-input-center-wrap">
        <input id="wizard-full-name" class="wizard-input-large" type="text" autocomplete="name" placeholder="${t("profile.fullName")}" value="${escapeHtml(draft.fullName || "")}" autofocus />
      </div>
      <div class="wizard-bottom">
        <button type="button" class="wizard-btn-next" data-action="wizard-step-next">
          ${t("wizard.next")}
        </button>
        <button type="button" class="wizard-btn-back" data-action="wizard-step-prev">
          ${t("wizard.back")}
        </button>
      </div>
    `;
  } else if (step === 4) {
    // 4. Online Profil Bağlamak İster misiniz? (Evet / Hayır)
    bodyHtml = `
      <h1 class="wizard-title">${t("wizard.onlinePromptTitle")}</h1>
      <p class="wizard-subtitle">${t("wizard.onlinePromptDesc")}</p>
      ${errorMessage ? `<div class="error-panel">${escapeHtml(errorMessage)}</div>` : ""}
      <div class="wizard-choice-group wizard-online-prompt-group">
        <button type="button" class="wizard-choice-card wizard-prompt-card" data-action="wizard-online-choice-yes">
          <span class="wizard-prompt-icon">${icon("globe")}</span>
          <strong class="wizard-choice-label">${t("wizard.yesOnline")}</strong>
          <small class="wizard-choice-desc">${t("wizard.yesOnlineDesc")}</small>
        </button>
        <button type="button" class="wizard-choice-card wizard-prompt-card" data-action="wizard-online-choice-no">
          <span class="wizard-prompt-icon">${icon("user")}</span>
          <strong class="wizard-choice-label">${t("wizard.noLocal")}</strong>
          <small class="wizard-choice-desc">${t("wizard.noLocalDesc")}</small>
        </button>
      </div>
      <div class="wizard-bottom">
        <button type="button" class="wizard-btn-back" data-action="wizard-step-prev">
          ${t("wizard.back")}
        </button>
      </div>
    `;
  } else if (step === 5) {
    // 5. Yerel Akış: Kullanıcı Adı Belirleme ve En Sonda "Bundan sonra direkt bu hesapla aç"
    const suggestedUsername = draft.username || slugifyUsername(draft.fullName);
    bodyHtml = `
      <h1 class="wizard-title">${t("wizard.usernameTitle")}</h1>
      <p class="wizard-subtitle">${t("wizard.usernameDesc")}</p>
      ${errorMessage ? `<div class="error-panel">${escapeHtml(errorMessage)}</div>` : ""}
      <div class="wizard-input-center-wrap">
        <div class="wizard-username-large-box">
          <span class="wizard-at-large">@</span>
          <input id="wizard-username" class="wizard-username-large-input" type="text" autocomplete="username" placeholder="${t("profile.username")}" value="${escapeHtml(suggestedUsername)}" autofocus />
        </div>
        <label class="check-row wizard-check">
          <input type="checkbox" id="wizard-open-directly" ${draft.openDirectly ? "checked" : ""} />
          <span>${t("profile.openDirectly")}</span>
        </label>
      </div>
      <div class="wizard-bottom">
        <button type="button" class="wizard-btn-next" data-action="wizard-finish-local">
          ${t("wizard.createAndStart")}
        </button>
        <button type="button" class="wizard-btn-back" data-action="wizard-step-prev">
          ${t("wizard.back")}
        </button>
      </div>
    `;
  } else if (step === 6) {
    // 6. Online Akış: Online Kullanıcı Adı veya E-posta
    bodyHtml = `
      <h1 class="wizard-title">${t("wizard.onlineIdentifierTitle")}</h1>
      <p class="wizard-subtitle">${t("wizard.onlineIdentifierDesc")}</p>
      ${errorMessage ? `<div class="error-panel">${escapeHtml(errorMessage)}</div>` : ""}
      <div class="wizard-input-center-wrap">
        <input id="wizard-online-identifier" class="wizard-input-large" type="text" autocomplete="username" placeholder="${t("profile.onlineIdentifier")}" value="${escapeHtml(draft.onlineIdentifier || "")}" autofocus />
      </div>
      <div class="wizard-bottom">
        <button type="button" class="wizard-btn-next" data-action="wizard-step-next">
          ${t("wizard.next")}
        </button>
        <button type="button" class="wizard-btn-back" data-action="wizard-step-prev">
          ${t("wizard.back")}
        </button>
      </div>
    `;
  } else if (step === 7) {
    // 7. Online Akış: Şifre ve En Sonda "Bundan sonra direkt bu hesapla aç"
    bodyHtml = `
      <h1 class="wizard-title">${t("wizard.onlinePasswordTitle")}</h1>
      <p class="wizard-subtitle">${t("wizard.onlinePasswordDesc")}</p>
      ${errorMessage ? `<div class="error-panel">${escapeHtml(errorMessage)}</div>` : ""}
      <div class="wizard-input-center-wrap">
        <div class="wizard-pwd-container">
          <input id="wizard-online-password" class="wizard-input-large" type="password" autocomplete="current-password" placeholder="••••••••" value="${escapeHtml(draft.onlinePassword || "")}" autofocus />
          <button type="button" class="wizard-pwd-eye" data-action="wizard-toggle-pwd" aria-label="Toggle">${icon("eye")}</button>
        </div>
        <label class="check-row wizard-check">
          <input type="checkbox" id="wizard-online-directly" ${draft.openDirectly ? "checked" : ""} />
          <span>${t("profile.openDirectly")}</span>
        </label>
      </div>
      <div class="wizard-bottom">
        <button type="button" class="wizard-btn-next" data-action="wizard-online-finish">
          ${t("wizard.finish")}
        </button>
        <button type="button" class="wizard-btn-back" data-action="wizard-step-prev">
          ${t("wizard.back")}
        </button>
      </div>
    `;
  }

  profileGate.innerHTML = `
    <section class="profile-wizard-container">
      <div class="wizard-top">
        <img src="./assets/logo/${state.theme === "light" ? "logo-siyah.png" : "logo.png"}" alt="Amele" class="wizard-logo" />
      </div>
      ${bodyHtml}
    </section>
  `;
}

function attachProfileGateEvents() {
  if (!profileGate) return;
  const fullNameInput = profileGate.querySelector("#wizard-full-name");
  if (fullNameInput) {
    fullNameInput.addEventListener("input", (e) => {
      state.profileDraft.fullName = e.target.value;
      if (!state.profileDraft?.usernameManual) {
        state.profileDraft.username = slugifyUsername(e.target.value);
      }
    });
    fullNameInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        captureProfileDraft();
        if (!state.profileDraft.fullName) {
          showProfileGate(t("profile.required"));
          return;
        }
        if (!state.profileDraft.username) {
          state.profileDraft.username = slugifyUsername(state.profileDraft.fullName) || "kullanici";
        }
        state.wizardStep = 4;
        renderProfileGate();
      }
    });
  }
  const usernameInput = profileGate.querySelector("#wizard-username");
  if (usernameInput) {
    usernameInput.addEventListener("input", (e) => {
      state.profileDraft.usernameManual = true;
      state.profileDraft.username = e.target.value;
    });
    usernameInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        createLocalProfileFromWizard();
      }
    });
  }
  const identInput = profileGate.querySelector("#wizard-online-identifier");
  if (identInput) {
    identInput.addEventListener("input", (e) => {
      state.profileDraft.onlineIdentifier = e.target.value;
    });
    identInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        captureProfileDraft();
        if (!state.profileDraft.onlineIdentifier) {
          showProfileGate(t("profile.onlineRequired"));
          return;
        }
        state.wizardStep = 7;
        renderProfileGate();
      }
    });
  }
  const pwdInput = profileGate.querySelector("#wizard-online-password");
  if (pwdInput) {
    pwdInput.addEventListener("input", (e) => {
      state.profileDraft.onlinePassword = e.target.value;
    });
    pwdInput.addEventListener("keydown", (e) => {
      if (e.key === "Enter") {
        e.preventDefault();
        connectOnlineProfileFromWizard();
      }
    });
  }
  const directInput =
    profileGate.querySelector("#wizard-open-directly") ||
    profileGate.querySelector("#wizard-online-directly");
  if (directInput) {
    directInput.addEventListener("change", (e) => {
      state.profileDraft.openDirectly = e.target.checked;
    });
  }
}

function captureProfileDraft() {
  const fullName =
    document.querySelector("#wizard-full-name") || document.querySelector("#profile-full-name");
  const username =
    document.querySelector("#wizard-username") || document.querySelector("#profile-username");
  const onlineIdentifier =
    document.querySelector("#wizard-online-identifier") ||
    document.querySelector("#profile-online-identifier");
  const onlinePassword =
    document.querySelector("#wizard-online-password") ||
    document.querySelector("#profile-online-password");
  const direct =
    document.querySelector("#wizard-open-directly") ||
    document.querySelector("#profile-create-directly");
  const onlineDirect =
    document.querySelector("#wizard-online-directly") ||
    document.querySelector("#profile-online-directly");

  state.profileDraft = {
    fullName: fullName ? fullName.value.trim() : state.profileDraft?.fullName || "",
    username: username ? username.value.trim() : state.profileDraft?.username || "",
    usernameManual: Boolean(state.profileDraft?.usernameManual),
    onlineIdentifier: onlineIdentifier
      ? onlineIdentifier.value.trim()
      : state.profileDraft?.onlineIdentifier || "",
    onlinePassword: onlinePassword
      ? onlinePassword.value
      : state.profileDraft?.onlinePassword || "",
    language: state.profileDraft?.language || state.language || "tr",
    theme: state.profileDraft?.theme || state.theme || "dark",
    openDirectly: direct
      ? direct.checked
      : onlineDirect
        ? onlineDirect.checked
        : Boolean(state.profileDraft?.openDirectly),
  };
}

function profileInitials(profile) {
  const source = profile.full_name || profile.username || "A";
  return (
    source
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part[0]?.toLocaleUpperCase(state.language === "tr" ? "tr-TR" : "en-US") || "")
      .join("") || "A"
  );
}

function getAvatarUrl(profile) {
  if (!profile) return "";
  let rawUrl =
    profile.avatar_url ||
    profile.avatarUrl ||
    profile.online?.avatar_url ||
    profile.online?.avatarUrl ||
    profile.avatar ||
    profile.online?.avatar ||
    "";
  if (!rawUrl) return "";
  rawUrl = rawUrl.replace("://www.amele.noirlang.tr", "://amele.noirlang.tr");
  if (rawUrl.startsWith("http://") || rawUrl.startsWith("https://") || rawUrl.startsWith("data:")) {
    return rawUrl;
  }
  if (rawUrl.startsWith("/")) {
    let base =
      profile.online?.apiBase ||
      profile.online?.api_base ||
      profile.apiBase ||
      profile.api_base ||
      "https://amele.noirlang.tr";
    base = base.replace("://www.amele.noirlang.tr", "://amele.noirlang.tr");
    return `${base.replace(/\/+$/, "")}${rawUrl}`;
  }
  return rawUrl;
}

function renderProfileAvatar(profile, extraClass = "") {
  const avatarUrl = getAvatarUrl(profile);
  const initials = profileInitials(profile || {});
  const sizeClass = extraClass ? ` ${extraClass}` : "";
  if (avatarUrl) {
    let rawPath =
      profile.avatar_url ||
      profile.avatarUrl ||
      profile.online?.avatar_url ||
      profile.online?.avatarUrl ||
      profile.avatar ||
      profile.online?.avatar ||
      "";
    rawPath = rawPath.replace("://www.amele.noirlang.tr", "://amele.noirlang.tr");
    let altBase = "";
    if (rawPath.startsWith("/")) {
      altBase = `https://aamele-noirlang-tr.onrender.com${rawPath}`;
    } else if (rawPath.includes("amele.noirlang.tr/api/images/")) {
      altBase = rawPath.replace("://amele.noirlang.tr", "://aamele-noirlang-tr.onrender.com");
    }
    return `<span class="profile-avatar${sizeClass}"><img src="${escapeHtml(avatarUrl)}" alt="Avatar" class="avatar-img" data-alt-src="${escapeHtml(altBase)}" onerror="if(this.dataset.altSrc && this.src !== this.dataset.altSrc){ this.src = this.dataset.altSrc; this.dataset.altSrc = ''; } else { this.style.display='none'; if(this.nextElementSibling) this.nextElementSibling.style.display='inline-grid'; }" /><span class="avatar-fallback" style="display:none;">${initials}</span></span>`;
  }
  return `<span class="profile-avatar${sizeClass}"><span class="avatar-fallback">${initials}</span></span>`;
}

function syncProfileButton() {
  const label = document.querySelector("[data-profile-label]");
  if (label) {
    const name = state.activeProfile?.display_name || t("profile.button");
    const online = state.activeProfile?.online;
    const isOffline =
      online &&
      (online.status === "offline" ||
        online.status === "session_expired" ||
        (typeof navigator !== "undefined" && !navigator.onLine));
    label.textContent = isOffline ? `${name} (Offline)` : name;
  }
  const button = document.querySelector(".profile-action");
  if (button && state.activeProfile) {
    const avatarEl = button.querySelector(".profile-avatar, [data-icon='user']");
    if (avatarEl) {
      avatarEl.outerHTML = renderProfileAvatar(state.activeProfile, "small-header");
    }
  }
}

function cachedMobileToolsAccess(profile = state.activeProfile) {
  const online = profile?.online;
  const licenses = Array.isArray(online?.licenses) ? online.licenses : [];
  const hasMobileLicense = licenses.some(
    (license) =>
      String(license?.status || "").toLowerCase() === "active" &&
      String(license?.plan || "")
        .toLowerCase()
        .replace(/[_\s]+/g, "-") === "mobile-tools"
  );
  const allowed = Boolean(online?.mobile_tools_enabled || hasMobileLicense);
  return {
    allowed,
    reason: allowed ? "" : "",
    profile,
  };
}

async function selectProfile(username) {
  const openDirectly = Boolean(document.querySelector("#profile-open-directly")?.checked);
  const result = await apiRequest("/api/profiles/select", {
    method: "POST",
    body: JSON.stringify({ username, open_directly: openDirectly }),
  });
  state.activeProfile = result.profile;
  state.activeCase = null;
  state.pendingCaseName = "";
  if (result.access) {
    state.mobileToolsAccess = result.access;
  }
  upsertProfile(result.profile);
  setLanguage(state.activeProfile.language === "en" ? "en" : "tr");
  setTheme(state.activeProfile.theme === "light" ? "light" : "dark");
  if (!result.access) {
    await refreshMobileToolsAccess({ silent: true });
  }
  hideProfileGate();
  syncProfileButton();
  await loadPersistedSettings();
  await loadEvidenceCases();
  render();
  setRoute("home");
}

async function createLocalProfileFromWizard() {
  captureProfileDraft();
  const fullName = state.profileDraft.fullName || "";
  let username = state.profileDraft.username || slugifyUsername(fullName);
  if (!username) username = "kullanici";
  const language = state.profileDraft.language || state.language || "tr";
  const theme = state.profileDraft.theme || state.theme || "dark";
  const openDirectly = Boolean(state.profileDraft.openDirectly);

  if (!fullName) {
    state.wizardStep = 3;
    showProfileGate(t("profile.required"));
    return;
  }

  try {
    const result = await apiRequest("/api/profiles/create", {
      method: "POST",
      body: JSON.stringify({
        full_name: fullName,
        username,
        language,
        theme,
        open_directly: openDirectly,
      }),
    });
    upsertProfile(result.profile);
    state.activeProfile = result.profile;
    state.activeCase = null;
    state.pendingCaseName = "";
    state.mobileToolsAccess = { allowed: false, reason: "" };
    state.profileDraft = {
      fullName: "",
      username: "",
      usernameManual: false,
      onlineIdentifier: "",
      onlinePassword: "",
      openDirectly: false,
      language,
      theme,
    };
    setLanguage(result.profile.language === "en" ? "en" : "tr");
    setTheme(result.profile.theme === "light" ? "light" : "dark");
    hideProfileGate();
    syncProfileButton();
    await loadPersistedSettings();
    await loadEvidenceCases();
    render();
    setRoute("home");
    showToast(
      t("profile.created", { name: result.profile.full_name }) || "Profil başarıyla oluşturuldu.",
      "success"
    );
  } catch (error) {
    showToast(t("profile.createFailed", { message: error.message }), "error");
    showProfileGate(error.message);
  }
}

async function connectOnlineProfileFromWizard() {
  captureProfileDraft();
  const identifier = state.profileDraft.onlineIdentifier || "";
  const password = state.profileDraft.onlinePassword || "";
  const language = state.profileDraft.language || state.language || "tr";
  const theme = state.profileDraft.theme || state.theme || "dark";
  const openDirectly = Boolean(state.profileDraft.openDirectly);

  if (!identifier || !password) {
    showProfileGate(t("profile.onlineRequired"));
    return;
  }

  try {
    const result = await apiRequest("/api/profiles/online-login", {
      method: "POST",
      body: JSON.stringify({ identifier, password, language, theme, open_directly: openDirectly }),
    });
    state.activeProfile = result.profile;
    state.activeCase = null;
    state.pendingCaseName = "";
    upsertProfile(result.profile);
    state.mobileToolsAccess = result.access || { allowed: false, reason: "" };
    state.profileDraft = {
      fullName: "",
      username: "",
      usernameManual: false,
      onlineIdentifier: "",
      onlinePassword: "",
      language,
      theme,
      openDirectly,
    };
    setLanguage(result.profile.language === "en" ? "en" : "tr");
    setTheme(result.profile.theme === "light" ? "light" : "dark");
    hideProfileGate();
    syncProfileButton();
    await loadPersistedSettings();
    await loadEvidenceCases();
    render();
    setRoute("home");
    showToast(t("profile.onlineConnected"), "success");
  } catch (error) {
    showToast(t("profile.onlineFailed", { message: error.message }), "error");
    showProfileGate(error.message);
  }
}

async function createProfileFromGate() {
  await createLocalProfileFromWizard();
}

async function connectOnlineProfileFromGate() {
  await connectOnlineProfileFromWizard();
}

async function syncOnlineProfile(button, silent = false) {
  if (button) button.disabled = true;
  try {
    const isOffline = typeof navigator !== "undefined" && !navigator.onLine;
    if (isOffline) {
      if (state.activeProfile?.online) {
        state.activeProfile.online.status = "offline";
        syncProfileButton();
        render();
      }
      if (!silent) {
        showToast(
          state.language === "tr"
            ? "İnternet bağlantısı yok. Hesap çevrimdışı (offline) modda kullanılıyor."
            : "No internet connection. Account is running in offline mode.",
          "info"
        );
      }
      return;
    }

    const result = await apiRequest("/api/profiles/online-sync", { method: "POST" });
    if (result.profile) {
      state.activeProfile = result.profile;
      upsertProfile(result.profile);
    }
    state.mobileToolsAccess = result.access || cachedMobileToolsAccess(state.activeProfile);
    syncProfileButton();
    render();

    if (result.status === "online" || result.ok) {
      if (!silent) showToast(t("profile.onlineSynced"), "success");
    } else if (result.session_expired || result.status === "session_expired") {
      if (!silent) {
        showToast(
          state.language === "tr"
            ? "Online oturum süresi dolmuş. Hesap çevrimdışı (offline) modda çalışıyor. Yeniden giriş yapabilirsiniz."
            : "Online session has expired. Account is running in offline mode. You can log in again.",
          "warn"
        );
      }
    } else {
      if (!silent) {
        showToast(
          state.language === "tr"
            ? "Sunucuya ulaşılamadı. Hesap çevrimdışı (offline) modda kullanılıyor."
            : "Server unreachable. Account is running in offline mode.",
          "info"
        );
      }
    }
  } catch (error) {
    if (state.activeProfile?.online) {
      state.activeProfile.online.status = "offline";
      syncProfileButton();
      render();
    }
    if (!silent) {
      showToast(
        state.language === "tr"
          ? "Sunucuya ulaşılamadı. Hesap çevrimdışı (offline) modda kullanılıyor."
          : "Server unreachable. Account is running in offline mode.",
        "warn"
      );
    }
  } finally {
    if (button) button.disabled = false;
  }
}

async function disconnectOnlineProfile(button) {
  button.disabled = true;
  try {
    const result = await apiRequest("/api/profiles/online-logout", { method: "POST" });
    state.activeProfile = result.profile || state.activeProfile;
    if (result.profile) upsertProfile(result.profile);
    state.mobileToolsAccess = result.access || { allowed: false, reason: "" };
    syncProfileButton();
    render();
    showToast(t("profile.onlineDisconnectedToast"), "success");
  } catch (error) {
    showToast(t("profile.onlineDisconnectFailed", { message: error.message }), "error");
  } finally {
    button.disabled = false;
  }
}

function upsertProfile(profile) {
  if (!profile) return;
  const index = state.profiles.findIndex((item) => item.username === profile.username);
  if (index >= 0) {
    state.profiles[index] = profile;
  } else {
    state.profiles.push(profile);
  }
}

async function refreshMobileToolsAccess({ silent = false } = {}) {
  if (!backendReady() || !state.activeProfile) {
    state.mobileToolsAccess = cachedMobileToolsAccess(state.activeProfile);
    return state.mobileToolsAccess;
  }
  try {
    const result = await apiRequest("/api/profiles/mobile-access");
    state.mobileToolsAccess = result.access || cachedMobileToolsAccess(state.activeProfile);
    if (result.access?.profile) {
      state.activeProfile = result.access.profile;
      upsertProfile(result.access.profile);
    }
  } catch (error) {
    state.mobileToolsAccess = cachedMobileToolsAccess(state.activeProfile);
    if (!silent) showToast(error.message, "error");
  }
  return state.mobileToolsAccess;
}

function isMobileToolsRoute(route) {
  return route === "android" || route === "ios" || route.startsWith("android:");
}

function onlineMobileToolsAllowed() {
  return Boolean(state.mobileToolsAccess?.allowed);
}

function mobileToolsLockedPage({ t, icon, pageTitle }) {
  return `
    <section class="page">
      ${pageTitle(t("mobile.locked.title"), t("mobile.locked.desc"), "key", icon)}
      <div class="workflow-layout locked-layout">
        <div class="workflow-panel mobile-lock-panel">
          <span class="metric-icon">${icon("shield")}</span>
          <h3>${t("mobile.locked.heading")}</h3>
          <p>${t("mobile.locked.body")}</p>
          <div class="button-row">
            <button class="primary-button" data-action="profile-online-start">${icon("globe")} ${t("profile.onlineConnect")}</button>
            <button class="secondary-button" data-route="profile">${icon("user")} ${t("profile.title")}</button>
          </div>
        </div>
      </div>
    </section>
  `;
}

function render() {
  if (state.isDevConsole) return;
  syncSidebarState();
  const activeGroup = routeGroup(state.route);
  const mobileAllowed = onlineMobileToolsAllowed();

  document.querySelectorAll("[data-route]").forEach((button) => {
    const r = button.dataset.route;
    button.classList.toggle("active", r === activeGroup);

    if (r === "android" || r === "ios") {
      button.classList.toggle("is-locked", !mobileAllowed);
    }
  });

  const boundCasePanel = (subdir, hint) =>
    casePanel(subdir, hint, {
      t,
      icon,
      state,
      caseSelectOptions,
      caseOutputLabel,
      escapeHtml,
    });

  if (isMobileToolsRoute(state.route) && !onlineMobileToolsAllowed()) {
    view.innerHTML = mobileToolsLockedPage({ t, icon, pageTitle });
  } else if (state.route.startsWith("workflow:")) {
    view.innerHTML = workflowPage({
      id: state.route.split(":")[1],
      workflows,
      state,
      t,
      icon,
      localText: localizeText,
      canonicalRamFileName,
      caseSelectOptions,
      caseOutputLabel,
      escapeHtml,
    });
  } else if (state.route.startsWith("android:")) {
    view.innerHTML = androidModePage({
      modeId: state.route.split(":")[1],
      t,
      icon,
      pageTitle,
      state,
      escapeHtml,
      backendReady,
      casePanel: boundCasePanel,
      field,
    });
  } else if (state.route === "ios") {
    view.innerHTML = iosPage({
      t,
      icon,
      pageTitle,
      state,
      escapeHtml,
      backendReady,
      casePanel: boundCasePanel,
      field,
    });
  } else {
    const pageCtx = {
      t,
      icon,
      state,
      assetPath,
      pageTitle,
      pickerField: boundPickerField,
      field,
      escapeHtml,
      caseSelectOptions,
      casePanel: boundCasePanel,
      detailPanel: boundDetailPanel,
      toolHub: (platform) => toolHub(platform),
      platformLabel,
      APP_VERSION,
      theme: state.theme,
    };

    syncBrandLogo();

    view.innerHTML = routes[state.route]?.(pageCtx) || homePage(pageCtx);
  }

  const pageEl =
    (typeof view?.querySelector === "function" ? view.querySelector(".page") : null) ||
    document.querySelector("#view .page");
  if (pageEl && pageEl.classList) {
    pageEl.classList.remove?.("page-forward", "page-back", "page-standard", "page-refresh");
    if (pageEl.offsetWidth !== undefined) {
      void pageEl.offsetWidth;
    }
    if (state.navDirection === "forward") {
      pageEl.classList.add?.("page-forward");
    } else if (state.navDirection === "back") {
      pageEl.classList.add?.("page-back");
    } else if (state.navDirection === "refresh") {
      pageEl.classList.add?.("page-refresh");
    } else {
      pageEl.classList.add?.("page-standard");
    }
  }

  hydrateIcons(view);
  if (state.route === "home" || !state.route) {
    loadEvidenceCases();
  }
  if (state.route === "other" && ["evidence", "reports", "history"].includes(state.activeTab)) {
    loadEvidenceCases();
  }
  if (state.route === "other" && state.activeTab === "history") {
    loadAcquisitionHistory();
  }
  if (state.route === "profile") {
    loadEvidenceCases();
  }
  if (state.route.startsWith("workflow:")) {
    const workflow = workflows[state.route.split(":")[1]];
    if (workflow && workflow.mode.includes("disk")) loadEvidenceCases();
  }
  if (state.route === "about") {
    loadDevelopers(true).catch(() => {});
  }
  if (
    state.route === "android:logical" ||
    state.route === "android:filesystem" ||
    state.route === "android:ram"
  )
    loadEvidenceCases();
  if (state.route === "ios") loadEvidenceCases();
  if (state.route === "settings" && state.updateAvailable) {
    const statusEl = document.querySelector("[data-update-status]");
    const resultArea = document.querySelector("[data-update-result]");
    const dlBtn = resultArea?.querySelector("[data-action='download-update']");
    const isTr = (state.language || "en") === "tr";
    if (statusEl && !statusEl.innerHTML.trim()) {
      statusEl.innerHTML = `<span class="status-icon-update">${icon("rocket")}</span> <span>${isTr ? "Yeni sürüm:" : "New version:"} <b>${escapeHtml(state.updateAvailable.latestTag)}</b></span>`;
    }
    if (resultArea) resultArea.style.display = "block";
    if (dlBtn) dlBtn.style.display = "inline-flex";
  }
  view.focus({ preventScroll: true });
  renderRadialNavDOM();
  renderCaseSidebarDOM();
  renderReportSidebarDOM();
  if (typeof document !== "undefined" && document.body?.classList?.contains?.("show-key-hints")) {
    syncShortcutBadges();
  }
}

let navCloseTimer = null;

function renderRadialNavDOM() {
  const container = document.getElementById("radial-nav-container");
  if (!container) return;

  // Menü açıkken veya kapanma animasyonu sürerken dış render çağrılarının DOM'u ezmesini engelle
  if (state.navMenu.isOpen || state.navMenu.isClosing) {
    return;
  }

  container.innerHTML = renderRadialNav(state, t, icon, escapeHtml, getAvatarUrl);
  hydrateIcons(container);
}

function renderCaseSidebarDOM() {
  const container = document.getElementById("case-sidebar-container");
  if (!container) return;
  const wasOpen = Boolean(
    document.getElementById("home-case-sidebar")?.classList?.contains?.("is-open")
  );
  const wasBackdropOpen = Boolean(
    document.querySelector(".home-case-sidebar-backdrop")?.classList?.contains?.("is-open")
  );
  container.innerHTML = renderCaseSidebar(state, t, icon, escapeHtml);
  if (wasOpen) {
    document.getElementById("home-case-sidebar")?.classList?.add?.("is-open");
  }
  if (wasBackdropOpen) {
    document.querySelector(".home-case-sidebar-backdrop")?.classList?.add?.("is-open");
  }
  hydrateIcons(container);
}

function renderReportSidebarDOM() {
  const container = document.getElementById("report-sidebar-container");
  if (!container) return;
  const wasOpen = Boolean(
    document.getElementById("home-report-sidebar")?.classList?.contains?.("is-open")
  );
  const wasCollapsed = Boolean(state.reportSidebarCollapsed);
  const wasBackdropOpen = Boolean(
    document.querySelector(".home-report-sidebar-backdrop")?.classList?.contains?.("is-open")
  );
  container.innerHTML = renderReportSidebar(state, t, icon, escapeHtml);
  if (wasOpen && !wasCollapsed) {
    document.getElementById("home-report-sidebar")?.classList?.add?.("is-open");
  }
  if (wasCollapsed) {
    document.getElementById("home-report-sidebar")?.classList?.add?.("is-collapsed");
    document.querySelector(".home-report-sidebar-toggle")?.classList?.add?.("is-visible");
    document.getElementById("app")?.classList?.add?.("report-sidebar-collapsed");
  } else {
    document.getElementById("app")?.classList?.remove?.("report-sidebar-collapsed");
  }
  if (wasBackdropOpen && !wasCollapsed) {
    document.querySelector(".home-report-sidebar-backdrop")?.classList?.add?.("is-open");
  }
  hydrateIcons(container);
}

function handleReportFile(file) {
  if (!file) return;
  if (!file.type.startsWith("image/")) {
    showToast(
      t("onlineReport.error", { message: "Yalnızca görsel dosyaları seçilebilir" }) ||
        "Geçersiz dosya türü",
      "warn"
    );
    return;
  }
  if (file.size > 10 * 1024 * 1024) {
    showToast("Görsel 10MB'den büyük olamaz.", "warn");
    return;
  }
  state.reportDraft = state.reportDraft || {};
  state.reportDraft.images = state.reportDraft.images || [];
  if (state.reportDraft.images.length >= 5) {
    showToast("En fazla 5 görsel ekleyebilirsiniz.", "warn");
    return;
  }
  const reader = new FileReader();
  reader.onload = (e) => {
    state.reportDraft.images.push({
      dataUrl: e.target.result,
      name: file.name,
      type: file.type,
      size: file.size,
    });
    renderReportSidebarDOM();
  };
  reader.readAsDataURL(file);
}

async function handleSubmitReport() {
  const draft = state.reportDraft || {};
  const title = (draft.title || "").trim();
  const description = (draft.description || "").trim();
  if (!title) {
    showToast(t("onlineReport.titleRequired") || "Başlık alanı boş bırakılamaz.", "warn");
    return;
  }
  if (!description) {
    showToast(t("onlineReport.descRequired") || "Açıklama alanı boş bırakılamaz.", "warn");
    return;
  }

  const online = state?.activeProfile?.online;
  const isOnline = Boolean(
    online &&
    online.username &&
    online.status !== "offline" &&
    online.status !== "session_expired" &&
    (typeof navigator === "undefined" || navigator.onLine !== false)
  );
  if (!isOnline) {
    showToast(
      t("onlineReport.lockedDesc") || "Rapor göndermek için online profil bağlayın.",
      "warn"
    );
    return;
  }

  draft.submitting = true;
  draft.error = "";
  draft.success = "";
  renderReportSidebarDOM();

  try {
    const imageUrls = [];
    const images = Array.isArray(draft.images) ? draft.images : [];
    for (const img of images) {
      if (img.url) {
        imageUrls.push(img.url);
      } else if (img.dataUrl) {
        const uploadRes = await apiRequest("/api/reports/upload-image", {
          method: "POST",
          body: JSON.stringify({
            imageBase64: img.dataUrl,
            filename: img.name || "screenshot.png",
            contentType: img.type || "image/png",
          }),
        });
        if (uploadRes && uploadRes.url) {
          imageUrls.push(uploadRes.url);
        }
      }
    }

    await apiRequest("/api/reports/submit", {
      method: "POST",
      body: JSON.stringify({
        title,
        description,
        imageUrls,
      }),
    });

    draft.submitting = false;
    draft.title = "";
    draft.description = "";
    draft.images = [];
    draft.success = t("onlineReport.success") || "Raporunuz başarıyla iletildi!";
    draft.error = "";
    showToast(t("onlineReport.success") || "Raporunuz başarıyla iletildi!", "success");
    renderReportSidebarDOM();
  } catch (err) {
    draft.submitting = false;
    draft.error =
      err.message || t("onlineReport.error", { message: err.message }) || "Rapor gönderilemedi";
    showToast(`Rapor gönderilemedi: ${err.message}`, "error");
    renderReportSidebarDOM();
  }
}

function openNavMenu() {
  if (state.navMenu.isClosing) return;
  if (navCloseTimer) clearTimeout(navCloseTimer);
  state.navMenu.isOpen = true;
  state.navMenu.isClosing = false;

  const container = document.getElementById("radial-nav-container");
  if (!container) return;

  const nav = container.querySelector(".nav.centered-nav");
  const island = container.querySelector(".nav-center-island");
  const trigger = container.querySelector(".nav-center-trigger");

  if (nav && island && trigger) {
    // Varsa eski backdrop ve wheel'i temizle
    container.querySelector(".nav-backdrop")?.remove();
    island.querySelector(".nav-radial-wheel")?.remove();

    // Backdrop ekle
    const backdrop = document.createElement("div");
    backdrop.className = "nav-backdrop";
    backdrop.dataset.navAction = "close-menu";
    backdrop.setAttribute("aria-label", "Kapat");
    container.insertBefore(backdrop, nav);

    // Çark HTML'ini ekle
    const wheelHtml = renderRadialWheelHtml(state, t, icon, escapeHtml, getAvatarUrl);
    const tempDiv = document.createElement("div");
    tempDiv.innerHTML = wheelHtml;
    const wheelEl = tempDiv.firstElementChild;
    if (wheelEl) {
      island.appendChild(wheelEl);
      hydrateIcons(wheelEl);
    }

    // Bir sonraki frame'de sınıfları ekleyerek CSS top ve transform transition'larını tetikle
    requestAnimationFrame(() => {
      nav.classList.remove("is-closing");
      nav.classList.add("is-open");
      trigger.classList.remove("is-closing");
      trigger.classList.add("is-open");
      trigger.setAttribute("aria-expanded", "true");
      trigger.setAttribute("aria-label", t("nav.closeMenu") || "Menüyü Kapat");
    });
  } else {
    container.innerHTML = renderRadialNav(state, t, icon, escapeHtml, getAvatarUrl);
    hydrateIcons(container);
  }
}

function closeNavMenu() {
  if (!state.navMenu.isOpen || state.navMenu.isClosing) return;
  if (navCloseTimer) clearTimeout(navCloseTimer);
  state.navMenu.isClosing = true;

  const container = document.getElementById("radial-nav-container");
  if (container) {
    const backdrop = container.querySelector(".nav-backdrop");
    const nav = container.querySelector(".nav.centered-nav");
    const trigger = container.querySelector(".nav-center-trigger");
    const wheel = container.querySelector(".nav-radial-wheel");

    if (backdrop) backdrop.classList.add("is-closing");
    if (wheel) wheel.classList.add("is-closing");
    if (nav) {
      nav.classList.add("is-closing");
      nav.classList.remove("is-open");
    }
    if (trigger) {
      trigger.classList.add("is-closing");
      trigger.classList.remove("is-open");
      trigger.setAttribute("aria-expanded", "false");
      trigger.setAttribute("aria-label", t("nav.openMenu") || "Menüyü Aç");
    }
  }

  navCloseTimer = setTimeout(() => {
    state.navMenu.isOpen = false;
    state.navMenu.isClosing = false;
    if (container) {
      container.querySelector(".nav-backdrop")?.remove();
      container.querySelector(".nav-radial-wheel")?.remove();
      const nav = container.querySelector(".nav.centered-nav");
      const trigger = container.querySelector(".nav-center-trigger");
      nav?.classList.remove("is-closing");
      trigger?.classList.remove("is-closing");
    }
  }, 180);
}

function toggleNavMenu() {
  if (state.navMenu.isClosing) return;
  if (state.navMenu.isOpen) {
    closeNavMenu();
  } else {
    openNavMenu();
  }
}

function toggleCaseSidebar() {
  const sidebar = document.getElementById("home-case-sidebar");
  const backdrop = document.querySelector(".home-case-sidebar-backdrop");
  if (sidebar) sidebar.classList.toggle("is-open");
  if (backdrop) backdrop.classList.toggle("is-open");
}

function toggleReportSidebar() {
  const sidebar = document.getElementById("home-report-sidebar");
  const backdrop = document.querySelector(".home-report-sidebar-backdrop");
  const toggleBtn = document.querySelector(".home-report-sidebar-toggle");
  if (sidebar) {
    if (sidebar.classList.contains("is-collapsed")) {
      sidebar.classList.remove("is-collapsed");
      if (toggleBtn) toggleBtn.classList.remove("is-visible");
      sidebar.classList.add("is-open");
      if (backdrop) backdrop.classList.add("is-open");
      state.reportSidebarCollapsed = false;
      document.getElementById("app")?.classList?.remove?.("report-sidebar-collapsed");
    } else {
      sidebar.classList.toggle("is-open");
      if (backdrop) backdrop.classList.toggle("is-open");
    }
  }
}

function routeGroup(route) {
  if (route.startsWith("android:")) return "android";
  if (route === "ios") return "ios";
  if (!route.startsWith("workflow:")) return route;
  const workflowId = route.split(":")[1] || "";
  if (workflowId.startsWith("windows")) return "windows";
  if (workflowId.startsWith("linux")) return "linux";
  return route;
}

function toolHub(platform) {
  const cards = toolCards[platform]
    .map((card, index) => {
      const workflow = workflows[card.id];
      const blocked = workflow && isLocalWorkflowBlocked(workflow);
      const targetRoute = card.route || `workflow:${card.id}`;
      const shortcutKey = String(index + 1);
      return `
        <button class="forensic-card ${blocked ? "is-disabled" : ""}" data-route="${targetRoute}" data-shortcut="${shortcutKey}" data-nav-dir="forward" style="--accent:${card.accent}" ${blocked ? `aria-disabled="true" data-disabled-reason="${workflow.platform}"` : ""}>
          <span class="card-icon">${icon(card.icon)}</span>
          <span class="shortcut-key-badge" aria-hidden="true">${shortcutKey}</span>
          <h3>${localizeText(card.title)}</h3>
          <p>${localizeText(card.desc)}</p>
          <span class="meta">${blocked ? t("localUnsupported") : localizeText(card.badge)}</span>
        </button>
      `;
    })
    .join("");

  const isWindows = platform === "windows";
  const detectedIcon =
    state.platform === "windows"
      ? "windows"
      : state.platform === "linux"
        ? "linux"
        : state.platform === "android"
          ? "android"
          : "monitor";
  return `
    <section class="page">
      <div class="platform-note">
        ${icon(detectedIcon)} ${t("hub.detected", { platform: `<strong>${platformLabel(state.platform)}</strong>` })}
      </div>
      ${pageTitle(
        t(isWindows ? "hub.windows.title" : "hub.linux.title"),
        t(isWindows ? "hub.windows.desc" : "hub.linux.desc"),
        isWindows ? "windows" : "linux",
        icon
      )}
      <div class="tool-grid">${cards}</div>
    </section>
  `;
}

function platformLabel(platform) {
  return platformName(platform, t("unknown"));
}

function contributorCard(initials, name, role, photo, links) {
  return `
    <article class="contributor-card">
      <img class="avatar" src="${assetPath}/contributors/${photo}" alt="${name}" />
      <h3>${name}</h3>
      <p>${role}</p>
      <div class="social-row" aria-label="${name} bağlantıları">
        ${links.map(([label, url]) => socialLink(label, url)).join("")}
      </div>
    </article>
  `;
}

function socialLink(label, url) {
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

const routes = {
  home: homePage,
  tools: toolsPage,
  windows: () => toolHub("windows"),
  linux: () => toolHub("linux"),
  android: () => androidPage({ t, icon, pageTitle, state, escapeHtml, backendReady }),
  ios: () => "",
  docker: dockerPage,
  help: helpPage,
  "remote-acq": remoteAcqPage,
  profile: profilePage,
  other: otherPage,
  settings: settingsPage,
  about: aboutPage,
};

function formatDisplayDate(dateStr) {
  if (!dateStr) return "";
  try {
    const d = new Date(dateStr);
    if (isNaN(d.getTime())) return String(dateStr).split("T")[0] || String(dateStr);
    return d.toLocaleDateString(undefined, { year: "numeric", month: "long", day: "numeric" });
  } catch {
    return String(dateStr);
  }
}

function profilePage({ t, icon, state, pageTitle, escapeHtml }) {
  const profile = state.activeProfile;
  const online = profile?.online || null;
  const mobileAllowed = onlineMobileToolsAllowed();
  const onlineName = onlineDisplayName(online);
  const fullName =
    onlineName || profile?.full_name || profile?.display_name || t("profile.noActive");
  const username = online?.username || profile?.username || "-";
  const email = online?.email || null;
  const rawDate = profile?.created_at || online?.linked_at || online?.created_at || null;
  const registeredDate = formatDisplayDate(rawDate);
  const isBrowserOffline = typeof navigator !== "undefined" && !navigator.onLine;
  const status = isBrowserOffline ? "offline" : online?.status || (online ? "offline" : "local");
  const isOnline = status === "online";
  const isExpired = status === "session_expired";

  let statusBadge = "";
  if (!online) {
    statusBadge = `
      <span class="status-pill ok case-active-pill">${icon("check")} ${t("profile.active") || "Aktif"}</span>
      <span class="status-pill warn">${icon("user")} ${t("profile.localAccount") || "Yerel Profil"}</span>
    `;
  } else if (isOnline) {
    statusBadge = `
      <span class="status-pill ok case-active-pill">${icon("check")} ${t("profile.active") || "Aktif"}</span>
      <span class="status-pill ok">${icon("globe")} ${t("profile.onlineConnectedShort") || "Online"}</span>
    `;
  } else if (isExpired) {
    statusBadge = `
      <span class="status-pill ok case-active-pill">${icon("check")} ${t("profile.active") || "Aktif"}</span>
      <span class="status-pill danger" title="${t("profile.sessionExpiredHint") || "Online oturum süresi doldu"}">${icon("alert-circle")} Offline (${t("profile.sessionExpiredShort") || "Oturum Doldu"})</span>
    `;
  } else {
    statusBadge = `
      <span class="status-pill ok case-active-pill">${icon("check")} ${t("profile.active") || "Aktif"}</span>
      <span class="status-pill warn">${icon("shield")} Offline</span>
    `;
  }

  const rolesHtml = online
    ? onlineRoleBadges(online, t, escapeHtml)
    : `<span class="status-pill warn">${escapeHtml(t("profile.localUser") || "Yerel Kullanıcı")}</span>`;

  const licenseHtml = online ? onlineLicenseText(online, t, escapeHtml) : "";

  const activeText =
    t("case.active") === "case.active"
      ? state.language === "en"
        ? "Active"
        : "Aktif"
      : t("case.active");

  // Cases grid
  const caseCards = state.cases.length
    ? state.cases
        .map((item) => {
          const isActive = state.activeCase?.case_name === item.case_name;
          return `
          <article class="case-profile-card ${isActive ? "is-active-case" : ""}" data-case-name="${escapeHtml(item.case_name || "")}" role="button" tabindex="0">
            <div class="case-profile-top">
              <strong title="${escapeHtml(item.case_name || "")}">${escapeHtml(item.case_name || "-")}</strong>
              ${isActive ? `<span class="status-pill ok case-active-pill">${icon("check")} ${escapeHtml(activeText)}</span>` : ""}
            </div>
            <small title="${escapeHtml(item.case_dir || "")}">${escapeHtml(item.case_dir || "")}</small>
            <div class="case-profile-counts">
              <span class="case-count-pill" title="${t("profile.caseImages") || "İmaj"}">${icon("disk")} ${item.output_count || 0}</span>
              <span class="case-count-pill" title="RAM">${icon("ram")} ${item.ram_count || 0}</span>
              <span class="case-count-pill" title="Android">${icon("android")} ${item.android_count || 0}</span>
              <span class="case-count-pill" title="iOS">${icon("ios")} ${item.ios_count || 0}</span>
              <span class="case-count-pill" title="Docker">${icon("docker")} ${item.docker_count || 0}</span>
            </div>
          </article>
        `;
        })
        .join("")
    : `<div class="log-box">${t("profile.noCases")}</div>`;

  return `
    <section class="page profile-page">
      ${pageTitle(t("profile.title"), "", "user", icon)}

      <!-- 1. Büyük Yatay Profil Kartı -->
      <div class="profile-hero-card">
        <div class="profile-hero-avatar-box">
          ${renderProfileAvatar(profile || { full_name: fullName, username, online }, "profile-card-avatar")}
        </div>
        <div class="profile-hero-content">
          <div class="profile-hero-header">
            <div class="profile-hero-identity">
              <div class="profile-hero-title-row">
                <h2 class="profile-hero-fullname">${escapeHtml(fullName)}</h2>
                <div class="profile-hero-status">
                  ${statusBadge}
                </div>
              </div>
              <span class="profile-hero-username">@${escapeHtml(username)}</span>
            </div>
          </div>

          <div class="profile-hero-meta">
            ${email ? `<div class="profile-meta-item">${icon("mail")} <span>${escapeHtml(email)}</span></div>` : ""}
            ${registeredDate ? `<div class="profile-meta-item">${icon("calendar")} <span>${t("profile.registeredAt") || "Kayıt Tarihi"}: ${escapeHtml(registeredDate)}</span></div>` : ""}
            ${online?.last_sync_at ? `<div class="profile-meta-item">${icon("refresh")} <span>${t("profile.lastSync") || "Son Eşitleme"}: ${escapeHtml(online.last_sync_at)}</span></div>` : ""}
          </div>

          <div class="profile-hero-roles">
            <div class="profile-roles-group">
              <span class="profile-section-label">${t("profile.roles") || "Roller"}:</span>
              <div class="profile-role-list">${rolesHtml}</div>
            </div>
            ${
              online
                ? `
              <div class="profile-roles-group">
                <span class="profile-section-label">${t("profile.license") || "Lisans"}:</span>
                <span class="status-pill ${online.has_license ? "ok" : "warn"}">${licenseHtml}</span>
              </div>
            `
                : ""
            }
          </div>
        </div>
      </div>

      <!-- 2. Alttaki Yatay Aksiyon Barı -->
      <div class="profile-action-bar">
        <button class="secondary-button" data-action="profile-new">${icon("user-plus")} <span>${t("profile.newProfile")}</span></button>
        ${online ? `<button class="secondary-button" data-action="profile-online-sync">${icon("refresh")} <span>${t("profile.onlineSync")}</span></button>` : ""}
        ${isExpired ? `<button class="primary-button" data-action="profile-online-start">${icon("globe")} <span>${t("profile.onlineLogin") || "Giriş Yap"}</span></button>` : ""}
        ${online ? `<button class="secondary-button btn-disconnect" data-action="profile-online-logout">${icon("unlink")} <span>${t("profile.onlineDisconnect")}</span></button>` : `<button class="primary-button" data-action="profile-online-start">${icon("globe")} <span>${t("profile.onlineConnect")}</span></button>`}
        <button class="danger-button btn-logout" data-action="profile-logout">${icon("stop")} <span>${t("profile.logout")}</span></button>
      </div>

      <!-- 3. Vakalar (Cases) Ayrı Div Olarak Yan Yana Sütunlar -->
      <div class="profile-cases-section">
        <div class="profile-cases-header">
          <div class="profile-cases-title-wrap">
            <span class="settings-kicker">${t("profile.cases") || "VAKALAR"}</span>
            <h3>${t("profile.caseTitle") || "Vaka Kütüphanesi"}</h3>
          </div>
          <span class="status-pill ok">${state.cases.length} ${t("profile.caseCountSuffix") || "Vaka"}</span>
        </div>
        <div class="profile-cases-grid">
          ${caseCards}
        </div>
      </div>
    </section>
  `;
}

function onlineDisplayName(online) {
  const fullName = [online?.first_name, online?.last_name]
    .map((value) => String(value || "").trim())
    .filter(Boolean)
    .join(" ");
  return fullName || online?.username || "";
}

function onlineRoleBadges(online, t, escapeHtml) {
  const roles = Array.isArray(online?.roles) ? online.roles : [];
  if (!roles.length)
    return `<span class="status-pill warn">${escapeHtml(t("profile.noRoles"))}</span>`;
  return roles
    .map((role) => `<span class="status-pill ok">${escapeHtml(roleLabel(role, t))}</span>`)
    .join("");
}

function roleLabel(role, t) {
  const labels = {
    bdfl: "BDFL",
    developer: t("profile.role.developer"),
    member: t("profile.role.member"),
    "windows-maintainer": "Windows Maintainer",
    "linux-maintainer": "Linux Maintainer",
    "android-maintainer": "Android Maintainer",
  };
  return labels[role] || role;
}

function onlineLicenseText(online, t, escapeHtml) {
  if (!online) return escapeHtml(t("profile.onlineRequiredForLicense"));
  const licenses = Array.isArray(online.licenses) ? online.licenses : [];
  const active = licenses.find((license) => license.status === "active");
  if (active) {
    const plan = active.plan || t("unknown");
    return `${escapeHtml(t("profile.licenseActive"))} · ${escapeHtml(plan)}`;
  }
  return escapeHtml(online.has_license ? t("profile.licenseActive") : t("profile.licenseNone"));
}

function workedCaseTypesText(online, t, escapeHtml) {
  const values = Array.isArray(online?.worked_case_types) ? online.worked_case_types : [];
  if (!values.length) return escapeHtml(t("profile.workedTypesEmpty"));
  return values.map((value) => escapeHtml(value)).join(" · ");
}

function profileActivityHtml(profile, t, icon, escapeHtml) {
  const log = Array.isArray(profile?.activity_log) ? profile.activity_log.slice(-5).reverse() : [];
  if (!log.length)
    return `<div class="log-box profile-activity-log">• ${escapeHtml(t("profile.activityEmpty"))}</div>`;
  return `
    <div class="profile-activity-log">
      <p class="section-label">${icon("clock")} ${t("profile.activity")}</p>
      ${log
        .map(
          (entry) => `
        <div class="tree-node">
          <strong>${escapeHtml(entry.category || "-")} · ${escapeHtml(entry.action || "-")}</strong>
          <span>
            ${escapeHtml(entry.case_name || entry.details || "-")}
            <small>${escapeHtml(entry.timestamp || "")}</small>
          </span>
        </div>
      `
        )
        .join("")}
    </div>
  `;
}

function isExternalUrl(url) {
  try {
    if (typeof url === "string" && (url.startsWith("#") || url.startsWith("/api/"))) {
      return false;
    }
    const parsed = new URL(url, window.location.href);
    if (parsed.origin === window.location.origin) {
      return false;
    }
    return ["http:", "https:", "mailto:"].includes(parsed.protocol);
  } catch {
    return false;
  }
}

async function openExternalUrl(url) {
  try {
    await apiRequest("/api/open-url", {
      method: "POST",
      body: JSON.stringify({ url }),
    });
    return;
  } catch (error) {
    console.warn("External link could not be opened by backend", error);
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

async function loadEvidenceCases({ silent = true } = {}) {
  if (!backendReady()) return;
  try {
    const result = await apiRequest("/api/evidence-cases");
    state.caseBaseDir = result.base_dir || "";
    state.cases = Array.isArray(result.cases) ? result.cases : [];

    // Keep frontend selected/pending case active even before it exists on disk.
    const activeUser = state.activeProfile?.username || "";
    const backendCase = result.current_case?.case_name || "";
    const rememberedCaseName = getPersistedCaseName(activeUser);
    const activeCaseName =
      state.pendingCaseName || backendCase || state.activeCase?.case_name || rememberedCaseName;
    if (activeCaseName) {
      const stillExists = state.cases.find((c) => c.case_name === activeCaseName);
      if (stillExists) {
        state.activeCase = stillExists;
        state.pendingCaseName = "";
        persistCaseName(stillExists.case_name, activeUser);
      } else if (state.pendingCaseName) {
        state.activeCase = { case_name: state.pendingCaseName };
      } else if (result.current_case) {
        state.activeCase = result.current_case;
        persistCaseName(result.current_case.case_name, activeUser);
      } else if (rememberedCaseName) {
        persistCaseName("", activeUser);
      }
    } else if (result.current_case) {
      state.activeCase = result.current_case;
      persistCaseName(result.current_case.case_name, activeUser);
    } else if (state.cases.length) {
      state.activeCase = state.cases[0];
      persistCaseName(state.activeCase.case_name, activeUser);
    }

    updateCaseControls();
    if (!silent) showToast(t("case.loaded", { count: String(state.cases.length) }));
  } catch (error) {
    if (!silent) showToast(t("case.listFailed", { message: error.message }), "error");
  }
}

function updateCaseControls() {
  document.querySelectorAll("[data-case-base]").forEach((node) => {
    node.textContent = state.caseBaseDir || "~/Amele/Vakalar";
  });

  const selected = state.pendingCaseName || state.activeCase?.case_name || "";
  document.querySelectorAll("[data-case-select]").forEach((select) => {
    const allowNew = select.dataset.allowNewCase === "1";
    select.innerHTML = caseSelectOptions(selected, { allowNew });
    select.value = selected;
    toggleCaseCreateInput(select);
  });
  updateCaseSidebarDOM();
}

function updateCaseSidebarDOM() {
  renderCaseSidebarDOM();
  renderReportSidebarDOM();
}

function caseSelectOptions(selected = "", { allowNew = false } = {}) {
  const effectiveSelected = selected || (allowNew && !state.cases.length ? "__new__" : "");
  if (!state.cases.length && !allowNew) {
    return `<option value="">${t("case.noCases")}</option>`;
  }
  const hasSelected =
    effectiveSelected && effectiveSelected !== "__new__"
      ? state.cases.some((item) => item.case_name === effectiveSelected)
      : true;
  const pendingOption = !hasSelected
    ? `<option value="${escapeHtml(effectiveSelected)}" selected>${escapeHtml(effectiveSelected)}</option>`
    : "";
  const options = state.cases
    .map((item) => {
      const name = escapeHtml(item.case_name || "");
      const isSelected = item.case_name === effectiveSelected ? " selected" : "";
      return `<option value="${name}"${isSelected}>${name}</option>`;
    })
    .join("");
  const newSelected =
    effectiveSelected === "__new__" || (allowNew && !state.cases.length) ? " selected" : "";
  const newOption = allowNew
    ? `<option value="__new__"${newSelected}>${t("workflow.newCase")}</option>`
    : "";
  return `${pendingOption}${options}${newOption}`;
}

function toggleCaseCreateInput(select) {
  document.querySelectorAll("[data-case-output]").forEach((output) => {
    output.textContent = caseOutputLabel(
      select.value,
      output.dataset.caseOutputSubdir || "ciktilar"
    );
  });
}

function imageCaseOutputLabel(caseName) {
  return caseOutputLabel(caseName, "ciktilar");
}

function caseOutputLabel(caseName, subdir = "ciktilar") {
  if (caseName === "__new__") caseName = state.pendingCaseName || "";
  const selected =
    state.cases.find((item) => item.case_name === caseName) ||
    (state.activeCase?.case_name === caseName ? state.activeCase : null);
  const keyBySubdir = {
    android: "android_dir",
    ios: "ios_dir",
    ram: "ram_dir",
    ciktilar: "output_dir",
  };
  const key = keyBySubdir[subdir] || "output_dir";
  if (caseName && caseName !== "__new__" && selected?.[key]) return selected[key];
  const folderBySubdir = {
    android: "android",
    ios: "ios",
    ram: "ram",
    ciktilar: "ciktilar",
  };
  const folder = folderBySubdir[subdir] || "ciktilar";
  if (state.caseBaseDir) return `${state.caseBaseDir}/${caseName || "vaka"}/${folder}`;
  return `~/Amele/Vakalar/${caseName || "vaka"}/${folder}`;
}

function reportCaseName() {
  return resolveSelectedCaseName("#report-case", { fallbackToDefault: true });
}

function resolveSelectedCaseName(selector = "#workflow-case", { fallbackToDefault = false } = {}) {
  const selected = document.querySelector(selector)?.value.trim() || "";
  if (selected && selected !== "__new__") return selected;
  if (state.pendingCaseName) return state.pendingCaseName;
  if (state.activeCase?.case_name) return state.activeCase.case_name;
  return fallbackToDefault ? defaultCaseName() : "";
}

async function ensureImageCase() {
  const selected = resolveSelectedCaseName("#workflow-case");
  if (selected) {
    const existing = state.cases.find((item) => item.case_name === selected);
    if (existing && (existing.case_dir || existing.case_path)) return existing;

    // Create new case folder on backend disk if not already existing
    const created = await apiRequest("/api/evidence-create", {
      method: "POST",
      body: JSON.stringify({ case_name: selected }),
    });
    state.activeCase = created;
    state.pendingCaseName = "";
    state.cachedDefaultCaseName = "";
    await loadEvidenceCases();
    return created;
  }

  const caseName = defaultCaseName();
  const created = await apiRequest("/api/evidence-create", {
    method: "POST",
    body: JSON.stringify({ case_name: caseName }),
  });
  state.activeCase = created;
  state.pendingCaseName = "";
  state.cachedDefaultCaseName = "";
  await loadEvidenceCases();
  return created;
}

function defaultCaseName() {
  const now = new Date();
  const pad = (value) => String(value).padStart(2, "0");
  return `Case_${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
}

function stableDefaultCaseName() {
  if (!state.cachedDefaultCaseName) {
    state.cachedDefaultCaseName = defaultCaseName();
  }
  return state.cachedDefaultCaseName;
}

function selectedTargetName() {
  const select = document.querySelector("[data-field='target']");
  const option = select?.selectedOptions?.[0];
  return option?.dataset.diskName || option?.textContent?.split("·")[0]?.trim() || "";
}

function backendReady() {
  return backendAvailable;
}

function connectionPayload() {
  const tokenText = document.querySelector("[data-field='token']")?.value.trim() || "";
  if (tokenText && !state.approvedSecurityKey) {
    throw new Error(t("connection.keyApproveFirst"));
  }
  if (tokenText && tokenText !== state.approvedSecurityKey) {
    throw new Error(t("connection.keyChanged"));
  }
  return {
    ip: document.querySelector("[data-field='ip']")?.value.trim() || "",
    port: Number(document.querySelector("[data-field='port']")?.value.trim() || 0),
    token: tokenText ? state.approvedSecurityKey : null,
  };
}

function sshPayload() {
  const ip = document.querySelector("[data-field='ip']")?.value.trim() || "";
  const port = Number(document.querySelector("[data-field='port']")?.value.trim() || 22);
  const user = document.querySelector("[data-field='ssh-user']")?.value.trim() || "";
  const password = document.querySelector("[data-field='ssh-pass']")?.value || "";
  const keyPath = document.querySelector("#ssh-key-file")?.value.trim() || "";
  if (!ip) throw new Error(t("connection.ipPortRequired"));
  if (!user) throw new Error("SSH kullanıcı adı gereklidir");
  return {
    ip,
    port: port || 22,
    user,
    password: password || null,
    key_path: keyPath || null,
  };
}

function vpnPayload() {
  const endpoint = document.querySelector("[data-field='vpn-endpoint']")?.value.trim() || "";
  const configFile = document.querySelector("#vpn-config-file")?.value.trim() || "";
  if (!endpoint) throw new Error(t("vpn.endpointRequired"));
  if (!configFile) throw new Error(t("vpn.configRequired"));
  return {
    config_file: configFile,
    private_key: document.querySelector("[data-field='vpn-private-key']")?.value.trim() || "",
    public_key: document.querySelector("[data-field='vpn-public-key']")?.value.trim() || "",
    endpoint,
    allowed_ips: document.querySelector("[data-field='vpn-allowed']")?.value.trim() || "0.0.0.0/0",
    address: document.querySelector("[data-field='vpn-address']")?.value.trim() || "10.0.0.2/24",
    dns: document.querySelector("[data-field='vpn-dns']")?.value.trim() || "1.1.1.1",
    keepalive: Number(document.querySelector("[data-field='vpn-keepalive']")?.value.trim() || 25),
  };
}

function currentWorkflowId() {
  return state.route.startsWith("workflow:") ? state.route.split(":")[1] : "";
}

function currentWorkflow() {
  return workflows[currentWorkflowId()];
}

function rememberConnection(workflowId, payload, details) {
  state.remoteConnections[workflowId] = {
    ip: payload.ip,
    port: payload.port,
    token: payload.token || "",
    serverName: details.server_name || "",
    serverVersion: details.server_version || "",
    features: details.features || [],
  };
}

function forgetConnection(workflowId = currentWorkflowId()) {
  if (workflowId) delete state.remoteConnections[workflowId];
}

function requireActiveConnection(workflow, payload) {
  if (!workflow?.mode.startsWith("remote")) return true;
  const connection = state.remoteConnections[currentWorkflowId()];
  const matches =
    connection &&
    connection.ip === payload.ip &&
    Number(connection.port) === Number(payload.port) &&
    (connection.token || "") === (payload.token || "");
  if (!matches) {
    showToast(t("connection.connectFirst"), "error");
    updateSide("connection", t("connection.none"));
    writeWorkflowLog(t("connection.required"));
    return false;
  }
  return true;
}

// Polyfill/safe helper for non-element nodes (like Text nodes during selectstart)
if (typeof Node !== "undefined" && !Node.prototype.closest) {
  Node.prototype.closest = function (selector) {
    const el = this.nodeType === 1 ? this : this.parentElement;
    return el && typeof el.closest === "function" ? el.closest(selector) : null;
  };
}

// Prevent image drag and accidental selection highlight artifacts across chrome/sidebar/brand
document.addEventListener("dragstart", (event) => {
  const el = event.target?.nodeType === 1 ? event.target : event.target?.parentElement;
  if (
    event.target?.tagName === "IMG" ||
    el?.closest?.(".sidebar, .brand-row, .brand-mark, .about-hero")
  ) {
    event.preventDefault();
  }
});

document.addEventListener("selectstart", (event) => {
  const el = event.target?.nodeType === 1 ? event.target : event.target?.parentElement;
  if (
    el?.closest?.(
      ".brand-mark, .sidebar-head, .brand-row, .sidebar, .sidebar-toggle, .about-hero-center, .about-hero-logo"
    )
  ) {
    event.preventDefault();
  }
});

const clearSidebarSelection = () => {
  try {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return;
    const inChrome = (node) => {
      if (!node) return false;
      const el = node.nodeType === Node.ELEMENT_NODE ? node : node.parentElement;
      return !!el?.closest?.(
        ".sidebar, .brand-row, .brand-mark, #brand-logo, .about-hero-logo, .about-hero-center, .topbar"
      );
    };
    if (inChrome(sel.anchorNode) || inChrome(sel.focusNode)) {
      sel.removeAllRanges();
    }
  } catch (_) {}
};

const clearChromeArtifacts = () => {
  try {
    window.getSelection()?.removeAllRanges();
    if (
      document.activeElement &&
      document.activeElement !== document.body &&
      document.activeElement.closest?.(".sidebar, .brand-row, #brand-logo, .brand-mark, .topbar")
    ) {
      document.activeElement.blur();
    }
  } catch (_) {}
};

for (const delay of [0, 40, 100, 250, 500, 1000, 2000]) {
  setTimeout(clearChromeArtifacts, delay);
}

document.addEventListener("selectionchange", clearSidebarSelection);
document.addEventListener(
  "mousedown",
  (event) => {
    const el = event.target?.nodeType === 1 ? event.target : event.target?.parentElement;
    if (
      el?.closest?.(".sidebar, .brand-row, .brand-mark, #brand-logo, .about-hero-logo, .topbar")
    ) {
      clearChromeArtifacts();
    }
  },
  { capture: true }
);

document.addEventListener(
  "mouseup",
  (event) => {
    const el = event.target?.nodeType === 1 ? event.target : event.target?.parentElement;
    if (
      el?.closest?.(".sidebar, .brand-row, .brand-mark, #brand-logo, .about-hero-logo, .topbar")
    ) {
      clearChromeArtifacts();
    }
  },
  { capture: true }
);

window.addEventListener("focus", clearChromeArtifacts);
clearChromeArtifacts();

window.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && state.navMenu?.isOpen) {
    closeNavMenu();
  }
});

document.addEventListener("click", async (event) => {
  // Kilitli rapor formu veya devre dışı bırakılmış ajan kutusu tıklandığında kesinlikle işlem yapma
  const lockedReport = event.target.closest(".report-sidebar-form.is-locked");
  if (lockedReport && !event.target.closest("[data-action='profile-online-start']")) {
    event.preventDefault();
    event.stopPropagation();
    return;
  }

  const agentDisabled = event.target.closest(
    ".agent-input-wrapper.is-disabled, .agent-box.is-disabled, #agent-prompt-input[disabled], #agent-prompt-input[readonly]"
  );
  if (agentDisabled) {
    event.preventDefault();
    event.stopPropagation();
    return;
  }

  // Dairesel Gezinti Menüsü (Radial Wheel Nav) İşlemleri
  const navAction = event.target.closest("[data-nav-action]");
  if (navAction) {
    const action = navAction.dataset.navAction;
    if (action === "toggle-menu") {
      event.preventDefault();
      event.stopPropagation();
      toggleNavMenu();
      return;
    }
    if (action === "close-menu") {
      event.preventDefault();
      event.stopPropagation();
      closeNavMenu();
      return;
    }
    if (action === "toggle-language") {
      event.preventDefault();
      event.stopPropagation();
      closeNavMenu();
      const nextLang = state.language === "tr" ? "en" : "tr";
      setLanguage(nextLang);
      return;
    }
    if (action === "logout") {
      event.preventDefault();
      event.stopPropagation();
      closeNavMenu();
      showProfileGate();
      return;
    }
  }

  // Özel Agent ve Model Dropdown Açma / Kapama / Seçme
  const toggleAgentMenu = event.target.closest("[data-agent-action='toggle-agent-menu']");
  if (toggleAgentMenu) {
    event.preventDefault();
    event.stopPropagation();
    const agentDropdown = document.getElementById("agent-custom-dropdown");
    const modelDropdown = document.getElementById("model-custom-dropdown");
    if (modelDropdown) modelDropdown.classList.remove("is-open");
    if (agentDropdown) {
      agentDropdown.classList.toggle("is-open");
    }
    return;
  }

  const toggleModelMenu = event.target.closest("[data-agent-action='toggle-model-menu']");
  if (toggleModelMenu) {
    event.preventDefault();
    event.stopPropagation();
    const agentDropdown = document.getElementById("agent-custom-dropdown");
    const modelDropdown = document.getElementById("model-custom-dropdown");
    if (agentDropdown) agentDropdown.classList.remove("is-open");
    if (modelDropdown) {
      modelDropdown.classList.toggle("is-open");
    }
    return;
  }

  const selectAgentItem = event.target.closest("[data-agent-action='select-agent-item']");
  if (selectAgentItem) {
    event.preventDefault();
    event.stopPropagation();
    if (selectAgentItem.classList.contains("disabled")) return;
    const agentDropdown = document.getElementById("agent-custom-dropdown");
    if (agentDropdown) agentDropdown.classList.remove("is-open");
    const agentId = selectAgentItem.dataset.agentId;
    if (agentId) {
      handleAgentChange(agentId, state, render);
    }
    return;
  }

  const selectModelItem = event.target.closest("[data-agent-action='select-model-item']");
  if (selectModelItem) {
    event.preventDefault();
    event.stopPropagation();
    const modelDropdown = document.getElementById("model-custom-dropdown");
    if (modelDropdown) modelDropdown.classList.remove("is-open");
    const modelId = selectModelItem.dataset.modelId;
    if (modelId) {
      handleModelChange(modelId, state);
      render();
    }
    return;
  }

  if (!event.target.closest(".agent-custom-dropdown")) {
    document.querySelectorAll(".agent-custom-dropdown.is-open").forEach((el) => {
      el.classList.remove("is-open");
    });
  }

  const agentSend = event.target.closest(
    "[data-agent-action='send'], [data-copilot-action='send']"
  );
  if (agentSend) {
    event.preventDefault();
    if (agentSend.disabled || agentSend.hasAttribute("disabled")) return;
    const input = document.querySelector("#agent-prompt-input, #copilot-prompt-input");
    if (input && (input.disabled || input.hasAttribute("disabled"))) return;
    const prompt = input
      ? input.value
      : state.agent?.promptDraft || state.copilot?.promptDraft || "";
    submitAgentPrompt(prompt, state, render, showToast, t);
    return;
  }

  const agentChip = event.target.closest("[data-agent-chip], [data-copilot-chip]");
  if (agentChip) {
    event.preventDefault();
    const input = document.querySelector("#agent-prompt-input, #copilot-prompt-input");
    if (input && (input.disabled || input.hasAttribute("disabled") || input.readOnly)) return;
    const chipType = agentChip.dataset.agentChip || agentChip.dataset.copilotChip;
    const promptText = getQuickChipPrompt(chipType, state.language === "en");
    if (input) {
      input.value = promptText;
      input.focus();
    }
    if (state.agent) state.agent.promptDraft = promptText;
    if (state.copilot) state.copilot.promptDraft = promptText;
    return;
  }

  const addCtx = event.target.closest(
    "[data-agent-action='add-context'], [data-copilot-action='add-context']"
  );
  if (addCtx) {
    event.preventDefault();
    const input = document.querySelector("#agent-prompt-input, #copilot-prompt-input");
    if (input && (input.disabled || input.hasAttribute("disabled") || input.readOnly)) return;
    const caseName = state.activeCase?.case_name || "varsayilan_vaka";
    const platform = state.platform;
    const ctxSnippet = ` [@vaka: ${caseName}, @platform: ${platform}] `;
    if (input) {
      input.value = (input.value ? input.value + " " : "") + ctxSnippet;
      if (state.agent) state.agent.promptDraft = input.value;
      if (state.copilot) state.copilot.promptDraft = input.value;
      input.focus();
    }
    showToast(t("copilot.contextAdded"));
    return;
  }

  const copyCmd = event.target.closest(
    "[data-agent-action='copy-cmd'], [data-copilot-action='copy-cmd']"
  );
  if (copyCmd) {
    event.preventDefault();
    const cmd = copyCmd.dataset.cmd;
    if (cmd && navigator.clipboard) {
      navigator.clipboard.writeText(cmd);
      copyCmd.textContent = t("copilot.copied");
      setTimeout(() => {
        copyCmd.textContent = t("copilot.copy");
      }, 1800);
    }
    return;
  }

  const runCmd = event.target.closest(
    "[data-agent-action='run-cmd'], [data-copilot-action='run-cmd']"
  );
  if (runCmd) {
    event.preventDefault();
    const cmd = runCmd.dataset.cmd;
    const msgId = runCmd.dataset.msgId;
    executeAmeleCommand(cmd, null, null, msgId, state, render, showToast, t, null, true);
    return;
  }

  const relaunchBtn = event.target.closest("[data-agent-action='relaunch-terminal']");
  if (relaunchBtn) {
    event.preventDefault();
    const prompt = relaunchBtn.dataset.prompt;
    const agentId = relaunchBtn.dataset.agentId;
    const modelId = relaunchBtn.dataset.modelId;
    if (agentId && state.agent) handleAgentChange(agentId, state, render);
    if (modelId && state.agent && (!agentId || state.agent.selectedAgent === agentId)) {
      state.agent.selectedModel = modelId;
    }
    submitAgentPrompt(prompt, state, render, showToast, t);
    return;
  }

  const linuxConfirm = event.target.closest("[data-elevation-action='confirm-linux']");
  if (linuxConfirm) {
    event.preventDefault();
    const cmd = linuxConfirm.dataset.cmd;
    const msgId = linuxConfirm.dataset.msgId;
    const pwdInput = document.querySelector("#sudo-password-input");
    const sudoPassword = pwdInput ? pwdInput.value : null;
    executeAmeleCommand(cmd, sudoPassword, null, msgId, state, render, showToast, t, true);
    return;
  }

  const togglePwd = event.target.closest("[data-elevation-action='toggle-pwd']");
  if (togglePwd) {
    event.preventDefault();
    const pwdInput = document.querySelector("#sudo-password-input");
    if (pwdInput) {
      pwdInput.type = pwdInput.type === "password" ? "text" : "password";
    }
    return;
  }

  const winConfirm = event.target.closest("[data-elevation-action='confirm-windows']");
  if (winConfirm) {
    event.preventDefault();
    const cmd = winConfirm.dataset.cmd;
    const msgId = winConfirm.dataset.msgId;
    executeAmeleCommand(cmd, null, true, msgId, state, render, showToast, t, null, true);
    return;
  }

  const elevCancel = event.target.closest("[data-elevation-action='cancel']");
  if (elevCancel) {
    event.preventDefault();
    if (state.agent) state.agent.elevationModal = null;
    if (state.copilot) state.copilot.elevationModal = null;
    render();
    return;
  }

  const newsHScroll = event.target.closest("[data-news-h-scroll]");
  if (newsHScroll) {
    event.preventDefault();
    const dir = newsHScroll.dataset.newsHScroll;
    const track = document.querySelector("#news-horizontal-track");
    if (track) {
      const scrollAmount = 320;
      track.scrollBy({ left: dir === "left" ? -scrollAmount : scrollAmount, behavior: "smooth" });
    }
    return;
  }

  const tocBtn = event.target.closest("[data-action='help-toc']");
  if (tocBtn) {
    event.preventDefault();
    const targetId = tocBtn.dataset.target;
    if (targetId) {
      const targetEl = document.getElementById(targetId);
      if (targetEl) {
        targetEl.scrollIntoView({ behavior: "smooth", block: "start" });
        document
          .querySelectorAll(".help-toc-link")
          .forEach((btn) => btn.classList.remove("active"));
        tocBtn.classList.add("active");
      }
    }
    return;
  }

  const anchorLink = event.target.closest("a[href^='#']");
  if (anchorLink) {
    event.preventDefault();
    const hash = anchorLink.getAttribute("href").replace(/^#/, "");
    if (hash) {
      const targetEl =
        document.getElementById(decodeURIComponent(hash)) || document.getElementById(hash);
      if (targetEl) {
        targetEl.scrollIntoView({ behavior: "smooth", block: "start" });
      }
    }
    return;
  }

  const externalLink = event.target.closest("a[href]");
  if (externalLink && isExternalUrl(externalLink.href)) {
    event.preventDefault();
    openExternalUrl(externalLink.href);
    return;
  }

  const sidebarToggle = event.target.closest("[data-sidebar-toggle]");
  if (sidebarToggle) {
    event.preventDefault();
    setSidebarCollapsed(sidebarToggle.dataset.sidebarToggle === "collapse");
    return;
  }

  const newsLink = event.target.closest("[data-news-link]");
  if (newsLink) {
    event.preventDefault();
    const url = newsLink.dataset.newsLink || newsLink.getAttribute("href");
    if (url) {
      openExternalUrl(url);
    }
    return;
  }

  const newsAction = event.target.closest("[data-news-action]");
  if (newsAction) {
    event.preventDefault();
    const action = newsAction.dataset.newsAction;
    const total = Math.min(state.news?.length || 1, 5);
    if (action === "prev") {
      state.activeNewsIndex = (state.activeNewsIndex - 1 + total) % total;
    } else if (action === "next") {
      state.activeNewsIndex = (state.activeNewsIndex + 1) % total;
    }
    render();
    return;
  }

  const newsDot = event.target.closest("[data-news-dot]");
  if (newsDot) {
    event.preventDefault();
    const idx = parseInt(newsDot.dataset.newsDot, 10);
    if (!isNaN(idx)) {
      state.activeNewsIndex = idx;
      render();
    }
    return;
  }

  const caseProfileCard = event.target.closest(".case-profile-card[data-case-name]");
  if (caseProfileCard) {
    const caseName = caseProfileCard.dataset.caseName;
    const matched = state.cases.find((c) => c.case_name === caseName);
    if (matched && state.activeCase?.case_name !== caseName) {
      state.activeCase = matched;
      persistCaseName(caseName, state.activeProfile?.username || "");
      render();
      return;
    }
  }

  const helpDocBtn = event.target.closest("[data-action='help-select-doc'][data-doc]");
  if (helpDocBtn) {
    event.preventDefault();
    state.activeHelpDoc = helpDocBtn.dataset.doc;
    render();
    return;
  }

  const helpCopyBtn = event.target.closest("[data-action='copy-help-code']");
  if (helpCopyBtn) {
    event.preventDefault();
    const pre = helpCopyBtn.closest(".help-code-block")?.querySelector("pre code");
    if (pre) {
      if (navigator.clipboard) {
        navigator.clipboard.writeText(pre.textContent || "");
      }
      const label = helpCopyBtn.querySelector("span") || helpCopyBtn;
      const orig = label.textContent;
      label.textContent = state.language === "en" ? "Copied!" : "Kopyalandı!";
      setTimeout(() => {
        label.textContent = orig;
      }, 1800);
    }
    return;
  }

  const routeButton = event.target.closest("[data-route]");
  if (routeButton) {
    const radialWheel = routeButton.closest?.(".nav-radial-wheel");
    const radialItem = routeButton.closest?.(".nav-radial-item");
    if (radialWheel && radialItem) {
      radialWheel.classList?.add("has-selection");
      radialItem.classList?.add("is-selected");
    }
    closeNavMenu();
    if (routeButton.dataset.tab) {
      state.activeTab = routeButton.dataset.tab;
    }
    setRoute(routeButton.dataset.route, routeButton.dataset.navDir);
    return;
  }

  const actionButton = event.target.closest("[data-action]");
  if (actionButton) {
    handleAction(actionButton);
    return;
  }

  const dockerButton = event.target.closest("[data-docker-action]");
  if (dockerButton) {
    await handleDockerAction(event, {
      apiRequest,
      setRoute,
      render,
      state,
      resolveCase() {
        return resolveSelectedCaseName("#workflow-case") || null;
      },
    });
    return;
  }

  const tabButton = event.target.closest("[data-tab]");
  if (tabButton) {
    state.activeTab = tabButton.dataset.tab;
    const detail = document.querySelector("#other-detail");
    if (detail) detail.innerHTML = boundDetailPanel(state.activeTab);
    hydrateIcons(detail);
    if (["evidence", "reports"].includes(state.activeTab)) loadEvidenceCases();
    if (state.activeTab === "history") {
      loadEvidenceCases();
      loadAcquisitionHistory();
    }
    return;
  }

  const analysisTabButton = event.target.closest("[data-analysis-tab]");
  if (analysisTabButton) {
    const imgInput = document.querySelector("#image-path");
    if (imgInput) state.imagePathInput = imgInput.value.trim();
    const ramInput = document.querySelector("#ram-analysis-path");
    if (ramInput) state.ramAnalysisPathInput = ramInput.value.trim();
    const ramOsSelect = document.querySelector("#ram-os-profile");
    if (ramOsSelect) state.ramOsProfile = ramOsSelect.value || "windows";
    const ramSymbolDir = document.querySelector("#ram-symbol-dir");
    if (ramSymbolDir) state.ramSymbolDirInput = ramSymbolDir.value.trim();

    state.activeAnalysisTab = analysisTabButton.dataset.analysisTab;
    render();
    return;
  }

  const treeNode = event.target.closest(".tree-node");
  if (treeNode && treeNode.closest("#image-tree-root")) {
    const isDir = treeNode.dataset.isDir === "true";
    const relativePath = treeNode.dataset.path;
    const isVirtual = treeNode.dataset.virtual === "true";
    document.querySelectorAll(".tree-node").forEach((el) => el.classList.remove("active"));
    treeNode.classList.add("active");
    if (isVirtual) {
      if (isDir && treeNode.dataset.hasChildren === "true") {
        toggleExistingTreeChildren(treeNode);
      }
      showVirtualTreeInfo(treeNode);
      return;
    }
    if (isDir) {
      expandTreeNode(treeNode, relativePath);
    } else {
      previewImageFile(relativePath);
    }
    return;
  }

  const procRow = event.target.closest(".proc-row");
  if (procRow) {
    document.querySelectorAll(".proc-row").forEach((el) => el.classList.remove("active"));
    procRow.classList.add("active");
    const pid = procRow.dataset.pid;
    const name = procRow.dataset.name;
    inspectProcessDetails(pid, name);
    return;
  }

  const carvedPreviewBtn = event.target.closest("[data-carved-preview]");
  if (carvedPreviewBtn) {
    const path = carvedPreviewBtn.dataset.carvedPreview;
    previewCarvedFile(path);
    return;
  }
});

document.addEventListener("change", async (event) => {
  const agentSelect = event.target.closest(
    "[data-agent-action='change-agent'], [data-copilot-action='change-agent']"
  );
  if (agentSelect) {
    handleAgentChange(agentSelect.value, state, render);
    return;
  }

  const modelSelect = event.target.closest(
    "[data-agent-action='change-model'], [data-copilot-action='change-model']"
  );
  if (modelSelect) {
    handleModelChange(modelSelect.value, state);
    return;
  }

  const scopeSelect = event.target.closest(
    "[data-agent-action='change-scope'], [data-copilot-action='change-scope']"
  );
  if (scopeSelect) {
    handleScopeChange(scopeSelect.value, state);
    return;
  }

  const optSelect = event.target.closest(
    "[data-agent-action='change-opt'], [data-copilot-action='change-opt']"
  );
  if (optSelect) {
    if (state.agent) state.agent.selectedOpt = optSelect.value;
    if (state.copilot) state.copilot.selectedOpt = optSelect.value;
    return;
  }

  const profileLanguage = event.target.closest("#profile-language");
  if (profileLanguage) {
    captureProfileDraft();
    state.profileDraft.language = profileLanguage.value;
    setLanguage(profileLanguage.value);
    renderProfileGate();
    return;
  }

  const profileTheme = event.target.closest("#profile-theme");
  if (profileTheme) {
    captureProfileDraft();
    state.profileDraft.theme = profileTheme.value;
    setTheme(profileTheme.value);
    renderProfileGate();
    return;
  }

  const profileDirect = event.target.closest("#profile-create-directly");
  if (profileDirect) {
    captureProfileDraft();
    return;
  }

  const profileOnlineDirect = event.target.closest("#profile-online-directly");
  if (profileOnlineDirect) {
    captureProfileDraft();
    return;
  }

  const select = event.target.closest("[data-action='language-select']");
  if (select) {
    try {
      await persistSettingsFromControls();
    } catch (error) {
      showToast(`Ayarlar kaydedilemedi: ${error.message}`, "error");
    }
    return;
  }

  const target = event.target.closest("[data-field='target']");
  if (target) {
    updateSide("target", target.value || t("targetNotSelected"));
  }

  const caseSelect = event.target.closest("[data-case-select]");
  if (caseSelect) {
    const activeUser = state.activeProfile?.username || "";
    if (caseSelect.value === "__new__") {
      const promptTitle =
        t("case.promptNewName") || "Lütfen oluşturmak istediğiniz yeni vaka adını girin:";
      const newName = prompt(promptTitle);
      if (newName && newName.trim()) {
        const cleanName = newName.trim();
        state.pendingCaseName = cleanName;
        state.activeCase = { case_name: cleanName };
        persistCaseName(cleanName, activeUser);

        // Mirror to all data-case-select fields on the page
        document.querySelectorAll("[data-case-select]").forEach((el) => {
          el.innerHTML = caseSelectOptions(cleanName, {
            allowNew: el.dataset.allowNewCase === "1",
          });
          el.value = cleanName;
        });

        // Set value to legacy hidden input
        const legacyInput = document.querySelector("#workflow-case-name");
        if (legacyInput) {
          legacyInput.value = cleanName;
        }
      } else {
        // Revert to first case or empty if cancelled
        const fallback = state.cases.length ? state.cases[0].case_name : "";
        state.pendingCaseName = "";
        state.activeCase = state.cases.find((c) => c.case_name === fallback) || null;
        persistCaseName(fallback, activeUser);
        caseSelect.value = fallback;
        document.querySelectorAll("[data-case-select]").forEach((el) => {
          el.value = fallback;
        });
      }
    } else {
      state.pendingCaseName = "";
      state.activeCase = state.cases.find((c) => c.case_name === caseSelect.value) || {
        case_name: caseSelect.value,
      };
      persistCaseName(caseSelect.value, activeUser);
    }
    toggleCaseCreateInput(caseSelect);
  }

  const androidDeviceSelect = event.target.closest("[data-android-device-select]");
  if (androidDeviceSelect) {
    syncAndroidDeviceSelection(androidDeviceSelect, { state, t, showToast });
  }

  const iosBackupPath = event.target.closest("#ios-backup-path");
  if (iosBackupPath) {
    syncIosBackupPathInput(iosBackupPath, state);
  }

  const ramOsSelect = event.target.closest("#ram-os-profile");
  if (ramOsSelect) {
    state.ramOsProfile = ramOsSelect.value || "windows";
  }

  const ramSymbolDir = event.target.closest("#ram-symbol-dir");
  if (ramSymbolDir) {
    state.ramSymbolDirInput = ramSymbolDir.value.trim();
  }

  const reportFileInput = event.target.closest("#report-file-input");
  if (reportFileInput && reportFileInput.files?.[0]) {
    handleReportFile(reportFileInput.files[0]);
    reportFileInput.value = "";
    return;
  }
});

document.addEventListener("input", (event) => {
  const input = event.target.closest("[data-agent-input], [data-copilot-input]");
  if (input) {
    if (state.agent) state.agent.promptDraft = input.value;
    if (state.copilot) state.copilot.promptDraft = input.value;
  }
});

document.addEventListener("keydown", (event) => {
  const elevInput = event.target.closest("[data-elevation-input='password']");
  if (elevInput && event.key === "Enter") {
    event.preventDefault();
    const modalConfirmBtn = document.querySelector("[data-elevation-action='confirm-linux']");
    if (modalConfirmBtn) modalConfirmBtn.click();
    return;
  }

  const input = event.target.closest("[data-agent-input], [data-copilot-input]");
  if (input && event.key === "Enter" && !event.shiftKey) {
    event.preventDefault();
    if (input.disabled || input.hasAttribute("disabled")) return;
    const sendBtn = document.querySelector(
      "[data-agent-action='send'], [data-copilot-action='send']"
    );
    if (sendBtn && (sendBtn.disabled || sendBtn.hasAttribute("disabled"))) return;
    const prompt = input.value;
    submitAgentPrompt(prompt, state, render, showToast, t);
  }
});

document.addEventListener("input", (event) => {
  if (
    event.target.closest("#profile-full-name") ||
    event.target.closest("#profile-username") ||
    event.target.closest("#profile-online-identifier")
  ) {
    captureProfileDraft();
  }
  const iosBackupPath = event.target.closest("#ios-backup-path");
  if (iosBackupPath) {
    syncIosBackupPathInput(iosBackupPath, state);
  }
  const dockerSearch = event.target.closest("[data-docker-action='search']");
  if (dockerSearch) {
    handleDockerAction(event, { apiRequest, setRoute, render, state });
  }

  const reportTitle = event.target.closest("#report-title-input");
  if (reportTitle) {
    state.reportDraft = state.reportDraft || {};
    state.reportDraft.title = reportTitle.value;
    const submitBtn = document.querySelector(".report-sidebar-submit-btn");
    const desc = state.reportDraft.description || "";
    if (submitBtn && !state.reportDraft.submitting) {
      submitBtn.disabled = !reportTitle.value.trim() || !desc.trim();
    }
    const countEl = document.querySelector("label[for='report-title-input'] span:last-child");
    if (countEl) countEl.textContent = `${reportTitle.value.length}/200`;
  }

  const reportDesc = event.target.closest("#report-desc-input");
  if (reportDesc) {
    state.reportDraft = state.reportDraft || {};
    state.reportDraft.description = reportDesc.value;
    const submitBtn = document.querySelector(".report-sidebar-submit-btn");
    const title = state.reportDraft.title || "";
    if (submitBtn && !state.reportDraft.submitting) {
      submitBtn.disabled = !title.trim() || !reportDesc.value.trim();
    }
    const countEl = document.querySelector("label[for='report-desc-input'] span:last-child");
    if (countEl) countEl.textContent = `${reportDesc.value.length}/5000`;
  }
});

document.addEventListener("dragover", (e) => {
  const dropzone = e.target.closest("#report-sidebar-dropzone");
  if (dropzone) {
    e.preventDefault();
    dropzone.classList.add("is-dragover");
  }
});

document.addEventListener("dragleave", (e) => {
  const dropzone = e.target.closest("#report-sidebar-dropzone");
  if (dropzone) {
    dropzone.classList.remove("is-dragover");
  }
});

document.addEventListener("drop", (e) => {
  const dropzone = e.target.closest("#report-sidebar-dropzone");
  if (dropzone) {
    e.preventDefault();
    dropzone.classList.remove("is-dragover");
    const file = e.dataTransfer.files?.[0];
    if (file) handleReportFile(file);
  }
});

document.addEventListener("paste", (e) => {
  if (e.target.closest("#home-report-sidebar")) {
    const items = e.clipboardData?.items;
    if (items) {
      for (const item of items) {
        if (item.type && item.type.startsWith("image/")) {
          const file = item.getAsFile();
          if (file) {
            e.preventDefault();
            handleReportFile(file);
            break;
          }
        }
      }
    }
  }
});

async function handleAction(button) {
  const action = button.dataset.action;
  if (action === "toggle-news-expand") {
    state.newsExpanded = !state.newsExpanded;
    render();
    return;
  }
  if (action === "profile-create-start" || action === "profile-new") {
    captureProfileDraft();
    state.profileGateMode = "wizard";
    state.wizardMode = "create";
    state.wizardStep = 1;
    state.profileDraft = {
      fullName: "",
      username: "",
      usernameManual: false,
      onlineIdentifier: "",
      onlinePassword: "",
      language: state.language || "tr",
      theme: state.theme || "dark",
      openDirectly: false,
    };
    showProfileGate();
    return;
  }
  if (action === "profile-online-start") {
    captureProfileDraft();
    state.profileGateMode = "wizard";
    state.wizardMode = "online";
    state.wizardStep = 1;
    state.profileDraft = {
      fullName: "",
      username: "",
      usernameManual: false,
      onlineIdentifier: "",
      onlinePassword: "",
      language: state.language || "tr",
      theme: state.theme || "dark",
      openDirectly: false,
    };
    showProfileGate();
    return;
  }
  if (action === "profile-online-back" || action === "profile-select-back") {
    captureProfileDraft();
    state.profileGateMode = state.profiles.length ? "select" : "wizard";
    state.wizardStep = 1;
    showProfileGate();
    return;
  }
  if (action === "wizard-set-lang") {
    const lang = button.dataset.lang || "tr";
    state.profileDraft.language = lang;
    setLanguage(lang);
    renderProfileGate();
    return;
  }
  if (action === "wizard-set-theme") {
    const theme = button.dataset.theme || "dark";
    state.profileDraft.theme = theme;
    setTheme(theme);
    renderProfileGate();
    return;
  }
  if (action === "wizard-theme-toggle") {
    const nextTheme = state.theme === "dark" ? "light" : "dark";
    state.profileDraft.theme = nextTheme;
    setTheme(nextTheme);
    const isDark = nextTheme === "dark";
    button.classList.toggle("is-dark", isDark);
    button.classList.toggle("is-light", !isDark);
    button.setAttribute("aria-checked", isDark ? "true" : "false");
    button.setAttribute("aria-label", t("settings.darkTheme") || "Karanlık Tema");
    const wizardLogo = profileGate?.querySelector(".wizard-logo");
    if (wizardLogo) {
      wizardLogo.src =
        nextTheme === "light" ? "./assets/logo/logo-siyah.png" : "./assets/logo/logo.png";
    }
    return;
  }
  if (action === "wizard-online-choice-no") {
    captureProfileDraft();
    state.wizardStep = 5;
    renderProfileGate();
    return;
  }
  if (action === "wizard-online-choice-yes") {
    captureProfileDraft();
    state.wizardStep = 6;
    renderProfileGate();
    return;
  }
  if (action === "wizard-finish-local") {
    captureProfileDraft();
    await createLocalProfileFromWizard();
    return;
  }
  if (action === "wizard-toggle-pwd") {
    const pwdInput = profileGate?.querySelector("#wizard-online-password");
    if (pwdInput) {
      pwdInput.type = pwdInput.type === "password" ? "text" : "password";
    }
    return;
  }
  if (action === "wizard-step-next") {
    captureProfileDraft();
    if (state.wizardStep === 1) {
      state.wizardStep = 2;
    } else if (state.wizardStep === 2) {
      state.wizardStep = 3;
    } else if (state.wizardStep === 3) {
      if (!state.profileDraft.fullName) {
        showProfileGate(t("profile.required"));
        return;
      }
      if (!state.profileDraft.username) {
        state.profileDraft.username = slugifyUsername(state.profileDraft.fullName) || "kullanici";
      }
      state.wizardStep = 4;
    } else if (state.wizardStep === 6) {
      if (!state.profileDraft.onlineIdentifier) {
        showProfileGate(t("profile.onlineRequired"));
        return;
      }
      state.wizardStep = 7;
    }
    renderProfileGate();
    return;
  }
  if (action === "wizard-step-prev") {
    captureProfileDraft();
    if (state.wizardStep === 7) {
      state.wizardStep = 6;
    } else if (state.wizardStep === 6 || state.wizardStep === 5) {
      state.wizardStep = 4;
    } else if (state.wizardStep === 4) {
      state.wizardStep = 3;
    } else if (state.wizardStep === 3) {
      state.wizardStep = 2;
    } else if (state.wizardStep === 2) {
      state.wizardStep = 1;
    } else {
      state.wizardStep = 1;
    }
    renderProfileGate();
    return;
  }
  if (action === "wizard-online-finish" || action === "profile-online-submit") {
    captureProfileDraft();
    await connectOnlineProfileFromWizard();
    return;
  }
  if (action === "profile-select") {
    try {
      await selectProfile(button.dataset.username || "");
    } catch (error) {
      showToast(t("profile.selectFailed", { message: error.message }), "error");
      showProfileGate(error.message);
    }
    return;
  }
  if (action === "profile-submit") {
    await createLocalProfileFromWizard();
    return;
  }
  if (action === "profile-online-sync") {
    await syncOnlineProfile(button);
    return;
  }
  if (action === "profile-online-logout") {
    await disconnectOnlineProfile(button);
    return;
  }
  if (action === "profile-logout") {
    try {
      await apiRequest("/api/profiles/logout", { method: "POST" });
      const profilesResp = await apiRequest("/api/profiles").catch(() => null);
      state.profiles = Array.isArray(profilesResp?.profiles) ? profilesResp.profiles : [];
      state.activeProfile = null;
      state.activeCase = null;
      state.cases = [];
      state.mobileToolsAccess = { allowed: false, reason: "" };
      state.route = "home";
      state.profileGateMode = state.profiles.length ? "select" : "wizard";
      state.wizardStep = 1;
      state.wizardMode = "create";
      syncProfileButton();
      showProfileGate();
    } catch (error) {
      showToast(t("profile.logoutFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "select-case") {
    const caseName = button.dataset.caseName;
    if (caseName) {
      const found = state.cases.find((c) => c.case_name === caseName);
      if (found) {
        state.activeCase = found;
        state.pendingCaseName = "";
      } else {
        state.pendingCaseName = caseName;
        state.activeCase = { case_name: caseName };
      }
      persistCaseName(caseName, state.activeProfile?.username || "");
      updateCaseControls();
      showToast(t("case.selected", { name: caseName }) || `Varsayılan vaka: ${caseName}`);
    }
    return;
  }

  if (action === "toggle-case-sidebar") {
    toggleCaseSidebar();
    return;
  }

  if (action === "close-case-sidebar") {
    const sidebar = document.getElementById("home-case-sidebar");
    const backdrop = document.querySelector(".home-case-sidebar-backdrop");
    if (sidebar) sidebar.classList.remove("is-open");
    if (backdrop) backdrop.classList.remove("is-open");
    return;
  }

  if (action === "toggle-report-sidebar") {
    toggleReportSidebar();
    return;
  }

  if (action === "close-report-sidebar") {
    const sidebar = document.getElementById("home-report-sidebar");
    const backdrop = document.querySelector(".home-report-sidebar-backdrop");
    const toggleBtn = document.querySelector(".home-report-sidebar-toggle");
    if (sidebar) {
      sidebar.classList.remove("is-open");
      sidebar.classList.add("is-collapsed");
    }
    if (backdrop) backdrop.classList.remove("is-open");
    if (toggleBtn) toggleBtn.classList.add("is-visible");
    state.reportSidebarCollapsed = true;
    document.getElementById("app")?.classList?.add?.("report-sidebar-collapsed");
    return;
  }

  if (action === "report-pick-image") {
    const fileInput = document.getElementById("report-file-input");
    if (fileInput) fileInput.click();
    return;
  }

  if (action === "report-remove-image") {
    const idx = parseInt(button.dataset.index, 10);
    if (!isNaN(idx) && state.reportDraft?.images) {
      state.reportDraft.images.splice(idx, 1);
      renderReportSidebarDOM();
    }
    return;
  }

  if (action === "submit-report") {
    await handleSubmitReport();
    return;
  }

  if (action === "refresh-cases") {
    await loadEvidenceCases({ silent: false });
    return;
  }

  if (action === "new-case-prompt") {
    const defaultName = defaultCaseName();
    const name = window.prompt(
      t("case.promptNewName") || "Lütfen oluşturmak istediğiniz yeni vaka adını girin:",
      defaultName
    );
    if (!name || !name.trim()) return;
    const cleanName = name.trim();
    try {
      const result = await apiRequest("/api/evidence-create", {
        method: "POST",
        body: JSON.stringify({ case_name: cleanName }),
      });
      state.activeCase = result;
      state.pendingCaseName = "";
      await loadEvidenceCases();
      showToast(t("case.created", { path: cleanName }));
    } catch (error) {
      showToast(t("case.createFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action?.startsWith("android-")) {
    if (!onlineMobileToolsAllowed()) {
      showToast(t("mobile.locked.toast"), "warning");
      state.profileGateMode = "online";
      showProfileGate();
      return;
    }
    const handled = await handleAndroidAction(button, {
      apiRequest,
      backendReady,
      state,
      t,
      showToast,
      render,
      resolveCase() {
        return resolveSelectedCaseName("#workflow-case") || null;
      },
    });
    if (handled) return;
  }

  if (action?.startsWith("ios-")) {
    if (!onlineMobileToolsAllowed()) {
      showToast(t("mobile.locked.toast"), "warning");
      state.profileGateMode = "online";
      showProfileGate();
      return;
    }
    const handled = await handleIosAction(button, {
      apiRequest,
      backendReady,
      state,
      t,
      showToast,
      render,
      resolveCase() {
        return resolveSelectedCaseName("#workflow-case") || null;
      },
    });
    if (handled) return;
  }

  if (action === "theme-toggle") {
    const nextTheme = state.theme === "dark" ? "light" : "dark";
    setTheme(nextTheme);

    // Update the button state and row icon in place so the Day/Night animation plays smoothly without DOM destruction
    const isDark = nextTheme === "dark";
    button.classList.toggle("is-dark", isDark);
    button.classList.toggle("is-light", !isDark);
    button.setAttribute("aria-checked", isDark ? "true" : "false");
    button.setAttribute("aria-label", t("settings.darkTheme") || "Karanlık Tema");

    const rowIcon = button.closest(".settings-row")?.querySelector(".settings-row-icon");
    if (rowIcon) {
      rowIcon.innerHTML = icon(isDark ? "moon" : "sun");
    }

    try {
      await saveSettingsFromControls();
    } catch (error) {
      showToast(`Ayarlar kaydedilemedi: ${error.message}`, "error");
    }
    return;
  }

  if (action === "set-language") {
    const lang = button.dataset.lang || (state.language === "tr" ? "en" : "tr");
    if (lang !== state.language) {
      setLanguage(lang);
      try {
        await saveSettingsFromControls();
      } catch (error) {
        showToast(`Ayarlar kaydedilemedi: ${error.message}`, "error");
      }
      render();
    }
    return;
  }

  if (action === "pick-file") {
    await pickFile(button.dataset.target);
    return;
  }

  if (action === "pick-folder") {
    await pickFolder(button.dataset.target);
    return;
  }

  if (action === "toggle-vpn") {
    button.classList.toggle("on");
    const panel = document.querySelector(".vpn-panel");
    if (panel) panel.hidden = !button.classList.contains("on");
    writeWorkflowLog(button.classList.contains("on") ? t("vpn.enabled") : t("vpn.disabled"));
    updateSide("connection", button.classList.contains("on") ? t("vpn.waiting") : t("vpn.off"));
    return;
  }

  if (action === "vpn-config") {
    const panel = document.querySelector(".vpn-panel");
    if (panel) panel.hidden = false;
    document.querySelector("[data-action='toggle-vpn']")?.classList.add("on");
    writeWorkflowLog(t("vpn.opened"));
    return;
  }

  if (action === "save-vpn") {
    try {
      const payload = vpnPayload();
      const result = await apiRequest("/api/wireguard-config", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      writeWorkflowLog(t("vpn.configured", { endpoint: payload.endpoint }));
      updateSide("connection", t("vpn.ready"));
      showToast(t("vpn.saved"));
      if (result.path) document.querySelector("#vpn-config-file").value = result.path;
    } catch (error) {
      showToast(t("vpn.failed", { message: error.message }), "error");
      writeWorkflowLog(t("vpn.failed", { message: error.message }));
    }
    return;
  }

  if (action === "start-vpn") {
    const configFile = document.querySelector("#vpn-config-file")?.value.trim();
    if (!configFile) {
      showToast(t("vpn.configRequired"), "error");
      return;
    }
    try {
      await apiRequest("/api/wireguard-start", {
        method: "POST",
        body: JSON.stringify({ config_file: configFile }),
      });
      writeWorkflowLog(t("vpn.started"));
      updateSide("connection", t("vpn.ready"));
      showToast(t("vpn.started"));
    } catch (error) {
      showToast(t("vpn.failed", { message: error.message }), "error");
      writeWorkflowLog(t("vpn.failed", { message: error.message }));
    }
    return;
  }

  if (action === "stop-vpn") {
    try {
      await apiRequest("/api/wireguard-stop", { method: "POST" });
      writeWorkflowLog(t("vpn.stopped"));
      updateSide("connection", t("vpn.off"));
      showToast(t("vpn.stopped"));
    } catch (error) {
      showToast(t("vpn.failed", { message: error.message }), "error");
      writeWorkflowLog(t("vpn.failed", { message: error.message }));
    }
    return;
  }

  if (action === "approve-key") {
    const token = document.querySelector("[data-field='token']");
    const value = token?.value.trim() || "";
    if (!value) {
      showToast(t("key.required"), "error");
      return;
    }
    state.approvedSecurityKey = value;
    if (token) token.readOnly = true;
    writeWorkflowLog(t("key.approved"));
    showToast(t("key.active"));
    return;
  }

  if (action === "reset-key") {
    const token = document.querySelector("[data-field='token']");
    state.approvedSecurityKey = "";
    if (token) {
      token.value = "";
      token.readOnly = false;
    }
    forgetConnection();
    writeWorkflowLog(t("key.reset"));
    return;
  }

  if (action === "connect") {
    const workflowId = currentWorkflowId();
    const workflow = currentWorkflow();
    const isSsh = workflow?.mode.startsWith("ssh");
    let payload;
    try {
      payload = isSsh ? sshPayload() : connectionPayload();
    } catch (error) {
      showToast(error.message, "error");
      return;
    }
    if (!payload.ip || !payload.port) {
      showToast(t("connection.ipPortRequired"), "error");
      return;
    }
    if (payload.port <= 0 || payload.port > 65535) {
      showToast(t("connection.invalidPort"), "error");
      return;
    }
    if (!workflow?.mode.startsWith("remote") && !isSsh) {
      showToast(t("connection.remoteOnly"), "error");
      return;
    }

    forgetConnection(workflowId);
    button.disabled = true;
    updateSide("connection", t("connection.connecting"));
    writeWorkflowLog(t("connection.starting", { host: `${payload.ip}:${payload.port}` }));
    try {
      const endpoint = isSsh ? "/api/ssh-connect" : "/api/connect";
      const result = await apiRequest(endpoint, {
        method: "POST",
        body: JSON.stringify(payload),
      });
      rememberConnection(workflowId, payload, result);
      updateSide("connection", t("connection.connected", { ip: payload.ip }));
      writeWorkflowLog(t("connection.connectedLog", { ip: payload.ip }));
      showToast(t("connection.success"));
    } catch (error) {
      forgetConnection(workflowId);
      updateSide("connection", t("connection.failed"));
      writeWorkflowLog(t("connection.failedLog", { ip: payload.ip, message: error.message }));
      showToast(t("connection.cannotConnect", { message: error.message }), "error");
    } finally {
      button.disabled = false;
    }
    return;
  }

  if (action === "scan") {
    await scanTargets();
    return;
  }

  if (action === "download") {
    await installWinpmem(button);
    return;
  }

  if (action === "install-avml") {
    await installAvml(button);
    return;
  }

  if (action === "start") {
    await startAcquisition(button);
    return;
  }

  if (action === "pause") {
    try {
      await sendAcquisitionControl("pause");
    } catch (error) {
      showToast(t("workflow.pauseFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "resume") {
    try {
      await sendAcquisitionControl("resume");
    } catch (error) {
      showToast(t("workflow.resumeFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "stop") {
    try {
      await sendAcquisitionControl("stop");
    } catch (error) {
      showToast(t("workflow.stopFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "mount-readonly") {
    const imagePath = document.querySelector("#image-path")?.value.trim();
    if (!imagePath || imagePath.startsWith(".")) {
      showToast(t("analysis.imageRequired"), "error");
      return;
    }
    setAnalysisStatus(t("analysis.mounting"), t("analysis.mounting"));
    try {
      const result = await apiRequest("/api/image-mount-readonly", {
        method: "POST",
        body: JSON.stringify({ path: imagePath }),
      });
      state.imageMount = {
        imagePath: result.image_path,
        mountDir: result.mount_dir || "",
        mountMode: result.mount_mode || "mounted",
        label:
          result.mount_mode === "analysis-only"
            ? t("analysis.analysisOnlyStatus")
            : t("analysis.mounted", { path: result.mount_dir }),
      };
      state.imageMountLogHTML = renderMountResultInfo(result);
      state.imageMountTreeHTML = renderTree(result.tree);
      const container = document.querySelector("#image-tree-root");
      if (container) {
        container.innerHTML = state.imageMountTreeHTML;
      }
      const summaryContainer = document.querySelector("#disk-analysis-results");
      if (summaryContainer && result.analysis) {
        summaryContainer.style.display = "block";
        summaryContainer.innerHTML = renderDiskAnalysisSummary(result.analysis);
        hydrateIcons(summaryContainer);
      }
      setAnalysisStatus(state.imageMount.label, state.imageMountLogHTML);
      showToast(
        result.mount_mode === "analysis-only"
          ? t("analysis.analysisOnlyPrepared")
          : t("analysis.mountPrepared")
      );
    } catch (error) {
      state.imageMountLogHTML = "";
      setAnalysisStatus(t("analysis.noImage"), renderErrorPanel(t("analysis.errorTitle"), error));
      showToast(t("analysis.mountFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "unmount-image") {
    try {
      await apiRequest("/api/image-unmount", { method: "POST" });
      state.imageMount = null;
      state.imageMountTreeHTML = "";
      state.imageMountLogHTML = "";
      const container = document.querySelector("#image-tree-root");
      if (container) {
        container.innerHTML = `<div class="log-box">${t("analysis.outputWaiting")}</div>`;
      }
      const preview = document.querySelector("#image-file-preview");
      if (preview) {
        preview.innerHTML = `
          <div class="log-box" style="display:flex;align-items:center;justify-content:center;color:var(--muted);text-align:center;padding:20px">
            Klasör yapısında bir dosyaya tıklayarak içeriğini inceleyebilirsiniz.<br/>Click a file on the left to preview it.
          </div>
        `;
      }
      const summary = document.querySelector("#disk-analysis-results");
      if (summary) {
        summary.style.display = "none";
        summary.innerHTML = "";
      }
      setAnalysisStatus(t("analysis.unmounted"), t("analysis.noActiveMount"));
      showToast(t("analysis.unmounted"));
    } catch (error) {
      showToast(t("analysis.unmountFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "image-analyze") {
    const imagePath = document.querySelector("#image-path")?.value.trim();
    if (!imagePath || imagePath.startsWith(".")) {
      showToast(t("analysis.imageRequired"), "error");
      return;
    }
    const container = document.querySelector("#disk-analysis-results");
    if (container) {
      container.style.display = "block";
      container.innerHTML = `<div class="log-box">${escapeHtml(t("analysis.runningAnalysis"))}</div>`;
    }
    try {
      const result = await apiRequest("/api/image-analyze", {
        method: "POST",
        body: JSON.stringify({ path: imagePath }),
      });
      if (container) {
        container.innerHTML = renderDiskAnalysisSummary(result);
        hydrateIcons(container);
      }
      showToast(t("analysis.doneAnalysis"));
    } catch (error) {
      if (container) container.innerHTML = renderErrorPanel(t("analysis.errorTitle"), error);
      showToast(t("analysis.summaryFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "ram-summary") {
    const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
    if (!ramPath || ramPath.startsWith(".")) {
      showToast("Önce geçerli bir RAM dosyası seçin / Select a valid RAM file first", "error");
      return;
    }
    const osProfile = document.querySelector("#ram-os-profile")?.value || "windows";
    const symbolDir = ramSymbolDirValue();
    document.querySelector("#ram-analysis-results").style.display = "block";
    document.querySelector("#ram-split-view").style.display = "none";
    document.querySelector("#ram-flat-results-panel").style.display = "block";
    const statusLbl = document.querySelector("#stat-status-lbl");
    if (statusLbl) statusLbl.textContent = t("analysis.runningAnalysis");
    const flatTitle = document.querySelector("#ram-flat-title");
    const flatResults = document.querySelector("#ram-flat-results-list");
    if (flatTitle) flatTitle.textContent = t("analysis.ramSummary");

    if (flatResults)
      flatResults.innerHTML = ramConsoleHtml("Canlı Analiz Konsolu", [
        "Volatility3 ile uçucu bellek analizi başlatılıyor...",
        "İlk çalıştırmada sembol çözümleme/indirme sürebilir.",
      ]);
    try {
      const start = await apiRequest("/api/ram-analyze-summary-start", {
        method: "POST",
        body: JSON.stringify({ path: ramPath, os_type: osProfile, symbol_dir: symbolDir }),
      });
      if (!start.job_id) throw new Error(t("workflow.jobIdMissing"));
      const result = await waitForAcquisitionJob(start.job_id, {
        onUpdate(job) {
          updateRamConsole("#ram-flat-results-list", job, "Canlı Analiz Konsolu");
          if (statusLbl && job.message) statusLbl.textContent = job.message;
        },
      });
      document.querySelector("#stat-strings-count").textContent = String(
        result.string_match_count || 0
      );
      document.querySelector("#stat-carved-count").textContent = "-";
      document.querySelector("#stat-procs-count").textContent = String(result.process_count || 0);
      if (statusLbl) statusLbl.textContent = t("analysis.doneAnalysis");
      if (flatResults) {
        flatResults.innerHTML = renderRamAnalysisSummary(result);
        hydrateIcons(flatResults);
      }
      showToast(t("analysis.doneAnalysis"));
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      if (flatResults) {
        flatResults.innerHTML += renderErrorPanel(t("analysis.errorTitle"), error);
      }
      showToast(t("analysis.summaryFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "ram-preflight") {
    const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
    if (!ramPath || ramPath.startsWith(".")) {
      showToast("Önce geçerli bir RAM dosyası seçin / Select a valid RAM file first", "error");
      return;
    }
    const osProfile = document.querySelector("#ram-os-profile")?.value || "windows";
    const symbolDir = ramSymbolDirValue();
    document.querySelector("#ram-analysis-results").style.display = "block";
    document.querySelector("#ram-split-view").style.display = "none";
    document.querySelector("#ram-flat-results-panel").style.display = "block";
    const flatTitle = document.querySelector("#ram-flat-title");
    const flatResults = document.querySelector("#ram-flat-results-list");
    const statusLbl = document.querySelector("#stat-status-lbl");
    if (flatTitle) flatTitle.textContent = t("analysis.preflightTitle");
    if (statusLbl) statusLbl.textContent = "Ön kontrol / Preflight";
    if (flatResults)
      flatResults.innerHTML = ramConsoleHtml(t("analysis.preflightTitle"), [
        "Volatility yolu, symbol dizini ve Linux kernel banner bilgisi kontrol ediliyor...",
      ]);
    try {
      const result = await apiRequest("/api/ram-volatility-preflight", {
        method: "POST",
        body: JSON.stringify({ path: ramPath, os_type: osProfile, symbol_dir: symbolDir }),
      });
      if (statusLbl) statusLbl.textContent = result.ready ? "Hazır / Ready" : "Eksik / Missing";
      if (flatResults) flatResults.innerHTML = renderVolatilityPreflight(result);
      showToast(
        result.ready
          ? "Volatility ön kontrol hazır."
          : "Volatility ön kontrol eksik uyarılar verdi.",
        result.ready ? "success" : "error"
      );
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      if (flatResults)
        flatResults.innerHTML = renderErrorPanel(t("analysis.preflightFailedTitle"), error);
      showToast("Volatility ön kontrol başarısız: " + error.message, "error");
    }
    return;
  }

  if (action === "ram-symbol-install") {
    const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
    if (!ramPath || ramPath.startsWith(".")) {
      showToast("Önce geçerli bir RAM dosyası seçin / Select a valid RAM file first", "error");
      return;
    }
    const osProfile = document.querySelector("#ram-os-profile")?.value || "windows";
    const symbolDir = ramSymbolDirValue();
    document.querySelector("#ram-analysis-results").style.display = "block";
    document.querySelector("#ram-split-view").style.display = "none";
    document.querySelector("#ram-flat-results-panel").style.display = "block";
    const flatTitle = document.querySelector("#ram-flat-title");
    const flatResults = document.querySelector("#ram-flat-results-list");
    const statusLbl = document.querySelector("#stat-status-lbl");
    if (flatTitle) flatTitle.textContent = t("analysis.symbolInstallTitle");
    if (statusLbl) statusLbl.textContent = t("analysis.symbolInstallRunning");
    if (flatResults)
      flatResults.innerHTML = ramConsoleHtml(t("analysis.symbolInstallTitle"), [
        t("analysis.symbolInstallScanning"),
        t("analysis.symbolInstallExact"),
      ]);
    try {
      const result = await apiRequest("/api/ram-volatility-symbol-install", {
        method: "POST",
        body: JSON.stringify({ path: ramPath, os_type: osProfile, symbol_dir: symbolDir }),
      });
      if (result.symbol_dir) {
        state.ramSymbolDirInput = result.symbol_dir;
        const input = document.querySelector("#ram-symbol-dir");
        if (input) input.value = result.symbol_dir;
      }
      const symbolReady = result.installed || result.status === "windows-automatic";
      if (statusLbl)
        statusLbl.textContent = symbolReady
          ? t("analysis.symbolInstallDone")
          : t("analysis.symbolInstallMissing");
      if (flatResults) flatResults.innerHTML = renderVolatilitySymbolInstall(result);
      showToast(
        result.message || t("analysis.symbolInstallDone"),
        symbolReady ? "success" : "error"
      );
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      if (flatResults)
        flatResults.innerHTML = renderErrorPanel(t("analysis.symbolInstallFailedTitle"), error);
      showToast(t("analysis.symbolInstallFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "android-analysis") {
    const caseName = document.querySelector("#android-analysis-case")?.value.trim();
    if (!caseName) {
      showToast(t("analysis.androidRequired"), "error");
      return;
    }
    const container = document.querySelector("#android-analysis-results");
    if (container)
      container.innerHTML = `<div class="log-box">${escapeHtml(t("analysis.runningAnalysis"))}</div>`;
    try {
      const result = await apiRequest("/api/android-case-analysis", {
        method: "POST",
        body: JSON.stringify({ case_name: caseName }),
      });
      if (container) {
        container.innerHTML = renderAndroidAnalysisSummary(result);
        hydrateIcons(container);
      }
      showToast(t("analysis.doneAnalysis"));
    } catch (error) {
      if (container) container.innerHTML = renderErrorPanel(t("analysis.errorTitle"), error);
      showToast(t("analysis.summaryFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "ram-strings") {
    const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
    if (!ramPath || ramPath.startsWith(".")) {
      showToast("Önce geçerli bir RAM dosyası seçin / Select a valid RAM file first", "error");
      return;
    }

    document.querySelector("#ram-analysis-results").style.display = "block";
    document.querySelector("#ram-split-view").style.display = "none";
    document.querySelector("#ram-flat-results-panel").style.display = "block";

    const statusLbl = document.querySelector("#stat-status-lbl");
    if (statusLbl) statusLbl.textContent = t("analysis.runningAnalysis");

    const flatResults = document.querySelector("#ram-flat-results-list");
    flatResults.innerHTML = `<div class="log-box" style="text-align:center;padding:30px">⌛ Uçucu bellek taranıyor, dizgiler çıkartılıyor... (Bu işlem bir miktar sürebilir)<br/>Scanning dynamic memory heap and extracting evidential strings...</div>`;

    try {
      const result = await apiRequest("/api/ram-analyze-strings", {
        method: "POST",
        body: JSON.stringify({ path: ramPath }),
      });

      const count = result.length || 0;
      document.querySelector("#stat-strings-count").textContent = count;
      document.querySelector("#stat-carved-count").textContent = "0";
      document.querySelector("#stat-procs-count").textContent = "0";
      if (statusLbl) statusLbl.textContent = "Analiz Edildi / Analysed";

      if (count === 0) {
        flatResults.innerHTML = `<div class="log-box" style="text-align:center;padding:20px;color:var(--muted)">Hiçbir bulgu dizgisi bulunamadı / No evidential strings found.</div>`;
      } else {
        flatResults.innerHTML = result
          .map(
            (item) => `
          <div class="string-match-item">
            <div class="match-meta">
              <span>Kategori: <strong>${escapeHtml(item.category)}</strong></span>
              <span>Ofset: <strong>0x${item.offset.toString(16).toUpperCase()}</strong></span>
            </div>
            <div class="match-value">${escapeHtml(item.value)}</div>
            <div class="match-context">${escapeHtml(item.context)}</div>
          </div>
        `
          )
          .join("");
      }
      showToast("Dizgi analizi başarıyla tamamlandı.");
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      flatResults.innerHTML = renderErrorPanel(t("analysis.stringsFailedTitle"), error);
      showToast("RAM dizgi analizi başarısız oldu: " + error.message, "error");
    }
    return;
  }

  if (action === "ram-carver") {
    const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
    if (!ramPath || ramPath.startsWith(".")) {
      showToast("Önce geçerli bir RAM dosyası seçin / Select a valid RAM file first", "error");
      return;
    }

    document.querySelector("#ram-analysis-results").style.display = "block";
    document.querySelector("#ram-split-view").style.display = "grid";
    document.querySelector("#ram-flat-results-panel").style.display = "none";

    document.querySelector("#ram-left-panel-title").textContent = t("analysis.lblCarved");
    document.querySelector("#ram-right-panel-title").textContent = "Dosya Önizleme / File Preview";

    const leftList = document.querySelector("#ram-left-list");
    leftList.innerHTML = `<div class="log-box" style="text-align:center;padding:20px">⌛ Bellekten gömülü dosyalar kurtarılıyor... / Carving files from memory...</div>`;
    const rightContent = document.querySelector("#ram-right-content");
    rightContent.innerHTML = `<div class="log-box" style="display:flex;align-items:center;justify-content:center;color:var(--muted);text-align:center">Kurtarılan bir dosyaya tıklayarak içeriğini inceleyin.<br/>Click a carved file on the left to preview.</div>`;

    const statusLbl = document.querySelector("#stat-status-lbl");
    if (statusLbl) statusLbl.textContent = t("analysis.runningAnalysis");

    try {
      const result = await apiRequest("/api/ram-carve-files", {
        method: "POST",
        body: JSON.stringify({ path: ramPath }),
      });

      const count = result.length || 0;
      document.querySelector("#stat-strings-count").textContent = "0";
      document.querySelector("#stat-carved-count").textContent = count;
      document.querySelector("#stat-procs-count").textContent = "0";
      if (statusLbl) statusLbl.textContent = "Kurtarıldı / Carved";

      if (count === 0) {
        leftList.innerHTML = `<div class="log-box" style="text-align:center;padding:12px;color:var(--muted)">Kurtarılan dosya bulunamadı / No carved files.</div>`;
      } else {
        leftList.innerHTML = result
          .map((file) => {
            const isImg = file.mime_type.startsWith("image/");
            const fileIcon = isImg ? "🖼️" : "📄";
            return `
            <div class="tree-node" data-carved-preview="${escapeHtml(file.file_path)}" style="padding:10px;border-bottom:1px solid var(--line)">
              <span class="node-icon">${fileIcon}</span>
              <div style="display:flex;flex-direction:column;min-width:0;flex:1">
                <strong style="font-size:12px;color:var(--text);overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escapeHtml(file.file_name)}</strong>
                <small style="font-size:10px;color:var(--muted)">Ofset: 0x${file.offset.toString(16).toUpperCase()} · ${formatBytes(file.size)}</small>
              </div>
            </div>
          `;
          })
          .join("");
      }
      showToast("Dosya kurtarma (carving) başarıyla tamamlandı.");
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      leftList.innerHTML = renderErrorPanel(t("analysis.carvingFailedTitle"), error);
      showToast("RAM dosya kurtarma başarısız: " + error.message, "error");
    }
    return;
  }

  if (action === "ram-processes") {
    const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
    if (!ramPath || ramPath.startsWith(".")) {
      showToast("Önce geçerli bir RAM dosyası seçin / Select a valid RAM file first", "error");
      return;
    }
    const osProfile = document.querySelector("#ram-os-profile")?.value || "windows";
    const symbolDir = ramSymbolDirValue();

    document.querySelector("#ram-analysis-results").style.display = "block";
    document.querySelector("#ram-split-view").style.display = "grid";
    document.querySelector("#ram-flat-results-panel").style.display = "none";

    document.querySelector("#ram-left-panel-title").textContent = t("analysis.lblProcesses");
    document.querySelector("#ram-right-panel-title").textContent =
      "Proses Detayları / Process Inspector";

    const leftList = document.querySelector("#ram-left-list");
    leftList.innerHTML = ramConsoleHtml("Canlı Proses Analiz Konsolu", [
      "Volatility3 ile proses tablosu çıkartılıyor...",
    ]);
    const rightContent = document.querySelector("#ram-right-content");
    rightContent.innerHTML = `<div class="log-box" style="display:flex;align-items:center;justify-content:center;color:var(--muted);text-align:center">Proses seçildiğinde bellek haritası ve arama alanları burada açılacak.<br/>Select a process from the left to inspect memory maps.</div>`;

    const statusLbl = document.querySelector("#stat-status-lbl");
    if (statusLbl) statusLbl.textContent = t("analysis.runningAnalysis");

    try {
      const start = await apiRequest("/api/ram-list-processes-start", {
        method: "POST",
        body: JSON.stringify({ path: ramPath, os_type: osProfile, symbol_dir: symbolDir }),
      });
      if (!start.job_id) throw new Error(t("workflow.jobIdMissing"));
      const result = await waitForAcquisitionJob(start.job_id, {
        onUpdate(job) {
          updateRamConsole("#ram-left-list", job, "Canlı Proses Analiz Konsolu");
          if (statusLbl && job.message) statusLbl.textContent = job.message;
        },
      });

      const count = result.length || 0;
      document.querySelector("#stat-strings-count").textContent = "0";
      document.querySelector("#stat-carved-count").textContent = "0";
      document.querySelector("#stat-procs-count").textContent = count;
      if (statusLbl) statusLbl.textContent = "Hazır / Ready";

      if (count === 0) {
        leftList.innerHTML = `<div class="log-box" style="text-align:center;padding:12px;color:var(--muted)">Bu arşivde proses bulunamadı (Sadece .tar arşivleri desteklenir) / No processes.</div>`;
      } else {
        leftList.innerHTML = result
          .map(
            (proc) => `
          <div class="proc-row" data-pid="${escapeHtml(proc.pid)}" data-name="${escapeHtml(proc.name)}">
            <strong>${escapeHtml(proc.pid)}</strong>
            <span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escapeHtml(proc.name)}</span>
            <small style="text-align:right">${formatBytes(proc.dump_size)}</small>
          </div>
        `
          )
          .join("");
      }
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      leftList.innerHTML = renderErrorPanel(t("analysis.processFailedTitle"), error);
      showToast("Proses listeleme başarısız: " + error.message, "error");
    }
    return;
  }

  if (action === "hash") {
    await calculateHashes();
    return;
  }

  if (action === "compare") {
    compareHash();
    return;
  }

  if (action === "save-settings") {
    try {
      await persistSettingsFromControls();
    } catch (error) {
      const message = `Ayarlar kaydedilemedi: ${error.message}`;
      setStatus("[data-settings-status]", `${icon("info")} ${escapeHtml(message)}`);
      showToast(message, "error");
    }
    return;
  }

  if (action === "about-check-update") {
    button.classList.add("is-checking");
    const lang = state.language || "en";
    const isTr = lang === "tr";
    try {
      let result = null;
      if (backendReady()) {
        try {
          result = await apiRequest("/api/update-check");
        } catch (_) {}
      }
      if (!result || (!result.tag_name && !result.name && !result.version)) {
        const controller = new AbortController();
        const tid = setTimeout(() => controller.abort(), 25000);
        const res = await fetch("https://download.amele.noirlang.tr/version.json", {
          signal: controller.signal,
        });
        clearTimeout(tid);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const vdata = await res.json();
        const vStr = vdata.version || "0.1.1";
        const tag = vStr.startsWith("v") ? vStr : `v${vStr}`;
        result = {
          tag_name: tag,
          name: `Amele ${tag}`,
          version: vStr,
          html_url: "https://amele.noirlang.tr",
          mandatory: vdata.mandatory,
        };
      }

      state.latestUpdate = result;
      const latestTag = (result.tag_name || result.name || "").trim();
      const releaseUrl = result.html_url || "https://github.com/noirlang/amele/releases/latest";

      const parseVer = (v) => v.replace(/^v/i, "").split("-")[0].split(".").map(Number);
      const [cMaj, cMin, cPatch] = parseVer(APP_VERSION);
      const [lMaj, lMin, lPatch] = parseVer(latestTag);
      const hasUpdate =
        lMaj > cMaj ||
        (lMaj === cMaj && lMin > cMin) ||
        (lMaj === cMaj && lMin === cMin && lPatch > cPatch);

      if (hasUpdate) {
        showToast(
          isTr
            ? `Yeni sürüm mevcut: ${latestTag}! (Mevcut: ${APP_VERSION})`
            : `New update available: ${latestTag}! (Current: ${APP_VERSION})`,
          "success"
        );
        showUpdateToast({ latestTag, releaseUrl, isTr });
      } else {
        showToast(
          isTr
            ? `Amele güncel! En son sürümü kullanıyorsunuz (${APP_VERSION}).`
            : `Amele is up to date! You are on the latest release (${APP_VERSION}).`,
          "info"
        );
      }
    } catch (error) {
      showToast(
        isTr
          ? `Güncelleme kontrolü başarısız: ${error.message}`
          : `Update check failed: ${error.message}`,
        "error"
      );
    } finally {
      setTimeout(() => button.classList.remove("is-checking"), 600);
    }
    return;
  }

  if (action === "check-update") {
    button.classList.add("is-checking");
    const lang = state.language || "en";
    const isTr = lang === "tr";
    try {
      setStatus(
        "[data-update-status]",
        `<span class="status-spinner">${icon("refresh")}</span> <span>${t("settings.updateChecking") || (isTr ? "Güncellemeler denetleniyor..." : "Checking for updates...")}</span>`
      );
      let result = null;
      if (backendReady()) {
        try {
          result = await apiRequest("/api/update-check");
        } catch (_) {}
      }
      if (!result || (!result.tag_name && !result.name && !result.version)) {
        const controller = new AbortController();
        const tid = setTimeout(() => controller.abort(), 25000);
        const res = await fetch("https://download.amele.noirlang.tr/version.json", {
          signal: controller.signal,
        });
        clearTimeout(tid);
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const vdata = await res.json();
        const vStr = vdata.version || "0.1.1";
        const tag = vStr.startsWith("v") ? vStr : `v${vStr}`;
        result = {
          tag_name: tag,
          name: `Amele ${tag}`,
          version: vStr,
          html_url: "https://amele.noirlang.tr",
          mandatory: vdata.mandatory,
        };
      }

      state.latestUpdate = result;
      state.updateTarget = result.update_target || state.updateTarget;

      const latestTag = (result.tag_name || result.name || "").trim();
      const releaseUrl = result.html_url || "https://github.com/noirlang/amele/releases/latest";

      const parseVer = (v) => v.replace(/^v/i, "").split("-")[0].split(".").map(Number);
      const [cMaj, cMin, cPatch] = parseVer(APP_VERSION);
      const [lMaj, lMin, lPatch] = parseVer(latestTag);
      const hasUpdate =
        lMaj > cMaj ||
        (lMaj === cMaj && lMin > cMin) ||
        (lMaj === cMaj && lMin === cMin && lPatch > cPatch);

      if (hasUpdate) {
        state.updateAvailable = { latestTag, releaseUrl };
        showToast(
          isTr
            ? `Yeni sürüm mevcut: ${latestTag}! (Mevcut: ${APP_VERSION})`
            : `New update available: ${latestTag}! (Current: ${APP_VERSION})`,
          "success"
        );
        showUpdateToast({ latestTag, releaseUrl, isTr });
        setStatus(
          "[data-update-status]",
          `<span class="status-icon-update">${icon("rocket")}</span> <span>${isTr ? "Yeni sürüm:" : "New version:"} <b>${escapeHtml(latestTag)}</b></span>`
        );
        const resultArea = document.querySelector("[data-update-result]");
        if (resultArea) {
          resultArea.style.display = "block";
          const dl = resultArea.querySelector("[data-action='download-update']");
          if (dl) dl.style.display = "inline-flex";
        }
      } else {
        state.updateAvailable = null;
        showToast(
          isTr
            ? `Amele güncel! En son sürümü kullanıyorsunuz (${APP_VERSION}).`
            : `Amele is up to date! You are on the latest release (${APP_VERSION}).`,
          "info"
        );
        setStatus(
          "[data-update-status]",
          `<span class="status-icon-success">${icon("check")}</span> <span>${isTr ? "Amele güncel" : "Amele is up to date"} (${APP_VERSION})</span>`
        );
      }
    } catch (error) {
      showToast(t("settings.updateFailed", { message: error.message }), "error");
      setStatus(
        "[data-update-status]",
        `<span class="status-icon-error">${icon("alert-circle")}</span> <span>${escapeHtml(error.message)}</span>`
      );
    } finally {
      setTimeout(() => button.classList.remove("is-checking"), 600);
    }
    return;
  }

  if (action === "download-update") {
    await downloadUpdatePackage();
    return;
  }

  if (action === "list-files") {
    await listEvidenceFiles();
    return;
  }

  if (action === "refresh-cases") {
    await loadEvidenceCases({ silent: false });
    return;
  }

  if (action === "create-case") {
    await createEvidenceCase();
    return;
  }

  if (action === "create-manifest") {
    await createCaseManifest();
    return;
  }

  if (action === "refresh-cases") {
    await loadEvidenceCases({ silent: false });
    return;
  }

  if (action === "list-files") {
    await listEvidenceFiles();
    return;
  }

  if (action === "load-history" || action === "refresh-history") {
    await loadAcquisitionHistory({ silent: false });
    return;
  }

  if (action === "add-note") {
    await addEvidenceNote();
    return;
  }

  if (action === "create-report") {
    await createEvidenceReport();
    return;
  }

  if (action === "list-reports") {
    await listEvidenceReports();
    return;
  }

  if (action === "refresh-logs") {
    await loadEvidenceLogs();
    return;
  }

  if (action === "run-hash") {
    await calculateHashInOther();
    return;
  }
  if (action === "timestamp-hash") {
    await timestampHashInOther();
    return;
  }

  const label = button.textContent.trim().replace(/\s+/g, " ");
  writeWorkflowLog(`${label}: ${t("ready")}`);
  showToast(`${label}: ${t("ready")}`);
}

function renderErrorPanel(title, errorOrMessage) {
  const message = errorOrMessage?.message || errorOrMessage || t("unknown");
  return errorBoxHtml(title, message);
}

function installUiErrorHandlers() {
  window.addEventListener("error", (event) => {
    const location = event.filename
      ? `${event.filename}:${event.lineno || 0}:${event.colno || 0}`
      : "bilinmeyen dosya";
    showToast(`Arayüz hatası: ${event.message}\nKonum: ${location}`, "error");
  });
  window.addEventListener("unhandledrejection", (event) => {
    const reason = event.reason?.message || event.reason || "Bilinmeyen hata";
    showToast(`Arayüz işlemi tamamlanamadı:\n${reason}`, "error");
  });
}

async function pickFile(targetSelector) {
  const target = targetSelector ? document.querySelector(targetSelector) : null;
  if (backendReady()) {
    try {
      const result = await apiRequest("/api/pick-file", { method: "POST" });
      if (target) {
        target.value = result.path;
        delete state.files[targetSelector];
      }
      showToast(t("workflow.selectFile", { path: result.path }));
      return result.path;
    } catch (error) {
      if (String(error?.message || "").includes("cancelled")) return null;
      showToast(t("workflow.filePickerFailed", { message: error.message }), "error");
      return null;
    }
  }

  try {
    if (window.showOpenFilePicker) {
      const [handle] = await window.showOpenFilePicker({ multiple: false });
      const file = await handle.getFile();
      if (target) {
        target.value = file.name;
        state.files[targetSelector] = file;
      }
      showToast(t("workflow.selectFile", { path: file.name }));
      return file;
    }
  } catch (error) {
    if (error?.name === "AbortError") return null;
    showToast(t("workflow.filePickerFailedShort"), "error");
    return null;
  }

  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.style.position = "fixed";
    input.style.opacity = "0";
    input.addEventListener("change", () => {
      const file = input.files?.[0] || null;
      if (file && target) {
        target.value = file.name;
        state.files[targetSelector] = file;
        showToast(t("workflow.selectFile", { path: file.name }));
      }
      input.remove();
      resolve(file);
    });
    document.body.appendChild(input);
    input.click();
  });
}

async function pickFolder(targetSelector) {
  const target = targetSelector ? document.querySelector(targetSelector) : null;
  if (backendReady()) {
    try {
      const result = await apiRequest("/api/pick-folder", { method: "POST" });
      if (target) {
        target.value = result.path;
        if (targetSelector === "#ios-backup-path") syncIosBackupPathInput(target, state);
      }
      showToast(t("workflow.selectFolder", { path: result.path }));
      return result.path;
    } catch (error) {
      if (String(error?.message || "").includes("cancelled")) return null;
      showToast(t("workflow.folderPickerFailed", { message: error.message }), "error");
      return null;
    }
  }

  try {
    if (window.showDirectoryPicker) {
      const handle = await window.showDirectoryPicker();
      if (target) {
        target.value = handle.name;
        if (targetSelector === "#ios-backup-path") syncIosBackupPathInput(target, state);
      }
      showToast(t("workflow.selectFolder", { path: handle.name }));
      return handle;
    }
  } catch (error) {
    if (error?.name === "AbortError") return null;
    showToast(t("workflow.folderPickerFailedShort"), "error");
    return null;
  }

  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.webkitdirectory = true;
    input.style.position = "fixed";
    input.style.opacity = "0";
    input.addEventListener("change", () => {
      const first = input.files?.[0];
      const folder = first?.webkitRelativePath?.split("/")?.[0] || first?.name || "";
      if (folder && target) {
        target.value = folder;
        if (targetSelector === "#ios-backup-path") syncIosBackupPathInput(target, state);
      }
      if (folder) showToast(t("workflow.selectFolder", { path: folder }));
      input.remove();
      resolve(folder);
    });
    document.body.appendChild(input);
    input.click();
  });
}

function writeWorkflowLog(message) {
  const next = compactLogLine(message);
  if (!next) return;
  if (state.lastLog[0] !== next) state.lastLog.unshift(next);
  state.lastLog = state.lastLog.slice(0, 5);
  const log = document.querySelector("#workflow-log");
  if (log) log.innerHTML = state.lastLog.map((line) => escapeHtml(line)).join("<br />");
  updateSide("last-action", escapeHtml(next));
}

function updateSide(key, value) {
  const item = document.querySelector(`[data-side="${key}"] small`);
  if (item) item.innerHTML = value;
}

// edinim surerken is log'unun tamamini konsol kutusuna basar, alta kaydirir.
function renderAcquisitionConsole(job, extraLines = []) {
  const box = document.querySelector("#workflow-log");
  if (!box) return;
  const logs = Array.isArray(job?.logs) ? job.logs : [];
  const lines = [...logs, ...extraLines].filter((line) => String(line ?? "").trim() !== "");
  if (!lines.length) return;
  box.innerHTML = lines.map((line) => escapeHtml(line)).join("<br />");
  box.scrollTop = box.scrollHeight;
}

// saniyeyi kisa sure metnine cevirir.
function formatJobDuration(totalSecs, isEn) {
  const secs = Math.max(0, Math.round(Number(totalSecs) || 0));
  if (secs < 60) return isEn ? `${secs} sec` : `${secs} sn`;
  if (secs < 3600) {
    const mins = Math.floor(secs / 60);
    const rest = secs % 60;
    return isEn ? `${mins} min ${rest} sec` : `${mins} dk ${rest} sn`;
  }
  const hours = Math.floor(secs / 3600);
  const mins = Math.floor((secs % 3600) / 60);
  return isEn ? `${hours} h ${mins} min` : `${hours} sa ${mins} dk`;
}

function setAcquisitionControlsVisible(active, startButton) {
  const controls = document.querySelector("[data-acquisition-controls]");
  if (controls) controls.hidden = !active;
  const start = startButton || document.querySelector("[data-action='start']");
  if (start) {
    start.hidden = active;
    start.disabled = active;
  }
}

async function scanTargets() {
  const routeId = state.route.split(":")[1];
  const workflow = workflows[routeId];
  const isRam = workflow?.mode.includes("ram");

  if (isRam) {
    if (backendReady()) {
      try {
        const toolKey = workflow.platform === "Windows" ? "winpmem" : "avml";
        let status;
        if (workflow.mode.startsWith("remote")) {
          const payload = connectionPayload();
          if (!payload.ip || !payload.port) {
            showToast(t("connection.ipPortRequired"), "error");
            return;
          }
          if (!requireActiveConnection(workflow, payload)) return;
          const result = await apiRequest("/api/remote-tool-check", {
            method: "POST",
            body: JSON.stringify({ ...payload, tool: toolKey }),
          });
          status = result.status;
          updateSide(
            "connection",
            t("connection.checked", { host: `${payload.ip}:${payload.port}` })
          );
        } else {
          const result = await apiRequest("/api/ram-status");
          status = result[toolKey];
        }
        const toolName = workflow.platform === "Windows" ? "WinPMEM" : "AVML";
        const statusMessage = String(status?.message || "");
        const missingTool =
          status?.tool_present === false || /not found|bulunamad/i.test(statusMessage);
        if (missingTool) {
          updateSide("target", t("scan.toolMissing", { tool: toolName }));
          writeWorkflowLog(t("scan.toolMissing", { tool: toolName }));
          showToast(t("scan.toolMissing", { tool: toolName }), "error");
          return;
        }
        if (status?.tool_path) {
          state.ramToolPath = status.tool_path;
        }
        const label = status?.tool_path || statusMessage || t("scan.toolReady", { tool: toolName });
        updateSide("target", escapeHtml(label));
        writeWorkflowLog(
          t("scan.toolDoneLog", { target: toolName, message: statusMessage || t("ready") })
        );
        showToast(t("scan.toolReady", { tool: toolName }));
      } catch (error) {
        if (workflow?.mode.startsWith("remote")) {
          forgetConnection();
          updateSide("connection", t("connection.toolFailed"));
        }
        showToast(t("scan.failed", { message: error.message }), "error");
        writeWorkflowLog(t("scan.ramFailedLog", { message: error.message }));
      }
      return;
    }

    updateSide("target", t("localCheckWaiting"));
    writeWorkflowLog(t("scan.appModeRequired"));
    showToast(t("workflow.appModeRequired"), "error");
    return;
  }

  const select = document.querySelector("[data-field='target']");
  if (!select) return;

  if (backendReady()) {
    try {
      let disks = [];
      if (workflow.mode.startsWith("ssh")) {
        const payload = sshPayload();
        if (workflow.mode.includes("ram")) {
          const result = await apiRequest("/api/ssh-tool-check", {
            method: "POST",
            body: JSON.stringify(payload),
          });
          const status = result.status || {};
          updateSide("target", status.tool_path || "AVML / kcore");
          updateSide(
            "connection",
            t("connection.alive", { host: `${payload.ip}:${payload.port}` })
          );
          writeWorkflowLog(`SSH RAM Kontrolü: ${status.message || "Hazır"}`);
          showToast(`SSH RAM kontrolü: ${status.message || "Hazır"}`);
          return;
        } else {
          const result = await apiRequest("/api/ssh-disks", {
            method: "POST",
            body: JSON.stringify(payload),
          });
          disks = result.disks || [];
          updateSide(
            "connection",
            t("connection.alive", { host: `${payload.ip}:${payload.port}` })
          );
        }
      } else if (workflow.mode.startsWith("remote")) {
        const payload = connectionPayload();
        if (!payload.ip || !payload.port) {
          showToast(t("connection.ipPortRequired"), "error");
          return;
        }
        if (!requireActiveConnection(workflow, payload)) return;
        const result = await apiRequest("/api/remote-disks", {
          method: "POST",
          body: JSON.stringify(payload),
        });
        disks = result.disks || [];
        updateSide("connection", t("connection.alive", { host: `${payload.ip}:${payload.port}` }));
      } else {
        const result = await apiRequest("/api/disk-list");
        disks = result.disks || [];
        if (result.elevated) {
          writeWorkflowLog(t("scan.elevated"));
        } else if (result.elevation_error) {
          writeWorkflowLog(t("scan.elevationFailed", { message: result.elevation_error }));
          showToast(t("scan.elevationFailed", { message: result.elevation_error }), "error");
        }
      }

      const options = disks
        .map((disk) => {
          const value = disk.id || disk.device || disk.name || disk.path || "";
          if (!value) return "";
          const size = disk.boyut || disk.total_size || 0;
          const name = disk.ad || disk.device || disk.name || value;
          const access = disk.accessible === false ? ` ${t("scan.accessDenied")}` : "";
          return `<option value="${escapeHtml(value)}" data-disk-name="${escapeHtml(name)}">${escapeHtml(name)} · ${formatBytes(size)}${access}</option>`;
        })
        .filter(Boolean);

      if (options.length === 0) {
        select.innerHTML = `<option value="" disabled selected>${t("scan.noDisk")}</option>`;
        updateSide("target", t("targetNotSelected"));
        writeWorkflowLog(t("scan.noDiskLog"));
        showToast(t("scan.noDisk"), "error");
        return;
      }

      select.innerHTML = options.join("");
      updateSide("target", select.value);
      writeWorkflowLog(t("scan.diskDoneLog"));
      showToast(t("scan.diskDone"));
      return;
    } catch (error) {
      if (workflow?.mode.startsWith("remote")) {
        forgetConnection();
        updateSide("connection", t("connection.disksFailed"));
      }
      showToast(t("scan.diskFailed", { message: error.message }), "error");
      writeWorkflowLog(t("scan.diskFailed", { message: error.message }));
      return;
    }
  }

  const tauriInvoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke;
  if (tauriInvoke) {
    try {
      const disks = await tauriInvoke(
        workflow.mode.startsWith("remote") ? "remote_disk_list" : "local_disk_list",
        {
          platform: workflow.platform.toLowerCase(),
        }
      );
      const targets = Array.isArray(disks)
        ? disks.map((disk) => disk.id || disk.name || disk.path || disk).filter(Boolean)
        : [];
      if (targets.length > 0) {
        select.innerHTML = targets
          .map(
            (target) =>
              `<option value="${escapeHtml(target)}" data-disk-name="${escapeHtml(target)}">${escapeHtml(target)}</option>`
          )
          .join("");
        updateSide("target", targets[0]);
        writeWorkflowLog(t("scan.diskDoneLog"));
        showToast(t("scan.diskDone"));
        return;
      }
    } catch (error) {
      showToast(t("scan.diskFailedShort"), "error");
      writeWorkflowLog(t("scan.diskFailed", { message: error?.message || error }));
      return;
    }
  }

  select.innerHTML = `<option value="" disabled selected>${t("scan.waiting")}</option>`;
  updateSide("target", t("targetNotSelected"));
  writeWorkflowLog(t("scan.appModeRequired"));
  showToast(t("scan.completed"));
}

async function installAvml(button) {
  const workflow = currentWorkflow();
  if (!workflow || workflow.platform !== "Linux" || workflow.mode !== "local-ram") {
    showToast(t("workflow.avmlUnsupported"), "error");
    return;
  }
  if (!backendReady()) {
    showToast(t("workflow.appModeRequired"), "error");
    return;
  }

  button.disabled = true;
  writeWorkflowLog(t("workflow.avmlInstalling"));
  updateSide("last-action", t("workflow.avmlInstalling"));
  try {
    const result = await apiRequest("/api/avml-install", { method: "POST" });
    const status = result.status || {};
    const path = status.tool_path || result.path || "/usr/bin/avml";
    const label = status.message || result.message || "AVML ready";
    updateSide("target", escapeHtml(path));
    writeWorkflowLog(t("scan.toolDoneLog", { target: "AVML", message: escapeHtml(label) }));
    writeWorkflowLog(t("workflow.avmlInstalled", { path: escapeHtml(path) }));
    showToast(t("workflow.avmlInstalled", { path }));
  } catch (error) {
    writeWorkflowLog(t("workflow.avmlInstallFailed", { message: escapeHtml(error.message) }));
    showToast(t("workflow.avmlInstallFailed", { message: error.message }), "error");
  } finally {
    button.disabled = false;
  }
}

async function installWinpmem(button) {
  const workflow = currentWorkflow();
  if (!workflow || workflow.platform !== "Windows" || workflow.mode !== "local-ram") {
    showToast(t("workflow.winpmemUnsupported"), "error");
    return;
  }
  if (!backendReady()) {
    showToast(t("workflow.appModeRequired"), "error");
    return;
  }

  button.disabled = true;
  writeWorkflowLog(t("workflow.winpmemInstalling"));
  updateSide("last-action", t("workflow.winpmemInstalling"));
  setProgress(0, "0%");
  try {
    const start = await apiRequest("/api/winpmem-install", { method: "POST" });
    if (!start.job_id) throw new Error(t("workflow.jobIdMissing"));

    // Wait for the download/install job to finish
    const result = await waitForAcquisitionJob(start.job_id);

    const status = result.status || {};
    const path = status.tool_path || result.path || "C:\\Tools\\winpmem.exe";
    const label = status.message || result.message || "WinPMEM ready";
    updateSide("target", escapeHtml(path));
    writeWorkflowLog(t("scan.toolDoneLog", { target: "WinPMEM", message: escapeHtml(label) }));
    writeWorkflowLog(t("workflow.winpmemInstalled", { path: escapeHtml(path) }));
    showToast(t("workflow.winpmemInstalled", { path }));
  } catch (error) {
    setProgress(0);
    writeWorkflowLog(t("workflow.winpmemInstallFailed", { message: escapeHtml(error.message) }));
    showToast(t("workflow.winpmemInstallFailed", { message: error.message }), "error");
  } finally {
    button.disabled = false;
  }
}

function setProgress(value, labelText = `${value}%`) {
  const progress = document.querySelector("[data-progress]");
  if (!progress) return;
  setProgressElement(progress, value, labelText);
}

function setProgressElement(progress, value, labelText = `${value}%`) {
  const numericValue = Math.max(0, Math.min(100, Number(value) || 0));
  const next = `${numericValue}%`;
  progress.style.setProperty("--value", next);
  const label = progress.querySelector("b");
  if (label) label.textContent = labelText;
}

function acquisitionPercent(job) {
  const done = Number(job?.done || 0);
  const total = Number(job?.total || 0);
  if (!Number.isFinite(done) || !Number.isFinite(total) || total <= 0) return 0;
  return Math.max(0, Math.min(100, Math.floor((done * 100) / total)));
}

async function waitForAcquisitionJob(jobId, options = {}) {
  while (true) {
    const job = await apiRequest("/api/acquisition-status", {
      method: "POST",
      body: JSON.stringify({ job_id: jobId }),
    });
    const customLabel = typeof options.onUpdate === "function" ? options.onUpdate(job) : null;
    const percent = acquisitionPercent(job);
    setProgress(
      percent,
      typeof customLabel === "string" && customLabel ? customLabel : `${percent}%`
    );
    if (job.message) updateSide("last-action", job.message);

    if (job.status === "completed") {
      setProgress(100, "100%");
      return job.result || {};
    }
    if (job.status === "failed") {
      throw new Error(job.error || job.message || t("acquisitionFailed"));
    }

    await new Promise((resolve) => window.setTimeout(resolve, 500));
  }
}

function ramConsoleHtml(title, logs = []) {
  const entries =
    Array.isArray(logs) && logs.length
      ? logs
      : ["Analiz başlatıldı. Volatility3 çıktısı bekleniyor..."];
  return `
    <div class="log-box ram-analysis-console">
      <strong>${escapeHtml(title)}</strong>
      <pre>${entries.map((line) => escapeHtml(line)).join("\n")}</pre>
    </div>
  `;
}

function updateRamConsole(selector, job, title = "Canlı Analiz Konsolu") {
  const container = document.querySelector(selector);
  if (!container) return;
  const logs = Array.isArray(job.logs) ? job.logs : [];
  container.innerHTML = ramConsoleHtml(title, logs);
  container.scrollTop = container.scrollHeight;
}

function ramSymbolDirValue() {
  const value = document.querySelector("#ram-symbol-dir")?.value.trim() || "";
  if (!value || value.startsWith(".")) return null;
  state.ramSymbolDirInput = value;
  return value;
}

function renderVolatilityPreflight(result) {
  const warnings = Array.isArray(result.warnings) ? result.warnings : [];
  const recommendations = Array.isArray(result.recommendations) ? result.recommendations : [];
  const banners = Array.isArray(result.banners) ? result.banners : [];
  const symbolDirs = Array.isArray(result.symbol_dirs) ? result.symbol_dirs : [];
  const matches = Array.isArray(result.matching_symbols) ? result.matching_symbols : [];
  const badge = result.ready
    ? `<span class="status-pill ok">${escapeHtml(t("analysis.preflightReady"))}</span>`
    : `<span class="status-pill danger">${escapeHtml(t("analysis.preflightMissing"))}</span>`;
  return `
    <div class="analysis-summary">
      <p class="section-label">${escapeHtml(t("analysis.preflightTitle"))}</p>
      <div class="summary-grid">
        <div><strong>${escapeHtml(t("analysis.preflightStatus"))}</strong><span>${badge}</span></div>
        <div><strong>vol.py</strong><span>${escapeHtml(result.vol_py || t("analysis.preflightNotFound"))}</span></div>
        <div><strong>${escapeHtml(t("analysis.preflightSymbols"))}</strong><span>${Number(result.symbol_count || 0)} ${escapeHtml(t("analysis.preflightTotal"))} · ${Number(result.linux_symbol_count || 0)} Linux</span></div>
        <div><strong>${escapeHtml(t("analysis.preflightMatch"))}</strong><span>${matches.length ? `${matches.length} symbol` : escapeHtml(t("analysis.preflightNoMatch"))}</span></div>
      </div>
      <div class="section-divider"></div>
      <div class="log-box">
        <strong>${escapeHtml(t("analysis.preflightBanners"))}</strong>
        <pre>${banners.length ? banners.map(escapeHtml).join("\n") : escapeHtml(t("analysis.preflightNoBanner"))}</pre>
      </div>
      <div class="log-box">
        <strong>${escapeHtml(t("analysis.preflightSymbolDirs"))}</strong>
        <pre>${symbolDirs.length ? symbolDirs.map(escapeHtml).join("\n") : escapeHtml(t("analysis.preflightNoSymbolDir"))}</pre>
      </div>
      ${warnings.length ? `<div class="log-box" style="color:#ffb4b4"><strong>${escapeHtml(t("analysis.preflightWarnings"))}</strong><pre>${warnings.map(escapeHtml).join("\n")}</pre></div>` : ""}
      ${recommendations.length ? `<div class="log-box"><strong>${escapeHtml(t("analysis.preflightRecommendations"))}</strong><pre>${recommendations.map(escapeHtml).join("\n")}</pre></div>` : ""}
    </div>
  `;
}

function renderVolatilitySymbolInstall(result) {
  const banners = Array.isArray(result.banners) ? result.banners : [];
  const matches = Array.isArray(result.matches) ? result.matches : [];
  const recommendations = Array.isArray(result.recommendations) ? result.recommendations : [];
  const preflight = result.preflight || null;
  const ready = Boolean(preflight?.ready || result.status === "windows-automatic");
  const badge = ready
    ? `<span class="status-pill ok">${escapeHtml(t("analysis.preflightReady"))}</span>`
    : `<span class="status-pill danger">${escapeHtml(t("analysis.preflightMissing"))}</span>`;
  const matchLines = matches.map((item) => {
    const remotePath = item.remote_path || item.url || "";
    return `${item.banner || "-"}\n  -> ${remotePath}`;
  });
  return `
    <div class="analysis-summary">
      <p class="section-label">${escapeHtml(t("analysis.symbolInstallTitle"))}</p>
      <div class="summary-grid">
        <div><strong>${escapeHtml(t("analysis.preflightStatus"))}</strong><span>${badge}</span></div>
        <div><strong>${escapeHtml(t("analysis.symbolDir"))}</strong><span>${escapeHtml(result.symbol_dir || "-")}</span></div>
        <div><strong>${escapeHtml(t("analysis.symbolInstallTarget"))}</strong><span>${escapeHtml(result.target || "-")}</span></div>
        <div><strong>SHA256</strong><span>${escapeHtml(result.sha256 || "-")}</span></div>
      </div>
      <div class="section-divider"></div>
      <div class="log-box">
        <strong>${escapeHtml(t("analysis.symbolInstallMessage"))}</strong>
        <pre>${escapeHtml(result.message || "-")}</pre>
      </div>
      <div class="log-box">
        <strong>${escapeHtml(t("analysis.preflightBanners"))}</strong>
        <pre>${banners.length ? banners.map(escapeHtml).join("\n") : escapeHtml(t("analysis.preflightNoBanner"))}</pre>
      </div>
      <div class="log-box">
        <strong>${escapeHtml(t("analysis.symbolInstallMatches"))}</strong>
        <pre>${matchLines.length ? matchLines.map(escapeHtml).join("\n") : escapeHtml(t("analysis.preflightNoMatch"))}</pre>
      </div>
      ${preflight ? renderVolatilityPreflight(preflight) : ""}
      ${recommendations.length ? `<div class="log-box"><strong>${escapeHtml(t("analysis.preflightRecommendations"))}</strong><pre>${recommendations.map(escapeHtml).join("\n")}</pre></div>` : ""}
    </div>
  `;
}

async function sendAcquisitionControl(action) {
  const active = state.activeAcquisition;
  if (!active || !active.jobId || !active.workflowId) {
    showToast(t("workflow.activeJobMissing"), "error");
    return;
  }
  const workflow = workflows[active.workflowId];
  const body = {
    job_id: active.jobId,
    action,
  };
  if (workflow?.mode.startsWith("remote")) {
    Object.assign(body, active.payload || {});
  }

  await apiRequest("/api/acquisition-control", {
    method: "POST",
    body: JSON.stringify(body),
  });
  const label =
    action === "stop"
      ? t("workflow.stopLabel")
      : action === "pause"
        ? t("workflow.pauseLabel")
        : t("workflow.resumeLabel");
  const message = workflow?.mode.startsWith("remote")
    ? t("workflow.controlSent", { label })
    : t("workflow.controlApplied", { label });
  writeWorkflowLog(message);
  updateSide("last-action", message);
  showToast(message);
}

async function startAcquisition(button) {
  const routeId = state.route.split(":")[1];
  const workflow = workflows[routeId];
  const isRam = workflow?.mode.includes("ram");
  let payload = null;
  const isSsh = workflow?.mode.startsWith("ssh");
  if (isSsh) {
    try {
      payload = sshPayload();
    } catch (error) {
      showToast(error.message, "error");
      return;
    }
  } else if (workflow?.mode.startsWith("remote")) {
    try {
      payload = connectionPayload();
    } catch (error) {
      showToast(error.message, "error");
      return;
    }
    if (!requireActiveConnection(workflow, payload)) return;
  }
  const target = isRam
    ? state.ramToolPath || document.querySelector("[data-field='target']")?.value.trim() || ""
    : document.querySelector("[data-field='target']")?.value.trim() || "";
  const selectedFormatMode = document.querySelector("[data-field='output-format']")?.value || "raw";
  let outputFormat = "raw";
  let sparseAcquisition = true;

  if (selectedFormatMode === "raw_full" || selectedFormatMode === "raw-full") {
    outputFormat = "raw";
    sparseAcquisition = false;
  } else if (selectedFormatMode === "raw_sparse" || selectedFormatMode === "raw-sparse") {
    outputFormat = "raw";
    sparseAcquisition = true;
  } else if (selectedFormatMode === "aff4_full" || selectedFormatMode === "aff4-full") {
    outputFormat = "aff4";
    sparseAcquisition = false;
  } else if (selectedFormatMode === "aff4_sparse" || selectedFormatMode === "aff4-sparse") {
    outputFormat = "aff4";
    sparseAcquisition = true;
  } else {
    outputFormat = selectedFormatMode.startsWith("aff4") ? "aff4" : "raw";
    sparseAcquisition =
      document.querySelector("[data-field='sparse-acquisition']")?.checked ?? true;
  }
  if (workflow && !workflow.mode.includes("ram") && !target) {
    showToast(t("workflow.diskRequired"), "error");
    return;
  }
  let output = document.querySelector("#workflow-output")?.value.trim() || "";
  const diskName = isRam ? "" : selectedTargetName();
  let caseName = null;
  button.disabled = true;
  window.clearInterval(state.jobs.workflow);
  setProgress(0, "0%");
  state.lastLog = [];
  const initialLogBox = document.querySelector("#workflow-log");
  if (initialLogBox) initialLogBox.innerHTML = "";
  const operation = isRam ? t("ramAcquisition") : t("imageAcquisition");
  // son is durumu catch blogundan da gorunsun diye try disinda tutulur.
  let lastJob = null;

  try {
    setAcquisitionControlsVisible(true, button);
    await loadEvidenceCases();
    const evidenceCase = await ensureImageCase();
    caseName = evidenceCase.case_name;
    if (isRam) {
      const remoteIp = workflow?.mode.startsWith("remote") || isSsh ? payload?.ip : "";
      const fileName = canonicalRamFileName(remoteIp, new Date(), workflow?.platform || "");
      const outputInput = document.querySelector("#workflow-output");
      if (outputInput) outputInput.value = fileName;
      const ramDir = evidenceCase.ram_dir || `${evidenceCase.case_dir}/ram`;
      output = `${ramDir}/${fileName}`;
    } else {
      output = evidenceCase.output_dir || `${evidenceCase.case_dir}/ciktilar`;
    }
    document.querySelectorAll("[data-case-output]").forEach((outputNode) => {
      outputNode.textContent =
        outputNode.dataset.caseOutputSubdir === "ram"
          ? evidenceCase.ram_dir || `${evidenceCase.case_dir}/ram`
          : evidenceCase.output_dir || `${evidenceCase.case_dir}/ciktilar`;
    });

    writeWorkflowLog(t("workflow.operationStarted", { operation }));
    updateSide("last-action", t("workflow.operationRunning", { operation }));
    if (workflow?.mode.startsWith("remote") || isSsh)
      updateSide("connection", t("workflow.operationRunning", { operation }));

    const start = isSsh
      ? await apiRequest(isRam ? "/api/ssh-ram" : "/api/ssh-image", {
          method: "POST",
          body: JSON.stringify(
            isRam
              ? {
                  ...payload,
                  output,
                  case_name: caseName,
                  output_format: outputFormat,
                }
              : {
                  ...payload,
                  disk_path: target,
                  output,
                  case_name: caseName,
                  output_format: outputFormat,
                }
          ),
        })
      : workflow?.mode.startsWith("remote")
        ? await apiRequest(isRam ? "/api/remote-ram" : "/api/remote-image", {
            method: "POST",
            body: JSON.stringify(
              isRam
                ? {
                    ...payload,
                    output,
                    case_name: caseName,
                    output_format: outputFormat,
                  }
                : {
                    ...payload,
                    disk_id: target,
                    disk_name: diskName,
                    output,
                    case_name: caseName,
                    output_format: outputFormat,
                    sparse: sparseAcquisition,
                  }
            ),
          })
        : await apiRequest(isRam ? "/api/local-ram" : "/api/local-image", {
            method: "POST",
            body: JSON.stringify(
              isRam
                ? {
                    output,
                    tool: workflow.platform === "Windows" ? "winpmem" : "avml",
                    tool_path: target || state.ramToolPath || undefined,
                    case_name: caseName,
                    output_format: outputFormat,
                  }
                : {
                    source: target,
                    disk_name: diskName,
                    output,
                    case_name: caseName,
                    output_format: outputFormat,
                    sparse: sparseAcquisition,
                  }
            ),
          });
    if (!start.job_id) throw new Error(t("workflow.jobIdMissing"));
    state.activeAcquisition = {
      jobId: start.job_id,
      workflowId: routeId,
      payload,
    };
    // hiz ve kalan sure icin bir onceki orneklem burada tutulur.
    let lastSample = null;
    const isEn = state.language === "en";
    const result = await waitForAcquisitionJob(start.job_id, {
      onUpdate: (job) => {
        lastJob = job;
        renderAcquisitionConsole(job);
        const done = Number(job?.done || 0);
        const total = Number(job?.total || 0);
        const now = Date.now();
        if (!total) return job?.message || null;
        let extra = "";
        if (lastSample && now - lastSample.t > 400 && done >= lastSample.done) {
          const dBytes = done - lastSample.done;
          const dt = (now - lastSample.t) / 1000;
          if (dBytes > 0 && dt > 0) {
            const speed = dBytes / dt;
            const eta = Math.round((total - done) / speed);
            if (Number.isFinite(eta) && eta >= 0 && done < total) {
              extra = ` • ${formatBytes(speed)}/sn • ${t("workflow.remaining")} ${formatJobDuration(eta, isEn)}`;
            }
          }
        }
        if (!lastSample || now - lastSample.t > 400) lastSample = { done, t: now };
        const pct = acquisitionPercent(job);
        const phasePrefix = job?.phase
          ? `${job.phase} • `
          : job?.message && (job.message.includes("SHA") || job.message.includes("hash"))
            ? "SHA-256 hesaplanıyor • "
            : "";
        return `${phasePrefix}${formatBytes(done)} / ${formatBytes(total)} • %${pct}${extra}`;
      },
    });

    setProgress(100, "100%");
    const targetPath = result.target_path || result.target || output;
    const doneLines = [t("workflow.operationCompletedPath", { operation, path: targetPath })];
    writeWorkflowLog(t("workflow.operationCompletedPath", { operation, path: targetPath }));
    if (result.sha256) {
      doneLines.push(t("workflow.hashWritten", { hash: result.sha256 }));
      writeWorkflowLog(t("workflow.hashWritten", { hash: escapeHtml(result.sha256) }));
    }
    if (result.output_format) {
      doneLines.push(
        t("workflow.formatCompleted", { format: String(result.output_format).toUpperCase() })
      );
      writeWorkflowLog(
        t("workflow.formatCompleted", { format: String(result.output_format).toUpperCase() })
      );
    }
    // ozet satirlari konsolun en altina eklenir, tam log korunur.
    renderAcquisitionConsole(lastJob, doneLines);
    updateSide("last-action", t("workflow.operationCompleted", { operation }));
    if (workflow?.mode.startsWith("remote") && payload) {
      updateSide("connection", t("connection.connected", { ip: payload.ip }));
    }
    showToast(t("workflow.operationCompleted", { operation }));
  } catch (error) {
    setProgress(0, "0%");
    writeWorkflowLog(t("workflow.operationFailedDetail", { operation, message: error.message }));
    renderAcquisitionConsole(lastJob, [
      t("workflow.operationFailedDetail", { operation, message: error.message }),
    ]);
    updateSide("last-action", t("workflow.operationFailed", { operation }));
    if (workflow?.mode.startsWith("remote")) {
      updateSide("connection", t("workflow.operationFailed", { operation }));
    }
    showToast(t("workflow.operationFailedDetail", { operation, message: error.message }), "error");
  } finally {
    state.activeAcquisition = null;
    setAcquisitionControlsVisible(false, button);
  }
}

function setAnalysisStatus(status, log) {
  const statusNode = document.querySelector("[data-analysis-status]");
  const logNode = document.querySelector("[data-analysis-log]");
  if (statusNode) statusNode.textContent = status;
  state.imageMountLogHTML = log || "";
  if (logNode) {
    logNode.innerHTML = log || "";
    logNode.style.display = log ? "" : "none";
  }
}

function renderMountResultInfo(result) {
  const mode = result.mount_mode || "mounted";
  const analysis = result.analysis || {};
  const filesystems = Array.isArray(analysis.filesystems) ? analysis.filesystems : [];
  const partitions = Array.isArray(analysis.partitions) ? analysis.partitions : [];
  const status =
    mode === "analysis-only" ? t("analysis.analysisOnlyLog") : t("analysis.mountedLog");
  const details = [
    analysis.image_type ? `${t("analysis.imageType")}: ${analysis.image_type}` : "",
    analysis.size ? `${t("analysis.imageSize")}: ${formatBytes(analysis.size)}` : "",
    analysis.partition_scheme
      ? `${t("analysis.partitionScheme")}: ${analysis.partition_scheme}`
      : "",
    `${t("analysis.partitionCount")}: ${partitions.length}`,
    `${t("analysis.filesystemCount")}: ${filesystems.length}`,
  ].filter(Boolean);
  return `
    <div class="mount-info-panel">
      <strong>${escapeHtml(status)}</strong>
      ${result.mount_error ? `<pre>${escapeHtml(result.mount_error)}</pre>` : ""}
      ${details.length ? `<div class="mount-info-grid">${details.map((item) => `<span>${escapeHtml(item)}</span>`).join("")}</div>` : ""}
    </div>
  `;
}

function renderDiskAnalysisSummary(result) {
  const partitions = Array.isArray(result.partitions) ? result.partitions : [];
  const filesystems = Array.isArray(result.filesystems) ? result.filesystems : [];
  const mounted = result.mounted || null;
  return `
    <p class="section-label">${t("analysis.diskSummary")}</p>
    <div class="hash-grid">
      ${analysisMetric("İmaj", escapeHtml(result.image_type || "-"))}
      ${analysisMetric("Boyut", formatBytes(result.size || 0))}
      ${analysisMetric("Bölüm", escapeHtml(result.partition_scheme || "-"))}
      ${analysisMetric("FS", String(filesystems.length))}
    </div>
    ${analysisList(
      "Bölümler",
      partitions.map(
        (part) =>
          `${part.index}. ${part.scheme} ${part.type_name} LBA ${part.start_lba} · ${formatBytes(part.size || 0)}`
      )
    )}
    ${analysisList(
      "Dosya sistemi imzaları",
      filesystems.map((fs) => `${fs.source}: ${fs.fs_type} @ ${fs.offset}`)
    )}
    ${
      mounted
        ? `
      <div class="section-divider"></div>
      <div class="hash-grid">
        ${analysisMetric("Dosya", String(mounted.file_count || 0))}
        ${analysisMetric("Klasör", String(mounted.directory_count || 0))}
        ${analysisMetric("Görünen veri", formatBytes(mounted.total_visible_bytes || 0))}
        ${analysisMetric("Taranan", String(mounted.scanned_entries || 0))}
      </div>
      ${analysisList(
        "Uzantılar",
        (mounted.top_extensions || []).map((item) => `${item.extension}: ${item.count}`)
      )}
      ${analysisList(
        "En büyük dosyalar",
        (mounted.largest_files || []).map((item) => `${item.path} · ${formatBytes(item.size || 0)}`)
      )}
    `
        : ""
    }
    ${analysisList("Uyarılar", result.warnings || [], "warning")}
    ${analysisList("Öneriler", result.recommendations || [])}
  `;
}

function renderRamAnalysisSummary(result) {
  return `
    <div class="hash-grid">
      ${analysisMetric("Tip", escapeHtml(result.dump_type || "-"))}
      ${analysisMetric("Boyut", formatBytes(result.size || 0))}
      ${analysisMetric("Entropi", Number(result.entropy_sample || 0).toFixed(2))}
      ${analysisMetric("IOC", String(result.string_match_count || 0))}
    </div>
    ${analysisList(
      "Kategori sayımları",
      (result.category_counts || []).map((item) => `${item.category}: ${item.count}`)
    )}
    ${analysisList(
      "Proses özeti",
      (result.largest_processes || []).map(
        (proc) => `${proc.pid} ${proc.name} · ${formatBytes(proc.dump_size || 0)}`
      )
    )}
    ${analysisList(
      "Örnek bulgular",
      (result.sample_matches || []).slice(0, 20).map(
        (item) =>
          `${item.category} @ 0x${Number(item.offset || 0)
            .toString(16)
            .toUpperCase()}: ${item.value}`
      )
    )}
    ${analysisList("Uyarılar", result.warnings || [], "warning")}
    ${analysisList("Öneriler", result.recommendations || [])}
  `;
}

function renderAndroidAnalysisSummary(result) {
  const profile = result.device_profile || {};
  const profileBits = [
    profile.manufacturer,
    profile.model,
    profile.android_release ? `Android ${profile.android_release}` : "",
    profile.security_patch ? `Patch ${profile.security_patch}` : "",
  ]
    .filter(Boolean)
    .join(" · ");
  return `
    <p class="section-label">${t("analysis.androidSummary")}</p>
    <div class="hash-grid">
      ${analysisMetric("Vaka", escapeHtml(result.case_name || "-"))}
      ${analysisMetric("Kayıt", String(result.record_count || 0))}
      ${analysisMetric("Timeline", String(result.timeline_event_count || 0))}
      ${analysisMetric("Korelasyon", String(result.correlation_count || 0))}
    </div>
    ${profileBits ? `<div class="side-info"><span class="metric-icon">${icon("android")}</span><span><strong>Cihaz</strong><small>${escapeHtml(profileBits)}</small></span></div>` : ""}
    ${analysisList(
      "Kayıt türleri",
      (result.record_types || []).map((item) => `${item.record_type}: ${item.count}`)
    )}
    ${analysisList(
      "Önemli timeline olayları",
      (result.recent_events || [])
        .slice(0, 15)
        .map((event) => `${event.type || "event"} [${event.severity || 0}] ${event.summary || ""}`)
    )}
    ${analysisList("Uçucu veri bölümleri", result.volatile_sections || [])}
    ${analysisList(
      "Dosyalar",
      (result.files || [])
        .slice(0, 20)
        .map((file) => `${file.name} · ${formatBytes(file.size || 0)}`)
    )}
    ${result.report_preview ? `<div class="section-divider"></div><pre class="log-box" style="white-space:pre-wrap;max-height:260px">${escapeHtml(result.report_preview)}</pre>` : ""}
    ${analysisList("Uyarılar", result.warnings || [], "warning")}
    ${analysisList("Öneriler", result.recommendations || [])}
  `;
}

function analysisMetric(label, value) {
  return `
    <div class="hash-result">
      <small>${escapeHtml(label)}</small>
      <strong>${value}</strong>
    </div>
  `;
}

function analysisList(title, items, tone = "") {
  const safeItems = Array.isArray(items) ? items.filter((item) => String(item || "").trim()) : [];
  if (!safeItems.length) return "";
  const color = tone === "warning" ? ' style="color:#ffb86b"' : "";
  return `
    <div class="section-divider"></div>
    <p class="section-label"${color}>${escapeHtml(title)}</p>
    <div class="strings-results-list">
      ${safeItems.map((item) => `<div class="string-match-item"><div class="match-value">${escapeHtml(String(item))}</div></div>`).join("")}
    </div>
  `;
}

function renderTree(node, depth = 0) {
  if (!node) return `<div class="log-box">${escapeHtml(t("analysis.outputWaiting"))}</div>`;
  const isVirtual = Boolean(node.virtual);
  const hasChildren = Array.isArray(node.children) && node.children.length > 0;
  const expanded = isVirtual && depth === 0;
  const fileIcon = node.is_dir ? "📁" : "📄";
  const toggle = node.is_dir ? `<span class="toggle-icon">${expanded ? "▾" : "▸"}</span>` : "";
  const sizeStr = node.is_dir ? "" : `<span class="node-size">${formatBytes(node.size)}</span>`;
  const note = node.note || node.name || "";

  let relativePath = node.path;
  if (!isVirtual && state.imageMount && state.imageMount.mountDir) {
    if (node.path.startsWith(state.imageMount.mountDir)) {
      relativePath = node.path.substring(state.imageMount.mountDir.length);
    }
  }

  const current = `
    <div class="tree-node" data-path="${escapeHtml(relativePath)}" data-is-dir="${node.is_dir}" data-virtual="${isVirtual}" data-has-children="${hasChildren}" data-note="${escapeHtml(note)}">
      <span style="width:16px;display:inline-block">${toggle}</span>
      <span class="node-icon">${fileIcon}</span>
      <span class="node-name">${escapeHtml(node.name || node.path.split("/").pop() || "/")}</span>
      ${sizeStr}
    </div>
    <div class="tree-children-container"></div>
  `;

  const children = hasChildren
    ? `<div class="tree-children" style="padding-left:14px; display:${expanded ? "block" : "none"}">${node.children.map((child) => renderTree(child, depth + 1)).join("")}</div>`
    : "";

  return current + children;
}

async function calculateHashes() {
  const inputPath = document.querySelector("#hash-file")?.value.trim();
  if (backendReady() && inputPath) {
    try {
      const hashes = await apiRequest("/api/hash", {
        method: "POST",
        body: JSON.stringify({
          path: inputPath,
          algorithms: ["md5", "sha1", "sha256", "sha512"],
        }),
      });
      setHashResult("md5", hashes.md5 || "-");
      setHashResult("sha1", hashes.sha1 || "-");
      setHashResult("sha256", hashes.sha256 || "-");
      setHashResult("sha512", hashes.sha512 || "-");
      showToast(t("hash.done"));
      return;
    } catch (error) {
      showToast(t("hash.failed", { message: error.message }), "error");
      return;
    }
  }

  const file = state.files["#hash-file"];
  if (!file) {
    showToast(t("fileRequired"), "error");
    return;
  }
  const buffer = await file.arrayBuffer();
  setHashResult("md5", t("hash.fullAppRequired"));
  setHashResult("sha1", await digestHex("SHA-1", buffer));
  setHashResult("sha256", await digestHex("SHA-256", buffer));
  setHashResult("sha512", await digestHex("SHA-512", buffer));
  showToast(t("hash.done"));
}

async function digestHex(algorithm, buffer) {
  const hash = await crypto.subtle.digest(algorithm, buffer.slice(0));
  return [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function setHashResult(key, value) {
  const node = document.querySelector(`[data-hash-result="${key}"] strong`);
  if (node) node.textContent = value;
}

function compareHash() {
  const expected = document.querySelector("[data-hash-expected]")?.value.trim().toLowerCase();
  const values = [...document.querySelectorAll("[data-hash-result] strong")].map((node) =>
    node.textContent.trim().toLowerCase()
  );
  const result = document.querySelector("[data-hash-compare-result] small");
  if (!expected) {
    showToast(t("hash.compareRequired"), "error");
    return;
  }
  const matched = values.includes(expected);
  if (result) result.textContent = matched ? t("hash.matched") : t("hash.notMatched");
  showToast(
    matched ? t("hash.matchedToast") : t("hash.notMatchedToast"),
    matched ? "success" : "error"
  );
}

function setStatus(selector, html) {
  const node = document.querySelector(selector);
  if (node) {
    node.innerHTML = html;
    node.style.display = html ? "inline-flex" : "none";
  }
}

async function createEvidenceCase() {
  const caseName = document.querySelector("#case-name")?.value.trim();
  if (!caseName) {
    showToast(t("case.required"), "error");
    return;
  }
  try {
    const result = await apiRequest("/api/evidence-create", {
      method: "POST",
      body: JSON.stringify({ case_name: caseName }),
    });
    state.activeCase = result;
    state.pendingCaseName = "";
    await loadEvidenceCases();
    setStatus(
      "[data-case-status]",
      `${icon("info")} ${t("case.created", { path: escapeHtml(result.case_dir) })}`
    );
    showToast(t("case.created", { path: result.case_dir }));
  } catch (error) {
    setStatus(
      "[data-case-status]",
      `${icon("info")} ${t("case.createFailed", { message: escapeHtml(error.message) })}`
    );
    showToast(t("case.createFailed", { message: error.message }), "error");
  }
}

async function listEvidenceFiles() {
  if (!state.activeCase) {
    showToast(t("case.required"), "error");
    return;
  }
  const subdir = document.querySelector("#case-folder")?.value || "ciktilar";
  try {
    const result = await apiRequest("/api/evidence-list-files", {
      method: "POST",
      body: JSON.stringify({ subdir }),
    });
    const files = result.files || [];
    const select = document.querySelector("#case-file-list");
    if (select) {
      select.innerHTML = files.length
        ? files
            .map(
              (file) =>
                `<option value="${escapeHtml(file.path)}">${escapeHtml(file.name)} · ${formatBytes(file.size)}</option>`
            )
            .join("")
        : `<option>${t("case.empty")}</option>`;
    }
    setStatus(
      "[data-case-status]",
      `${icon("info")} ${t("case.filesListed", { count: String(files.length) })}`
    );
    showToast(t("case.filesListed", { count: String(files.length) }));
  } catch (error) {
    showToast(t("case.listFailed", { message: error.message }), "error");
  }
}

async function createCaseManifest() {
  const caseName = state.pendingCaseName || state.activeCase?.case_name || "";
  if (!caseName) {
    showToast(t("case.required"), "error");
    return;
  }
  try {
    const result = await apiRequest("/api/evidence-manifest", {
      method: "POST",
      body: JSON.stringify({ case_name: caseName }),
    });
    if (state.activeCase) {
      state.activeCase.manifest_path = result.path || state.activeCase.manifest_path;
    }
    await loadEvidenceCases();
    setStatus(
      "[data-manifest-status]",
      `${icon("shield")} ${t("case.manifest.ready", { path: escapeHtml(result.path || "") })}`
    );
    showToast(t("case.manifest.created", { path: result.path || "" }));
  } catch (error) {
    setStatus(
      "[data-manifest-status]",
      `${icon("info")} ${t("case.manifest.failed", { message: escapeHtml(error.message) })}`
    );
    showToast(t("case.manifest.failed", { message: error.message }), "error");
  }
}

async function loadAcquisitionHistory({ silent = true } = {}) {
  if (!backendReady()) return;
  const caseName = resolveSelectedCaseName("#history-case", { fallbackToDefault: true });
  if (!caseName) {
    if (!silent) showToast(t("case.required"), "error");
    return;
  }
  try {
    const result = await apiRequest("/api/acquisition-history", {
      method: "POST",
      body: JSON.stringify({ case_name: caseName }),
    });
    state.acquisitionHistory = Array.isArray(result.history) ? result.history : [];
    const detail = document.querySelector("#other-detail");
    if (detail && state.activeTab === "history") {
      detail.innerHTML = boundDetailPanel(state.activeTab);
      hydrateIcons(detail);
    }
    if (!silent)
      showToast(
        t("acquisition.history.loaded", { count: String(state.acquisitionHistory.length) })
      );
  } catch (error) {
    if (!silent) showToast(t("acquisition.history.failed", { message: error.message }), "error");
  }
}

async function addEvidenceNote() {
  const note = document.querySelector("#report-note")?.value.trim();
  if (!note) {
    showToast(t("report.noteRequired"), "error");
    return;
  }
  const caseName = reportCaseName();
  try {
    const result = await apiRequest("/api/evidence-add-note", {
      method: "POST",
      body: JSON.stringify({ note, case_name: caseName }),
    });
    await loadEvidenceCases();
    setStatus(
      "[data-report-status]",
      `${icon("info")} ${t("report.noteAdded", { path: escapeHtml(result.path) })}`
    );
    showToast(t("report.noteAdded", { path: result.path }));
  } catch (error) {
    setStatus(
      "[data-report-status]",
      `${icon("info")} ${t("report.noteFailed", { message: escapeHtml(error.message) })}`
    );
    showToast(t("report.noteFailed", { message: error.message }), "error");
  }
}

async function createEvidenceReport() {
  const caseName = reportCaseName();
  const title = document.querySelector("#report-title")?.value.trim() || t("report.defaultTitle");
  const format = document.querySelector("#report-format")?.value || "txt";
  const description = document.querySelector("#report-note")?.value.trim() || "";
  try {
    const result = await apiRequest("/api/report-create", {
      method: "POST",
      body: JSON.stringify({ case_name: caseName, title, description, format }),
    });
    await loadEvidenceCases();
    setStatus(
      "[data-report-status]",
      `${icon("info")} ${t("report.created", { path: escapeHtml(result.path) })}`
    );
    showToast(t("report.created", { path: result.path }));
  } catch (error) {
    setStatus(
      "[data-report-status]",
      `${icon("info")} ${t("report.failed", { message: escapeHtml(error.message) })}`
    );
    showToast(t("report.failed", { message: error.message }), "error");
  }
}

async function calculateHashInOther() {
  const inputEl = document.querySelector("#hash-target-path");
  const path = inputEl?.value?.trim();
  if (!path) {
    showToast(t("hash.fileRequired"), "error");
    return;
  }

  const algSelect = document.querySelector("#hash-algorithm");
  const method = algSelect?.value || "sha256";
  state.hashTargetInput = path;
  state.hashMethod = method;

  const algorithms =
    method === "all"
      ? ["blake3", "sha256", "md5"]
      : method === "both"
        ? ["sha256", "md5"]
        : [method];
  setStatus("[data-hash-status]", `${icon("refresh")} ${t("hash.calculating")}`);
  try {
    const res = await apiRequest("/api/hash", {
      method: "POST",
      body: JSON.stringify({ path, algorithms }),
    });
    state.hashResult = {
      path,
      blake3: res.blake3,
      sha256: res.sha256,
      md5: res.md5,
    };
    const outBox = document.querySelector("[data-hash-output]");
    if (outBox) {
      let outHtml = `<strong>${t("hash.file") || "Dosya"}:</strong> ${escapeHtml(path)}<br/>`;
      if (res.blake3)
        outHtml += `<strong>BLAKE3:</strong> <code style="word-break:break-all">${escapeHtml(res.blake3)}</code><br/>`;
      if (res.sha256)
        outHtml += `<strong>SHA-256:</strong> <code style="word-break:break-all">${escapeHtml(res.sha256)}</code><br/>`;
      if (res.md5)
        outHtml += `<strong>MD5:</strong> <code style="word-break:break-all">${escapeHtml(res.md5)}</code><br/>`;
      outBox.innerHTML = outHtml;
    }
    setStatus("[data-hash-status]", `${icon("shield")} ${t("hash.done")}`);
    showToast(t("hash.done"));
  } catch (err) {
    setStatus(
      "[data-hash-status]",
      `${icon("info")} ${t("hash.failed", { message: escapeHtml(err.message) })}`
    );
    showToast(t("hash.failed", { message: err.message }), "error");
  }
}

async function timestampHashInOther() {
  const path = document.querySelector("#hash-target-path")?.value?.trim();
  const tsaUrl = document.querySelector("#hash-tsa-url")?.value?.trim();
  if (!path) {
    showToast(t("hash.fileRequired"), "error");
    return;
  }
  if (!tsaUrl) {
    showToast(t("hash.tsaRequired"), "error");
    return;
  }
  state.hashTargetInput = path;
  state.tsaUrl = tsaUrl;
  setStatus("[data-hash-status]", `${icon("clock")} ${t("hash.timestamping")}`);
  try {
    const result = await apiRequest("/api/hash-timestamp", {
      method: "POST",
      body: JSON.stringify({ path, tsa_url: tsaUrl }),
    });
    state.hashResult = {
      ...(state.hashResult || {}),
      path,
      sha256: result.sha256,
      tsa_url: result.tsa_url,
      timestamped_at: result.timestamped_at,
      timestamp_response_path: result.timestamp_response_path,
    };
    const outBox = document.querySelector("[data-hash-output]");
    if (outBox) outBox.innerHTML = renderHashResult(state.hashResult, t);
    setStatus("[data-hash-status]", `${icon("shield")} ${t("hash.timestamped")}`);
    showToast(t("hash.timestamped"));
  } catch (err) {
    setStatus(
      "[data-hash-status]",
      `${icon("info")} ${t("hash.timestampFailed", { message: escapeHtml(err.message) })}`
    );
    showToast(t("hash.timestampFailed", { message: err.message }), "error");
  }
}

async function listEvidenceReports() {
  if (!state.activeCase) {
    showToast(t("case.required"), "error");
    return;
  }
  try {
    const result = await apiRequest("/api/evidence-list-files", {
      method: "POST",
      body: JSON.stringify({ subdir: "raporlar" }),
    });
    const files = result.files || [];
    const outBox = document.querySelector("[data-report-output]");
    if (outBox) {
      outBox.innerHTML = files.length
        ? files
            .map(
              (f) =>
                `<strong>${escapeHtml(f.name)}</strong> (${formatBytes(f.size)})<br/><small style="color:var(--muted)">${escapeHtml(f.path)}</small>`
            )
            .join("<hr style='border:0;border-top:1px solid var(--line);margin:8px 0;'/>")
        : t("report.noReports");
    }
    showToast(t("report.refreshDone"));
  } catch (error) {
    showToast(t("case.listFailed", { message: error.message }), "error");
  }
}

async function loadEvidenceLogs() {
  if (!state.activeCase) {
    showToast(t("case.required"), "error");
    return;
  }
  try {
    const result = await apiRequest("/api/evidence-list-files", {
      method: "POST",
      body: JSON.stringify({ subdir: "gunlukler" }),
    });
    const files = result.files || [];
    const outBox = document.querySelector("[data-logs-output]");
    if (outBox) {
      outBox.innerHTML = files.length
        ? files
            .map(
              (f) =>
                `<strong>${escapeHtml(f.name)}</strong> (${formatBytes(f.size)})<br/><small style="color:var(--muted)">${escapeHtml(f.path)}</small>`
            )
            .join("<hr style='border:0;border-top:1px solid var(--line);margin:8px 0;'/>")
        : t("logs.empty");
    }
    showToast(t("logs.refreshed"));
  } catch (error) {
    showToast(t("case.listFailed", { message: error.message }), "error");
  }
}

async function downloadUpdatePackage() {
  const status = document.querySelector("[data-update-status]");
  const resultArea = document.querySelector("[data-update-result]");
  const updateBtn = resultArea?.querySelector("[data-action='download-update']");
  let update = state.latestUpdate;
  if (!update || !update.platform_asset?.download_url) {
    try {
      update = await apiRequest("/api/update-check");
      state.latestUpdate = update;
    } catch (_) {}
  }
  const asset = update?.platform_asset || {};
  if (!asset.download_url) {
    const target = update.update_target || state.updateTarget || {};
    const message =
      update.asset_error ||
      t("settings.noAssetForPackage", {
        package: target.asset_package_label || target.package_label || "-",
      });
    if (status) status.innerHTML = `${icon("info")} ${escapeHtml(message)}`;
    setStatus("[data-update-log]", escapeHtml(message));
    showToast(message, "error");
    return;
  }

  // Progress container oluştur veya var olanı al
  let progressContainer = resultArea?.querySelector(".update-progress-container");
  if (!progressContainer && resultArea) {
    progressContainer = document.createElement("div");
    progressContainer.className = "update-progress-container";
    progressContainer.style.cssText =
      "margin-top: 14px; width: 100%; max-width: 380px; margin-left: auto; margin-right: auto;";
    progressContainer.innerHTML = `
      <div class="job-progress-bg" style="height: 6px; background: rgba(255, 255, 255, 0.08); border-radius: 999px; overflow: hidden;">
        <div class="job-progress-fill" data-update-progress-fill style="width: 0%; height: 100%; background: var(--text, #fff); transition: width 0.2s ease;"></div>
      </div>
      <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 6px; font-size: 12px; color: var(--muted, #a1a1aa);">
        <span data-update-progress-msg>İndirme başlatılıyor...</span>
        <span data-update-progress-pct style="font-weight: 600; color: var(--text, #fff);">%0</span>
      </div>
    `;
    resultArea.appendChild(progressContainer);
  }
  if (progressContainer) progressContainer.style.display = "block";

  const progressFill = progressContainer?.querySelector("[data-update-progress-fill]");
  const progressMsg = progressContainer?.querySelector("[data-update-progress-msg]");
  const progressPct = progressContainer?.querySelector("[data-update-progress-pct]");

  if (updateBtn) {
    updateBtn.disabled = true;
    updateBtn.style.opacity = "0.5";
  }

  if (status)
    status.innerHTML = `${icon("download")} <span>${t("settings.downloading") || "İndiriliyor..."}</span>`;

  try {
    const res = await apiRequest("/api/update-download", {
      method: "POST",
      body: JSON.stringify({
        url: asset.download_url,
        name: asset.name,
        expected_sha256: asset.digest || "",
      }),
    });

    const jobId = res?.job_id;
    if (!jobId) throw new Error("İndirme işi oluşturulamadı");

    // İşi poll ederek canlı yüzdeli takip et
    const result = await waitForAcquisitionJob(jobId, {
      onUpdate: (job) => {
        const pct = acquisitionPercent(job);
        const rawMsg = job.message || "";
        const cleanMsg = rawMsg ? rawMsg.replace(/:\s*\d+%/g, "").trim() : `${pct}%`;
        if (progressFill) progressFill.style.width = `${pct}%`;
        if (progressPct) progressPct.textContent = `%${pct}`;
        if (progressMsg)
          progressMsg.textContent = cleanMsg.includes("indirildi")
            ? cleanMsg
            : `${cleanMsg} indirildi`;
        return `%${pct}`;
      },
    });

    if (progressFill) progressFill.style.width = "100%";
    if (progressPct) progressPct.textContent = "%100";
    if (progressMsg) progressMsg.textContent = "İndirme tamamlandı! Kurulum başlatılıyor...";
    if (status)
      status.innerHTML = `${icon("shield")} <span>${t("settings.installing") || "Kurulum başlatılıyor..."}</span>`;

    const install = await apiRequest("/api/update-install", {
      method: "POST",
      body: JSON.stringify({ path: result.path }),
    });

    if (status)
      status.innerHTML = `${icon("shield")} <span>${t("settings.installStarted") || "Kurulum başlatıldı"}</span>`;
    if (progressMsg) progressMsg.textContent = t("settings.installStarted") || "Kurulum başlatıldı";
    setStatus(
      "[data-update-log]",
      `${t("settings.downloaded", { path: escapeHtml(result.path) })}<br />${escapeHtml(install.message || t("settings.installStarted"))}`
    );
    showToast(t("settings.installStarted") || "Güncelleme paketi çalıştırıldı", "success");
  } catch (error) {
    if (progressFill) progressFill.style.width = "0%";
    if (progressPct) progressPct.textContent = "%0";
    if (progressMsg) progressMsg.textContent = "İndirme başarısız";
    const failedKey = String(error.message || "")
      .toLowerCase()
      .includes("installer")
      ? "settings.installFailed"
      : "settings.downloadFailed";
    if (status)
      status.innerHTML = `${icon("info")} <span>${t(failedKey, { message: escapeHtml(error.message) })}</span>`;
    showToast(t(failedKey, { message: error.message }), "error");
  } finally {
    if (updateBtn) {
      updateBtn.disabled = false;
      updateBtn.style.opacity = "1";
    }
  }
}

async function expandTreeNode(nodeElement, relativePath) {
  if (toggleExistingTreeChildren(nodeElement)) {
    return;
  }

  const tempContainer = document.createElement("div");
  tempContainer.className = "tree-children";
  tempContainer.style.paddingLeft = "14px";
  const placeholder = nodeElement.nextElementSibling?.classList.contains("tree-children-container")
    ? nodeElement.nextElementSibling
    : null;
  nodeElement.parentNode.insertBefore(
    tempContainer,
    placeholder ? placeholder.nextSibling : nodeElement.nextSibling
  );

  try {
    nodeElement.querySelector(".toggle-icon").innerHTML = "⌛";
    const result = await apiRequest("/api/image-browse", {
      method: "POST",
      body: JSON.stringify({ path: relativePath }),
    });

    let html = "";
    if (result.files && result.files.length > 0) {
      result.files.sort((a, b) => b.is_dir - a.is_dir || a.name.localeCompare(b.name));
      result.files.forEach((file) => {
        const fileIcon = file.is_dir ? "📁" : "📄";
        const toggle = file.is_dir ? `<span class="toggle-icon">▸</span>` : "";
        const sizeStr = file.is_dir
          ? ""
          : `<span class="node-size">${formatBytes(file.size)}</span>`;
        html += `
          <div class="tree-node" data-path="${escapeHtml(file.relative_path)}" data-is-dir="${file.is_dir}">
            <span style="width:16px;display:inline-block">${toggle}</span>
            <span class="node-icon">${fileIcon}</span>
            <span class="node-name">${escapeHtml(file.name)}</span>
            ${sizeStr}
          </div>
          <div class="tree-children-container"></div>
        `;
      });
    } else {
      html += `<div class="tree-node" style="opacity:0.5;padding-left:20px">Boş Klasör / Empty Directory</div>`;
    }

    nodeElement.querySelector(".toggle-icon").innerHTML = "▾";
    tempContainer.innerHTML = html;
  } catch (error) {
    nodeElement.querySelector(".toggle-icon").innerHTML = "▸";
    tempContainer.remove();
    showToast("Klasör açma başarısız: " + error.message, "error");
  }
}

function findExistingTreeChildren(nodeElement) {
  let sibling = nodeElement.nextElementSibling;
  if (sibling?.classList.contains("tree-children-container")) {
    sibling = sibling.nextElementSibling;
  }
  return sibling?.classList.contains("tree-children") ? sibling : null;
}

function toggleExistingTreeChildren(nodeElement) {
  const childrenContainer = findExistingTreeChildren(nodeElement);
  if (!childrenContainer) return false;
  const toggleIcon = nodeElement.querySelector(".toggle-icon");
  if (childrenContainer.style.display === "none") {
    childrenContainer.style.display = "block";
    if (toggleIcon) toggleIcon.innerHTML = "▾";
  } else {
    childrenContainer.style.display = "none";
    if (toggleIcon) toggleIcon.innerHTML = "▸";
  }
  return true;
}

function showVirtualTreeInfo(nodeElement) {
  const container = document.querySelector("#image-file-preview");
  if (!container) return;
  const title =
    nodeElement.querySelector(".node-name")?.textContent?.trim() || t("analysis.virtualInfo");
  const note = nodeElement.dataset.note || title;
  container.innerHTML = `
    <div class="log-box" style="padding:20px;white-space:pre-wrap">
      <strong>${escapeHtml(title)}</strong>
      <div class="section-divider"></div>
      ${escapeHtml(note)}
    </div>
  `;
}

async function previewImageFile(relativePath) {
  const container = document.querySelector("#image-file-preview");
  if (!container) return;
  container.innerHTML = `<div class="log-box" style="display:flex;align-items:center;justify-content:center;color:var(--muted);height:200px">⌛ Yükleniyor / Loading...</div>`;

  try {
    const result = await apiRequest("/api/image-read-file", {
      method: "POST",
      body: JSON.stringify({ path: relativePath }),
    });

    let contentHtml = "";
    if (result.type === "image") {
      contentHtml = `
        <div class="image-viewer-area" style="height:320px">
          <img src="${result.content}" alt="preview" style="max-height:300px" />
        </div>
      `;
    } else if (result.type === "text") {
      contentHtml = `
        <textarea class="text-viewer-area" style="height:320px" readonly>${escapeHtml(result.content)}</textarea>
      `;
    } else {
      contentHtml = `
        <div class="hex-viewer-area" style="height:320px">${escapeHtml(result.content)}</div>
      `;
    }

    container.innerHTML = `
      <div style="display:flex;flex-direction:column;gap:12px;padding:14px">
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid var(--line);padding-bottom:10px">
          <strong style="color:var(--text);font-size:14px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escapeHtml(relativePath.split("/").pop())}</strong>
          <small class="meta" style="margin-top:0">${formatBytes(result.size)}</small>
        </div>
        <div style="flex:1;overflow:hidden">
          ${contentHtml}
        </div>
      </div>
    `;
  } catch (error) {
    container.innerHTML = renderErrorPanel(t("analysis.previewFailedTitle"), error);
  }
}

async function inspectProcessDetails(pid, name) {
  const rightContent = document.querySelector("#ram-right-content");
  if (!rightContent) return;

  const osProfile = document.querySelector("#ram-os-profile")?.value || "windows";
  rightContent.innerHTML = ramConsoleHtml("Canlı Proses Detay Konsolu", [
    "Volatility3 ile proses detayları yükleniyor...",
    osProfile === "windows" ? "DLL listesi çıkarılıyor." : "Açık dosyalar listeleniyor.",
  ]);

  const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
  const symbolDir = ramSymbolDirValue();
  try {
    const start = await apiRequest("/api/ram-process-details-start", {
      method: "POST",
      body: JSON.stringify({ path: ramPath, pid, os_type: osProfile, symbol_dir: symbolDir }),
    });
    if (!start.job_id) throw new Error(t("workflow.jobIdMissing"));
    const result = await waitForAcquisitionJob(start.job_id, {
      onUpdate(job) {
        updateRamConsole("#ram-right-content", job, "Canlı Proses Detay Konsolu");
      },
    });

    const label =
      osProfile === "windows"
        ? "Yüklenen DLL Modülleri (Loaded DLLs)"
        : "Açık Dosyalar (Open Files / lsof)";

    rightContent.innerHTML = `
      <div style="display:flex;flex-direction:column;gap:14px;padding:12px">
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid var(--line);padding-bottom:8px">
          <strong style="color:var(--text)">${escapeHtml(name)} (${escapeHtml(pid)})</strong>
          <span style="font-size:12px;color:var(--muted)">Döküm: ${result.dumps?.length || 0} segment</span>
        </div>
        
        <p style="margin:0;font-size:12px;font-weight:bold;color:var(--muted)">${escapeHtml(label)}</p>
        <div class="maps-pre-box">${escapeHtml(result.maps || "Detay bulunamadı")}</div>
        
        <div class="section-divider" style="margin:8px 0"></div>
        
        <p style="margin:0;font-size:12px;font-weight:bold;color:var(--muted)">Proses Belleğinde Kelime/Dizgi Ara</p>
        <div class="input-action">
          <input type="text" id="proc-search-query" class="input" placeholder="Örn: whatsapp, telegram, token, password..." />
          <button class="primary-button" data-action="proc-search" data-pid="${escapeHtml(pid)}">Bellekte Ara</button>
        </div>
        
        <div id="proc-search-results" style="margin-top:10px"></div>
      </div>
    `;
    hydrateIcons(rightContent);
  } catch (error) {
    rightContent.innerHTML = renderErrorPanel(t("analysis.processDetailFailedTitle"), error);
  }
}

// Add global listener to inspect proc-search data action
document.addEventListener("click", async (event) => {
  const searchBtn = event.target.closest("[data-action='proc-search']");
  if (searchBtn) {
    const pid = searchBtn.dataset.pid;
    await runProcessMemorySearch(pid);
  }
});

async function runProcessMemorySearch(pid) {
  const query = document.querySelector("#proc-search-query")?.value.trim();
  const resultsDiv = document.querySelector("#proc-search-results");
  if (!query || !resultsDiv) {
    showToast("Arama sorgusu boş olamaz", "error");
    return;
  }

  const osProfile = document.querySelector("#ram-os-profile")?.value || "windows";
  resultsDiv.innerHTML = `<div class="log-box" style="text-align:center;padding:10px">⌛ Uçucu bellek taranıyor...</div>`;

  const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
  try {
    const result = await apiRequest("/api/ram-process-search", {
      method: "POST",
      body: JSON.stringify({ path: ramPath, pid, query, os_type: osProfile }),
    });

    const count = result.length || 0;
    if (count === 0) {
      resultsDiv.innerHTML = `<div class="log-box" style="color:var(--muted);text-align:center;padding:10px">Hiçbir eşleşme bulunamadı.</div>`;
    } else {
      resultsDiv.innerHTML = `
        <div class="strings-results-list" style="max-height:220px">
          ${result
            .map(
              (item) => `
            <div class="string-match-item">
              <div class="match-meta">
                <span>Segment: <strong>${escapeHtml(item.category)}</strong></span>
                <span>Ofset: <strong>0x${item.offset.toString(16).toUpperCase()}</strong></span>
              </div>
              <div class="match-value" style="color:var(--text)">${escapeHtml(item.value)}</div>
              <div class="match-context">${escapeHtml(item.context)}</div>
            </div>
          `
            )
            .join("")}
        </div>
      `;
    }
  } catch (error) {
    resultsDiv.innerHTML = renderErrorPanel(t("analysis.searchFailedTitle"), error);
  }
}

async function previewCarvedFile(filePath) {
  const rightContent = document.querySelector("#ram-right-content");
  if (!rightContent) return;

  rightContent.innerHTML = `<div class="log-box" style="text-align:center;padding:20px">⌛ Kurtarılan dosya yükleniyor... / Loading carved file...</div>`;

  try {
    const result = await apiRequest("/api/ram-read-carved", {
      method: "POST",
      body: JSON.stringify({ path: filePath }),
    });

    let contentHtml = "";
    if (result.type === "image") {
      contentHtml = `
        <div class="image-viewer-area" style="height:320px">
          <img src="${result.content}" alt="carved preview" style="max-height:300px" />
        </div>
      `;
    } else if (result.type === "text") {
      contentHtml = `
        <textarea class="text-viewer-area" style="height:320px" readonly>${escapeHtml(result.content)}</textarea>
      `;
    } else {
      contentHtml = `
        <div class="hex-viewer-area" style="height:320px">${escapeHtml(result.content)}</div>
      `;
    }

    rightContent.innerHTML = `
      <div style="display:flex;flex-direction:column;gap:12px;padding:12px">
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid var(--line);padding-bottom:10px">
          <strong style="color:var(--text);font-size:14px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escapeHtml(filePath.split("/").pop())}</strong>
          <small class="meta" style="margin-top:0">${formatBytes(result.size)}</small>
        </div>
        <div style="flex:1;overflow:hidden">
          ${contentHtml}
        </div>
      </div>
    `;
  } catch (error) {
    rightContent.innerHTML = renderErrorPanel(t("analysis.previewFailedTitle"), error);
  }
}

async function bootApp() {
  setLanguage(state.language);
  setTheme(state.theme);
  syncSidebarState();
  installUiErrorHandlers();
  hydrateIcons();
  try {
    const cachedDevs = localStorage.getItem("amele_contributors");
    if (cachedDevs) {
      const parsed = JSON.parse(cachedDevs);
      if (Array.isArray(parsed) && parsed.length > 0) {
        state.contributors = parsed;
      }
    }
  } catch (_) {}
  await loadProfiles();
  await loadUpdateTarget();
  if (state.activeProfile) {
    await loadPersistedSettings();
    await loadEvidenceCases();
  }
  render();
  if (state.profileGateVisible) renderProfileGate();

  // Klavye kısayollarını ve Shift ipucu rozet yöneticisini başlat
  initKeyboardShortcuts({
    state,
    t,
    toggleNavMenu,
    openNavMenu,
    closeNavMenu,
    setRoute,
    toggleCaseSidebar,
    toggleReportSidebar,
    setLanguage: async (lang) => {
      setLanguage(lang);
      try {
        await saveSettingsFromControls();
      } catch (_) {}
      render();
    },
    logout: () => showProfileGate(),
  });

  // Yapay zeka ajanlarını ve modellerini arka planda yükle
  loadAgents(state, render).catch(() => {});

  // Background network tasks and timers (only in real browser / webview, not during Node unit tests)
  const isNodeTest = typeof process !== "undefined" && Boolean(process.versions?.node);
  if (!isNodeTest) {
    // Online profil kontrolü & otomatik senkronizasyon:
    // İnternet bağlıysa token geçerliliğini test edip senkronize et,
    // internet yoksa hesabı offline modda tut.
    if (state.activeProfile?.online) {
      if (typeof navigator !== "undefined" && !navigator.onLine) {
        state.activeProfile.online.status = "offline";
        syncProfileButton();
      } else {
        syncOnlineProfile(null, true).catch(() => {});
      }
    }

    // Load latest news & announcements from website in background
    loadNewsAnnouncements().catch(() => {});
    startNewsCarouselTimer();

    // Geliştirici listesini açılışta hemen çek
    loadDevelopers().catch(() => {});

    // İlk açılışta yeni sürüm kontrolü yap
    setTimeout(() => checkForUpdates().catch(() => {}), 2000);

    // Her 5 dakikada bir düzenli olarak arkada version.json kontrolü yap
    setInterval(() => checkForUpdates().catch(() => {}), 5 * 60 * 1000);
  }

  // Developer mode — 5 kez logoya tıklayınca aktifleşir
  initDeveloperMode({ apiRequest, backendReady });
  if (backendAvailable) initJobWidget({ onNavigate: (route) => setRoute(route) });
  devLog(
    "INFO",
    "ui:startup",
    `Amele ${APP_VERSION} başlatıldı — platform: ${state.platform}, dil: ${state.language}, tema: ${state.theme}, backend: ${backendAvailable}`,
    apiRequest,
    backendReady
  );
}

const DEFAULT_CONTRIBUTORS = [
  {
    id: "melih-emik",
    name: "Melih Emik",
    role: "BDFL & Maintainer",
    photo: "melih-emik.jpg",
    links: [
      ["GitHub", "https://github.com/melihemik"],
      ["LinkedIn", "https://linkedin.com/in/melihemik"],
      ["Website", "https://melihemik.com.tr"],
      ["E-posta", "mailto:melihemik@noirlang.tr"],
      ["GPG", "https://keys.openpgp.org/search?q=melihemik@noirlang.tr"],
    ],
  },
  {
    id: "m-ali-guner",
    name: "MuhammetAli Güner",
    role: "Lead Maintainer",
    photo: "muhammet-ali-guner.jpg",
    links: [
      ["GitHub", "https://github.com/kafkaskrtl"],
      ["LinkedIn", "https://www.linkedin.com/in/muhammetali-g%C3%BCner/"],
    ],
  },
  {
    id: "toretto",
    name: "Abdulhalim Altuntaş",
    role: "Maintainer",
    photo: "abdulhalim.jpg",
    links: [
      ["GitHub", "https://github.com/abdulhalimaltuntas"],
      ["LinkedIn", "https://www.linkedin.com/in/abdulhalimaltuntas/"],
    ],
  },
];

let developersLoading = false;
async function loadDevelopers(force = false) {
  if (developersLoading) return;

  developersLoading = true;
  try {
    let data = null;

    // Doğrudan web'den fetch et — R2/domain üzerinde CORS izinleri mevcuttur
    try {
      const res = await fetch(
        `https://download.amele.noirlang.tr/developers.json?_t=${Date.now()}`,
        { cache: "no-store" }
      );
      if (res.ok) {
        data = await res.json();
      }
    } catch (_) {}

    // JSON formatlarını normalize et:
    // { developers: [...] } veya { developer: [...] } veya [...] veya tek nesne { ... }
    const rawList = data
      ? Array.isArray(data)
        ? data
        : Array.isArray(data.developers)
          ? data.developers
          : Array.isArray(data.developer)
            ? data.developer
            : data.name
              ? [data]
              : []
      : [];

    if (rawList.length > 0) {
      state.contributors = rawList.map((d) => {
        const name = d.name || d.full_name || "Melih Emik";
        const role = d.role || d.title || d.defaultRole || "Developer";
        const photo = d.avatar_url || d.photo || d.avatar || d.image || "melih-emik.jpg";

        let links = [];
        if (Array.isArray(d.links)) {
          links = d.links;
        } else if (d.links && typeof d.links === "object") {
          links = Object.entries(d.links);
        } else {
          if (d.github) links.push(["GitHub", d.github]);
          if (d.linkedin) links.push(["LinkedIn", d.linkedin]);
          if (d.website || d.site) links.push(["Website", d.website || d.site]);
          if (d.email || d.mail) links.push(["E-posta", d.email || d.mail]);
          if (d.gpg_url || d.pgp_url) links.push(["GPG/PGP", d.gpg_url || d.pgp_url]);
          else if (d.gpg_key || d.pgp_key)
            links.push([
              "GPG Key",
              `https://keys.openpgp.org/search?q=${encodeURIComponent(d.email || d.gpg_key || d.pgp_key)}`,
            ]);
          if (d.twitter || d.x) links.push(["X", d.twitter || d.x]);
        }

        return {
          id: d.id || name.toLowerCase().replace(/\s+/g, "-"),
          name,
          role,
          photo,
          links,
        };
      });

      try {
        localStorage.setItem("amele_contributors", JSON.stringify(state.contributors));
        localStorage.setItem("amele_contributors_last_fetch", String(Date.now()));
      } catch (_) {}
    } else {
      // Çekilemediyse önce localStorage'dan dene, yoksa varsayılan listeyi yükle
      let cached = null;
      try {
        const rawCached = localStorage.getItem("amele_contributors");
        if (rawCached) cached = JSON.parse(rawCached);
      } catch (_) {}

      if (Array.isArray(cached) && cached.length > 0) {
        state.contributors = cached;
      } else {
        state.contributors = DEFAULT_CONTRIBUTORS;
      }
    }

    if (state.route === "about") {
      render();
    }
  } catch (_) {
    if (!state.contributors || state.contributors.length === 0) {
      state.contributors = DEFAULT_CONTRIBUTORS;
      if (state.route === "about") render();
    }
  } finally {
    developersLoading = false;
  }
}
const loadGitHubContributors = loadDevelopers;

let updateCheckInProgress = false;
async function checkForUpdates() {
  if (updateCheckInProgress) return;
  updateCheckInProgress = true;
  const SKIP_KEY = "amele_update_skip";
  const skippedVersion = localStorage.getItem(SKIP_KEY);
  const lang = state.language || "en";
  const isTr = lang === "tr";

  try {
    let result = null;
    if (backendReady()) {
      try {
        result = await apiRequest("/api/update-check");
      } catch (_) {}
    }

    if (!result || (!result.tag_name && !result.name && !result.version)) {
      const controller = new AbortController();
      const tid = setTimeout(() => controller.abort(), 15000);
      const res = await fetch("https://download.amele.noirlang.tr/version.json", {
        signal: controller.signal,
      });
      clearTimeout(tid);
      if (res.ok) {
        const vdata = await res.json();
        const vStr = vdata.version || "0.1.1";
        const tag = vStr.startsWith("v") ? vStr : `v${vStr}`;
        result = {
          tag_name: tag,
          name: `Amele ${tag}`,
          version: vStr,
          html_url: "https://amele.noirlang.tr",
          mandatory: vdata.mandatory,
        };
      }
    }

    if (!result) return;
    state.latestUpdate = result;
    state.updateTarget = result.update_target || state.updateTarget;

    const latestTag = (result.tag_name || result.name || "").trim();
    const releaseUrl = result.html_url || "https://amele.noirlang.tr";
    if (!latestTag) return;

    const parseVer = (v) => v.replace(/^v/i, "").split("-")[0].split(".").map(Number);
    const [cMaj, cMin, cPatch] = parseVer(APP_VERSION);
    const [lMaj, lMin, lPatch] = parseVer(latestTag);
    const hasUpdate =
      lMaj > cMaj ||
      (lMaj === cMaj && lMin > cMin) ||
      (lMaj === cMaj && lMin === cMin && lPatch > cPatch);

    if (!hasUpdate) return;
    state.updateAvailable = { latestTag, releaseUrl };
    if (skippedVersion === latestTag) return;

    showUpdateToast({ latestTag, releaseUrl, isTr });
  } catch (_) {
    // Arka plan kontrolü sessizce hata yutar
  } finally {
    updateCheckInProgress = false;
  }
}

function showUpdateToast({ latestTag, releaseUrl, isTr }) {
  // Var olan toast varsa kaldır
  const old = document.getElementById("amele-update-toast");
  if (old) old.remove();

  const toast = document.createElement("div");
  toast.id = "amele-update-toast";
  toast.className = "update-toast";

  const title = isTr ? "Yeni Güncelleme Hazır" : "New Update Available";
  const downloadLabel = isTr ? "İndir" : "Download";
  const dismissLabel = isTr ? "Şimdi değil" : "Not now";

  toast.innerHTML = `
    <div class="update-toast-header">
      <div class="update-toast-header-left">
        <span class="update-toast-dot"></span>
        <span class="update-toast-category">${isTr ? "Güncelleme" : "Update"}</span>
      </div>
      <button class="update-toast-close" title="${isTr ? "Kapat" : "Close"}">✕</button>
    </div>
    <div class="update-toast-body">
      <div class="update-toast-row">
        <span class="update-toast-icon">${icon("rocket")}</span>
        <span class="update-toast-title">${title}</span>
      </div>
      <div class="update-toast-version">
        ${isTr ? "Mevcut" : "Current"}: <b>${APP_VERSION}</b> → ${isTr ? "Yeni" : "New"}: <b>${latestTag}</b>
      </div>
    </div>
    <div class="update-toast-actions">
      <button type="button" class="update-toast-btn primary update-goto-settings-btn">
        ${icon("download")}
        <span>${downloadLabel}</span>
      </button>
      <button type="button" class="update-toast-btn secondary dismiss-btn">${dismissLabel}</button>
    </div>
  `;

  document.body.appendChild(toast);

  // Kapatma aksiyonları
  const hide = (skip = false) => {
    toast.classList.remove("visible");
    if (skip) {
      try {
        localStorage.setItem("amele_update_skip", latestTag);
      } catch {}
    }
    setTimeout(() => toast.remove(), 350);
  };

  toast.querySelector(".update-toast-close")?.addEventListener("click", () => hide(false));
  toast.querySelector(".dismiss-btn")?.addEventListener("click", () => hide(true));
  const gotoSettingsBtn = toast.querySelector(".update-goto-settings-btn");
  if (gotoSettingsBtn) {
    gotoSettingsBtn.addEventListener("click", () => {
      state.updateAvailable = { latestTag, releaseUrl };
      hide(false);
      setRoute("settings");
      setTimeout(() => {
        const targetEl =
          document.querySelector("[data-update-result]") ||
          document.querySelector(".settings-update-row-centered");
        if (targetEl) {
          targetEl.scrollIntoView({ behavior: "smooth", block: "center" });
        }
        const dlBtn = document.querySelector("[data-action='download-update']");
        if (dlBtn) dlBtn.focus();
      }, 120);
    });
  }

  // Göster
  requestAnimationFrame(() => {
    requestAnimationFrame(() => toast.classList.add("visible"));
  });

  // 60 saniye sonra otomatik kapat (skip etmeden)
  setTimeout(() => {
    if (document.body.contains(toast)) hide(false);
  }, 60000);
}

async function loadNewsAnnouncements(forceRefresh = false) {
  try {
    const items = await fetchNewsAnnouncements("https://amele.noirlang.tr", forceRefresh);
    if (Array.isArray(items)) {
      const oldLen = state.news?.length || 0;
      const oldFirstId = state.news?.[0]?.id || state.news?.[0]?._id;
      const newFirstId = items[0]?.id || items[0]?._id;

      state.news = items;
      if (state.activeNewsIndex >= items.length) {
        state.activeNewsIndex = 0;
      }

      // Değişiklik varsa veya ilk yükleme ise ana sayfayı güncelle
      if (state.route === "home" && (oldLen !== items.length || oldFirstId !== newFirstId)) {
        render();
      }
    }
  } catch (err) {
    console.warn("News announcements fetch failed:", err);
  }
}

let newsPollingTimer = null;
function startNewsCarouselTimer() {
  // Periyodik olarak 60 saniyede bir web sitesinden yeni duyuru/haber kontrolü yap
  if (newsPollingTimer) clearInterval(newsPollingTimer);
  newsPollingTimer = setInterval(() => {
    loadNewsAnnouncements(true).catch(() => {});
  }, 60000);

  // Pencereye odaklanıldığında veya internet geldiğinde anında yenile
  window.addEventListener("focus", () => {
    loadNewsAnnouncements(true).catch(() => {});
  });
  window.addEventListener("online", () => {
    loadNewsAnnouncements(true).catch(() => {});
    if (state.activeProfile?.online) {
      syncOnlineProfile(null, true).catch(() => {});
    }
  });
  window.addEventListener("offline", () => {
    if (state.activeProfile?.online) {
      state.activeProfile.online.status = "offline";
      syncProfileButton();
      render();
    }
  });
}

bootApp().catch((error) => {
  console.error("Amele UI boot failed", error);
  showToast(`Arayüz başlatılamadı: ${error.message || error}`, "error");
});
