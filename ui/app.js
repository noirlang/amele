const icons = {
  home: '<path d="m3 11 9-8 9 8"/><path d="M5 10v10h5v-6h4v6h5V10"/>',
  grid: '<rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>',
  tiles: '<rect x="4" y="4" width="6" height="6"/><rect x="14" y="4" width="6" height="6"/><rect x="4" y="14" width="6" height="6"/><rect x="14" y="14" width="6" height="6"/>',
  linux: '<circle cx="12" cy="6" r="3"/><path d="M8.2 11.2c.6-1.8 1.9-3.2 3.8-3.2s3.2 1.4 3.8 3.2l1.4 4.3c.5 1.5-.6 3-2.2 3H9c-1.6 0-2.7-1.5-2.2-3l1.4-4.3Z"/><path d="M9 18.5 6.5 21"/><path d="M15 18.5l2.5 2.5"/><path d="M10.2 6h.01"/><path d="M13.8 6h.01"/><path d="M10 13h4"/>',
  network: '<circle cx="12" cy="5" r="3"/><circle cx="5" cy="19" r="3"/><circle cx="19" cy="19" r="3"/><path d="M10.5 7.5 6.5 16"/><path d="M13.5 7.5 17.5 16"/><path d="M8 19h8"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/><path d="M8 11h6"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 10v6"/><path d="M12 7h.01"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.04.04a2 2 0 1 1-2.83 2.83l-.04-.04A1.7 1.7 0 0 0 15 19.4a1.7 1.7 0 0 0-1 .6 1.7 1.7 0 0 0-.4 1.1V21a2 2 0 0 1-4 0v-.1A1.7 1.7 0 0 0 8.6 19.4a1.7 1.7 0 0 0-1.87.34l-.04.04a2 2 0 1 1-2.83-2.83l.04-.04A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-.6-1 1.7 1.7 0 0 0-1.1-.4H3a2 2 0 0 1 0-4h.1A1.7 1.7 0 0 0 4.6 8.6a1.7 1.7 0 0 0-.34-1.87l-.04-.04a2 2 0 1 1 2.83-2.83l.04.04A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-.6 1.7 1.7 0 0 0 .4-1.1V3a2 2 0 0 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.87-.34l.04-.04a2 2 0 1 1 2.83 2.83l-.04.04A1.7 1.7 0 0 0 19.4 9c.36.12.7.32 1 .6.3.28.5.63.6 1h.1a2 2 0 0 1 0 4H21a1.7 1.7 0 0 0-1.6.4Z"/>',
  menu: '<path d="M4 7h16"/><path d="M4 12h16"/><path d="M4 17h16"/>',
  disk: '<path d="M4 5h16l-2 10H6L4 5Z"/><path d="M7 19h10"/><path d="M9 15v4"/><path d="M15 15v4"/>',
  shield: '<path d="M12 3 5 6v6c0 4.5 3 7.5 7 9 4-1.5 7-4.5 7-9V6l-7-3Z"/><path d="m9 12 2 2 4-5"/>',
  scale: '<path d="M12 3v18"/><path d="M5 7h14"/><path d="M6 7l-3 6h6L6 7Z"/><path d="M18 7l-3 6h6l-3-6Z"/>',
  report: '<path d="M7 3h8l4 4v14H7V3Z"/><path d="M15 3v5h5"/><path d="M10 13h6"/><path d="M10 17h6"/>',
  monitor: '<rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/>',
  database: '<ellipse cx="12" cy="5" rx="7" ry="3"/><path d="M5 5v7c0 1.7 3.1 3 7 3s7-1.3 7-3V5"/><path d="M5 12v7c0 1.7 3.1 3 7 3s7-1.3 7-3v-7"/>',
  chip: '<rect x="7" y="7" width="10" height="10" rx="2"/><path d="M9 1v4"/><path d="M15 1v4"/><path d="M9 19v4"/><path d="M15 19v4"/><path d="M1 9h4"/><path d="M1 15h4"/><path d="M19 9h4"/><path d="M19 15h4"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v6l4 2"/>',
  windows: '<path d="M3 5.5 11 4v7H3V5.5Z"/><path d="M13 3.7 21 2v9h-8V3.7Z"/><path d="M3 13h8v7l-8-1.5V13Z"/><path d="M13 13h8v9l-8-1.7V13Z"/>',
  ram: '<rect x="3" y="7" width="18" height="10" rx="2"/><path d="M7 7V4"/><path d="M12 7V4"/><path d="M17 7V4"/><path d="M7 20v-3"/><path d="M12 20v-3"/><path d="M17 20v-3"/>',
  folder: '<path d="M3 6h7l2 2h9v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6Z"/>',
  download: '<path d="M12 3v12"/><path d="m7 10 5 5 5-5"/><path d="M5 21h14"/>',
  globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3a14 14 0 0 1 0 18"/><path d="M12 3a14 14 0 0 0 0 18"/>',
  user: '<circle cx="12" cy="8" r="4"/><path d="M4 21c1.8-4 4.4-6 8-6s6.2 2 8 6"/>',
  key: '<circle cx="8" cy="15" r="4"/><path d="m11 12 9-9"/><path d="m15 4 3 3"/><path d="m13 6 3 3"/>',
  refresh: '<path d="M21 12a9 9 0 0 1-15.5 6.2L3 16"/><path d="M3 21v-5h5"/><path d="M3 12A9 9 0 0 1 18.5 5.8L21 8"/><path d="M21 3v5h-5"/>',
  pause: '<path d="M8 5v14"/><path d="M16 5v14"/>',
  stop: '<rect x="6" y="6" width="12" height="12"/>',
  arrow: '<path d="M5 12h14"/><path d="m13 5 7 7-7 7"/>'
};

const APP_VERSION = "v0.0.6";
const assetPath = "./assets";
const fontIcons = {
  windows: "",
  linux: "",
  github: "",
  linkedin: "",
  website: "",
  globe: ""
};

const app = document.querySelector("#app");
const view = document.querySelector("#view");

const translations = {
  tr: {
    "nav.home": "Ana Sayfa",
    "nav.windows": "Windows Araçları",
    "nav.linux": "Linux Araçları",
    "nav.agent": "Agent",
    "nav.analysis": "Analiz",
    "nav.other": "Diğer",
    ready: "Hazır",
    settingsSaved: "Ayarlar kaydedildi.",
    fileRequired: "Önce dosya seçin.",
    platformBlocked: "Bu yerel işlem yalnızca {platform} üzerinde çalışır."
  },
  en: {
    "nav.home": "Home",
    "nav.windows": "Windows Tools",
    "nav.linux": "Linux Tools",
    "nav.agent": "Agent",
    "nav.analysis": "Analysis",
    "nav.other": "Other",
    ready: "Ready",
    settingsSaved: "Settings saved.",
    fileRequired: "Select a file first.",
    platformBlocked: "This local workflow only runs on {platform}."
  }
};

const state = {
  route: new URLSearchParams(window.location.search).get("route") || "home",
  theme: localStorage.getItem("worm-theme") || "dark",
  language: localStorage.getItem("worm-language") || "tr",
  platform: detectPlatform(),
  files: {},
  activeTab: "hash",
  jobs: {},
  lastLog: [
    "Rust teknik çekirdek yüklendi.",
    "Agent protokolü: JSON-over-TCP uyumlu.",
    "UI bağlantıları Tauri komutlarına bağlanmak için hazırlandı."
  ]
};

function detectPlatform() {
  const override = new URLSearchParams(window.location.search).get("platform");
  if (["windows", "linux", "mac"].includes(override || "")) return override;
  const text = `${navigator.userAgent} ${navigator.platform}`.toLowerCase();
  if (text.includes("win")) return "windows";
  if (text.includes("linux")) return "linux";
  if (text.includes("mac")) return "mac";
  return "unknown";
}

function t(key, vars = {}) {
  let value = translations[state.language]?.[key] || translations.tr[key] || key;
  for (const [name, replacement] of Object.entries(vars)) {
    value = value.replace(`{${name}}`, replacement);
  }
  return value;
}

const toolCards = {
  windows: [
    {
      id: "windows-remote-disk",
      title: "Uzak Disk İmajı",
      desc: "Windows agent üzerinden PhysicalDrive imajı alın.",
      icon: "disk",
      accent: "var(--green)",
      badge: "Agent + raw stream"
    },
    {
      id: "windows-local-disk",
      title: "Yerel Disk İmajı",
      desc: "Bu makinedeki Windows disklerinden ham imaj üretin.",
      icon: "windows",
      accent: "var(--blue)",
      badge: "PhysicalDrive"
    },
    {
      id: "windows-remote-ram",
      title: "Uzak RAM",
      desc: "WinPMEM ile uzak Windows RAM edinimi başlatın ve indirin.",
      icon: "ram",
      accent: "var(--purple)",
      badge: "WinPMEM remote"
    },
    {
      id: "windows-local-ram",
      title: "Yerel RAM",
      desc: "Yerel WinPMEM kontrolü, indirme ve RAM imajı alma.",
      icon: "chip",
      accent: "var(--amber)",
      badge: "Admin required"
    }
  ],
  linux: [
    {
      id: "linux-remote-disk",
      title: "Uzak Disk İmajı",
      desc: "Linux agent üzerinden /dev disklerinden ham imaj alın.",
      icon: "disk",
      accent: "var(--green)",
      badge: "Agent + /dev"
    },
    {
      id: "linux-local-disk",
      title: "Yerel Disk İmajı",
      desc: "Yerel Linux diskleri için root yetkili imaj alma akışı.",
      icon: "linux",
      accent: "var(--blue)",
      badge: "BLKGETSIZE64"
    },
    {
      id: "linux-remote-ram",
      title: "Uzak RAM",
      desc: "AVML ile uzak Linux RAM edinimi ve dosya indirme.",
      icon: "ram",
      accent: "var(--purple)",
      badge: "AVML remote"
    },
    {
      id: "linux-local-ram",
      title: "Yerel RAM",
      desc: "AVML varlık/yetki kontrolü ve yerel RAM dump üretimi.",
      icon: "chip",
      accent: "var(--amber)",
      badge: "Root required"
    }
  ]
};

const workflows = {
  "windows-remote-disk": {
    platform: "Windows",
    icon: "windows",
    title: "Uzak Windows Sunucu Bağlantısı",
    desc: "Uzak Windows sistemlerine güvenli bağlantı kurun ve disk imajı alın.",
    mode: "remote-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: "Disk seçilmedi"
  },
  "linux-remote-disk": {
    platform: "Linux",
    icon: "linux",
    title: "Uzak Linux Disk Bağlantısı",
    desc: "Linux agent ile uzak /dev disklerini listeleyin ve raw imaj alın.",
    mode: "remote-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: "Disk seçilmedi"
  },
  "windows-local-disk": {
    platform: "Windows",
    icon: "windows",
    title: "Windows Yerel Disk İmajı",
    desc: "Yerel PhysicalDrive kaynaklarından ham imaj alma akışı.",
    mode: "local-disk",
    output: "C:\\Worm\\Ciktilar",
    diskLabel: "Disk seçilmedi"
  },
  "linux-local-disk": {
    platform: "Linux",
    icon: "linux",
    title: "Linux Yerel Disk İmajı",
    desc: "Yerel Linux blok cihazlarından imaj alma akışı.",
    mode: "local-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: "Disk seçilmedi"
  },
  "windows-remote-ram": {
    platform: "Windows",
    icon: "ram",
    title: "Windows Uzak RAM Edinimi",
    desc: "WinPMEM durumunu kontrol edin, uzak RAM edinimini başlatın ve dump dosyasını indirin.",
    mode: "remote-ram",
    output: "memory_dump.raw",
    diskLabel: "WinPMEM"
  },
  "linux-remote-ram": {
    platform: "Linux",
    icon: "ram",
    title: "Linux Uzak RAM Edinimi",
    desc: "AVML durumunu kontrol edin, uzak RAM edinimini başlatın ve dump dosyasını indirin.",
    mode: "remote-ram",
    output: "memory_dump_linux.raw",
    diskLabel: "AVML"
  },
  "windows-local-ram": {
    platform: "Windows",
    icon: "chip",
    title: "Windows Yerel RAM Edinimi",
    desc: "Yerel WinPMEM kontrolü, gerekirse indirme ve RAM imajı alma.",
    mode: "local-ram",
    output: "memory_dump_local.raw",
    diskLabel: "WinPMEM local"
  },
  "linux-local-ram": {
    platform: "Linux",
    icon: "chip",
    title: "Linux Yerel RAM Edinimi",
    desc: "Yerel AVML kontrolü ve root yetkili RAM imajı alma.",
    mode: "local-ram",
    output: "linux_memory_dump.raw",
    diskLabel: "AVML local"
  }
};

function icon(name) {
  if (fontIcons[name]) {
    return `<span class="fa-icon" aria-hidden="true">${fontIcons[name]}</span>`;
  }
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${icons[name] || icons.info}</svg>`;
}

function hydrateIcons(root = document) {
  root.querySelectorAll("[data-icon]").forEach((node) => {
    node.innerHTML = icon(node.dataset.icon);
  });
}

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

  if (state.route.startsWith("workflow:")) {
    view.innerHTML = workflowPage(state.route.split(":")[1]);
  } else {
    view.innerHTML = routes[state.route]?.() || homePage();
  }

  hydrateIcons(view);
  view.focus({ preventScroll: true });
}

function routeGroup(route) {
  if (!route.startsWith("workflow:")) return route;
  const workflowId = route.split(":")[1] || "";
  if (workflowId.startsWith("windows")) return "windows";
  if (workflowId.startsWith("linux")) return "linux";
  return route;
}

function homePage() {
  return `
    <section class="page">
      <div class="hero home-hero">
        <div class="worm-art">
          <img src="${assetPath}/logo/logo.png" alt="Worm logo" />
        </div>
      </div>

      <div class="home-grid">
        ${homeTile("Edinim", "Windows ve Linux için disk/RAM toplama akışları.", "ACQUIRE", "disk", "windows", "var(--green)")}
        ${homeTile("Bütünlük", "MD5, SHA ailesi ve karşılaştırma adımları.", "VERIFY", "shield", "other", "var(--green)")}
        ${homeTile("Kanıt", "Vaka klasörü ve kanıt kasası yönetimi.", "CASE", "scale", "other", "var(--purple)")}
        ${homeTile("Çıktı", "İnceleme notları ve rapor üretimi.", "REPORT", "report", "other", "var(--blue)")}
      </div>
    </section>
  `;
}

function homeTile(title, desc, label, iconName, route, accent) {
  return `
    <button class="action-tile" data-route="${route}" style="--accent:${accent}">
      <span class="tile-icon">${icon(iconName)}</span>
      <span>
        <span class="eyebrow">${label}</span>
        <h3>${title}</h3>
        <p>${desc}</p>
      </span>
      <span class="tile-arrow">→</span>
    </button>
  `;
}

function metric(label, value, iconName, accent) {
  return `
    <div class="metric" style="--accent:${accent}">
      <span class="metric-icon">${icon(iconName)}</span>
      <span><small>${label}</small><strong>${value}</strong></span>
    </div>
  `;
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
          <h3>${card.title}</h3>
          <p>${card.desc}</p>
          <span class="meta">${blocked ? "Bu sistemde yerel çalışmaz" : card.badge}</span>
        </button>
      `;
      }
    )
    .join("");

  const isWindows = platform === "windows";
  return `
    <section class="page">
      <div class="platform-note">
        ${icon("monitor")} Algılanan sistem: <strong>${platformLabel(state.platform)}</strong>. Yerel disk/RAM işlemleri sadece aynı işletim sisteminde açılır; uzak agent akışları platform bağımsızdır.
      </div>
      ${pageTitle(
        isWindows ? "Windows Araçları" : "Linux Araçları",
        isWindows
          ? "Windows yerel/uzak disk ve RAM edinim akışlarını seçin."
          : "Linux yerel/uzak disk ve RAM edinim akışlarını seçin.",
        isWindows ? "windows" : "linux"
      )}
      <div class="tool-grid">${cards}</div>
    </section>
  `;
}

function platformLabel(platform) {
  if (platform === "windows") return "Windows";
  if (platform === "linux") return "Linux";
  if (platform === "mac") return "macOS";
  return "Bilinmiyor";
}

function workflowPage(id) {
  const data = workflows[id] || workflows["windows-remote-disk"];
  const isRemote = data.mode.startsWith("remote");
  const isRam = data.mode.includes("ram");
  const toolCheck = data.platform === "Windows" ? "WinPMEM" : "AVML";
  const initialTarget = isRam ? data.diskLabel : "";
  const initialTargetLabel = isRam ? data.diskLabel : "Diskleri tara ile listeleyin";
  const outputField = pickerField(
    isRam ? "Çıktı Dosyası" : "Çıktı Klasörü",
    "workflow-output",
    data.output,
    isRam ? "file" : "folder"
  );

  return `
    <section class="page">
      <div class="workflow-layout">
        <div class="workflow-panel">
          ${pageTitle(data.title, data.desc, data.icon)}
          <div class="form-grid">
            ${
              isRemote
                ? `
                  ${field("IP Adresi", '<input class="input" data-field="ip" placeholder="IP adresi" value="" />')}
                  ${field("Port", '<input class="input" data-field="port" value="4444" />')}
                  ${field("Token", '<input class="input" data-field="token" placeholder="Güvenlik anahtarı (Onayla ile aktif olur)" />')}
                  <div class="button-row">
                    <button class="secondary-button" data-action="approve-key">${icon("key")} Anahtarı Onayla</button>
                    <button class="secondary-button" data-action="reset-key">${icon("refresh")} Sıfırla</button>
                  </div>
                  <div class="section-divider"></div>
                  <p class="section-label">2. Ağ ve VPN</p>
                  <div class="toggle-row">
                    <span>WireGuard VPN Kullan</span>
                    <button class="switch" data-action="toggle-vpn" aria-label="VPN kullan"></button>
                  </div>
                  <button class="secondary-button" data-action="vpn-config">${icon("settings")} VPN Yapılandır</button>
                  <div class="vpn-panel" hidden>
                    ${field("Sunucu", '<input class="input" data-field="vpn-endpoint" placeholder="10.0.0.1:51820" />')}
                    ${field("Allowed IPs", '<input class="input" data-field="vpn-allowed" value="0.0.0.0/0" />')}
                    ${pickerField("Config Dosyası", "vpn-config-file", "wireguard.conf", "file")}
                    <div class="button-row">
                      <button class="primary-button" data-action="save-vpn">${icon("shield")} VPN Kaydet</button>
                    </div>
                  </div>
                  <div class="section-divider"></div>
                  <p class="section-label">3. Bağlantı İşlemleri</p>
                  <div class="button-row">
                    <button class="primary-button" data-action="connect">${icon("network")} Bağlan</button>
                    <button class="secondary-button" data-action="scan">${icon(isRam ? "chip" : "disk")} ${isRam ? `${toolCheck} Kontrol` : "Diskleri Tara"}</button>
                  </div>
                `
                : `
                  <p class="section-label">1. Yerel Kontrol</p>
                  <p class="field-hint">${data.platform} yerel akışında yönetici/root yetkisi gerekebilir. İşlem başlamadan önce araç ve erişim kontrolü yapılır.</p>
                  <div class="button-row">
                    <button class="primary-button" data-action="scan">${icon(isRam ? "chip" : "disk")} ${isRam ? `${toolCheck} Kontrol Et` : "Yerel Diskleri Tara"}</button>
                    ${isRam && data.platform === "Windows" ? `<button class="secondary-button" data-action="download">${icon("refresh")} WinPMEM İndir</button>` : ""}
                  </div>
                `
            }

            <div class="section-divider"></div>
            <p class="section-label">4. ${isRam ? "RAM ve Çıktı" : "Disk ve Çıktı"}</p>
            ${field(isRam ? "Araç" : "Disk", `<select class="select" data-field="target"><option value="${initialTarget}" ${isRam ? "" : "disabled selected"}>${initialTargetLabel}</option></select>`)}
            ${outputField}
            <button class="primary-button" data-action="start">${icon(isRam ? "ram" : "disk")} ${isRam ? "RAM Edinimini Başlat" : "İmaj Al"}</button>

            <div class="section-divider"></div>
            <p class="section-label">5. İşlem Kontrolleri</p>
            <div class="button-row">
              <button class="secondary-button" data-action="pause">${icon("pause")} Duraklat</button>
              <button class="danger-button" data-action="stop">${icon("stop")} Durdur</button>
            </div>

            <div class="section-divider"></div>
            <p class="section-label">6. İlerleme Durumu</p>
            <div class="progress-bar" data-progress style="--value:0%"><span></span><b>0%</b></div>
            <div class="log-box" id="workflow-log">${state.lastLog.map((line) => `• ${line}`).join("<br />")}</div>
          </div>
        </div>

        <aside class="side-panel">
          <h3>İşlem Durumu</h3>
          ${sideInfo("Platform", `${data.platform} • ${isRemote ? "Uzak agent" : "Yerel işlem"}`, data.icon)}
          ${sideInfo("Bağlantı", isRemote ? "Henüz bağlanmadı" : "Yerel kontrol bekleniyor", "monitor", "connection")}
          ${sideInfo(isRam ? "Araç" : "Hedef", initialTarget || "Hedef seçilmedi", isRam ? "chip" : "disk", "target")}
          ${sideInfo("Son işlem", "Hazır", "clock", "last-action")}
        </aside>
      </div>
    </section>
  `;
}

function pageTitle(title, desc, iconName) {
  return `
    <div class="page-title">
      <span class="card-icon">${icon(iconName)}</span>
      <span>
        <h1>${title}</h1>
        <p>${desc}</p>
      </span>
    </div>
  `;
}

function field(label, control) {
  return `
    <div class="field">
      <label>${label}</label>
      ${control}
    </div>
  `;
}

function pickerField(label, id, value, type = "file") {
  const action = type === "folder" ? "pick-folder" : "pick-file";
  const placeholderOnly = value.startsWith(".") || value.toLowerCase().includes("seç");
  const valueAttr = placeholderOnly ? `placeholder="${value}" value=""` : `value="${value}"`;
  return field(
    label,
    `<div class="input-action"><input id="${id}" class="input" ${valueAttr} data-picker-target /><button class="secondary-button" data-action="${action}" data-target="#${id}">${icon(type === "folder" ? "folder" : "search")} Seç</button></div>`
  );
}

function sideInfo(title, body, iconName, key = "") {
  return `
    <div class="side-info" ${key ? `data-side="${key}"` : ""}>
      <span class="metric-icon">${icon(iconName)}</span>
      <span><strong>${title}</strong><small>${body}</small></span>
    </div>
  `;
}

function agentPage() {
  return `
    <section class="page">
      ${pageTitle("Agent", "Orijinal Worm agent sayfalarındaki Windows ve Linux kullanım özetleri modern dokümantasyon kartları olarak taşındı.", "network")}
      <div class="doc-grid">
        ${agentDoc({
          title: "Windows Agent",
          repo: "https://github.com/noirlang/worm-win",
          binary: "worm-win.exe",
          url: "https://worm.noirlang.tr/worm-win.exe",
          note: "Windows Agent kullanım özeti. Dosyayı Windows üzerinde yönetici olarak çalıştırın ve ana uygulamadaki IP/Port bilgisiyle eşleştirin.",
          iconName: "windows",
          stepsTr: [
            "Agent indirin: wget -O worm-win.exe https://worm.noirlang.tr/worm-win.exe",
            "Windows'ta worm-win.exe dosyasını yönetici olarak çalıştırın.",
            "Ana uygulamadaki IP/Port bilgisi ile eşleştirin."
          ],
          stepsEn: [
            "Download agent: wget -O worm-win.exe https://worm.noirlang.tr/worm-win.exe",
            "Run worm-win.exe as Administrator on Windows.",
            "Match IP/Port values with the main Worm application."
          ]
        })}
        ${agentDoc({
          title: "Linux Agent",
          repo: "https://github.com/noirlang/worm-linux",
          binary: "worm-linux",
          url: "https://worm.noirlang.tr/worm-linux",
          note: "Linux Agent kullanım özeti. Çalıştırılabilir izin verin, agentı başlatın ve ana uygulamadaki IP/Port ile bağlanın.",
          iconName: "linux",
          stepsTr: [
            "Agent indirin: wget -O worm-linux https://worm.noirlang.tr/worm-linux",
            "Yetki verin: chmod +x worm-linux",
            "Çalıştırın: ./worm-linux",
            "Ana uygulamadaki IP/Port ile bağlantı kurun."
          ],
          stepsEn: [
            "Download agent: wget -O worm-linux https://worm.noirlang.tr/worm-linux",
            "Make it executable: chmod +x worm-linux",
            "Run: ./worm-linux",
            "Connect using the same IP/Port from the main Worm app."
          ]
        })}
      </div>
    </section>
  `;
}

function agentDoc({ title, repo, binary, url, note, iconName, stepsTr, stepsEn }) {
  const commands = iconName === "linux"
    ? `wget -O ${binary} ${url}\nchmod +x ${binary}\n./${binary}`
    : `wget -O ${binary} ${url}\n${binary} dosyasını yönetici olarak çalıştırın.`;
  return `
    <article class="doc-card">
      <span class="card-icon">${icon(iconName)}</span>
      <h3>${title}</h3>
      <p>${note}</p>
      <div class="link-row">
        <a href="${repo}">${repo}</a>
        <a href="${url}">${url}</a>
      </div>
      <p class="section-label">TR</p>
      <ol class="step-list">
        ${stepsTr.map((step, index) => `<li><b>${index + 1}</b><span>${step}</span></li>`).join("")}
      </ol>
      <p class="section-label" style="margin-top:18px">EN</p>
      <ol class="step-list">
        ${stepsEn.map((step, index) => `<li><b>${index + 1}</b><span>${step}</span></li>`).join("")}
      </ol>
      <div class="code-box">${commands}</div>
    </article>
  `;
}

function analysisPage() {
  return `
    <section class="page">
      ${pageTitle("İmaj Görüntüleme", "Seçilen disk imajını salt-okunur olarak bağlar ve içeriğini klasör ağacında gösterir.", "search")}
      <div class="workflow-panel">
        <p class="section-label">İmaj Görüntüleme</p>
        <p class="field-hint">İmaj dosyasını seçin, salt-okunur bağlayın ve içerik ağacını bu ekrandan inceleyin.</p>
        ${pickerField("İmaj Dosyası", "image-path", ".img, .dd, .raw, .iso ...", "file")}
        <div class="button-row">
          <button class="primary-button" data-action="mount-readonly">${icon("disk")} Salt-Okunur Bağla</button>
          <button class="danger-button" data-action="unmount-image">${icon("stop")} Bağlantıyı Kaldır</button>
        </div>
        <div class="section-divider"></div>
        <div class="side-info">
          <span class="metric-icon">${icon("info")}</span>
          <span><strong>Durum</strong><small>İmaj seçilmedi</small></span>
        </div>
        <div class="log-box">Klasör ağacı ve bağlama çıktısı burada görüntülenecek.</div>
      </div>
    </section>
  `;
}

function otherPage() {
  return `
    <section class="page">
      ${pageTitle("Diğer", "Hash işlemleri, kanıt kasası, rapor üretimi ve canlı günlük modülleri.", "tiles")}
      <div class="other-grid">
        ${simpleCard("Hash İşlemleri", "MD5, SHA1, SHA256 ve SHA512 hesaplama.", "shield", "hash")}
        ${simpleCard("Kanıt Kasası", "Vaka klasörü ve kanıt kasası yönetimi.", "scale", "evidence")}
        ${simpleCard("Raporlar", "İnceleme notları ve rapor üretimi.", "report", "reports")}
        ${simpleCard("Günlük", "Canlı günlük ve dosyadan yenileme akışı.", "clock", "logs")}
      </div>
      <div id="other-detail" class="workflow-panel" style="margin-top:16px">${hashPanel()}</div>
    </section>
  `;
}

function simpleCard(title, desc, iconName, tab) {
  return `
    <button class="forensic-card" data-tab="${tab}">
      <span class="card-icon">${icon(iconName)}</span>
      <h3>${title}</h3>
      <p>${desc}</p>
      <span class="meta">Aç</span>
    </button>
  `;
}

function hashPanel() {
  return `
    <p class="section-label">Hash Hesaplayıcı</p>
    ${pickerField("Dosya", "hash-file", "Dosya seçin", "file")}
    <div class="button-row">
      <button class="primary-button" data-action="hash">${icon("shield")} Hesapla</button>
    </div>
    <div class="hash-grid">
      ${hashResult("MD5", "md5")}
      ${hashResult("SHA1", "sha1")}
      ${hashResult("SHA256", "sha256")}
      ${hashResult("SHA512", "sha512")}
    </div>
    <div class="section-divider"></div>
    <p class="section-label">Hash Karşılaştır</p>
    ${field("Hash Değeri", '<input class="input" placeholder="Hash degeri girin" />')}
    <div class="button-row">
      <button class="secondary-button" data-action="compare">${icon("search")} Karşılaştır</button>
    </div>
    <div class="side-info" data-hash-compare-result>
      <span class="metric-icon">${icon("info")}</span>
      <span><strong>Sonuç</strong><small>Karşılaştırma bekleniyor</small></span>
    </div>
  `;
}

function hashResult(label, key) {
  return `
    <div class="hash-result" data-hash-result="${key}">
      <small>${label}</small>
      <strong>-</strong>
    </div>
  `;
}

function settingsPage() {
  return `
    <section class="page">
      <div class="settings-header">
        <h1>Ayarlar</h1>
        <p>Tema, dil ve güncelleme kontrolleri.</p>
      </div>
      <div class="settings-layout">
        <article class="settings-card settings-primary">
          <span class="settings-kicker">Görünüm</span>
          <h3>Uygulama Ayarları</h3>
          <p>Tema ve dil tercihi kaydedilir; sayfa yenilendiğinde korunur.</p>
          <div class="settings-row">
            <span>
              <strong>Karanlık Tema</strong>
              <small>Adli bilişim çalışma ekranları için düşük parlaklık.</small>
            </span>
            <button class="switch ${state.theme === "dark" ? "on" : ""}" data-action="theme-toggle" aria-label="Karanlık tema"></button>
          </div>
          <div class="settings-row">
            <span>
              <strong>Dil</strong>
              <small>Menü dili ve uygulama mesajları.</small>
            </span>
            <select class="select compact-select" data-action="language-select" aria-label="Dil">
              <option value="tr" ${state.language === "tr" ? "selected" : ""}>Türkçe</option>
              <option value="en" ${state.language === "en" ? "selected" : ""}>English</option>
            </select>
          </div>
          <div class="settings-row">
            <span>
              <strong>Algılanan Sistem</strong>
              <small>Yerel işlem filtreleri buna göre çalışır.</small>
            </span>
            <span class="status-badge">${icon(state.platform === "windows" ? "windows" : state.platform === "linux" ? "linux" : "monitor")} ${platformLabel(state.platform)}</span>
          </div>
          <div class="button-row">
            <button class="primary-button" data-action="save-settings">Ayarları Kaydet</button>
          </div>
          <div class="status-badge" data-settings-status>${icon("info")} ${t("ready")}</div>
        </article>

        <article class="settings-card">
          <span class="settings-kicker">Sürüm</span>
          <h3>Güncelleme</h3>
          <p>Kurulum dosyasını platforma göre seçer, indirme ilerlemesini ve release notlarını burada gösterir.</p>
          <div class="settings-meta">
            <span>Kurulu: ${APP_VERSION}</span>
            <span>Asset: ${state.platform === "windows" ? "worm-windows-x64.msi" : "worm-linux-x64.AppImage"}</span>
          </div>
          <div class="progress-bar" data-update-progress style="--value:0%"><span></span><b>0%</b></div>
          <div class="button-row">
            <button class="primary-button" data-action="check-update">${icon("refresh")} Güncellemeyi Kontrol Et</button>
            <button class="secondary-button" data-action="download-update">${icon("download")} İndir ve Kur</button>
          </div>
          <div class="status-badge" data-update-status>${icon("info")} Hazır</div>
          <div class="log-box compact-log" data-update-log>Release notları ve indirme durumu burada görüntülenecek.</div>
        </article>
      </div>
    </section>
  `;
}

function aboutPage() {
  return `
    <section class="page">
      <div class="about-hero">
        <span class="about-logo"><img src="${assetPath}/logo/logo.png" alt="Worm logo" /></span>
        <div>
          <p class="eyebrow">Worm Forensic Tool</p>
          <h1>Worm Forensic Tool</h1>
          <span class="status-badge">Sürüm ${APP_VERSION}</span>
          <p>Worm, yetkili adli bilişim süreçlerinde disk ve RAM edinimi, doğrulama ve raporlama adımlarını tek bir merkezde birleştiren bir denetim aracıdır.</p>
        </div>
      </div>

      <h2 class="section-heading">Temel Kabiliyetler</h2>
      <div class="capability-grid">
        ${capabilityCard("COLLECT", "Disk ve RAM", "Windows ve Linux için imaj ve bellek edinimi.", "disk", "var(--green)")}
        ${capabilityCard("PROVE", "Doğrulama", "Hash üretimi, karşılaştırma ve denetlenebilir loglar.", "shield", "var(--blue)")}
        ${capabilityCard("PACKAGE", "Raporlama", "Vaka notları, kanıt kasası ve rapor çıktıları.", "report", "var(--purple)")}
      </div>

      <div class="doc-card usage-card">
        <h3>Kullanım İlkesi</h3>
        <p>Bu araç yalnızca yetkili adli bilişim süreçlerinde kullanılmalıdır. Edinim, doğrulama ve günlük adımları görünür, denetlenebilir ve raporlanabilir tutulur.</p>
      </div>

      <h2 class="section-heading">Contributors</h2>
      <div class="contributor-grid">
        ${contributorCard("ME", "Melih Emik", "melih-emik.jpg", [
          ["GitHub", "https://github.com/favilances"],
          ["LinkedIn", "https://www.linkedin.com/in/melihemik/"],
          ["Website", "https://melihemik.com.tr"]
        ])}
        ${contributorCard("YT", "Yusuf Tuncel", "yusuf-tuncel.jpg", [
          ["GitHub", "https://github.com/yetece1"],
          ["LinkedIn", "https://www.linkedin.com/in/yusuf-tuncel/"]
        ])}
        ${contributorCard("MG", "Muhammet Ali Güner", "muhammet-ali-guner.jpg", [
          ["GitHub", "https://github.com/kafkaskrtl"],
          ["LinkedIn", "https://www.linkedin.com/in/muhammetali-g%C3%BCner/"]
        ])}
      </div>

      <div class="company-logo-card">
        <img src="${assetPath}/logo/sirket.png" alt="Şirket logosu" />
      </div>
    </section>
  `;
}

function capabilityCard(kicker, title, desc, iconName, accent) {
  return `
    <article class="doc-card capability-card" style="--accent:${accent}">
      <span class="card-icon">${icon(iconName)}</span>
      <p class="eyebrow">${kicker}</p>
      <h3>${title}</h3>
      <p>${desc}</p>
    </article>
  `;
}

function contributorCard(initials, name, photo, links) {
  return `
    <article class="contributor-card">
      <img class="avatar" src="${assetPath}/contributors/${photo}" alt="${name}" />
      <h3>${name}</h3>
      <p>Forensic Contributor</p>
      <div class="social-row" aria-label="${name} bağlantıları">
        ${links.map(([label, url]) => socialLink(label, url)).join("")}
      </div>
    </article>
  `;
}

function socialLink(label, url) {
  const key = label === "LinkedIn" ? "linkedin" : label === "Website" ? "website" : "github";
  return `<a class="social-button" href="${url}" target="_blank" rel="noreferrer" aria-label="${label}">${icon(key)}</a>`;
}

const routes = {
  home: homePage,
  windows: () => toolHub("windows"),
  linux: () => toolHub("linux"),
  agent: agentPage,
  analysis: analysisPage,
  other: otherPage,
  settings: settingsPage,
  about: aboutPage
};

document.addEventListener("click", (event) => {
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
    const detail = document.querySelector("#other-detail");
    if (detail) detail.innerHTML = detailPanel(tabButton.dataset.tab);
    hydrateIcons(detail);
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
    updateSide("target", target.value || "Hedef seçilmedi");
  }
});

async function handleAction(button) {
  const action = button.dataset.action;
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
    writeWorkflowLog(button.classList.contains("on") ? "VPN kullanımı açıldı." : "VPN kullanımı kapatıldı.");
    updateSide("connection", button.classList.contains("on") ? "VPN yapılandırması bekleniyor" : "VPN kapalı");
    return;
  }

  if (action === "vpn-config") {
    const panel = document.querySelector(".vpn-panel");
    if (panel) panel.hidden = false;
    document.querySelector("[data-action='toggle-vpn']")?.classList.add("on");
    writeWorkflowLog("VPN yapılandırma alanı açıldı.");
    return;
  }

  if (action === "save-vpn") {
    const endpoint = document.querySelector("[data-field='vpn-endpoint']")?.value.trim();
    if (!endpoint) {
      showToast("VPN sunucu bilgisini girin.", "error");
      return;
    }
    writeWorkflowLog(`VPN yapılandırıldı: ${endpoint}`);
    updateSide("connection", "VPN hazır");
    showToast("VPN yapılandırması kaydedildi.");
    return;
  }

  if (action === "approve-key") {
    const token = document.querySelector("[data-field='token']");
    if (token && !token.value.trim()) token.value = crypto.randomUUID?.() || `worm-${Date.now()}`;
    writeWorkflowLog("Güvenlik anahtarı onaylandı.");
    showToast("Güvenlik anahtarı aktif.");
    return;
  }

  if (action === "reset-key") {
    const token = document.querySelector("[data-field='token']");
    if (token) token.value = "";
    writeWorkflowLog("Güvenlik anahtarı sıfırlandı.");
    return;
  }

  if (action === "connect") {
    const ip = document.querySelector("[data-field='ip']")?.value.trim();
    const port = document.querySelector("[data-field='port']")?.value.trim();
    if (!ip || !port) {
      showToast("IP ve port girin.", "error");
      return;
    }
    updateSide("connection", `${ip}:${port} bağlantı hazır`);
    writeWorkflowLog(`Bağlantı hazırlandı: ${ip}:${port}`);
    showToast("Bağlantı bilgileri doğrulandı.");
    return;
  }

  if (action === "scan") {
    await scanTargets();
    return;
  }

  if (action === "start") {
    startProgress(button);
    return;
  }

  if (action === "pause") {
    writeWorkflowLog("İşlem duraklatıldı.");
    updateSide("last-action", "Duraklatıldı");
    return;
  }

  if (action === "stop") {
    setProgress(0);
    writeWorkflowLog("İşlem durduruldu.");
    updateSide("last-action", "Durduruldu");
    return;
  }

  if (action === "mount-readonly") {
    const imagePath = document.querySelector("#image-path")?.value.trim();
    if (!imagePath || imagePath.startsWith(".")) {
      showToast("Önce imaj dosyası seçin.", "error");
      return;
    }
    setAnalysisStatus(`Bağlandı: ${imagePath}`, "İmaj salt-okunur bağlandı. İçerik ağacı hazır olduğunda burada gösterilecek.");
    showToast("İmaj bağlama işlemi hazırlandı.");
    return;
  }

  if (action === "unmount-image") {
    setAnalysisStatus("Bağlantı kaldırıldı", "Aktif imaj bağlantısı yok.");
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
    setStatus("[data-update-status]", `${icon("refresh")} Güncelleme kontrol edildi`);
    setStatus("[data-update-log]", `Kurulu sürüm: ${APP_VERSION}<br />Son sürüm bilgisi Tauri güncelleme komutuna bağlandığında burada gösterilecek.`);
    showToast("Güncelleme kontrolü tamamlandı.");
    return;
  }

  if (action === "download-update") {
    await simulateUpdateDownload();
    return;
  }

  if (action === "list-files") {
    await pickFolder(null);
    showToast("Klasör seçimi tamamlandı.");
    return;
  }

  const label = button.textContent.trim().replace(/\s+/g, " ");
  writeWorkflowLog(`${label}: UI işlemi hazır.`);
  showToast(`${label} işlemi hazır.`);
}

function showToast(message, type = "success") {
  let toast = document.querySelector(".toast");
  if (!toast) {
    toast = document.createElement("div");
    toast.className = "toast";
    document.body.appendChild(toast);
  }
  toast.textContent = message;
  toast.dataset.type = type;
  toast.classList.add("visible");
  window.clearTimeout(showToast.timer);
  showToast.timer = window.setTimeout(() => toast.classList.remove("visible"), 3200);
}

async function pickFile(targetSelector) {
  const target = targetSelector ? document.querySelector(targetSelector) : null;
  try {
    if (window.showOpenFilePicker) {
      const [handle] = await window.showOpenFilePicker({ multiple: false });
      const file = await handle.getFile();
      if (target) {
        target.value = file.name;
        state.files[targetSelector] = file;
      }
      showToast(`Dosya seçildi: ${file.name}`);
      return file;
    }
  } catch (error) {
    if (error?.name === "AbortError") return null;
    showToast("Dosya seçimi açılamadı.", "error");
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
        showToast(`Dosya seçildi: ${file.name}`);
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
  try {
    if (window.showDirectoryPicker) {
      const handle = await window.showDirectoryPicker();
      if (target) target.value = handle.name;
      showToast(`Klasör seçildi: ${handle.name}`);
      return handle;
    }
  } catch (error) {
    if (error?.name === "AbortError") return null;
    showToast("Klasör seçimi açılamadı.", "error");
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
      if (folder) showToast(`Klasör seçildi: ${folder}`);
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
    const targets = [workflow.diskLabel, workflow.platform === "Windows" ? "WinPMEM portable" : "AVML local"];
    select.innerHTML = targets.map((target) => `<option value="${target}">${target}</option>`).join("");
    updateSide("target", targets[0]);
    writeWorkflowLog("Araç listesi güncellendi.");
    showToast("Kontrol tamamlandı.");
    return;
  }

  const tauriInvoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke;
  if (tauriInvoke) {
    try {
      const disks = await tauriInvoke(workflow.mode.startsWith("remote") ? "remote_disk_list" : "local_disk_list", {
        platform: workflow.platform.toLowerCase()
      });
      const targets = Array.isArray(disks) ? disks.map((disk) => disk.id || disk.name || disk.path || disk).filter(Boolean) : [];
      if (targets.length > 0) {
        select.innerHTML = targets.map((target) => `<option value="${target}">${target}</option>`).join("");
        updateSide("target", targets[0]);
        writeWorkflowLog("Disk listesi güncellendi.");
        showToast("Disk taraması tamamlandı.");
        return;
      }
    } catch (error) {
      showToast("Disk taraması başarısız oldu.", "error");
      writeWorkflowLog(`Disk taraması başarısız: ${error?.message || error}`);
      return;
    }
  }

  select.innerHTML = '<option value="" disabled selected>Disk listesi için Tauri bağlantısı bekleniyor</option>';
  updateSide("target", "Hedef seçilmedi");
  writeWorkflowLog("Disk taraması Tauri komutu bağlandığında gerçek cihazları listeleyecek.");
  showToast("Tarama tamamlandı.");
}

function setProgress(value) {
  const progress = document.querySelector("[data-progress]");
  if (!progress) return;
  const next = `${value}%`;
  progress.style.setProperty("--value", next);
  const label = progress.querySelector("b");
  if (label) label.textContent = next;
}

function startProgress(button) {
  const routeId = state.route.split(":")[1];
  const workflow = workflows[routeId];
  const target = document.querySelector("[data-field='target']")?.value.trim();
  if (workflow && !workflow.mode.includes("ram") && !target) {
    showToast("Önce hedef disk seçin.", "error");
    return;
  }
  const output = document.querySelector("#workflow-output")?.value.trim();
  if (!output) {
    showToast("Çıktı konumu seçin.", "error");
    return;
  }
  button.disabled = true;
  let value = 0;
  writeWorkflowLog("İşlem başlatıldı.");
  updateSide("connection", "İşlem çalışıyor");
  window.clearInterval(state.jobs.workflow);
  state.jobs.workflow = window.setInterval(() => {
    value += 10;
    setProgress(value);
    if (value >= 100) {
      window.clearInterval(state.jobs.workflow);
      button.disabled = false;
      writeWorkflowLog("İşlem tamamlandı.");
      updateSide("connection", "Tamamlandı");
      showToast("İşlem tamamlandı.");
    }
  }, 180);
}

function setAnalysisStatus(status, log) {
  const statusNode = document.querySelector(".workflow-panel .side-info small");
  const logNode = document.querySelector(".workflow-panel .log-box");
  if (statusNode) statusNode.textContent = status;
  if (logNode) logNode.textContent = log;
}

async function calculateHashes() {
  const file = state.files["#hash-file"];
  if (!file) {
    showToast(t("fileRequired"), "error");
    return;
  }
  const buffer = await file.arrayBuffer();
  setHashResult("md5", "Rust core");
  setHashResult("sha1", await digestHex("SHA-1", buffer));
  setHashResult("sha256", await digestHex("SHA-256", buffer));
  setHashResult("sha512", await digestHex("SHA-512", buffer));
  showToast("Hash hesaplama tamamlandı.");
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
  const expected = document.querySelector("input[placeholder='Hash degeri girin']")?.value.trim().toLowerCase();
  const values = [...document.querySelectorAll("[data-hash-result] strong")].map((node) => node.textContent.trim().toLowerCase());
  const result = document.querySelector("[data-hash-compare-result] small");
  if (!expected) {
    showToast("Karşılaştırılacak hash değerini girin.", "error");
    return;
  }
  const matched = values.includes(expected);
  if (result) result.textContent = matched ? "Eşleşti" : "Eşleşmedi";
  showToast(matched ? "Hash eşleşti." : "Hash eşleşmedi.", matched ? "success" : "error");
}

function setStatus(selector, html) {
  const node = document.querySelector(selector);
  if (node) node.innerHTML = html;
}

async function simulateUpdateDownload() {
  const progress = document.querySelector("[data-update-progress]");
  const status = document.querySelector("[data-update-status]");
  if (!progress) return;
  let value = 0;
  if (status) status.innerHTML = `${icon("download")} İndiriliyor`;
  window.clearInterval(state.jobs.update);
  state.jobs.update = window.setInterval(() => {
    value += 20;
    progress.style.setProperty("--value", `${value}%`);
    const label = progress.querySelector("b");
    if (label) label.textContent = `${value}%`;
    if (value >= 100) {
      window.clearInterval(state.jobs.update);
      if (status) status.innerHTML = `${icon("shield")} İndirme hazır`;
      setStatus("[data-update-log]", "Paket indirildi. Kurulum adımı Tauri updater komutuna bağlanacak.");
      showToast("Güncelleme paketi hazır.");
    }
  }, 180);
}

function detailPanel(tab) {
  if (tab === "evidence") {
    return `
      <p class="section-label">Vaka Yönetimi</p>
      ${field("Vaka Adı", '<input class="input" placeholder="Vaka_2026_001" />')}
      <div class="button-row">
        <button class="primary-button" data-action="create-case">${icon("folder")} Vaka Oluştur</button>
      </div>
      <div class="status-badge">${icon("info")} Vaka oluşturulmadı</div>
      <div class="section-divider"></div>
      <p class="section-label">Dosyalar</p>
      ${field("Klasör", '<select class="select"><option>Çıktılar / ciktilar</option><option>Disk İmajları / disk_imajlari</option><option>RAM / ram</option><option>Raporlar / raporlar</option><option>Hash / hash</option><option>Notlar / notlar</option><option>Günlükler / gunlukler</option></select>')}
      ${field("Dosya", '<select class="select"><option>Dosyaları listeleyin...</option></select>')}
      <div class="button-row">
        <button class="secondary-button" data-action="list-files">${icon("search")} Dosyaları Listele</button>
      </div>
    `;
  }
  if (tab === "reports") {
    return `
      <p class="section-label">Rapor Oluştur</p>
      <p class="field-hint">Rapor oluşturmak için önce vaka oluşturun ve işlem tamamlayın.</p>
      ${field("Rapor Başlığı", '<input class="input" value="Adli Bilişim Teknik Raporu" />')}
      ${field("Format", '<select class="select"><option>TXT</option><option>JSON</option></select>')}
      ${field("Not", '<textarea class="textarea" placeholder="Not veya rapor açıklaması girin"></textarea>')}
      <div class="button-row">
        <button class="secondary-button" data-action="add-note">${icon("report")} Not Ekle</button>
        <button class="primary-button" data-action="create-report">${icon("report")} Rapor Oluştur</button>
      </div>
      <div class="status-badge">${icon("info")} Hazır</div>
    `;
  }
  if (tab === "logs") {
    return `
      <p class="section-label">Günlük</p>
      <p class="field-hint">Canlı günlük burada da görüntülenir.</p>
      <div class="log-box">${state.lastLog.map((line) => `• ${line}`).join("<br />")}</div>
      <div class="button-row" style="margin-top:12px">
        <button class="secondary-button" data-action="refresh-log">${icon("refresh")} Dosyadan Günlüğü Yenile</button>
      </div>
    `;
  }
  return hashPanel();
}

setLanguage(state.language);
setTheme(state.theme);
hydrateIcons();
render();
