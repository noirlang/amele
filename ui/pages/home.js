export function homePage({ t, icon, assetPath, theme, state }) {
  const logoFile = "amele.png";
  const defaultFallbackImage = `${assetPath}/logo/${logoFile}`;

  const newsItems = (Array.isArray(state?.news) && state.news.length > 0)
    ? state.news.slice(0, 5)
    : [
        {
          id: "default-1",
          title: "Amele Forensic Platform v0.0.18",
          summary: "Adli bilişim, mobil edinim (Android & iOS) ve bellek analizi platformu hazır.",
          imageUrl: defaultFallbackImage,
          link: "https://amele.noirlang.tr",
        },
      ];

  const activeIndex = Math.min(
    Math.max(0, Number(state?.activeNewsIndex) || 0),
    newsItems.length - 1
  );

  const slidesHtml = newsItems
    .map((item, idx) => {
      const isActive = idx === activeIndex;
      const imgUrl = item.imageUrl || item.image || defaultFallbackImage;
      const title = item.title || "Amele Forensic Platform";
      const summary = item.summary || (item.content ? item.content.slice(0, 160) + "..." : "");
      return `
        <div class="news-slide ${isActive ? "is-active" : ""}" data-index="${idx}">
          <div class="news-media">
            <img src="${imgUrl}" onerror="this.src='${defaultFallbackImage}'" alt="${title}" />
          </div>
          <div class="news-overlay">
            <span class="news-badge">${newsItems.length > 1 ? `HABER ${idx + 1}/${newsItems.length}` : "DUYURU"}</span>
            <h3 class="news-title">${title}</h3>
            <p class="news-summary">${summary}</p>
          </div>
        </div>
      `;
    })
    .join("");

  const dotsHtml = newsItems.length > 1
    ? `<div class="news-dots">
        ${newsItems
          .map(
            (_, idx) =>
              `<button type="button" class="news-dot ${idx === activeIndex ? "is-active" : ""}" data-news-dot="${idx}" aria-label="Haber ${idx + 1}"></button>`
          )
          .join("")}
      </div>`
    : "";

  const navButtonsHtml = newsItems.length > 1
    ? `
        <button type="button" class="news-nav-btn prev" data-news-action="prev" aria-label="Önceki haber">‹</button>
        <button type="button" class="news-nav-btn next" data-news-action="next" aria-label="Sonraki haber">›</button>
      `
    : "";

  return `
    <section class="page">
      <div class="hero home-hero">
        <div class="news-carousel-card" id="news-carousel" data-total-slides="${newsItems.length}">
          <div class="news-slides-container">
            ${slidesHtml}
          </div>
          ${navButtonsHtml}
          ${dotsHtml}
        </div>
      </div>

      <div class="home-grid">
        ${homeTile(t("home.windows.title"), t("home.windows.desc"), "windows", "windows", "var(--text)", icon, state)}
        ${homeTile(t("home.linux.title"), t("home.linux.desc"), "linux", "linux", "var(--text)", icon, state)}
        ${homeTile(t("home.docker.title"), t("home.docker.desc"), "docker", "docker", "var(--text)", icon, state)}
        ${homeTile(t("home.android.title"), t("home.android.desc"), "android", "android", "var(--text)", icon, state)}
        ${homeTile(t("home.ios.title"), t("home.ios.desc"), "ios", "ios", "var(--text)", icon, state)}
        ${homeTile(t("home.agent.title"), t("home.agent.desc"), "network", "agent", "var(--text)", icon, state)}
        ${homeTile(t("home.remote.title"), t("home.remote.desc"), "key", "remote-acq", "var(--text)", icon, state)}
        ${homeTile(t("home.other.title"), t("home.other.desc"), "tiles", "other", "var(--text)", icon, state)}
      </div>
    </section>
  `;
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
