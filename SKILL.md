---
name: amele
description: >-
  Kapsamlı Amele Adli Bilişim (Forensic Tool) CLI & Agent Kullanım Kılavuzu.
  Linux, Windows, Android, iOS ve Docker adli veri edinimi, yerel ve uzak agent kurulumu,
  SSH üzerinden disksiz/ramsız edinim, profil ve vaka yönetimi için operasyonel rehber.
---

# Amele Forensic Tool — Kapsamlı CLI & Agent Kullanım Kılavuzu

**Amele Forensic Tool (`amele`)**, olay müdahale (Incident Response) ve adli bilişim (Digital Forensics) uzmanları için geliştirilmiş; disk, RAM, Android, iOS ve Docker konteynerlerinden canlı/ölü adli delil toplama platformudur.

Bu kılavuz; CLI komutlarının kullanımını, uzak sunucuya Linux/Windows agent kurulumunu, SSH ile agentsız veri edinimini, güvenlik izolasyonlarını ve otomatik adli bilişim iş akışlarını kapsar.

---

## 📑 İçindekiler
1. [Mimari ve Çalışma Modelleri](#-mimari-ve-çalışma-modelleri)
2. [Genel Seçenekler & Kabuk Tamamlama](#-genel-seçenekler--kabuk-tamamlama)
3. [CLI Komut Referansı](#-cli-komut-referansı)
   - [1. Linux & Windows Adli Edinimi (`linux`, `windows`)](#1-linux--windows-adli-edinimi)
   - [2. Android Mobil Adli Bilişim (`android`)](#2-android-mobil-adli-bilişim)
   - [3. iOS Mobil Adli Bilişim (`ios`)](#3-ios-mobil-adli-bilişim)
   - [4. Docker Konteyner Adli Bilişimi (`docker`)](#4-docker-konteyner-adli-bilişimi)
   - [5. Profil Yönetimi (`profile`)](#5-profil-yönetimi)
   - [6. Vaka Paketleme, Bilgi & Doğrulama (`case`)](#6-vaka-paketleme-bilgi--doğrulama)
   - [7. Bütünlük & İmaj Bağlama (`hash`, `verify`, `mount`)](#7-bütünlük--imaj-bağlama)
   - [8. Sistem & Güncelleme (`update`, `wireguard`, `ui`)](#8-sistem--güncelleme)
4. [Standart POSIX Çıkış Kodları](#-standart-posix-çıkış-kodları)
5. [Uzak Sunuculara Agent Kurulum Kılavuzu](#-uzak-sunuculara-agent-kurulum-kılavuzu)
   - [Linux Sunucuya Agent Kurulumu (`amele-linux`)](#linux-sunucuya-agent-kurulumu)
   - [Windows Sunucuya Agent Kurulumu (`amele-win`)](#windows-sunucuya-agent-kurulumu)
   - [SSH ile Agent'sız Doğrudan Edinim](#ssh-ile-agentsız-doğrudan-edinim)
6. [Örnek Olay Müdahale & Edinim Senaryoları](#-örnek-olay-müdahale--edinim-senaryoları)
7. [Güvenlik Mimarisi & Sertleştirme](#-güvenlik-mimarisi--sertleştirme)
8. [Sorun Giderme İlkeleri](#-sorun-giderme-ilkeleri)

---

## 🏛️ Mimari ve Çalışma Modelleri

Amele, hedef sisteme göre 3 farklı edinim yöntemini destekler:

```text
┌─────────────────────────────────────────────────────────────────────────┐
│                              Amele CLI                                  │
├───────────────────┬───────────────────────────┬─────────────────────────┤
│ 1. Yerel (Local)  │ 2. Uzak Agent (TCP Stream)│ 3. Agent'sız SSH Tüneli │
│ • Raw Disk Block  │ • amele-linux agent       │ • scp / dd / raw pipe   │
│ • AVML / WinPMEM  │ • amele-win agent         │ • Parola / SSH Key      │
│ • ADB USB / Wi-Fi │ • Token Doğrulamalı       │ • Otomatik Temizlik     │
└───────────────────┴───────────────────────────┴─────────────────────────┘
```

1. **Yerel Edinim (Local):** Doğrudan çalıştırıldığı makinedeki blok cihazları veya RAM'i okur.
2. **Uzak Agent Edinimi (`--agent`):** Hedef Linux/Windows sunucuya kurulan hafif Python agent'ı üzerinden port dinler (varsayılan: 9000), token ile şifreli binary veri akışı (TCP stream) sağlar.
3. **Agent'sız SSH Edinimi (`--ssh`):** Hedef makineye hiçbir yazılım kurmadan, SSH yetkisiyle `dd` veya geçici bellek araçları çalıştırıp veriyi yerel vakaya çeker.

---

## ⚙️ Genel Seçenekler & Kabuk Tamamlama

Tüm CLI komutları için geçerli küresel bayraklar:

```bash
amele [seçenekler] <komut> [alt-komut] [argümanlar]
```

| Bayrak | Kısa | Açıklama |
| :--- | :--- | :--- |
| `--version` | `-V` | Sürüm bilgisini (`amele 0.0.19`) basar, logo ve profil gerektirmez, çıkış kodu `0`dır. |
| `--quiet` | `-q` | ASCII logo başlığını gizler (script, boru hattı ve otomasyon dostu temiz çıktı). |
| `--no-logo` | | ASCII logo başlığını bastırır (`--quiet` eşdeğeri). |
| `--verbose` | `-v` | Hata ayıklama modunu açar; tüm arka plan loglarını konsola (`stderr`) yazdırır. |
| `--lang <tr\|en>` | | CLI çalışma dilini geçici olarak Türkçe veya İngilizce seçer. |
| `--profile <ad>` | | Komutu geçici olarak belirtilen analist profili ile çalıştırır. |
| `--json` | | Sonuçları JSON formatında yapılandırılmış veri olarak döndürür. |
| `--help` | `-h` | Komut veya alt komut kullanım kılavuzunu ekrana basar. |

### Kabuk Otomatik Tamamlama (Shell Autocompletion)

Bash, Zsh ve Fish kabukları için tam tamamlama desteği dahildir:

```bash
# Bash: Anlık oturumda etkinleştirme
source <(amele completion bash)
# Bash: Kalıcı sistem kurulumu
sudo amele completion bash > /etc/bash_completion.d/amele

# Zsh: Anlık oturumda etkinleştirme
source <(amele completion zsh)
# Zsh: Kalıcı kurulum (fpath dizinine ekleme)
amele completion zsh > ~/.zsh/completion/_amele

# Fish: Kalıcı kurulum
amele completion fish > ~/.config/fish/completions/amele.fish
```

---

## 💻 CLI Komut Referansı

---

### 1. Linux & Windows Adli Edinimi

#### 📀 Disk Edinimi (`amele linux disk` / `amele windows disk` / `amele disk`)

* **Yerel Diskleri Listeleme:**
  ```bash
  amele linux disk --list
  amele windows disk --list
  amele disk --list
  ```

* **Yerel Disk İmajı Alma:**
  ```bash
  # amele linux disk [acquire] <kaynak> <vaka_adi> [disk_adi] [raw|aff4]
  amele linux disk /dev/nvme0n1 vaka_2026_01 disk1 raw
  amele linux disk acquire /dev/sda vaka_2026_01 disk1 raw
  amele windows disk \\.\PhysicalDrive0 vaka_2026_01 os_disk aff4
  ```

* **Disk İmajı Yapısal Analizi:**
  ```bash
  # amele linux disk analyze <imaj_dosyasi> [mount_klasoru]
  amele linux disk analyze /delil/disk1.raw
  ```

* **Uzak Agent Üzerinden Disk Listeleme:**
  ```bash
  # amele linux disk --agent <ip> <port> --list [token]
  amele linux disk --agent 192.168.1.100 9000 --list gizli_token_123
  ```

* **Uzak Agent Üzerinden Disk İmajı Alma:**
  ```bash
  # amele linux disk --agent <ip> <port> <disk_id> <cikti_klasoru> [token] [raw|aff4]
  amele linux disk --agent 192.168.1.100 9000 0 /home/ra/Amele/vaka1/outputs gizli_token_123 raw
  ```

* **SSH ile Agent'sız Disk Edinimi:**
  ```bash
  # Diskleri listele:
  amele linux disk --ssh 192.168.1.100 22 --list root parola123

  # İmaj al:
  # amele linux disk --ssh <ip> <port> <disk_id> <vaka> [kullanici] [parola] [key_yolu] [raw|aff4]
  amele linux disk --ssh 192.168.1.100 22 /dev/sda vaka1 root parola123 ~/.ssh/id_rsa raw
  ```

---

#### 🧠 RAM Edinimi & Analizi (`amele linux ram` / `amele windows ram` / `amele ram`)

* **Yerel Araç Durumu (AVML / WinPMEM):**
  ```bash
  amele linux ram --status
  amele windows ram --status
  ```

* **AVML / WinPMEM Otomatik İndirme ve Kurulumu:**
  ```bash
  # Linux: Microsoft AVML aracını GitHub release üzerinden otomatik kurar (sudo)
  amele linux ram install

  # Windows: WinPMEM signed sürücüsünü otomatik indirir
  amele windows ram install
  ```

* **Yerel Canlı RAM İmajı Alma:**
  ```bash
  # amele linux ram [acquire] <vaka_adi> [arac_yolu] [raw|aff4]
  amele linux ram vaka_2026_01
  amele linux ram acquire vaka_2026_01
  amele windows ram vaka_2026_01 C:\tools\winpmem.exe raw
  ```

* **RAM İmajı Volatility & Adli Analiz Komutları:**
  ```bash
  # 1. RAM İmajı Özet ve Profil Analizi (Kernel, mimari, zaman)
  amele linux ram analyze /delil/ram.raw [linux|windows]

  # 2. IOC ve Metin Dizgisi Araması (Regex, IP, domain, URL)
  amele linux ram strings /delil/ram.raw

  # 3. Dosya Çıkarma (Forensic Carving - PDF, ELF, PE, JPEG vb.)
  amele linux ram carve /delil/ram.raw /cikti/klasoru

  # 4. Bellek İçi Çalışan Süreçleri (Processes) Dökümleme
  amele linux ram processes /delil/ram.raw [linux|windows]
  ```

* **Uzak Agent Üzerinden RAM İmajı Alma:**
  ```bash
  # amele linux ram --agent <ip> <port> <vaka_adi> [token] [raw|aff4]
  amele linux ram --agent 192.168.1.100 9000 vaka1 gizli_token_123 raw
  amele windows ram --agent 192.168.1.105 9000 vaka1 gizli_token_123 aff4
  ```

* **SSH ile Agent'sız Canlı RAM Edinimi:**
  ```bash
  # amele linux ram --ssh <ip> <port> <vaka_adi> [kullanici] [parola] [key_yolu] [raw|aff4]
  amele linux ram --ssh 192.168.1.100 22 vaka1 root parola123 ~/.ssh/id_rsa raw
  ```

---

### 2. Android Mobil Adli Bilişim

Android edinimleri ADB USB, Wi-Fi veya Mesh TCP tünelleri üzerinden çalışır:

```bash
# 1. ADB Kurulum ve Cihaz Kontrolü
amele android status                          # ADB durumunu denetle
amele android install                         # ADB yoksa sistem paketiyle otomatik kur
amele android devices                         # USB/Ağdaki bağlı cihazları listele

# 2. Cihaz Analizi & Donanım Uyumluluğu
amele android profile <seri_no>               # Şifreleme, root, build, donanım detayları
amele android capabilities <seri_no>          # Hangi edinim yöntemlerinin desteklendiği raporu
amele android lemon <seri_no>                 # Lemon kernel modülü ile fiziksel RAM ön kontrolü

# 3. Veri Edinimi Modları
# Mantıksal veri çekme (quick|full|root|volatile)
amele android logical <seri_no> <vaka> full

# Dosya sistemi /data yedeği (Root yetkisiyle)
amele android filesystem <seri_no> <vaka> --root

# Canlı Android RAM edinimi (volatile|root|physical)
amele android ram <seri_no> <vaka> physical --root

# 4. Uzak Ağ Bağlantıları
amele android connect 192.168.1.150 5555 tcp "hedef_telefon"
amele android disconnect 192.168.1.150:5555

# 5. Vaka İnceleme Özeti
amele android analysis <vaka_adi>
```

---

### 3. iOS Mobil Adli Bilişim

iTunes veya libimobiledevice formatındaki iOS yedeklerinin metadata ve adli normalizasyonu:

```bash
# iOS yedeğinin metadata ve cihaz bilgilerini incele
amele ios profile /path/to/ios_backup

# iOS yedeğindeki verileri analiz edilebilir standart vaka klasörüne aktar
amele ios normalize /path/to/ios_backup vaka_2026_01
```

---

### 4. Docker Konteyner Adli Bilişimi

Çalışan veya durdurulmuş Docker konteynerlerinden UpperDir katmanı, yapılandırma ve logların izolasyonu:

```bash
# Yerel Docker İncelemesi
amele docker status                           # Daemon durumu ve escape riskleri
amele docker list                             # Konteynerleri, yetkileri ve secret risklerini listele
amele docker logs <konteyner_id> 500          # Son 500 satır logu oku
amele docker acquire <konteyner_id> vaka1     # UpperDir drift + configs + logs delillerini vakaya al

# Uzak Sunucu Docker Edinimi (Agent Üzerinden)
amele docker --agent 192.168.1.100 9000 status token123
amele docker --agent 192.168.1.100 9000 list token123
amele docker --agent 192.168.1.100 9000 logs <konteyner_id> 200 token123
amele docker --agent 192.168.1.100 9000 acquire <konteyner_id> vaka1 token123
```

---

### 5. Profil Yönetimi

Yerel analist profilleri ve `amele.noirlang.tr` lisans eşitlemesi:

```bash
amele profile list                            # Yerel ve aktif profilleri listele
amele profile create "Melih Emik" melih tr dark --direct  # Yeni profil oluştur
amele profile use melih --direct              # Aktif profili değiştir
amele profile logout                          # Otomatik girişi kapat
amele profile sync                            # Online lisans ve yetkileri senkronize et
```

---

### 6. Vaka Paketleme, Bilgi & Doğrulama

Oluşturulan vakaların incelenmesi ve taşınabilir `.amelecase` adli arşiv paketine dönüştürülmesi:

```bash
# Vakaları listele
amele case list

# Yeni vaka deposu aç
amele case create vaka1

# Vaka detaylarını, delil dosyalarını ve günlük istatistiklerini görüntüle
amele case info vaka1

# Vakayı taşınabilir imzalı .amelecase dosyasına aktar
amele case export vaka1 /delil/vaka1.amelecase

# Başka bir bilgisayardan alınan .amelecase paketini içeri aktar (Zip Slip korumalı)
amele case import /delil/vaka1.amelecase

# .amelecase paketinin SHA-256 ve delil bütünlüğünü doğrula
amele case verify /delil/vaka1.amelecase
```

---

### 7. Bütünlük & İmaj Bağlama

```bash
# Dosya Hash Hesaplama
amele hash /path/to/evidence.raw sha256       # md5, sha1, sha256, sha512

# İmaj Bütünlük Doğrulama
amele verify /path/to/evidence.raw 7f83b1657ff1fc53b92dc18148a1d65dfc2d4b1fa3d677284addd200126d9069

# Adli İmaj Bağlama (Mount)
amele mount /path/to/evidence.raw             # Salt-okunur loop/partition mount (sudo)
amele mount list                              # Aktif bağlı imajları listele
amele mount cleanup [vaka]                    # Bağlı imajları güvenle çöz (unmount)
amele mount unmount [vaka]                    # cleanup aliası
```

---

### 8. Sistem, Dil & Güncelleme

```bash
amele --lang en                               # CLI dilini İngilizce yap
amele --lang tr                               # CLI dilini Türkçe yap
amele ui                                      # Native masaüstü penceresini aç
amele ui-browser                              # Web tarayıcısında debug modunda aç
amele update                                  # Güncellemeleri kontrol et (AppImage, deb, rpm, arch, msi)
amele update --json                           # Güncelleme bilgisini JSON olarak ver
amele wireguard /tmp/wg0.conf                 # Varsayılan güvenli WireGuard VPN yapılandırması üret
```

---

## 🚦 Standart POSIX Çıkış Kodları

CLI komutları otomatik script entegrasyonları için standart POSIX çıkış kodlarını kullanır:

| Kod | Durum | Açıklama |
| :---: | :--- | :--- |
| **`0`** | `SUCCESS` | Komut hatasız tamamlandı (`--version`, `--help`, `completion`, adli edinim vb.). |
| **`1`** | `RUNTIME_ERROR` | Genel çalışma hatası (dosya sistemi, ağ veya donanım okuma hatası). |
| **`2`** | `USAGE_SYNTAX_ERROR` | Komut sözdizimi, eksik veya geçersiz argüman hatası. |
| **`126`** | `PERMISSION_DENIED` | Root veya yönetici yetkisi reddedildi (sudo başarısız / erişim engellendi). |
| **`127`** | `COMMAND_NOT_FOUND` | İkili dosya veya harici bağımlılık bulunamadı (`avml`, `winpmem`, `adb`). |

---

## 🌐 Uzak Sunuculara Agent Kurulum Kılavuzu

### Linux Sunucuya Agent Kurulumu (`amele-linux`)

`amele-linux`, hedef Linux sunucularda çalışan hafif, tek dosya Python tabanlı adli bilişim servisidir.

#### 1. Gereksinimler & Dosyaları Aktarma
Hedef makinede `python3` bulunmalıdır. Sunucuya dosyayı aktarın:
```bash
# SSH üzerinden agent dosyasını sunucuya kopyalayın
scp /home/ra/Projects/amele-pack/amele-linux-next/linux.py root@192.168.1.100:/opt/amele-linux.py
```

#### 2. Canlı Olarak Çalıştırma (Hızlı Triage)
```bash
# Hedef sunucuda:
sudo python3 /opt/amele-linux.py --port 9000 --token "GucluGizliToken2026!"
```

#### 3. Systemd Servisi Olarak Kurma (Kalıcı Sunucu Ajanı)
Sunucunun arka planında sürekli çalışmasını sağlamak için:

1. Servis dosyasını oluşturun: `/etc/systemd/system/amele-agent.service`
```ini
[Unit]
Description=Amele Linux Forensic Agent
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt
ExecStart=/usr/bin/python3 /opt/amele-linux.py --port 9000 --token "GucluGizliToken2026!"
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

2. Servisi aktif edip başlatın:
```bash
sudo systemctl daemon-reload
sudo systemctl enable amele-agent
sudo systemctl start amele-agent
sudo systemctl status amele-agent
```

3. Güvenlik Duvarı (Firewall) Ayarı:
```bash
# UFW (Ubuntu/Debian)
sudo ufw allow 9000/tcp

# Firewalld (RHEL/CentOS/Rocky)
sudo firewall-cmd --add-port=9000/tcp --permanent
sudo firewall-cmd --reload
```

---

### Windows Sunucuya Agent Kurulumu (`amele-win`)

`amele-win`, Windows 10/11 ve Windows Server 2016-2025 sistemlerde WinPMEM ve PhysicalDrive erişimi sağlayan servistir.

#### 1. Manuel Çalıştırma (Yönetici Olarak)
1. `windows.py` veya derlenmiş `amele-win.exe` dosyasını `C:\AmeleAgent\` klasörüne kopyalayın.
2. **Yönetici olarak açılan PowerShell/CMD** terminalinde:
```powershell
python windows.py --port 9000 --token "GucluGizliToken2026!"
```

#### 2. Windows Servisi (NSSM ile Kalıcı Kurulum)
```powershell
# NSSM ile servis olarak kaydetme
nssm install AmeleAgent "C:\Python311\python.exe" "C:\AmeleAgent\windows.py --port 9000 --token GucluGizliToken2026!"
nssm start AmeleAgent

# Windows Güvenlik Duvarında Port Açma:
New-NetFirewallRule -DisplayName "Amele Forensic Agent" -Direction Inbound -LocalPort 9000 -Protocol TCP -Action Allow
```

---

### SSH ile Agent'sız Doğrudan Edinim

Hedef sunucuya herhangi bir agent kurmak istemiyorsanız veya kurma izniniz yoksa, yalnızca SSH kimlik bilgisiyle adli edinim yapabilirsiniz:

```bash
# 1. SSH üzerinden hedef diskleri inceleme:
amele linux disk --ssh 192.168.1.100 22 --list root parola123

# 2. SSH üzerinden disk imajını doğrudan yerel vakaya akıtma:
amele linux disk --ssh 192.168.1.100 22 /dev/nvme0n1 vaka_sunucu root parola123 ~/.ssh/id_rsa raw

# 3. SSH üzerinden canlı RAM dökümü alma:
amele linux ram --ssh 192.168.1.100 22 vaka_sunucu root parola123
```

---

## 🎯 Örnek Olay Müdahale & Edinim Senaryoları

### Senaryo 1: Ele Geçirilen Bir Linux Web Sunucusunun Tam Triage'ı
1. **Adli Vakayı Başlatın:**
   ```bash
   amele profile use melih --direct
   ```
2. **Canlı RAM'i Alın (Uzak Agent):**
   ```bash
   amele linux ram --agent 10.0.0.50 9000 vaka_incident_01 token123 raw
   ```
3. **Konteyner Log ve Driftlerini İnceleyin:**
   ```bash
   amele docker --agent 10.0.0.50 9000 list token123
   amele docker --agent 10.0.0.50 9000 acquire suspicious_nginx vaka_incident_01 token123
   ```
4. **Kök Diskin Adli İmajını Alın:**
   ```bash
   amele linux disk --agent 10.0.0.50 9000 0 /home/ra/Amele/vaka_incident_01/outputs token123 aff4
   ```
5. **Vakayı İnceleyin ve İmzalı Pakete Dönüştürün:**
   ```bash
   amele case info vaka_incident_01
   amele case export vaka_incident_01 /delil/vaka_incident_01.amelecase
   amele case verify /delil/vaka_incident_01.amelecase
   ```

---

## 🛡️ Güvenlik Mimarisi & Sertleştirme

Amele codebase'i, adli bilişim standartlarına uygun sıkı güvenlik filtreleri barındırır:

1. **Zip Slip & Arşiv Koruması:**
   - `.amelecase` paketleri içe aktarılırken dosya yolları taranır. `ParentDir` (`..`), kök dizin kaçışları ve hedef vaka sınırları dışına dosya yazımı engellenir.
2. **Dizin Atlatma (Path Traversal) Engelleme:**
   - Vaka adları ve dosya kökleri (`sanitize_case_name`, `EvidenceVault::create`) sıkı beyaz liste denetiminden geçer; `.` veya `/` barındıran güvensiz vaka adları reddedilir.
3. **SSH Komut Enjeksiyonu Koruması:**
   - Uzak disk yolları (`disk_path`) kabuk kaçış karakterlerine (`"`, `'`, `;`, `&`, `|`, `` ` ``, `$`) karşı doğrulanır.
4. **Güvenli Çalışma Dizini (`secure_runtime_dir`):**
   - Paylaşımlı ve dünyaca yazılabilir `/tmp` dizini yerine `$XDG_RUNTIME_DIR/amele` (/run/user/<uid>) veya `0700` izinli `~/.amele/run` dizini kullanılır. Bu sayede yerel sembolik bağ (symlink) saldırıları ve yarış koşulları (race conditions) engellenir.
5. **Alt Kabuk İzolasyonu:**
   - Harici ikili arama fonksiyonları `sh -c` çağırmak yerine doğrudan Rust tabanlı `split_paths` kullanarak ortam değişkeni zehirlenmesini önler.

---

## 🔍 Sorun Giderme İlkeleri

1. **`No authentication agent found` / Yetki Hatası (Çıkış Kodu: 126):**
   - Linux'ta `sudo` veya `pkexec` yetkisi için terminalde `sudo -v` çalıştırın veya GUI polkit agent kurun.
2. **`ADB / AVML bulunamadı` Uyarısı (Çıkış Kodu: 127):**
   - `amele android install` ve `amele linux ram install` komutları gerekli araçları otomatik kurar.
3. **Uzak Agent Bağlantı Sorunları:**
   - IP, Port (9000) ve Token değerlerini kontrol edin.
   - Hedef makinede güvenlik duvarının (UFW/Firewalld/Windows Defender Firewall) 9000 portuna izin verdiğinden emin olun.
4. **Geriye Dönük Uyumluluk (Legacy Alias):**
   - Eski düz komutlar (`disk-list`, `local-image`, `remote-ram`, `adb-status`) tamamen çalışmaya devam eder.
