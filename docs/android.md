# Android Adli Bilişim Modülü

Worm uygulamasının Android cihazlardan veri toplama modülü. Üç aşamadan oluşur:

## Aşamalar

| Aşama | Yetki | Durum | Açıklama |
|-------|-------|-------|----------|
| **Mantıksal İmaj** | Root gerektirmez | ✅ Aktif | ADB ile erişilebilen tüm veriler |
| **Dosya Sistemi İmajı** | Root / exploit | 🔜 Planlanıyor | Protected alanlar dahil dosya sistemi |
| **Fiziksel İmaj** | Bootloader / EDL | 🔜 Planlanıyor | En düşük seviye bit-by-bit imaj |

---

## Mantıksal İmaj — Toplanan Veriler

ADB ve USB hata ayıklama açıkken, root yetkisi **olmadan** toplanan veriler:

### Sistem Bilgileri

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `device_info` | `device_info.txt` | `adb shell getprop` | Cihaz modeli, üretici, Android sürümü, seri no, IMEI, build numarası vb. tüm sistem özellikleri |
| `packages` | `packages.txt` | `adb shell pm list packages -f` | Cihazda yüklü tüm uygulamalar ve APK yolları |
| `processes` | `processes.txt` | `adb shell ps -A` | Çalışan süreçler listesi |
| `disk_usage` | `disk_usage.txt` | `adb shell df -h` | Disk bölümleri ve kullanım durumu |
| `logcat` | `logcat.txt` | `adb logcat -d` | Sistem logları (buffer boyutu kadar) |

### Dumpsys Servisleri

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `dumpsys_battery` | `dumpsys_battery.txt` | `adb shell dumpsys battery` | Pil durumu, şarj seviyesi, sıcaklık |
| `dumpsys_wifi` | `dumpsys_wifi.txt` | `adb shell dumpsys wifi` | Bağlı/kayıtlı Wi-Fi ağları, SSID'ler, MAC adresleri |
| `dumpsys_bluetooth` | `dumpsys_bluetooth.txt` | `adb shell dumpsys bluetooth_manager` | Eşleşmiş Bluetooth cihazları |
| `dumpsys_usagestats` | `dumpsys_usagestats.txt` | `adb shell dumpsys usagestats` | Uygulama kullanım istatistikleri (ne zaman, ne kadar açılmış) |
| `dumpsys_account` | `dumpsys_account.txt` | `adb shell dumpsys account` | Cihazda oturum açmış hesaplar (Google, Samsung vb.) |
| `dumpsys_connectivity` | `dumpsys_connectivity.txt` | `adb shell dumpsys connectivity` | Ağ bağlantı durumu, VPN, mobil veri |
| `dumpsys_notification` | `dumpsys_notification.txt` | `adb shell dumpsys notification` + `cmd notification dump_history` | Bildirim geçmişi — mevcut bildirimler her zaman alınır; cihazda bildirim geçmişi etkinse (`notification_history_enabled=1`) tam geçmiş de eklenir |

### Ağ Bilgileri

| Adım | Dosya | ADB Komutları | Açıklama |
|------|-------|---------------|----------|
| `network_info` | `network_info.txt` | `ip addr`, `ip route`, `netstat -tlnp`, `ip neigh` | IP adresleri, yönlendirme tablosu, açık portlar, ARP tablosu |

### Ekran Görüntüsü

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `screenshot` | `screenshot.png` | `adb shell screencap -p` | Ekranın anlık görüntüsü (imaj alma anında ne görünüyordu) |

### Uygulama Medyaları

Root olmadan `/sdcard/Android/media/<paket>/` altındaki medya dosyalarına erişilebilir. Bu veriler uygulamaların paylaşılan depolama alanında tuttuğu fotoğraf, video, ses dosyası ve belgelerdir.

| Adım | Hedef Klasör | Kaynak Yol | Açıklama |
|------|-------------|------------|----------|
| `whatsapp_media` | `whatsapp_media/` | `/sdcard/Android/media/com.whatsapp/` | WhatsApp medya dosyaları (fotoğraf, video, sesli mesaj, belge) |
| `telegram_media` | `telegram_media/` | `/sdcard/Android/media/org.telegram.messenger/` | Telegram medya dosyaları |
| `app_media` | `app_media/` | Birden fazla uygulama | Diğer uygulamaların medyaları |

**`app_media` adımında taranan uygulamalar:**

| Uygulama | Paket Adı | Çıktı Klasörü |
|----------|-----------|---------------|
| WhatsApp Business | `com.whatsapp.w4b` | `app_media/whatsapp_business/` |
| Instagram | `com.instagram.android` | `app_media/instagram/` |
| Facebook Messenger | `com.facebook.orca` | `app_media/messenger/` |
| Viber | `com.viber.voip` | `app_media/viber/` |
| Google Messages | `com.google.android.apps.messaging` | `app_media/google_messages/` |

### Tanı Raporu

| Adım | Dosya | ADB Komutu | Açıklama |
|------|-------|------------|----------|
| `bugreport` | `bugreport-*.zip` | `adb bugreport` | Kapsamlı tanı dosyası (logcat + tüm dumpsys + kernel log + vs.) |

### Paylaşılan Depolama

| Adım | Hedef Klasör | ADB Komutu | Açıklama |
|------|-------------|------------|----------|
| `shared_storage` | `shared_storage/` | `adb pull /sdcard/` | Dahili depolamadaki tüm kullanıcı dosyaları: DCIM (kamera), Download, Documents, Pictures, Music vb. |

### Manifest

Her imaj alma işleminin sonunda otomatik oluşturulur:

| Dosya | Açıklama |
|-------|----------|
| `manifest.json` | Toplanan tüm verilerin özeti: kategori, dosya adı, boyut, başarı durumu, hata mesajı |
| `manifest.json.sha256` | Manifest dosyasının SHA-256 hash'i — bütünlük doğrulaması için |

---

## Çıktı Klasör Yapısı

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
├── network_info.txt
├── screenshot.png
├── whatsapp_media/
│   ├── WhatsApp Images/
│   ├── WhatsApp Video/
│   ├── WhatsApp Voice Notes/
│   └── ...
├── telegram_media/
│   └── ...
├── app_media/
│   ├── whatsapp_business/
│   ├── instagram/
│   ├── messenger/
│   ├── viber/
│   └── google_messages/
├── bugreport-{cihaz}-{tarih}.zip
├── shared_storage/
│   ├── DCIM/
│   ├── Download/
│   ├── Documents/
│   ├── Pictures/
│   └── ...
├── manifest.json
└── manifest.json.sha256
```

---

## Root Olmadan Erişilemeyen Veriler

Aşağıdaki veriler **sadece root veya fiziksel imaj** ile alınabilir:

| Veri | Neden Erişilemez |
|------|-----------------|
| SMS / MMS veritabanı (`mmssms.db`) | `/data/data/` altında, sandbox korumalı |
| Arama kayıtları (`calllog.db`) | Sistem uygulaması, sandbox korumalı |
| Rehber veritabanı (`contacts2.db`) | Content provider izni gerekli |
| WhatsApp mesaj veritabanı (`msgstore.db`) | Şifreli, `/data/data/com.whatsapp/` altında |
| WhatsApp şifre anahtarı (`key`) | TEE bağlı, root gerekli |
| Tarayıcı geçmişi (Chrome vb.) | `/data/data/com.android.chrome/` altında |
| Signal mesajları | Uçtan uca şifreli, özel depolama |
| Silinmiş dosyalar | Fiziksel imaj / NAND dump gerekli |
| Uygulama sandbox verileri | Her uygulama kendi sandboxında |

---

## Dosya Sistemi İmajı (Planlanıyor)

Root erişimi veya exploit ile mantıksal imajda erişilemeyen alanlar dahil dosya sistemi alınacak:

- `/data/data/` — tüm uygulama veritabanları
- SMS, arama kayıtları, rehber veritabanları
- WhatsApp `msgstore.db` + şifre anahtarı
- Chrome/Firefox tarayıcı geçmişi ve çerezleri
- Wi-Fi yapılandırması (`wpa_supplicant.conf`)

## Fiziksel İmaj (Planlanıyor)

Bootloader veya EDL modunda bit-by-bit disk imajı:

- Silinmiş dosya kurtarma
- Şifreli bölüm analizi
- NAND flash dump
- Üretici/çipset seviyesinde erişim
