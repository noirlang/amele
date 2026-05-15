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

const state = {
  route: "home",
  theme: localStorage.getItem("worm-theme") || "dark",
  lastLog: [
    "Rust teknik çekirdek yüklendi.",
    "Agent protokolü: JSON-over-TCP uyumlu.",
    "UI bağlantıları Tauri komutlarına bağlanmak için hazırlandı."
  ]
};

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
    diskLabel: "PhysicalDrive0"
  },
  "linux-remote-disk": {
    platform: "Linux",
    icon: "linux",
    title: "Uzak Linux Disk Bağlantısı",
    desc: "Linux agent ile uzak /dev disklerini listeleyin ve raw imaj alın.",
    mode: "remote-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: "/dev/sda"
  },
  "windows-local-disk": {
    platform: "Windows",
    icon: "windows",
    title: "Windows Yerel Disk İmajı",
    desc: "Yerel PhysicalDrive kaynaklarından ham imaj alma akışı.",
    mode: "local-disk",
    output: "C:\\Worm\\Ciktilar",
    diskLabel: "\\\\.\\PhysicalDrive0"
  },
  "linux-local-disk": {
    platform: "Linux",
    icon: "linux",
    title: "Linux Yerel Disk İmajı",
    desc: "Yerel Linux blok cihazlarından imaj alma akışı.",
    mode: "local-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: "/dev/sda"
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
  state.route = route;
  render();
}

function setTheme(theme) {
  state.theme = theme;
  localStorage.setItem("worm-theme", theme);
  app.classList.toggle("theme-light", theme === "light");
  app.classList.toggle("theme-dark", theme !== "light");
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
      <div class="hero">
        <div>
          <p class="eyebrow">Digital Forensics Core</p>
          <h1>Worm</h1>
          <p>Disk, RAM, hash, kanıt kasası ve rapor akışını tek ekranda toparlayan modern adli bilişim merkezi.</p>
          <div class="hero-actions">
            <button class="primary-button" data-route="windows">Hemen Başla ${icon("arrow")}</button>
          </div>
        </div>
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

      <div class="status-strip">
        ${metric("Sistem Durumu", "Hazır", "monitor", "var(--green)")}
        ${metric("Disk Alanı", "1.24 TB boş / 1.82 TB", "database", "var(--purple)")}
        ${metric("RAM", "15.6 GB / 31.9 GB", "chip", "var(--green)")}
        ${metric("Son Oturum", "Bugün 14:32", "clock", "var(--blue)")}
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
      (card) => `
        <button class="forensic-card" data-route="workflow:${card.id}" style="--accent:${card.accent}">
          <span class="card-icon">${icon(card.icon)}</span>
          <h3>${card.title}</h3>
          <p>${card.desc}</p>
          <span class="meta">${card.badge}</span>
        </button>
      `
    )
    .join("");

  const isWindows = platform === "windows";
  return `
    <section class="page">
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

function workflowPage(id) {
  const data = workflows[id] || workflows["windows-remote-disk"];
  const isRemote = data.mode.startsWith("remote");
  const isRam = data.mode.includes("ram");
  const toolCheck = data.platform === "Windows" ? "WinPMEM" : "AVML";

  return `
    <section class="page">
      <div class="workflow-layout">
        <div class="workflow-panel">
          ${pageTitle(data.title, data.desc, data.icon)}
          <div class="form-grid">
            ${
              isRemote
                ? `
                  ${field("IP Adresi", '<input class="input" value="192.168.1.100" />')}
                  ${field("Port", '<input class="input" value="4444" />')}
                  ${field("Token", '<input class="input" placeholder="Güvenlik anahtarı (Onayla ile aktif olur)" />')}
                  <div class="button-row">
                    <button class="secondary-button" data-action="approve-key">${icon("key")} Anahtarı Onayla</button>
                    <button class="secondary-button" data-action="reset-key">${icon("refresh")} Sıfırla</button>
                  </div>
                  <div class="section-divider"></div>
                  <p class="section-label">2. Ağ ve VPN</p>
                  <div class="toggle-row">
                    <span>WireGuard VPN Kullan</span>
                    <button class="switch" data-action="toggle"></button>
                  </div>
                  <button class="secondary-button" data-action="vpn-config">${icon("settings")} VPN Yapılandır</button>
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
            ${field(isRam ? "Araç" : "Disk", `<select class="select"><option>${data.diskLabel}</option><option>Tarama sonrası seçenekler burada listelenir</option></select>`)}
            ${field(isRam ? "Çıktı Dosyası" : "Çıktı Klasörü", `<input class="input" value="${data.output}" />`)}
            <button class="primary-button" data-action="start">${icon(isRam ? "ram" : "disk")} ${isRam ? "RAM Edinimini Başlat" : "İmaj Al"}</button>

            <div class="section-divider"></div>
            <p class="section-label">5. İşlem Kontrolleri</p>
            <div class="button-row">
              <button class="secondary-button" data-action="pause">${icon("pause")} Duraklat</button>
              <button class="danger-button" data-action="stop">${icon("stop")} Durdur</button>
            </div>

            <div class="section-divider"></div>
            <p class="section-label">6. İlerleme Durumu</p>
            <div class="progress-bar" style="--value:0%"><span></span><b>0%</b></div>
            <div class="log-box" id="workflow-log">${state.lastLog.map((line) => `• ${line}`).join("<br />")}</div>
          </div>
        </div>

        <aside class="side-panel">
          <h3>Durum Özeti</h3>
          ${sideInfo("Bağlantı Durumu", isRemote ? "Bağlantı bekleniyor" : "Yerel kontrol bekleniyor", "monitor")}
          ${sideInfo(isRam ? "Araç Bilgisi" : "Seçili Disk", `${data.diskLabel}<br />Kullanılabilir: -`, isRam ? "chip" : "disk")}
          ${sideInfo("Tahmini Çıktı", "Tahmini boyut: -<br />Kullanılabilir alan: -", "database")}
          ${sideInfo("Son İşlem", "Tarih: -<br />Süre: -", "clock")}
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

function sideInfo(title, body, iconName) {
  return `
    <div class="side-info">
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
      <div class="doc-card" style="margin-top:16px">
        <h3>Güvenlik Anahtarı Davranışı</h3>
        <p>Agent tarafında anahtar açıksa istemci <code>guvenlik_anahtar_b64</code> gönderir. İstemci anahtar gönderip agent tarafında anahtar yoksa bağlantı fail-closed şekilde reddedilir.</p>
        <div class="code-box">{"komut":"merhaba","istemci":"worm","surum":"0.1","guvenlik_anahtar_b64":"..."}</div>
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
        ${field("İmaj Dosyası", '<input class="input" placeholder=".img, .dd, .raw, .iso ..." />')}
        <div class="button-row">
          <button class="secondary-button" data-action="open-image">${icon("folder")} Dosya Seç</button>
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
    ${field("Dosya", '<input class="input" placeholder="/path/to/image.raw" />')}
    <div class="button-row">
      <button class="primary-button" data-action="hash">${icon("shield")} Hesapla</button>
    </div>
    <div class="hash-grid">
      ${hashResult("MD5")}
      ${hashResult("SHA1")}
      ${hashResult("SHA256")}
      ${hashResult("SHA512")}
    </div>
    <div class="section-divider"></div>
    <p class="section-label">Hash Karşılaştır</p>
    ${field("Hash Değeri", '<input class="input" placeholder="Hash degeri girin" />')}
    <div class="button-row">
      <button class="secondary-button" data-action="compare">${icon("search")} Karşılaştır</button>
    </div>
    <div class="side-info">
      <span class="metric-icon">${icon("info")}</span>
      <span><strong>Sonuç</strong><small>Karşılaştırma bekleniyor</small></span>
    </div>
  `;
}

function hashResult(label) {
  return `
    <div class="hash-result">
      <small>${label}</small>
      <strong>-</strong>
    </div>
  `;
}

function settingsPage() {
  return `
    <section class="page">
      ${pageTitle("Ayarlar", "Uygulama teması, dil seçimi ve güncelleme yönetimi.", "settings")}
      <div class="settings-grid">
        <div class="settings-card">
          <h3>Uygulama Ayarları</h3>
          <div class="toggle-row">
            <span>Karanlık Tema</span>
            <button class="switch ${state.theme === "dark" ? "on" : ""}" data-action="theme-toggle"></button>
          </div>
          ${field("Dil", '<select class="select"><option>Türkçe</option><option>English</option></select>')}
          <button class="primary-button" data-action="save-settings">Ayarları Kaydet</button>
          <div class="status-badge">${icon("info")} Hazır</div>
        </div>
        <div class="settings-card">
          <h3>Güncelleme</h3>
          <p>Orijinal güncelleme sayfası Ayarlar içine taşındı; kontrol, indirme, kurulum ve release notları burada yönetilir.</p>
          <div class="settings-meta">
            <span>Kurulu: ${APP_VERSION}</span>
            <span>Asset: worm-windows-x64.msi / worm-linux-x64.AppImage</span>
          </div>
          <div class="progress-bar" style="--value:0%"><span></span><b>0%</b></div>
          <div class="button-row">
            <button class="primary-button" data-action="check-update">${icon("refresh")} Güncellemeyi Kontrol Et</button>
            <button class="secondary-button" data-action="download-update">${icon("download")} İndir ve Kur</button>
          </div>
          <div class="status-badge">${icon("info")} Hazır</div>
          <div class="log-box">Release notları ve indirme durumu burada görüntülenecek.</div>
        </div>
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

function handleAction(button) {
  if (button.dataset.action === "theme-toggle") {
    setTheme(state.theme === "dark" ? "light" : "dark");
    render();
    return;
  }

  if (button.dataset.action === "toggle") {
    button.classList.toggle("on");
    return;
  }

  const label = button.textContent.trim().replace(/\s+/g, " ");
  state.lastLog.unshift(`${label}: Tauri komut bağlantısı için hazır.`);
  state.lastLog = state.lastLog.slice(0, 6);
  const log = document.querySelector("#workflow-log");
  if (log) log.innerHTML = state.lastLog.map((line) => `• ${line}`).join("<br />");
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

setTheme(state.theme);
hydrateIcons();
render();
