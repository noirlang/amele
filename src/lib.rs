//! ============================================================================
//! # AMELE FORENSIC TOOL - GENEL SİSTEM MİMARİSİ VE KÜTÜPHANE GİRİŞİ
//! ============================================================================
//!
//! Amele, uçtan uca adli bilişim (forensic) veri edinimi, delil yönetimi,
//! bellek (RAM) ve disk analizi yapabilen; yüksek performanslı, bellek güvenli
//! ve sıfır dış bağımlılık (zero external runtime dependency) ilkesiyle tasarlanmış
//! yeni nesil bir adli inceleme platformudur.
//!
//! ## 🏛️ TEMEL MİMARİ PRENSİPLER
//!
//! 1. **RFC 3227 - Order of Volatility (Uçuculuk Sırası) Uyumu:**
//!    Canlı sistem incelemelerinde en uçucu delil olan RAM ve ağ durumundan başlanarak
//!    en kalıcı olan disk ve harici depolamaya doğru sıra izlenir.
//!
//! 2. **Zero-Footprint (Hedefte Sıfır İz Bırakma):**
//!    Uzak veya yerel incelemelerde hedef makinenin sabit diskine hiçbir geçici veri
//!    yazılmaz. Çıktılar stdout veya borular üzerinden şifreli tünelle çekilir;
//!    böylece silinmiş dosyaların veya boş alanların (unallocated space) ezilmesi önlenir.
//!
//! 3. **In-Flight Streaming & Dual Hashing (Uçuş Anında Çift Özetleme):**
//!    Veri akarken (streaming) aynı 4 MB'lık bellek tamponu (buffer) üzerinden hem
//!    hedefe yazılır hem de SHA-256 ve MD5 hash bağlamları eşzamanlı güncellenir.
//!    İşlem bittiğinde delil özetleri anında hazırdır; diski 2. kez okuma gerekmez.
//!
//! 4. **Gömülü Sıfır Bağımlılık (Zero-Dependency Embedded Engine):**
//!    Harici web sunucusu (Nginx, Apache vb.) veya ağır web çatıları (Actix, Axum)
//!    yerine doğrudan standart kütüphanenin `std::net::TcpListener` yapısı kullanılır.
//!    Tüm kullanıcı arayüzü (UI) HTML/JS varlıkları derleme anında ikiliye (binary) gömülür.
//!
//! 5. **Hibrit Çoklu Masaüstü Pencere Motoru:**
//!    Sistemde Chromium tabanlı bir motor (Chrome, Chromium, Brave, Edge) varsa
//!    120 FPS donanım hızlandırmalı izole kiosk penceresi (`--app`) açılır;
//!    bulunamazsa doğrudan C FFI bağlantısıyla WebKitGTK / GTK3 penceresine dönülür.
//!
//! 6. **Adli Delil Zinciri ve Kasa (Evidence Vault & Chain of Custody):**
//!    Tüm işlemler operatör profiliyle ilişkilendirilir, her vaka izole bir dizin
//!    ağacında tutulur ve her dosya `vaka.json` manifestosunda SHA-256 ile mühürlenir.
//! ============================================================================

pub mod android;
pub mod android_analysis;
pub mod android_mft;
pub mod api;
pub mod case_package;
pub mod completion;
pub mod diagnostics;
pub mod disk;
pub mod disk_analysis;
pub mod docker;
pub mod error;
pub mod evidence;
pub mod hash;
pub mod ios;
pub mod job;
pub mod logging;
pub mod mount_tracker;
pub mod native_window;
pub mod output_format;
pub mod profile;
pub mod ram;
pub mod ram_analysis;
pub mod remote;
pub mod report;
pub mod router;
pub mod server;
pub mod settings;
pub mod ssh;
pub mod storage_guard;
pub mod volatility;
pub mod wireguard;

pub use error::{AmeleError, AmeleResult, ErrorInfo, HataKodu};
