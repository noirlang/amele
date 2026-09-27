/**
 * Sağ kenarda dikey listelenen hata ve öneri raporlama paneli.
 * amele.noirlang.tr web sitesindeki yeni rapor formu ile birebir uyumlu.
 * Tüm sayfalarda sabit ve sol vaka paneli ile tam simetrik olarak yerleşir.
 */
export function renderReportSidebar(state, t, icon, esc) {
  const online = state?.activeProfile?.online;
  const isOnline = Boolean(
    online &&
    online.username &&
    online.status !== "offline" &&
    online.status !== "session_expired" &&
    (typeof navigator === "undefined" || navigator.onLine !== false)
  );

  const draft = state?.reportDraft || {
    title: "",
    description: "",
    images: [],
    submitting: false,
    error: "",
    success: ""
  };

  const images = Array.isArray(draft.images) ? draft.images : [];
  const submitting = Boolean(draft.submitting);
  const errorMsg = draft.error || "";
  const successMsg = draft.success || "";
  const titleVal = draft.title || "";
  const descVal = draft.description || "";

  const isCollapsed = Boolean(state?.reportSidebarCollapsed);

  return `
    <!-- Mobil ve Dar Ekranlar İçin Sağ Tetikleyici Buton -->
    <button
      type="button"
      class="home-report-sidebar-toggle${isCollapsed ? " is-visible" : ""}"
      data-action="toggle-report-sidebar"
      title="${t("onlineReport.sidebarTitle") || "Rapor Bildir"}"
      aria-label="Rapor Menüsü"
    >
      <span class="toggle-ico">${icon("bug")}</span>
      <span class="toggle-txt">${t("onlineReport.sidebarTitle") || "Rapor"}</span>
    </button>

    <div class="home-report-sidebar-backdrop" data-action="close-report-sidebar"></div>

    <aside class="home-report-sidebar${isCollapsed ? " is-collapsed" : ""}" id="home-report-sidebar">
      <div class="report-sidebar-header">
        <div class="report-sidebar-header-left">
          <span class="report-sidebar-header-icon">${icon("bug")}</span>
          <span class="report-sidebar-header-title">${t("onlineReport.sidebarTitle") || "Rapor Bildir"}</span>
        </div>
        <div class="report-sidebar-header-right">
          <button
            type="button"
            class="report-sidebar-close-btn"
            data-action="close-report-sidebar"
            title="Kapat"
          >
            <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
              <line x1="18" y1="6" x2="6" y2="18"></line>
              <line x1="6" y1="6" x2="18" y2="18"></line>
            </svg>
          </button>
        </div>
      </div>

      <div class="report-sidebar-body">
        ${!isOnline ? `
          <!-- Kilit Kartı: Hesap girilmediğinde doldurma kesinlikle engellenir -->
          <div class="report-locked-card">
            <span class="report-locked-icon">${icon("lock")}</span>
            <h4 class="report-locked-title">${t("onlineReport.lockedTitle") || "Profil Bağlantısı Gerekli"}</h4>
            <p class="report-locked-desc">${t("onlineReport.lockedDesc") || "Hata bildirmek veya geri bildirim iletmek için profilinizi bağlayın."}</p>
            <button type="button" class="report-locked-connect-btn" data-action="profile-online-start">
              ${icon("user")}
              <span>${t("onlineReport.connectBtn") || "Profili Bağla"}</span>
            </button>
          </div>

          <!-- Kilitli Form: Üzerine gelince veya tıklanınca asla doldurulamaz -->
          <form class="report-sidebar-form is-locked" onsubmit="return false;" aria-disabled="true">
            <div class="report-sidebar-field">
              <label class="report-sidebar-label">${t("onlineReport.titleLabel") || "Başlık"}</label>
              <input
                class="report-sidebar-input"
                disabled
                readonly
                tabindex="-1"
                placeholder="${t("onlineReport.titlePlaceholder") || "Hata veya öneri başlığı..."}"
              />
            </div>
            <div class="report-sidebar-field">
              <label class="report-sidebar-label">${t("onlineReport.descLabel") || "Açıklama"}</label>
              <textarea
                class="report-sidebar-textarea"
                disabled
                readonly
                tabindex="-1"
                rows="3"
                placeholder="${t("onlineReport.descPlaceholder") || "Karşılaştığınız hatayı veya talebinizi detaylandırın..."}"
              ></textarea>
            </div>
            <div class="report-sidebar-field">
              <label class="report-sidebar-label">${t("onlineReport.screenshotLabel") || "Ekran Görüntüsü"}</label>
              <div class="report-sidebar-dropzone" style="cursor: not-allowed; opacity: 0.55;">
                <span class="report-dropzone-ico">${icon("image")}</span>
                <p class="report-dropzone-text">${t("onlineReport.dropzoneHint") || "Görsel yüklemek için tıklayın veya sürükleyin"}</p>
              </div>
            </div>
            <button type="button" class="report-sidebar-submit-btn" disabled tabindex="-1">
              ${icon("bug")}
              <span>${t("onlineReport.submit") || "Rapor Gönder"}</span>
            </button>
          </form>
        ` : `
          <!-- Çevrimiçi Aktif Form: Hesap bağlıysa serbestçe doldurulup gönderilebilir -->
          <form class="report-sidebar-form" onsubmit="event.preventDefault();">
            <div class="report-sidebar-field">
              <label class="report-sidebar-label" for="report-title-input">
                <span>${t("onlineReport.titleLabel") || "Başlık"}</span>
                <span style="font-size:10px; color:#737373;">${titleVal.length}/200</span>
              </label>
              <input
                id="report-title-input"
                class="report-sidebar-input"
                type="text"
                maxlength="200"
                placeholder="${t("onlineReport.titlePlaceholder") || "Hata veya öneri başlığı..."}"
                value="${esc(titleVal)}"
                ${submitting ? "disabled" : ""}
                required
              />
            </div>

            <div class="report-sidebar-field">
              <label class="report-sidebar-label" for="report-desc-input">
                <span>${t("onlineReport.descLabel") || "Açıklama"}</span>
                <span style="font-size:10px; color:#737373;">${descVal.length}/5000</span>
              </label>
              <textarea
                id="report-desc-input"
                class="report-sidebar-textarea"
                rows="3"
                maxlength="5000"
                placeholder="${t("onlineReport.descPlaceholder") || "Karşılaştığınız hatayı veya talebinizi detaylandırın..."}"
                ${submitting ? "disabled" : ""}
                required
              >${esc(descVal)}</textarea>
            </div>

            <div class="report-sidebar-field">
              <label class="report-sidebar-label">
                <span>${t("onlineReport.screenshotLabel") || "Ekran Görüntüsü"}</span>
                ${images.length > 0 ? `<span style="font-size:10px; color:var(--text, #ffffff);">${images.length}/5</span>` : ""}
              </label>
              <div
                class="report-sidebar-dropzone"
                id="report-sidebar-dropzone"
                data-action="report-pick-image"
                role="button"
                tabindex="0"
                title="Görsel seçmek için tıklayın"
              >
                <span class="report-dropzone-ico">${icon("image")}</span>
                <p class="report-dropzone-text">${t("onlineReport.dropzoneHint") || "Görsel yüklemek için tıklayın veya sürükleyin"}</p>
                <span class="report-dropzone-sub">${t("onlineReport.dropzoneSub") || "PNG, JPG, WEBP (Maks 10MB)"}</span>
                <input
                  type="file"
                  id="report-file-input"
                  accept="image/png,image/jpeg,image/webp,image/gif"
                  hidden
                />
              </div>

              ${images.length > 0 ? `
                <div class="report-preview-list">
                  ${images.map((img, idx) => `
                    <div class="report-preview-item" title="${esc(img.name || "Görsel")}">
                      <img src="${img.dataUrl}" alt="Preview" />
                      <button
                        type="button"
                        class="report-preview-remove-btn"
                        data-action="report-remove-image"
                        data-index="${idx}"
                        title="${t("onlineReport.removeScreenshot") || "Görseli kaldır"}"
                        aria-label="Kaldır"
                      >×</button>
                    </div>
                  `).join("")}
                </div>
              ` : ""}
            </div>

            ${errorMsg ? `
              <div class="report-feedback-msg error">
                ${esc(errorMsg)}
              </div>
            ` : ""}

            ${successMsg ? `
              <div class="report-feedback-msg success">
                ${esc(successMsg)}
              </div>
            ` : ""}

            <button
              type="button"
              class="report-sidebar-submit-btn"
              data-action="submit-report"
              ${submitting || !titleVal.trim() || !descVal.trim() ? "disabled" : ""}
            >
              ${submitting ? `
                <svg viewBox="0 0 24 24" width="14" height="14" stroke="currentColor" fill="none" style="animation: spin 1s linear infinite;">
                  <circle cx="12" cy="12" r="10" stroke-width="3" stroke-dasharray="32" stroke-linecap="round"></circle>
                </svg>
                <span>${t("onlineReport.submitting") || "Gönderiliyor..."}</span>
              ` : `
                ${icon("bug")}
                <span>${t("onlineReport.submit") || "Rapor Gönder"}</span>
              `}
            </button>
          </form>
        `}
      </div>
    </aside>
  `;
}
