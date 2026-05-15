# Worm UI Prototype

This is a dependency-free Tauri-ready frontend shell. It mirrors the original Qt page coverage while following the modern dark forensic visual direction from the supplied screenshots.

Open `index.html` directly for a static preview, or point a future Tauri `frontendDist` to this directory.

Implemented UX structure:

- Home dashboard.
- Windows tool hub with four option cards: remote disk, local disk, remote RAM, local RAM.
- Linux tool hub with the same four-option structure.
- Shared acquisition workflow screens with connection, VPN, output, controls, progress, and side status.
- Agent documentation page.
- Analysis page.
- Other page covering hash, evidence, reports, and logs.
- Settings page with theme and update management; dark mode is not on the home/sidebar.
