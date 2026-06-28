# Worm Forensic Tool - Release Notes v0.0.11

This release brings Volatility3 RAM analysis for Windows and Linux, remote Android acquisition over MESH/TCP, an embedded developer console, and Linux/Windows packaging improvements.

---

## 🇹🇷 Türkçe Sürüm Notları

### Geliştirmeler ve Hata Düzeltmeleri

* **Volatility3 RAM Analizi:**
  * Volatility3 entegrasyonu ile Windows ve Linux RAM analizi desteği eklendi.
  * Volatility3 Python worker'ı AppImage içinde paketlendi.
  * RAM preflight kontrolleri (ön kontrol) eklendi.
  * Canlı RAM analiz logları eklendi.
  * Linux Volatility sembol indirici eklendi.
  * Volatility3 hata mesajlarına Türkçe çözüm önerileri eklendi. ✍️ **Katkıda Bulunanlar:** [@favilances](https://github.com/favilances)

* **Android Adli Bilişim:**
  * Uzak Android / MESH transport desteği eklendi (TCP/IP ADB bağlantısı, MESH röle).
  * Lemon eBPF fiziksel RAM ön kontrolü eklendi.
  * Root'suz Android dosya sistemi arşivleme desteği eklendi.
  * Android logical forensic kollektörler genişletildi.
  * Android forensic preflight durumu gösterimi eklendi.
  * Android edinim context manifestleri eklendi.
  * ADB hata yönlendirmeleri standardize edildi.
  * Edinim çıktıları ayrı klasörlere yazılacak şekilde düzenlendi. ✍️ **Katkıda Bulunanlar:** [@favilances](https://github.com/favilances)

* **Developer Konsolu:**
  * Logoya 5 kez tıklayınca açılan developer mod paneli eklendi (canlı log akışı, sistem bilgisi, dışa aktarım).
  * Native WebView ortamında çalışan standalone pencere desteği (`/api/open-dev-console`).
  * API interceptor ile tüm `/api/` istekleri otomatik loglanır (method, status, süre, request/response body).
  * Console.error/warn/log override, window.onerror ve unhandledrejection yakalama.
  * Expandable log detayları (stack trace, payload, süre badge).
  * Sistem bilgisi sekmesi (hostname, username, timezone, RAM, heap, port, env).
  * Runtime log entegrasyonu: disk, RAM, volatility, wireguard ve evidence modüllerinden zengin log. ✍️ **Katkıda Bulunan:** [@favilances](https://github.com/favilances)

* **Derleme ve Paketleme:**
  * Arch Linux paket çıktısı eklendi.
  * Her commit'te yüklenebilir paket yayınlama (CI).
  * MSI WiX düzeltmeleri: ICE43/ICE57 ve ARPNOMODIFY çakışması giderildi.
  * Worm CLI komutları genişletildi.
  * Privileged operasyonlar sağlamlaştırıldı. ✍️ **Katkıda Bulunan:** [@favilances](https://github.com/favilances)

* **Kod Yapısı ve Bakım:**
  * Disk analiz ağacı mount hatasında gösterilir hale getirildi.
  * Windows disk görüntüsü mount fallback iyileştirildi.
  * Detaylı hata rehberliği eklendi.
  * Kod biçimi Rust fmt ile düzenlendi.
  * Türkçe Rust kod yorumları eklendi.
  * Bekleyen vakalar korundu, sahte RAM hedefleri gizlendi.
  * Linux native UI animasyonları optimize edildi. ✍️ **Katkıda Bulunan:** [@favilances](https://github.com/favilances)

---

## 🇬🇧 English Release Notes

### Enhancements and Fixes

* **Volatility3 RAM Analysis:**
  * Added Volatility3 integration for Windows and Linux RAM analysis.
  * Bundled Volatility3 Python worker inside AppImage.
  * Added RAM preflight checks.
  * Added live RAM analysis logs.
  * Added Linux Volatility symbol downloader.
  * Added Turkish-language error resolution suggestions for Volatility3. ✍️ **Contributors:** [@favilances](https://github.com/favilances)

* **Android Forensics:**
  * Added remote Android / MESH transport support (TCP/IP ADB connection, MESH relay).
  * Added Lemon eBPF physical RAM preflight.
  * Added non-root Android filesystem archive support.
  * Expanded Android logical forensic collectors.
  * Added Android forensic preflight status display.
  * Added Android acquisition context manifests.
  * Standardized ADB error guidance.
  * Organized acquisition outputs into separate folders per case. ✍️ **Contributors:** [@favilances](https://github.com/favilances)

* **Developer Console:**
  * Added a developer mode panel accessible by clicking the logo 5 times (live log stream, system info, export).
  * Added standalone window support for native WebView environments (`/api/open-dev-console`).
  * API interceptor auto-logs all `/api/` requests (method, status, duration, request/response body).
  * Console.error/warn/log override, window.onerror and unhandledrejection capture.
  * Expandable log details (stack trace, payload, duration badge).
  * System info tab (hostname, username, timezone, RAM, heap, port, env).
  * Runtime log integration across disk, RAM, volatility, wireguard, and evidence modules. ✍️ **Contributor:** [@favilances](https://github.com/favilances)

* **Build and Packaging:**
  * Added Arch Linux package output.
  * Publishing installable packages on every commit (CI).
  * Fixed MSI WiX issues: ICE43/ICE57 violations and ARPNOMODIFY conflict resolved.
  * Expanded Worm CLI commands.
  * Hardened privileged forensic operations. ✍️ **Contributor:** [@favilances](https://github.com/favilances)

* **Codebase and Maintenance:**
  * Disk analysis tree now shown when mount fails.
  * Improved Windows disk image mount fallback.
  * Added detailed error guidance throughout the app.
  * Code formatted with Rust fmt.
  * Added Turkish Rust code comments.
  * Preserved pending cases and hid fake RAM targets.
  * Optimized Linux native UI animations. ✍️ **Contributor:** [@favilances](https://github.com/favilances)

---

📦 **Downloads:** AppImage, .deb, .rpm, Windows MSI — available on the [Releases](https://github.com/noirlang/worm/releases) page.
