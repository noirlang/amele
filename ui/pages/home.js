export function homePage({ t, icon, assetPath }) {
  return `
    <section class="page">
      <div class="hero home-hero">
        <div class="worm-art">
          <img src="${assetPath}/logo/logo.png" alt="Worm logo" />
        </div>
      </div>

      <div class="home-grid">
        ${homeTile(t("home.acquire.title"), t("home.acquire.desc"), "ACQUIRE", "disk", "windows", "var(--green)", icon)}
        ${homeTile(t("home.integrity.title"), t("home.integrity.desc"), "VERIFY", "shield", "other", "var(--green)", icon)}
        ${homeTile(t("home.evidence.title"), t("home.evidence.desc"), "CASE", "scale", "other", "var(--purple)", icon)}
        ${homeTile(t("home.output.title"), t("home.output.desc"), "REPORT", "report", "other", "var(--blue)", icon)}
      </div>
    </section>
  `;
}

function homeTile(title, desc, label, iconName, route, accent, icon) {
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

export function metric(label, value, iconName, accent, icon) {
  return `
    <div class="metric" style="--accent:${accent}">
      <span class="metric-icon">${icon(iconName)}</span>
      <span><small>${label}</small><strong>${value}</strong></span>
    </div>
  `;
}
