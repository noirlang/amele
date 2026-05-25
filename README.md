
<p align="center">
  <img src="ui/assets/logo/logo.png" alt="Worm Logo" width="120" />
</p>

<p align="center">
  <video src="https://github.com/user-attachments/assets/1b57bf23-cf54-4f0e-b3b3-f745d298ba09" width="700" controls></video>
</p>

<p align="center">
  <a href="https://github.com/noirlang/worm/actions/workflows/ci.yml">
    <img src="https://github.com/noirlang/worm/actions/workflows/ci.yml/badge.svg" alt="Worm CI" />
  </a>
</p>

---

## 🇹🇷 Türkçe

Worm; disk ve RAM edinimi, hash doğrulama, uzak agent yönetimi, vaka çıktıları, imaj görüntüleme ve raporlama için geliştirilen masaüstü adli bilişim aracıdır.

Uygulama Rust backend üzerinde çalışır. Arayüz Linux'ta GTK/WebKit, Windows'ta WebView2 tabanlı gerçek masaüstü penceresi olarak açılır.

### Depolar

| Bileşen | Bağlantı |
| --- | --- |
| Worm | https://github.com/noirlang/worm |
| Linux Agent | https://github.com/noirlang/worm-linux |
| Windows Agent | https://github.com/noirlang/worm-win |
| Web Sitesi | https://worm.noirlang.tr |

### Platform Desteği

| Platform | Destek | Not |
| --- | --- | --- |
| Linux | Desteklenir | GTK/WebKit, AVML, yerel ve uzak edinim |
| Windows | Desteklenir | WebView2, WinPMEM, yerel ve uzak edinim |
| macOS | Desteklenmez | Bu sürümde hedef platform değildir |

### Özellikler

| Alan | Açıklama |
| --- | --- |
| Yerel disk edinimi | Dosya veya blok cihazından bit-by-bit imaj, SHA256 sidecar, `.partial` koruması |
| Uzak disk edinimi | `worm-linux` ve `worm-win` agentları üzerinden raw stream alma |
| Yerel RAM edinimi | Linux AVML ve Windows WinPMEM akışları |
| Uzak RAM edinimi | Agent üzerinde edinim başlatma, ilerleme izleme, dump indirme |
| İş kontrolü | Pause, resume ve stop komutlarının yerel/uzak işlere uygulanması |
| Vaka yönetimi | Sabit `~/Worm/Vakalar` altında vaka klasörleri, notlar, çıktılar ve raporlar |
| Hash | MD5, SHA1, SHA256 ve SHA512 hesaplama |
| İmaj görüntüleme | Linux salt-okunur loop mount, Windows salt-okunur disk mount |
| Android araçları | ADB kontrolü ve Android edinim türleri için modüler araç sayfası |
| Güncelleme | GitHub release kontrolü, paket indirme, SHA256 doğrulama ve installer başlatma |

### İndirme

Release paketleri GitHub Releases ve web sitesi üzerinden dağıtılır:

- Linux AppImage: `worm-linux-x64.AppImage`
- Linux DEB: `worm-linux-x64.deb`
- Linux RPM: `worm-linux-x64.rpm`
- Windows MSI: `worm-windows-x64.msi`

Agent ikilileri:

```text
https://worm.noirlang.tr/worm-linux
https://worm.noirlang.tr/worm-win.exe
```

### Gereksinimler

Linux geliştirme ortamı:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

Rust toolchain:

```bash
rustup toolchain install stable --component rustfmt
rustup default stable
```

Windows'ta yerel pencere için Microsoft Edge WebView2 Runtime gerekir. Windows buildleri Visual C++ runtime bağımlılığını azaltmak için statik MSVC runtime ile üretilir.

### Derleme ve Test

```bash
cargo build --locked
cargo build --release --locked
cargo test --locked
cargo fmt --all -- --check
node --check ui/app.js
```

Linux AppImage üretimi:

```bash
./scripts/build-appimage.sh
```

### Çalıştırma

```bash
cargo run -- ui
./target/release/worm ui
cargo run -- ui-browser
```

Temel CLI kontrolleri:

```bash
cargo run -- settings-default
cargo run -- hash <dosya> sha256
cargo run -- disk-list
cargo run -- disk-size <cihaz-veya-dosya>
cargo run -- verify <imaj> <sha256>
cargo run -- ram-status
cargo run -- remote-disks <ip> <port> [token]
cargo run -- remote-image <ip> <port> <disk_id> <cikti_klasoru> [token]
cargo run -- remote-tool-check <ip> <port> <winpmem|avml> [token]
cargo run -- wireguard-config <dosya>
```

### Uzak Agent Kullanımı

Linux agent:

```bash
wget -O worm-linux https://worm.noirlang.tr/worm-linux
chmod +x worm-linux
./worm-linux
```

Windows agent:

```text
https://worm.noirlang.tr/worm-win.exe
```

Agent bağlantısı uygulama içinden IP, port ve opsiyonel token ile yapılır. Disk/RAM edinimi başlatıldığında çıktı dosyaları seçilen vaka klasörü altında tutulur.

### Çıktı Yapısı

```text
~/Worm/Vakalar/{vaka_adı}/
├── {ip}_{disk}_{tarih}.img
├── {ip}_{disk}_{tarih}.img.sha256
├── {ip}_{disk}_{tarih}.img.md5
├── ram_{tarih}.raw
└── ram_{tarih}.raw.sha256
```

### CI

GitHub Actions her push ve pull request için Linux/Windows testlerini, format kontrolünü, JavaScript söz dizimi kontrolünü ve release build üretimini çalıştırır.

Workflow:

```text
.github/workflows/ci.yml
```

### Katkıda Bulunanlar

Katkıcılar GitHub geçmişinden otomatik gösterilir; README içinde elle kişi listesi tutulmaz.

<a href="https://github.com/noirlang/worm/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=noirlang/worm" alt="Worm GitHub contributors" />
</a>

Katkı ve destek rehberi: [CONTRIBUTORS.md](CONTRIBUTORS.md)

### Güvenlik

Worm yalnızca yetkili adli bilişim süreçlerinde kullanılmalıdır. Gerçek disk veya RAM ediniminden önce hedef sistem, yetki kapsamı ve çıktı konumu doğrulanmalıdır.

---

## 🇬🇧 English

Worm is a desktop forensic tool for disk and memory acquisition, hash verification, remote agent control, case output management, image viewing, and reporting.

The application runs on a Rust backend. The UI opens as a real desktop window: GTK/WebKit on Linux and WebView2 on Windows.

### Repositories

| Component | Link |
| --- | --- |
| Worm | https://github.com/noirlang/worm |
| Linux Agent | https://github.com/noirlang/worm-linux |
| Windows Agent | https://github.com/noirlang/worm-win |
| Website | https://worm.noirlang.tr |

### Platform Support

| Platform | Support | Notes |
| --- | --- | --- |
| Linux | Supported | GTK/WebKit, AVML, local and remote acquisition |
| Windows | Supported | WebView2, WinPMEM, local and remote acquisition |
| macOS | Not supported | Not a target platform for this release |

### Features

| Area | Description |
| --- | --- |
| Local disk acquisition | Bit-by-bit image from file or block device, SHA256 sidecar, `.partial` preservation |
| Remote disk acquisition | Raw stream acquisition through `worm-linux` and `worm-win` agents |
| Local memory acquisition | Linux AVML and Windows WinPMEM flows |
| Remote memory acquisition | Start acquisition on the agent, track progress, download dump |
| Job control | Pause, resume, and stop commands for local and remote jobs |
| Case management | Case folders, notes, outputs, and reports under fixed `~/Worm/Vakalar` |
| Hashing | MD5, SHA1, SHA256, and SHA512 calculation |
| Image viewing | Linux read-only loop mount, Windows read-only disk mount |
| Android tools | Modular Android page for ADB checks and Android acquisition types |
| Updates | GitHub release check, package download, SHA256 verification, installer launch |

### Downloads

Release packages are distributed through GitHub Releases and the website:

- Linux AppImage: `worm-linux-x64.AppImage`
- Linux DEB: `worm-linux-x64.deb`
- Linux RPM: `worm-linux-x64.rpm`
- Windows MSI: `worm-windows-x64.msi`

Agent binaries:

```text
https://worm.noirlang.tr/worm-linux
https://worm.noirlang.tr/worm-win.exe
```

### Requirements

Linux development environment:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

Rust toolchain:

```bash
rustup toolchain install stable --component rustfmt
rustup default stable
```

The Windows native window requires Microsoft Edge WebView2 Runtime. Windows builds use static MSVC runtime linking to reduce Visual C++ runtime dependency issues.

### Build and Test

```bash
cargo build --locked
cargo build --release --locked
cargo test --locked
cargo fmt --all -- --check
node --check ui/app.js
```

Build Linux AppImage:

```bash
./scripts/build-appimage.sh
```

### Run

```bash
cargo run -- ui
./target/release/worm ui
cargo run -- ui-browser
```

Basic CLI checks:

```bash
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

### Remote Agent Usage

Linux agent:

```bash
wget -O worm-linux https://worm.noirlang.tr/worm-linux
chmod +x worm-linux
./worm-linux
```

Windows agent:

```text
https://worm.noirlang.tr/worm-win.exe
```

Agent connections are configured in the application with IP, port, and optional token. Disk and memory acquisition outputs are stored under the selected case folder.

### Output Layout

```text
~/Worm/Vakalar/{case_name}/
├── {ip}_{disk}_{date}.img
├── {ip}_{disk}_{date}.img.sha256
├── {ip}_{disk}_{date}.img.md5
├── ram_{date}.raw
└── ram_{date}.raw.sha256
```

### CI

GitHub Actions runs Linux/Windows tests, formatting checks, JavaScript syntax checks, and release builds on every push and pull request.

Workflow:

```text
.github/workflows/ci.yml
```

### Contributors

Contributors are rendered from GitHub history; no manual person list is kept in the README.

<a href="https://github.com/noirlang/worm/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=noirlang/worm" alt="Worm GitHub contributors" />
</a>

Contribution and support guide: [CONTRIBUTORS.md](CONTRIBUTORS.md)

### Security

Worm must be used only in authorized forensic workflows. Before real disk or memory acquisition, verify the target system, authorization scope, and output location.
