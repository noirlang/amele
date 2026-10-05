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

- `origin` / `amelenext` → `amele-next/amele-next` (aktif geliştirme reposu)
- `upstream` → `noirlang/amele` (ana repo - geliştirme esnasında push atılmaz, sadece ana sürüm tamamlandıktan sonra aktarılır)


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

## Agent, Dal (Branch) & PR Kuralları (Kesin Kurallar)

Bu proje birden fazla geliştirici ve yapay zeka ajanları ile ortak yürütülmektedir. Projede çalışan tüm geliştiriciler ve yapay zeka ajanları aşağıdaki kurallara **istisnasız** uymak zorundadır:

### 1. Ajanların (AI Agents) Görevi ve Sınırları
- Yapay zeka ajanları kod tabanında araştırma yapma, hata çözme, yeni özellik geliştirme, test yazma ve formatlama işlerinde geliştiricilere eşlik eder.
- Ajanlar kullanıcıdan habersiz veya izinsiz kritik yapılandırmaları değiştiremez, diskleri biçimlendiremez veya doğrudan uzak sunucuya yetkisiz müdahalede bulunamaz.
- Ajanlar kod tabanının mevcut mimarisine (Rust 2024 edition, wry/WebKit, vanilla ES modules) ve Türkçe kod içi yorum standartlarına tam sadık kalır.

### 2. Dal (Branch) Açma ve PR (Pull Request) Zorunluluğu
- **Doğrudan `main` dalına commit veya push atılması kesinlikle YASAKTIR.**
- Yapılacak her iş, düzeltme veya özellik için ilgili amaç doğrultusunda ayrı bir dal açılmalıdır:
  - `feat/<ozellik-adi>`: Yeni özellikler
  - `fix/<hata-adi>`: Hata düzeltmeleri
  - `refactor/<alan-adi>`: Kod düzenlemeleri ve iyileştirmeler
  - `docs/<konu>`: Dokümantasyon güncellemeleri
- İş tamamlandığında değişiklikler açılan dala commit'lenir ve `main` dalına PR olarak açılır.

### 3. PR Onay ve Merge Yetkisi
- Açılan PR'ları inceleme, dokunma, onaylama ve `main` dalına merge etme yetkisi **YALNIZCA** aşağıdaki hesaplara aittir:
  - `@melihemik`
  - `@kafkaskrtl`
- Yetkili hesapların onayı ve incelemesi olmadan hiçbir PR merge edilemez, kapatılamaz veya doğrudan ana dala aktarılamaz.

### 4. Commit Mesajları
- **Dil**: Türkçe.
- **Biçim**: Basit, kısa ve net. Gereksiz yapay zeka süslemesi, kurumsal laf kalabalığı ve akademik ağız kesinlikle yasaktır.
- Standart önekler kullanın (`feat:`, `fix:`, `refactor:`, `chore:`, `docs:`).
- `[build]` etiketi: CI üzerinde tam derleme ve prerelease çıktısı üretilecekse commit mesajına `[build]` eklenir.
- Örnek: `feat: [build] exfat sparse tarama destegi eklendi`, `fix: baglanti kopma sorunu duzeltildi`.
- ❌ Yasak: Emojili veya süslü mesajlar (`✨ feat(core): enterprise grade resilient containerization`).

### 5. PR Başlık ve Açıklamaları
- **PR Başlığı**: Basit, yapılan işi doğrudan anlatan kısa başlık.
- **PR Açıklaması**: Düz, samimi ve sade Türkçe.
- **KESİNLİKLE YASAK**:
  - Robotik AI şablonları (`## 🛠️ Çözülen Sorunlar`, `## 📝 Özet`, `## 🧪 Test Planı` vb.).
  - Emojiler (`🚀`, `✨`, `🎉`, `🔥` vb.).
  - Uzun uzadıya yapay zeka gevezelikleri.
- Ne yapıldıysa maddeler halinde veya birkaç düz cümleyle, bir yazılımcının ekip arkadaşına anlattığı gibi yazılmalıdır.

### 6. Kod İçi Yorum Satırları
- **Dil**: Türkçe.
- **Biçim**: Samimi, küçük harf ağırlıklı, mantığı düz ve basit anlatan geliştirici tarzı.
- Aşırı resmi/bürokratik olmadan mantığı net açıklayın.
- Örnek: `// docker icinde browser acilmasin diye kontrol ediyoruz`

### 7. Git Push & Remote Kuralları (ÖNEMLİ)
- Kullanıcı açıkça onay vermeden ASLA remote repoya push yapılmaz (`git push`).
- Tüm geliştirmeler `amele-next/amele-next` (`origin` / `amelenext`) üzerinde yürütülür.
- `upstream` (`noirlang/amele`) ana reposuna geliştirme sürecinde kesinlikle push gönderilmez. Yalnızca ana sürüm tamamlandıktan sonra sürüm aktarımı yapılır.
- Tüm değişiklikler yerel branch'te tutulur ve kullanıcının kontrolüne bırakılır.

### 8. Kod Formatı ve Testler
Her PR açılmadan önce aşağıdaki kontrollerin yerelde hatasız geçmesi şarttır:
- **Rust formatı**: `cargo fmt --all -- --check` (Düzeltmek için: `cargo fmt`)
- **Rust derleme ve test**: `RUSTFLAGS="-D warnings" cargo test --lib`
- **JavaScript formatı**: `npx prettier --write ui/app.js tests/routes.test.js`
- **Frontend testleri**: `node --check ui/app.js` ve `node --test tests/routes.test.js`
- **Tam entegrasyon paketi**: `bash tests/run_tests.sh`

