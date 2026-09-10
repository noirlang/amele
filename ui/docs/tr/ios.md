# iOS Adli Bilişim Modülü 

### Genel Bakış

Amele iOS modülü, iTunes/Finder tarzı iOS backup klasörlerini `Manifest.db`
indeksine göre okunabilir bir dosya sistemi ağacına dönüştürür. Mantık
Backup2FS ile aynıdır: hashlenmiş düz backup dosyaları gerçek domain ve
relative path bilgilerine göre yeniden klasörlenir.

Bu entegrasyon Rust/native katmanda çalışır; Windows ve Linux üzerinde aynı
kod yolu kullanılır. Harici `.exe`, WPF arayüzü, Wine veya dotnet runtime
gerektirmez.

### Desteklenen Backup Tipi

| Tip | Durum | Açıklama |
|-----|-------|----------|
| Şifresiz iTunes/Finder backup | Aktif | `Manifest.db` doğrudan okunur ve dosyalar normalize edilir. |
| Önceden decrypt edilmiş backup | Aktif | Backup2FS veya eşdeğer araçla decrypt edilmiş klasör işlenebilir. |
| Şifreli iOS backup | Kontrollü uyarı | Amele kaynak backup'ı değiştirmez ve şifre çözmez. iTunes/Finder içinde backup şifrelemesini kapatıp yeni şifresiz backup alın veya Manifest.db ve dosyaları önceden decrypt edilmiş klasörü seçin. |

### Çıktı Klasör Yapısı

```
~/Amele/Vakalar/{vaka}/ios/{backup_adi}_{tarih}/
├── ios_manifest.json
├── ios_manifest.json.sha256
├── extraction_log_{tarih}.csv
└── private/
    └── var/
        ├── mobile/
        ├── Keychains/
        ├── db/
        └── ...
```

### Üretilen Log

`extraction_log_*.csv` her `Manifest.db` girdisi için bir satır üretir:

```text
Timestamp,Status,Domain,RelativePath,FileID,OutputPath,SizeBytes,MD5,SHA1,SHA256
```

`Status` değerleri:

- `Copied`: Dosya kopyalandı ve seçili hashler hesaplandı.
- `Directory`: Manifest girdisi klasör olarak oluşturuldu.
- `Symlink`: Symlink girdisi işlendi; hedef bilgisi `*.symlink.txt` dosyasına yazılır. Linux üzerinde hedef güvenli biçimde vaka çıktı ağacına eşlenebiliyorsa native symlink de oluşturulur.
- `Missing`: Backup içindeki hashlenmiş kaynak dosya bulunamadı.
- `Error`: Girdi işlenirken hata oluştu.

### Kullanım

1. Sol menüden **iOS Araçları** sayfasını açın.
2. `Manifest.db` içeren iTunes/Finder backup klasörünü seçin.
3. **Backup Profilini Oku** ile cihaz ve backup durumunu kontrol edin.
4. Vaka seçin veya yeni vaka oluşturun.
5. MD5/SHA1/SHA256 hash seçeneklerini belirleyin.
6. **Backup'ı Normalize Et** ile işlemi başlatın.

İşlem sırasında pause/resume/stop kontrolleri ve canlı log paneli kullanılabilir.

### CLI

```bash
amele ios-backup-profile /path/to/ios-backup
amele ios-backup-normalize /path/to/ios-backup Case_001
```
