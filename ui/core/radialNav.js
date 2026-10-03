// Dairesel saçılan gezinti menüsü (Radial Wheel Navigation).
// Sol menü ve sağ üstteki 12 ana uygulama modülü (Home, Windows, Linux, Docker, Android, iOS, Diğer, Yardım, Hakkında, Ayarlar, Profil, Çıkış)
// 360 derecelik dairesel çarkta toplanmıştır (Dil ayarı Ayarlar içinde yer aldığından çarktan çıkarılmıştır).

export function buildRadialNavItems(state, t, icon, escapeHtml, getAvatarUrl) {
  const profile = state.activeProfile;
  const username = profile?.username || profile?.display_name || profile?.full_name || (state.language === "en" ? "guest" : "misafir");
  const avatarUrl = typeof getAvatarUrl === "function" ? getAvatarUrl(profile) : "";
  const profileTooltip = `@${username} · ${t("profile.title") || "Profil"}`;
  const mobileAllowed = Boolean(state.mobileToolsAccess?.allowed);

  // 12 Uygulama Menü Elemanı:
  const list = [
    {
      id: "home",
      shortcut: "H",
      label: t("nav.home") || "Ana Sayfa",
      tooltip: t("nav.home") || "Ana Sayfa",
      isActive: state.route === "home",
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "home" ? " is-active" : ""}"
          data-route="home"
          data-shortcut="H"
          aria-label="${escapeHtml(t("nav.home") || "Ana Sayfa")}"
        >
          ${icon("home")}
        </button>
      `
    },
    {
      id: "windows",
      shortcut: "W",
      label: t("nav.windows") || "Windows Araçları",
      tooltip: t("nav.windows") || "Windows Araçları",
      isActive: state.route === "windows" || state.route.startsWith("workflow:windows"),
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "windows" || state.route.startsWith("workflow:windows") ? " is-active" : ""}"
          data-route="windows"
          data-shortcut="W"
          aria-label="${escapeHtml(t("nav.windows") || "Windows Araçları")}"
        >
          ${icon("windows")}
        </button>
      `
    },
    {
      id: "linux",
      shortcut: "L",
      label: t("nav.linux") || "Linux Araçları",
      tooltip: t("nav.linux") || "Linux Araçları",
      isActive: state.route === "linux" || state.route.startsWith("workflow:linux"),
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "linux" || state.route.startsWith("workflow:linux") ? " is-active" : ""}"
          data-route="linux"
          data-shortcut="L"
          aria-label="${escapeHtml(t("nav.linux") || "Linux Araçları")}"
        >
          ${icon("linux")}
        </button>
      `
    },
    {
      id: "docker",
      shortcut: "D",
      label: t("nav.docker") || "Docker Araçları",
      tooltip: t("nav.docker") || "Docker Araçları",
      isActive: state.route === "docker",
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "docker" ? " is-active" : ""}"
          data-route="docker"
          data-shortcut="D"
          aria-label="${escapeHtml(t("nav.docker") || "Docker Araçları")}"
        >
          ${icon("docker")}
        </button>
      `
    },
    {
      id: "android",
      shortcut: "A",
      label: t("nav.android") || "Android Araçları",
      tooltip: t("nav.android") || "Android Araçları",
      isActive: state.route === "android" || state.route.startsWith("android:"),
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "android" || state.route.startsWith("android:") ? " is-active" : ""}${!mobileAllowed ? " is-locked" : ""}"
          data-route="android"
          data-shortcut="A"
          aria-label="${escapeHtml(t("nav.android") || "Android Araçları")}"
        >
          ${icon("android")}
        </button>
      `
    },
    {
      id: "ios",
      shortcut: "I",
      label: t("nav.ios") || "iOS Araçları",
      tooltip: t("nav.ios") || "iOS Araçları",
      isActive: state.route === "ios",
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "ios" ? " is-active" : ""}${!mobileAllowed ? " is-locked" : ""}"
          data-route="ios"
          data-shortcut="I"
          aria-label="${escapeHtml(t("nav.ios") || "iOS Araçları")}"
        >
          ${icon("ios")}
        </button>
      `
    },
    {
      id: "other",
      shortcut: "O",
      label: t("nav.other") || "Diğer",
      tooltip: t("nav.otherTooltip") || "Diğer (Delil, Rapor, Hash)",
      isActive: state.route === "other",
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "other" ? " is-active" : ""}"
          data-route="other"
          data-shortcut="O"
          aria-label="${escapeHtml(t("nav.other") || "Diğer")}"
        >
          ${icon("tiles")}
        </button>
      `
    },
    {
      id: "help",
      shortcut: "Y",
      label: t("top.help") || "Yardım",
      tooltip: t("help.title") || "Yardım & Dokümantasyon",
      isActive: state.route === "help",
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "help" ? " is-active" : ""}"
          data-route="help"
          data-shortcut="Y"
          aria-label="${escapeHtml(t("top.help") || "Yardım")}"
        >
          ${icon("help")}
        </button>
      `
    },
    {
      id: "about",
      shortcut: "B",
      label: t("about.title") || "Hakkında",
      tooltip: t("about.tooltip") || t("about.title") || "Hakkında",
      isActive: state.route === "about",
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "about" ? " is-active" : ""}"
          data-route="about"
          data-shortcut="B"
          aria-label="${escapeHtml(t("about.title") || "Hakkında")}"
        >
          ${icon("info")}
        </button>
      `
    },
    {
      id: "settings",
      shortcut: "S",
      label: t("settings.title") || "Ayarlar",
      tooltip: t("settings.tooltip") || t("settings.title") || "Ayarlar",
      isActive: state.route === "settings",
      content: `
        <button
          type="button"
          class="nav-circle-btn${state.route === "settings" ? " is-active" : ""}"
          data-route="settings"
          data-shortcut="S"
          aria-label="${escapeHtml(t("settings.title") || "Ayarlar")}"
        >
          ${icon("settings")}
        </button>
      `
    },
    {
      id: "profile",
      shortcut: "P",
      label: t("profile.title") || "Profil",
      tooltip: profileTooltip,
      isActive: state.route === "profile",
      content: `
        <button
          type="button"
          class="nav-circle-btn nav-circle-profile${state.route === "profile" ? " is-active" : ""}"
          data-route="profile"
          data-shortcut="P"
          aria-label="${escapeHtml(username)}"
        >
          ${
            avatarUrl
              ? `<img src="${escapeHtml(avatarUrl)}" alt="${escapeHtml(username)}" class="nav-circle-avatar" />`
              : icon("user")
          }
        </button>
      `
    },
    {
      id: "logout",
      shortcut: "Q",
      label: t("nav.logout") || "Çıkış",
      tooltip: t("nav.logoutTooltip") || "Profili Değiştir / Çıkış",
      content: `
        <button
          type="button"
          class="nav-circle-btn nav-circle-logout"
          data-nav-action="logout"
          data-shortcut="Q"
          aria-label="${escapeHtml(t("nav.logout") || "Çıkış")}"
        >
          ${icon("logout")}
        </button>
      `
    }
  ];

  const total = list.length;
  const items = list.map((item, index) => {
    const angle = -Math.PI / 2 + (index * 2 * Math.PI) / total;
    const cos = Number(Math.cos(angle).toFixed(5));
    const sin = Number(Math.sin(angle).toFixed(5));

    let placement = "top";
    if (sin < -0.55) {
      placement = "top";
    } else if (sin > 0.55) {
      placement = "bottom";
    } else if (cos > 0) {
      placement = "right";
    } else {
      placement = "left";
    }

    return {
      ...item,
      cos,
      sin,
      placement,
      index
    };
  });

  return { items, total };
}

export function renderRadialWheelHtml(state, t, icon, escapeHtml, getAvatarUrl) {
  const isClosing = Boolean(state.navMenu?.isClosing);
  const { items, total } = buildRadialNavItems(state, t, icon, escapeHtml, getAvatarUrl);

  return `
    <div class="nav-radial-wheel${isClosing ? " is-closing" : ""}">
      ${items
        .map(
          (item) => `
        <div
          class="nav-radial-item"
          style="--cos: ${item.cos}; --sin: ${item.sin}; --btn-index: ${item.index}; --total-btns: ${total};"
        >
          <div class="nav-circle-btn-wrap">
            ${item.content}
            ${item.shortcut ? `<span class="shortcut-key-badge" aria-hidden="true">${item.shortcut}</span>` : ""}
            <span class="nav-circle-tooltip placement-${item.placement}">${escapeHtml(item.tooltip)}</span>
          </div>
        </div>
      `
        )
        .join("")}
    </div>
  `;
}

export function renderRadialNav(state, t, icon, escapeHtml, getAvatarUrl) {
  const isOpen = Boolean(state.navMenu?.isOpen);
  const isClosing = Boolean(state.navMenu?.isClosing);
  const isLight = state.theme === "light";
  const logoSrc = isLight ? "./assets/logo/logo-siyah.png" : "./assets/logo/logo.webp";

  return `
    ${isOpen ? `<div class="nav-backdrop${isClosing ? " is-closing" : ""}" data-nav-action="close-menu" aria-label="Kapat"></div>` : ""}
    <nav class="nav centered-nav${isOpen && !isClosing ? " is-open" : ""}${isClosing ? " is-closing" : ""}" aria-label="Main navigation">
      <div class="nav-center-island">
        <button
          type="button"
          class="nav-center-trigger${isOpen && !isClosing ? " is-open" : ""}${isClosing ? " is-closing" : ""}"
          data-nav-action="toggle-menu"
          data-shortcut="M"
          aria-label="${isOpen && !isClosing ? (t("nav.closeMenu") || "Menüyü Kapat") : (t("nav.openMenu") || "Menüyü Aç")}"
          aria-expanded="${isOpen && !isClosing}"
        >
          <img src="${logoSrc}" alt="Amele" class="nav-center-logo" draggable="false" />
          <span class="nav-trigger-chevron" aria-hidden="true">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </span>
          <span class="shortcut-key-badge" aria-hidden="true">M</span>
        </button>

        ${isOpen ? renderRadialWheelHtml(state, t, icon, escapeHtml, getAvatarUrl) : ""}
      </div>
    </nav>
  `;
}
