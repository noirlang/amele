<div align="center">

<img src="ui/assets/logo/logo.png" alt="Worm Logo" width="120" />

# Worm Forensic Tool

*Collect digital evidence in one place. Disk, RAM, and Android acquisition.*

[Website](https://worm.noirlang.tr) | [Releases](https://github.com/noirlang/worm/releases) | [Contributing](CONTRIBUTING.md) | [Security](SECURITY.md) | [Linux Agent](https://github.com/noirlang/worm-linux) | [Windows Agent](https://github.com/noirlang/worm-win)

<video src="https://github.com/user-attachments/assets/a2094bcf-3488-4983-b47f-02e240d503bb" width="700" controls></video>

</div>

## Overview

Worm is a desktop forensic acquisition tool for authorized investigations. It brings disk imaging, memory acquisition, Android collection, hash verification, case output handling, image viewing, and reporting into one native application.

The app runs as a real desktop window on Linux and Windows.

## Features

- **Local disk acquisition:** create raw disk images from local disks or image files.
- **Remote disk acquisition:** collect disk images through the Linux and Windows agents.
- **Local memory acquisition:** capture RAM with AVML on Linux and WinPMEM on Windows.
- **Remote memory acquisition:** start, pause, resume, stop, track, and download RAM dumps from agents.
- **Android tools:** check ADB, list devices, collect logical data, collect filesystem data, capture volatile data, and analyze Android case outputs.
- **Case management:** store acquisitions, notes, hashes, Android outputs, and reports under selected cases.
- **Hashing and verification:** calculate MD5, SHA1, SHA256, and SHA512; generate sidecar hashes for acquired evidence.
- **Image viewing:** mount supported images read-only for inspection.
- **Reports:** create case reports from collected outputs and notes.
- **Updates:** check GitHub releases and download platform installers from inside the app.

## Downloads

Release builds are published on GitHub Releases and on the website.

- Linux AppImage: `worm-linux-x64.AppImage`
- Linux DEB: `worm-linux-x64.deb`
- Linux RPM: `worm-linux-x64.rpm`
- Arch Linux package: `worm-linux-x64.pkg.tar.zst`
- Windows MSI: `worm-windows-x64.msi`

Agent binaries:

```text
https://worm.noirlang.tr/worm-linux
https://worm.noirlang.tr/worm-win.exe
```

## Build Requirements

Install the Rust stable toolchain:

```bash
rustup toolchain install stable --component rustfmt
rustup default stable
```

Linux development packages:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

Windows builds require the Microsoft Edge WebView2 Runtime on the target system.

## Build

Debug build:

```bash
cargo build --locked
```

Release build:

```bash
cargo build --release --locked
```

Run tests and checks:

```bash
cargo test --locked
cargo fmt --all -- --check
node --check ui/app.js
```

Build the Linux AppImage:

```bash
./scripts/build-appimage.sh
```

Build Linux DEB, RPM, and Arch packages:

```bash
./scripts/build-linux-packages.sh
```

## Run

Start the native desktop app:

```bash
cargo run -- ui
```

Run the release binary:

```bash
./target/release/worm ui
```

Open the browser-backed debug UI:

```bash
cargo run -- ui-browser
```

## Agents

Run the Linux agent on the target machine:

```bash
wget -O worm-linux https://worm.noirlang.tr/worm-linux
chmod +x worm-linux
./worm-linux
```

Download the Windows agent:

```text
https://worm.noirlang.tr/worm-win.exe
```

Connect to an agent from the app with IP address, port, and optional token.
