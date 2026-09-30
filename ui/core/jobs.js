// arkada çalışan işlerin durumunu sorgulayıp sağ alttaki widgeta basan kısım.

import { escapeHtml } from "./utils.js";

let widgetElement = null;
let minimizedElement = null;
let containerElement = null;
let currentJobs = {};
let isMinimized = false;
let pollingInterval = null;

// sistem ikonlariyla uyumlu ince cizgi svg seti (jobs.css .job-icon svg bekler).
const TOOL_SVGS = {
  disk: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="9"/><circle cx="12" cy="12" r="2.5"/></svg>`,
  android: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="6" y="8" width="12" height="11" rx="3"/><path d="M9 8L7.5 4.5M15 8l1.5-3.5M9.5 13h.01M14.5 13h.01"/></svg>`,
  ios: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="8" y="2.5" width="8" height="19" rx="2.5"/><path d="M11 18.5h2"/></svg>`,
  docker: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2.5l8 4.5v9l-8 4.5-8-4.5v-9z"/><path d="M12 11.5L4 7M12 11.5l8-4.5M12 11.5V20"/></svg>`,
  ram: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><rect x="7" y="7" width="10" height="10" rx="1.5"/><path d="M10 2.5v4M14 2.5v4M10 17.5v4M14 17.5v4M2.5 10h4M2.5 14h4M17.5 10h4M17.5 14h4"/></svg>`,
  update: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/></svg>`,
  default: `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="3"/><path d="M12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3"/></svg>`
};

function getSvgForJob(job) {
  const text = `${job?.id || ""} ${(job?.logs || []).join(" ")}`.toLowerCase();
  if (text.includes("güncelle") || text.includes("update") || text.includes("indir")) return TOOL_SVGS.update;
  if (text.includes("disk") || text.includes("imaj") || text.includes("image")) return TOOL_SVGS.disk;
  if (text.includes("android")) return TOOL_SVGS.android;
  if (text.includes("ios")) return TOOL_SVGS.ios;
  if (text.includes("docker")) return TOOL_SVGS.docker;
  if (text.includes("ram") || text.includes("memory") || text.includes("bellek")) return TOOL_SVGS.ram;
  return TOOL_SVGS.default;
}

// is basligi olarak ilk log satiri gosterilir (acq-12 yerine "Yerel RAM edinimi...").
function getTitleForJob(id, job) {
  const first = Array.isArray(job?.logs) ? job.logs.find((line) => String(line || "").trim() !== "") : "";
  return first || id;
}

function truncate(str, maxLength = 30) {
  if (!str) return "";
  if (str.length <= maxLength) return str;
  return str.slice(0, maxLength - 3) + "...";
}

// stiller styles/jobs.css dosyasindan gelir, burada gomulu stil yok.
function createDOM() {
  if (typeof document === "undefined" || typeof document.createElement !== "function" || !document.body) return;
  widgetElement = document.createElement("div");
  widgetElement.id = "amele-jobs-widget";
  widgetElement.className = "hidden-state";
  
  containerElement = document.createElement("div");
  containerElement.id = "amele-jobs-container";
  
  const header = document.createElement("div");
  header.className = "jobs-header";
  header.innerHTML = `
    <div class="jobs-header-left">
      <span class="jobs-live-dot"></span>
      <div class="jobs-title">Aktif İşlemler</div>
    </div>
    <button class="jobs-minimize-btn">Gizle</button>
  `;
  
  header.querySelector(".jobs-minimize-btn").addEventListener("click", () => {
    isMinimized = true;
    render();
  });
  
  const jobsList = document.createElement("div");
  jobsList.id = "amele-jobs-list";
  
  containerElement.appendChild(header);
  containerElement.appendChild(jobsList);
  
  minimizedElement = document.createElement("div");
  minimizedElement.id = "amele-jobs-minimized";
  minimizedElement.innerHTML = `
    <div class="spinner"></div>
    <span class="count-text">0 İşlem</span>
  `;
  
  minimizedElement.addEventListener("click", () => {
    isMinimized = false;
    render();
  });
  
  widgetElement.appendChild(containerElement);
  widgetElement.appendChild(minimizedElement);
  document.body.appendChild(widgetElement);
}

let navigateHandler = null;

function handleJobClick(jobId) {
  let targetRoute = "home";
  const lower = String(jobId || "").toLowerCase();
  if (lower.includes("android")) targetRoute = "android";
  else if (lower.includes("ios")) targetRoute = "ios";
  else if (lower.includes("docker")) targetRoute = "docker";
  else if (lower.includes("windows")) targetRoute = "windows";
  else if (lower.includes("linux")) targetRoute = "linux";
  else if (lower.includes("disk") || lower.includes("ram")) targetRoute = "windows";
  else targetRoute = "other";

  if (typeof navigateHandler === "function") {
    navigateHandler(targetRoute);
  } else {
    const el = document.querySelector(`[data-route="${targetRoute}"]`);
    if (el) {
      el.click();
    }
  }
}

function render() {
  const activeJobEntries = Object.entries(currentJobs).filter(([_, job]) => job.status === "running");
  
  if (activeJobEntries.length === 0) {
    widgetElement.classList.add("hidden-state");
    setTimeout(() => {
      if (Object.keys(currentJobs).filter(k => currentJobs[k].status === "running").length === 0) {
        widgetElement.style.display = "none";
      }
    }, 300);
    return;
  }
  
  widgetElement.style.display = "block";
  // Trigger reflow
  void widgetElement.offsetWidth;
  widgetElement.classList.remove("hidden-state");
  
  if (isMinimized) {
    containerElement.style.display = "none";
    minimizedElement.style.display = "flex";
    minimizedElement.querySelector(".count-text").textContent = `${activeJobEntries.length} İşlem`;
  } else {
    containerElement.style.display = "block";
    minimizedElement.style.display = "none";
    
    const listEl = containerElement.querySelector("#amele-jobs-list");
    listEl.innerHTML = "";
    
    activeJobEntries.forEach(([id, job]) => {
      const el = document.createElement("div");
      el.className = "job-item";
      el.onclick = () => handleJobClick(id);
      
      const pct = job.total ? Math.min(100, Math.round((job.done / job.total) * 100)) : 0;
      const phasePrefix = job.phase && !String(job.message || "").includes(job.phase) ? `${job.phase}: ` : "";
      const msg = truncate(`${phasePrefix}${job.message || "İşlem devam ediyor..."}`, 45);
      const title = truncate(getTitleForJob(id, job), 40);

      el.innerHTML = `
        <div class="job-info">
          <span class="job-icon">${getSvgForJob({ id, ...job })}</span>
          <span class="job-desc">${escapeHtml(title)}</span>
        </div>
        <div class="job-progress-bg">
          <div class="job-progress-fill" style="width: ${pct}%"></div>
        </div>
        <div class="job-status">
          <span class="job-msg">${escapeHtml(msg)}</span>
          <span class="job-pct">${pct}%</span>
        </div>
      `;
      listEl.appendChild(el);
    });
  }
}

async function pollJobs() {
  try {
    const res = await fetch("/api/acquisition-status");
    if (res.ok) {
      const data = await res.json();
      currentJobs = data.jobs || {};
      render();
    }
  } catch (err) {
    // silently fail and try again next tick
  }
}

export function initJobWidget(options = {}) {
  if (options && typeof options.onNavigate === "function") {
    navigateHandler = options.onNavigate;
  }
  if (widgetElement) return;
  createDOM();
  pollJobs();
  pollingInterval = setInterval(pollJobs, 800);
  if (typeof pollingInterval?.unref === "function") {
    pollingInterval.unref();
  }
}

export function getActiveJobs() {
  return Object.entries(currentJobs)
    .filter(([_, job]) => job.status === "running")
    .map(([id, job]) => ({ id, ...job }));
}

export function isToolBusy(toolName) {
  const toolLower = (toolName || "").toLowerCase();
  const jobs = getActiveJobs();
  return jobs.some(job => job.id.toLowerCase().includes(toolLower));
}
