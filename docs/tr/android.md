# Android Adli Bilişim Modülü 

### Genel Bakış

Amele'un Android modülü modüler bir altyapı üzerine inşa edilmiştir. Her toplama işi bağımsız bir modül olarak çalışır ve ortak `AndroidSession` / `AndroidCapabilityReport` altyapısını kullanır.

### Mimari

```
src/android/
├── adb.rs          — ADB sarmalayıcı, cihaz listeleme, komut çalıştırma
├── session.rs      — AndroidSession: serial, transport tipi, API seviyesi, ADB yolu
├── capability.rs   — AndroidCapabilityReport: her özellik için destekleniyor/kısmi/yok
├── profile.rs      — AndroidDeviceProfile: model, build, root, şifreleme, SoC
├── manifest.rs     — AndroidAcquisitionManifest: çıktı öğeleri, hash, boyut
├── errors.rs       — Standart hata formatı (kod + neden + çözüm + teknik detay)
├── extractors.rs   — Profil tanımları (QuickLogical, FullLogical, RootLogical, Volatile)
├── logical.rs      — 50+ adımlı mantıksal edinim motoru
├── filesystem.rs   — Non-root / Root dosya sistemi edinimi
├── ram.rs          — Uçucu veri, Root süreç belleği, Lemon fiziksel RAM
├── remote.rs       — TCP/IP ADB + MESH relay bağlantı yönetimi, Lemon preflight
└── orchestrator.rs — Tüm akışları koordine eder, profil + cihaz + manifest birleştirir
```

### Çıktı Klasör Yapısı

Her edinim aşağıdaki yapıyla organize edilir:

```
~/Amele/Vakalar/{vaka}/android/{cihaz}_{tarih}/
├── manifest.json          ← Tüm artefaktların özeti (hash, boyut, modül)
├── manifest.json.sha256
├── device_profile.json    ← Model, build, root, API, SoC, kernel
├── capabilities.json      ← Her özellik için destekleniyor/kısmi/yok raporu
├── logical/               ← Mantıksal edinim çıktıları
│   ├── device_info.txt
│   ├── packages.txt
│   ├── logcat.txt
│   ├── dumpsys_*.txt
│   ├── content_*.txt
│   └── ...
├── filesystem/            ← Dosya sistemi edinimi
│   ├── shared_storage/    (non-root: /sdcard, medya)
│   ├── userdata.img       (root: /data blok imajı)
│   └── filesystem.tar     (root fallback)
├── memory/                ← Bellek edinimi
│   ├── volatile_data.txt  (non-root: ps, meminfo, procstats, logcat)
│   ├── process_dumps/     (root: süreç belleği)
│   └── physical.lime      (Lemon: eBPF fiziksel RAM, LiME formatı)
└── hashes.json            ← Tüm dosyaların SHA-256 listesi
```

### Aşamalar

| Aşama | Yetki | Durum | Açıklama |
|-------|-------|-------|----------|
| **Mantıksal İmaj** | Root gerektirmez | ✅ Aktif | ADB ile 50+ modül |
| **Non-Root Dosya Sistemi** | Root gerektirmez | ✅ Aktif | `/sdcard`, medya, dosya indeksi |
| **Root Dosya Sistemi** | Root / su | ✅ Aktif | `/data`, `/system`, blok imajı veya tar |
| **Uçucu Veri** | ADB (root opsiyonel) | ✅ Aktif | ps, meminfo, procstats, logcat |
| **Root Süreç Belleği** | Root / su | ✅ Aktif | Seçili süreçlerin bellek dökümü |
| **Lemon Fiziksel RAM** | Root + eBPF | ✅ Deneysel | eBPF tabanlı fiziksel RAM (LiME) |
| **Remote / TCP ADB** | ADB (ağ) | ✅ Aktif | `adb connect ip:port` veya MESH relay |
| **Fiziksel İmaj (EDL)** | Bootloader / EDL | 🔜 Yakında | Donanım seviyesi bit-by-bit imaj |

---

### 1. Mantıksal İmaj — 50+ Adım

ADB ve USB hata ayıklama açıkken, root yetkisi **olmadan** toplanan veriler.

#### Profiller

| Profil | Adım Sayısı | İçerik |
|--------|------------|--------|
| `quick_logical` | ~16 adım | Cihaz bilgisi, paketler, dumpsys özeti, content sorguları, bugreport |
| `full_logical` | 50+ adım | Tüm modüller |
| `root_logical` | 50+ adım + root ekleri | Root binaries, keystore, heap dump adayları, procfs özeti |
| `volatile` | 8 adım | Bellek odaklı: procfs, meminfo, heap adayları, logcat |

#### Sistem Bilgileri

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `device_info` | `device_info.txt` | `adb shell getprop` | Cihaz modeli, üretici, Android sürümü, seri no, IMEI, build numarası |
| `packages` | `packages.txt` | `adb shell pm list packages -f` | Yüklü tüm uygulamalar ve APK yolları |
| `packages_json` | `packages.json` | `adb shell pm list packages` | Yapılandırılmış paket verisi (JSON) |
| `processes` | `processes.txt` | `adb shell ps -A` | Çalışan süreçler listesi |
| `disk_usage` | `disk_usage.txt` | `adb shell df -h` | Disk bölümleri ve kullanım durumu |
| `logcat` | `logcat.txt` | `adb logcat -d` | Sistem logları |
| `system_logs` | `system_logs.txt` | `adb shell logcat -b system -d` | Sistem buffer logları |

#### Dumpsys Servisleri

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `dumpsys_battery` | `dumpsys_battery.txt` | `dumpsys battery` | Pil durumu, şarj seviyesi, sıcaklık |
| `dumpsys_wifi` | `dumpsys_wifi.txt` | `dumpsys wifi` | Bağlı/kayıtlı Wi-Fi ağları, SSID'ler, MAC adresleri |
| `dumpsys_bluetooth` | `dumpsys_bluetooth.txt` | `dumpsys bluetooth_manager` | Eşleşmiş Bluetooth cihazları |
| `dumpsys_usagestats` | `dumpsys_usagestats.txt` | `dumpsys usagestats` | Uygulama kullanım istatistikleri |
| `dumpsys_account` | `dumpsys_account.txt` | `dumpsys account` | Cihazda oturum açmış hesaplar |
| `dumpsys_connectivity` | `dumpsys_connectivity.txt` | `dumpsys connectivity` | Ağ bağlantı durumu, VPN, mobil veri |
| `dumpsys_notification` | `dumpsys_notification.txt` | `dumpsys notification --noredact` | Bildirim geçmişi — SMS ve mesaj içerikleri |
| `dumpsys_telephony` | `dumpsys_telephony.txt` | `dumpsys telephony.registry` | SIM bilgileri, IMSI, telefon numarası |
| `dumpsys_location` | `dumpsys_location.txt` | `dumpsys location` | Konum sağlayıcıları, son konum verileri |
| `dumpsys_netstats` | `dumpsys_netstats.txt` | `dumpsys netstats` | Uygulama bazlı ağ trafik istatistikleri |
| `dumpsys_activity` | `dumpsys_activity.txt` | `dumpsys activity` | Çalışan activity'ler, son görevler |
| `dumpsys_meminfo` | `dumpsys_meminfo.txt` | `dumpsys meminfo` | Uygulama bazlı bellek kullanımı |
| `dumpsys_appops` | `dumpsys_appops.txt` | `dumpsys appops` | İzin kullanım geçmişi (kamera, mikrofon, konum) |
| `dumpsys_package` | `dumpsys_package.txt` | `dumpsys package` | Paket detayları, izin durumları, imzalar |
| `dumpsys_diskstats` | `dumpsys_diskstats.txt` | `dumpsys diskstats` | Paket başına disk kullanımı |
| `dumpsys_deviceidle` | `dumpsys_deviceidle.txt` | `dumpsys deviceidle` | Doze modu, beyaz liste uygulamaları |
| `dumpsys_alarm` | `dumpsys_alarm.txt` | `dumpsys alarm` | Planlanmış alarmlar, wakeup sıklıkları |
| `dumpsys_jobscheduler` | `dumpsys_jobscheduler.txt` | `dumpsys jobscheduler` | Arka plan işleri (persistence analizi için kritik) |
| `dumpsys_procstats` | `dumpsys_procstats.txt` | `dumpsys procstats` | Süreç çalışma süresi, bellek geçmişi |
| `dumpsys_sensorservice` | `dumpsys_sensorservice.txt` | `dumpsys sensorservice` | Sensörler ve dinleyen uygulamalar |
| `dumpsys_power` | `dumpsys_power.txt` | `dumpsys power` | Wake lock kilidi olan uygulamalar |
| `dumpsys_window` | `dumpsys_window.txt` | `dumpsys window` | Aktif pencereler, ekran planı |
| `dumpsys_clipboard` | `dumpsys_clipboard.txt` | `dumpsys clipboard` | Panodaki kopyalanmış son metinler, OTP'ler |
| `dumpsys_batterystats` | `dumpsys_batterystats.txt` | `dumpsys batterystats` | Ayrıntılı pil günlüğü, kullanım zaman çizelgesi |
| `dumpsys_keystore` | `dumpsys_keystore.txt` | `dumpsys keystore` | Keystore meta-verileri, anahtar takma adları |

#### Cihaz Ayarları

| Adım | Dosya | ADB Komutları | Açıklama |
|------|-------|---------------|----------|
| `device_settings` | `device_settings.txt` | `settings list system/secure/global` | Tüm cihaz ayarları, geliştirici seçenekleri |

#### Content Provider Sorguları

| Adım | Dosya | Açıklama |
|------|-------|----------|
| `content_sms` | `content_sms.txt` | SMS mesajları (gönderen, alıcı, tarih, içerik) |
| `content_calls` | `content_calls.txt` | Arama kayıtları (numara, süre, tarih, tür) |
| `content_contacts` | `content_contacts.txt` | Rehber (isim, numara) |
| `content_user_dictionary` | `content_user_dictionary.txt` | Kullanıcı sözlüğü |
| `content_calendar` | `content_calendar.txt` | Takvim etkinlikleri |
| `content_media_images` | `content_media_images.txt` | Görsellerin metadata indeksi (EXIF, konum) |
| `content_media_videos` | `content_media_videos.txt` | Videoların metadata indeksi |
| `content_media_audio` | `content_media_audio.txt` | Ses dosyaları ve kayıtların metadata indeksi |
| `content_media_files` | `content_media_files.txt` | Tüm dosyaların tam indeksi (gizli dosyalar dahil) |
| `content_telephony_carriers` | `content_telephony_carriers.txt` | Operatör ayarları, APN yapılandırmaları |

#### Ağ, Ekran ve Medya

| Adım | Dosya | Açıklama |
|------|-------|----------|
| `network_info` | `network_info.txt` | IP, rota, açık portlar, ARP tablosu |
| `screenshot` | `screenshot.png` | İmaj alma anındaki ekran görüntüsü |
| `whatsapp_media` | `whatsapp_media/` | WhatsApp medya dosyaları |
| `telegram_media` | `telegram_media/` | Telegram medya dosyaları |
| `app_media` | `app_media/` | 26 bilinen uygulama medyaları |
| `all_app_media` | `all_app_media/` | `/sdcard/Android/media/` altındaki TÜM paketler |

#### Güvenlik ve Tanı

| Adım | Dosya | Açıklama |
|------|-------|----------|
| `services` | `services.txt` | Çalışan Android servisleri |
| `environment` | `environment.txt` | Ortam değişkenleri |
| `temp_files` | `temp_files.txt` | `/tmp`, `/data/local/tmp` içerikleri |
| `intrusion_indicators` | `intrusion_indicators.txt` | Şüpheli binary, dosya ve süreç göstergeleri |
| `file_index` | `file_index.txt` | `/sdcard` altındaki tüm dosyaların indeksi |
| `adb_backup` | `adb_backup.ab` | `adb backup -all -shared -nosystem` |
| `bugreport` | `bugreport.zip` | Kapsamlı tanı (logcat + tüm dumpsys + kernel log) |
| `shared_storage` | `shared_storage/` | Dahili depolama: DCIM, Download, Documents, vb. |

#### Root Gerektiren Ek Adımlar

| Adım | Dosya | Açıklama |
|------|-------|----------|
| `root_status` | `root_status.txt` | Root türü ve aktiflik durumu |
| `root_binaries` | `root_binaries.txt` | `su`, `magisk`, `busybox` gibi root binary tespiti |
| `selinux_status` | `selinux_status.txt` | SELinux modu (Enforcing/Permissive) |
| `mounts` | `mounts.txt` | Bağlı dosya sistemleri |
| `procfs_summary` | `procfs_summary.txt` | `/proc` özeti: uptime, version, cpuinfo, meminfo |
| `proc_memory_maps` | `proc_memory_maps/` | Çalışan süreçlerin `/proc/[pid]/maps` çıktıları |
| `heapdump_candidates` | `heapdump_candidates.txt` | Debug heap dump alınabilecek süreçler |
| `debug_heap_dumps` | `debug_heap_dumps/` | JDWP ile debug mode uygulamalardan heap dump |
| `dumpsys_keystore` | `dumpsys_keystore.txt` | Keystore meta-verileri |

---

### 2. Dosya Sistemi Edinimi

#### Non-Root Modu
Root olmadan erişilebilen alanlar:
- `/sdcard` — dahili depolama (DCIM, Download, Documents, Pictures, Music)
- `/sdcard/Android/media/` — uygulama medya dosyaları
- MediaStore indeksi — tüm dosyaların metadata haritalama

#### Root Modu
Root veya `adb root` ile:
- `/data` bölümü — blok imajı (`userdata.img`) veya tar arşivi (`filesystem.tar`)
- `/system`, `/vendor` — sistem bölümleri
- Uygulama özel verisi (`/data/data/<paket>/`)

#### Fiziksel İmaj (EDL)
Bootloader / EDL erişimi gerektirir. UI'da "Cihaz modu gerekli" olarak işaretlenmiştir. Yakında.

---

### 3. Bellek Edinimi

#### Uçucu Veri (Non-Root)
Root gerekmez, güvenli:
- `ps -A` — çalışan süreçler
- `/proc/meminfo`, `/proc/vmstat` — bellek istatistikleri
- `dumpsys meminfo`, `dumpsys procstats`
- `logcat -d` — son loglar
- Activity process listesi

#### Root Süreç Belleği
Root / su gerektirir:
- Seçili süreçlerin `/proc/[pid]/mem` okuma
- `/proc/[pid]/maps` — bellek haritası
- JDWP heap dump (debug mode uygulamalar)

#### Lemon Fiziksel RAM (Deneysel)
eBPF tabanlı fiziksel RAM dump aracı. Önce **preflight kontrolü** gereklidir:

| Kontrol | Açıklama |
|---------|----------|
| **Mimari (ABI)** | `arm64-v8a` veya `x86_64` gerekli |
| **Root** | `su` veya `adb root` zorunlu |
| **eBPF/BTF** | `/sys/kernel/btf/vmlinux` mevcut olmalı |
| **/proc/kcore** | Okuma erişimi (varsa) |
| **Depolama** | RAM boyutu + 512 MB boş alan gerekli |
| **SoC Uyarısı** | Exynos/MediaTek → EL2 yeniden başlatma riski |

Çıktı: LiME formatında `.lime` dosyası — Volatility 3 ile doğrudan analiz edilebilir.

---

### 4. Remote Android / MESH

USB cihaz olmadan, ağ üzerinden bağlantı:

| Tip | Açıklama |
|-----|----------|
| **TCP/IP ADB** | `adb connect <ip>:<port>` — Wi-Fi ADB veya hotspost üzerinden |
| **MESH Relay** | MESH altyapısı üzerinden sağlanan ADB endpoint |

Bağlandıktan sonra cihaz otomatik olarak cihaz listesine eklenir ve tüm edinim modülleri bu cihazı hedef alabilir.

---

### 5. Preflight Kontrolü

Her edinim sayfasında otomatik olarak aşağıdaki bilgiler gösterilir:

```
ADB:          ✅ Hazır
Cihaz:        ✅ Yetkili (authorized)
Root:         ❌ Yok
API:          34
Mimari:       arm64-v8a
Şifreleme:    file-based

Desteklenen:  Mantıksal, Bugreport, Non-Root Dosya Sistemi, Uçucu Veri
Desteklenmiyor: Root Dosya Sistemi, Root Süreç Belleği, Lemon RAM
```

---

### 6. Hata Formatı

Tüm Android hataları standart formatta raporlanır:

```
İşlem başarısız: Bugreport alınamadı
Kod:            ANDROID_BUGREPORT_FAILED
Neden:          Cihaz bugreport komutunu tamamlamadı veya bağlantı kesildi.
Çözüm:          Cihazın kilidini açın, USB hata ayıklamayı onaylayın ve tekrar deneyin.
Teknik detay:   adb bugreport exited with code 1
```
