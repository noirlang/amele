# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

Worm is a desktop forensic tool (Rust backend + vanilla HTML/CSS/JS UI) for disk/RAM imaging, hash verification, remote agent control, case management, and reporting. Targets Linux and Windows only; macOS is explicitly unsupported.

## Build & development commands

```bash
cargo check                         # fast compile check
cargo build --locked                # debug build
cargo build --release --locked      # release build
cargo test --locked                 # run all tests
cargo fmt --all -- --check          # format check (required before commits)
node --check ui/app.js              # UI JS syntax check
```

Run the app:

```bash
cargo run -- ui                     # native GTK/WebKit window (Linux) or WebView2 (Windows)
cargo run -- ui-browser             # open in browser for debugging (no native window)
```

Preview UI without building: `python3 -m http.server 4173 --bind 127.0.0.1` from repo root.

## Linux system dependencies

```bash
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

## Architecture

The binary serves dual roles: a **CLI** (dispatched via `main.rs` match on the first argument) and a **UI host** (`ui` / `ui-browser` subcommands). The UI talks only to a loopback HTTP API served by `ui_server.rs`; it never connects to the internet directly.

### Rust modules (`src/`)

| Module | Responsibility |
|---|---|
| `main.rs` | CLI dispatch and privileged helper entry points (`image-helper`, `ram-helper`, `disk-list-helper`, `mount-helper`) |
| `lib.rs` | Re-exports all modules for use from `main.rs` and future embedding |
| `ui_server.rs` | Loopback HTTP API + static UI asset serving; spawns native window |
| `native_window.rs` | GTK/WebKit (Linux) and WebView2 (Windows) window wrappers |
| `disk.rs` | Local disk/file imaging with SHA256 sidecar, `.partial` safety |
| `ram.rs` | AVML (Linux) and WinPMEM (Windows) acquisition with cancellation token |
| `remote.rs` | JSON-over-TCP client compatible with `worm-linux` / `worm-win` agents |
| `hash.rs` | MD5, SHA1, SHA256, SHA512 computation |
| `evidence.rs` | Case tree under `~/Worm/Vakalar` |
| `report.rs` | JSON/TXT report generation |
| `wireguard.rs` | WireGuard config generation and wg-quick wrapper |
| `settings.rs` | `AppSettings` struct and defaults |
| `error.rs` | `WormError` / `HataKodu` error types |
| `logging.rs` | Logging helpers |

### Elevated helper pattern

Operations requiring root (disk listing on Linux, disk imaging, RAM acquisition, image mounting) are performed by re-invoking the same binary as a privileged subprocess via `pkexec` or `sudo`. The helper subcommands (`disk-list-helper`, `image-helper`, `ram-helper`, `mount-helper`) communicate through JSON files (request, result, progress, control) rather than stdio so the unprivileged parent can poll for progress and send pause/resume/stop signals.

### Remote agent protocol

`remote.rs` implements the JSON-over-TCP protocol used by `worm-linux` and `worm-win`. Turkish field names (`komut`, `durum`, `guvenlik_anahtar_b64`, etc.) must be preserved — changing them requires updating both agents simultaneously.

Core agent commands: `merhaba`, `disk_listele`, `imaj_baslat`, `winpmem_kontrol`, `avml_kontrol`, `ram_edinim_baslat`, `ram_dosya_indir`, `edinim_kontrol`.

## Naming conventions

- Rust: standard `snake_case`; public types are descriptive (`RemoteTransferResult`, `RamAcquisitionResult`, `DiskAcquisitionControl`)
- Turkish protocol field names are intentional — keep them as-is
- UI: dependency-free vanilla JS/CSS; semantic selectors; all assets local

## Testing

Unit tests live in `#[cfg(test)]` blocks within each module. Tests must not require real disks, RAM dumps, VPN secrets, or live network services — use `tempfile` and local mock TCP listeners.

Run a single test:
```bash
cargo test --locked <test_name>
```

## What not to commit

Evidence files, memory dumps, VPN private keys, tokens, generated reports, or any file from `~/Worm/Vakalar`.
