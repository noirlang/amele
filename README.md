<div align="center">

<img src="ui/assets/logo/logo.png" alt="Amele Logo" width="120" />

# Amele Forensic Tool

*Windows, Linux, Android, and iOS unified digital forensics platform. Disk, RAM, logical acquisition, and backup analysis.*

[Website](https://amele.noirlang.tr) | [Releases](https://github.com/noirlang/amele/releases) | [Contributing](CONTRIBUTING.md) | [Security](SECURITY.md) | [Linux Agent](agent/README.md) | [Windows Agent](agent/README.md)

<img src="amele.gif" alt="Amele Forensic Tool Demo" width="700" />

</div>

## Overview

Amele is a desktop forensic acquisition tool for authorized investigations. It brings disk imaging, memory acquisition, Android collection, iOS backup normalization, hash verification, case output handling, image viewing, and reporting into one native application.

The app runs as a real desktop window on Linux and Windows.

## Features

- **Local disk acquisition:** create raw disk images from local disks or image files with RAW or AFF4 output formatting.
- **Remote disk acquisition (Agent):** collect disk images through the Amele Linux and Windows agents with live throughput and hashing.
- **Agentless remote acquisition (SSH / WinRM):** acquire full remote physical disks and RAM over OpenSSH / WinRM directly without installing any agent on Linux or Windows targets.
- **Local memory acquisition:** capture RAM with AVML on Linux and WinPMEM on Windows.
- **Remote memory acquisition:** capture remote memory dumps via Agent or Agentless SSH streaming (`/proc/kcore`, AVML, WinPMEM).
- **Output format flexibility:** choose between bit-by-bit RAW (`.raw` / `.img`) or forensically packaged AFF4 (`.aff4`) formats.
- **Android tools:** check ADB, list devices, collect logical data, collect filesystem data, capture volatile data, and analyze Android case outputs.
- **iOS tools:** normalize unencrypted or already decrypted iTunes/Finder backups into a browsable Backup2FS-style file system, with per-file MD5/SHA1/SHA256 CSV logging.
- **Docker & Container DFIR:** audit container escape risks, scan environment variables for exposed secrets/API keys, extract raw configs and logs, package runtime Overlay2 UpperDir drift layers, and stream remote container evidence via Linux Agent.
- **Case management:** store acquisitions, notes, hashes, Android outputs, iOS outputs, reports, and `case_manifest.json` integrity inventories under selected cases.
- **Hashing and verification:** calculate MD5, SHA1, SHA256, and SHA512; generate sidecar hashes and verify file integrity on the fly.
- **Image viewing:** mount supported images read-only for inspection.
- **Reports:** create case reports from collected outputs, notes, and iOS backup metadata summaries.
- **Updates:** check GitHub releases and download platform installers from inside the app.

## Downloads

Release builds are published on GitHub Releases and on the website.

- Linux AppImage: `amele-linux-x64.AppImage`
- Linux DEB: `amele-linux-x64.deb`
- Linux RPM: `amele-linux-x64.rpm`
- Arch Linux package: `amele-linux-x64.pkg.tar.zst`
- Windows MSI: `amele-windows-x64.msi`

Agent binaries:

```text
https://amele.noirlang.tr/amele-linux
https://amele.noirlang.tr/amele-win.exe
```

## Build Requirements

Module documentation:

- [Windows forensic module](docs/windows.md)
- [Linux forensic module](docs/linux.md)
- [Android forensic module](docs/android.md)
- [iOS forensic module](docs/ios.md)
- [Docker forensic module](docs/docker.md)

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
./target/release/amele ui
```

Open the browser-backed debug UI:

```bash
cargo run -- ui-browser
```

## Agents

Run the Linux agent on the target machine:

```bash
wget -O amele-linux https://amele.noirlang.tr/amele-linux
chmod +x amele-linux
./amele-linux
```

Download the Windows agent:

```text
https://amele.noirlang.tr/amele-win.exe
```

Connect to an agent from the app with IP address, port, and optional token.

---

<div align="center">

<img src="ui/assets/logo/logo.png" alt="Amele Logo" width="120" />

# Amele Adli Bilişim Aracı (Forensic Tool)

*Windows, Linux, Android ve iOS için bütünleşik adli bilişim platformu. Disk, RAM, mantıksal edinim ve yedekleme analizi.*

[Web Sitesi](https://amele.noirlang.tr) | [Sürümler](https://github.com/noirlang/amele/releases) | [Katkıda Bulunma](CONTRIBUTING.md) | [Güvenlik](SECURITY.md) | [Linux Ajanı](https://github.com/noirlang/amele-linux) | [Windows Ajanı](https://github.com/noirlang/amele-win)

<img src="amele.gif" alt="Amele Forensic Tool Demo" width="700" />

</div>

## Genel Bakış

Amele, yetkili incelemeler için geliştirilmiş bir masaüstü adli edinim aracıdır. Disk imajı alma, bellek (RAM) edinimi, Android veri toplama, iOS backup normalizasyonu, hash doğrulama, vaka çıktısı yönetimi, imaj görüntüleme ve raporlama özelliklerini tek bir yerel uygulamada bir araya getirir.

Uygulama Linux ve Windows üzerinde gerçek bir masaüstü penceresi olarak çalışır.

## Özellikler

- **Yerel disk edinimi:** Yerel disklerden veya imaj dosyalarından RAW veya AFF4 formatında disk imajları oluşturun.
- **Uzak disk edinimi (Ajan):** Linux ve Windows ajanları (agents) aracılığıyla canlı akış ve hash doğrulama ile uzak disk imajları toplayın.
- **Agentsız uzak edinim (SSH / WinRM):** Hedef sisteme ajan kurmadan OpenSSH veya WinRM bağlantısıyla uzaktan fiziksel disk ve RAM edinimini doğrudan gerçekleştirin.
- **Yerel bellek (RAM) edinimi:** Linux üzerinde AVML ve Windows üzerinde WinPMEM ile RAM bellek kopyasını alın.
- **Uzak bellek (RAM) edinimi:** Ajanlar veya agentsız SSH akışı (`/proc/kcore`, AVML, WinPMEM) üzerinden canlı RAM dökümü alın.
- **Format esnekliği:** Ham bit-by-bit RAW (`.raw` / `.img`) veya adli paketli AFF4 (`.aff4`) edinim formatlarını seçin.
- **Android araçları:** ADB durumunu kontrol edin, cihazları listeleyin, mantıksal veri toplayın, dosya sistemi verisi toplayın, uçucu (volatile) verileri alın ve Android vaka çıktılarını analiz edin.
- **iOS araçları:** Şifresiz veya önceden decrypt edilmiş iTunes/Finder backup klasörlerini Backup2FS düzeninde gezilebilir dosya sistemine dönüştürün; her dosya için MD5/SHA1/SHA256 CSV log üretin.
- **Docker ve Konteyner Adli Bilişimi:** Konteyner kaçış risklerini denetleyin, ortam değişkenlerindeki parola/token sızıntılarını tespit edin, runtime Overlay2 UpperDir drift katmanını arşivleyin ve canlı/uzak konteyner delillerini toplayın.
- **Vaka yönetimi:** Edinimleri, notları, hash değerlerini, Android/iOS çıktılarını, raporları ve `case_manifest.json` bütünlük envanterlerini seçilen vakalar altında saklayın.
- **Hash hesaplama ve doğrulama:** MD5, SHA1, SHA256 ve SHA512 hesaplayın; elde edilen deliller için yan dosya (sidecar) hash dosyaları oluşturun.
- **İmaj görüntüleme:** Desteklenen imajları inceleme amacıyla salt okunur (read-only) olarak bağlayın (mount).
- **Raporlar:** Toplanan çıktılardan, notlardan ve iOS backup metadata özetlerinden vaka raporları oluşturun.
- **Güncellemeler:** Uygulama içerisinden GitHub sürümlerini kontrol edin ve platform yükleyicilerini indirin.

## İndirmeler

Kararlı sürümler GitHub Sürümleri (Releases) sayfasında ve web sitesinde yayınlanmaktadır.

- Linux AppImage: `amele-linux-x64.AppImage`
- Linux DEB: `amele-linux-x64.deb`
- Linux RPM: `amele-linux-x64.rpm`
- Arch Linux Paketi: `amele-linux-x64.pkg.tar.zst`
- Windows MSI: `amele-windows-x64.msi`

Ajan (Agent) ikili dosyaları:

```text
https://amele.noirlang.tr/amele-linux
https://amele.noirlang.tr/amele-win.exe
```

## Derleme Gereksinimleri

Modül dokümantasyonu:

- [Windows adli bilişim modülü](docs/windows.md)
- [Linux adli bilişim modülü](docs/linux.md)
- [Android adli bilişim modülü](docs/android.md)
- [iOS adli bilişim modülü](docs/ios.md)

Stabil Rust araç zincirini (toolchain) kurun:

```bash
rustup toolchain install stable --component rustfmt
rustup default stable
```

Linux geliştirme paketleri:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

Windows derlemeleri, hedef sistemde Microsoft Edge WebView2 Çalışma Zamanı (Runtime) gerektirir.

## Derleme

Geliştirici (Debug) derlemesi:

```bash
cargo build --locked
```

Kararlı (Release) derlemesi:

```bash
cargo build --release --locked
```

Testleri ve kontrolleri çalıştırın:

```bash
cargo test --locked
cargo fmt --all -- --check
node --check ui/app.js
```

Linux AppImage derleme:

```bash
./scripts/build-appimage.sh
```

Linux DEB, RPM ve Arch paketlerini derleme:

```bash
./scripts/build-linux-packages.sh
```

## Çalıştırma

Yerel masaüstü uygulamasını başlatın:

```bash
cargo run -- ui
```

Kararlı ikili dosyayı çalıştırın:

```bash
./target/release/amele ui
```

Tarayıcı tabanlı hata ayıklama arayüzünü (debug UI) açın:

```bash
cargo run -- ui-browser
```

## Ajanlar (Agents)

Hedef makinede Linux ajanını çalıştırın:

```bash
wget -O amele-linux https://amele.noirlang.tr/amele-linux
chmod +x amele-linux
./amele-linux
```

Windows ajanını indirin:

```text
https://amele.noirlang.tr/amele-win.exe
```

IP adresi, port ve isteğe bağlı token ile uygulama içerisinden ajana bağlanın.
