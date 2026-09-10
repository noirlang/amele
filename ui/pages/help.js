import { helpDocs } from "../docs/helpContent.js";

export const helpModules = [
  { id: "windows", icon: "windows", titleTr: "Windows Modülü", titleEn: "Windows Module", badge: "Live & Remote" },
  { id: "linux", icon: "linux", titleTr: "Linux Modülü", titleEn: "Linux Module", badge: "Live & Remote" },
  { id: "docker", icon: "docker", titleTr: "Docker Modülü", titleEn: "Docker Module", badge: "Containers" },
  { id: "android", icon: "android", titleTr: "Android Modülü", titleEn: "Android Module", badge: "ADB & Lemon" },
  { id: "ios", icon: "ios", titleTr: "iOS Modülü", titleEn: "iOS Module", badge: "Backup2FS" }
];

export function slugify(text = "") {
  return text
    .toString()
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9\u00C0-\u024F\u1E00-\u1EFF\u0400-\u04FF]+/gi, "-")
    .replace(/^-+|-+$/g, "");
}

export function helpPage({ t, icon, state, pageTitle, escapeHtml }) {
  const activeDocId = state.activeHelpDoc || "windows";
  const activeLang = state.language === "en" ? "en" : "tr";
  const docMarkdown = (helpDocs[activeLang] && helpDocs[activeLang][activeDocId]) || 
                      (helpDocs.tr && helpDocs.tr[activeDocId]) || "";

  const renderedContent = renderMarkdown(docMarkdown, escapeHtml);
  const headings = extractHeadings(docMarkdown);

  const navItems = helpModules.map((mod) => {
    const isActive = mod.id === activeDocId;
    const title = activeLang === "en" ? mod.titleEn : mod.titleTr;
    return `
      <button class="help-nav-btn ${isActive ? "active" : ""}" data-action="help-select-doc" data-doc="${mod.id}" type="button">
        <span class="help-nav-icon">${icon(mod.icon)}</span>
        <span class="help-nav-label">${escapeHtml(title)}</span>
        <span class="help-nav-badge">${escapeHtml(mod.badge)}</span>
      </button>
    `;
  }).join("");

  const tocItems = headings.length
    ? headings.map((h) => {
        const indentClass = `help-toc-level-${h.level}`;
        return `<li><button type="button" class="help-toc-link ${indentClass}" data-action="help-toc" data-target="${h.id}">${escapeHtml(h.title)}</button></li>`;
      }).join("")
    : `<li><span class="help-toc-empty">${t("help.noHeadings") || (activeLang === "en" ? "No headings" : "Başlık yok")}</span></li>`;

  return `
    <section class="page help-page">
      <div class="help-top-header">
        <div class="help-header-title">
          ${pageTitle(t("help.title") || (activeLang === "en" ? "Help & Documentation" : "Yardım ve Dokümantasyon"), t("help.desc") || (activeLang === "en" ? "User guides and technical references for Amele Forensic modules." : "Amele Adli Bilişim modülleri kullanım kılavuzları ve teknik referanslar."), "help")}
        </div>
      </div>

      <div class="help-layout">
        <!-- Sol Modül Listesi -->
        <aside class="help-sidebar">
          <div class="help-sidebar-title">
            <span>${t("help.modules") || (activeLang === "en" ? "Modules" : "Modüller")}</span>
          </div>
          <div class="help-nav-list">
            ${navItems}
          </div>

          <div class="help-sidebar-title" style="margin-top: 20px;">
            <span>${t("help.toc") || (activeLang === "en" ? "Table of Contents" : "İçindekiler")}</span>
          </div>
          <nav class="help-toc-nav">
            <ul class="help-toc-list">
              ${tocItems}
            </ul>
          </nav>
        </aside>

        <!-- Sağ Doküman İçeriği -->
        <main class="help-main-content">
          <article class="help-doc-body markdown-body">
            ${renderedContent}
          </article>
        </main>
      </div>
    </section>
  `;
}

function extractHeadings(md = "") {
  const lines = md.split(/\r?\n/);
  const headings = [];
  const occurrences = new Map();
  let inCode = false;

  for (const line of lines) {
    if (line.trim().startsWith("```")) {
      inCode = !inCode;
      continue;
    }
    if (inCode) continue;

    const match = line.trim().match(/^(#{1,3})\s+(.*)$/);
    if (match) {
      const level = match[1].length;
      const title = match[2].trim();
      const base = slugify(title) || "section";
      const count = occurrences.get(base) || 0;
      occurrences.set(base, count + 1);
      const id = count === 0 ? base : `${base}-${count}`;
      headings.push({ level, title, id });
    }
  }

  return headings;
}

export function renderMarkdown(md = "", escapeHtml = (s) => s) {
  if (!md) return "";
  const lines = md.split(/\r?\n/);
  const out = [];
  const headingOccurrences = new Map();
  let inCodeBlock = false;
  let codeLang = "";
  let codeLines = [];
  let inTable = false;
  let inList = false;
  let listType = "";

  function closeList() {
    if (inList) {
      out.push(listType === "ul" ? "</ul>" : "</ol>");
      inList = false;
      listType = "";
    }
  }

  function closeTable() {
    if (inTable) {
      out.push("</tbody></table></div>");
      inTable = false;
    }
  }

  function formatInline(text) {
    let s = escapeHtml(text);
    s = s.replace(/\*\*\*(.+?)\*\*\*/g, "<strong><em>$1</em></strong>");
    s = s.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
    s = s.replace(/\*([^\*]+)\*/g, "<em>$1</em>");
    s = s.replace(/`([^`]+)`/g, "<code>$1</code>");
    s = s.replace(/\[([^\]]+)\]\(([^)]+)\)/g, `<a href="$2" target="_blank" rel="noopener noreferrer" class="help-link">$1</a>`);
    return s;
  }

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Code blocks
    if (line.trim().startsWith("```")) {
      closeList();
      closeTable();
      if (!inCodeBlock) {
        inCodeBlock = true;
        codeLang = line.trim().slice(3).trim();
        codeLines = [];
      } else {
        inCodeBlock = false;
        const codeText = escapeHtml(codeLines.join("\n"));
        out.push(`
          <div class="help-code-block">
            <div class="help-code-bar">
              <span class="help-code-lang">${escapeHtml(codeLang || "text")}</span>
              <button class="help-copy-btn" data-action="copy-help-code" type="button" title="Kopyala">
                <span>Kopyala</span>
              </button>
            </div>
            <pre><code>${codeText}</code></pre>
          </div>
        `);
        codeLines = [];
        codeLang = "";
      }
      continue;
    }

    if (inCodeBlock) {
      codeLines.push(line);
      continue;
    }

    const trimmed = line.trim();

    if (!trimmed) {
      closeList();
      closeTable();
      continue;
    }

    // Horizontal rule
    if (/^(---|___|\*\*\*)$/.test(trimmed)) {
      closeList();
      closeTable();
      out.push("<hr class=\"help-divider\" />");
      continue;
    }

    // Headings
    if (trimmed.startsWith("#")) {
      closeList();
      closeTable();
      const match = trimmed.match(/^(#{1,6})\s+(.*)$/);
      if (match) {
        const level = match[1].length;
        const title = match[2].trim();
        const base = slugify(title) || "section";
        const count = headingOccurrences.get(base) || 0;
        headingOccurrences.set(base, count + 1);
        const id = count === 0 ? base : `${base}-${count}`;
        out.push(`<h${level} id="${id}" class="help-h${level}">${formatInline(title)}</h${level}>`);
        continue;
      }
    }

    // Table rows
    if (trimmed.startsWith("|") && trimmed.endsWith("|")) {
      closeList();
      if (/^\|(?:\s*:?-+:?\s*\|)+$/.test(trimmed)) {
        continue;
      }
      const rawCols = trimmed.slice(1, -1).split("|").map(c => c.trim());
      if (!inTable) {
        inTable = true;
        out.push("<div class=\"help-table-container\"><table class=\"help-table\"><thead><tr>");
        for (const col of rawCols) {
          out.push(`<th>${formatInline(col)}</th>`);
        }
        out.push("</tr></thead><tbody>");
      } else {
        out.push("<tr>");
        for (const col of rawCols) {
          out.push(`<td>${formatInline(col)}</td>`);
        }
        out.push("</tr>");
      }
      continue;
    } else {
      closeTable();
    }

    // Blockquote
    if (trimmed.startsWith(">")) {
      closeList();
      closeTable();
      const quoteText = trimmed.replace(/^>\s*/, "");
      out.push(`<blockquote class="help-quote">${formatInline(quoteText)}</blockquote>`);
      continue;
    }

    // Unordered list
    if (/^[-*+]\s+/.test(trimmed)) {
      closeTable();
      if (!inList || listType !== "ul") {
        closeList();
        inList = true;
        listType = "ul";
        out.push("<ul class=\"help-list\">");
      }
      const itemText = trimmed.replace(/^[-*+]\s+/, "");
      out.push(`<li>${formatInline(itemText)}</li>`);
      continue;
    }

    // Ordered list
    if (/^\d+\.\s+/.test(trimmed)) {
      closeTable();
      if (!inList || listType !== "ol") {
        closeList();
        inList = true;
        listType = "ol";
        out.push("<ol class=\"help-ordered-list\">");
      }
      const itemText = trimmed.replace(/^\d+\.\s+/, "");
      out.push(`<li>${formatInline(itemText)}</li>`);
      continue;
    }

    closeList();
    closeTable();

    out.push(`<p class="help-para">${formatInline(trimmed)}</p>`);
  }

  closeList();
  closeTable();

  return out.join("\n");
}
