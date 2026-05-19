# Android Adli Bilişim Modülü / Android Forensic Module

---

## 🇹🇷 Türkçe

### Aşamalar

| Aşama | Yetki | Durum | Açıklama |
|-------|-------|-------|----------|
| **Mantıksal İmaj** | Root gerektirmez | ✅ Aktif | ADB ile erişilebilen tüm veriler |
| **Dosya Sistemi İmajı** | Root / exploit | 🔜 Planlanıyor | Protected alanlar dahil dosya sistemi |
| **Fiziksel İmaj** | Bootloader / EDL | 🔜 Planlanıyor | En düşük seviye bit-by-bit imaj |

### Mantıksal İmaj — Toplanan Veriler

ADB ve USB hata ayıklama açıkken, root yetkisi **olmadan** toplanan veriler:

#### Sistem Bilgileri

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `device_info` | `device_info.txt` | `adb shell getprop` | Cihaz modeli, üretici, Android sürümü, seri no, IMEI, build numarası vb. |
| `packages` | `packages.txt` | `adb shell pm list packages -f` | Yüklü tüm uygulamalar ve APK yolları |
| `processes` | `processes.txt` | `adb shell ps -A` | Çalışan süreçler listesi |
| `disk_usage` | `disk_usage.txt` | `adb shell df -h` | Disk bölümleri ve kullanım durumu |
| `logcat` | `logcat.txt` | `adb logcat -d` | Sistem logları (buffer boyutu kadar) |

#### Dumpsys Servisleri

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `dumpsys_battery` | `dumpsys_battery.txt` | `adb shell dumpsys battery` | Pil durumu, şarj seviyesi, sıcaklık |
| `dumpsys_wifi` | `dumpsys_wifi.txt` | `adb shell dumpsys wifi` | Bağlı/kayıtlı Wi-Fi ağları, SSID'ler, MAC adresleri |
| `dumpsys_bluetooth` | `dumpsys_bluetooth.txt` | `adb shell dumpsys bluetooth_manager` | Eşleşmiş Bluetooth cihazları |
| `dumpsys_usagestats` | `dumpsys_usagestats.txt` | `adb shell dumpsys usagestats` | Uygulama kullanım istatistikleri |
| `dumpsys_account` | `dumpsys_account.txt` | `adb shell dumpsys account` | Cihazda oturum açmış hesaplar |
| `dumpsys_connectivity` | `dumpsys_connectivity.txt` | `adb shell dumpsys connectivity` | Ağ bağlantı durumu, VPN, mobil veri |
| `dumpsys_notification` | `dumpsys_notification.txt` | `dumpsys notification --noredact` + `cmd notification dump_history` | Bildirim geçmişi — `--noredact` ile SMS içerikleri ve mesaj metinleri tam olarak yakalanır; cihazda bildirim geçmişi etkinse tam geçmiş de eklenir |

#### Ağ Bilgileri

| Adım | Dosya | ADB Komutları | Açıklama |
|------|-------|---------------|----------|
| `network_info` | `network_info.txt` | `ip addr`, `ip route`, `netstat -tlnp`, `ip neigh` | IP adresleri, yönlendirme tablosu, açık portlar, ARP tablosu |

#### Ekran Görüntüsü

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `screenshot` | `screenshot.png` | `adb shell screencap -p` | İmaj alma anındaki ekran görüntüsü |

#### Uygulama Medyaları

Root olmadan `/sdcard/Android/media/<paket>/` altındaki medya dosyalarına erişilebilir.

| Adım | Hedef Klasör | Kaynak Yol | Açıklama |
|------|-------------|------------|----------|
| `whatsapp_media` | `whatsapp_media/` | `/sdcard/Android/media/com.whatsapp/` | WhatsApp medya dosyaları (fotoğraf, video, sesli mesaj, belge) |
| `telegram_media` | `telegram_media/` | `/sdcard/Android/media/org.telegram.messenger/` | Telegram medya dosyaları |
| `app_media` | `app_media/` | Birden fazla uygulama | Diğer uygulamaların medyaları |

**`app_media` adımında taranan uygulamalar:**

| Uygulama | Paket Adı | Çıktı Klasörü |
|----------|-----------|---------------|
| WhatsApp Business | `com.whatsapp.w4b` | `whatsapp_business/` |
| Instagram | `com.instagram.android` | `instagram/` |
| Facebook Messenger | `com.facebook.orca` | `messenger/` |
| Viber | `com.viber.voip` | `viber/` |
| Google Messages | `com.google.android.apps.messaging` | `google_messages/` |

#### Tanı Raporu ve Depolama

| Adım | Dosya | Açıklama |
|------|-------|----------|
| `bugreport` | `bugreport-*.zip` | Kapsamlı tanı dosyası (logcat + tüm dumpsys + kernel log + vs.) |
| `shared_storage` | `shared_storage/` | Dahili depolamadaki tüm kullanıcı dosyaları: DCIM, Download, Documents, Pictures, Music vb. |

#### Manifest

| Dosya | Açıklama |
|-------|----------|
| `manifest.json` | Toplanan tüm verilerin özeti: kategori, dosya adı, boyut, başarı durumu |
| `manifest.json.sha256` | Manifest dosyasının SHA-256 hash'i — bütünlük doğrulaması |

### Çıktı Klasör Yapısı

```
~/Worm/Vakalar/{vaka_adı}/android/
├── device_info.txt
├── packages.txt
├── processes.txt
├── disk_usage.txt
├── logcat.txt
├── dumpsys_battery.txt
├── dumpsys_wifi.txt
├── dumpsys_bluetooth.txt
├── dumpsys_usagestats.txt
├── dumpsys_account.txt
├── dumpsys_connectivity.txt
├── dumpsys_notification.txt
├── network_info.txt
├── screenshot.png
├── whatsapp_media/
├── telegram_media/
├── app_media/
├── bugreport-*.zip
├── shared_storage/
├── manifest.json
└── manifest.json.sha256
```

### Root Olmadan Erişilemeyen Veriler

| Veri | Neden |
|------|-------|
| SMS / MMS veritabanı (`mmssms.db`) | Sandbox korumalı |
| Arama kayıtları (`calllog.db`) | Sandbox korumalı |
| Rehber veritabanı (`contacts2.db`) | Content provider izni gerekli |
| WhatsApp mesaj veritabanı (`msgstore.db`) | Şifreli + sandbox |
| Tarayıcı geçmişi (Chrome vb.) | Sandbox korumalı |
| Signal mesajları | Uçtan uca şifreli |
| Silinmiş dosyalar | Fiziksel imaj gerekli |

---

## 🇬🇧 English

### Phases

| Phase | Privilege | Status | Description |
|-------|-----------|--------|-------------|
| **Logical Image** | No root required | ✅ Active | All data accessible via ADB |
| **File System Image** | Root / exploit | 🔜 Planned | File system including protected areas |
| **Physical Image** | Bootloader / EDL | 🔜 Planned | Lowest level bit-by-bit image |

### Logical Image — Collected Data

Data collected via ADB with USB debugging enabled, **without root**:

#### System Information

| Step | File | ADB Command | Description |
|------|------|-------------|-------------|
| `device_info` | `device_info.txt` | `adb shell getprop` | Device model, manufacturer, Android version, serial, IMEI, build number, etc. |
| `packages` | `packages.txt` | `adb shell pm list packages -f` | All installed applications and APK paths |
| `processes` | `processes.txt` | `adb shell ps -A` | Running processes list |
| `disk_usage` | `disk_usage.txt` | `adb shell df -h` | Disk partitions and usage |
| `logcat` | `logcat.txt` | `adb logcat -d` | System logs (buffer size) |

#### Dumpsys Services

| Step | File | ADB Command | Description |
|------|------|-------------|-------------|
| `dumpsys_battery` | `dumpsys_battery.txt` | `adb shell dumpsys battery` | Battery status, charge level, temperature |
| `dumpsys_wifi` | `dumpsys_wifi.txt` | `adb shell dumpsys wifi` | Connected/saved Wi-Fi networks, SSIDs, MAC addresses |
| `dumpsys_bluetooth` | `dumpsys_bluetooth.txt` | `adb shell dumpsys bluetooth_manager` | Paired Bluetooth devices |
| `dumpsys_usagestats` | `dumpsys_usagestats.txt` | `adb shell dumpsys usagestats` | App usage statistics |
| `dumpsys_account` | `dumpsys_account.txt` | `adb shell dumpsys account` | Accounts signed in on the device |
| `dumpsys_connectivity` | `dumpsys_connectivity.txt` | `adb shell dumpsys connectivity` | Network connection status, VPN, mobile data |
| `dumpsys_notification` | `dumpsys_notification.txt` | `dumpsys notification --noredact` + `cmd notification dump_history` | Notification history — `--noredact` captures full SMS content and message text; if notification history is enabled on the device, the full log is also included |

#### Network Information

| Step | File | ADB Commands | Description |
|------|------|--------------|-------------|
| `network_info` | `network_info.txt` | `ip addr`, `ip route`, `netstat -tlnp`, `ip neigh` | IP addresses, routing table, open ports, ARP table |

#### Screenshot

| Step | File | ADB Command | Description |
|------|------|-------------|-------------|
| `screenshot` | `screenshot.png` | `adb shell screencap -p` | Screenshot at the moment of acquisition |

#### Application Media

Without root, media files in `/sdcard/Android/media/<package>/` are accessible.

| Step | Target Folder | Source Path | Description |
|------|--------------|-------------|-------------|
| `whatsapp_media` | `whatsapp_media/` | `/sdcard/Android/media/com.whatsapp/` | WhatsApp media (photos, videos, voice notes, documents) |
| `telegram_media` | `telegram_media/` | `/sdcard/Android/media/org.telegram.messenger/` | Telegram media files |
| `app_media` | `app_media/` | Multiple apps | Media from other messaging/social apps |

**Apps scanned in `app_media` step:**

| App | Package Name | Output Folder |
|-----|-------------|---------------|
| WhatsApp Business | `com.whatsapp.w4b` | `whatsapp_business/` |
| Instagram | `com.instagram.android` | `instagram/` |
| Facebook Messenger | `com.facebook.orca` | `messenger/` |
| Viber | `com.viber.voip` | `viber/` |
| Google Messages | `com.google.android.apps.messaging` | `google_messages/` |

#### Diagnostic Report & Storage

| Step | File | Description |
|------|------|-------------|
| `bugreport` | `bugreport-*.zip` | Comprehensive diagnostic (logcat + all dumpsys + kernel log + more) |
| `shared_storage` | `shared_storage/` | All user files from internal storage: DCIM, Download, Documents, Pictures, Music, etc. |

#### Manifest

| File | Description |
|------|-------------|
| `manifest.json` | Summary of all collected data: category, filename, size, success status |
| `manifest.json.sha256` | SHA-256 hash of the manifest file — integrity verification |

### Data NOT Accessible Without Root

| Data | Reason |
|------|--------|
| SMS / MMS database (`mmssms.db`) | App sandbox protected |
| Call logs (`calllog.db`) | App sandbox protected |
| Contacts database (`contacts2.db`) | Content provider permission required |
| WhatsApp message database (`msgstore.db`) | Encrypted + sandbox |
| Browser history (Chrome, etc.) | App sandbox protected |
| Signal messages | End-to-end encrypted |
| Deleted files | Physical image required |
