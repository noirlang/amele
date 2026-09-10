# Windows Adli Bilişim Modülü 

### Genel Bakış

Windows modülü iki farklı çalışma modunu destekler:

1. **Yerel İmaj Alma** — Amele'un çalıştığı Windows makinedeki disklerden doğrudan imaj alma
2. **Uzak İmaj Alma** — Hedef Windows makineye yerleştirilen `amele-win` ajanı üzerinden ağdan imaj ve RAM edinimi

### Yerel İmaj Alma

Administrator yetkisi gerektirir.

#### Disk İmajı

| Özellik | Açıklama |
|---------|----------|
| **Kaynak** | `\\.\PhysicalDrive0`, `\\.\PhysicalDrive1`, ... (0–31) |
| **Çıktı** | RAW (`.img`) bit-by-bit tam kopyası |
| **Hash** | SHA-256 (otomatik, `.sha256` sidecar dosyası) |
| **Boyut Algılama** | `DeviceIoControl(IOCTL_DISK_GET_LENGTH_INFO)` ile disk boyutu |
| **Dosya Açma** | `CreateFileW` + `GENERIC_READ` + `FILE_SHARE_READ/WRITE` |
| **Chunk Boyutu** | 4 MB (varsayılan) |
| **Kontrol** | Pause / Resume / Cancel desteği |
| **Kısmî Kurtarma** | Hata durumunda `.partial` dosya olarak saklanır |

**İş Akışı:**
1. `\\.\PhysicalDrive[0-31]` taranarak disk listesi oluşturulur
2. Kullanıcı disk ve vaka seçer
3. Bit-by-bit kopyalama başlatılır
4. SHA-256 hash eşzamanlı hesaplanır
5. `.img` + `.sha256` dosyaları vaka klasörüne yazılır

#### RAM İmajı (WinPMEM)

| Özellik | Açıklama |
|---------|----------|
| **Araç** | [go-winpmem](https://github.com/Velocidex/go-winpmem) (Velocidex) |
| **Dosya** | `go-winpmem_amd64_1.0-rc2_signed.exe` |
| **Yetki** | Administrator gerekli |
| **Çıktı** | AFF4 veya RAW bellek dökümü |
| **Arama Yolları** | `PATH`, çalışma dizini, `C:\Forensics\`, `C:\Tools\` |
| **Komut Varyantları** | 3 farklı CLI sözdizimi otomatik denenir |
| **İlerleme** | Çıktı dosyası boyutu izlenerek takip edilir |
| **Zaman Aşımı** | 1 saat |
| **Kontrol** | `Suspend-Process`/`Resume-Process` ile pause/resume |

**WinPMEM Komut Varyantları (sırasıyla denenir):**
```
1. winpmem.exe acquire <output_file>
2. winpmem.exe acquire --format raw <output_file>
```

### Uzak İmaj Alma (amele-win Ajanı)

Hedef Windows makineye `amele-win` ajanı yerleştirilir ve TCP üzerinden JSON protokolü ile iletişim kurulur.

#### Protokol

| Alan | Değer | Açıklama |
|------|-------|----------|
| Transport | TCP | JSON-over-TCP |
| Başlatma | `{"komut":"merhaba"}` | El sıkışma (handshake) |
| Kimlik Doğrulama | `guvenlik_anahtar_b64` | Base64 token (opsiyonel) |
| Veri Akışı | Binary stream | JSON kontrol mesajları arasında ham veri |

> **Not:** Protokol alanları Türkçe'dir (`komut`, `durum`, `diskler`, `boyut` vb.). Bu `amele-linux` ve `amele-win` ajanlarıyla geriye dönük uyumluluk için zorunludur.

#### Uzak Disk İmajı

| Komut | Açıklama |
|-------|----------|
| `disk_listele` | Hedef makinedeki diskleri listeler (`id`, `ad`, `boyut`) |
| `imaj_baslat` | Disk imaj transferini başlatır (RAW binary stream) |
| `edinim_kontrol` | Devam et / duraklat / iptal et |

**Toplanan Veri:**
- RAW disk imajı (bit-by-bit, `\\.\PhysicalDriveN`)
- SHA-256 + MD5 hash (ajan tarafından hesaplanır)
- İlerleme bilgisi (gerçek zamanlı)

#### Uzak RAM İmajı (WinPMEM)

| Komut | Açıklama |
|-------|----------|
| `winpmem_kontrol` | Ajanda WinPMEM mevcut mu? Administrator yetkisi var mı? RAM boyutu? |
| `ram_edinim_baslat` | WinPMEM'i uzaktan başlatır, RAM bellek dökümünü alır |
| `ram_dosya_indir` | Alınan RAM dökümünü ağ üzerinden indirir |

### Agentsız Uzak İmaj ve RAM Edinimi (OpenSSH / WinRM)

Hedef Windows makineye hiçbir ajan yüklemeden, yerel Windows OpenSSH servisi veya WinRM üzerinden doğrudan `PhysicalDrive` ve WinPMEM bellek edinimini gerçekleştirir.

#### Desteklenen İşlemler:
- **Disk Tarama:** `Get-CimInstance Win32_DiskDrive` veya `wmic diskdrive` ile tüm `PhysicalDrive` sürücülerini JSON olarak listeler.
- **Disk İmajı:** `PhysicalDrive[0-31]` üzerinden doğrudan ham binary stream ile disk kopyasını çeker.
- **RAM Edinimi:** `winpmem.exe` aracını uzaktan tetikleyerek bellek dökümünü canlı pipe ile yerel vakaya aktarır.
- **Format Desteği:** RAW (`.raw` / `.img`) veya AFF4 (`.aff4`) olarak vaka klasörüne kaydeder.
- **Canlı Hashleme:** İndirme esnasında eşzamanlı SHA-256 ve MD5 hash üretir.

#### CLI Komutları:
```bash
amele ssh-disks <ip> <kullanici> [port] [parola] [anahtar_yolu]
amele ssh-tool-check <ip> <kullanici> [port] [parola] [anahtar_yolu]
amele ssh-image <ip> <kullanici> <PhysicalDriveN> <cikti_klasoru> [vaka_adi] [port] [parola] [anahtar_yolu] [raw|aff4]
amele ssh-ram <ip> <kullanici> <cikti_klasoru> [vaka_adi] [port] [parola] [anahtar_yolu] [raw|aff4]
```

### Çıktı Klasör Yapısı

```
~/Amele/Vakalar/{vaka_adı}/
├── {ip}_{disk}_{tarih}.img          # Disk imajı
├── {ip}_{disk}_{tarih}.img.sha256   # SHA-256 hash
├── {ip}_{disk}_{tarih}.img.md5      # MD5 hash (uzak edinimde)
├── ram_{tarih}.raw                   # RAM dökümü
└── ram_{tarih}.raw.sha256            # RAM hash
```

### Desteklenen Disk Tipleri

| Tip | Yol | Açıklama |
|-----|-----|----------|
| Tüm fiziksel sürücüler | `\\.\PhysicalDrive[0-31]` | SATA, NVMe, USB, SAS — Windows'un gördüğü tüm fiziksel diskler |
