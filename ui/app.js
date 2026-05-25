import { androidModePage, androidPage, handleAndroidAction, syncAndroidDeviceSelection } from "./android.js";
import { icon, hydrateIcons, fontIcons } from "./icons.js";
import { translate } from "./i18n.js";
import { homePage, metric } from "./pages/home.js";
import { windowsPage } from "./pages/windows.js";
import { linuxPage } from "./pages/linux.js";
import { agentPage } from "./pages/agent.js";
import { analysisPage } from "./pages/analysis.js";
import { otherPage, detailPanel, settingsPage, aboutPage, hashPanel } from "./pages/other.js";
import { workflowPage, pickerField, field, pageTitle, casePanel } from "./pages/workflow.js";

const APP_VERSION = "v0.0.8";
const assetPath = "./assets";
const backendAvailable = location.protocol === "http:" || location.protocol === "https:";
const isNativeWebView = new URLSearchParams(window.location.search).get("native") === "1";
if (isNativeWebView) document.documentElement.classList.add("native-webview");

const app = document.querySelector("#app");
const view = document.querySelector("#view");
const preferredLanguage = localStorage.getItem("worm-language") || "tr";

function initialLogMessages(language) {
  return [
    translate(language, backendAvailable ? "log.appReady" : "log.previewMode"),
    translate(language, "log.agentProtocol"),
    translate(language, "log.workflowsReady")
  ];
}

const state = {
  route: new URLSearchParams(window.location.search).get("route") || "home",
  theme: localStorage.getItem("worm-theme") || "dark",
  language: preferredLanguage,
  platform: detectPlatform(),
  files: {},
  activeTab: "hash",
  approvedSecurityKey: "",
  remoteConnections: {},
  activeAcquisition: null,
  activeCase: null,
  cases: [],
  caseBaseDir: "",
  imageMount: null,
  latestUpdate: null,
  android: {
    adbStatus: null,
    devices: [],
    selectedDevice: ""
  },
  jobs: {},
  cachedDefaultCaseName: "",
  lastLog: initialLogMessages(preferredLanguage)
};

function detectPlatform() {
  const override = new URLSearchParams(window.location.search).get("platform");
  if (["windows", "linux", "android", "mac"].includes(override || "")) return override;
  const text = `${navigator.userAgent} ${navigator.platform}`.toLowerCase();
  if (text.includes("android")) return "android";
  if (text.includes("win")) return "windows";
  if (text.includes("linux")) return "linux";
  if (text.includes("mac")) return "mac";
  return "unknown";
}

function t(key, vars = {}) {
  return translate(state.language, key, vars);
}

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
    hashPanel
  });
}

function boundPickerField(label, id, value, type = "file") {
  return pickerField(label, id, value, type, icon, t);
}

function L(tr, en) {
  return { tr, en };
}

function localText(value) {
  if (value && typeof value === "object" && "tr" in value) {
    return value[state.language] || value.tr;
  }
  return value;
}

const toolCards = {
  windows: [
    {
      id: "windows-remote-disk",
      title: L("Uzak Disk İmajı", "Remote Disk Image"),
      desc: L("Windows agent üzerinden PhysicalDrive imajı alın.", "Acquire a PhysicalDrive image through the Windows agent."),
      icon: "disk",
      accent: "var(--green)",
      badge: "Agent + raw stream"
    },
    {
      id: "windows-local-disk",
      title: L("Yerel Disk İmajı", "Local Disk Image"),
      desc: L("Bu makinedeki Windows disklerinden ham imaj üretin.", "Create a raw image from Windows disks on this machine."),
      icon: "windows",
      accent: "var(--blue)",
      badge: "PhysicalDrive"
    },
    {
      id: "windows-remote-ram",
      title: L("Uzak RAM", "Remote RAM"),
      desc: L("WinPMEM ile uzak Windows RAM edinimi başlatın ve indirin.", "Start and download remote Windows RAM acquisition with WinPMEM."),
      icon: "ram",
      accent: "var(--purple)",
      badge: "WinPMEM remote"
    },
    {
      id: "windows-local-ram",
      title: L("Yerel RAM", "Local RAM"),
      desc: L("Yerel WinPMEM kontrolü, indirme ve RAM imajı alma.", "Check local WinPMEM, download if needed, and acquire RAM."),
      icon: "chip",
      accent: "var(--amber)",
      badge: L("Yönetici gerekli", "Admin required")
    }
  ],
  linux: [
    {
      id: "linux-remote-disk",
      title: L("Uzak Disk İmajı", "Remote Disk Image"),
      desc: L("Linux agent üzerinden /dev disklerinden ham imaj alın.", "Acquire raw images from /dev disks through the Linux agent."),
      icon: "disk",
      accent: "var(--green)",
      badge: "Agent + /dev"
    },
    {
      id: "linux-local-disk",
      title: L("Yerel Disk İmajı", "Local Disk Image"),
      desc: L("Yerel Linux diskleri için root yetkili imaj alma akışı.", "Root-level acquisition workflow for local Linux disks."),
      icon: "linux",
      accent: "var(--blue)",
      badge: "BLKGETSIZE64"
    },
    {
      id: "linux-remote-ram",
      title: L("Uzak RAM", "Remote RAM"),
      desc: L("AVML ile uzak Linux RAM edinimi ve dosya indirme.", "Acquire remote Linux RAM with AVML and download the dump file."),
      icon: "ram",
      accent: "var(--purple)",
      badge: "AVML remote"
    },
    {
      id: "linux-local-ram",
      title: L("Yerel RAM", "Local RAM"),
      desc: L("AVML varlık/yetki kontrolü ve yerel RAM dump üretimi.", "Check AVML availability/privileges and create a local RAM dump."),
      icon: "chip",
      accent: "var(--amber)",
      badge: L("Root gerekli", "Root required")
    }
  ]
};

const workflows = {
  "windows-remote-disk": {
    platform: "Windows",
    icon: "windows",
    title: L("Uzak Windows Sunucu Bağlantısı", "Remote Windows Server Connection"),
    desc: L("Uzak Windows sistemlerine güvenli bağlantı kurun ve disk imajı alın.", "Connect securely to remote Windows systems and acquire disk images."),
    mode: "remote-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: L("Disk seçilmedi", "No disk selected")
  },
  "linux-remote-disk": {
    platform: "Linux",
    icon: "linux",
    title: L("Uzak Linux Disk Bağlantısı", "Remote Linux Disk Connection"),
    desc: L("Linux agent ile uzak /dev disklerini listeleyin ve raw imaj alın.", "List remote /dev disks through the Linux agent and acquire raw images."),
    mode: "remote-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: L("Disk seçilmedi", "No disk selected")
  },
  "windows-local-disk": {
    platform: "Windows",
    icon: "windows",
    title: L("Windows Yerel Disk İmajı", "Windows Local Disk Image"),
    desc: L("Yerel PhysicalDrive kaynaklarından ham imaj alma akışı.", "Raw image acquisition workflow for local PhysicalDrive sources."),
    mode: "local-disk",
    output: "C:\\Worm\\Ciktilar",
    diskLabel: L("Disk seçilmedi", "No disk selected")
  },
  "linux-local-disk": {
    platform: "Linux",
    icon: "linux",
    title: L("Linux Yerel Disk İmajı", "Linux Local Disk Image"),
    desc: L("Yerel Linux blok cihazlarından imaj alma akışı.", "Image acquisition workflow for local Linux block devices."),
    mode: "local-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: L("Disk seçilmedi", "No disk selected")
  },
  "windows-remote-ram": {
    platform: "Windows",
    icon: "ram",
    title: L("Windows Uzak RAM Edinimi", "Windows Remote RAM Acquisition"),
    desc: L("WinPMEM durumunu kontrol edin, uzak RAM edinimini başlatın ve dump dosyasını indirin.", "Check WinPMEM, start remote RAM acquisition, and download the dump file."),
    mode: "remote-ram",
    output: "memory_dump.raw",
    diskLabel: "WinPMEM"
  },
  "linux-remote-ram": {
    platform: "Linux",
    icon: "ram",
    title: L("Linux Uzak RAM Edinimi", "Linux Remote RAM Acquisition"),
    desc: L("AVML durumunu kontrol edin, uzak RAM edinimini başlatın ve dump dosyasını indirin.", "Check AVML, start remote RAM acquisition, and download the dump file."),
    mode: "remote-ram",
    output: "memory_dump_linux.raw",
    diskLabel: "AVML"
  },
  "windows-local-ram": {
    platform: "Windows",
    icon: "chip",
    title: L("Windows Yerel RAM Edinimi", "Windows Local RAM Acquisition"),
    desc: L("Yerel WinPMEM kontrolü, gerekirse indirme ve RAM imajı alma.", "Check local WinPMEM, download if needed, and acquire a RAM image."),
    mode: "local-ram",
    output: "memory_dump_local.raw",
    diskLabel: L("WinPMEM local", "Local WinPMEM")
  },
  "linux-local-ram": {
    platform: "Linux",
    icon: "chip",
    title: L("Linux Yerel RAM Edinimi", "Linux Local RAM Acquisition"),
    desc: L("Yerel AVML kontrolü ve root yetkili RAM imajı alma.", "Check local AVML and acquire RAM with root privileges."),
    mode: "local-ram",
    output: "linux_memory_dump.raw",
    diskLabel: L("AVML local", "Local AVML")
  }
};

function setRoute(route) {
  if (route.startsWith("workflow:")) {
    const workflow = workflows[route.split(":")[1]];
    if (workflow && isLocalWorkflowBlocked(workflow)) {
      showToast(t("platformBlocked", { platform: workflow.platform }), "error");
      return;
    }
  }
  state.route = route;
  render();
}

function isLocalWorkflowBlocked(workflow) {
  if (!workflow.mode.startsWith("local")) return false;
  return workflow.platform.toLowerCase() !== state.platform;
}

function setTheme(theme) {
  state.theme = theme;
  localStorage.setItem("worm-theme", theme);
  app.classList.toggle("theme-light", theme === "light");
  app.classList.toggle("theme-dark", theme !== "light");
}

function setLanguage(language) {
  state.language = language;
  localStorage.setItem("worm-language", language);
  document.documentElement.lang = language;
  document.querySelectorAll("[data-i18n]").forEach((node) => {
    node.textContent = t(node.dataset.i18n);
  });
}

function render() {
  const activeGroup = routeGroup(state.route);
  document.querySelectorAll("[data-route]").forEach((button) => {
    button.classList.toggle("active", button.dataset.route === activeGroup);
  });

  const boundCasePanel = (subdir, hint) => casePanel(subdir, hint, {
    t,
    icon,
    state,
    caseSelectOptions,
    caseOutputLabel,
    escapeHtml
  });

  if (state.route.startsWith("workflow:")) {
    view.innerHTML = workflowPage({
      id: state.route.split(":")[1],
      workflows,
      state,
      t,
      icon,
      localText,
      canonicalRamFileName,
      caseSelectOptions,
      caseOutputLabel,
      escapeHtml
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
      field
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
      detailPanel: boundDetailPanel,
      toolHub: (platform) => toolHub(platform),
      platformLabel,
      APP_VERSION
    };
    view.innerHTML = routes[state.route]?.(pageCtx) || homePage(pageCtx);
  }

  hydrateIcons(view);
  if (state.route === "other" && ["evidence", "reports"].includes(state.activeTab)) {
    loadEvidenceCases();
  }
  if (state.route.startsWith("workflow:")) {
    const workflow = workflows[state.route.split(":")[1]];
    if (workflow && workflow.mode.includes("disk")) loadEvidenceCases();
  }
  if (state.route === "android:logical" || state.route === "android:filesystem" || state.route === "android:ram") loadEvidenceCases();
  view.focus({ preventScroll: true });
}

function routeGroup(route) {
  if (route.startsWith("android:")) return "android";
  if (!route.startsWith("workflow:")) return route;
  const workflowId = route.split(":")[1] || "";
  if (workflowId.startsWith("windows")) return "windows";
  if (workflowId.startsWith("linux")) return "linux";
  return route;
}

function toolHub(platform) {
  const cards = toolCards[platform]
    .map(
      (card) => {
        const workflow = workflows[card.id];
        const blocked = workflow && isLocalWorkflowBlocked(workflow);
        return `
        <button class="forensic-card ${blocked ? "is-disabled" : ""}" data-route="workflow:${card.id}" style="--accent:${card.accent}" ${blocked ? `aria-disabled="true" data-disabled-reason="${workflow.platform}"` : ""}>
          <span class="card-icon">${icon(card.icon)}</span>
          <h3>${localText(card.title)}</h3>
          <p>${localText(card.desc)}</p>
          <span class="meta">${blocked ? t("localUnsupported") : localText(card.badge)}</span>
        </button>
      `;
      }
    )
    .join("");

  const isWindows = platform === "windows";
  return `
    <section class="page">
      <div class="platform-note">
        ${icon("monitor")} ${t("hub.detected", { platform: `<strong>${platformLabel(state.platform)}</strong>` })}
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
  if (platform === "windows") return "Windows";
  if (platform === "linux") return "Linux";
  if (platform === "android") return "Android";
  if (platform === "mac") return "macOS";
  return t("unknown");
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
  const key = label === "LinkedIn" ? "linkedin" : label === "Website" ? "website" : "github";
  return `<a class="social-button" href="${url}" target="_blank" rel="noopener noreferrer" aria-label="${label}">${icon(key)}</a>`;
}

const routes = {
  home: homePage,
  windows: () => toolHub("windows"),
  linux: () => toolHub("linux"),
  android: () => androidPage({ t, icon, pageTitle, state, escapeHtml, backendReady }),
  agent: agentPage,
  analysis: analysisPage,
  other: otherPage,
  settings: settingsPage,
  about: aboutPage
};

async function apiRequest(path, options = {}) {
  const headers = new Headers(options.headers || {});
  if (options.body && !headers.has("content-type")) {
    headers.set("content-type", "application/json");
  }
  let response;
  try {
    response = await fetch(path, { ...options, headers });
  } catch (error) {
    throw new Error(formatBackendConnectionError(path, error));
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
}

function formatBackendConnectionError(path, error) {
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

function isExternalUrl(url) {
  try {
    const parsed = new URL(url, window.location.href);
    return ["http:", "https:", "mailto:"].includes(parsed.protocol);
  } catch {
    return false;
  }
}

async function openExternalUrl(url) {
  try {
    await apiRequest("/api/open-url", {
      method: "POST",
      body: JSON.stringify({ url })
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
    
    // Keep frontend selected case active if it still exists
    const activeCaseName = state.activeCase?.case_name;
    if (activeCaseName) {
      const stillExists = state.cases.find(c => c.case_name === activeCaseName);
      if (stillExists) {
        state.activeCase = stillExists;
      } else if (result.current_case) {
        state.activeCase = result.current_case;
      }
    } else if (result.current_case) {
      state.activeCase = result.current_case;
    } else if (state.cases.length) {
      state.activeCase = state.cases[0];
    }

    updateCaseControls();
    if (!silent) showToast(t("case.loaded", { count: String(state.cases.length) }));
  } catch (error) {
    if (!silent) showToast(t("case.listFailed", { message: error.message }), "error");
  }
}

function updateCaseControls() {
  document.querySelectorAll("[data-case-base]").forEach((node) => {
    node.textContent = state.caseBaseDir || "~/Worm/Vakalar";
  });

  const selected = state.activeCase?.case_name || "";
  document.querySelectorAll("[data-case-select]").forEach((select) => {
    const allowNew = select.dataset.allowNewCase === "1";
    select.innerHTML = caseSelectOptions(selected, { allowNew });
    select.value = selected;
    toggleCaseCreateInput(select);
  });
}

function caseSelectOptions(selected = "", { allowNew = false } = {}) {
  const effectiveSelected = selected || (allowNew && !state.cases.length ? "__new__" : "");
  if (!state.cases.length && !allowNew) {
    return `<option value="">${t("case.noCases")}</option>`;
  }
  const options = state.cases
    .map((item) => {
      const name = escapeHtml(item.case_name || "");
      const isSelected = item.case_name === effectiveSelected ? " selected" : "";
      return `<option value="${name}"${isSelected}>${name}</option>`;
    })
    .join("");
  const newSelected = effectiveSelected === "__new__" || (allowNew && !state.cases.length) ? " selected" : "";
  const newOption = allowNew ? `<option value="__new__"${newSelected}>${t("workflow.newCase")}</option>` : "";
  return `${options}${newOption}`;
}

function toggleCaseCreateInput(select) {
  document.querySelectorAll("[data-case-output]").forEach((output) => {
    output.textContent = caseOutputLabel(select.value, output.dataset.caseOutputSubdir || "ciktilar");
  });
}

function imageCaseOutputLabel(caseName) {
  return caseOutputLabel(caseName, "ciktilar");
}

function caseOutputLabel(caseName, subdir = "ciktilar") {
  const selected = state.cases.find((item) => item.case_name === caseName)
    || (state.activeCase?.case_name === caseName ? state.activeCase : null);
  const key = subdir === "ram" ? "ram_dir" : "output_dir";
  if (caseName && caseName !== "__new__" && selected?.[key]) return selected[key];
  const folder = subdir === "ram" ? "ram" : "ciktilar";
  if (state.caseBaseDir) return `${state.caseBaseDir}/${caseName || "vaka"}/${folder}`;
  return `~/Worm/Vakalar/${caseName || "vaka"}/${folder}`;
}

function reportCaseName() {
  const selected = document.querySelector("#report-case")?.value.trim() || "";
  return selected || defaultCaseName();
}

async function ensureImageCase() {
  const select = document.querySelector("#workflow-case");
  const selected = select?.value || "";
  if (selected && selected !== "__new__") {
    const existing = state.cases.find((item) => item.case_name === selected);
    if (existing && existing.case_path) return existing;
    
    // Create new case folder on backend disk if not already existing
    const created = await apiRequest("/api/evidence-create", {
      method: "POST",
      body: JSON.stringify({ case_name: selected })
    });
    state.activeCase = created;
    state.cachedDefaultCaseName = "";
    await loadEvidenceCases();
    return created;
  }

  const caseName = defaultCaseName();
  const created = await apiRequest("/api/evidence-create", {
    method: "POST",
    body: JSON.stringify({ case_name: caseName })
  });
  state.activeCase = created;
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

function timestampForFileName(date = new Date()) {
  const pad = (value) => String(value).padStart(2, "0");
  return `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}_${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`;
}

function sanitizeFileStem(value) {
  return String(value || "")
    .trim()
    .replace(/[<>:"/\\|?*\x00-\x1F\s]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function canonicalRamFileName(remoteIp = "", date = new Date()) {
  const ip = sanitizeFileStem(remoteIp);
  return `${ip ? `${ip}_` : ""}ram_${timestampForFileName(date)}.raw`;
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
    token: tokenText ? state.approvedSecurityKey : null
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
    keepalive: Number(document.querySelector("[data-field='vpn-keepalive']")?.value.trim() || 25)
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
    features: details.features || []
  };
}

function forgetConnection(workflowId = currentWorkflowId()) {
  if (workflowId) delete state.remoteConnections[workflowId];
}

function requireActiveConnection(workflow, payload) {
  if (!workflow?.mode.startsWith("remote")) return true;
  const connection = state.remoteConnections[currentWorkflowId()];
  const matches = connection
    && connection.ip === payload.ip
    && Number(connection.port) === Number(payload.port)
    && (connection.token || "") === (payload.token || "");
  if (!matches) {
    showToast(t("connection.connectFirst"), "error");
    updateSide("connection", t("connection.none"));
    writeWorkflowLog(t("connection.required"));
    return false;
  }
  return true;
}

function formatBytes(bytes) {
  const value = Number(bytes || 0);
  if (!Number.isFinite(value) || value <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let size = value;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return `${size.toFixed(size >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

document.addEventListener("click", (event) => {
  const externalLink = event.target.closest("a[href]");
  if (externalLink && isExternalUrl(externalLink.href)) {
    event.preventDefault();
    openExternalUrl(externalLink.href);
    return;
  }

  const routeButton = event.target.closest("[data-route]");
  if (routeButton) {
    setRoute(routeButton.dataset.route);
    return;
  }

  const actionButton = event.target.closest("[data-action]");
  if (actionButton) {
    handleAction(actionButton);
    return;
  }

  const tabButton = event.target.closest("[data-tab]");
  if (tabButton) {
    state.activeTab = tabButton.dataset.tab;
    const detail = document.querySelector("#other-detail");
    if (detail) detail.innerHTML = boundDetailPanel(state.activeTab);
    hydrateIcons(detail);
    if (["evidence", "reports"].includes(state.activeTab)) loadEvidenceCases();
    return;
  }

  const analysisTabButton = event.target.closest("[data-analysis-tab]");
  if (analysisTabButton) {
    const imgInput = document.querySelector("#image-path");
    if (imgInput) state.imagePathInput = imgInput.value.trim();
    const ramInput = document.querySelector("#ram-analysis-path");
    if (ramInput) state.ramAnalysisPathInput = ramInput.value.trim();

    state.activeAnalysisTab = analysisTabButton.dataset.analysisTab;
    render();
    return;
  }

  const treeNode = event.target.closest(".tree-node");
  if (treeNode) {
    const isDir = treeNode.dataset.isDir === "true";
    const relativePath = treeNode.dataset.path;
    document.querySelectorAll(".tree-node").forEach(el => el.classList.remove("active"));
    treeNode.classList.add("active");
    if (isDir) {
      expandTreeNode(treeNode, relativePath);
    } else {
      previewImageFile(relativePath);
    }
    return;
  }

  const procRow = event.target.closest(".proc-row");
  if (procRow) {
    document.querySelectorAll(".proc-row").forEach(el => el.classList.remove("active"));
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

document.addEventListener("change", (event) => {
  const select = event.target.closest("[data-action='language-select']");
  if (select) {
    setLanguage(select.value);
    showToast(t("settingsSaved"));
    render();
  }

  const target = event.target.closest("[data-field='target']");
  if (target) {
    updateSide("target", target.value || t("targetNotSelected"));
  }

  const caseSelect = event.target.closest("[data-case-select]");
  if (caseSelect) {
    if (caseSelect.value === "__new__") {
      const promptTitle = t("case.promptNewName") || "Lütfen oluşturmak istediğiniz yeni vaka adını girin:";
      const newName = prompt(promptTitle);
      if (newName && newName.trim()) {
        const cleanName = newName.trim();
        const exists = state.cases.some((c) => c.case_name.toLowerCase() === cleanName.toLowerCase());
        if (!exists) {
          state.cases.push({ case_name: cleanName });
        }
        state.activeCase = { case_name: cleanName };
        caseSelect.value = cleanName;
        
        // Mirror to all data-case-select fields on the page
        document.querySelectorAll("[data-case-select]").forEach((el) => {
          el.innerHTML = caseSelectOptions(cleanName, { allowNew: el.dataset.allowNewCase === "1" });
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
        caseSelect.value = fallback;
        document.querySelectorAll("[data-case-select]").forEach((el) => {
          el.value = fallback;
        });
      }
    } else {
      state.activeCase = state.cases.find((c) => c.case_name === caseSelect.value) || { case_name: caseSelect.value };
    }
    toggleCaseCreateInput(caseSelect);
  }

  const androidDeviceSelect = event.target.closest("[data-android-device-select]");
  if (androidDeviceSelect) {
    syncAndroidDeviceSelection(androidDeviceSelect, { state, t, showToast });
  }
});

document.addEventListener("input", (event) => {
  // Empty
});

async function handleAction(button) {
  const action = button.dataset.action;
  if (action?.startsWith("android-")) {
    const handled = await handleAndroidAction(button, {
      apiRequest,
      backendReady,
      state,
      t,
      showToast,
      render,
      resolveCase() {
        const select = document.querySelector("#workflow-case");
        return select?.value || null;
      }
    });
    if (handled) return;
  }

  if (action === "theme-toggle") {
    setTheme(state.theme === "dark" ? "light" : "dark");
    render();
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
        body: JSON.stringify(payload)
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
        body: JSON.stringify({ config_file: configFile })
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
    let payload;
    try {
      payload = connectionPayload();
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
    if (!workflow?.mode.startsWith("remote")) {
      showToast(t("connection.remoteOnly"), "error");
      return;
    }

    forgetConnection(workflowId);
    button.disabled = true;
    updateSide("connection", t("connection.connecting"));
    writeWorkflowLog(t("connection.starting", { host: `${payload.ip}:${payload.port}` }));
    try {
      const result = await apiRequest("/api/connect", {
        method: "POST",
        body: JSON.stringify(payload)
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
        body: JSON.stringify({ path: imagePath })
      });
      state.imageMount = {
        imagePath: result.image_path,
        mountDir: result.mount_dir
      };
      state.imageMountTreeHTML = renderTree(result.tree);
      const container = document.querySelector("#image-tree-root");
      if (container) {
        container.innerHTML = state.imageMountTreeHTML;
      }
      setAnalysisStatus(
        t("analysis.mounted", { path: result.mount_dir }),
        t("analysis.mountedLog")
      );
      showToast(t("analysis.mountPrepared"));
    } catch (error) {
      setAnalysisStatus(t("analysis.noImage"), t("analysis.mountFailed", { message: error.message }));
      showToast(t("analysis.mountFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "unmount-image") {
    try {
      await apiRequest("/api/image-unmount", { method: "POST" });
      state.imageMount = null;
      state.imageMountTreeHTML = "";
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
      setAnalysisStatus(t("analysis.unmounted"), t("analysis.noActiveMount"));
      showToast(t("analysis.unmounted"));
    } catch (error) {
      showToast(t("analysis.unmountFailed", { message: error.message }), "error");
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
        body: JSON.stringify({ path: ramPath })
      });
      
      const count = result.length || 0;
      document.querySelector("#stat-strings-count").textContent = count;
      document.querySelector("#stat-carved-count").textContent = "0";
      document.querySelector("#stat-procs-count").textContent = "0";
      if (statusLbl) statusLbl.textContent = "Analiz Edildi / Analysed";

      if (count === 0) {
        flatResults.innerHTML = `<div class="log-box" style="text-align:center;padding:20px;color:var(--muted)">Hiçbir bulgu dizgisi bulunamadı / No evidential strings found.</div>`;
      } else {
        flatResults.innerHTML = result.map(item => `
          <div class="string-match-item">
            <div class="match-meta">
              <span>Kategori: <strong>${escapeHtml(item.category)}</strong></span>
              <span>Ofset: <strong>0x${item.offset.toString(16).toUpperCase()}</strong></span>
            </div>
            <div class="match-value">${escapeHtml(item.value)}</div>
            <div class="match-context">${escapeHtml(item.context)}</div>
          </div>
        `).join("");
      }
      showToast("Dizgi analizi başarıyla tamamlandı.");
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      flatResults.innerHTML = `<div class="log-box" style="color:#ff5f68">Analiz başarısız: ${escapeHtml(error.message)}</div>`;
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
        body: JSON.stringify({ path: ramPath })
      });

      const count = result.length || 0;
      document.querySelector("#stat-strings-count").textContent = "0";
      document.querySelector("#stat-carved-count").textContent = count;
      document.querySelector("#stat-procs-count").textContent = "0";
      if (statusLbl) statusLbl.textContent = "Kurtarıldı / Carved";

      if (count === 0) {
        leftList.innerHTML = `<div class="log-box" style="text-align:center;padding:12px;color:var(--muted)">Kurtarılan dosya bulunamadı / No carved files.</div>`;
      } else {
        leftList.innerHTML = result.map(file => {
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
        }).join("");
      }
      showToast("Dosya kurtarma (carving) başarıyla tamamlandı.");
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      leftList.innerHTML = `<div class="log-box" style="color:#ff5f68">Kurtarma başarısız: ${escapeHtml(error.message)}</div>`;
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

    document.querySelector("#ram-analysis-results").style.display = "block";
    document.querySelector("#ram-split-view").style.display = "grid";
    document.querySelector("#ram-flat-results-panel").style.display = "none";

    document.querySelector("#ram-left-panel-title").textContent = t("analysis.lblProcesses");
    document.querySelector("#ram-right-panel-title").textContent = "Proses Detayları / Process Inspector";

    const leftList = document.querySelector("#ram-left-list");
    leftList.innerHTML = `<div class="log-box" style="text-align:center;padding:20px">⌛ Proses tablosu çıkartılıyor... / Reading process lists...</div>`;
    const rightContent = document.querySelector("#ram-right-content");
    rightContent.innerHTML = `<div class="log-box" style="display:flex;align-items:center;justify-content:center;color:var(--muted);text-align:center">Proses seçildiğinde bellek haritası ve arama alanları burada açılacak.<br/>Select a process from the left to inspect memory maps.</div>`;

    const statusLbl = document.querySelector("#stat-status-lbl");
    if (statusLbl) statusLbl.textContent = t("analysis.runningAnalysis");

    try {
      const result = await apiRequest("/api/ram-list-processes", {
        method: "POST",
        body: JSON.stringify({ path: ramPath })
      });

      const count = result.length || 0;
      document.querySelector("#stat-strings-count").textContent = "0";
      document.querySelector("#stat-carved-count").textContent = "0";
      document.querySelector("#stat-procs-count").textContent = count;
      if (statusLbl) statusLbl.textContent = "Hazır / Ready";

      if (count === 0) {
        leftList.innerHTML = `<div class="log-box" style="text-align:center;padding:12px;color:var(--muted)">Bu arşivde proses bulunamadı (Sadece .tar arşivleri desteklenir) / No processes.</div>`;
      } else {
        leftList.innerHTML = result.map(proc => `
          <div class="proc-row" data-pid="${escapeHtml(proc.pid)}" data-name="${escapeHtml(proc.name)}">
            <strong>${escapeHtml(proc.pid)}</strong>
            <span style="overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escapeHtml(proc.name)}</span>
            <small style="text-align:right">${formatBytes(proc.dump_size)}</small>
          </div>
        `).join("");
      }
    } catch (error) {
      if (statusLbl) statusLbl.textContent = "Hata / Failed";
      leftList.innerHTML = `<div class="log-box" style="color:#ff5f68">Proses tablosu alınamadı: ${escapeHtml(error.message)}</div>`;
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
    setStatus("[data-settings-status]", `${icon("info")} ${t("settingsSaved")}`);
    showToast(t("settingsSaved"));
    return;
  }

  if (action === "check-update") {
    try {
      setStatus("[data-update-status]", `${icon("refresh")} ${t("settings.updateChecked")}`);
      const result = await apiRequest("/api/update-check");
      state.latestUpdate = result;
      const asset = result.platform_asset || {};
      const assetLine = asset.name ? `<br />Asset: ${escapeHtml(asset.name)} (${formatBytes(asset.size)})` : `<br />${t("settings.noAsset")}`;
      setStatus("[data-update-status]", `${icon("info")} ${t("settings.latestVersion", { version: result.tag_name || result.name || "-" })}`);
      setStatus("[data-update-log]", `${escapeHtml(result.body || t("settings.releaseNotes")).replaceAll("\n", "<br />")}${assetLine}`);
      showToast(t("settings.updateDone"));
    } catch (error) {
      setStatus("[data-update-status]", `${icon("info")} ${t("settings.updateFailed", { message: escapeHtml(error.message) })}`);
      showToast(t("settings.updateFailed", { message: error.message }), "error");
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

  if (action === "add-note") {
    await addEvidenceNote();
    return;
  }

  if (action === "create-report") {
    await createEvidenceReport();
    return;
  }

  const label = button.textContent.trim().replace(/\s+/g, " ");
  writeWorkflowLog(`${label}: ${t("ready")}`);
  showToast(`${label}: ${t("ready")}`);
}

function showToast(message, type = "success") {
  let toast = document.querySelector(".toast");
  if (!toast) {
    toast = document.createElement("div");
    toast.className = "toast";
    document.body.appendChild(toast);
  }
  toast.textContent = message;
  toast.title = message;
  toast.dataset.type = type;
  toast.classList.add("visible");
  window.clearTimeout(showToast.timer);
  showToast.timer = window.setTimeout(
    () => toast.classList.remove("visible"),
    type === "error" ? 12000 : 3200
  );
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
      if (target) target.value = result.path;
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
      if (target) target.value = handle.name;
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
      if (folder && target) target.value = folder;
      if (folder) showToast(t("workflow.selectFolder", { path: folder }));
      input.remove();
      resolve(folder);
    });
    document.body.appendChild(input);
    input.click();
  });
}

function writeWorkflowLog(message) {
  state.lastLog.unshift(message);
  state.lastLog = state.lastLog.slice(0, 8);
  const log = document.querySelector("#workflow-log");
  if (log) log.innerHTML = state.lastLog.map((line) => `• ${line}`).join("<br />");
  updateSide("last-action", message);
}

function updateSide(key, value) {
  const item = document.querySelector(`[data-side="${key}"] small`);
  if (item) item.innerHTML = value;
}

async function scanTargets() {
  const select = document.querySelector("[data-field='target']");
  if (!select) return;
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
            body: JSON.stringify({ ...payload, tool: toolKey })
          });
          status = result.status;
          updateSide("connection", t("connection.checked", { host: `${payload.ip}:${payload.port}` }));
        } else {
          const result = await apiRequest("/api/ram-status");
          status = result[toolKey];
        }
        const label = status?.tool_path || status?.message || localText(workflow.diskLabel);
        const targets = [localText(workflow.diskLabel), label].filter(Boolean);
        select.innerHTML = targets.map((target) => `<option value="${target}">${target}</option>`).join("");
        updateSide("target", targets[0]);
        writeWorkflowLog(t("scan.toolDoneLog", { target: localText(workflow.diskLabel), message: status?.message || t("ready") }));
        showToast(t("scan.done"));
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

    const fallbackTargets = [localText(workflow.diskLabel), workflow.platform === "Windows" ? "WinPMEM portable" : "AVML local"];
    select.innerHTML = fallbackTargets.map((target) => `<option value="${target}">${target}</option>`).join("");
    updateSide("target", fallbackTargets[0]);
    writeWorkflowLog(t("scan.toolListUpdated"));
    showToast(t("scan.done"));
    return;
  }

  if (backendReady()) {
    try {
      let disks = [];
      if (workflow.mode.startsWith("remote")) {
        const payload = connectionPayload();
        if (!payload.ip || !payload.port) {
          showToast(t("connection.ipPortRequired"), "error");
          return;
        }
        if (!requireActiveConnection(workflow, payload)) return;
        const result = await apiRequest("/api/remote-disks", {
          method: "POST",
          body: JSON.stringify(payload)
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
      const disks = await tauriInvoke(workflow.mode.startsWith("remote") ? "remote_disk_list" : "local_disk_list", {
        platform: workflow.platform.toLowerCase()
      });
      const targets = Array.isArray(disks) ? disks.map((disk) => disk.id || disk.name || disk.path || disk).filter(Boolean) : [];
      if (targets.length > 0) {
        select.innerHTML = targets.map((target) => `<option value="${escapeHtml(target)}" data-disk-name="${escapeHtml(target)}">${escapeHtml(target)}</option>`).join("");
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
    const select = document.querySelector("[data-field='target']");
    if (select) {
      const localLabel = localText(workflow.diskLabel);
      select.innerHTML = [
        `<option value="${escapeHtml(localLabel)}">${escapeHtml(localLabel)}</option>`,
        `<option value="${escapeHtml(path)}">${escapeHtml(path)}</option>`
      ].join("");
      select.value = path;
    }
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
    const path = status.tool_path || result.path || "C:\\Tools\\go-winpmem_amd64_1.0-rc2_signed.exe";
    const label = status.message || result.message || "WinPMEM ready";
    const select = document.querySelector("[data-field='target']");
    if (select) {
      const localLabel = localText(workflow.diskLabel);
      select.innerHTML = [
        `<option value="${escapeHtml(localLabel)}">${escapeHtml(localLabel)}</option>`,
        `<option value="${escapeHtml(path)}">${escapeHtml(path)}</option>`
      ].join("");
      select.value = path;
    }
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
  const next = `${value}%`;
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

async function waitForAcquisitionJob(jobId) {
  while (true) {
    const job = await apiRequest("/api/acquisition-status", {
      method: "POST",
      body: JSON.stringify({ job_id: jobId })
    });
    const percent = acquisitionPercent(job);
    setProgress(percent, `${percent}%`);
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

async function sendAcquisitionControl(action) {
  const active = state.activeAcquisition;
  if (!active || !active.jobId || !active.workflowId) {
    showToast(t("workflow.activeJobMissing"), "error");
    return;
  }
  const workflow = workflows[active.workflowId];
  const body = {
    job_id: active.jobId,
    action
  };
  if (workflow?.mode.startsWith("remote")) {
    Object.assign(body, active.payload || {});
  }

  await apiRequest("/api/acquisition-control", {
    method: "POST",
    body: JSON.stringify(body)
  });
  const label = action === "stop" ? t("workflow.stopLabel") : action === "pause" ? t("workflow.pauseLabel") : t("workflow.resumeLabel");
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
  if (workflow?.mode.startsWith("remote")) {
    try {
      payload = connectionPayload();
    } catch (error) {
      showToast(error.message, "error");
      return;
    }
    if (!requireActiveConnection(workflow, payload)) return;
  }
  const target = document.querySelector("[data-field='target']")?.value.trim();
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
  const operation = isRam ? t("ramAcquisition") : t("imageAcquisition");

  try {
    await loadEvidenceCases();
    const evidenceCase = await ensureImageCase();
    caseName = evidenceCase.case_name;
    if (isRam) {
      const remoteIp = workflow?.mode.startsWith("remote") ? payload?.ip : "";
      const fileName = canonicalRamFileName(remoteIp);
      const outputInput = document.querySelector("#workflow-output");
      if (outputInput) outputInput.value = fileName;
      const ramDir = evidenceCase.ram_dir || `${evidenceCase.case_dir}/ram`;
      output = `${ramDir}/${fileName}`;
    } else {
      output = evidenceCase.output_dir || `${evidenceCase.case_dir}/ciktilar`;
    }
    document.querySelectorAll("[data-case-output]").forEach((outputNode) => {
      outputNode.textContent = outputNode.dataset.caseOutputSubdir === "ram"
        ? (evidenceCase.ram_dir || `${evidenceCase.case_dir}/ram`)
        : (evidenceCase.output_dir || `${evidenceCase.case_dir}/ciktilar`);
    });

    writeWorkflowLog(t("workflow.operationStarted", { operation }));
    updateSide("last-action", t("workflow.operationRunning", { operation }));
    if (workflow?.mode.startsWith("remote")) updateSide("connection", t("workflow.operationRunning", { operation }));

    const start = workflow?.mode.startsWith("remote")
      ? await apiRequest(isRam ? "/api/remote-ram" : "/api/remote-image", {
          method: "POST",
          body: JSON.stringify(isRam
            ? {
                ...payload,
                output,
                case_name: caseName
              }
            : {
                ...payload,
                disk_id: target,
                disk_name: diskName,
                output,
                case_name: caseName
              })
        })
      : await apiRequest(isRam ? "/api/local-ram" : "/api/local-image", {
          method: "POST",
          body: JSON.stringify(isRam
            ? {
                output,
                tool: workflow.platform === "Windows" ? "winpmem" : "avml",
                tool_path: target,
                case_name: caseName
            }
            : {
                source: target,
                disk_name: diskName,
                output,
                case_name: caseName
              })
        });
    if (!start.job_id) throw new Error(t("workflow.jobIdMissing"));
    state.activeAcquisition = {
      jobId: start.job_id,
      workflowId: routeId,
      payload
    };
    const result = await waitForAcquisitionJob(start.job_id);

    setProgress(100);
    const targetPath = result.target_path || result.target || output;
    writeWorkflowLog(t("workflow.operationCompletedPath", { operation, path: targetPath }));
    updateSide("last-action", t("workflow.operationCompleted", { operation }));
    if (workflow?.mode.startsWith("remote") && payload) {
      updateSide("connection", t("connection.connected", { ip: payload.ip }));
    }
    showToast(t("workflow.operationCompleted", { operation }));
  } catch (error) {
    setProgress(0);
    writeWorkflowLog(t("workflow.operationFailedDetail", { operation, message: error.message }));
    updateSide("last-action", t("workflow.operationFailed", { operation }));
    if (workflow?.mode.startsWith("remote")) {
      updateSide("connection", t("workflow.operationFailed", { operation }));
    }
    showToast(t("workflow.operationFailedDetail", { operation, message: error.message }), "error");
  } finally {
    state.activeAcquisition = null;
    button.disabled = false;
  }
}

function setAnalysisStatus(status, log) {
  const statusNode = document.querySelector("[data-analysis-status]");
  const logNode = document.querySelector("[data-analysis-log]");
  if (statusNode) statusNode.textContent = status;
  if (logNode) logNode.innerHTML = log;
}

function renderTree(node, depth = 0) {
  if (!node) return `<div class="log-box">${escapeHtml(t("analysis.outputWaiting"))}</div>`;
  const fileIcon = node.is_dir ? "📁" : "📄";
  const toggle = node.is_dir ? `<span class="toggle-icon">▸</span>` : "";
  const sizeStr = node.is_dir ? "" : `<span class="node-size">${formatBytes(node.size)}</span>`;
  
  let relativePath = node.path;
  if (state.imageMount && state.imageMount.mountDir) {
    if (node.path.startsWith(state.imageMount.mountDir)) {
      relativePath = node.path.substring(state.imageMount.mountDir.length);
    }
  }

  const current = `
    <div class="tree-node" data-path="${escapeHtml(relativePath)}" data-is-dir="${node.is_dir}">
      <span style="width:16px;display:inline-block">${toggle}</span>
      <span class="node-icon">${fileIcon}</span>
      <span class="node-name">${escapeHtml(node.name || node.path.split('/').pop() || "/")}</span>
      ${sizeStr}
    </div>
    <div class="tree-children-container"></div>
  `;

  const children = Array.isArray(node.children) && node.children.length > 0
    ? `<div class="tree-children" style="padding-left:14px; display:none">${node.children.map(child => renderTree(child, depth + 1)).join("")}</div>`
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
          algorithms: ["md5", "sha1", "sha256", "sha512"]
        })
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
  const values = [...document.querySelectorAll("[data-hash-result] strong")].map((node) => node.textContent.trim().toLowerCase());
  const result = document.querySelector("[data-hash-compare-result] small");
  if (!expected) {
    showToast(t("hash.compareRequired"), "error");
    return;
  }
  const matched = values.includes(expected);
  if (result) result.textContent = matched ? t("hash.matched") : t("hash.notMatched");
  showToast(matched ? t("hash.matchedToast") : t("hash.notMatchedToast"), matched ? "success" : "error");
}

function setStatus(selector, html) {
  const node = document.querySelector(selector);
  if (node) node.innerHTML = html;
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
      body: JSON.stringify({ case_name: caseName })
    });
    state.activeCase = result;
    await loadEvidenceCases();
    setStatus("[data-case-status]", `${icon("info")} ${t("case.created", { path: escapeHtml(result.case_dir) })}`);
    showToast(t("case.created", { path: result.case_dir }));
  } catch (error) {
    setStatus("[data-case-status]", `${icon("info")} ${t("case.createFailed", { message: escapeHtml(error.message) })}`);
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
      body: JSON.stringify({ subdir })
    });
    const files = result.files || [];
    const select = document.querySelector("#case-file-list");
    if (select) {
      select.innerHTML = files.length
        ? files.map((file) => `<option value="${escapeHtml(file.path)}">${escapeHtml(file.name)} · ${formatBytes(file.size)}</option>`).join("")
        : `<option>${t("case.empty")}</option>`;
    }
    setStatus("[data-case-status]", `${icon("info")} ${t("case.filesListed", { count: String(files.length) })}`);
    showToast(t("case.filesListed", { count: String(files.length) }));
  } catch (error) {
    showToast(t("case.listFailed", { message: error.message }), "error");
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
      body: JSON.stringify({ note, case_name: caseName })
    });
    await loadEvidenceCases();
    setStatus("[data-report-status]", `${icon("info")} ${t("report.noteAdded", { path: escapeHtml(result.path) })}`);
    showToast(t("report.noteAdded", { path: result.path }));
  } catch (error) {
    setStatus("[data-report-status]", `${icon("info")} ${t("report.noteFailed", { message: escapeHtml(error.message) })}`);
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
      body: JSON.stringify({ case_name: caseName, title, description, format })
    });
    await loadEvidenceCases();
    setStatus("[data-report-status]", `${icon("info")} ${t("report.created", { path: escapeHtml(result.path) })}`);
    showToast(t("report.created", { path: result.path }));
  } catch (error) {
    setStatus("[data-report-status]", `${icon("info")} ${t("report.failed", { message: escapeHtml(error.message) })}`);
    showToast(t("report.failed", { message: error.message }), "error");
  }
}

async function downloadUpdatePackage() {
  const progress = document.querySelector("[data-update-progress]");
  const status = document.querySelector("[data-update-status]");
  const update = state.latestUpdate || await apiRequest("/api/update-check");
  state.latestUpdate = update;
  const asset = update.platform_asset || {};
  if (!asset.download_url) {
    showToast(t("settings.noAsset"), "error");
    return;
  }
  if (progress) {
    progress.style.setProperty("--value", "35%");
    const label = progress.querySelector("b");
    if (label) label.textContent = "35%";
  }
  if (status) status.innerHTML = `${icon("download")} ${t("settings.downloading")}`;
  try {
    const result = await apiRequest("/api/update-download", {
      method: "POST",
      body: JSON.stringify({
        url: asset.download_url,
        name: asset.name,
        expected_sha256: asset.digest || ""
      })
    });
    if (progress) {
      progress.style.setProperty("--value", "75%");
      const label = progress.querySelector("b");
      if (label) label.textContent = "75%";
    }
    if (status) status.innerHTML = `${icon("download")} ${t("settings.installing")}`;
    const install = await apiRequest("/api/update-install", {
      method: "POST",
      body: JSON.stringify({ path: result.path })
    });
    if (progress) {
      progress.style.setProperty("--value", "100%");
      const label = progress.querySelector("b");
      if (label) label.textContent = "100%";
    }
    if (status) status.innerHTML = `${icon("shield")} ${t("settings.installStarted")}`;
    setStatus(
      "[data-update-log]",
      `${t("settings.downloaded", { path: escapeHtml(result.path) })}<br />${t("settings.sha256", { hash: escapeHtml(result.sha256) })}<br />${escapeHtml(install.message || t("settings.installStarted"))}`
    );
    showToast(t("settings.installStarted"));
  } catch (error) {
    if (progress) {
      progress.style.setProperty("--value", "0%");
      const label = progress.querySelector("b");
      if (label) label.textContent = "0%";
    }
    const failedKey = String(error.message || "").toLowerCase().includes("installer")
      ? "settings.installFailed"
      : "settings.downloadFailed";
    if (status) status.innerHTML = `${icon("info")} ${t(failedKey, { message: escapeHtml(error.message) })}`;
    showToast(t(failedKey, { message: error.message }), "error");
  }
}

async function expandTreeNode(nodeElement, relativePath) {
  const childrenContainer = nodeElement.nextElementSibling;
  if (childrenContainer && childrenContainer.classList.contains("tree-children")) {
    if (childrenContainer.style.display === "none") {
      childrenContainer.style.display = "block";
      nodeElement.querySelector(".toggle-icon").innerHTML = "▾";
    } else {
      childrenContainer.style.display = "none";
      nodeElement.querySelector(".toggle-icon").innerHTML = "▸";
    }
    return;
  }

  const tempContainer = document.createElement("div");
  tempContainer.className = "tree-children";
  tempContainer.style.paddingLeft = "14px";
  nodeElement.parentNode.insertBefore(tempContainer, nodeElement.nextSibling.nextSibling);

  try {
    nodeElement.querySelector(".toggle-icon").innerHTML = "⌛";
    const result = await apiRequest("/api/image-browse", {
      method: "POST",
      body: JSON.stringify({ path: relativePath })
    });
    
    let html = "";
    if (result.files && result.files.length > 0) {
      result.files.sort((a, b) => b.is_dir - a.is_dir || a.name.localeCompare(b.name));
      result.files.forEach(file => {
        const fileIcon = file.is_dir ? "📁" : "📄";
        const toggle = file.is_dir ? `<span class="toggle-icon">▸</span>` : "";
        const sizeStr = file.is_dir ? "" : `<span class="node-size">${formatBytes(file.size)}</span>`;
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

async function previewImageFile(relativePath) {
  const container = document.querySelector("#image-file-preview");
  if (!container) return;
  container.innerHTML = `<div class="log-box" style="display:flex;align-items:center;justify-content:center;color:var(--muted);height:200px">⌛ Yükleniyor / Loading...</div>`;
  
  try {
    const result = await apiRequest("/api/image-read-file", {
      method: "POST",
      body: JSON.stringify({ path: relativePath })
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
          <strong style="color:var(--green);font-size:14px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escapeHtml(relativePath.split('/').pop())}</strong>
          <small class="meta" style="margin-top:0">${formatBytes(result.size)}</small>
        </div>
        <div style="flex:1;overflow:hidden">
          ${contentHtml}
        </div>
      </div>
    `;
  } catch (error) {
    container.innerHTML = `<div class="log-box" style="color:#ff5f68;padding:20px">Hata / Error: ${escapeHtml(error.message)}</div>`;
  }
}

async function inspectProcessDetails(pid, name) {
  const rightContent = document.querySelector("#ram-right-content");
  if (!rightContent) return;
  
  rightContent.innerHTML = `<div class="log-box" style="text-align:center;padding:20px">⌛ Proses bellek haritası yükleniyor...</div>`;
  
  const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
  try {
    const result = await apiRequest("/api/ram-process-details", {
      method: "POST",
      body: JSON.stringify({ path: ramPath, pid })
    });
    
    rightContent.innerHTML = `
      <div style="display:flex;flex-direction:column;gap:14px;padding:12px">
        <div style="display:flex;justify-content:space-between;align-items:center;border-bottom:1px solid var(--line);padding-bottom:8px">
          <strong style="color:var(--green)">${escapeHtml(name)} (${escapeHtml(pid)})</strong>
          <span style="font-size:12px;color:var(--muted)">Döküm: ${result.dumps?.length || 0} segment</span>
        </div>
        
        <p style="margin:0;font-size:12px;font-weight:bold;color:var(--muted)">Bellek Haritaları (Maps)</p>
        <div class="maps-pre-box">${escapeHtml(result.maps || "Bellek haritası yok")}</div>
        
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
    rightContent.innerHTML = `<div class="log-box" style="color:#ff5f68">Hata: ${escapeHtml(error.message)}</div>`;
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
  
  resultsDiv.innerHTML = `<div class="log-box" style="text-align:center;padding:10px">⌛ Proses hafızası taranıyor...</div>`;
  
  const ramPath = document.querySelector("#ram-analysis-path")?.value.trim();
  try {
    const result = await apiRequest("/api/ram-process-search", {
      method: "POST",
      body: JSON.stringify({ path: ramPath, pid, query })
    });
    
    const count = result.length || 0;
    if (count === 0) {
      resultsDiv.innerHTML = `<div class="log-box" style="color:var(--muted);text-align:center;padding:10px">Hiçbir eşleşme bulunamadı.</div>`;
    } else {
      resultsDiv.innerHTML = `
        <div class="strings-results-list" style="max-height:220px">
          ${result.map(item => `
            <div class="string-match-item">
              <div class="match-meta">
                <span>Segment: <strong>${escapeHtml(item.category)}</strong></span>
                <span>Ofset: <strong>0x${item.offset.toString(16).toUpperCase()}</strong></span>
              </div>
              <div class="match-value" style="color:var(--green)">${escapeHtml(item.value)}</div>
              <div class="match-context">${escapeHtml(item.context)}</div>
            </div>
          `).join("")}
        </div>
      `;
    }
  } catch (error) {
    resultsDiv.innerHTML = `<div class="log-box" style="color:#ff5f68">Arama başarısız: ${escapeHtml(error.message)}</div>`;
  }
}

async function previewCarvedFile(filePath) {
  const rightContent = document.querySelector("#ram-right-content");
  if (!rightContent) return;
  
  rightContent.innerHTML = `<div class="log-box" style="text-align:center;padding:20px">⌛ Kurtarılan dosya yükleniyor... / Loading carved file...</div>`;
  
  try {
    const result = await apiRequest("/api/ram-read-carved", {
      method: "POST",
      body: JSON.stringify({ path: filePath })
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
          <strong style="color:var(--green);font-size:14px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">${escapeHtml(filePath.split('/').pop())}</strong>
          <small class="meta" style="margin-top:0">${formatBytes(result.size)}</small>
        </div>
        <div style="flex:1;overflow:hidden">
          ${contentHtml}
        </div>
      </div>
    `;
  } catch (error) {
    rightContent.innerHTML = `<div class="log-box" style="color:#ff5f68;padding:20px">Önizleme başarısız: ${escapeHtml(error.message)}</div>`;
  }
}

setLanguage(state.language);
setTheme(state.theme);
installUiErrorHandlers();
hydrateIcons();
render();
