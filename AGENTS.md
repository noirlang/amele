# AI Agent Guide

This repository is the Rust rewrite of the Worm forensic application's technical core. The original application lives in `../worm`; existing remote agents live in `../worm-linux` and `../worm-win`.

## Non-Negotiable Goals

- Preserve existing forensic behavior while replacing the C/C++ technical layer with Rust.
- Keep compatibility with the current `worm-linux` and `worm-win` agents.
- Tauri UI work has started as a static frontend shell under `ui/`; keep the Rust core independent and wire UI actions through Tauri commands later.
- Keep the Rust core usable from a future Tauri frontend through library modules, not only through CLI code.
- Commit work in small, meaningful steps.

## Current Architecture

- `src/settings.rs`: default app settings matching the C defaults.
- `src/logging.rs`: case-aware log file writer.
- `src/error.rs`: Worm error codes and last-error storage.
- `src/job.rs`: job status/type models and a basic queue.
- `src/disk.rs`: local disk/file size, disk discovery, raw acquisition, cancellation, SHA256 sidecar, image verification.
- `src/hash.rs`: MD5/SHA1/SHA256/SHA512 calculation and comparison.
- `src/evidence.rs`: evidence vault/case directory layout: `gunlukler`, `ciktilar`, `raporlar`, `hash`, `notlar`.
- `src/report.rs`: TXT/JSON forensic report generation and file summaries.
- `src/remote.rs`: JSON-over-TCP client for existing Python agents.
- `src/ram.rs`: local AVML/WinPMEM status and acquisition helpers.
- `src/wireguard.rs`: WireGuard config creation and Linux `wg-quick` wrapper.
- `src/main.rs`: temporary technical CLI for smoke testing until Tauri is added.
- `ui/`: dependency-free Tauri-ready frontend prototype. It is static for now and should later call Rust through Tauri commands.

## Existing Agent Protocol

The controller connects over TCP and sends one JSON object per line. Binary payloads are sent only after a `{"tur":"veri_basliyor"}` JSON line.

Handshake:

```json
{"komut":"merhaba","istemci":"worm","surum":"0.1","token":"optional","guvenlik_anahtar_b64":"optional-base64"}
```

Expected success:

```json
{"durum":"ok","sunucu":"linux-ajan|windows-ajan","surum":"...","ozellikler":["disk_imaj"]}
```

Supported commands that must remain compatible:

- `disk_listele`
- `imaj_baslat`
- `edinim_kontrol` with `eylem`: `pause`, `resume`, `stop`
- `winpmem_kontrol`
- `avml_kontrol`
- `ram_edinim_baslat`
- `ram_dosya_indir`

Do not change these JSON field names unless the existing Python agents are updated at the same time.

## Acquisition Rules

- Incomplete local and remote image files must be renamed to `<target>.partial`.
- Disk imaging reads and writes raw bytes.
- Default chunk size is 4 MiB.
- SHA256 sidecar format is compatible with the C version: `<hash>  <filename>`.
- Linux local disk discovery should cover `/dev/sd*`, `/dev/nvme*n1`, and `/dev/vd*`.
- Windows local disk discovery should cover `\\.\PhysicalDrive0..31`.

## Future Tauri UI Direction

Use the screenshots supplied by the user as the design direction: modern dark forensic dashboard, glassy cards, green accent, strong icons, and a cleaner UX than the old Qt UI.

Preserve the original page coverage from `../worm/kaynak/qt/ana_qt.cpp`, but reorganize it for Tauri:

- Home dashboard with status cards and clear entry points.
- Windows Tools page should first show option cards: Windows local RAM, Windows remote RAM, Windows remote disk, Windows local disk.
- Linux Tools page should mirror the same pattern: Linux local RAM, Linux remote RAM, Linux remote disk, Linux local disk.
- Agent page should include a short, polished documentation-style guide for Linux and Windows agents, download/run steps, security token behavior, and port notes.
- Analysis, hash, evidence vault, reports, and settings pages must keep the required original functionality.
- Dark mode must not be toggled from the home screen; keep it under Settings.
- Move the application update section into Settings rather than keeping it as a separate top-level page.
- Do not copy the screenshots exactly; follow the visual language while making the UI coherent and production-ready.

## Build And Verification

Run these before committing Rust changes:

```bash
cargo fmt --all --check
cargo check
cargo test
```

If touching Windows-specific code, try a Windows target check when available. If no Windows Rust target is installed in the environment, note that explicitly in the final response.

## Security Notes

- Never commit evidence files, memory dumps, VPN private keys, tokens, or generated reports.
- Treat RAM and disk acquisition paths as sensitive.
- Keep agent authentication fail-closed: if the client sends a key, an agent without a configured key must reject it; if the agent requires a key, the client must send `guvenlik_anahtar_b64`.
- Avoid destructive filesystem behavior. Preserve partial data whenever possible.
