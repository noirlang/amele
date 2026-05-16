# Worm Rust Rewrite

Rust rewrite of the Worm forensic technical core. The C/Qt UI is being replaced by a dependency-free frontend shell served by the Rust binary for local testing, with a future Tauri packaging path still available.

## Current Scope

- Local disk/file imaging with `.partial` preservation on failure.
- SHA256 sidecar output for acquired images.
- MD5, SHA1, SHA256, SHA512 hashing and hash comparison.
- Evidence vault directory management, notes, summaries, and reports.
- Remote agent client compatible with existing `worm-linux` and `worm-win` JSON-over-TCP protocol.
- Remote disk imaging, remote RAM acquisition control, RAM file download, pause/resume/stop commands.
- Local AVML/WinPMEM status and acquisition helpers.
- WireGuard config generation and Linux `wg-quick` start/stop wrapper.
- Dependency-free UI shell under `ui/`, served by `cargo run -- ui` inside a native GTK/WebKit window and wired to selected Rust core APIs.

## CLI Smoke Commands

```bash
cargo test
cargo run -- settings-default
cargo run -- hash <file> sha256
cargo run -- disk-list
cargo run -- disk-size <device-or-file>
cargo run -- verify <image> <sha256>
cargo run -- ram-status
cargo run -- remote-disks <ip> <port> [token]
cargo run -- remote-image <ip> <port> <disk_id> <output_dir> [token]
cargo run -- remote-tool-check <ip> <port> <winpmem|avml> [token]
cargo run -- wireguard-config <path>
cargo run -- ui
cargo run -- ui-browser
```

## Packaging Direction

- Windows: Tauri application packaged as MSI.
- Linux: Tauri application packaged as AppImage.
- Release asset names should stay aligned with the original updater convention: `worm-windows-x64.msi`, `worm-linux-x64.AppImage`, `SHA256SUMS`.

## UI Status

The frontend shell is under `ui/`. Run `cargo run -- ui` to start the Rust localhost backend and open it in a native GTK/WebKit window. `cargo run -- ui-browser` is available only as a browser-based debug fallback. Disk listing, RAM tool checks, native file/folder picking, remote disk listing, remote RAM tool checks, and hash calculation are wired to Rust APIs.
