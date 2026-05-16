# Worm Rust Rewrite

![Worm Logo](ui/assets/logo/logo.png)

[![Rust Rewrite CI](https://github.com/noirlang/worm/actions/workflows/ci.yml/badge.svg?branch=rewrite-rust)](https://github.com/noirlang/worm/actions/workflows/ci.yml)

## Türkçe

Worm Rust Rewrite, Worm Forensic Tool'un Rust ile yeniden yazılan masaüstü uygulama ve teknik çekirdek çalışmasıdır. Amaç; disk/RAM edinimi, doğrulama, uzak agent haberleşmesi, vaka çıktıları ve raporlama adımlarını tek, denetlenebilir ve paketlenebilir bir uygulama çatısı altında toplamaktır.

Bu depo/branch, eski C/Qt uygulamasındaki çalışma mantığını koruyarak Rust tarafında daha güvenli hata yönetimi, test edilebilir modüller, yerel pencere deneyimi ve `worm-linux` / `worm-win` agent protokolüyle uyumlu uzak edinim akışları sağlar.

### Depolar

- Rust rewrite branch: https://github.com/noirlang/worm/tree/rewrite-rust
- Ana Worm deposu: https://github.com/noirlang/worm
- Linux Agent: https://github.com/noirlang/worm-linux
- Windows Agent: https://github.com/noirlang/worm-win

### Web Sitesi

- https://worm.noirlang.tr

### Agent İkili İndirme

- Linux binary: `https://worm.noirlang.tr/worm-linux`
- Windows EXE: `https://worm.noirlang.tr/worm-win.exe`

### Proje Durumu

Rust rewrite aktif geliştirme aşamasındadır. Mevcut yapı, masaüstü pencere içinde çalışan dependency-free web arayüzünü Rust HTTP backend ile aynı binary üzerinden sunar. Uygulama tarayıcı sekmesi gibi değil, GTK/WebKit tabanlı yerel pencere olarak açılır.

Çalışan ana başlıklar:

- Yerel disk/dosya imaj alma.
- Uzak Linux/Windows agent üzerinden disk imaj alma.
- Yerel AVML/WinPMEM durum kontrolü.
- Yerel AVML/WinPMEM RAM edinimi.
- Uzak Linux/Windows agent üzerinden RAM edinimi ve dump indirme.
- Disk/RAM edinimlerinde yüzde ilerleme.
- Uzak ve yerel işler için duraklat/devam/durdur kontrolü.
- Hash hesaplama ve hash karşılaştırma.
- Vaka klasörü, kanıt kasası, not ve rapor modülleri.
- WireGuard config üretimi ve Linux `wg-quick` wrapper akışı.
- GitHub Actions üzerinden format, test, release build ve artifact doğrulaması.

### Öne Çıkan Özellikler

| Alan | Açıklama |
| --- | --- |
| Yerel disk edinimi | Dosya/blok cihazı kopyalama, SHA256 sidecar, `.partial` koruması |
| Uzak disk edinimi | `imaj_baslat` JSON-over-TCP protokolüyle raw stream alma |
| Yerel RAM edinimi | Linux AVML, Windows WinPMEM helper akışları |
| Uzak RAM edinimi | Agent tarafında edinim başlatma, ilerleme izleme, dump indirme |
| İş kontrolü | Pause/resume/stop komutlarının yerel ve uzak işlere uygulanması |
| Hash | MD5, SHA1, SHA256, SHA512 hesaplama |
| Kanıt | Vaka ağacı, notlar, çıktı klasörleri ve rapor JSON/TXT üretimi |
| UI | Vanilla HTML/CSS/JS, Rust binary tarafından servis edilen yerel pencere |
| CI | Ubuntu üzerinde test, release build ve binary artifact üretimi |

### Mimari

```text
worm-rewrite-rust/
├── src/
│   ├── disk.rs          # Yerel disk/dosya imaj alma
│   ├── ram.rs           # AVML / WinPMEM kontrol ve edinim helperları
│   ├── remote.rs        # worm-linux / worm-win JSON-over-TCP client
│   ├── ui_server.rs     # Yerel HTTP API ve UI asset servisi
│   ├── native_window.rs # GTK/WebKit yerel pencere
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
├── scripts/
│   └── update_contributors.py
├── CONTRIBUTORS.md
└── .github/workflows/ci.yml
```

### Uygulama Akışı

1. `cargo run -- ui` Rust binary'yi başlatır.
2. Binary `127.0.0.1` üzerinde geçici bir portta yerel HTTP API açar.
3. Aynı binary GTK/WebKit pencere başlatır.
4. UI yalnızca loopback API'ye istek atar.
5. Disk/RAM/Hash/Agent işlemleri Rust modüllerine bağlanır.
6. Uzak edinimlerde Rust client agent ile JSON-over-TCP konuşur.
7. Büyük veri transferleri binary stream olarak alınır ve yerel dosyaya yazılır.

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
./target/release/worm-rewrite-rust ui
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

### Paketleme Yönü

Hedef release asset adları ana Worm güncelleme mantığıyla uyumlu tutulmalıdır:

- `worm-windows-x64.msi`
- `worm-linux-x64.AppImage`
- `SHA256SUMS`

Mevcut CI Linux ve Windows release binary artifact üretir. MSI/AppImage installer paketleme işi release hattında ayrıca bağlanacaktır.

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

- https://github.com/noirlang/worm/actions/workflows/ci.yml?query=branch%3Arewrite-rust

### Katkıda Bulunanlar

Katkıcı listesi elle yazılmaz. `CONTRIBUTORS.md` dosyası git geçmişinden otomatik üretilir:

```bash
python3 scripts/update_contributors.py
```

Oluşan dosya:

- [CONTRIBUTORS.md](CONTRIBUTORS.md)

Yeni contributor veya author alias değişikliği olduğunda script çalıştırılıp dosya commitlenmelidir.

### Güvenlik Notu

Worm yalnızca yetkili adli bilişim süreçlerinde kullanılmalıdır. Disk ve RAM edinimi sistem bütünlüğünü, gizliliği ve yasal yetki sınırlarını doğrudan etkiler. Test verisi dışındaki gerçek edinimler için doğru izin, doğru hedef ve doğru çıktı konumu kontrol edilmelidir.

### Roadmap

- AppImage/MSI paketleme hattının release workflow'a bağlanması.
- Uzak agent protokol testlerinin daha geniş mock senaryolarla çoğaltılması.
- Windows tarafında yerel pencere kabuğunun GTK/WebKit dışı bir uygulama penceresine taşınması.
- Salt-okunur imaj mount akışının Linux dışı platformlarda ayrı sürücülerle tamamlanması.

---

## English

Worm Rust Rewrite is the Rust-based desktop application and technical core rewrite of Worm Forensic Tool. The goal is to bring disk/RAM acquisition, verification, remote agent communication, case outputs, and reporting into one auditable and packageable application.

This branch preserves the operating model of the older C/Qt application while adding safer Rust error handling, testable modules, a native window experience, and remote acquisition flows compatible with the `worm-linux` and `worm-win` agents.

### Repositories

- Rust rewrite branch: https://github.com/noirlang/worm/tree/rewrite-rust
- Main Worm repository: https://github.com/noirlang/worm
- Linux Agent: https://github.com/noirlang/worm-linux
- Windows Agent: https://github.com/noirlang/worm-win

### Website

- https://worm.noirlang.tr

### Agent Binary Downloads

- Linux binary: `https://worm.noirlang.tr/worm-linux`
- Windows EXE: `https://worm.noirlang.tr/worm-win.exe`

### Project Status

The Rust rewrite is under active development. The current build serves a dependency-free web UI from the Rust binary and opens it inside a GTK/WebKit native application window. It is intended to behave like an application window, not a normal browser tab.

Working areas:

- Local disk/file imaging.
- Remote Linux/Windows agent disk imaging.
- Local AVML/WinPMEM status checks.
- Local AVML/WinPMEM RAM acquisition.
- Remote Linux/Windows agent RAM acquisition and dump download.
- Percentage progress for disk/RAM acquisition.
- Pause/resume/stop controls for local and remote jobs.
- Hash calculation and hash comparison.
- Case folder, evidence vault, notes, and report modules.
- WireGuard config generation and Linux `wg-quick` wrapper flow.
- GitHub Actions formatting, tests, release build, and artifact verification.

### Feature Overview

| Area | Description |
| --- | --- |
| Local disk acquisition | File/block-device copy, SHA256 sidecar, `.partial` preservation |
| Remote disk acquisition | Raw stream through the `imaj_baslat` JSON-over-TCP protocol |
| Local RAM acquisition | Linux AVML and Windows WinPMEM helper flows |
| Remote RAM acquisition | Start acquisition on the agent, track progress, download dump |
| Job control | Pause/resume/stop commands for local and remote jobs |
| Hashing | MD5, SHA1, SHA256, SHA512 |
| Evidence | Case tree, notes, output folders, and JSON/TXT reports |
| UI | Vanilla HTML/CSS/JS served by the Rust binary in a native window |
| CI | Ubuntu tests, release build, and binary artifact upload |

### Architecture

```text
worm-rewrite-rust/
├── src/
│   ├── disk.rs          # Local disk/file imaging
│   ├── ram.rs           # AVML / WinPMEM checks and acquisition helpers
│   ├── remote.rs        # worm-linux / worm-win JSON-over-TCP client
│   ├── ui_server.rs     # Local HTTP API and UI asset server
│   ├── native_window.rs # GTK/WebKit native window
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
├── scripts/
│   └── update_contributors.py
├── CONTRIBUTORS.md
└── .github/workflows/ci.yml
```

### Application Flow

1. `cargo run -- ui` starts the Rust binary.
2. The binary starts a local HTTP API on a temporary `127.0.0.1` port.
3. The same binary opens a GTK/WebKit application window.
4. The UI talks only to the loopback API.
5. Disk/RAM/hash/agent actions are routed to Rust modules.
6. Remote acquisition uses the Rust client to talk to the agents over JSON-over-TCP.
7. Large transfers are received as binary streams and written to local files.

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
./target/release/worm-rewrite-rust ui
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

### Packaging Direction

Target release asset names must stay compatible with the main Worm updater convention:

- `worm-windows-x64.msi`
- `worm-linux-x64.AppImage`
- `SHA256SUMS`

The current CI produces Linux and Windows release binary artifacts. MSI/AppImage installer packaging will be connected in the release pipeline.

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

- https://github.com/noirlang/worm/actions/workflows/ci.yml?query=branch%3Arewrite-rust

### Contributors

The contributor list is not edited by hand. `CONTRIBUTORS.md` is generated from git history:

```bash
python3 scripts/update_contributors.py
```

Generated file:

- [CONTRIBUTORS.md](CONTRIBUTORS.md)

Run the script and commit the result when contributor history or author aliases change.

### Security Note

Worm should be used only in authorized forensic workflows. Disk and RAM acquisition directly affects system integrity, privacy, and legal authorization boundaries. For real acquisitions, verify permission, target selection, and output location before starting.

### Roadmap

- Connect AppImage/MSI packaging to the release workflow.
- Expand remote agent protocol tests with more mock scenarios.
- Move the Windows native shell to an application window outside the GTK/WebKit Linux path.
- Complete read-only image mount drivers for non-Linux platforms.
