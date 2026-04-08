# Worm

![Worm Logo](logo/logo.png)


## Turkce

Worm, dijital adli bilisim sureclerinde disk/RAM edinimi, dogrulama, vaka yonetimi ve raporlamayi tek bir masaustu uygulamasinda toplayan bir platformdur.

### Repositories

- Ana uygulama: https://github.com/noirlang/worm
- Linux Agent: https://github.com/noirlang/worm-linux
- Windows Agent: https://github.com/noirlang/worm-win

### Web Sitesi

- https://worm.noirlang.tr

### Agent Ikili Indirme

- Linux binary: `https://worm.noirlang.tr/worm-linux`
- Windows EXE: `https://worm.noirlang.tr/worm-win.exe`

### Linux'ta Derleme

Gereksinimler (ornek Ubuntu/Debian):

```bash
sudo apt update
sudo apt install -y build-essential meson ninja-build pkg-config libgtk-4-dev libjson-glib-dev libssl-dev qt6-base-dev
```

Derleme:

```bash
meson setup build
meson compile -C build
```

Calistirma:

```bash
./build/worm
```


### Windows'ta Derleme

Windows tarafinda MSYS2 veya benzeri bir ortama Qt6 + Meson + Ninja kurulumu gerekir.

Ornek adimlar:

1. Qt6, Meson ve Ninja kurun.
2. Proje dizinine girin.
3. Asagidaki komutlari calistirin:

```bash
meson setup build
meson compile -C build
```

Uretilen cikti:

- `build/worm.exe`

### Agent Kullanim Ozet

Linux Agent:

```bash
wget -O worm-linux https://worm.noirlang.tr/worm-linux
chmod +x worm-linux
./worm-linux
```

Windows Agent:

```bash
wget -O worm-win.exe https://worm.noirlang.tr/worm-win.exe
```

Ardindan `worm-win.exe` dosyasini Windows'ta yonetici olarak calistirin.

---

## English

Worm is a digital forensics desktop platform for disk/RAM acquisition, integrity verification, case handling, and reporting in one place.

### Repositories

- Main application: https://github.com/noirlang/worm
- Linux Agent: https://github.com/noirlang/worm-linux
- Windows Agent: https://github.com/noirlang/worm-win

### Website

- https://worm.noirlang.tr

### Agent Binary Downloads

- Linux binary: `https://worm.noirlang.tr/worm-linux`
- Windows EXE: `https://worm.noirlang.tr/worm-win.exe`

### Build on Linux

Dependencies (example for Ubuntu/Debian):

```bash
sudo apt update
sudo apt install -y build-essential meson ninja-build pkg-config libgtk-4-dev libjson-glib-dev libssl-dev qt6-base-dev
```

Build:

```bash
meson setup build
meson compile -C build
```

Run:

```bash
./build/worm
```

### Iced UI (Rust) - Preview

An iced-based UI is available in `iced-ui/` as a parallel migration track.

Requirements (Ubuntu/Debian example):

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libglib2.0-dev libjson-glib-dev libssl-dev
curl https://sh.rustup.rs -sSf | sh
```

Build check:

```bash
cd iced-ui
cargo check
```

Run:

```bash
cd iced-ui
cargo run
```

Current iced coverage:

- Disk listing (C backend bridge)
- Local disk acquisition trigger (C backend bridge)
- Remote disk acquisition trigger (C backend bridge)
- Hash calculation and SHA256 compare
- Case creation and evidence listing
- TXT + JSON report generation (C report module)
- Log viewing with level filtering

### Build on Windows

Use MSYS2 (or a similar environment) with Qt6 + Meson + Ninja installed.

Example steps:

1. Install Qt6, Meson, and Ninja.
2. Open the project directory.
3. Run:

```bash
meson setup build
meson compile -C build
```

Expected output:

- `build/worm.exe`

### Agent Quick Usage

Linux Agent:

```bash
wget -O worm-linux https://worm.noirlang.tr/worm-linux
chmod +x worm-linux
./worm-linux
```

Windows Agent:

```bash
wget -O worm-win.exe https://worm.noirlang.tr/worm-win.exe
```

Then run `worm-win.exe` as Administrator.

