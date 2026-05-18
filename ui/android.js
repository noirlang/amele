export function androidPage({ t, icon, pageTitle, state, escapeHtml, backendReady }) {
  const android = state.android || {};
  const status = android.adbStatus || null;
  const installed = Boolean(status?.installed);
  const devices = Array.isArray(android.devices) ? android.devices : [];
  const selected = android.selectedDevice || devices[0]?.serial || "";
  const statusTitle = status
    ? installed ? t("android.adb.installed") : t("android.adb.missing")
    : t("android.adb.unknown");
  const statusDetail = status?.message || (backendReady() ? t("android.adb.checkHint") : t("android.appModeRequired"));

  return `
    <section class="page">
      ${pageTitle(t("hub.android.title"), t("hub.android.desc"), "android")}
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
      </div>
    </section>
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
  return false;
}

export function syncAndroidDeviceSelection(select, { state, t, showToast }) {
  if (!state.android) state.android = {};
  state.android.selectedDevice = select.value;
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
