export function androidPage({ t, icon, pageTitle, state, escapeHtml, backendReady }) {
  return `
    <section class="page">
      ${pageTitle(t("hub.android.title"), t("hub.android.desc"), "android")}
      <div class="tool-grid android-mode-grid">
        ${androidImageModeCard("physical", t("android.mode.physical.title"), t("android.mode.physical.desc"), "disk", "var(--text)", t("android.mode.soon"), icon, escapeHtml, { disabled: true })}
        ${androidImageModeCard("logical", t("android.mode.logical.title"), t("android.mode.logical.desc"), "android", "var(--text)", t("android.mode.logical.badge"), icon, escapeHtml)}
        ${androidImageModeCard("filesystem", t("android.mode.filesystem.title"), t("android.mode.filesystem.desc"), "folder", "var(--text)", t("android.mode.filesystem.badge"), icon, escapeHtml)}
        ${androidImageModeCard("ram", t("android.mode.ram.title"), t("android.mode.ram.desc"), "cpu", "var(--text)", t("android.mode.ram.badge"), icon, escapeHtml)}
      </div>
    </section>
  `;
}

export function androidModePage({ modeId, t, icon, pageTitle, state, escapeHtml, backendReady, casePanel, field }) {
  const mode = androidMode(modeId, t);
  const android = state.android || {};
  const status = android.adbStatus || null;
  const installed = Boolean(status?.installed);
  const devices = Array.isArray(android.devices) ? android.devices : [];
  const selected = android.selectedDevice || devices[0]?.serial || "";
  const statusTitle = status
    ? installed ? t("android.adb.installed") : t("android.adb.missing")
    : t("android.adb.unknown");
  const statusDetail = status?.message || (backendReady() ? t("android.adb.checkHint") : t("android.appModeRequired"));
  const deviceProfile = android.deviceProfile || null;
  const acquisitionProfile = android.logicalProfile || "full_logical";

  const isLogical = modeId === "logical";
  const isFilesystem = modeId === "filesystem";
  const isRam = modeId === "ram";
  const job = isLogical 
    ? (android.logicalJob || null) 
    : (isFilesystem ? (android.filesystemJob || null) : (android.ramJob || null));
  const isRunning = job?.status === "running";
  const isDone = job?.status === "completed";
  const isFailed = job?.status === "failed";
  const progressValue = job && job.total > 0 ? Math.round((job.done / job.total) * 100) : 0;

  return `
    <section class="page">
      <button class="secondary-button android-back-button" data-route="android">${icon("grid")} ${t("android.back")}</button>
      ${pageTitle(mode.title, mode.desc, mode.icon)}
      <div class="workflow-layout">
        <div class="workflow-panel">
          <p class="section-label">${t("android.adb.title")}</p>
          <div class="side-info">
            <span class="metric-icon">${icon(installed ? "shield" : "android")}</span>
            <span>
              <strong>${statusTitle}</strong>
              <small>${escapeHtml(statusDetail)}</small>
            </span>
          </div>
          <div class="button-row" style="margin-top:12px">
            <button class="primary-button" data-action="android-adb-check">${icon("android")} ${t("android.adb.check")}</button>
            ${installed ? `<button class="secondary-button" data-action="android-list-devices">${icon("refresh")} ${t("android.devices.list")}</button>` : ""}
          </div>

          <div class="section-divider"></div>
          <p class="section-label">${t("android.devices.title")}</p>
          <div class="field">
            <label>${t("android.devices.select")}</label>
            <select class="select" data-android-device-select ${devices.length ? "" : "disabled"}>
              ${deviceOptions(devices, selected, t, escapeHtml)}
            </select>
          </div>
          <div class="button-row" style="margin-top:12px">
            <button class="secondary-button" data-action="android-profile-fetch" ${selected ? "" : "disabled"}>${icon("search")} ${t("android.profile.fetch")}</button>
          </div>
          ${deviceProfile ? `
            <div class="log-box" style="margin-top:12px">
              ${deviceProfileSummary(deviceProfile, t, escapeHtml)}
            </div>
          ` : ""}

          ${isLogical ? `
            <div class="section-divider"></div>
            <p class="section-label">${t("android.logical.caseTitle")}</p>
            ${casePanel("android", t("android.logical.caseHint"))}

            <div class="section-divider"></div>
            <p class="section-label">${t("android.acquisition.profileTitle")}</p>
            <div class="field">
              <label>${t("android.acquisition.profile")}</label>
              <select class="select" data-android-acquisition-profile>
                ${acquisitionProfileOptions(acquisitionProfile, t)}
              </select>
            </div>

            <div class="section-divider"></div>
            <p class="section-label">${t("android.logical.acquisitionTitle")}</p>
            <div class="button-row">
              <button class="primary-button" data-action="android-start-logical" ${isRunning ? "disabled" : ""}>${icon("android")} ${t("android.logical.start")}</button>
              ${isRunning ? `<button class="danger-button" data-action="android-stop-logical">${icon("stop")} ${t("android.logical.stop")}</button>` : ""}
            </div>

            <div class="section-divider"></div>
            <p class="section-label">${t("android.logical.progress")}</p>
            <div class="progress-bar" data-progress style="--value:${progressValue}%"><span></span><b>${progressValue}%</b></div>
            <div class="log-box" id="android-log">${androidLogContent(android, t, modeId)}</div>
          ` : ""}

          ${isFilesystem ? `
            <div class="section-divider"></div>
            <p class="section-label">${t("android.filesystem.caseTitle") || "Vaka Notları"}</p>
            ${casePanel("android", t("android.filesystem.caseHint") || "Dosya sistemi imajı için vaka detayı belirtin.")}

            <div class="section-divider"></div>
            <p class="section-label">${t("android.filesystem.options") || "Seçenekler"}</p>
            <div class="field" style="flex-direction: row; align-items: center; gap: 10px;">
              <input type="checkbox" id="android-filesystem-has-root" data-android-filesystem-has-root style="width: 18px; height: 18px; cursor: pointer;" />
              <label for="android-filesystem-has-root" style="cursor: pointer; user-select: none; font-size: 0.9rem; color: #acc0e4;">${t("android.filesystem.hasRoot") || "Cihazda Root Yetkisi Var (Doğrudan imaj al)"}</label>
            </div>

            <div class="section-divider"></div>
            <p class="section-label">${t("android.filesystem.acquisitionTitle") || "Aktarım"}</p>
            <div class="button-row">
              <button class="primary-button" data-action="android-start-filesystem" ${isRunning ? "disabled" : ""}>${icon("folder")} ${t("android.filesystem.start") || "Dosya Sistem İmajını Al"}</button>
              ${isRunning ? `<button class="danger-button" data-action="android-stop-filesystem">${icon("stop")} ${t("android.logical.stop")}</button>` : ""}
            </div>

            <div class="section-divider"></div>
            <p class="section-label">${t("android.logical.progress")}</p>
            <div class="progress-bar" data-progress style="--value:${progressValue}%"><span></span><b>${progressValue}%</b></div>
            <div class="log-box" id="android-log">${androidLogContent(android, t, modeId)}</div>
          ` : ""}

          ${isRam ? `
            <div class="section-divider"></div>
            <p class="section-label">${t("android.ram.caseTitle") || "Vaka Notları"}</p>
            ${casePanel("android", t("android.ram.caseHint") || "RAM imajı için vaka detayı belirtin.")}

            <div class="section-divider"></div>
            <p class="section-label">${t("android.ram.options") || "Seçenekler"}</p>
            <div class="field">
              <label>${t("android.ram.mode")}</label>
              <select class="select" data-android-ram-mode>
                ${ramModeOptions(android.ramMode || "volatile_data", t)}
              </select>
            </div>
            <div class="field" style="flex-direction: row; align-items: center; gap: 10px;">
              <input type="checkbox" id="android-ram-has-root" data-android-ram-has-root style="width: 18px; height: 18px; cursor: pointer;" />
              <label for="android-ram-has-root" style="cursor: pointer; user-select: none; font-size: 0.9rem; color: #acc0e4;">${t("android.filesystem.hasRoot") || "Cihazda Root Yetkisi Var (Doğrudan imaj al)"}</label>
            </div>

            <div class="section-divider"></div>
            <p class="section-label">${t("android.ram.acquisitionTitle") || "Aktarım"}</p>
            <div class="button-row">
              <button class="primary-button" data-action="android-start-ram" ${isRunning ? "disabled" : ""}>${icon("cpu")} ${t("android.ram.start") || "RAM İmajını Al"}</button>
              ${isRunning ? `<button class="danger-button" data-action="android-stop-ram">${icon("stop")} ${t("android.logical.stop")}</button>` : ""}
            </div>

            <div class="section-divider"></div>
            <p class="section-label">${t("android.logical.progress")}</p>
            <div class="progress-bar" data-progress style="--value:${progressValue}%"><span></span><b>${progressValue}%</b></div>
            <div class="log-box" id="android-log">${androidLogContent(android, t, modeId)}</div>
          ` : ""}
        </div>

        <aside class="side-panel">
          <h3>${t("android.side.status")}</h3>
          ${sideInfo(t("android.side.adb"), statusTitle, installed ? "shield" : "android", icon)}
          ${sideInfo(t("android.side.device"), selected || t("android.devices.none"), "android", icon)}
          ${deviceProfile ? sideInfo(t("android.side.profile"), sideProfileSummary(deviceProfile, t), "info", icon) : ""}
          ${(isLogical || isFilesystem || isRam) && job ? sideInfo(t("android.side.lastAction"), job.message || "—", "clock", icon) : ""}
          ${(isLogical || isFilesystem || isRam) && isDone && job.result ? sideInfo(t("android.side.totalBytes"), formatBytes(job.result.total_bytes || 0), "disk", icon) : ""}
        </aside>
      </div>
    </section>
  `;
}

function acquisitionProfileOptions(selected, t) {
  const options = [
    ["quick_logical", t("android.acquisition.quick")],
    ["full_logical", t("android.acquisition.full")],
    ["root_logical", t("android.acquisition.root")],
    ["volatile", t("android.acquisition.volatile")]
  ];
  return options
    .map(([value, label]) => `<option value="${value}"${value === selected ? " selected" : ""}>${label}</option>`)
    .join("");
}

function deviceProfileSummary(profile, t, escapeHtml) {
  const rows = [
    [t("android.profile.model"), profile.model || profile.device || "—"],
    [t("android.profile.api"), profile.api_level || "—"],
    [t("android.profile.selinux"), profile.selinux || "—"],
    [t("android.profile.encryption"), profile.encryption || "—"],
    [t("android.profile.root"), profile.is_rooted ? t("android.profile.rooted") : t("android.profile.notRooted")]
  ];
  return rows
    .map(([label, value]) => `<div class="tree-node"><strong>${escapeHtml(label)}</strong><span>${escapeHtml(value)}</span></div>`)
    .join("");
}

function sideProfileSummary(profile, t) {
  const model = profile.model || profile.device || t("unknown");
  const root = profile.is_rooted ? t("android.profile.rooted") : t("android.profile.notRooted");
  return `${model} · API ${profile.api_level || "?"} · ${root}`;
}

function sideInfo(title, body, iconName, icon) {
  return `
    <div class="side-info">
      <span class="metric-icon">${icon(iconName)}</span>
      <span><strong>${title}</strong><small>${body}</small></span>
    </div>
  `;
}

function androidLogContent(android, t, modeId) {
  const log = modeId === "logical" 
    ? (android.logicalLog || []) 
    : (modeId === "filesystem" ? (android.filesystemLog || []) : (android.ramLog || []));
  if (!log.length) return `• ${t("android.logical.waiting")}`;
  return log.map((line) => `• ${line}`).join("<br />");
}

function formatBytes(bytes) {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

function androidMode(modeId, t) {
  const modes = {
    physical: {
      title: t("android.mode.physical.title"),
      desc: t("android.mode.physical.desc"),
      icon: "disk"
    },
    logical: {
      title: t("android.mode.logical.title"),
      desc: t("android.mode.logical.desc"),
      icon: "android"
    },
    filesystem: {
      title: t("android.mode.filesystem.title"),
      desc: t("android.mode.filesystem.desc"),
      icon: "folder"
    },
    ram: {
      title: t("android.mode.ram.title"),
      desc: t("android.mode.ram.desc"),
      icon: "cpu"
    }
  };
  return modes[modeId] || modes.logical;
}

function ramModeOptions(selected, t) {
  const options = [
    ["volatile_data", t("android.ram.mode.volatile")],
    ["root_process_memory", t("android.ram.mode.rootProcess")],
    ["physical_memory_probe", t("android.ram.mode.physicalProbe")]
  ];
  return options
    .map(([value, label]) => `<option value="${value}"${value === selected ? " selected" : ""}>${label}</option>`)
    .join("");
}

function androidImageModeCard(modeId, title, desc, iconName, accent, badge, icon, escapeHtml, options = {}) {
  const disabled = options.disabled ? " disabled aria-disabled=\"true\"" : "";
  const route = options.disabled ? "" : ` data-route="android:${modeId}"`;
  const disabledClass = options.disabled ? " is-disabled" : "";
  return `
    <button class="forensic-card${disabledClass}"${route}${disabled} style="--accent:${accent}">
      <span class="card-icon">${icon(iconName)}</span>
      <h3>${escapeHtml(title)}</h3>
      <p>${escapeHtml(desc)}</p>
      <span class="meta">${escapeHtml(badge)}</span>
    </button>
  `;
}

export async function handleAndroidAction(button, deps) {
  const action = button.dataset.action;
  if (action === "android-adb-check") {
    await checkAdb(button, deps);
    return true;
  }
  if (action === "android-list-devices") {
    await listDevices(button, deps);
    return true;
  }
  if (action === "android-profile-fetch") {
    await fetchDeviceProfile(button, deps);
    return true;
  }
  if (action === "android-start-logical") {
    await startLogicalAcquisition(button, deps);
    return true;
  }
  if (action === "android-stop-logical") {
    await stopLogicalAcquisition(button, deps);
    return true;
  }
  if (action === "android-start-filesystem") {
    await startFilesystemAcquisition(button, deps);
    return true;
  }
  if (action === "android-stop-filesystem") {
    await stopFilesystemAcquisition(button, deps);
    return true;
  }
  if (action === "android-start-ram") {
    await startRamAcquisition(button, deps);
    return true;
  }
  if (action === "android-stop-ram") {
    await stopRamAcquisition(button, deps);
    return true;
  }
  return false;
}

export function syncAndroidDeviceSelection(select, { state, t, showToast }) {
  if (!state.android) state.android = {};
  state.android.selectedDevice = select.value;
  state.android.deviceProfile = null;
  if (select.value) {
    showToast(t("android.devices.selected", { serial: select.value }));
  }
}

async function checkAdb(button, { apiRequest, backendReady, state, t, showToast, render }) {
  if (!backendReady()) {
    showToast(t("android.appModeRequired"), "error");
    return;
  }

  button.disabled = true;
  try {
    const status = await apiRequest("/api/android-adb-status");
    if (!state.android) state.android = {};
    state.android.adbStatus = status;
    if (!status.installed) {
      state.android.devices = [];
      state.android.selectedDevice = "";
    }
    render();
    showToast(status.installed ? t("android.adb.installed") : t("android.adb.missing"), status.installed ? "success" : "error");
  } catch (error) {
    showToast(t("android.adb.checkFailed", { message: error.message }), "error");
  } finally {
    button.disabled = false;
  }
}

async function listDevices(button, { apiRequest, backendReady, state, t, showToast, render }) {
  if (!backendReady()) {
    showToast(t("android.appModeRequired"), "error");
    return;
  }
  if (!state.android?.adbStatus?.installed) {
    showToast(t("android.adb.checkFirst"), "error");
    return;
  }

  button.disabled = true;
  try {
    const result = await apiRequest("/api/android-devices");
    const devices = Array.isArray(result.devices) ? result.devices : [];
    if (!state.android) state.android = {};
    state.android.devices = devices;
    state.android.selectedDevice = devices[0]?.serial || "";
    state.android.deviceProfile = null;
    render();
    showToast(devices.length
      ? t("android.devices.listed", { count: String(devices.length) })
      : t("android.devices.none"),
      devices.length ? "success" : "error"
    );
  } catch (error) {
    showToast(t("android.devices.listFailed", { message: error.message }), "error");
  } finally {
    button.disabled = false;
  }
}

async function fetchDeviceProfile(button, { apiRequest, backendReady, state, t, showToast, render }) {
  if (!backendReady()) {
    showToast(t("android.appModeRequired"), "error");
    return;
  }
  if (!state.android?.selectedDevice) {
    showToast(t("android.logical.deviceRequired"), "error");
    return;
  }

  button.disabled = true;
  try {
    const result = await apiRequest("/api/android-device-profile", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ serial: state.android.selectedDevice }),
    });
    if (!state.android) state.android = {};
    state.android.deviceProfile = result.profile || null;
    render();
    showToast(t("android.profile.loaded"), "success");
  } catch (error) {
    showToast(t("android.profile.failed", { message: error.message }), "error");
  } finally {
    button.disabled = false;
  }
}

async function startLogicalAcquisition(button, { apiRequest, backendReady, state, t, showToast, render, resolveCase }) {
  if (!backendReady()) {
    showToast(t("android.appModeRequired"), "error");
    return;
  }
  if (!state.android?.selectedDevice) {
    showToast(t("android.logical.deviceRequired"), "error");
    return;
  }

  const caseName = resolveCase?.() || null;
  const profileSelect = document.querySelector("[data-android-acquisition-profile]");
  const profile = profileSelect?.value || "full_logical";
  if (!state.android) state.android = {};
  state.android.logicalProfile = profile;
  state.android.logicalLog = [t("android.logical.starting")];
  state.android.logicalJob = null;
  render();

  button.disabled = true;
  try {
    const body = { serial: state.android.selectedDevice, profile };
    if (caseName) body.case_name = caseName;
    const result = await apiRequest("/api/android-profile-acquisition", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    if (result.job_id) {
      state.android.logicalJob = { job_id: result.job_id, status: "running", done: 0, total: 0, message: t("android.logical.starting") };
      render();
      pollLogicalJob(result.job_id, { apiRequest, state, t, showToast, render });
    }
  } catch (error) {
    state.android.logicalLog.push(`❌ ${error.message}`);
    showToast(t("android.logical.failed", { message: error.message }), "error");
    render();
  } finally {
    button.disabled = false;
  }
}

async function stopLogicalAcquisition(button, { apiRequest, backendReady, state, t, showToast, render }) {
  if (!state.android?.logicalJob?.job_id) return;

  button.disabled = true;
  try {
    await apiRequest("/api/acquisition-control", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ job_id: state.android.logicalJob.job_id, action: "stop" }),
    });
    showToast(t("android.logical.stopped"), "success");
  } catch (error) {
    showToast(error.message, "error");
  } finally {
    button.disabled = false;
  }
}

function pollLogicalJob(jobId, { apiRequest, state, t, showToast, render }) {
  const interval = setInterval(async () => {
    try {
      const result = await apiRequest("/api/acquisition-status", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ job_id: jobId }),
      });
      if (!state.android) state.android = {};

      state.android.logicalJob = {
        job_id: jobId,
        status: result.status,
        done: result.done || 0,
        total: result.total || 0,
        message: result.message || "",
        result: result.result || null,
        error: result.error || null,
      };

      // Update log
      if (!state.android.logicalLog) state.android.logicalLog = [];
      const lastMsg = state.android.logicalLog[state.android.logicalLog.length - 1];
      if (result.message && result.message !== lastMsg) {
        state.android.logicalLog.push(result.message);
      }

      if (result.status === "completed") {
        clearInterval(interval);
        state.android.logicalLog.push(`✅ ${t("android.logical.done")}`);
        showToast(t("android.logical.done"), "success");
      } else if (result.status === "failed") {
        clearInterval(interval);
        state.android.logicalLog.push(`❌ ${result.error || t("android.logical.failed", { message: "" })}`);
        showToast(t("android.logical.failed", { message: result.error || "" }), "error");
      }

      render();
    } catch {
      // Silently retry on network hiccup
    }
  }, 1500);
}

async function startFilesystemAcquisition(button, { apiRequest, backendReady, state, t, showToast, render, resolveCase }) {
  if (!backendReady()) {
    showToast(t("android.appModeRequired"), "error");
    return;
  }
  if (!state.android?.selectedDevice) {
    showToast(t("android.logical.deviceRequired"), "error");
    return;
  }

  const hasRootCheckbox = document.querySelector("[data-android-filesystem-has-root]");
  const hasRoot = hasRootCheckbox ? Boolean(hasRootCheckbox.checked) : false;

  const caseName = resolveCase?.() || null;
  if (!state.android) state.android = {};
  state.android.filesystemLog = [t("android.filesystem.starting") || "Dosya sistemi aktarımı başlatılıyor..."];
  state.android.filesystemJob = null;
  render();

  button.disabled = true;
  try {
    const body = { 
      serial: state.android.selectedDevice,
      has_root: hasRoot
    };
    if (caseName) body.case_name = caseName;
    const result = await apiRequest("/api/android-filesystem-image", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    if (result.job_id) {
      state.android.filesystemJob = { job_id: result.job_id, status: "running", done: 0, total: 3, message: "Başlatılıyor..." };
      render();
      pollFilesystemJob(result.job_id, { apiRequest, state, t, showToast, render });
    }
  } catch (error) {
    state.android.filesystemLog.push(`❌ ${error.message}`);
    showToast(error.message, "error");
    render();
  } finally {
    button.disabled = false;
  }
}

async function stopFilesystemAcquisition(button, { apiRequest, backendReady, state, t, showToast, render }) {
  if (!state.android?.filesystemJob?.job_id) return;

  button.disabled = true;
  try {
    await apiRequest("/api/acquisition-control", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ job_id: state.android.filesystemJob.job_id, action: "stop" }),
    });
    showToast(t("android.logical.stopped"), "success");
  } catch (error) {
    showToast(error.message, "error");
  } finally {
    button.disabled = false;
  }
}

function pollFilesystemJob(jobId, { apiRequest, state, t, showToast, render }) {
  const interval = setInterval(async () => {
    try {
      const result = await apiRequest("/api/acquisition-status", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ job_id: jobId }),
      });
      if (!state.android) state.android = {};

      state.android.filesystemJob = {
        job_id: jobId,
        status: result.status,
        done: result.done || 0,
        total: result.total || 0,
        message: result.message || "",
        result: result.result || null,
        error: result.error || null,
      };

      // Update log
      if (!state.android.filesystemLog) state.android.filesystemLog = [];
      const lastMsg = state.android.filesystemLog[state.android.filesystemLog.length - 1];
      if (result.message && result.message !== lastMsg) {
        state.android.filesystemLog.push(result.message);
      }

      if (result.status === "completed") {
        clearInterval(interval);
        state.android.filesystemLog.push(`✅ ${t("android.logical.done")}`);
        showToast(t("android.logical.done"), "success");
      } else if (result.status === "failed") {
        clearInterval(interval);
        state.android.filesystemLog.push(`❌ ${result.error || "Aktarım başarısız"}`);
        showToast(result.error || "Aktarım başarısız", "error");
      }

      render();
    } catch {
      // Silently retry
    }
  }, 1500);
}

async function startRamAcquisition(button, { apiRequest, backendReady, state, t, showToast, render, resolveCase }) {
  if (!backendReady()) {
    showToast(t("android.appModeRequired"), "error");
    return;
  }
  if (!state.android?.selectedDevice) {
    showToast(t("android.logical.deviceRequired"), "error");
    return;
  }

  const hasRootCheckbox = document.querySelector("[data-android-ram-has-root]");
  const hasRoot = hasRootCheckbox ? Boolean(hasRootCheckbox.checked) : false;
  const modeSelect = document.querySelector("[data-android-ram-mode]");
  const mode = modeSelect?.value || "volatile_data";

  const caseName = resolveCase?.() || null;
  if (!state.android) state.android = {};
  state.android.ramMode = mode;
  state.android.ramLog = [t("android.ram.starting") || "RAM aktarımı başlatılıyor..."];
  state.android.ramJob = null;
  render();

  button.disabled = true;
  try {
    const body = { 
      serial: state.android.selectedDevice,
      has_root: hasRoot,
      mode
    };
    if (caseName) body.case_name = caseName;
    const result = await apiRequest("/api/android-ram-image", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    if (result.job_id) {
      state.android.ramJob = { job_id: result.job_id, status: "running", done: 0, total: 3, message: "Başlatılıyor..." };
      render();
      pollRamJob(result.job_id, { apiRequest, state, t, showToast, render });
    }
  } catch (error) {
    state.android.ramLog.push(`❌ ${error.message}`);
    showToast(error.message, "error");
    render();
  } finally {
    button.disabled = false;
  }
}

async function stopRamAcquisition(button, { apiRequest, backendReady, state, t, showToast, render }) {
  if (!state.android?.ramJob?.job_id) return;

  button.disabled = true;
  try {
    await apiRequest("/api/acquisition-control", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ job_id: state.android.ramJob.job_id, action: "stop" }),
    });
    showToast(t("android.logical.stopped"), "success");
  } catch (error) {
    showToast(error.message, "error");
  } finally {
    button.disabled = false;
  }
}

function pollRamJob(jobId, { apiRequest, state, t, showToast, render }) {
  const interval = setInterval(async () => {
    try {
      const result = await apiRequest("/api/acquisition-status", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ job_id: jobId }),
      });
      if (!state.android) state.android = {};

      state.android.ramJob = {
        job_id: jobId,
        status: result.status,
        done: result.done || 0,
        total: result.total || 0,
        message: result.message || "",
        result: result.result || null,
        error: result.error || null,
      };

      // Update log
      if (!state.android.ramLog) state.android.ramLog = [];
      const lastMsg = state.android.ramLog[state.android.ramLog.length - 1];
      if (result.message && result.message !== lastMsg) {
        state.android.ramLog.push(result.message);
      }

      if (result.status === "completed") {
        clearInterval(interval);
        state.android.ramLog.push(`✅ ${t("android.ram.done") || "RAM imajı başarıyla tamamlandı."}`);
        showToast(t("android.ram.done") || "RAM imajı başarıyla tamamlandı.", "success");
      } else if (result.status === "failed") {
        clearInterval(interval);
        state.android.ramLog.push(`❌ ${result.error || "Aktarım başarısız"}`);
        showToast(result.error || "Aktarım başarısız", "error");
      }

      render();
    } catch {
      // Silently retry
    }
  }, 1500);
}

function deviceOptions(devices, selected, t, escapeHtml) {
  if (!devices.length) {
    return `<option value="">${t("android.devices.none")}</option>`;
  }

  return devices
    .map((device) => {
      const serial = device.serial || "";
      const isSelected = serial === selected ? " selected" : "";
      const details = [device.state, device.model, device.product]
        .filter(Boolean)
        .join(" · ");
      const label = details ? `${serial} · ${details}` : serial;
      return `<option value="${escapeHtml(serial)}"${isSelected}>${escapeHtml(label)}</option>`;
    })
    .join("");
}
