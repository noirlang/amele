# Linux Adli Bilişim Modülü 

### Genel Bakış

Linux modülü iki farklı çalışma modunu destekler:

1. **Yerel İmaj Alma** — Amele'un çalıştığı makinedeki disklerden doğrudan imaj alma
2. **Uzak İmaj Alma** — Hedef Linux makineye yerleştirilen `amele-linux` ajanı üzerinden ağdan imaj ve RAM edinimi

### Yerel İmaj Alma

Root/sudo yetkisi gerektirir. Amele kendini `pkexec` ile yeniden başlatarak yetki yükseltir.

#### Disk İmajı

| Özellik | Açıklama |
|---------|----------|
| **Kaynak** | `/dev/sdX`, `/dev/nvmeXn1`, `/dev/vdX` blok cihazları |
| **Çıktı** | RAW (`.img`) bit-by-bit tam kopyası |
| **Hash** | SHA-256 (otomatik, `.sha256` sidecar dosyası) |
| **Boyut Algılama** | `ioctl(BLKGETSIZE64)` ile blok cihaz boyutu |
| **Chunk Boyutu** | 4 MB (varsayılan) |
| **Kontrol** | Pause / Resume / Cancel desteği |
| **Kısmî Kurtarma** | Hata durumunda `.partial` dosya olarak saklanır |

**İş Akışı:**
1. Disk listesi alınır (`/dev/sd[a-p]`, `/dev/nvme[0-7]n1`, `/dev/vd[a-h]`)
2. Kullanıcı disk ve vaka seçer
3. Bit-by-bit kopyalama başlatılır (`read()` → `write()`)
4. SHA-256 hash eşzamanlı hesaplanır
5. `.img` + `.sha256` dosyaları vaka klasörüne yazılır

#### RAM İmajı (AVML)

| Özellik | Açıklama |
|---------|----------|
| **Araç** | [AVML](https://github.com/microsoft/avml) (Microsoft) |
| **Yetki** | Root gerekli |
| **Çıktı** | Lime formatında bellek dökümü |
| **Arama Yolları** | `PATH`, `/usr/bin/avml`, `/usr/local/bin/avml` |
| **İlerleme** | Çıktı dosyası boyutu izlenerek takip edilir |
| **Zaman Aşımı** | 2 saat |
| **Kontrol** | `SIGSTOP`/`SIGCONT` ile pause/resume, `kill` ile cancel |

### Uzak İmaj Alma (amele-linux Ajanı)

Hedef Linux makineye `amele-linux` ajanı yerleştirilir ve TCP üzerinden JSON protokolü ile iletişim kurulur.

#### Protokol

| Alan | Değer | Açıklama |
|------|-------|----------|
| Transport | TCP | JSON-over-TCP |
| Başlatma | `{"komut":"merhaba"}` | El sıkışma (handshake) |
| Kimlik Doğrulama | `guvenlik_anahtar_b64` | Base64 token (opsiyonel) |
| Veri Akışı | Binary stream | JSON kontrol mesajları arasında ham veri |

#### Uzak Disk İmajı

| Komut | Açıklama |
|-------|----------|
| `disk_listele` | Hedef makinedeki diskleri listeler (`id`, `ad`, `boyut`) |
| `imaj_baslat` | Disk imaj transferini başlatır (RAW binary stream) |
| `edinim_kontrol` | Devam et / duraklat / iptal et |

**Toplanan Veri:**
- RAW disk imajı (bit-by-bit)
- SHA-256 + MD5 hash (ajan tarafından hesaplanır)
- İlerleme bilgisi (gerçek zamanlı)

#### Uzak RAM İmajı (AVML)

| Komut | Açıklama |
|-------|----------|
| `avml_kontrol` | Ajanda AVML mevcut mu? Root yetkisi var mı? RAM boyutu? |
| `ram_edinim_baslat` | AVML'yi uzaktan başlatır, RAM bellek dökümünü alır |
| `ram_dosya_indir` | Alınan RAM dökümünü ağ üzerinden indirir |

### Agentsız Uzak İmaj ve RAM Edinimi (SSH)

Hedef Linux makineye hiçbir ajan kurmadan, standart SSH bağlantısı (parola, private key veya ssh-agent) üzerinden fiziksel disk ve RAM edinimini gerçekleştirir.

#### Desteklenen İşlemler:
- **Disk Tarama:** `lsblk -J` veya `/proc/partitions` ile hedef blok aygıtlarını JSON formatında listeler.
- **Disk İmajı:** `dd` komutunu uzak kabukta çalıştırarak doğrudan binary stream olarak yerel sisteme çeker.
- **RAM Edinimi:** AVML veya `/proc/kcore` üzerinden doğrudan RAM dökümünü akıtır.
- **Format Desteği:** RAW (`.raw`) veya AFF4 (`.aff4`) olarak vaka klasörüne kaydeder.
- **Canlı Hashleme:** İndirme esnasında eşzamanlı SHA-256 ve MD5 hash üretir.

#### CLI Komutları:
```bash
amele ssh-disks <ip> <kullanici> [port] [parola] [anahtar_yolu]
amele ssh-tool-check <ip> <kullanici> [port] [parola] [anahtar_yolu]
amele ssh-image <ip> <kullanici> <disk_yolu> <cikti_klasoru> [vaka_adi] [port] [parola] [anahtar_yolu] [raw|aff4]
amele ssh-ram <ip> <kullanici> <cikti_klasoru> [vaka_adi] [port] [parola] [anahtar_yolu] [raw|aff4]
```

### Çıktı Klasör Yapısı

```
~/Amele/Vakalar/{vaka_adı}/
├── {disk_ismi}_{tarih}.img          # Disk imajı
├── {disk_ismi}_{tarih}.img.sha256   # SHA-256 hash
├── ram_{tarih}.lime                  # RAM dökümü
└── ram_{tarih}.lime.sha256           # RAM hash
```

### Desteklenen Disk Tipleri

| Tip | Yol | Açıklama |
|-----|-----|----------|
| SATA/SAS/USB | `/dev/sd[a-p]` | Geleneksel diskler + USB |
| NVMe | `/dev/nvme[0-7]n1` | M.2 SSD'ler |
| VirtIO | `/dev/vd[a-h]` | Sanal makine diskleri |
