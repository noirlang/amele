// Dairesel saçılan gezinti menüsü (Radial Wheel Navigation).
// amele.noirlang.tr arayüzünün 360 derecelik dairesel buton çarkını birebir amele-next'e taşır.

export function renderRadialNav(state, t, icon, escapeHtml, getAvatarUrl) {
  const isOpen = Boolean(state.navMenu?.isOpen);
  const isClosing = Boolean(state.navMenu?.isClosing);
  const activeSubmenu = state.navMenu?.activeSubmenu || null;

  let list = [];

  if (activeSubmenu === "tools") {
    // Adli Araçlar Alt Çarkı (Tools Sub-Wheel)
    list = [
      {
        id: "tools-hub",
        label: t("nav.allTools") || "Tüm Araçlar",
        tooltip: t("nav.allTools") || "Tüm Araçlar Paneli",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "tools" ? " is-active" : ""}"
            data-route="tools"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.allTools") || "Tüm Araçlar")}"
          >
            ${icon("tiles")}
          </button>
        `
      },
      {
        id: "windows",
        label: t("nav.windows") || "Windows Araçları",
        tooltip: t("nav.windows") || "Windows Araçları",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "windows" ? " is-active" : ""}"
            data-route="windows"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.windows") || "Windows Araçları")}"
          >
            ${icon("windows")}
          </button>
        `
      },
      {
        id: "linux",
        label: t("nav.linux") || "Linux Araçları",
        tooltip: t("nav.linux") || "Linux Araçları",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "linux" ? " is-active" : ""}"
            data-route="linux"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.linux") || "Linux Araçları")}"
          >
            ${icon("linux")}
          </button>
        `
      },
      {
        id: "docker",
        label: t("nav.docker") || "Docker Araçları",
        tooltip: t("nav.docker") || "Docker Araçları",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "docker" ? " is-active" : ""}"
            data-route="docker"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.docker") || "Docker Araçları")}"
          >
            ${icon("docker")}
          </button>
        `
      },
      {
        id: "android",
        label: t("nav.android") || "Android Araçları",
        tooltip: t("nav.android") || "Android Araçları",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route.startsWith("android") ? " is-active" : ""}"
            data-route="android"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.android") || "Android Araçları")}"
          >
            ${icon("android")}
          </button>
        `
      },
      {
        id: "ios",
        label: t("nav.ios") || "iOS Araçları",
        tooltip: t("nav.ios") || "iOS Araçları",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "ios" ? " is-active" : ""}"
            data-route="ios"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.ios") || "iOS Araçları")}"
          >
            ${icon("ios")}
          </button>
        `
      },
      {
        id: "remote-acq",
        label: t("nav.remoteAcq") || "Uzak / Canlı Edinim",
        tooltip: t("nav.remoteAcq") || "Uzak / Canlı Edinim",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "remote-acq" ? " is-active" : ""}"
            data-route="remote-acq"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.remoteAcq") || "Uzak / Canlı Edinim")}"
          >
            ${icon("globe")}
          </button>
        `
      },
      {
        id: "back",
        label: t("nav.back") || "Geri Dön",
        tooltip: t("nav.back") || "Ana Menüye Dön",
        content: `
          <button
            type="button"
            class="nav-circle-btn"
            data-nav-action="back-to-main"
            aria-label="${escapeHtml(t("nav.back") || "Geri Dön")}"
          >
            ${icon("arrowLeft")}
          </button>
        `
      }
    ];
  } else {
    // 11 Ana Menü Butonu (Website ve ekran görüntüsüyle birebir aynı sıra ve ikonlar)
    const profile = state.activeProfile;
    const username = profile?.username || profile?.display_name || profile?.full_name || "melihemik";
    const avatarUrl = typeof getAvatarUrl === "function" ? getAvatarUrl(profile) : "";
    const profileTooltip = `@${username} · ${t("profile.title") || "Profil"}`;
    const isEn = state.language === "en";
    const flagSrc = isEn ? "./assets/flags/tr.svg" : "./assets/flags/gb.svg";
    const flagAlt = isEn ? "Türkçe" : "English";

    list = [
      {
        id: "home",
        label: t("nav.home") || "Ana Sayfa",
        tooltip: t("nav.home") || "Ana Sayfa",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "home" ? " is-active" : ""}"
            data-route="home"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.home") || "Ana Sayfa")}"
          >
            ${icon("home")}
          </button>
        `
      },
      {
        id: "tools",
        label: t("nav.tools") || "Araçlar",
        tooltip: t("nav.tools") || "Adli Araçlar",
        content: `
          <button
            type="button"
            class="nav-circle-btn${["tools", "windows", "linux", "docker", "android", "ios", "remote-acq"].includes(state.route) ? " is-active" : ""}"
            data-nav-action="open-tools-sub"
            aria-label="${escapeHtml(t("nav.tools") || "Araçlar")}"
          >
            ${icon("tools")}
          </button>
        `
      },
      {
        id: "developers",
        label: t("nav.developers") || "Geliştirici",
        tooltip: t("nav.developers") || "Geliştirici & Konsol",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "about" ? " is-active" : ""}"
            data-route="about"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.developers") || "Geliştirici")}"
          >
            ${icon("code")}
          </button>
        `
      },
      {
        id: "download",
        label: t("nav.download") || "Deliller",
        tooltip: t("nav.download") || "Deliller & İndirmeler",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "other" && state.activeTab === "evidence" ? " is-active" : ""}"
            data-route="other"
            data-tab="evidence"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.download") || "Deliller")}"
          >
            ${icon("download")}
          </button>
        `
      },
      {
        id: "notes",
        label: t("nav.notes") || "Raporlar",
        tooltip: t("nav.notes") || "Vaka & Raporlar",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "other" && state.activeTab === "reports" ? " is-active" : ""}"
            data-route="other"
            data-tab="reports"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.notes") || "Raporlar")}"
          >
            ${icon("fileText")}
          </button>
        `
      },
      {
        id: "news",
        label: t("nav.news") || "Haberler",
        tooltip: t("nav.news") || "Haberler & Duyurular",
        content: `
          <button
            type="button"
            class="nav-circle-btn"
            data-nav-action="scroll-to-news"
            aria-label="${escapeHtml(t("nav.news") || "Haberler")}"
          >
            ${icon("bell")}
          </button>
        `
      },
      {
        id: "report",
        label: t("nav.report") || "Hata Bildir",
        tooltip: t("nav.report") || "Hata Bildir & Triyaj",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "help" ? " is-active" : ""}"
            data-route="help"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.report") || "Hata Bildir")}"
          >
            ${icon("bug")}
          </button>
        `
      },
      {
        id: "language",
        label: t("nav.language") || "Dil",
        tooltip: flagAlt,
        content: `
          <button
            type="button"
            class="nav-circle-btn nav-circle-flag-btn"
            data-nav-action="toggle-language"
            aria-label="${escapeHtml(flagAlt)}"
          >
            <img src="${flagSrc}" alt="${escapeHtml(flagAlt)}" class="nav-circle-flag" />
          </button>
        `
      },
      {
        id: "profile",
        label: t("profile.title") || "Profil",
        tooltip: profileTooltip,
        content: `
          <button
            type="button"
            class="nav-circle-btn nav-circle-profile${state.route === "profile" ? " is-active" : ""}"
            data-route="profile"
            data-nav-action="close-menu"
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
        id: "shield",
        label: t("nav.settings") || "Ayarlar",
        tooltip: t("nav.settings") || "Ayarlar & Bütünlük",
        content: `
          <button
            type="button"
            class="nav-circle-btn${state.route === "settings" ? " is-active" : ""}"
            data-route="settings"
            data-nav-action="close-menu"
            aria-label="${escapeHtml(t("nav.settings") || "Ayarlar")}"
          >
            ${icon("shield")}
          </button>
        `
      },
      {
        id: "logout",
        label: t("nav.logout") || "Çıkış",
        tooltip: t("nav.logout") || "Profili Değiştir / Çıkış",
        content: `
          <button
            type="button"
            class="nav-circle-btn nav-circle-logout"
            data-nav-action="logout"
            aria-label="${escapeHtml(t("nav.logout") || "Çıkış")}"
          >
            ${icon("logout")}
          </button>
        `
      }
    ];
  }

  const total = list.length;
  const items = list.map((item, index) => {
    const angle = -Math.PI / 2 + (index * 2 * Math.PI) / total;
    const cos = Number(Math.cos(angle).toFixed(5));
    const sin = Number(Math.sin(angle).toFixed(5));

    let placement = "top";
    if (sin < -0.5) {
      placement = "top";
    } else if (sin > 0.5) {
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

  return `
    ${isOpen ? `<div class="nav-backdrop${isClosing ? " is-closing" : ""}" data-nav-action="close-menu" aria-label="Kapat"></div>` : ""}
    <nav class="nav centered-nav${isOpen && !isClosing ? " is-open" : ""}" aria-label="Main navigation">
      <div class="nav-center-island">
        <button
          type="button"
          class="nav-center-trigger${isOpen && !isClosing ? " is-open" : ""}"
          data-nav-action="toggle-menu"
          aria-label="${isOpen && !isClosing ? "Menüyü Kapat" : "Menüyü Aç"}"
          aria-expanded="${isOpen && !isClosing}"
        >
          <img src="./assets/logo/logo.webp" alt="Amele" class="nav-center-logo" draggable="false" />
          <span class="nav-trigger-chevron" aria-hidden="true">
            <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <polyline points="6 9 12 15 18 9" />
            </svg>
          </span>
        </button>

        ${
          isOpen
            ? `
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
                  <span class="nav-circle-tooltip placement-${item.placement}">${escapeHtml(item.tooltip)}</span>
                </div>
              </div>
            `
              )
              .join("")}
          </div>
        `
            : ""
        }
      </div>
    </nav>
  `;
}
