// Adli Bilişim Araçları Merkezi (Forensic Tools Hub).
// Tüm işletim sistemleri ve platformlar için adli veri edinimi araçlarını tek bir merkezde toplar.

export function toolsPage({ t, icon, pageTitle, escapeHtml, state }) {
  const isEn = state?.language === "en";
  const detectedPlatform = state?.platform || "linux";

  const tools = [
    {
      id: "windows",
      title: isEn ? "Windows Forensic Tools" : "Windows Adli Araçları",
      desc: isEn
        ? "PhysicalDrive disk imaging, WinPMEM memory dump, and automated forensic triage collection."
        : "PhysicalDrive disk imajı, WinPMEM uçucu bellek dökümü ve otomatik adli triyaj veri edinimi.",
      route: "windows",
      icon: "windows",
      badge: isEn ? "RAM · Disk · Triage" : "RAM · Disk · Triyaj",
      accent: "var(--text)"
    },
    {
      id: "linux",
      title: isEn ? "Linux Forensic Tools" : "Linux Adli Araçları",
      desc: isEn
        ? "AVML / LiME raw volatile memory acquisition, block device dd/raw imaging, and live system logs."
        : "AVML / LiME ile uçucu bellek edinimi, blok cihaz disk imajı ve canlı sistem logları toplama.",
      route: "linux",
      icon: "linux",
      badge: isEn ? "AVML · BLKGETSIZE · Live" : "AVML · BLKGETSIZE · Canlı",
      accent: "var(--text)"
    },
    {
      id: "docker",
      title: isEn ? "Docker & Container Forensics" : "Docker & Konteyner Adli Bilişimi",
      desc: isEn
        ? "Container drift (Overlay2 diff), environment secrets, container logs, and security risk triage."
        : "Konteyner drift (Overlay2 diff), ortam değişkenleri parola taraması, loglar ve güvenlik analizi.",
      route: "docker",
      icon: "docker",
      badge: isEn ? "Overlay2 · Drift · Secrets" : "Overlay2 · Drift · Secret",
      accent: "var(--text)"
    },
    {
      id: "android",
      title: isEn ? "Android Mobile Forensics" : "Android Adli Bilişimi",
      desc: isEn
        ? "ADB daemon connection, logical APK & database backup, filesystem extraction, and volatile memory."
        : "ADB bağlantısı, mantıksal APK & veritabanı yedeği, dosya sistemi edinimi ve uçucu bellek.",
      route: "android",
      icon: "android",
      badge: isEn ? "ADB · Logical · Physical" : "ADB · Mantıksal · Fiziksel",
      accent: "var(--text)"
    },
    {
      id: "ios",
      title: isEn ? "iOS Mobile Forensics" : "iOS Adli Bilişimi",
      desc: isEn
        ? "Encrypted/unencrypted iTunes backup analysis, keychain, SMS, WhatsApp databases, and media extraction."
        : "Şifreli/şifresiz iTunes yedekleme analizi, keychain, SMS, WhatsApp ve medya delil çıkarımı.",
      route: "ios",
      icon: "ios",
      badge: isEn ? "Backup · Keychain · DB" : "Yedekleme · Keychain · DB",
      accent: "var(--text)"
    },
    {
      id: "remote-acq",
      title: isEn ? "Remote & Agentless Acquisition" : "Uzak & Agent'sız Canlı Edinim",
      desc: isEn
        ? "SSH / WinRM network pipe stream without installing agents or writing to target disk/RAM."
        : "Hedef sisteme ajan kurmadan veya disk/RAM'e yazmadan SSH / WinRM pipe akışıyla edinim.",
      route: "remote-acq",
      icon: "globe",
      badge: isEn ? "SSH · WinRM · Agentless" : "SSH · WinRM · Agentless",
      accent: "var(--text)"
    }
  ];

  return `
    <section class="page page-tools">
      <div class="platform-note">
        ${icon(detectedPlatform === "windows" ? "windows" : detectedPlatform === "linux" ? "linux" : "monitor")}
        ${t("hub.detected", { platform: `<strong>${escapeHtml(detectedPlatform.toUpperCase())}</strong>` })}
      </div>

      ${pageTitle(
        t("tools.title") || (isEn ? "Forensic Analysis Tools" : "Adli Bilişim Araçları"),
        t("tools.desc") || (isEn ? "Platform-based memory, disk, mobile, and container acquisition and analysis modules." : "Platform bazlı bellek (RAM), disk imajı, mobil ve konteyner veri edinimi ve analiz modülleri."),
        "tools",
        icon
      )}

      <div class="tool-grid">
        ${tools
          .map(
            (card, index) => `
          <button class="forensic-card" data-route="${card.route}" data-shortcut="${index + 1}" style="--accent:${card.accent}">
            <span class="card-icon">${icon(card.icon)}</span>
            <span class="shortcut-key-badge" aria-hidden="true">${index + 1}</span>
            <h3>${escapeHtml(card.title)}</h3>
            <p>${escapeHtml(card.desc)}</p>
            <span class="meta">${escapeHtml(card.badge)}</span>
          </button>
        `
          )
          .join("")}
      </div>

      <div class="workflow-panel" style="margin-top: 24px;">
        <div class="panel-head">
          <div>
            <h3>${isEn ? "Quick Forensic Utilities" : "Hızlı Adli Yardımcılar"}</h3>
            <p>${isEn ? "Integrated evidence hashing, report generator, and acquisition history." : "Entegre delil hash hesaplama, vaka raporlama ve edinim geçmişi yönetimi."}</p>
          </div>
        </div>
        <div class="button-row" style="margin-top: 14px; display: flex; gap: 10px; flex-wrap: wrap;">
          <button type="button" class="secondary-button" data-route="other" data-tab="hash">
            ${icon("key")} ${isEn ? "Hash Calculator" : "Hash Hesaplayıcı (SHA-256)"}
          </button>
          <button type="button" class="secondary-button" data-route="other" data-tab="evidence">
            ${icon("download")} ${isEn ? "Evidence Store" : "Delil Deposu & İndirmeler"}
          </button>
          <button type="button" class="secondary-button" data-route="other" data-tab="reports">
            ${icon("fileText")} ${isEn ? "Case Reports" : "Vaka Raporları & Notlar"}
          </button>
        </div>
      </div>
    </section>
  `;
}
