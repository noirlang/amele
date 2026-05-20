# Android Adli Bilişim Modülü / Android Forensic Module

---

## 🇹🇷 Türkçe

### Aşamalar

| Aşama | Yetki | Durum | Açıklama |
|-------|-------|-------|----------|
| **Mantıksal İmaj** | Root gerektirmez | ✅ Aktif | ADB ile erişilebilen tüm veriler |
| **Dosya Sistemi İmajı** | Root / exploit | 🔜 Planlanıyor | Protected alanlar dahil dosya sistemi |
| **Fiziksel İmaj** | Bootloader / EDL | 🔜 Planlanıyor | En düşük seviye bit-by-bit imaj |

### Mantıksal İmaj — Toplanan Veriler (31 Adım)

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
| `dumpsys_battery` | `dumpsys_battery.txt` | `dumpsys battery` | Pil durumu, şarj seviyesi, sıcaklık |
| `dumpsys_wifi` | `dumpsys_wifi.txt` | `dumpsys wifi` | Bağlı/kayıtlı Wi-Fi ağları, SSID'ler, MAC adresleri |
| `dumpsys_bluetooth` | `dumpsys_bluetooth.txt` | `dumpsys bluetooth_manager` | Eşleşmiş Bluetooth cihazları |
| `dumpsys_usagestats` | `dumpsys_usagestats.txt` | `dumpsys usagestats` | Uygulama kullanım istatistikleri (hangi uygulama ne kadar açıldı, süre) |
| `dumpsys_account` | `dumpsys_account.txt` | `dumpsys account` | Cihazda oturum açmış hesaplar (Google, Samsung, vb.) |
| `dumpsys_connectivity` | `dumpsys_connectivity.txt` | `dumpsys connectivity` | Ağ bağlantı durumu, VPN, mobil veri |
| `dumpsys_notification` | `dumpsys_notification.txt` | `dumpsys notification --noredact` + `cmd notification dump_history` | Bildirim geçmişi — SMS içerikleri ve mesaj metinleri tam olarak yakalanır |
| `dumpsys_telephony` | `dumpsys_telephony.txt` | `dumpsys telephony.registry` | SIM kart bilgileri, sinyal gücü, operatör, telefon numarası, IMSI |
| `dumpsys_location` | `dumpsys_location.txt` | `dumpsys location` | Konum sağlayıcıları, son konum verileri, geofence kuralları |
| `dumpsys_netstats` | `dumpsys_netstats.txt` | `dumpsys netstats` | Uygulama bazlı ağ kullanım istatistikleri (MB/GB indirme/yükleme) |
| `dumpsys_activity` | `dumpsys_activity.txt` | `dumpsys activity` | Çalışan activity'ler, son görevler, intent geçmişi |
| `dumpsys_meminfo` | `dumpsys_meminfo.txt` | `dumpsys meminfo` | Uygulama bazlı bellek kullanımı (PSS, heap, dalvik) — root olmadan tam dump alınamaz ama süreç bellek profili çıkarılır |
| `dumpsys_appops` | `dumpsys_appops.txt` | `dumpsys appops` | Uygulama izin kullanım geçmişi (kamera, mikrofon, konum erişimleri) |

#### Cihaz Ayarları

| Adım | Dosya | ADB Komutları | Açıklama |
|------|-------|---------------|----------|
| `device_settings` | `device_settings.txt` | `settings list system/secure/global` | Tüm cihaz ayarları: ekran parlaklığı, varsayılan uygulamalar, geliştirici seçenekleri, Google hesap ID'leri, erişilebilirlik servisleri vb. |

#### Content Provider Sorguları

Modern Android'de bu sorgular izin kısıtlamalarıyla başarısız olabilir ancak denenir. Eski cihazlarda veya özel durumlarda değerli veri sağlarlar.

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `content_sms` | `content_sms.txt` | `content query --uri content://sms` | SMS mesajları (gönderen, alıcı, tarih, içerik) |
| `content_calls` | `content_calls.txt` | `content query --uri content://call_log/calls` | Arama kayıtları (numara, süre, tarih, tür) |
| `content_contacts` | `content_contacts.txt` | `content query --uri content://contacts/phones` | Rehber (isim, numara) |

#### Ağ Bilgileri

| Adım | Dosya | ADB Komutları | Açıklama |
|------|-------|---------------|----------|
| `network_info` | `network_info.txt` | `ip addr`, `ip route`, `netstat -tlnp`, `ip neigh` | IP adresleri, yönlendirme tablosu, açık portlar, ARP tablosu |

#### Ekran Görüntüsü

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `screenshot` | `screenshot.png` | `adb shell screencap -p` | İmaj alma anındaki ekran görüntüsü |

#### Uygulama Medyaları (Bilinen Uygulamalar)

Root olmadan `/sdcard/Android/media/<paket>/` altındaki medya dosyalarına erişilebilir.

| Adım | Hedef Klasör | Kaynak Paketler | Açıklama |
|------|-------------|-----------------|----------|
| `whatsapp_media` | `whatsapp_media/` | `com.whatsapp` | WhatsApp medya dosyaları |
| `telegram_media` | `telegram_media/` | `org.telegram.messenger` | Telegram medya dosyaları |
| `app_media` | `app_media/` | 26 farklı uygulama | Bilinen uygulamaların medyaları |

**`app_media` adımında taranan uygulamalar (26 adet):**

| Uygulama | Paket Adı | Çıktı Klasörü |
|----------|-----------|---------------|
| WhatsApp Business | `com.whatsapp.w4b` | `whatsapp_business/` |
| Instagram | `com.instagram.android` | `instagram/` |
| Facebook Messenger | `com.facebook.orca` | `messenger/` |
| Facebook | `com.facebook.katana` | `facebook/` |
| Viber | `com.viber.voip` | `viber/` |
| Google Messages | `com.google.android.apps.messaging` | `google_messages/` |
| X (Twitter) | `com.twitter.android` | `x_twitter/` |
| Snapchat | `com.snapchat.android` | `snapchat/` |
| TikTok | `com.zhiliaoapp.musically` | `tiktok/` |
| TikTok (alternatif) | `com.ss.android.ugc.trill` | `tiktok_alt/` |
| Discord | `com.discord` | `discord/` |
| LinkedIn | `com.linkedin.android` | `linkedin/` |
| Pinterest | `com.pinterest` | `pinterest/` |
| Reddit | `com.reddit.frontpage` | `reddit/` |
| Spotify | `com.spotify.music` | `spotify/` |
| Signal | `org.thoughtcrime.securesms` | `signal/` |
| Skype | `com.skype.raider` | `skype/` |
| Zoom | `us.zoom.videomeetings` | `zoom/` |
| Microsoft Teams | `com.microsoft.teams` | `teams/` |
| BiP (Turkcell) | `com.turkcell.bip` | `bip/` |
| Wire | `com.wire` | `wire/` |
| Telegram Plus | `org.telegram.plus` | `telegram_plus/` |
| KakaoTalk | `com.kakao.talk` | `kakaotalk/` |
| LINE | `jp.naver.line.android` | `line/` |
| WeChat | `com.tencent.mm` | `wechat/` |
| IMO | `com.imo.android.imoim` | `imo/` |

#### Dinamik Uygulama Medya Tarama

| Adım | Hedef Klasör | Açıklama |
|------|-------------|----------|
| `all_app_media` | `all_app_media/` | `/sdcard/Android/media/` altındaki **TÜM** paketler dinamik olarak taranır ve çekilir. Yukarıdaki sabit listeye ek olarak, cihazdaki her uygulamayı otomatik yakalar. |

#### ADB Backup

| Adım | Dosya | Açıklama |
|------|-------|----------|
| `adb_backup` | `adb_backup.ab` | `adb backup -all -shared -nosystem` — `allowBackup=true` olan uygulamaların verilerini çeker. Modern Android'de deprecated ama eski cihazlarda değerli. |

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

### Bellek (RAM) Dökümü Hakkında

> ⚠️ Root yetkisi olmadan Android'de **tam bellek (RAM) dökümü almak mümkün değildir**. Android kernel'i `/proc/*/mem` ve `/dev/mem` erişimini kısıtlar. `dumpsys meminfo` komutu bellek **istatistiklerini** (her uygulamanın heap, PSS, RSS kullanımı) sağlar — tam bellek içeriğini değil. Tam RAM dökümü için cihazın rootlanması veya özel forensic araçlarının kullanılması gerekir.

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
├── dumpsys_telephony.txt
├── dumpsys_location.txt
├── dumpsys_netstats.txt
├── dumpsys_activity.txt
├── dumpsys_meminfo.txt
├── dumpsys_appops.txt
├── device_settings.txt
├── network_info.txt
├── content_sms.txt
├── content_calls.txt
├── content_contacts.txt
├── screenshot.png
├── whatsapp_media/
├── telegram_media/
├── app_media/
│   ├── instagram/
│   ├── x_twitter/
│   ├── snapchat/
│   ├── tiktok/
│   ├── discord/
│   ├── facebook/
│   ├── messenger/
│   ├── signal/
│   └── ... (26 uygulama)
├── all_app_media/
│   └── ... (dinamik olarak taranan tüm paketler)
├── adb_backup.ab
├── bugreport-*.zip
├── shared_storage/
├── manifest.json
└── manifest.json.sha256
```

### Root Olmadan Erişilemeyen Veriler

| Veri | Neden |
|------|-------|
| SMS / MMS veritabanı (`mmssms.db`) | Sandbox korumalı (content provider denenebilir) |
| Arama kayıtları (`calllog.db`) | Sandbox korumalı (content provider denenebilir) |
| Rehber veritabanı (`contacts2.db`) | Content provider izni gerekli |
| WhatsApp mesaj veritabanı (`msgstore.db`) | Şifreli + sandbox |
| Telegram kayıtlı numara / mesaj DB | Sandbox korumalı |
| Tarayıcı geçmişi (Chrome vb.) | Sandbox korumalı |
| Signal mesajları | Uçtan uca şifreli |
| Tam RAM dökümü | Kernel kısıtlaması, root gerekli |
| Silinmiş dosyalar | Fiziksel imaj gerekli |

---

## 🇬🇧 English

### Phases

| Phase | Privilege | Status | Description |
|-------|-----------|--------|-------------|
| **Logical Image** | No root required | ✅ Active | All data accessible via ADB |
| **File System Image** | Root / exploit | 🔜 Planned | File system including protected areas |
| **Physical Image** | Bootloader / EDL | 🔜 Planned | Lowest level bit-by-bit image |

### Logical Image — Collected Data (31 Steps)

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
| `dumpsys_battery` | `dumpsys_battery.txt` | `dumpsys battery` | Battery status, charge level, temperature |
| `dumpsys_wifi` | `dumpsys_wifi.txt` | `dumpsys wifi` | Connected/saved Wi-Fi networks, SSIDs, MAC addresses |
| `dumpsys_bluetooth` | `dumpsys_bluetooth.txt` | `dumpsys bluetooth_manager` | Paired Bluetooth devices |
| `dumpsys_usagestats` | `dumpsys_usagestats.txt` | `dumpsys usagestats` | App usage statistics (which app opened, duration, frequency) |
| `dumpsys_account` | `dumpsys_account.txt` | `dumpsys account` | Accounts signed in on the device (Google, Samsung, etc.) |
| `dumpsys_connectivity` | `dumpsys_connectivity.txt` | `dumpsys connectivity` | Network connection status, VPN, mobile data |
| `dumpsys_notification` | `dumpsys_notification.txt` | `dumpsys notification --noredact` + `cmd notification dump_history` | Notification history — full SMS content and message text captured |
| `dumpsys_telephony` | `dumpsys_telephony.txt` | `dumpsys telephony.registry` | SIM card info, signal strength, carrier, phone number, IMSI |
| `dumpsys_location` | `dumpsys_location.txt` | `dumpsys location` | Location providers, last known locations, geofence rules |
| `dumpsys_netstats` | `dumpsys_netstats.txt` | `dumpsys netstats` | Per-app network usage statistics (MB/GB download/upload) |
| `dumpsys_activity` | `dumpsys_activity.txt` | `dumpsys activity` | Running activities, recent tasks, intent history |
| `dumpsys_meminfo` | `dumpsys_meminfo.txt` | `dumpsys meminfo` | Per-app memory usage (PSS, heap, dalvik) — not a full RAM dump but process memory profiling |
| `dumpsys_appops` | `dumpsys_appops.txt` | `dumpsys appops` | App permission usage history (camera, microphone, location access records) |

#### Device Settings

| Step | File | ADB Commands | Description |
|------|------|--------------|-------------|
| `device_settings` | `device_settings.txt` | `settings list system/secure/global` | All device settings: brightness, default apps, developer options, Google account IDs, accessibility services, etc. |

#### Content Provider Queries

On modern Android these queries may fail due to permission restrictions, but they are attempted. On older devices or special configurations they yield valuable data.

| Step | File | ADB Command | Description |
|------|------|-------------|-------------|
| `content_sms` | `content_sms.txt` | `content query --uri content://sms` | SMS messages (sender, recipient, date, body) |
| `content_calls` | `content_calls.txt` | `content query --uri content://call_log/calls` | Call log (number, duration, date, type) |
| `content_contacts` | `content_contacts.txt` | `content query --uri content://contacts/phones` | Contacts (name, phone number) |

#### Network Information

| Step | File | ADB Commands | Description |
|------|------|--------------|-------------|
| `network_info` | `network_info.txt` | `ip addr`, `ip route`, `netstat -tlnp`, `ip neigh` | IP addresses, routing table, open ports, ARP table |

#### Screenshot

| Step | File | ADB Command | Description |
|------|------|-------------|-------------|
| `screenshot` | `screenshot.png` | `adb shell screencap -p` | Screenshot at the moment of acquisition |

#### Application Media (Known Apps — 26 apps)

Without root, media files in `/sdcard/Android/media/<package>/` are accessible.

| Step | Target Folder | Source Package | Description |
|------|--------------|----------------|-------------|
| `whatsapp_media` | `whatsapp_media/` | `com.whatsapp` | WhatsApp media (photos, videos, voice notes, documents) |
| `telegram_media` | `telegram_media/` | `org.telegram.messenger` | Telegram media files |
| `app_media` | `app_media/` | 26 apps | Media from social/messaging apps |

**Apps scanned in `app_media` step (26 apps):**

WhatsApp Business, Instagram, Facebook Messenger, Facebook, Viber, Google Messages, X (Twitter), Snapchat, TikTok (2 variants), Discord, LinkedIn, Pinterest, Reddit, Spotify, Signal, Skype, Zoom, Microsoft Teams, BiP (Turkcell), Wire, Telegram Plus, KakaoTalk, LINE, WeChat, IMO.

#### Dynamic App Media Scan

| Step | Target Folder | Description |
|------|--------------|-------------|
| `all_app_media` | `all_app_media/` | Dynamically scans **ALL** packages under `/sdcard/Android/media/` and pulls everything found. Catches apps not in the hardcoded list above. |

#### ADB Backup

| Step | File | Description |
|------|------|-------------|
| `adb_backup` | `adb_backup.ab` | `adb backup -all -shared -nosystem` — extracts data from apps with `allowBackup=true`. Deprecated on modern Android but valuable on older devices. |

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

### About Memory (RAM) Dumps

> ⚠️ A **full memory (RAM) dump is NOT possible without root** on Android. The kernel restricts access to `/proc/*/mem` and `/dev/mem`. The `dumpsys meminfo` command provides memory **statistics** (heap, PSS, RSS usage per app) — not actual memory contents. For a full RAM dump, the device must be rooted or specialized forensic tools must be used.

### Data NOT Accessible Without Root

| Data | Reason |
|------|--------|
| SMS / MMS database (`mmssms.db`) | App sandbox (content provider attempted) |
| Call logs (`calllog.db`) | App sandbox (content provider attempted) |
| Contacts database (`contacts2.db`) | Content provider permission required |
| WhatsApp message database (`msgstore.db`) | Encrypted + sandbox |
| Telegram registered number / message DB | App sandbox protected |
| Browser history (Chrome, etc.) | App sandbox protected |
| Signal messages | End-to-end encrypted |
| Full RAM dump | Kernel restriction, root required |
| Deleted files | Physical image required |
