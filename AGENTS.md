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
node --test tests/routes.test.js tests/shortcuts.test.js
```

## CI rules

- `cargo fmt --all -- --check` must pass. Always run `cargo fmt` before committing.
- CI uses `-D warnings` (converted from `RUSTFLAGS`). Fmt + no warnings required.
- `main` dalına veya PR'lara yapılan her push CI testlerini tetikler. Ancak **tam derleme ve prerelease paketleri (AppImage, DEB, RPM, MSI)** yalnızca commit mesajında `[build]` etiketi varsa (veya `workflow_dispatch` manuel tetiklendiğinde) üretilir.

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
| `src/disk.rs` | Disk imaging & control engine |
| `src/sparse/` | Akıllı seyrek blok motoru (NTFS, ext4, XFS, exFAT, FAT) |
| `src/hash.rs` | Çok çekirdekli BLAKE3 SIMD & SHA-256 doğrulama motoru |
| `src/android/` | Android ADB/acquisition modules |
| `src/volatility.rs` | Volatility3 integration |
| `ui/` | Frontend: ES modules, no framework |
| `ui/shortcuts.js` | Klavye kısayolları ve dinamik Shift overlay motoru |
| `ui/developer.js` | Dev console (5x logo click) |
| `tests/routes.test.js` | Frontend module health tests |
| `tests/shortcuts.test.js` | Klavye kısayol sistemi testleri |
| `scripts/` | Linux/Windows build scripts |
| `packaging/` | WiX MSI source |
| `.github/workflows/ci.yml` | CI pipeline definition |
| `SKILL.md` | CLI, Agent ve operasyonel komut referans kılavuzu |

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

---

## Tasarım, Mimari ve Ürün İlkeleri (Product Principles)

### 1. Tasarım Felsefesi ve Sadelik (UI/UX)
- **Amaca Yönelik ve Dikkat Dağıtmayan Arayüz**: Amele bir adli bilişim (forensic) aracıdır. Gösterişli, hantal, dikkati dağıtan animasyonlar veya gereksiz görsel karmaşadan kesinlikle kaçınılmalıdır.
- **Hafiflik & Bağımlılıksızlık**: Frontend tarafında harici framework (React, Vue, Tailwind derleyicileri, npm paketleri, bundler vb.) sokulamaz. Saf vanilya ES modülleri (`ui/*.js`) ve native CSS kullanılır.
- **Native WebView Uyumu**: Uygulama masaüstünde wry / WebKit / WebView2 ile çalışır. `window.open` gibi tarayıcıya özgü pop-up'lar wry'da çalışmaz; tüm pencereler/paneller tek sayfada veya backend üzerinden (`/api/open-dev-console`) yönetilir.
- **Yüksek Kontrast ve Tema Desteği**: Adli bilişim uzmanları karanlık ortamlarda da çalışabildiğinden koyu (dark) ve açık (light) tema tam uyumlu olmalı, kontrast oranları metin ve durum göstergelerinde net korunmalıdır.

### 2. Klavye Kısayol Sistemi (Keyboard Shortcuts Architecture)
Amele'de fare kullanımına mahkûm kalmadan klavye ile yüksek hızda adli operasyon yürütmek birincil tasarım kuralıdır.
- **Shift Overlay İlkesi**: Klavyede `Shift` tuşuna basılı tutulduğunda, ekranda tıklanabilir tüm ana aksiyonların ve araç kartlarının üzerinde kısayol harf/rakam rozetleri (badge) dinamik olarak belirir; tuş bırakıldığında rozetler kaybolur.
- **Hiyerarşi ve Öncelik**:
  - Sayfa içi aksiyonlar (`ui/shortcuts.js`), küresel gezinme rotalarından önceliklidir (örneğin Shift+D sayfada temaya veya özel bir butona atanmışsa sayfa eylemi çalışır).
  - Giriş alanlarında (`input`, `textarea`, `select` vb.) yazım esnasında kısayollar kesinlikle tetiklenmez.
  - Vaka kısayolları (`Shift+C` veya `Shift+V` + rakam), araç çalıştırma (`Shift+1..9` veya tek basımlı rakamlar), radial menü açma (`Shift+M`).
- **Geliştirme Standardı**: Yeni bir sayfa, modal veya buton eklendiğinde uygun `data-shortcut` özniteliği belirlenmeli ve `tests/shortcuts.test.js` test suite'i güncellenip hatasız geçmelidir.

### 3. CLI Her Zaman Olmalı (CLI First & Parity İlkesi)
- **Eksiksiz Eşlik Kuralı**: Arayüze (UI) eklenen **HER BİR** özellik (disk imajı alma, canlı RAM edinimi, hash doğrulama, imaj bağlama/analiz, vaka paketleme, export/import, profil yönetimi vb.) **MUTLAKA** CLI tarafında da (`amele <komut>`) eksiksiz çalışır durumda bulunmalıdır.
- **Headless & Sunucu Uyumu**: Adli bilişim operasyonları çoğunlukla sunucularda veya terminal ortamında grafik arayüz olmadan yürütülür. CLI asla ikinci plana itilemez; UI olmadan da tüm iş akışı CLI ile baştan sona yapılabilmelidir.
- **Makine Okunabilirliği (`--json`)**: Kritik CLI komutları script ve otomasyonlarda kullanılabilmek için `--json` bayrağını desteklemeli ve temiz JSON çıktısı üretmelidir.

### 4. Dokümantasyon Senkronizasyon Kuralı (Repo & Uygulama İçi Docs)
- **Repo Dokümantasyonu (`SKILL.md` & `README.md`)**:
  - Kök dizindeki `SKILL.md` dosyası, yapay zeka ajanları ve geliştiriciler için projenin operasyonel kılavuzudur.
  - Kod tabanına yeni bir CLI komutu, bayrak, edinim yöntemi, algoritma veya mimari eklendiğinde/değiştiğinde, **`SKILL.md` (ve gerekirse `README.md`) aynı işlem/PR kapsamında güncellenmek zorundadır**.
- **Uygulama İçi Dokümantasyon (`ui/` İçi Yardım, Rehber ve İpuçları)**:
  - Yapılan büyük eklemelerde ve arayüze eklenen yeni özelliklerde, yalnızca repodaki markdown dosyaları değil, **aynı zamanda uygulamanın kendi içindeki dokümantasyon, yardım sayfaları/menüleri (`ui/pages/help.js` vb.), araç rehberleri ve ipuçları da** eşzamanlı olarak güncellenmelidir.
- **Eksiksiz Tamamlanma İlkesi**:
  - Hem repo dokümantasyonu (`SKILL.md`) hem de uygulamanın kendi içindeki dokümantasyon güncellenmemiş veya CLI karşılığı yazılmamış hiçbir özellik tamamlanmış sayılamaz ve PR açılamaz.

---

## Agent, Dal (Branch) & PR Kuralları (Kesin Kurallar)

Bu proje birden fazla geliştirici ve yapay zeka ajanları ile ortak yürütülmektedir. Projede çalışan tüm geliştiriciler ve yapay zeka ajanları aşağıdaki kurallara **istisnasız** uymak zorundadır:

### 1. Ajanların (AI Agents) Görevi ve Sınırları
- Yapay zeka ajanları kod tabanında araştırma yapma, hata çözme, yeni özellik geliştirme, test yazma ve formatlama işlerinde geliştiricilere eşlik eder.
- Ajanlar kullanıcıdan habersiz veya izinsiz kritik yapılandırmaları değiştiremez, diskleri biçimlendiremez veya doğrudan uzak sunucuya yetkisiz müdahalede bulunamaz.
- Ajanlar kod tabanının mevcut mimarisine (Rust 2024 edition, wry/WebKit, vanilla ES modules), tasarım ilkelerine (sadelik, klavye kısayolları, CLI eşliği) ve Türkçe kod içi yorum standartlarına tam sadık kalır.

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
- **`[build]` Etiketi Kuralı**:
  - Her commit'e veya rastgele commit'lere `[build]` etiketi **KESİNLİKLE ATILMAZ**.
  - `[build]` etiketi **YALNIZCA VE YALNIZCA** kullanıcı açıkça tam derleme ve paket çıktısı (AppImage, DEB, RPM, MSI) üretilmesini istediğini belirttiğinde commit mesajına eklenir.
  - Normal geliştirme, refactor, dokümantasyon veya hata düzeltme commit'lerinde `[build]` kullanılmaz; aksi halde CI gereksiz yere uzun süren paketleme işleriyle kilitlenir.
- Örnek: `feat: [build] exfat sparse tarama destegi eklendi` (kullanıcı build istediyse), `fix: baglanti kopma sorunu duzeltildi` (normal).
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
- **Dil**: Kesinlikle Türkçe.
- **Biçim ve Ton**: Samimi, küçük harf ağırlıklı, mantığı düz ve basit anlatan geliştirici tarzı.
- **Sadelik ve Netlik**:
  - Aşırı resmi, bürokratik, tumturaklı ifadelerden ve yapay zeka çeviri jargonu kokan metinlerden kaçınılmalıdır.
  - Kodun "ne" yaptığını bariz şekilde papağan gibi tekrar etmek yerine (örn: `// i degiskenini bir artiriyoruz`), "neden" o şekilde yazıldığını, arkasındaki mantığı veya dikkat edilmesi gereken bir püf noktayı açıklayın.
- Örnekler:
  - `// docker icinde browser acilmasin diye kontrol ediyoruz`
  - `// ram ediniminde hash tek geciste sha256 ve blake3 olarak cift hesaplaniyor`
  - `// baglanti hemen kapanip porta kilit atmasin diye linger kapatiyoruz`

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

