export function analysisPage({ t, icon, state, pageTitle, pickerField }) {
  const activeTab = state.activeAnalysisTab || "image";
  return `
    <section class="page">
      ${pageTitle(t("analysis.title"), t("analysis.desc"), "search")}
      
      <div class="analysis-tabs">
        <button class="analysis-tab-btn ${activeTab === "image" ? "active" : ""}" data-analysis-tab="image">
          ${icon("disk")} ${t("analysis.tabImage")}
        </button>
        <button class="analysis-tab-btn ${activeTab === "ram" ? "active" : ""}" data-analysis-tab="ram">
          ${icon("shield")} ${t("analysis.tabRam")}
        </button>
      </div>

      <div id="analysis-content">
        ${activeTab === "image" ? renderImageAnalysis({ t, icon, state, pickerField }) : renderRamAnalysis({ t, icon, state, pickerField })}
      </div>
    </section>
  `;
}

function renderImageAnalysis({ t, icon, state, pickerField }) {
  const activeMount = state.imageMount?.mountDir || t("analysis.noImage");
  const defaultPath = state.imagePathInput || ".img, .dd, .raw, .iso ...";
  return `
    <div class="workflow-panel">
      <p class="section-label">${t("analysis.tabImage")}</p>
      <p class="field-hint">${t("analysis.hint")}</p>
      ${pickerField(t("analysis.imageFile"), "image-path", defaultPath, "file")}
      <div class="button-row">
        <button class="primary-button" data-action="mount-readonly">${icon("disk")} ${t("analysis.mount")}</button>
        <button class="danger-button" data-action="unmount-image">${icon("stop")} ${t("analysis.unmount")}</button>
      </div>
      <div class="section-divider"></div>
      <div class="side-info">
        <span class="metric-icon">${icon("info")}</span>
        <span><strong>${t("analysis.status")}</strong><small data-analysis-status>${activeMount}</small></span>
      </div>
      
      <div class="forensic-split">
        <div class="tree-panel">
          <div class="panel-header">
            <h3>📁 Klasör Yapısı / Directory Tree</h3>
          </div>
          <div class="file-tree-container" id="image-tree-root">
            ${state.imageMountTreeHTML || `<div class="log-box">${t("analysis.outputWaiting")}</div>`}
          </div>
        </div>
        <div class="preview-panel">
          <div class="panel-header">
            <h3>📄 Dosya Önizleme / File Preview</h3>
          </div>
          <div class="file-tree-container" id="image-file-preview" style="background: rgba(0,0,0,0.12)">
            <div class="log-box" style="display:flex;align-items:center;justify-content:center;color:var(--muted);text-align:center;padding:20px">
              Klasör yapısında bir dosyaya tıklayarak içeriğini inceleyebilirsiniz.<br/>Click a file on the left to preview it.
            </div>
          </div>
        </div>
      </div>
    </div>
  `;
}

function renderRamAnalysis({ t, icon, state, pickerField }) {
  const defaultPath = state.ramAnalysisPathInput || ".bin, .raw, .tar ...";
  return `
    <div class="workflow-panel">
      <p class="section-label">${t("analysis.tabRam")}</p>
      <p class="field-hint">${t("analysis.ramHint")}</p>
      ${pickerField(t("analysis.ramFile"), "ram-analysis-path", defaultPath, "file")}
      
      <div class="button-row" style="margin-top:14px">
        <button class="primary-button" data-action="ram-strings">${icon("shield")} ${t("analysis.btnStrings")}</button>
        <button class="primary-button" data-action="ram-carver">${icon("disk")} ${t("analysis.btnCarver")}</button>
        <button class="primary-button" data-action="ram-processes">${icon("menu")} ${t("analysis.btnProcess")}</button>
      </div>

      <div class="section-divider"></div>

      <div id="ram-analysis-results" class="ram-dashboard" style="display:none">
        <!-- Stats Row -->
        <div class="ram-stats">
          <div class="ram-stat-card">
            <span class="card-icon">${icon("shield")}</span>
            <div class="stat-info">
              <strong id="stat-strings-count">0</strong>
              <small>Bulgu Dizgileri / IOCs</small>
            </div>
          </div>
          <div class="ram-stat-card">
            <span class="card-icon">${icon("disk")}</span>
            <div class="stat-info">
              <strong id="stat-carved-count">0</strong>
              <small>Kurtarılan Dosya / Carved</small>
            </div>
          </div>
          <div class="ram-stat-card">
            <span class="card-icon">${icon("menu")}</span>
            <div class="stat-info">
              <strong id="stat-procs-count">0</strong>
              <small>Aktif Proses / PIDs</small>
            </div>
          </div>
          <div class="ram-stat-card">
            <span class="card-icon">${icon("info")}</span>
            <div class="stat-info">
              <strong id="stat-status-lbl">Hazır / Ready</strong>
              <small>Durum / Status</small>
            </div>
          </div>
        </div>

        <div class="forensic-split" id="ram-split-view" style="display:none">
          <!-- Left list -->
          <div class="tree-panel" id="ram-left-panel">
            <div class="panel-header">
              <h3 id="ram-left-panel-title">${t("analysis.lblProcesses")}</h3>
            </div>
            <div class="file-tree-container" id="ram-left-list" style="padding:0">
              <!-- Dynamic processes or carved files -->
            </div>
          </div>
          <!-- Right panel -->
          <div class="preview-panel" id="ram-right-panel">
            <div class="panel-header">
              <h3 id="ram-right-panel-title">${t("analysis.lblMaps")}</h3>
            </div>
            <div class="file-tree-container" id="ram-right-content">
              <!-- Dynamic details/maps -->
            </div>
          </div>
        </div>

        <!-- Flat Result Panel -->
        <div class="workflow-panel" id="ram-flat-results-panel" style="display:none;padding:16px;margin-top:14px">
          <p class="section-label" id="ram-flat-title">${t("analysis.lblStrings")}</p>
          <div class="strings-results-list" id="ram-flat-results-list">
            <!-- Matches list -->
          </div>
        </div>
      </div>
    </div>
  `;
}
