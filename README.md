# Worm

<p align="center">
  <img src="ui/assets/logo/logo.png" alt="Worm Logo" width="120" />
</p>

[![Worm CI](https://github.com/noirlang/worm/actions/workflows/ci.yml/badge.svg)](https://github.com/noirlang/worm/actions/workflows/ci.yml)

## Türkçe

Worm; disk/RAM imaj alma, hash doğrulama, uzak agent yönetimi, vaka çıktıları ve raporlama için masaüstü adli bilişim aracıdır.

Uygulama Rust backend ile çalışır, arayüzü yerel pencere içinde açılır ve `worm-linux` / `worm-win` agent protokolüyle uyumludur.

### Depolar

- Ana Worm deposu: https://github.com/noirlang/worm
- Linux Agent: https://github.com/noirlang/worm-linux
- Windows Agent: https://github.com/noirlang/worm-win

### Web Sitesi

- https://worm.noirlang.tr

### Agent İkili İndirme

- Linux binary: `https://worm.noirlang.tr/worm-linux`
- Windows EXE: `https://worm.noirlang.tr/worm-win.exe`

### Durum

Linux ve Windows hedeflenir. macOS desteği yoktur.

Arayüz tarayıcı sekmesi olarak değil, Linux'ta GTK/WebKit ve Windows'ta WebView2 tabanlı yerel pencere olarak açılır.

### Öne Çıkan Özellikler

| Alan | Açıklama |
| --- | --- |
| Yerel disk edinimi | Dosya/blok cihazı kopyalama, SHA256 sidecar, `.partial` koruması |
| Uzak disk edinimi | `imaj_baslat` JSON-over-TCP protokolüyle raw stream alma |
| Yerel RAM edinimi | Linux AVML, Windows WinPMEM helper akışları |
| Uzak RAM edinimi | Agent tarafında edinim başlatma, ilerleme izleme, dump indirme |
| İş kontrolü | Pause/resume/stop komutlarının yerel ve uzak işlere uygulanması |
| Hash | MD5, SHA1, SHA256, SHA512 hesaplama |
| Kanıt | Sabit `~/Worm/Vakalar` altında vaka ağacı, notlar, çıktı klasörleri ve rapor JSON/TXT üretimi |
| İmaj görüntüleme | Linux `mount -o ro,loop` ve partition'lı imajlar için `losetup --partscan`, Windows `Mount-DiskImage` salt-okunur mount |
| Güncelleme | GitHub release kontrolü, paket indirme, SHA256 doğrulama, installer başlatma |
| UI | Vanilla HTML/CSS/JS, Linux GTK/WebKit ve Windows WebView2 yerel pencere |
| CI | Ubuntu/Windows üzerinde test, release build ve binary artifact üretimi |

### Mimari

```text
worm/
├── src/
│   ├── disk.rs          # Yerel disk/dosya imaj alma
│   ├── ram.rs           # AVML / WinPMEM kontrol ve edinim helperları
│   ├── remote.rs        # worm-linux / worm-win JSON-over-TCP client
│   ├── ui_server.rs     # Yerel HTTP API ve UI asset servisi
│   ├── native_window.rs # Linux GTK/WebKit ve Windows WebView2 yerel pencere
│   ├── hash.rs          # Hash hesaplama
│   ├── evidence.rs      # Vaka ve kanıt klasörleri
│   ├── report.rs        # Rapor çıktıları
│   ├── settings.rs      # Varsayılan ayarlar
│   ├── wireguard.rs     # WireGuard config ve wrapper
│   └── main.rs          # CLI ve UI giriş noktası
├── ui/
│   ├── index.html
│   ├── app.js
│   ├── styles.css
│   └── assets/
├── CONTRIBUTORS.md
└── .github/workflows/ci.yml
```

### UI/API Akışı

1. `cargo run -- ui` uygulamayı yerel pencere olarak başlatır.
2. Backend `127.0.0.1` üzerinde geçici bir port açar.
3. UI yalnızca loopback API ile konuşur.
4. Uzak edinimler `worm-linux` / `worm-win` agentlarına JSON-over-TCP ile bağlanır.

### Gereksinimler

Ubuntu/Debian örneği:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

Rust toolchain:

```bash
rustup toolchain install stable --component rustfmt
rustup default stable
```

Windows native pencere için Microsoft Edge WebView2 Runtime gerekir. Windows 10/11 sistemlerde genellikle hazır gelir; eksikse Microsoft Evergreen Runtime kurulmalıdır.

### Derleme

Debug build:

```bash
cargo build --locked
```

Release build:

```bash
cargo build --release --locked
```

Testler:

```bash
cargo test --locked
```

Format kontrolü:

```bash
cargo fmt --all -- --check
```

### Çalıştırma

Yerel pencere olarak açma:

```bash
cargo run -- ui
```

Derlenmiş binary ile açma:

```bash
./target/release/worm ui
```

Tarayıcı debug modu:

```bash
cargo run -- ui-browser
```

### CLI Smoke Komutları

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

### Uzak Agent Kullanımı

Linux Agent:

```bash
wget -O worm-linux https://worm.noirlang.tr/worm-linux
chmod +x worm-linux
./worm-linux
```

Windows Agent:

`worm-win.exe` dosyasını https://worm.noirlang.tr/worm-win.exe adresinden indirin ve yönetici olarak çalıştırın.

Uygulama tarafında:

1. Windows veya Linux araç sayfasını açın.
2. Uzak disk veya uzak RAM akışını seçin.
3. Agent IP/port bilgisini girin.
4. Token kullanıyorsanız anahtarı onaylayın.
5. Bağlan düğmesiyle agent erişimini doğrulayın.
6. Disk taraması veya RAM araç kontrolünü çalıştırın.
7. Çıktı konumunu seçip edinimi başlatın.

### Agent Protokol Uyumluluğu

Rust client, mevcut `worm-linux` ve `worm-win` agentlarının JSON-over-TCP protokolüyle uyumlu çalışır.

Temel komutlar:

- `merhaba`: agent kimlik ve özellik doğrulama.
- `disk_listele`: uzak diskleri listeleme.
- `imaj_baslat`: uzak disk stream başlatma.
- `winpmem_kontrol`: Windows RAM aracı kontrolü.
- `avml_kontrol`: Linux RAM aracı kontrolü.
- `ram_edinim_baslat`: uzak RAM dump üretimi.
- `ram_dosya_indir`: oluşan RAM dump dosyasını indirme.
- `edinim_kontrol`: `pause`, `resume`, `stop` iş kontrolü.

### Edinim Güvenliği ve Çıktı Davranışı

- Başarısız disk transferlerinde hedef dosya `.partial` olarak korunur.
- Tamamlanan disk imajlarında SHA256 sidecar üretilir.
- Uzak stream sırasında stop geldiğinde bağlantı kapatılır; partial dosyanın içine JSON hata satırı yazılmaz.
- RAM edinimlerinde yüzde ilerleme mevcut dosya boyutu veya agent eventleri üzerinden hesaplanır.
- Uzak RAM akışında önce agent dump üretir, ardından aynı iş kimliğiyle dump dosyası indirilir.
- Yerel RAM edinimi root/administrator yetkisi gerektirebilir.

### Paketleme

Release asset adları:

- `worm-windows-x64.msi`
- `worm-linux-x64.AppImage`
- `SHA256SUMS`

CI Linux ve Windows release binary artifact üretir. MSI/AppImage paketleme ayrıca bağlanacaktır.

### GitHub Actions

Workflow: `.github/workflows/ci.yml`

Push ve pull requestlerde:

1. Sistem bağımlılıkları kurulur.
2. Rust stable toolchain hazırlanır.
3. `cargo fmt --all -- --check` çalışır.
4. `node --check ui/app.js` ile UI JavaScript söz dizimi doğrulanır.
5. `cargo test --locked` çalışır.
6. `cargo build --release --locked` çalışır.
7. Linux ve Windows release binary artifact olarak yüklenir.

Actions sayfası:

- https://github.com/noirlang/worm/actions/workflows/ci.yml

### Katkıda Bulunanlar

Katkıcılar README içinde GitHub geçmişinden otomatik gösterilir; elle kişi listesi tutulmaz.

<a href="https://github.com/noirlang/worm/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=noirlang/worm" alt="Worm GitHub contributors" />
</a>

- GitHub contributor grafiği: https://github.com/noirlang/worm/graphs/contributors
- Destek ve katkı rehberi: [CONTRIBUTORS.md](CONTRIBUTORS.md)

### Güvenlik Notu

Worm yalnızca yetkili adli bilişim süreçlerinde kullanılmalıdır. Disk ve RAM edinimi sistem bütünlüğünü, gizliliği ve yasal yetki sınırlarını doğrudan etkiler. Test verisi dışındaki gerçek edinimler için doğru izin, doğru hedef ve doğru çıktı konumu kontrol edilmelidir.

### Roadmap

- AppImage/MSI paketleme hattının release workflow'a bağlanması.
- Uzak agent protokol testlerinin daha geniş mock senaryolarla çoğaltılması.
- Windows raw DD/IMG mount için opsiyonel forensic image driver entegrasyonu.
- macOS bu dalda desteklenmez; hedef platformlar Linux ve Windows'tur.

---

## English

Worm is a desktop forensic tool for disk/RAM imaging, hash verification, remote agent control, case outputs, and reporting.

The application runs on a Rust backend, opens the UI in a native window, and works with the `worm-linux` / `worm-win` agent protocol.

### Repositories

- Main Worm repository: https://github.com/noirlang/worm
- Linux Agent: https://github.com/noirlang/worm-linux
- Windows Agent: https://github.com/noirlang/worm-win

### Website

- https://worm.noirlang.tr

### Agent Binary Downloads

- Linux binary: `https://worm.noirlang.tr/worm-linux`
- Windows EXE: `https://worm.noirlang.tr/worm-win.exe`

### Status

Linux and Windows are targeted. macOS is not supported.

The UI opens as a native application window: GTK/WebKit on Linux and WebView2 on Windows.

### Feature Overview

| Area | Description |
| --- | --- |
| Local disk acquisition | File/block-device copy, SHA256 sidecar, `.partial` preservation |
| Remote disk acquisition | Raw stream through the `imaj_baslat` JSON-over-TCP protocol |
| Local RAM acquisition | Linux AVML and Windows WinPMEM helper flows |
| Remote RAM acquisition | Start acquisition on the agent, track progress, download dump |
| Job control | Pause/resume/stop commands for local and remote jobs |
| Hashing | MD5, SHA1, SHA256, SHA512 |
| Evidence | Case tree, notes, output folders, and JSON/TXT reports under fixed `~/Worm/Vakalar` |
| Image viewing | Linux `mount -o ro,loop` plus `losetup --partscan` for partitioned images, Windows `Mount-DiskImage` read-only mount |
| Update | GitHub release check, package download, SHA256 verification, installer launch |
| UI | Vanilla HTML/CSS/JS served in Linux GTK/WebKit and Windows WebView2 native windows |
| CI | Ubuntu/Windows tests, release build, and binary artifact upload |

### Architecture

```text
worm/
├── src/
│   ├── disk.rs          # Local disk/file imaging
│   ├── ram.rs           # AVML / WinPMEM checks and acquisition helpers
│   ├── remote.rs        # worm-linux / worm-win JSON-over-TCP client
│   ├── ui_server.rs     # Local HTTP API and UI asset server
│   ├── native_window.rs # Linux GTK/WebKit and Windows WebView2 native window
│   ├── hash.rs          # Hash calculation
│   ├── evidence.rs      # Case and evidence folders
│   ├── report.rs        # Report output
│   ├── settings.rs      # Defaults
│   ├── wireguard.rs     # WireGuard config and wrapper
│   └── main.rs          # CLI and UI entry point
├── ui/
│   ├── index.html
│   ├── app.js
│   ├── styles.css
│   └── assets/
├── CONTRIBUTORS.md
└── .github/workflows/ci.yml
```

### UI/API Flow

1. `cargo run -- ui` starts the native app window.
2. The backend opens a temporary `127.0.0.1` port.
3. The UI talks only to the loopback API.
4. Remote acquisition connects to `worm-linux` / `worm-win` over JSON-over-TCP.

### Requirements

Ubuntu/Debian example:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

Rust toolchain:

```bash
rustup toolchain install stable --component rustfmt
rustup default stable
```

The Windows native window requires Microsoft Edge WebView2 Runtime. It is usually present on Windows 10/11; install Microsoft Evergreen Runtime if it is missing.

### Build

Debug build:

```bash
cargo build --locked
```

Release build:

```bash
cargo build --release --locked
```

Tests:

```bash
cargo test --locked
```

Format check:

```bash
cargo fmt --all -- --check
```

### Run

Open as a native application window:

```bash
cargo run -- ui
```

Open from the release binary:

```bash
./target/release/worm ui
```

Browser debug mode:

```bash
cargo run -- ui-browser
```

### CLI Smoke Commands

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

Linux Agent:

```bash
wget -O worm-linux https://worm.noirlang.tr/worm-linux
chmod +x worm-linux
./worm-linux
```

Windows Agent:

Download `worm-win.exe` from https://worm.noirlang.tr/worm-win.exe and run it as Administrator.

Application side:

1. Open the Windows or Linux tools page.
2. Select a remote disk or remote RAM workflow.
3. Enter the agent IP/port.
4. Approve the token if one is used.
5. Verify agent access with Connect.
6. Run disk scan or RAM tool check.
7. Select output location and start acquisition.

### Agent Protocol Compatibility

The Rust client is compatible with the JSON-over-TCP protocol used by the current `worm-linux` and `worm-win` agents.

Core commands:

- `merhaba`: agent identity and capability handshake.
- `disk_listele`: list remote disks.
- `imaj_baslat`: start remote disk stream.
- `winpmem_kontrol`: check Windows RAM acquisition tool.
- `avml_kontrol`: check Linux RAM acquisition tool.
- `ram_edinim_baslat`: produce remote RAM dump.
- `ram_dosya_indir`: download generated RAM dump.
- `edinim_kontrol`: `pause`, `resume`, `stop` job control.

### Acquisition Safety and Output Behavior

- Failed disk transfers preserve the target file as `.partial`.
- Completed disk images produce a SHA256 sidecar.
- When stop is requested during remote streaming, the connection is closed; JSON error lines are not written into partial binary output.
- RAM progress is calculated from output file size or agent progress events.
- Remote RAM first produces a dump on the agent, then downloads it with the same job id.
- Local RAM acquisition may require root/administrator privileges.

### Packaging

Release asset names:

- `worm-windows-x64.msi`
- `worm-linux-x64.AppImage`
- `SHA256SUMS`

CI produces Linux and Windows release binary artifacts. MSI/AppImage packaging will be connected separately.

### GitHub Actions

Workflow: `.github/workflows/ci.yml`

On push and pull requests:

1. System dependencies are installed.
2. Stable Rust toolchain is prepared.
3. `cargo fmt --all -- --check` runs.
4. `node --check ui/app.js` validates the UI JavaScript syntax.
5. `cargo test --locked` runs.
6. `cargo build --release --locked` runs.
7. Linux and Windows release binaries are uploaded as artifacts.

Actions page:

- https://github.com/noirlang/worm/actions/workflows/ci.yml

### Contributors

Contributors are rendered from GitHub history in this README; no manual person list is kept.

<a href="https://github.com/noirlang/worm/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=noirlang/worm" alt="Worm GitHub contributors" />
</a>

- GitHub contributors graph: https://github.com/noirlang/worm/graphs/contributors
- Support and contribution guide: [CONTRIBUTORS.md](CONTRIBUTORS.md)

### Security Note

Worm should be used only in authorized forensic workflows. Disk and RAM acquisition directly affects system integrity, privacy, and legal authorization boundaries. For real acquisitions, verify permission, target selection, and output location before starting.

### Roadmap

- Connect AppImage/MSI packaging to the release workflow.
- Expand remote agent protocol tests with more mock scenarios.
- Optional forensic image driver integration for Windows raw DD/IMG mounts.
- macOS is not supported in this branch; the target platforms are Linux and Windows.
