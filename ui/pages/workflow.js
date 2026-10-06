// adım adım vaka yönetim sihirbazı sayfası.

import { icon as defaultIcon } from "../icons.js";

export function workflowPage({
  id,
  workflows,
  state,
  t,
  icon,
  localText,
  canonicalRamFileName,
  caseSelectOptions,
  caseOutputLabel,
  escapeHtml,
}) {
  const data = workflows[id] || workflows["windows-remote-disk"];
  const isSsh = data.mode.startsWith("ssh");
  const isRemote = data.mode.startsWith("remote") || isSsh;
  const isRam = data.mode.includes("ram");
  const toolCheck = data.platform === "Windows" ? "WinPMEM" : "AVML";
  const initialTargetLabel = t("scanDisksFirst");
  const outputField = isRam
    ? ramCasePanel({
        t,
        icon,
        state,
        caseSelectOptions,
        caseOutputLabel,
        escapeHtml,
        canonicalRamFileName,
        platform: data.platform,
      })
    : imageCasePanel({ t, icon, state, caseSelectOptions, caseOutputLabel, escapeHtml });
  const targetSelect = isRam
    ? ""
    : field(
        t("workflow.disk"),
        `<select class="select" data-field="target"><option value="" disabled selected>${initialTargetLabel}</option></select>`
      );
  const scanLabel = isRam
    ? isRemote
      ? t("workflow.checkTool", { tool: toolCheck })
      : t("workflow.checkToolAction", { tool: toolCheck })
    : isRemote
      ? t("workflow.scanDisks")
      : t("workflow.scanLocalDisks");

  const isWindows = data.platform === "Windows";
  const defaultUser = isWindows ? "Administrator" : "root";
  const defaultPort = isWindows ? "22" : "22";

  const isAcquisitionRunning = Boolean(
    state?.activeAcquisition &&
    (state.activeAcquisition.workflowId === id || !state.activeAcquisition.workflowId)
  );

  const connectionBlock = isSsh
    ? `
        <p class="section-label">${t("workflow.sshConnection")}</p>
        ${field(t("workflow.ip"), `<input class="input" data-field="ip" placeholder="192.168.1.100" value="" />`)}
        ${field(t("workflow.port"), `<input class="input" data-field="port" value="${defaultPort}" />`)}
        ${field(t("workflow.sshUser"), `<input class="input" data-field="ssh-user" placeholder="${defaultUser}" />`)}
        ${field(t("workflow.sshPass"), `<input class="input" type="password" data-field="ssh-pass" placeholder="${t("workflow.sshPassPlaceholder")}" />`)}
        ${pickerField(t("workflow.sshKey"), "ssh-key-file", t("workflow.sshKeyPlaceholder"), "file", icon, t)}
        <div class="button-row">
          <button class="primary-button" data-action="connect" data-shortcut="Enter">${icon("key")} ${t("workflow.sshConnect")}<span class="shortcut-key-badge" aria-hidden="true">↵</span></button>
        </div>
      `
    : `
        <p class="section-label">${t("workflow.connectionOps")}</p>
        ${field(t("workflow.ip"), `<input class="input" data-field="ip" placeholder="${t("workflow.ipPlaceholder")}" value="" />`)}
        ${field(t("workflow.port"), `<input class="input" data-field="port" value="4444" />`)}
        ${field(t("workflow.token"), `<input class="input" data-field="token" placeholder="${t("workflow.tokenPlaceholder")}" />`)}
        <div class="button-row">
          <button class="secondary-button" data-action="approve-key" data-shortcut="K">${icon("key")} ${t("workflow.approveKey")}<span class="shortcut-key-badge" aria-hidden="true">K</span></button>
          <button class="secondary-button" data-action="reset-key" data-shortcut="X">${icon("refresh")} ${t("workflow.reset")}<span class="shortcut-key-badge" aria-hidden="true">X</span></button>
          <button class="primary-button" data-action="connect" data-shortcut="Enter">${icon("network")} ${t("workflow.connect")}<span class="shortcut-key-badge" aria-hidden="true">↵</span></button>
        </div>
        <div class="toggle-row">
          <span>${t("workflow.useVpn")}</span>
          <button class="switch" data-action="toggle-vpn" aria-label="${t("workflow.useVpn")}"></button>
        </div>
        <div class="vpn-panel" hidden>
          <button class="secondary-button" data-action="vpn-config">${icon("settings")} ${t("workflow.configureVpn")}</button>
          ${field(t("workflow.server"), `<input class="input" data-field="vpn-endpoint" placeholder="10.0.0.1:51820" />`)}
          ${field(t("workflow.vpnPrivateKey"), `<input class="input" data-field="vpn-private-key" placeholder="YOUR_PRIVATE_KEY" />`)}
          ${field(t("workflow.vpnPublicKey"), `<input class="input" data-field="vpn-public-key" placeholder="SERVER_PUBLIC_KEY" />`)}
          ${field(t("workflow.allowedIps"), `<input class="input" data-field="vpn-allowed" value="0.0.0.0/0" />`)}
          ${field(t("workflow.vpnAddress"), `<input class="input" data-field="vpn-address" value="10.0.0.2/24" />`)}
          ${field(t("workflow.vpnDns"), `<input class="input" data-field="vpn-dns" value="1.1.1.1" />`)}
          ${field(t("workflow.vpnKeepalive"), `<input class="input" data-field="vpn-keepalive" value="25" />`)}
          ${pickerField(t("workflow.configFile"), "vpn-config-file", "wireguard.conf", "file", icon, t)}
          <div class="button-row">
            <button class="primary-button" data-action="save-vpn">${icon("shield")} ${t("workflow.saveVpn")}</button>
            <button class="secondary-button" data-action="start-vpn">${icon("play")} ${t("workflow.startVpn")}</button>
            <button class="danger-button" data-action="stop-vpn">${icon("stop")} ${t("workflow.stopVpn")}</button>
          </div>
        </div>
      `;

  const backRoute = isWindows ? "windows" : "linux";
  const backLabel = isWindows
    ? t("nav.windows") || "Windows Araçları"
    : t("nav.linux") || "Linux Araçları";

  return `
    <section class="page">
      <div class="workflow-header-row">
        <button type="button" class="secondary-button workflow-back-btn" data-route="${backRoute}" data-shortcut="B" data-nav-dir="back">
          ${icon("arrowLeft")} <span>${backLabel}</span>
          <span class="shortcut-key-badge" aria-hidden="true">B</span>
        </button>
      </div>
      <div class="workflow-layout">
        <div class="workflow-panel">
          ${pageTitle(localText(data.title), localText(data.desc), data.icon, icon)}
          <div class="form-grid">
            ${isRemote ? connectionBlock : ""}

            <div class="section-divider"></div>
            <p class="section-label">${t("workflow.caseSection")}</p>
            ${outputField}
            ${
              isRam
                ? field(
                    t("workflow.outputFormat"),
                    `
              <select class="select" data-field="output-format">
                <option value="raw">${t("workflow.formatRaw")}</option>
                <option value="aff4">${t("workflow.formatAff4")}</option>
              </select>
            `
                  )
                : field(
                    t("workflow.outputFormat"),
                    `
              <select class="select" data-field="output-format">
                <option value="raw_sparse" selected>${t("workflow.formatRawSparse")}</option>
                <option value="raw_full">${t("workflow.formatRawFull")}</option>
                <option value="aff4_sparse">${t("workflow.formatAff4Sparse")}</option>
                <option value="aff4_full">${t("workflow.formatAff4Full")}</option>
              </select>
              <p class="field-hint" style="margin-top:6px; font-size:12px; color:var(--text-muted); line-height:1.4;">${t("workflow.sparseModeHint")}</p>
            `
                  )
            }
            <div class="section-divider"></div>
            <p class="section-label">${isRam ? t("workflow.ramOutput") : t("workflow.diskOutput")}</p>
            ${targetSelect}
            <div class="button-row workflow-target-actions">
              <button class="secondary-button" data-action="scan" data-shortcut="S">${icon(isRam ? "chip" : "disk")} ${scanLabel}<span class="shortcut-key-badge" aria-hidden="true">S</span></button>
              ${!isRemote && isRam && data.platform === "Windows" ? `<button class="secondary-button" data-action="download" data-shortcut="D">${icon("refresh")} ${t("workflow.downloadWinpmem")}<span class="shortcut-key-badge" aria-hidden="true">D</span></button>` : ""}
              ${!isRemote && isRam && data.platform === "Linux" ? `<button class="secondary-button" data-action="install-avml" data-shortcut="D">${icon("download")} ${t("workflow.downloadAvml")}<span class="shortcut-key-badge" aria-hidden="true">D</span></button>` : ""}
              <button class="primary-button" data-action="start" data-shortcut="E" ${isAcquisitionRunning ? "hidden disabled" : ""}>${icon(isRam ? "ram" : "disk")} ${isRam ? t("workflow.startRam") : t("workflow.startImage")}<span class="shortcut-key-badge" aria-hidden="true">E</span></button>
            </div>

            <div class="button-row acquisition-controls" data-acquisition-controls ${isAcquisitionRunning ? "" : "hidden"}>
              <button class="secondary-button" data-action="pause" data-shortcut="P">${icon("pause")} ${t("workflow.pause")}<span class="shortcut-key-badge" aria-hidden="true">P</span></button>
              <button class="secondary-button" data-action="resume" data-shortcut="R">${icon("play")} ${t("workflow.resume")}<span class="shortcut-key-badge" aria-hidden="true">R</span></button>
              <button class="danger-button" data-action="stop" data-shortcut="X">${icon("stop")} ${t("workflow.stop")}<span class="shortcut-key-badge" aria-hidden="true">X</span></button>
            </div>

            <div class="section-divider"></div>
            <p class="section-label">${t("workflow.progress")}</p>
            <div class="progress-bar" data-progress style="--value:0%"><span></span><b>0%</b></div>
            <div class="log-box" id="workflow-log">${state.lastLog.map((line) => escapeHtml(line)).join("<br />")}</div>
          </div>
        </div>

        <aside class="side-panel">
          <h3>${t("workflow.status")}</h3>
          ${sideInfo(t("workflow.platform"), `${data.platform} • ${isSsh ? "SSH (Agentless)" : isRemote ? t("remoteAgent") : t("localOperation")}`, data.icon, "", icon)}
          ${sideInfo(t("workflow.connection"), isRemote ? t("notConnected") : t("localCheckWaiting"), "monitor", "connection", icon)}
          ${sideInfo(isRam ? t("workflow.tool") : t("workflow.target"), isRam ? t("localCheckWaiting") : t("targetNotSelected"), isRam ? "chip" : "disk", "target", icon)}
          ${sideInfo(t("workflow.lastAction"), t("lastActionReady"), "clock", "last-action", icon)}
        </aside>
      </div>
    </section>
  `;
}

export function pageTitle(title, _desc, iconName, icon = defaultIcon) {
  return `
    <div class="page-title">
      <span class="page-title-icon">${icon(iconName)}</span>
      <h1 class="page-title-text">${title}</h1>
    </div>
  `;
}

export function field(label, control) {
  return `
    <div class="field">
      <label>${label}</label>
      ${control}
    </div>
  `;
}

export function pickerField(label, id, value, type = "file", icon = defaultIcon, t = (key) => key) {
  const action = type === "folder" ? "pick-folder" : "pick-file";
  const placeholderOnly =
    value.startsWith(".") ||
    value.toLowerCase().includes("seç") ||
    value.toLowerCase().includes("select");
  const valueAttr = placeholderOnly ? `placeholder="${value}" value=""` : `value="${value}"`;
  return field(
    label,
    `<div class="input-action"><input id="${id}" class="input" ${valueAttr} data-picker-target /><button class="secondary-button" data-action="${action}" data-target="#${id}" data-shortcut="F">${icon(type === "folder" ? "folder" : "search")} ${t("select")}<span class="shortcut-key-badge" aria-hidden="true">F</span></button></div>`
  );
}

function imageCasePanel({ t, icon, state, caseSelectOptions, caseOutputLabel, escapeHtml }) {
  return casePanel("ciktilar", t("workflow.caseHint"), {
    t,
    icon,
    state,
    caseSelectOptions,
    caseOutputLabel,
    escapeHtml,
  });
}

function ramCasePanel({
  t,
  icon,
  state,
  caseSelectOptions,
  caseOutputLabel,
  escapeHtml,
  canonicalRamFileName,
  platform = "",
}) {
  return `
    ${casePanel("ram", t("workflow.ramCaseHint"), { t, icon, state, caseSelectOptions, caseOutputLabel, escapeHtml })}
    ${field(t("workflow.outputFileName"), `<input id="workflow-output" class="input" value="${escapeHtml(canonicalRamFileName("", new Date(), platform))}" readonly />`)}
  `;
}

export function casePanel(
  subdir,
  hint,
  { t, icon, state, caseSelectOptions, caseOutputLabel, escapeHtml }
) {
  const selected =
    state.activeCase?.case_name || (state.cases.length ? state.cases[0].case_name : "");
  const output = caseOutputLabel(selected, subdir);
  void hint;
  return `
    ${field(t("workflow.case"), `<select id="workflow-case" class="select" data-case-select data-allow-new-case="1">${caseSelectOptions(selected, { allowNew: true })}</select>`)}
    <div class="button-row">
      <button class="secondary-button" data-action="refresh-cases" data-shortcut="R">${icon("refresh")} ${t("case.refresh")}<span class="shortcut-key-badge" aria-hidden="true">R</span></button>
    </div>
    <div class="side-info">
      <span class="metric-icon">${icon("folder")}</span>
      <span><strong>${t("workflow.caseOutput")}</strong><small data-case-output data-case-output-subdir="${subdir}">${escapeHtml(output)}</small></span>
    </div>
  `;
}

function sideInfo(title, body, iconName, key = "", icon) {
  return `
    <div class="side-info" ${key ? `data-side="${key}"` : ""}>
      <span class="metric-icon">${icon(iconName)}</span>
      <span><strong>${title}</strong><small>${body}</small></span>
    </div>
  `;
}
