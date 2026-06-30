# Amele UI Prototype

This is a dependency-free frontend shell. It mirrors the original Qt page coverage while following the modern dark forensic visual direction from the supplied screenshots.

Run `cargo run -- ui` from the repository root to open it as a native GTK/WebKit application window with the Rust backend. Opening `index.html` directly is only a static preview and will not expose local Rust APIs.

Implemented UX structure:

- Home dashboard.
- Windows tool hub with four option cards: remote disk, local disk, remote RAM, local RAM.
- Linux tool hub with the same four-option structure.
- Shared acquisition workflow screens with connection, VPN, output, controls, progress, and side status.
- Agent documentation page.
- Analysis page.
- Other page covering hash, evidence, reports, and logs.
- Settings page with theme and update management; dark mode is not on the home/sidebar.
