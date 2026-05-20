# Android Adli Bilişim Modülü / Android Forensic Module

---

## 🇹🇷 Türkçe

### Aşamalar

| Aşama | Yetki | Durum | Açıklama |
|-------|-------|-------|----------|
| **Mantıksal İmaj** | Root gerektirmez | ✅ Aktif | ADB ile erişilebilen tüm veriler |
| **Dosya Sistemi İmajı** | Root / exploit | 🔜 Planlanıyor | Protected alanlar dahil dosya sistemi |
| **Fiziksel İmaj** | Bootloader / EDL | 🔜 Planlanıyor | En düşük seviye bit-by-bit imaj |

### Mantıksal İmaj — Toplanan Veriler (50 Adım)

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
| `dumpsys_meminfo` | `dumpsys_meminfo.txt` | `dumpsys meminfo` | Uygulama bazlı bellek kullanımı (PSS, heap, dalvik) |
| `dumpsys_appops` | `dumpsys_appops.txt` | `dumpsys appops` | Uygulama izin kullanım geçmişi (kamera, mikrofon, konum erişimleri) |
| `dumpsys_package` | `dumpsys_package.txt` | `dumpsys package` | Yüklü paket detayları, izin talepleri/izin durumları, yüklenme ve güncellenme tarihleri, imzalar (Malware analizi için kritik!). |
| `dumpsys_diskstats` | `dumpsys_diskstats.txt` | `dumpsys diskstats` | Paket başına disk kullanımı, önbellek boyutları, dosya istatistikleri |
| `dumpsys_deviceidle` | `dumpsys_deviceidle.txt` | `dumpsys deviceidle` | Doze modu (güç tasarrufu) durumu, beyaz listedeki uygulamalar, uyku istatistikleri |
| `dumpsys_alarm` | `dumpsys_alarm.txt` | `dumpsys alarm` | Planlanmış alarmlar ve uygulamaların cihazı uyandırma (wakeup) sıklıkları |
| `dumpsys_jobscheduler` | `dumpsys_jobscheduler.txt` | `dumpsys jobscheduler` | Uygulamaların arka planda çalışmak üzere planladığı işler (Zararlı yazılımların kalıcılık / persistence analizinde hayati!) |
| `dumpsys_procstats` | `dumpsys_procstats.txt` | `dumpsys procstats` | Süreçlerin çalışma süreleri, geçmiş durumları ve bellek tüketim geçmişi |
| `dumpsys_sensorservice` | `dumpsys_sensorservice.txt` | `dumpsys sensorservice` | Cihazdaki donanım sensörleri (GPS, ivmeölçer vb.) ve bunları dinleyen uygulamalar |
| `dumpsys_power` | `dumpsys_power.txt` | `dumpsys power` | Güç yönetimi, wake lock (ekranı/cihazı açık tutma) kilidi olan uygulamalar |
| `dumpsys_window` | `dumpsys_window.txt` | `dumpsys window` | Aktif pencereler, ekran yerleşim planı, odaklanılan pencereler ve ekran boyut bilgileri |
| `dumpsys_clipboard` | `dumpsys_clipboard.txt` | `dumpsys clipboard` | Cihazın panosundaki (clipboard) kopyalanmış son metinler, şifreler, linkler ve OTP kodları. |
| `dumpsys_batterystats` | `dumpsys_batterystats.txt` | `dumpsys batterystats` | Aşırı detaylı pil tüketim günlüğü, şarj olma döngüleri, ekran açık/kapalı süreleri ve cihazın aktif kullanılma zaman çizelgesi. |
| `dumpsys_keystore` | `dumpsys_keystore.txt` | `dumpsys keystore` | Android Keystore güvenlik modülü teşhis bilgileri, şifreleme anahtarı meta-verileri ve takma adları (key aliases). |

#### Cihaz Ayarları

| Adım | Dosya | ADB Komutları | Açıklama |
|------|-------|---------------|----------|
| `device_settings` | `device_settings.txt` | `settings list system/secure/global` | Tüm cihaz ayarları: ekran parlaklığı, varsayılan uygulamalar, geliştirici seçenekleri, Google hesap ID'leri, erişilebilirlik servisleri vb. |

#### Content Provider Sorguları

Modern Android'de bu sorgular izin kısıtlamalarıyla başarısız olabilir ancak denenir. Eski cihazlarda veya özel durumlarda son derece değerli adli kanıt sağlarlar.

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `content_sms` | `content_sms.txt` | `content query --uri content://sms` | SMS mesajları (gönderen, alıcı, tarih, içerik) |
| `content_calls` | `content_calls.txt` | `content query --uri content://call_log/calls` | Arama kayıtları (numara, süre, tarih, tür) |
| `content_contacts` | `content_contacts.txt` | `content query --uri content://contacts/phones` | Rehber (isim, numara) |
| `content_user_dictionary` | `content_user_dictionary.txt` | `content query --uri content://user_dictionary/words` | Kullanıcının özel sözlüğü (yazdığı özel kelimeler, jargonlar, isimler) |
| `content_calendar` | `content_calendar.txt` | `content query --uri content://com.android.calendar/events` | Takvim etkinlikleri (etkinlik başlıkları, açıklamalar, tarihler) |
| `content_media_images` | `content_media_images.txt` | `content query --uri content://media/external/images/media` | MediaStore'daki tüm görsellerin metadata indeksi (tarih, konum koordinatları, kamera bilgisi) |
| `content_media_videos` | `content_media_videos.txt` | `content query --uri content://media/external/video/media` | MediaStore'daki tüm videoların metadata indeksi |
| `content_media_audio` | `content_media_audio.txt` | `content query --uri content://media/external/audio/media` | Cihazdaki tüm ses dosyalarının, ses kayıtlarının ve müziklerin adli metadata indeksi. |
| `content_media_files` | `content_media_files.txt` | `content query --uri content://media/external/file` | Depolama alanındaki **TÜM dosyaların** (belgeler, PDF'ler, gizli dosyalar) tam dosya listesi, boyutları ve zaman damgası indeks kaydı (Cihazı tamamen çekmeden dosya haritasını çıkarır). |
| `content_telephony_carriers` | `content_telephony_carriers.txt` | `content query --uri content://telephony/carriers` | Operatör ayarları, APN (Erişim Noktası Adı) yapılandırmaları ve internet bağlantı parametreleri. |

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

WhatsApp Business, Instagram, Facebook Messenger, Facebook, Viber, Google Messages, X (Twitter), Snapchat, TikTok (2 variants), Discord, LinkedIn, Pinterest, Reddit, Spotify, Signal, Skype, Zoom, Microsoft Teams, BiP (Turkcell), Wire, Telegram Plus, KakaoTalk, LINE, WeChat, IMO.

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
├── dumpsys_package.txt
├── dumpsys_diskstats.txt
├── dumpsys_deviceidle.txt
├── dumpsys_alarm.txt
├── dumpsys_jobscheduler.txt
├── dumpsys_procstats.txt
├── dumpsys_sensorservice.txt
├── dumpsys_power.txt
├── dumpsys_window.txt
├── dumpsys_clipboard.txt         <-- YENİ (Kopyalanan veriler)
├── dumpsys_batterystats.txt      <-- YENİ (Detaylı cihaz zaman çizelgesi)
├── dumpsys_keystore.txt          <-- YENİ (Güvenlik & Key anahtar takma adları)
├── device_settings.txt
├── network_info.txt
├── content_sms.txt
├── content_calls.txt
├── content_contacts.txt
├── content_user_dictionary.txt
├── content_calendar.txt
├── content_media_images.txt
├── content_media_videos.txt
├── content_media_audio.txt       <-- YENİ (Ses kayıtları meta-verileri)
├── content_media_files.txt       <-- YENİ (Tüm harici dosyaların indeksi)
├── content_telephony_carriers.txt <-- YENİ (Operatör ve internet ayarları)
├── screenshot.png
├── whatsapp_media/
├── telegram_media/
├── app_media/
│   ├── instagram/
│   ├── x_twitter/
│   ├── snapchat/
│   ├── tiktok/
│   └── ... (26 uygulama)
├── all_app_media/
├── adb_backup.ab
├── bugreport-*.zip
├── shared_storage/
├── manifest.json
└── manifest.json.sha256
```

---

## 🇬🇧 English

### Phases

| Phase | Privilege | Status | Description |
|-------|-----------|--------|-------------|
| **Logical Image** | No root required | ✅ Active | All data accessible via ADB |
| **File System Image** | Root / exploit | 🔜 Planned | File system including protected areas |
| **Physical Image** | Bootloader / EDL | 🔜 Planned | Lowest level bit-by-bit image |

### Logical Image — Collected Data (50 Steps)

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
| `dumpsys_meminfo` | `dumpsys_meminfo.txt` | `dumpsys meminfo` | Per-app memory usage (PSS, heap, dalvik) |
| `dumpsys_appops` | `dumpsys_appops.txt` | `dumpsys appops` | App permission usage history (camera, microphone, location access records) |
| `dumpsys_package` | `dumpsys_package.txt` | `dumpsys package` | Package details, requested vs granted permissions, install/update times, signatures (Critical for malware/spyware analysis!). |
| `dumpsys_diskstats` | `dumpsys_diskstats.txt` | `dumpsys diskstats` | Disk usage per package, cache sizes, file statistics |
| `dumpsys_deviceidle` | `dumpsys_deviceidle.txt` | `dumpsys deviceidle` | Doze mode (battery saver) states, whitelisted applications, sleep statistics |
| `dumpsys_alarm` | `dumpsys_alarm.txt` | `dumpsys alarm` | Scheduled alarms and app wakeup frequencies |
| `dumpsys_jobscheduler` | `dumpsys_jobscheduler.txt` | `dumpsys jobscheduler` | Background jobs scheduled by apps (Crucial for detecting persistence mechanisms of malicious apps!) |
| `dumpsys_procstats` | `dumpsys_procstats.txt` | `dumpsys procstats` | Process running times, historical memory consumption, and states |
| `dumpsys_sensorservice` | `dumpsys_sensorservice.txt` | `dumpsys sensorservice` | Active hardware sensors (GPS, accelerometer, etc.) and apps listening to them |
| `dumpsys_power` | `dumpsys_power.txt` | `dumpsys power` | Power management, active wake locks, battery saving configurations |
| `dumpsys_window` | `dumpsys_window.txt` | `dumpsys window` | Active windows, screen layout, current focus, and display metrics |
| `dumpsys_clipboard` | `dumpsys_clipboard.txt` | `dumpsys clipboard` | Clipboard dump containing copied texts, passwords, OTPs, or links. |
| `dumpsys_batterystats` | `dumpsys_batterystats.txt` | `dumpsys batterystats` | Extremely detailed battery logs, charge levels, screen stats, usage timeline. |
| `dumpsys_keystore` | `dumpsys_keystore.txt` | `dumpsys keystore` | Keystore diagnostic data, cryptographic key metadata, and aliases. |

#### Device Settings

| Step | File | ADB Commands | Description |
|------|------|--------------|-------------|
| `device_settings` | `device_settings.txt` | `settings list system/secure/global` | All device settings: brightness, default apps, developer options, Google account IDs, accessibility services, etc. |

#### Content Provider Queries

On modern Android these queries may fail due to permission restrictions, but they are attempted. On older devices or special configurations they yield valuable forensic data.

| Step | File | ADB Command | Description |
|------|------|-------------|-------------|
| `content_sms` | `content_sms.txt` | `content query --uri content://sms` | SMS messages (sender, recipient, date, body) |
| `content_calls` | `content_calls.txt` | `content query --uri content://call_log/calls` | Call log (number, duration, date, type) |
| `content_contacts` | `content_contacts.txt` | `content query --uri content://contacts/phones` | Contacts (name, phone number) |
| `content_user_dictionary` | `content_user_dictionary.txt` | `content query --uri content://user_dictionary/words` | User's custom dictionary (custom typed words, names, slangs) |
| `content_calendar` | `content_calendar.txt` | `content query --uri content://com.android.calendar/events` | Calendar events (titles, descriptions, dates) |
| `content_media_images` | `content_media_images.txt` | `content query --uri content://media/external/images/media` | Metadata index of all images in MediaStore (dates, location coordinates, camera info) |
| `content_media_videos` | `content_media_videos.txt` | `content query --uri content://media/external/video/media` | Metadata index of all videos in MediaStore |
| `content_media_audio` | `content_media_audio.txt` | `content query --uri content://media/external/audio/media` | Metadata index of all audio files, voice recordings, and music. |
| `content_media_files` | `content_media_files.txt` | `content query --uri content://media/external/file` | Entire file hierarchy index (documents, PDFs, downloads, hidden files) from MediaStore (allows mappings without pulling GBs of storage). |
| `content_telephony_carriers` | `content_telephony_carriers.txt` | `content query --uri content://telephony/carriers` | Operator configurations, Access Point Name (APN) data, internet profiles. |

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

#### Diagnostic & Storage

| Step | File | Description |
|------|------|-------------|
| `bugreport` | `bugreport-*.zip` | Comprehensive diagnostic (logcat + all dumpsys + kernel log + more) |
| `shared_storage` | `shared_storage/` | All user files from internal storage: DCIM, Download, Documents, Pictures, Music, etc. |

#### Manifest

| File | Description |
|------|-------------|
| `manifest.json` | Summary of all collected data: category, filename, size, success status |
| `manifest.json.sha256` | SHA-256 hash of the manifest file — integrity verification |
