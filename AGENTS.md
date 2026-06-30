# amele-next

Rust 2024 edition native desktop app (wry+WebKit/WebView2). Single binary, no workspace.

## Quick start

```bash
# run native app
cargo run -- ui

# run in browser for debug
cargo run -- ui-browser

# format + check before commit (CI enforces these)
cargo fmt
cargo test --locked
node --check ui/app.js
node --test tests/routes.test.js
```

## CI rules

- `cargo fmt --all -- --check` must pass. Always run `cargo fmt` before committing.
- CI uses `-D warnings` (converted from `RUSTFLAGS`). Fmt + no warnings required.
- Every push to the `dev` branch triggers the CI pipeline. However, **full builds and prereleases** are only run if the commit message contains the `[build]` tag (or via `workflow_dispatch` manual trigger).

## Architecture

- **Backend**: `src/api/router.rs` dispatches all HTTP API calls. No web framework -- manual match on `(method, path)`.
- **Frontend**: vanilla JS ES modules in `ui/`. No bundler. Raw `import` statements.
- **Communication**: `fetch()` to `localhost:{port}`. API endpoints defined in `src/api/router.rs`.
- **State**: `src/api/state.rs` holds mutable global state via `Mutex<ApiState>`.
- **Developer console**: open by clicking logo 5 times. Uses `/api/open-dev-console` in native mode (window.open doesn't work in wry WebView).
- **Native WebView detection**: `?native=1` URL query param sets `window.isNativeWebView`.

## File layout

| Path | Purpose |
|------|---------|
| `src/main.rs` | CLI entrypoint, subcommands |
| `src/lib.rs` | Module declarations |
| `src/api/` | HTTP API handlers + router + state |
| `src/server.rs` | HTTP server bootstrap |
| `src/ram.rs` | RAM acquisition (AVML/WinPMEM/Volatility) |
| `src/disk.rs` | Disk imaging |
| `src/android/` | Android ADB/acquisition modules |
| `src/volatility.rs` | Volatility3 integration |
| `ui/` | Frontend: ES modules, no framework |
| `ui/developer.js` | Dev console (5x logo click) |
| `tests/routes.test.js` | Frontend module health tests |
| `scripts/` | Linux/Windows build scripts |
| `packaging/` | WiX MSI source |
| `.github/workflows/ci.yml` | CI pipeline definition |

## Repos & remotes

- `upstream` → `noirlang/amele` (main repo)
- `wormnext` → `worm-next/worm-next` (active repo)


## Quirks

- `window.fetch` may be undefined in test environment (Node.js test runner). Guard calls with `typeof window.fetch === "function"`.
- `cargo run -- ui` starts the real desktop window. Browser debug requires `cargo run -- ui-browser` then open `http://localhost:{port}/?route=home`.
- Developer console log sequences: backend (1..99999) and frontend (100000..) use separate ranges to avoid collision.
- API interceptor in dev console skips `/api/developer-logs` and `/api/developer-log` to prevent loops.
- UI tests mock `globalThis.window`, `document`, `localStorage` in `tests/routes.test.js`.
- MSI builds use static CRT linking via `.cargo/config.toml` (`+crt-static`).

## Build prerequisites

- Linux: `libgtk-3-dev libwebkit2gtk-4.1-dev`
- Windows: WebView2 Runtime, WiX 3.14 for MSI
- Rust stable with `rustfmt` component
