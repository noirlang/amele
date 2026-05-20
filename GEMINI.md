# Worm Project Context

Worm is a desktop forensic tool for disk/RAM imaging, hash verification, remote agent control, case management, and reporting. It features a Rust backend and a vanilla HTML/CSS/JS UI served in a native window (GTK/WebKit on Linux, WebView2 on Windows).

## Project Overview

- **Purpose:** Forensic acquisition (Disk/RAM/Android), hash calculation, remote agent management.
- **Backend:** Rust (2024 edition).
- **Frontend:** Vanilla HTML/CSS/JS (no heavy frameworks).
- **Architecture:** The binary serves as both a CLI and a UI host. It uses an elevated helper pattern (re-invoking itself via `sudo`/`pkexec`) for privileged operations.
- **Target Platforms:** Linux and Windows. **macOS is NOT supported.**

## Building and Running

### Prerequisites

**Linux (Ubuntu/Debian):**
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

**Windows:**
Requires Microsoft Edge WebView2 Runtime.

### Commands

- **Build:** `cargo build --locked` (Debug) or `cargo build --release --locked` (Release).
- **Test:** `cargo test --locked`.
- **Format Check:** `cargo fmt --all -- --check`.
- **Run UI (Native):** `cargo run -- ui`.
- **Run UI (Browser):** `cargo run -- ui-browser` (for debugging).
- **UI Syntax Check:** `node --check ui/app.js`.

### CLI Tools

- `cargo run -- hash <file> [md5|sha1|sha256|sha512]`
- `cargo run -- disk-list`
- `cargo run -- ram-status`
- `cargo run -- remote-disks <ip> <port> [token]`

## Development Conventions

### Rust
- **Style:** Standard `rustfmt` formatting.
- **Modules:** 
    - `src/main.rs`: CLI dispatch and privileged helper entry points.
    - `src/lib.rs`: Core logic exports.
    - `src/ui_server.rs`: HTTP API and UI asset serving.
    - `src/native_window.rs`: Native window wrappers.
    - `src/disk.rs`, `src/ram.rs`, `src/remote.rs`: Acquisition and agent protocols.
    - `src/android.rs`: Android ADB logical acquisition (device info, app list, media, notifications, etc.).
    - `src/hash.rs`, `src/evidence.rs`, `src/report.rs`: Utility and management.
- **Privileged Helpers:** Commands like `image-helper`, `ram-helper`, etc., are intended to be run with elevated privileges and communicate via JSON files.

### UI
- **Technology:** Vanilla HTML, CSS, and JavaScript.
- **Location:** `ui/` directory.
- **Interactions:** The UI communicates with the backend via a loopback HTTP API (127.0.0.1).

### Protocols
- **Agent Protocol:** Uses JSON-over-TCP. Turkish field names (`komut`, `durum`, `guvenlik_anahtar_b64`) are **mandatory** for compatibility with `worm-linux` and `worm-win` agents. Do not rename them.

### Testing
- **Unit Tests:** Located within modules in `#[cfg(test)]` blocks.
- **Mocks:** Use `tempfile` and local mock TCP listeners. Do not require real hardware or external network services.

## Security and Safety
- **Evidence Protection:** Never commit evidence files, memory dumps, VPN private keys, or generated reports (typically stored in `~/Worm/Vakalar`).
- **Authorization:** Only use this tool in authorized forensic workflows.

## Key Files
- `Cargo.toml`: Project dependencies and configuration.
- `GEMINI.md`: Detailed guidance for AI assistants.
- `AGENTS.md`: Repository guidelines and structure.
- `src/main.rs`: Entry point for CLI and UI.
- `src/android.rs`: Android logical acquisition backend (19 collection steps).
- `ui/app.js`: Main frontend logic.
- `ui/android.js`: Android UI (mode selection, ADB check, device selection, case selection, acquisition).
- `docs/android.md`: Android module documentation (collected data, folder structure, limitations).

## Android Module

### Logical Acquisition (Active)
Root yetkisi olmadan ADB ile erişilebilir verileri toplar. `POST /api/android-logical-image` endpoint'i üzerinden çalışır.

**İş akışı:** ADB Kontrol → Cihaz Seç → Vaka Seç → İmaj Al

**Toplanan veriler (19 adım):**
- Sistem: `getprop`, paket listesi, logcat, çalışan süreçler, disk kullanımı
- Dumpsys: battery, wifi, bluetooth, usagestats, account, connectivity, notification
- Bildirim geçmişi: `dumpsys notification` + etkinse `cmd notification dump_history`
- Ağ: ip addr, ip route, netstat, ip neigh
- Ekran görüntüsü: `screencap`
- Medya: WhatsApp, Telegram, WA Business, Instagram, Messenger, Viber, Google Messages
- Bugreport + paylaşılan depolama (`/sdcard/`)
- Manifest: SHA-256 hash'li `manifest.json`

Detaylı bilgi: `docs/android.md`
