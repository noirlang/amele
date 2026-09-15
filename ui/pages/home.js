// ana dashboard ekranı. sistem durumu, haberler ve hızlı edinim butonları burda.

export function homePage({ t, icon, assetPath, theme, state }) {
  const logoFile = "amele.png";
  const defaultFallbackImage = `${assetPath}/logo/${logoFile}`;

  const hasNews = Array.isArray(state?.news) && state.news.length > 0;
  const newsItems = hasNews
    ? state.news.slice(0, 5)
    : [
        {
          id: "default",
          imageUrl: defaultFallbackImage,
        },
      ];

  const activeIndex = Math.min(
    Math.max(0, Number(state?.activeNewsIndex) || 0),
    newsItems.length - 1
  );

  const isEn = state?.language === "en";
  const slidesHtml = newsItems
    .map((item, idx) => {
      const isActive = idx === activeIndex;
      let rawImg = item.imageUrl || item.image;
      if (rawImg && typeof rawImg === "string") {
        rawImg = rawImg.trim();
        if (rawImg.startsWith("/")) {
          rawImg = `https://amele.noirlang.tr${rawImg}`;
        }
      }
      const imgUrl = rawImg || defaultFallbackImage;
      const title = isEn
        ? (item.titleEn || item.title || item.titleTr || "")
        : (item.titleTr || item.title || item.titleEn || "");
      const summary = isEn
        ? (item.summaryEn || item.summary || item.summaryTr || (item.contentEn || item.content ? (item.contentEn || item.content).slice(0, 160) + "..." : ""))
        : (item.summaryTr || item.summary || item.summaryEn || (item.contentTr || item.content ? (item.contentTr || item.content).slice(0, 160) + "..." : ""));
      const hasText = Boolean(title || summary);

      let targetUrl = "";
      if (item.link && typeof item.link === "string" && item.link.trim()) {
        targetUrl = item.link.trim();
      } else if (item.url && typeof item.url === "string" && item.url.trim()) {
        targetUrl = item.url.trim();
      } else if (item.slug && typeof item.slug === "string" && item.slug.trim()) {
        targetUrl = `https://amele.noirlang.tr/news/${encodeURIComponent(item.slug.trim())}`;
      } else if (item.id && item.id !== "default" && typeof item.id === "string" && item.id.trim()) {
        targetUrl = `https://amele.noirlang.tr/news/${encodeURIComponent(item.id.trim())}`;
      } else if (hasNews) {
        targetUrl = "https://amele.noirlang.tr/news";
      } else {
        targetUrl = "https://amele.noirlang.tr";
      }
      if (targetUrl.startsWith("/")) {
        targetUrl = `https://amele.noirlang.tr${targetUrl}`;
      }
      targetUrl = targetUrl.replace("/announcements/", "/news/");
      if (targetUrl.endsWith("/announcements")) {
        targetUrl = targetUrl.replace(/\/announcements$/, "/news");
      }

      const openTooltip = isEn ? "Open announcement in browser" : "Duyuruyu tarayıcıda aç";
      const openAria = isEn ? "Open announcement" : "Duyuruyu aç";

      const linkBtnHtml = targetUrl
        ? ` <a href="${targetUrl}" data-news-link="${targetUrl}" class="news-link-btn" target="_blank" rel="noopener noreferrer" title="${openTooltip}" aria-label="${openAria}">${icon("external-link")}</a>`
        : "";

      return `
        <div class="news-slide ${isActive ? "is-active" : ""}" data-index="${idx}">
          <div class="news-media">
            <img src="${imgUrl}" onerror="this.src='${defaultFallbackImage}'" alt="${title || "Amele"}" />
          </div>
          ${
            hasText
              ? `
            <div class="news-overlay">
              ${title ? `<h3 class="news-title"><span>${title}</span>${linkBtnHtml}</h3>` : ""}
              ${summary ? `<p class="news-summary">${summary}</p>` : ""}
            </div>
          `
              : ""
          }
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
      <div class="hero home-hero news-hero-container" id="news-carousel" data-total-slides="${newsItems.length}">
        <div class="news-slides-container">
          ${slidesHtml}
        </div>
        ${navButtonsHtml}
        ${dotsHtml}
      </div>

      <div class="home-grid">
        ${homeTile(t("home.windows.title"), t("home.windows.desc"), "windows", "windows", "var(--text)", icon, state)}
        ${homeTile(t("home.linux.title"), t("home.linux.desc"), "linux", "linux", "var(--text)", icon, state)}
        ${homeTile(t("home.docker.title"), t("home.docker.desc"), "docker", "docker", "var(--text)", icon, state)}
        ${homeTile(t("home.android.title"), t("home.android.desc"), "android", "android", "var(--text)", icon, state)}
        ${homeTile(t("home.ios.title"), t("home.ios.desc"), "ios", "ios", "var(--text)", icon, state)}
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
