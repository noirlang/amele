# Repository Guidelines

## Project Structure & Module Organization

This repository contains the Rust implementation of Worm Forensic Tool. Core backend code is in `src/`: acquisition logic lives in `disk.rs`, `ram.rs`, `android.rs`, `remote.rs`, and API handlers are under `src/api/`. The native/web UI is in `ui/`, with route pages in `ui/pages/`, shared icons/i18n in `ui/icons.js` and `ui/i18n.js`, and static assets under `ui/assets/`. Rust unit tests are colocated in source modules; JavaScript route and translation checks live in `tests/`. CI configuration is in `.github/workflows/ci.yml`; packaging helpers are in `scripts/` and `packaging/`.

## Build, Test, and Development Commands

- `cargo build --locked`: build a debug binary using the locked dependency graph.
- `cargo build --release --locked`: produce the optimized release binary.
- `cargo run -- ui`: run the native UI locally.
- `cargo test --locked`: run Rust unit tests.
- `cargo fmt --all -- --check`: verify Rust formatting.
- `node --check ui/app.js`: validate frontend JavaScript syntax used by CI.
- `node --test tests/i18n.test.js` and `node --test tests/routes.test.js`: check translation keys and route module loading.
- `scripts/build-appimage.sh`: build the Linux AppImage when packaging is needed.

## Coding Style & Naming Conventions

Use `rustfmt` defaults for Rust. Prefer explicit, small modules over large mixed-purpose files. Keep Rust names idiomatic: `snake_case` for functions/modules, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants. UI files use plain ES modules; keep route pages in `ui/pages/<name>.js` and bind shared helpers through `ui/app.js` instead of duplicating logic.

## Testing Guidelines

Add Rust tests near the module they validate with `#[cfg(test)]`. Use deterministic fixtures and avoid touching real disks, RAM devices, or privileged system paths in tests. For UI changes, run the JavaScript syntax check and route/i18n tests. Before pushing, run the same checks as CI where possible: formatting, `node --check`, `cargo test --locked`, and release build for touched platform code.

## Commit & Pull Request Guidelines

History uses short imperative or Conventional Commit-style messages, for example `Fix UI route rendering helper bindings`, `feat: ...`, or `ux(case): ...`. Keep commits scoped to one behavior. Pull requests should include a concise summary, test results, linked issues when applicable, and screenshots for visible UI changes.

## Security & Configuration Tips

Do not commit generated artifacts from `target/`, `dist/`, AppImages, raw images, memory dumps, or `agent.md`. Treat forensic outputs and case data as sensitive. Privileged operations must go through existing helper flows rather than ad hoc shell commands.
