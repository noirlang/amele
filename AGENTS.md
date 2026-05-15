# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the Rust forensic core. Key modules include `disk.rs`, `ram.rs`, `remote.rs`, `hash.rs`, `evidence.rs`, `report.rs`, `settings.rs`, and `wireguard.rs`; `main.rs` is a temporary CLI smoke-test entry point and `lib.rs` exposes reusable modules for future Tauri commands. The static Tauri-ready UI prototype lives in `ui/`, with `ui/index.html`, `ui/app.js`, `ui/styles.css`, and bundled assets under `ui/assets/`. The original C/Qt app is outside this repo at `../worm`; existing remote agents are `../worm-linux` and `../worm-win`.

## Build, Test, and Development Commands
Run `cargo check` for a fast Rust compile check. Use `cargo test` to run unit tests for acquisition helpers, hashing, reports, settings, WireGuard, and remote protocol behavior. Run `cargo fmt --all --check` before commits. CLI smoke examples: `cargo run -- hash <file> sha256`, `cargo run -- disk-list`, `cargo run -- ram-status`, and `cargo run -- remote-disks <ip> <port> [token]`. Preview the UI with `python3 -m http.server 4173 --bind 127.0.0.1` from the repo root.

## Coding Style & Naming Conventions
Use Rust 2024 with standard `rustfmt` formatting and idiomatic snake_case for functions, variables, and modules. Keep public types descriptive, for example `RemoteTransferResult` or `RamAcquisitionResult`. Preserve Turkish protocol field names where they mirror existing agents, such as `komut`, `durum`, and `guvenlik_anahtar_b64`. UI code is dependency-free vanilla HTML/CSS/JS; keep selectors semantic and assets local.

## Testing Guidelines
Place Rust unit tests near the module they cover under `#[cfg(test)]`. Name tests by expected behavior, for example `calculates_known_hashes` or `acquires_image_stream_with_agent_protocol`. Do not require real disks, RAM dumps, VPN secrets, or network services for normal tests; use temporary files and local mock TCP listeners.

## Commit & Pull Request Guidelines
History uses concise conventional-style commits such as `feat: port local acquisition modules`, `fix: align ui pages with qt source`, and `style: modernize ui branding and responsive layout`. Keep commits focused. PRs should include a summary, tests run, affected platforms, screenshots for UI changes, and notes for any untested Windows/Linux-specific behavior.

## Security & Compatibility Notes
Never commit evidence files, memory dumps, VPN private keys, tokens, or generated reports. Keep JSON-over-TCP compatibility with `worm-linux` and `worm-win`; do not rename protocol fields unless the agents are updated together.
