# Amele Forensic Tool — User Interface (UI)

A dependency-free, high-performance vanilla ES Module frontend designed for native desktop (WebKitGTK on Linux, WebView2 on Windows) and browser-based remote operation.

---

## Architecture Overview

The UI layer runs as a client communicating with the embedded Amele Rust backend via local loopback HTTP/REST APIs and Server-Sent Events (SSE).

```
ui/
├── app.js               # Application state, route dispatching, modal system & lifecycle
├── icons.js             # Scalable inline SVG icon repository
├── i18n.js              # Turkish / English translation dictionary and formatters
├── developer.js         # Embedded real-time developer console & telemetry inspector
├── index.html           # Desktop shell container with shortcut blockers & viewport setup
├── styles.css           # Master stylesheet aggregator
├── core/                # Reusable runtime engines (API client, agents, profiles, state)
├── pages/               # Primary application view renderers (Home, Workflow, Hubs, Settings)
├── tools/               # Forensic platform modules (Android, iOS, Docker)
└── styles/              # Modular design system stylesheets
```

---

## Key Modules & Capabilities

- **Dashboard & Hubs**: Real-time acquisition status, recent news/announcements, and platform-specific acquisition hubs for Windows and Linux.
- **Mobile Forensics**:
  - **Android**: Logical extraction, physical/filesystem dumps, volatile RAM inspection, application cataloging with risk analysis, and MFT extraction.
  - **iOS**: Automated iTunes/Finder backup normalization, Manifest.db parsing, and domain tree reconstruction.
- **Cloud & Container Forensics**:
  - **Docker**: Overlay2 UpperDir drift analysis, container escape risk grading, environment variable secret detection, and runtime configuration auditing.
- **Telemetry & Developer Mode**: Pressing `Ctrl+Shift+D` or triple-clicking the version badge opens the embedded developer panel for live log streaming, active background job tracking, and memory monitoring.
- **Multilingual (i18n)**: Fully synchronized English and Turkish UI strings with automatic placeholder validation.
- **Visual Design**: High-contrast forensic theme system supporting Dark, Light, and Monochrome tactical modes.

---

## Running and Testing

1. **Native Desktop Window**:
   ```bash
   cargo run -- ui
   ```
2. **Browser Server Mode**:
   ```bash
   cargo run -- server
   # Access via http://127.0.0.1:8080
   ```
3. **Frontend Automated Test Suite**:
   ```bash
   node --test tests/i18n.test.js
   node --test tests/routes.test.js
   ```
