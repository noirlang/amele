# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Project Is

Worm is a desktop digital forensics tool for Linux and Windows. It provides local and remote disk/RAM acquisition, hash calculation, evidence case management, forensic image mounting, WireGuard VPN config generation, and report generation. The UI is a native window (GTK/WebKit on Linux, WebView2 on Windows) backed by a loopback HTTP server.

## Build & Run Commands

```bash
cargo build --locked                  # debug build
cargo build --release --locked        # release build
cargo run -- ui                       # launch native UI
cargo run -- ui-browser               # open UI in default browser (debug mode)
cargo fmt --all -- --check            # check Rust formatting
```

## Testing

```bash
bash tests/run_tests.sh               # run all tests (Rust + JS)
cargo test --locked                   # Rust unit tests only
node --test tests/i18n.test.js        # translation key parity check
node --test tests/routes.test.js      # JS route module loading check
node --check ui/app.js                # JS syntax validation
```

Run a single Rust test: `cargo test --locked <test_name>`

CI runs: fmt check → `node --check` → `cargo test` → release build on both Ubuntu and Windows runners.

## Architecture

### Dual-process privilege model

`src/main.rs` serves both as the app entrypoint and as a privileged helper. The app re-invokes itself (via `pkexec` or equivalent) using subcommands like `image-helper`, `ram-helper`, and `mount-helper` to perform privileged I/O. Parent and helper communicate via JSON files on disk — the helper polls a "control file" every 200ms for pause/resume/stop signals.

### Loopback HTTP API pattern

The native window is a thin WebView wrapper. All logic lives in Rust; the frontend is stateless HTML/JS that calls the loopback API. `src/server.rs` binds a random port on `127.0.0.1`; `src/router.rs` dispatches requests; `src/api/` contains domain handlers (evidence, ram, system, android, wireguard).

### Frontend

Pure vanilla HTML/CSS/ES module JavaScript — no bundler, no framework, no build step. Served directly by the Rust server. Route pages live in `ui/pages/<name>.js`; shared helpers go through `ui/app.js`. Translations (Turkish/English) are in `ui/i18n.js`.

### Platform gating

Windows-specific GUI (winit/wry) and syscall crates are gated with `#[cfg(windows)]` and `[target.'cfg(windows)'.dependencies]`. Linux-specific code uses `#[cfg(unix)]` or `#[cfg(target_os = "linux")]`.

### Key source files

| File | Purpose |
|---|---|
| `src/main.rs` | CLI entry point and all subcommand handlers |
| `src/server.rs` | Loopback HTTP server |
| `src/router.rs` | Request routing |
| `src/api/` | HTTP API handler modules |
| `src/disk.rs` | Local disk imaging |
| `src/ram.rs` | RAM acquisition (AVML/WinPMEM) |
| `src/remote.rs` | JSON-over-TCP client for remote agents |
| `src/job.rs` | Job pause/resume/stop lifecycle |
| `src/error.rs` | `WormError`, `HataKodu`, `WormResult<T>` |
| `src/settings.rs` | `AppSettings` struct and defaults |
| `ui/app.js` | Frontend bootstrap and shared helpers |
| `ui/i18n.js` | Bilingual translation strings |

### Evidence storage

All evidence is always under `~/Worm/Vakalar` — this path is fixed, not configurable.

## Coding Conventions

- `rustfmt` defaults for Rust; `snake_case` functions/modules, `PascalCase` types, `SCREAMING_SNAKE_CASE` constants.
- Rust tests are colocated with source modules using `#[cfg(test)]`. Tests must not touch real disks, RAM devices, or privileged paths.
- New UI route pages go in `ui/pages/<name>.js`; bind shared helpers via `ui/app.js`.
- Privileged operations must go through existing helper flows, not ad hoc shell commands.
- Commit style: short imperative or Conventional Commit (`feat: ...`, `fix: ...`, `ux(case): ...`), one behavior per commit.

## What Not to Commit

`target/`, `dist/`, AppImages, raw disk images, memory dumps, `agent.md`, or any forensic case output.
