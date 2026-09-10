# Docker Adli Bilişim Modülü 

### Genel Bakış

Amele Docker Adli Bilişim Modülü, Linux ve Windows konteyner ortamlarında meydana gelen güvenlik ihlalleri, zararlı yazılım aktiviteleri ve konteyner kaçış (escape) girişimlerini analiz etmek için geliştirilmiş bütünleşik bir adli analiz ve edinim motorudur.

Modül hem **yerel canlı sistem** (Linux `/var/lib/docker`, Windows `C:\ProgramData\Docker`), hem **bağlanmış adli disk imajı**, hem de **Amele Agent** üzerinden uzak sunucularda çalışabilir.

---

### Temel Yetenekler

| Özellik | Açıklama |
|---------|----------|
| **Overlay2 UpperDir Drift Analizi** | Konteyner başlatıldıktan sonra dosya sisteminde sonradan oluşturulan, değiştirilen veya silinen tüm dosyaları (websheller, saldırgan binaryleri, konfigürasyon değişiklikleri) minimum boyutta `.tar.gz` olarak paketler. |
| **Konteyner Kaçış Riski Değerlendirmesi** | `--privileged`, `/var/run/docker.sock` mountu, `hostPID`, `hostNetwork`, `hostIPC` ad alanları, tehlikeli Linux yetkileri (`SYS_ADMIN`, `SYS_PTRACE`, `NET_ADMIN`) ve `AppArmor/Seccomp: unconfined` durumlarını denetleyerek `CRITICAL`, `HIGH`, `MEDIUM`, `LOW` risk puanı üretir. |
| **Ortam Değişkeni / Secret Tespiti** | Konteyner konfigürasyonundaki (ENV) parola, veritabanı şifresi, API anahtarı, AWS tokenı ve JWT kalıplarını regex ile tespit eder. |
| **Ham Konfigürasyon ve Loglar** | `config.v2.json`, `hostconfig.json` ve `<id>-json.log` dosyalarını adli vaka klasörüne toplar. Kısa (12 karakter) ID veya tam ID eşleştirmesini destekler. |
| **Çapraz Platform ve Uzak Destek** | Linux soketi (`/var/run/docker.sock`) ve Windows named pipe (`\\.\pipe\docker_engine`) desteği ile hem yerel hem de Agent üzerinden uzak Docker sunucularını inceler. |
| **Profil ve Vaka Entegrasyonu** | Toplanan deliller aktif vaka klasörüne kaydedilir ve Profil > Vaka Geçmişi (Acquisition History) ekranında otomatik listelenir. |

---

### Çıktı Klasör Yapısı

```text
~/Amele/Vakalar/{vaka}/docker/{konteyner_adi}_{kisa_id}/
├── docker_metadata.json          # Konteyner meta bilgileri, risk puanı, secret listesi
├── manifest.csv                  # SHA-256 dosya bütünlük tablosu
├── config.v2.json                # Ham Docker runtime konfigürasyonu
├── hostconfig.json               # Host izolasyon ve mount ayarları
├── container.log                 # Konteyner konsol/çalışma logları
└── overlay2_diff.tar.gz          # UpperDir drift / saldırgan dosya katmanı arşivi
```

---

### Üretilen Bütünlük Manifestosu (`manifest.csv`)

```text
Dosya_Adi,Boyut_Byte,SHA256
config.v2.json,12450,e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
hostconfig.json,4210,a1b2c3d4e5f6...
container.log,851200,9f8e7d6c5b4a...
overlay2_diff.tar.gz,14520300,7c6b5a4...
```

---

### Masaüstü Arayüzü Kullanımı

1. Sol menüden **Docker Araçları** sekmesini açın.
2. Mod seçimi yapın:
   - **Yerel / İmaj Modu:** Canlı `/var/lib/docker` veya bağlanmış adli imaj yolunu girip **Docker Kök Dizinini Tara** butonuna basın.
   - **Uzak Agent Modu:** Hedef Linux sunucusunun IP, Port ve Token bilgilerini girip **Uzak Docker Tara** butonuna basın.
3. Konteyner tablosunda durumları, imajları ve risk rozetlerini (`CRITICAL`, `HIGH`, `MEDIUM`, `LOW`) inceleyin.
4. **İncele** butonu ile:
   - **Genel Bilgi:** İmaj, PID, oluşturulma zamanı, ağ modu, UpperDir ve log yolları.
   - **Güvenlik & Kaçış Riski:** Tespit edilen kaçış vektörleri ve açıklayıcı uyarılar.
   - **Ortam & Gizli Veri:** Açıkta kalan parola ve token sızıntıları.
   - **Overlay2 Drift:** Konteyner içindeki dosya değişiklikleri.
   - **Canlı Loglar:** Son konsol çıktıları.
5. **Vakaya Edin** butonu ile delilleri otomatik paketleyip vaka klasörüne aktarın.

---

### CLI Komutları

```bash
# Docker sistem/kök dizin durumunu denetle
amele docker-status [/var/lib/docker]

# Konteynerleri, kaçış risklerini ve secret sızıntılarını listele
amele docker-list [/var/lib/docker]

# Konteyner loglarını oku
amele docker-logs <container_id> [tail_satir_sayisi] [/var/lib/docker]

# Yerel konteyner delillerini vaka klasörüne edin
amele docker-acquire <container_id> [vaka_adi] [/var/lib/docker]

# Uzak Linux Agent üzerinden Docker durumunu sorgula
amele docker-remote-status <ip> <port> [token]

# Uzak Agent üzerindeki konteynerleri listele
amele docker-remote-list <ip> <port> [token]

# Uzak konteyner loglarını çek
amele docker-remote-logs <ip> <port> <container_id> [tail] [token]

# Uzak konteyner delillerini vaka klasörüne aktar
amele docker-remote-acquire <ip> <port> <container_id> [vaka_adi] [token]
```
