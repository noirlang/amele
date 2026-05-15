# Worm Rust Rewrite

Rust rewrite of the Worm forensic technical core. The C/Qt UI is intentionally not ported here yet; this crate exposes system modules that a future Tauri UI can call.

## Current Scope

- Local disk/file imaging with `.partial` preservation on failure.
- SHA256 sidecar output for acquired images.
- MD5, SHA1, SHA256, SHA512 hashing and hash comparison.
- Evidence vault directory management, notes, summaries, and reports.
- Remote agent client compatible with existing `worm-linux` and `worm-win` JSON-over-TCP protocol.
- Remote disk imaging, remote RAM acquisition control, RAM file download, pause/resume/stop commands.
- Local AVML/WinPMEM status and acquisition helpers.
- WireGuard config generation and Linux `wg-quick` start/stop wrapper.

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
```

## Packaging Direction

- Windows: Tauri application packaged as MSI.
- Linux: Tauri application packaged as AppImage.
- Release asset names should stay aligned with the original updater convention: `worm-windows-x64.msi`, `worm-linux-x64.AppImage`, `SHA256SUMS`.

## UI Status

No Tauri UI is implemented in this stage. UI requirements and future design direction are documented in `AGENTS.md`.
