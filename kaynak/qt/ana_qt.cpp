#include <QApplication>
#include <QMainWindow>
#include <QWidget>
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QTabWidget>
#include <QLabel>
#include <QLineEdit>
#include <QPushButton>
#include <QProgressBar>
#include <QTextEdit>
#include <QTextBrowser>
#include <QGroupBox>
#include <QComboBox>
#include <QCheckBox>
#include <QFrame>
#include <QMessageBox>
#include <QDateTime>
#include <QCoreApplication>
#include <QMetaObject>
#include <QCloseEvent>
#include <QSysInfo>
#include <QFileInfo>
#include <QFileDialog>
#include <QDialog>
#include <QFormLayout>
#include <QDialogButtonBox>
#include <QSpinBox>
#include <QInputDialog>
#include <QDir>
#include <QProcess>
#include <QFile>
#include <QJsonArray>
#include <QJsonDocument>
#include <QJsonObject>
#include <QStackedWidget>
#include <QTreeView>
#include <QFileSystemModel>
#include <QToolButton>
#include <QMenuBar>
#include <QMenu>
#include <QAction>
#include <QStyle>
#include <QPixmap>
#include <QIcon>
#include <QFont>
#include <QPainter>
#include <QRegularExpression>
#include <QStandardPaths>

#include <atomic>
#include <chrono>
#include <thread>
#include <vector>
#include <string>
#include <cstring>
#include <locale.h>
#include <unordered_map>

#include <glib.h>

extern "C" {
#include "gunluk.h"
#include "ayarlar.h"
#include "is_kuyrugu.h"
#include "kanit_kasasi.h"
#include "hash.h"
#include "rapor.h"
#include "disk_edinim.h"
#include "uzak_disk_edinim.h"
#include "winpmem_ram.h"
#include "avml_ram.h"
#include "wireguard.h"
}

class AnaPencere : public QMainWindow {
public:
    AnaPencere() {
        kurulum_cekirdek();
        kurulum_arayuz();
        log_ekle("Arayuz olusturuldu", GUNLUK_SEVIYE_INFO);
        log_ekle("Worm basladi", GUNLUK_SEVIYE_INFO);
        log_ekle("Sistem hazir", GUNLUK_SEVIYE_INFO);
    }

    ~AnaPencere() override {
        temizle();
    }

protected:
    void closeEvent(QCloseEvent* event) override {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma surerken uygulama kapatilamaz.");
            event->ignore();
            return;
        }
        temizle();
        QMainWindow::closeEvent(event);
    }

private:
    char* ayar_dosya_yolu_ = nullptr;
    UygulamaAyarlar* ayarlar_ = nullptr;
    GunlukYonetici* gunluk_ = nullptr;
    IsKuyrugu* is_kuyrugu_ = nullptr;
    KanitKasasi* kasa_ = nullptr;
    UzakDiskBaglanti* baglanti_ = nullptr;

    std::vector<UzakDisk> uzak_diskler_;
    std::atomic<bool> imaj_calisiyor_ = false;
    std::atomic<bool> baglaniyor_ = false;

    QLabel* durum_label_ = nullptr;
    QProgressBar* genel_ilerleme_ = nullptr;
    QTextEdit* gunluk_metin_ = nullptr;
    QStackedWidget* icerik_stack_ = nullptr;

    QToolButton* top_ana_menu_btn_ = nullptr;
    QToolButton* top_sistem_btn_ = nullptr;
    QToolButton* top_hakkinda_btn_ = nullptr;
    QToolButton* top_ayarlar_btn_ = nullptr;

    QIcon top_ana_menu_icon_;
    QIcon top_sistem_icon_;
    QIcon top_hakkinda_icon_;
    QIcon top_ayarlar_icon_;

    int sayfa_ana_menu_idx_ = -1;
    int sayfa_sistem_idx_ = -1;
    int sayfa_hakkinda_idx_ = -1;
    int sayfa_ayarlar_idx_ = -1;

    QLineEdit* ip_giris_ = nullptr;
    QLineEdit* port_giris_ = nullptr;
    QLineEdit* token_giris_ = nullptr;
    QPushButton* guvenlik_onay_btn_ = nullptr;
    QPushButton* guvenlik_sifirla_btn_ = nullptr;
    QLabel* guvenlik_durum_label_ = nullptr;
    QString onayli_guvenlik_anahtari_;
    QCheckBox* vpn_kullan_secim_ = nullptr;
    QPushButton* vpn_yapilandir_btn_ = nullptr;
    QComboBox* disk_secim_ = nullptr;
    QLineEdit* cikti_klasor_giris_ = nullptr;
    QPushButton* baglan_btn_ = nullptr;
    QPushButton* disk_getir_btn_ = nullptr;
    QPushButton* imaj_btn_ = nullptr;
    QProgressBar* uzak_ilerleme_ = nullptr;
    QLabel* uzak_durum_label_ = nullptr;
    QProgressBar* aktif_uzak_ilerleme_ = nullptr;

    QLineEdit* linux_uzak_ip_giris_ = nullptr;
    QLineEdit* linux_uzak_port_giris_ = nullptr;
    QLineEdit* linux_uzak_token_giris_ = nullptr;
    QComboBox* linux_uzak_disk_secim_ = nullptr;
    QLineEdit* linux_uzak_cikti_klasor_giris_ = nullptr;
    QPushButton* linux_uzak_baglan_btn_ = nullptr;
    QPushButton* linux_uzak_disk_getir_btn_ = nullptr;
    QPushButton* linux_uzak_imaj_btn_ = nullptr;
    QProgressBar* linux_uzak_ilerleme_ = nullptr;
    QLabel* linux_uzak_durum_label_ = nullptr;
    QCheckBox* linux_uzak_vpn_kullan_secim_ = nullptr;
    QPushButton* linux_uzak_vpn_yapilandir_btn_ = nullptr;
    WireGuardYonetici* vpn_yonetici_ = nullptr;
    QString vpn_config_yolu_;

    QComboBox* yerel_disk_secim_ = nullptr;
    QLineEdit* yerel_cikti_klasor_giris_ = nullptr;
    QPushButton* yerel_disk_getir_btn_ = nullptr;
    QPushButton* yerel_imaj_btn_ = nullptr;
    QProgressBar* yerel_ilerleme_ = nullptr;
    QLabel* yerel_durum_label_ = nullptr;
    std::vector<DiskBilgisi> yerel_diskler_;

    QLineEdit* hash_dosya_giris_ = nullptr;
    QLabel* md5_label_ = nullptr;
    QLabel* sha1_label_ = nullptr;
    QLabel* sha256_label_ = nullptr;
    QLabel* sha512_label_ = nullptr;
    QLineEdit* hash_karsilastir_giris_ = nullptr;
    QLabel* hash_sonuc_label_ = nullptr;

    QLineEdit* vaka_giris_ = nullptr;
    QLabel* vaka_durum_label_ = nullptr;
    QComboBox* dosya_listesi_ = nullptr;
    QComboBox* klasor_secim_ = nullptr;

    QLineEdit* rapor_baslik_giris_ = nullptr;
    QTextEdit* rapor_not_giris_ = nullptr;
    QComboBox* rapor_format_secim_ = nullptr;
    QLabel* rapor_durum_label_ = nullptr;

    QTextEdit* gunluk_sekme_metin_ = nullptr;

    // WinPMEM RAM UI elemanlari (Hyper-V yerine)
    QPushButton* winpmem_kontrol_btn_ = nullptr;
    QPushButton* winpmem_baslat_btn_ = nullptr;
    QPushButton* winpmem_indir_btn_ = nullptr;
    QProgressBar* winpmem_ilerleme_ = nullptr;
    QLabel* winpmem_durum_label_ = nullptr;
    QLabel* winpmem_bilgi_label_ = nullptr;
    QLineEdit* winpmem_cikti_giris_ = nullptr;
    QLineEdit* winpmem_ip_giris_ = nullptr;
    QLineEdit* winpmem_port_giris_ = nullptr;
    QLineEdit* winpmem_token_giris_ = nullptr;
    QPushButton* winpmem_baglan_btn_ = nullptr;
    QCheckBox* winpmem_vpn_kullan_secim_ = nullptr;
    QPushButton* winpmem_vpn_yapilandir_btn_ = nullptr;
    WinPMEMEdinim* winpmem_edinim_ = nullptr;
    bool winpmem_indirme_asamasi_ = false;
    int winpmem_son_log_oran_ = -1;

    // Windows yerel RAM edinimi
    QPushButton* winpmem_yerel_kontrol_btn_ = nullptr;
    QPushButton* winpmem_yerel_indir_btn_ = nullptr;
    QPushButton* winpmem_yerel_baslat_btn_ = nullptr;
    QProgressBar* winpmem_yerel_ilerleme_ = nullptr;
    QLabel* winpmem_yerel_durum_label_ = nullptr;
    QLabel* winpmem_yerel_bilgi_label_ = nullptr;
    QLineEdit* winpmem_yerel_cikti_giris_ = nullptr;
    WinPMEMEdinim* winpmem_yerel_edinim_ = nullptr;

    // Linux yerel RAM edinimi (AVML)
    QPushButton* avml_kontrol_btn_ = nullptr;
    QPushButton* avml_baslat_btn_ = nullptr;
    QProgressBar* avml_ilerleme_ = nullptr;
    QLabel* avml_durum_label_ = nullptr;
    QLabel* avml_bilgi_label_ = nullptr;
    QLineEdit* avml_cikti_giris_ = nullptr;
    AVMLEdinim* avml_edinim_ = nullptr;

    // Linux uzak RAM edinimi (AVML)
    QLineEdit* linux_uzak_ram_ip_giris_ = nullptr;
    QLineEdit* linux_uzak_ram_port_giris_ = nullptr;
    QLineEdit* linux_uzak_ram_token_giris_ = nullptr;
    QLineEdit* linux_uzak_ram_cikti_giris_ = nullptr;
    QPushButton* linux_uzak_ram_baglan_btn_ = nullptr;
    QPushButton* linux_uzak_ram_kontrol_btn_ = nullptr;
    QPushButton* linux_uzak_ram_baslat_btn_ = nullptr;
    QPushButton* linux_uzak_ram_indir_btn_ = nullptr;
    QCheckBox* linux_uzak_ram_vpn_kullan_secim_ = nullptr;
    QPushButton* linux_uzak_ram_vpn_yapilandir_btn_ = nullptr;
    QProgressBar* linux_uzak_ram_ilerleme_ = nullptr;
    QLabel* linux_uzak_ram_bilgi_label_ = nullptr;
    QLabel* linux_uzak_ram_durum_label_ = nullptr;

    QCheckBox* ayar_karanlik_tema_secim_ = nullptr;
    QComboBox* ayar_dil_secim_ = nullptr;
    QLabel* ayar_durum_label_ = nullptr;

    // Imaj goruntuleme
    QLineEdit* imaj_yol_giris_ = nullptr;
    QPushButton* imaj_sec_btn_ = nullptr;
    QPushButton* imaj_bagla_btn_ = nullptr;
    QPushButton* imaj_ayir_btn_ = nullptr;
    QLabel* imaj_durum_label_ = nullptr;
    QTreeView* imaj_agac_ = nullptr;
    QFileSystemModel* imaj_model_ = nullptr;
    QString bagli_loop_aygiti_;
    QString bagli_mount_aygiti_;
    QString bagli_mount_noktasi_;

    bool temizlendi_ = false;
    QString aktif_dil_ = "tr";
    std::unordered_map<std::string, std::string> ceviri_tr_en_;
    std::unordered_map<std::string, std::string> ceviri_en_tr_;

    static void ilerleme_kopru(int64_t okunan, int64_t toplam, void* kullanici_verisi) {
        AnaPencere* pencere = static_cast<AnaPencere*>(kullanici_verisi);
        if (!pencere) {
            return;
        }
        pencere->ilerleme_guncelle_is_parcacigi(okunan, toplam);
    }

    const char* metin_dil(const char* turkce, const char* ingilizce) const {
        if (ayarlar_ && strcmp(ayarlar_->dil, "en") == 0) {
            return ingilizce;
        }
        return turkce;
    }

    static std::string qstr_std(const QString& s) {
        QByteArray ba = s.toUtf8();
        return std::string(ba.constData(), static_cast<size_t>(ba.size()));
    }

    QString cevir_metin(const QString& metin) const {
        if (metin.isEmpty()) {
            return metin;
        }

        const std::string anahtar = qstr_std(metin);
        if (aktif_dil_ == "en") {
            auto it = ceviri_tr_en_.find(anahtar);
            if (it != ceviri_tr_en_.end()) {
                return QString::fromUtf8(it->second.c_str());
            }
        } else {
            auto it = ceviri_en_tr_.find(anahtar);
            if (it != ceviri_en_tr_.end()) {
                return QString::fromUtf8(it->second.c_str());
            }
        }
        return metin;
    }

    bool dil_sozlugu_yukle() {
        ceviri_tr_en_.clear();
        ceviri_en_tr_.clear();

        const QString app_dir = QCoreApplication::applicationDirPath();
        const QStringList adaylar = {
            app_dir + "/veriler/ui_dil.json",
            app_dir + "/../veriler/ui_dil.json",
            app_dir + "/../../veriler/ui_dil.json",
            QDir::currentPath() + "/veriler/ui_dil.json",
            QDir::currentPath() + "/../veriler/ui_dil.json"
        };

        QString bulunan;
        for (const QString& yol : adaylar) {
            if (QFileInfo::exists(yol)) {
                bulunan = yol;
                break;
            }
        }

        if (bulunan.isEmpty()) {
            return false;
        }

        QFile dosya(bulunan);
        if (!dosya.open(QIODevice::ReadOnly)) {
            return false;
        }

        QJsonParseError json_hata;
        const QJsonDocument doc = QJsonDocument::fromJson(dosya.readAll(), &json_hata);
        if (json_hata.error != QJsonParseError::NoError || !doc.isObject()) {
            return false;
        }

        const QJsonObject kok = doc.object();
        const QJsonArray dizi = kok.value("entries").toArray();
        for (const QJsonValue& v : dizi) {
            const QJsonObject obj = v.toObject();
            const QString tr = obj.value("tr").toString();
            const QString en = obj.value("en").toString();
            if (tr.isEmpty() || en.isEmpty()) {
                continue;
            }
            ceviri_tr_en_[qstr_std(tr)] = qstr_std(en);
            ceviri_en_tr_[qstr_std(en)] = qstr_std(tr);
        }

        return !ceviri_tr_en_.empty();
    }

    void widget_metinlerini_cevir(QWidget* kok) {
        if (!kok) {
            return;
        }

        const auto labels = kok->findChildren<QLabel*>();
        for (QLabel* w : labels) {
            w->setText(cevir_metin(w->text()));
        }

        const auto buttons = kok->findChildren<QPushButton*>();
        for (QPushButton* w : buttons) {
            w->setText(cevir_metin(w->text()));
        }

        const auto checks = kok->findChildren<QCheckBox*>();
        for (QCheckBox* w : checks) {
            w->setText(cevir_metin(w->text()));
        }

        const auto groups = kok->findChildren<QGroupBox*>();
        for (QGroupBox* w : groups) {
            w->setTitle(cevir_metin(w->title()));
        }

        const auto edits = kok->findChildren<QLineEdit*>();
        for (QLineEdit* w : edits) {
            w->setPlaceholderText(cevir_metin(w->placeholderText()));
        }

        const auto combos = kok->findChildren<QComboBox*>();
        for (QComboBox* w : combos) {
            for (int i = 0; i < w->count(); ++i) {
                w->setItemText(i, cevir_metin(w->itemText(i)));
            }
        }

        const auto sekmeler = kok->findChildren<QTabWidget*>();
        for (QTabWidget* w : sekmeler) {
            for (int i = 0; i < w->count(); ++i) {
                w->setTabText(i, cevir_metin(w->tabText(i)));
            }
        }
    }

    void tema_uygula(bool karanlik) {
        if (karanlik) {
            qApp->setStyleSheet(
                "QWidget { background-color: #000000; color: #f2f5f8; font-family: 'Manrope', 'Noto Sans', 'Segoe UI Variable Text', 'Ubuntu'; font-size: 11pt; }"
                "QMainWindow { background-color: #000000; border: none; }"
                "QWidget#RootPanel { background-color: #000000; border: none; }"
                "QStackedWidget, QStackedWidget > QWidget { background-color: #000000; border: none; }"
                "QTabWidget::pane { background-color: #000000; border: 1px solid #161616; border-radius: 10px; }"
                "QGroupBox { border: 1px solid #202020; border-radius: 10px; margin-top: 10px; padding: 10px; background-color: #050505; }"
                "QGroupBox::title { subcontrol-origin: margin; left: 10px; padding: 0 6px; color: #d7dde3; }"
                "QFrame#AboutHeaderCard { background-color: #0b0b0b; border: 1px solid #242424; border-radius: 12px; }"
                "QLineEdit, QTextEdit, QComboBox, QProgressBar {"
                "  background-color: #080808; border: 1px solid #242424; border-radius: 8px; padding: 5px 7px; color: #f2f5f8;"
                "}"
                "QLineEdit:focus, QTextEdit:focus, QComboBox:focus { border: 1px solid #8ab4ff; }"
                "QPushButton {"
                "  background-color: #121212; border: 1px solid #2c2c2c; border-radius: 8px; padding: 6px 10px; color: #f2f5f8;"
                "}"
                "QPushButton:hover { background-color: #1b1b1b; }"
                "QPushButton:pressed { background-color: #101010; }"
                "QTabBar::tab {"
                "  background: #080808; border: 1px solid #222222; border-bottom: none; border-top-left-radius: 8px;"
                "  border-top-right-radius: 8px; padding: 7px 12px; margin-right: 4px; color: #d7dde3;"
                "}"
                "QTabBar::tab:selected { background: #111111; color: #ffffff; }"
                "QTabBar::tab:hover:!selected { background: #101010; }"
                "QMenuBar { background-color: #000000; border-top: 1px solid #1a1a1a; border-bottom: 1px solid #1a1a1a; padding: 3px 8px; }"
                "QMenuBar::item { background: transparent; color: #f2f5f8; padding: 6px 11px; border-radius: 6px; }"
                "QMenuBar::item:selected { background: #161616; }"
                "QMenu { background-color: #050505; border: 1px solid #242424; padding: 6px; color: #f2f5f8; }"
                "QMenu::item { padding: 7px 22px; border-radius: 5px; }"
                "QMenu::item:selected { background-color: #1b1b1b; }"
                "QToolButton#TopNavButton { border: none; border-radius: 7px; padding: 5px; }"
                "QToolButton#TopNavButton:hover { background-color: #1a1a1a; }"
                "QProgressBar { text-align: center; }"
                "QProgressBar::chunk { background-color: #f2f5f8; border-radius: 6px; }"
                "QScrollBar:vertical { background: #070707; width: 10px; margin: 0; }"
                "QScrollBar::handle:vertical { background: #2a2a2a; min-height: 30px; border-radius: 5px; }"
                "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height: 0; }"
            );
            ust_ikonlari_guncelle(true);
        } else {
            qApp->setStyleSheet(
                "QWidget { background-color: #f2f6fb; color: #1c2733; font-family: 'Manrope', 'Noto Sans', 'Segoe UI Variable Text', 'Ubuntu'; font-size: 11pt; }"
                "QMainWindow { background-color: #f2f6fb; border: none; }"
                "QWidget#RootPanel { background-color: #f2f6fb; border: none; }"
                "QStackedWidget, QStackedWidget > QWidget { background-color: #f2f6fb; border: none; }"
                "QTabWidget::pane { background-color: #f2f6fb; border: 1px solid #d5e0ea; border-radius: 10px; }"
                "QGroupBox { border: 1px solid #d7e1ea; border-radius: 10px; margin-top: 10px; padding: 10px; background-color: #ffffff; }"
                "QGroupBox::title { subcontrol-origin: margin; left: 10px; padding: 0 6px; color: #35526d; }"
                "QFrame#AboutHeaderCard { background-color: #ffffff; border: 1px solid #d7e1ea; border-radius: 12px; }"
                "QLineEdit, QTextEdit, QComboBox, QProgressBar {"
                "  background-color: #ffffff; border: 1px solid #c9d8e6; border-radius: 8px; padding: 5px 7px; color: #1c2733;"
                "}"
                "QLineEdit:focus, QTextEdit:focus, QComboBox:focus { border: 1px solid #2d8cff; }"
                "QPushButton {"
                "  background-color: #eef4fb; border: 1px solid #bfd2e4; border-radius: 8px; padding: 6px 10px; color: #1c3147;"
                "}"
                "QPushButton:hover { background-color: #e2edf8; }"
                "QPushButton:pressed { background-color: #d6e6f7; }"
                "QTabBar::tab {"
                "  background: #edf3f9; border: 1px solid #cbd9e7; border-bottom: none; border-top-left-radius: 8px;"
                "  border-top-right-radius: 8px; padding: 7px 12px; margin-right: 4px; color: #35526d;"
                "}"
                "QTabBar::tab:selected { background: #ffffff; color: #183047; }"
                "QTabBar::tab:hover:!selected { background: #e3edf7; }"
                "QMenuBar { background-color: #ffffff; border-top: 1px solid #d3deea; border-bottom: 1px solid #d3deea; padding: 3px 8px; }"
                "QMenuBar::item { background: transparent; color: #1f3550; padding: 6px 11px; border-radius: 6px; }"
                "QMenuBar::item:selected { background: #e7eff9; }"
                "QMenu { background-color: #ffffff; border: 1px solid #c9d8e7; padding: 6px; }"
                "QMenu::item { padding: 7px 22px; border-radius: 5px; }"
                "QMenu::item:selected { background-color: #e4eef9; }"
                "QToolButton#TopNavButton { border: none; border-radius: 7px; padding: 5px; }"
                "QToolButton#TopNavButton:hover { background-color: #e8f0fa; }"
                "QProgressBar { text-align: center; }"
                "QProgressBar::chunk { background-color: #2d8cff; border-radius: 6px; }"
                "QScrollBar:vertical { background: #eef3f8; width: 10px; margin: 0; }"
                "QScrollBar::handle:vertical { background: #c2d3e4; min-height: 30px; border-radius: 5px; }"
                "QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical { height: 0; }"
            );
            ust_ikonlari_guncelle(false);
        }
    }

    void dil_uygula(const QString& dil) {
        aktif_dil_ = (dil == "en") ? "en" : "tr";
        if (ayarlar_) {
            strncpy(ayarlar_->dil, aktif_dil_.toUtf8().constData(), sizeof(ayarlar_->dil) - 1);
            ayarlar_->dil[sizeof(ayarlar_->dil) - 1] = '\0';
        }

        setWindowTitle(cevir_metin("Worm Forensic Tool"));
        if (durum_label_) {
            durum_label_->setText(cevir_metin("Hazir"));
        }
        widget_metinlerini_cevir(centralWidget());
    }

    void kurulum_cekirdek() {
        const char* ev = g_get_home_dir();
        char* ayar_klasor = g_build_filename(ev, "Worm", nullptr);
        g_mkdir_with_parents(ayar_klasor, 0755);
        ayar_dosya_yolu_ = g_build_filename(ayar_klasor, "ayarlar.json", nullptr);
        vpn_config_yolu_ = QString::fromUtf8(ayar_klasor) + "/wg0.conf";
        g_free(ayar_klasor);

        ayarlar_ = ayarlar_yukle(ayar_dosya_yolu_);
        if (!ayarlar_) {
            ayarlar_ = static_cast<UygulamaAyarlar*>(g_malloc0(sizeof(UygulamaAyarlar)));
            ayarlar_varsayilan(ayarlar_);
        }

        char* gunluk_klasor = g_build_filename(ev, "Worm", "gunlukler", nullptr);
        gunluk_ = gunluk_baslat("Genel", gunluk_klasor);
        g_free(gunluk_klasor);

        is_kuyrugu_ = is_kuyrugu_olustur(gunluk_);
        vpn_yonetici_ = wireguard_yonetici_olustur();
        dil_sozlugu_yukle();
    }

    QString logo_dosyasi_bul(const QString& dosya_adi) const {
        const QString app_dir = QCoreApplication::applicationDirPath();
        const QStringList adaylar = {
            QDir::currentPath() + "/logo/" + dosya_adi,
            QDir::currentPath() + "/../logo/" + dosya_adi,
            app_dir + "/logo/" + dosya_adi,
            app_dir + "/../logo/" + dosya_adi,
            app_dir + "/../../logo/" + dosya_adi,
        };

        for (const QString& yol : adaylar) {
            if (QFileInfo::exists(yol)) {
                return yol;
            }
        }
        return QString();
    }

    QWidget* olustur_ana_menu_sayfasi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        duzen->addStretch();

        QLabel* logo = new QLabel();
        logo->setAlignment(Qt::AlignCenter);
        const QString logo_yol = logo_dosyasi_bul("logo.png");
        if (!logo_yol.isEmpty()) {
            QPixmap pix(logo_yol);
            if (!pix.isNull()) {
                logo->setPixmap(pix.scaled(240, 240, Qt::KeepAspectRatio, Qt::SmoothTransformation));
            }
        }
        duzen->addWidget(logo);

        QLabel* alt1 = new QLabel("Forensic Tool");
        alt1->setAlignment(Qt::AlignCenter);
        QFont f1 = alt1->font();
        f1.setPointSize(22);
        f1.setBold(true);
        alt1->setFont(f1);
        duzen->addWidget(alt1);

        QLabel* alt2 = new QLabel("Worm v0.0.1 alpha");
        alt2->setAlignment(Qt::AlignCenter);
        QFont f2 = alt2->font();
        f2.setPointSize(13);
        alt2->setFont(f2);
        duzen->addWidget(alt2);

        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_windows_agent_sayfasi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* ust = new QGroupBox("Windows Agent");
        QVBoxLayout* ust_icerik = new QVBoxLayout(ust);
        QLabel* aciklama = new QLabel();
        aciklama->setTextFormat(Qt::RichText);
        aciklama->setTextInteractionFlags(Qt::TextBrowserInteraction);
        aciklama->setOpenExternalLinks(true);
        aciklama->setText(
            "Windows Agent kullanim ozeti. Ayrintili dokuman: "
            "<a href=\"https://github.com/noirlang/worm-win\">https://github.com/noirlang/worm-win</a><br>"
            "Indirme: <a href=\"https://worm.noirlang.tr/worm-win.exe\">https://worm.noirlang.tr/worm-win.exe</a>"
        );
        aciklama->setWordWrap(true);
        ust_icerik->addWidget(aciklama);
        duzen->addWidget(ust);

        QGroupBox* win = new QGroupBox("Hizli Notlar");
        QVBoxLayout* win_icerik = new QVBoxLayout(win);
        QTextBrowser* win_metin = new QTextBrowser();
        win_metin->setReadOnly(true);
        win_metin->setOpenExternalLinks(true);
        win_metin->setHtml(
            "TR:<br>"
            "1) Agent indirin: wget -O worm-win.exe "
            "<a href=\"https://worm.noirlang.tr/worm-win.exe\">https://worm.noirlang.tr/worm-win.exe</a><br>"
            "2) Windows'ta worm-win.exe dosyasini yonetici olarak calistirin.<br>"
            "3) Ana uygulamadaki IP/Port bilgisi ile eslestirin.<br><br>"
            "EN:<br>"
            "1) Download agent: wget -O worm-win.exe "
            "<a href=\"https://worm.noirlang.tr/worm-win.exe\">https://worm.noirlang.tr/worm-win.exe</a><br>"
            "2) Run worm-win.exe as Administrator on Windows.<br>"
            "3) Match IP/Port values with the main Worm application."
        );
        win_icerik->addWidget(win_metin);
        duzen->addWidget(win);

        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_linux_agent_sayfasi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* ust = new QGroupBox("Linux Agent");
        QVBoxLayout* ust_icerik = new QVBoxLayout(ust);
        QLabel* aciklama = new QLabel();
        aciklama->setTextFormat(Qt::RichText);
        aciklama->setTextInteractionFlags(Qt::TextBrowserInteraction);
        aciklama->setOpenExternalLinks(true);
        aciklama->setText(
            "Linux Agent kullanim ozeti. Ayrintili dokuman: "
            "<a href=\"https://github.com/noirlang/worm-linux\">https://github.com/noirlang/worm-linux</a><br>"
            "Indirme: <a href=\"https://worm.noirlang.tr/worm-linux\">https://worm.noirlang.tr/worm-linux</a>"
        );
        aciklama->setWordWrap(true);
        ust_icerik->addWidget(aciklama);
        duzen->addWidget(ust);

        QGroupBox* lin = new QGroupBox("Hizli Notlar");
        QVBoxLayout* lin_icerik = new QVBoxLayout(lin);
        QTextBrowser* lin_metin = new QTextBrowser();
        lin_metin->setReadOnly(true);
        lin_metin->setOpenExternalLinks(true);
        lin_metin->setHtml(
            "TR:<br>"
            "1) Agent indirin: wget -O worm-linux "
            "<a href=\"https://worm.noirlang.tr/worm-linux\">https://worm.noirlang.tr/worm-linux</a><br>"
            "2) Yetki verin: chmod +x worm-linux<br>"
            "3) Calistirin: ./worm-linux<br>"
            "4) Ana uygulamadaki IP/Port ile baglanti kurun.<br><br>"
            "EN:<br>"
            "1) Download agent: wget -O worm-linux "
            "<a href=\"https://worm.noirlang.tr/worm-linux\">https://worm.noirlang.tr/worm-linux</a><br>"
            "2) Make it executable: chmod +x worm-linux<br>"
            "3) Run: ./worm-linux<br>"
            "4) Connect using the same IP/Port from the main Worm app."
        );
        lin_icerik->addWidget(lin_metin);
        duzen->addWidget(lin);

        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_imaj_goruntuleme_sayfasi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Imaj Goruntuleme");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);

        QLabel* aciklama = new QLabel(
            "Secilen disk imajini salt-okunur olarak baglar ve icerigini klasor agacinda gosterir.\n"
            "Linux'ta udisksctl kullanilir; baglama islemi basarisizsa durum bilgisinde sebebi gorursunuz."
        );
        aciklama->setWordWrap(true);
        icerik->addWidget(aciklama);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Imaj Dosyasi:"));
            imaj_yol_giris_ = new QLineEdit();
            imaj_yol_giris_->setPlaceholderText(".img, .dd, .raw, .iso ...");
            satir->addWidget(imaj_yol_giris_, 1);
            imaj_sec_btn_ = new QPushButton("Dosya Sec");
            satir->addWidget(imaj_sec_btn_);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            imaj_bagla_btn_ = new QPushButton("Salt-Okunur Bagla");
            imaj_ayir_btn_ = new QPushButton("Baglantiyi Kaldir");
            imaj_ayir_btn_->setEnabled(false);
            satir->addWidget(imaj_bagla_btn_);
            satir->addWidget(imaj_ayir_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        imaj_durum_label_ = new QLabel("Hazir");
        icerik->addWidget(imaj_durum_label_);

        imaj_model_ = new QFileSystemModel(sayfa);
        imaj_model_->setFilter(QDir::AllEntries | QDir::NoDotAndDotDot | QDir::AllDirs);
        imaj_model_->setRootPath(QDir::homePath());

        imaj_agac_ = new QTreeView();
        imaj_agac_->setModel(imaj_model_);
        imaj_agac_->setRootIndex(imaj_model_->index(QDir::homePath()));
        imaj_agac_->setAlternatingRowColors(true);
        imaj_agac_->setMinimumHeight(380);
        imaj_agac_->setSortingEnabled(true);
        icerik->addWidget(imaj_agac_, 1);

        connect(imaj_sec_btn_, &QPushButton::clicked, this, [this]() { imaj_dosya_sec(); });
        connect(imaj_bagla_btn_, &QPushButton::clicked, this, [this]() { imaj_bagla(); });
        connect(imaj_ayir_btn_, &QPushButton::clicked, this, [this]() { imaj_baglanti_kaldir(); });

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    bool komut_calistir_ve_oku(const QString& program,
                               const QStringList& args,
                               QString* cikti,
                               QString* hata,
                               int timeout_ms = 30000) const {
        QProcess proc;
        proc.start(program, args);
        if (!proc.waitForStarted(5000)) {
            if (cikti) {
                *cikti = QString();
            }
            if (hata) {
                *hata = "Komut baslatilamadi: " + program;
            }
            return false;
        }

        if (!proc.waitForFinished(timeout_ms)) {
            proc.kill();
            if (cikti) {
                *cikti = QString::fromUtf8(proc.readAllStandardOutput());
            }
            if (hata) {
                *hata = "Komut zaman asimina ugratildi: " + program;
            }
            return false;
        }

        if (cikti) {
            *cikti = QString::fromUtf8(proc.readAllStandardOutput());
        }
        if (hata) {
            *hata = QString::fromUtf8(proc.readAllStandardError());
        }

        return proc.exitStatus() == QProcess::NormalExit && proc.exitCode() == 0;
    }

    QStringList imaj_baglanabilir_aygitlari(const QString& loop_aygit) const {
        QStringList fs_partitions;
        QStringList any_partitions;

        QString cikti;
        QString hata;
        if (komut_calistir_ve_oku("lsblk", QStringList() << "-lnpo" << "NAME,TYPE,FSTYPE" << loop_aygit, &cikti, &hata, 8000)) {
            const QStringList satirlar = cikti.split('\n', Qt::SkipEmptyParts);
            for (const QString& satir : satirlar) {
                const QStringList parcalar = satir.simplified().split(' ');
                if (parcalar.size() < 2) {
                    continue;
                }

                const QString aygit = parcalar.at(0).trimmed();
                const QString tip = parcalar.at(1).trimmed();
                const QString fs_tipi = (parcalar.size() >= 3) ? parcalar.at(2).trimmed() : QString();

                if (tip == "part") {
                    any_partitions << aygit;
                    if (!fs_tipi.isEmpty()) {
                        fs_partitions << aygit;
                    }
                }
            }
        }

        if (!fs_partitions.isEmpty()) {
            return fs_partitions;
        }
        if (!any_partitions.isEmpty()) {
            return any_partitions;
        }

        return QStringList() << loop_aygit;
    }

    QString imaj_mount_noktasi_bul(const QString& aygit) const {
        QString cikti;
        QString hata;
        if (!komut_calistir_ve_oku("findmnt", QStringList() << "-nr" << "-o" << "TARGET" << "-S" << aygit, &cikti, &hata, 8000)) {
            return QString();
        }

        const QString ilk_satir = cikti.split('\n', Qt::SkipEmptyParts).value(0).trimmed();
        if (ilk_satir.startsWith('/')) {
            return ilk_satir;
        }
        return QString();
    }

    void imaj_dosya_sec() {
        if (!imaj_yol_giris_) {
            return;
        }

        const QString secilen = QFileDialog::getOpenFileName(
            this,
            "Imaj Dosyasi Sec",
            QDir::homePath(),
            "Disk Imajlari (*.img *.dd *.raw *.iso *.bin *.001 *.e01);;Tum Dosyalar (*)"
        );

        if (!secilen.isEmpty()) {
            imaj_yol_giris_->setText(secilen);
        }
    }

    void imaj_baglanti_kaldir(bool sessiz = false) {
#ifdef _WIN32
        if (!sessiz) {
            QMessageBox::information(this, "Bilgi", "Bu ozellik su an Linux odakli uygulanmistir.");
        }
        return;
#else
        if (bagli_mount_aygiti_.isEmpty() && bagli_loop_aygiti_.isEmpty()) {
            if (!sessiz && imaj_durum_label_) {
                imaj_durum_label_->setText("Bagli imaj yok");
            }
            return;
        }

        QString cikti;
        QString hata;

        if (!bagli_mount_aygiti_.isEmpty()) {
            komut_calistir_ve_oku("udisksctl", QStringList() << "unmount" << "-b" << bagli_mount_aygiti_, &cikti, &hata, 20000);
        }
        if (!bagli_loop_aygiti_.isEmpty()) {
            komut_calistir_ve_oku("udisksctl", QStringList() << "loop-delete" << "-b" << bagli_loop_aygiti_, &cikti, &hata, 20000);
        }

        bagli_mount_aygiti_.clear();
        bagli_loop_aygiti_.clear();
        bagli_mount_noktasi_.clear();

        if (imaj_model_ && imaj_agac_) {
            imaj_agac_->setRootIndex(imaj_model_->index(QDir::homePath()));
        }

        if (imaj_bagla_btn_) {
            imaj_bagla_btn_->setEnabled(true);
        }
        if (imaj_ayir_btn_) {
            imaj_ayir_btn_->setEnabled(false);
        }

        if (imaj_durum_label_) {
            imaj_durum_label_->setText("Hazir");
        }

        if (!sessiz) {
            log_ekle("Imaj baglantisi kaldirildi", GUNLUK_SEVIYE_INFO);
        }
#endif
    }

    void imaj_bagla() {
#ifdef _WIN32
        QMessageBox::warning(this, "Bilgi", "Imaj baglama goruntuleyici su an Linux odakli uygulanmistir.");
        return;
#else
        if (!imaj_yol_giris_) {
            return;
        }

        const QString imaj_yolu = imaj_yol_giris_->text().trimmed();
        if (imaj_yolu.isEmpty()) {
            QMessageBox::warning(this, "Bilgi", "Lutfen once imaj dosyasi secin.");
            return;
        }

        QFileInfo fi(imaj_yolu);
        if (!fi.exists() || !fi.isFile()) {
            QMessageBox::critical(this, "Hata", "Imaj dosyasi bulunamadi veya gecersiz.");
            return;
        }

        if (QStandardPaths::findExecutable("udisksctl").isEmpty()) {
            QMessageBox::critical(this, "Hata", "udisksctl bulunamadi. Lutfen udisks2 kurun.");
            return;
        }

        imaj_baglanti_kaldir(true);

        if (imaj_durum_label_) {
            imaj_durum_label_->setText("Imaj baglaniyor...");
        }

        QString loop_out;
        QString loop_err;
        if (!komut_calistir_ve_oku("udisksctl", QStringList() << "loop-setup" << "-r" << "-f" << imaj_yolu, &loop_out, &loop_err, 25000)) {
            QMessageBox::critical(this, "Hata", "Loop baglama basarisiz:\n" + (loop_out + "\n" + loop_err).trimmed());
            if (imaj_durum_label_) {
                imaj_durum_label_->setText("Imaj baglama basarisiz");
            }
            return;
        }

        QRegularExpression loop_re("(/dev/loop\\d+)");
        QRegularExpressionMatch loop_match = loop_re.match(loop_out + "\n" + loop_err);
        if (!loop_match.hasMatch()) {
            QMessageBox::critical(this, "Hata", "Loop aygiti tespit edilemedi.\n" + (loop_out + "\n" + loop_err).trimmed());
            if (imaj_durum_label_) {
                imaj_durum_label_->setText("Loop aygiti tespit edilemedi");
            }
            return;
        }

        bagli_loop_aygiti_ = loop_match.captured(1);

        // Partition tablosu gec algilanabiliyor; birkac denemeyle mount adaylarini topla.
        QStringList mount_adaylari;
        for (int deneme = 0; deneme < 6; ++deneme) {
            mount_adaylari = imaj_baglanabilir_aygitlari(bagli_loop_aygiti_);
            if (!(mount_adaylari.size() == 1 && mount_adaylari.first() == bagli_loop_aygiti_)) {
                break;
            }
            std::this_thread::sleep_for(std::chrono::milliseconds(300));
        }

        QString mount_aygit;
        QString mount_out;
        QString mount_err;
        bool mount_ok = false;
        QString tum_hatalar;

        for (const QString& aday : mount_adaylari) {
            QString aday_out;
            QString aday_err;
            if (komut_calistir_ve_oku("udisksctl", QStringList() << "mount" << "-b" << aday, &aday_out, &aday_err, 25000)) {
                mount_ok = true;
                mount_aygit = aday;
                mount_out = aday_out;
                mount_err = aday_err;
                break;
            }

            tum_hatalar += "[" + aday + "]\n" + (aday_out + "\n" + aday_err).trimmed() + "\n\n";
        }

        if (!mount_ok) {
            QMessageBox::critical(this, "Hata", "Imaj mount islemi basarisiz:\n" + tum_hatalar.trimmed());
            imaj_baglanti_kaldir(true);
            if (imaj_durum_label_) {
                imaj_durum_label_->setText("Imaj mount basarisiz");
            }
            return;
        }

        bagli_mount_aygiti_ = mount_aygit;
        QRegularExpression mnt_re(" at (/.+?)(?:\\.|\\n|$)");
        QRegularExpressionMatch mnt_match = mnt_re.match(mount_out + "\n" + mount_err);
        if (mnt_match.hasMatch()) {
            bagli_mount_noktasi_ = mnt_match.captured(1).trimmed();
        }
        if (bagli_mount_noktasi_.isEmpty()) {
            bagli_mount_noktasi_ = imaj_mount_noktasi_bul(bagli_mount_aygiti_);
        }

        if (bagli_mount_noktasi_.isEmpty()) {
            QMessageBox::warning(this, "Bilgi", "Mount noktasi tespit edilemedi ama aygit baglanmis olabilir.");
            if (imaj_durum_label_) {
                imaj_durum_label_->setText("Mount noktasi tespit edilemedi");
            }
            return;
        }

        if (imaj_model_ && imaj_agac_) {
            imaj_model_->setRootPath(bagli_mount_noktasi_);
            imaj_agac_->setRootIndex(imaj_model_->index(bagli_mount_noktasi_));
            imaj_agac_->resizeColumnToContents(0);
        }

        if (imaj_bagla_btn_) {
            imaj_bagla_btn_->setEnabled(false);
        }
        if (imaj_ayir_btn_) {
            imaj_ayir_btn_->setEnabled(true);
        }
        if (imaj_durum_label_) {
            imaj_durum_label_->setText("Baglandi: " + bagli_mount_noktasi_);
        }

        log_ekle("Imaj baglandi: " + imaj_yolu + " -> " + bagli_mount_noktasi_, GUNLUK_SEVIYE_INFO);
#endif
    }

    QIcon tema_ikonu(const QString& ad, QStyle::StandardPixmap yedek) const {
        const QIcon tema = QIcon::fromTheme(ad);
        if (!tema.isNull()) {
            return tema;
        }
        return style()->standardIcon(yedek);
    }

    QIcon icon_renklendir(const QIcon& kaynak, const QColor& renk) const {
        if (kaynak.isNull()) {
            return QIcon();
        }

        auto boya = [&kaynak, &renk](const QSize& boyut) -> QPixmap {
            QPixmap temel = kaynak.pixmap(boyut);
            if (temel.isNull()) {
                return QPixmap();
            }

            QPixmap hedef(temel.size());
            hedef.fill(Qt::transparent);

            QPainter cizici(&hedef);
            cizici.drawPixmap(0, 0, temel);
            cizici.setCompositionMode(QPainter::CompositionMode_SourceIn);
            cizici.fillRect(hedef.rect(), renk);
            cizici.end();
            return hedef;
        };

        QIcon sonuc;
        const QPixmap kucuk = boya(QSize(18, 18));
        const QPixmap buyuk = boya(QSize(22, 22));
        if (!kucuk.isNull()) {
            sonuc.addPixmap(kucuk, QIcon::Normal, QIcon::Off);
        }
        if (!buyuk.isNull()) {
            sonuc.addPixmap(buyuk, QIcon::Normal, QIcon::On);
        }
        return sonuc;
    }

    void ust_ikonlari_guncelle(bool karanlik) {
        const QColor ikon_renk = karanlik ? QColor("#ffffff") : QColor("#24374a");
        if (top_ana_menu_btn_) {
            top_ana_menu_btn_->setIcon(icon_renklendir(top_ana_menu_icon_, ikon_renk));
        }
        if (top_sistem_btn_) {
            top_sistem_btn_->setIcon(icon_renklendir(top_sistem_icon_, ikon_renk));
        }
        if (top_hakkinda_btn_) {
            top_hakkinda_btn_->setIcon(icon_renklendir(top_hakkinda_icon_, ikon_renk));
        }
        if (top_ayarlar_btn_) {
            top_ayarlar_btn_->setIcon(icon_renklendir(top_ayarlar_icon_, ikon_renk));
        }
    }

    void kurulum_arayuz() {
        setWindowTitle("Worm Forensic Tool");
        resize(1340, 860);

        QFont arayuz_font;
        arayuz_font.setFamilies(QStringList() << "Manrope" << "Noto Sans" << "Segoe UI Variable Text" << "Ubuntu" << "Arial");
        arayuz_font.setPointSize(11);
        qApp->setFont(arayuz_font);

        QWidget* merkez = new QWidget(this);
        merkez->setObjectName("RootPanel");
        QVBoxLayout* ana = new QVBoxLayout(merkez);
        ana->setContentsMargins(0, 0, 0, 0);
        ana->setSpacing(0);

        QHBoxLayout* ust = new QHBoxLayout();
        ust->setContentsMargins(12, 8, 12, 8);
        ust->setSpacing(10);

        QLabel* app_baslik = new QLabel("Worm");
        app_baslik->setObjectName("AppBrand");
        QFont app_font;
        app_font.setFamilies(QStringList() << "Ubuntu" << "Cantarell" << "Noto Sans" << "DejaVu Sans");
        app_font.setPointSize(24);
        app_font.setWeight(QFont::DemiBold);
        app_font.setLetterSpacing(QFont::PercentageSpacing, 103.0);
        app_baslik->setFont(app_font);
        ust->addWidget(app_baslik);

        top_ana_menu_btn_ = new QToolButton();
        top_ana_menu_btn_->setObjectName("TopNavButton");
        top_ana_menu_icon_ = tema_ikonu("go-home-symbolic", QStyle::SP_DirHomeIcon);
        top_ana_menu_btn_->setIcon(top_ana_menu_icon_);
        top_ana_menu_btn_->setToolTip("Ana Menu");
        top_ana_menu_btn_->setAutoRaise(true);
        ust->addWidget(top_ana_menu_btn_);

        ust->addStretch();

        top_sistem_btn_ = new QToolButton();
        top_sistem_btn_->setObjectName("TopNavButton");
        top_sistem_icon_ = tema_ikonu("computer-symbolic", QStyle::SP_ComputerIcon);
        top_sistem_btn_->setIcon(top_sistem_icon_);
        top_sistem_btn_->setToolTip("Sistem Bilgisi");
        top_sistem_btn_->setAutoRaise(true);
        ust->addWidget(top_sistem_btn_);

        top_hakkinda_btn_ = new QToolButton();
        top_hakkinda_btn_->setObjectName("TopNavButton");
        top_hakkinda_icon_ = tema_ikonu("help-about-symbolic", QStyle::SP_MessageBoxInformation);
        top_hakkinda_btn_->setIcon(top_hakkinda_icon_);
        top_hakkinda_btn_->setToolTip("Hakkinda");
        top_hakkinda_btn_->setAutoRaise(true);
        ust->addWidget(top_hakkinda_btn_);

        top_ayarlar_btn_ = new QToolButton();
        top_ayarlar_btn_->setObjectName("TopNavButton");
        top_ayarlar_icon_ = tema_ikonu("preferences-system-symbolic", QStyle::SP_FileDialogDetailedView);
        top_ayarlar_btn_->setIcon(top_ayarlar_icon_);
        top_ayarlar_btn_->setToolTip("Ayarlar");
        top_ayarlar_btn_->setAutoRaise(true);
        ust->addWidget(top_ayarlar_btn_);

        ana->addLayout(ust);

        QMenuBar* menu_cubugu = new QMenuBar();
        menu_cubugu->setNativeMenuBar(false);
        ana->addWidget(menu_cubugu);

        genel_ilerleme_ = new QProgressBar();
        genel_ilerleme_->setRange(0, 100);
        genel_ilerleme_->setValue(0);
        genel_ilerleme_->setTextVisible(false);
        genel_ilerleme_->setFixedHeight(12);
        ana->addWidget(genel_ilerleme_);

        icerik_stack_ = new QStackedWidget();
        ana->addWidget(icerik_stack_, 1);

        auto menu_sayfa_ekle = [this](QMenu* menu, const QString& metin, QWidget* sayfa) -> int {
            const int idx = icerik_stack_->addWidget(sayfa);
            QAction* aksiyon = menu->addAction(metin);
            connect(aksiyon, &QAction::triggered, this, [this, idx]() {
                if (icerik_stack_ && idx >= 0 && idx < icerik_stack_->count()) {
                    icerik_stack_->setCurrentIndex(idx);
                }
            });
            return idx;
        };

        QMenu* menu_windows = menu_cubugu->addMenu("Windows Araclari");
        QMenu* menu_linux = menu_cubugu->addMenu("Linux Araclari");
        QMenu* menu_agent = menu_cubugu->addMenu("Agent");
        QMenu* menu_analiz = menu_cubugu->addMenu("Analiz");
        QMenu* menu_diger = menu_cubugu->addMenu("Diger");

        sayfa_ana_menu_idx_ = icerik_stack_->addWidget(olustur_ana_menu_sayfasi());

        menu_sayfa_ekle(menu_windows, "Windows Uzak Disk Imaj Alma", olustur_uzak_disk_sekmesi());
        menu_sayfa_ekle(menu_windows, "Windows Yerel Imaj Alma", olustur_windows_yerel_disk_sekmesi());
        menu_sayfa_ekle(menu_windows, "Windows Uzak RAM Alma", olustur_windows_uzak_ram_sekmesi());
        menu_sayfa_ekle(menu_windows, "Windows Yerel RAM Alma", olustur_windows_yerel_ram_sekmesi());

        menu_sayfa_ekle(menu_linux, "Linux Uzak Disk Imaj Alma", olustur_linux_uzak_disk_sekmesi());
        menu_sayfa_ekle(menu_linux, "Linux Yerel Imaj Alma", olustur_yerel_disk_sekmesi());
        menu_sayfa_ekle(menu_linux, "Linux Uzak RAM Alma", olustur_linux_uzak_ram_sekmesi());
        menu_sayfa_ekle(menu_linux, "Linux Yerel RAM Alma", olustur_linux_yerel_ram_sekmesi());

        menu_sayfa_ekle(menu_agent, "Windows Agent", olustur_windows_agent_sayfasi());
        menu_sayfa_ekle(menu_agent, "Linux Agent", olustur_linux_agent_sayfasi());
        menu_sayfa_ekle(menu_analiz, "Imaj Goruntuleme", olustur_imaj_goruntuleme_sayfasi());

        menu_sayfa_ekle(menu_diger, "Hash Islemleri", olustur_hash_sekmesi());
        menu_sayfa_ekle(menu_diger, "Kanit Kasasi", olustur_kanit_sekmesi());
        menu_sayfa_ekle(menu_diger, "Raporlar", olustur_rapor_sekmesi());
        menu_sayfa_ekle(menu_diger, "Gunluk", olustur_gunluk_sekmesi());

        sayfa_sistem_idx_ = icerik_stack_->addWidget(olustur_sistem_bilgisi_sekmesi());
        sayfa_hakkinda_idx_ = icerik_stack_->addWidget(olustur_hakkinda_sekmesi());
        sayfa_ayarlar_idx_ = icerik_stack_->addWidget(olustur_ayarlar_sekmesi());

        auto sayfaya_git = [this](int idx) {
            if (!icerik_stack_ || idx < 0 || idx >= icerik_stack_->count()) {
                return;
            }
            icerik_stack_->setCurrentIndex(idx);
        };

        connect(top_ana_menu_btn_, &QToolButton::clicked, this, [sayfaya_git, this]() {
            sayfaya_git(sayfa_ana_menu_idx_);
        });
        connect(top_sistem_btn_, &QToolButton::clicked, this, [sayfaya_git, this]() {
            sayfaya_git(sayfa_sistem_idx_);
        });
        connect(top_hakkinda_btn_, &QToolButton::clicked, this, [sayfaya_git, this]() {
            sayfaya_git(sayfa_hakkinda_idx_);
        });
        connect(top_ayarlar_btn_, &QToolButton::clicked, this, [sayfaya_git, this]() {
            sayfaya_git(sayfa_ayarlar_idx_);
        });

        setCentralWidget(merkez);

        sayfaya_git(sayfa_ana_menu_idx_);

        if (ayarlar_) {
            tema_uygula(ayarlar_->karanlik_tema);
            dil_uygula(QString::fromUtf8(ayarlar_->dil));
        }
    }

    QWidget* olustur_uzak_disk_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Uzak Windows Sunucu Baglantisi");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(8);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("IP Adresi:"));
            ip_giris_ = new QLineEdit();
            ip_giris_->setPlaceholderText("192.168.1.100");
            satir->addWidget(ip_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Port:"));
            port_giris_ = new QLineEdit("4444");
            satir->addWidget(port_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Token (opsiyonel):"));
            token_giris_ = new QLineEdit();
            token_giris_->setPlaceholderText("Guvenlik anahtari (Onayla ile aktif olur)");
            satir->addWidget(token_giris_, 1);
            guvenlik_onay_btn_ = new QPushButton("Anahtari Onayla");
            guvenlik_sifirla_btn_ = new QPushButton("Anahtari Sifirla");
            satir->addWidget(guvenlik_onay_btn_);
            satir->addWidget(guvenlik_sifirla_btn_);
            icerik->addLayout(satir);
        }

        guvenlik_durum_label_ = new QLabel("Guvenlik anahtari: Kapali");
        icerik->addWidget(guvenlik_durum_label_);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            vpn_kullan_secim_ = new QCheckBox("WireGuard VPN Kullan");
            vpn_yapilandir_btn_ = new QPushButton("VPN Yapilandir");
            satir->addWidget(vpn_kullan_secim_);
            satir->addWidget(vpn_yapilandir_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        baglan_btn_ = new QPushButton("Baglan");
        disk_getir_btn_ = new QPushButton("Diskleri Getir");

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(baglan_btn_);
            satir->addWidget(disk_getir_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Disk:"));
            disk_secim_ = new QComboBox();
            satir->addWidget(disk_secim_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Cikti Klasoru:"));
            cikti_klasor_giris_ = new QLineEdit();
            const QString ev = QString::fromUtf8(g_get_home_dir());
            cikti_klasor_giris_->setText(ev + "/Worm/Ciktilar");
            satir->addWidget(cikti_klasor_giris_, 1);

            QPushButton* klasor_sec_btn = new QPushButton(QString::fromUtf8(metin_dil("Klasor Sec", "Browse Folder")));
            satir->addWidget(klasor_sec_btn);
            connect(klasor_sec_btn, &QPushButton::clicked, this, [this]() {
                klasor_sec_ve_ata(
                    cikti_klasor_giris_,
                    QString::fromUtf8(metin_dil("Cikti Klasoru Sec", "Select Output Folder"))
                );
            });

            icerik->addLayout(satir);
        }

        imaj_btn_ = new QPushButton("Imaj Al");
        icerik->addWidget(imaj_btn_);

        uzak_ilerleme_ = new QProgressBar();
        uzak_ilerleme_->setRange(0, 100);
        uzak_ilerleme_->setValue(0);
        icerik->addWidget(uzak_ilerleme_);

        uzak_durum_label_ = new QLabel("Baglanti yok");
        icerik->addWidget(uzak_durum_label_);

        duzen->addWidget(cerceve);
        duzen->addStretch();

        connect(baglan_btn_, &QPushButton::clicked, this, [this]() { uzak_baglan(); });
        connect(disk_getir_btn_, &QPushButton::clicked, this, [this]() { uzak_disk_getir(); });
        connect(imaj_btn_, &QPushButton::clicked, this, [this]() { uzak_imaj_baslat(); });
        connect(vpn_yapilandir_btn_, &QPushButton::clicked, this, [this]() { vpn_yapilandir(); });
        connect(guvenlik_onay_btn_, &QPushButton::clicked, this, [this]() { guvenlik_anahtari_onayla(); });
        connect(guvenlik_sifirla_btn_, &QPushButton::clicked, this, [this]() { guvenlik_anahtari_sifirla(); });

        return sayfa;
    }

    QWidget* olustur_linux_uzak_disk_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Linux Uzak Disk Imaj Alma");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(8);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("IP Adresi:"));
            linux_uzak_ip_giris_ = new QLineEdit();
            linux_uzak_ip_giris_->setPlaceholderText("192.168.1.110");
            satir->addWidget(linux_uzak_ip_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Port:"));
            linux_uzak_port_giris_ = new QLineEdit("4444");
            satir->addWidget(linux_uzak_port_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Token (opsiyonel):"));
            linux_uzak_token_giris_ = new QLineEdit();
            linux_uzak_token_giris_->setPlaceholderText("Guvenlik anahtari");
            satir->addWidget(linux_uzak_token_giris_, 1);
            icerik->addLayout(satir);
        }

        QLabel* bilgi = new QLabel("Linux ajan baglantisiyla uzak disk listesi alinip imaj edinimi baslatilir.");
        bilgi->setWordWrap(true);
        icerik->addWidget(bilgi);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            linux_uzak_vpn_kullan_secim_ = new QCheckBox("WireGuard VPN Kullan");
            linux_uzak_vpn_yapilandir_btn_ = new QPushButton("VPN Yapilandir");
            satir->addWidget(linux_uzak_vpn_kullan_secim_);
            satir->addWidget(linux_uzak_vpn_yapilandir_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        linux_uzak_baglan_btn_ = new QPushButton("Baglan");
        linux_uzak_disk_getir_btn_ = new QPushButton("Diskleri Getir");
        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(linux_uzak_baglan_btn_);
            satir->addWidget(linux_uzak_disk_getir_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Disk:"));
            linux_uzak_disk_secim_ = new QComboBox();
            satir->addWidget(linux_uzak_disk_secim_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Cikti Klasoru:"));
            linux_uzak_cikti_klasor_giris_ = new QLineEdit();
            const QString ev = QString::fromUtf8(g_get_home_dir());
            linux_uzak_cikti_klasor_giris_->setText(ev + "/Worm/Ciktilar");
            satir->addWidget(linux_uzak_cikti_klasor_giris_, 1);

            QPushButton* klasor_sec_btn = new QPushButton(QString::fromUtf8(metin_dil("Klasor Sec", "Browse Folder")));
            satir->addWidget(klasor_sec_btn);
            connect(klasor_sec_btn, &QPushButton::clicked, this, [this]() {
                klasor_sec_ve_ata(
                    linux_uzak_cikti_klasor_giris_,
                    QString::fromUtf8(metin_dil("Cikti Klasoru Sec", "Select Output Folder"))
                );
            });

            icerik->addLayout(satir);
        }

        linux_uzak_imaj_btn_ = new QPushButton("Imaj Al");
        icerik->addWidget(linux_uzak_imaj_btn_);

        linux_uzak_ilerleme_ = new QProgressBar();
        linux_uzak_ilerleme_->setRange(0, 100);
        linux_uzak_ilerleme_->setValue(0);
        icerik->addWidget(linux_uzak_ilerleme_);

        linux_uzak_durum_label_ = new QLabel("Baglanti yok");
        icerik->addWidget(linux_uzak_durum_label_);

        connect(linux_uzak_vpn_yapilandir_btn_, &QPushButton::clicked, this, [this]() { vpn_yapilandir(); });
        connect(linux_uzak_baglan_btn_, &QPushButton::clicked, this, [this]() { linux_uzak_baglan(); });
        connect(linux_uzak_disk_getir_btn_, &QPushButton::clicked, this, [this]() { linux_uzak_disk_getir(); });
        connect(linux_uzak_imaj_btn_, &QPushButton::clicked, this, [this]() { linux_uzak_imaj_baslat(); });

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_yerel_disk_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Linux Yerel Disk Imaji");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(8);

#ifdef _WIN32
        QLabel* not_label = new QLabel("Bu ozellik sadece Linux'ta calisir.");
        not_label->setWordWrap(true);
        icerik->addWidget(not_label);
        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
#else
        QLabel* not_label = new QLabel("Linux'ta yerel ham disk imaji almak icin root yetkisi gerekebilir.");
        not_label->setWordWrap(true);
        icerik->addWidget(not_label);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Kaynak Disk:"));
            yerel_disk_secim_ = new QComboBox();
            satir->addWidget(yerel_disk_secim_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Cikti Klasoru:"));
            yerel_cikti_klasor_giris_ = new QLineEdit();
            const QString ev = QString::fromUtf8(g_get_home_dir());
            yerel_cikti_klasor_giris_->setText(ev + "/Worm/Ciktilar");
            satir->addWidget(yerel_cikti_klasor_giris_, 1);

            QPushButton* klasor_sec_btn = new QPushButton(QString::fromUtf8(metin_dil("Klasor Sec", "Browse Folder")));
            satir->addWidget(klasor_sec_btn);
            connect(klasor_sec_btn, &QPushButton::clicked, this, [this]() {
                klasor_sec_ve_ata(
                    yerel_cikti_klasor_giris_,
                    QString::fromUtf8(metin_dil("Cikti Klasoru Sec", "Select Output Folder"))
                );
            });

            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            yerel_disk_getir_btn_ = new QPushButton("Yerel Diskleri Getir");
            yerel_imaj_btn_ = new QPushButton("Yerel Imaj Al");
            satir->addWidget(yerel_disk_getir_btn_);
            satir->addWidget(yerel_imaj_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        yerel_ilerleme_ = new QProgressBar();
        yerel_ilerleme_->setRange(0, 100);
        yerel_ilerleme_->setValue(0);
        icerik->addWidget(yerel_ilerleme_);

        yerel_durum_label_ = new QLabel("Hazir");
        icerik->addWidget(yerel_durum_label_);

        duzen->addWidget(cerceve);
        duzen->addStretch();

        connect(yerel_disk_getir_btn_, &QPushButton::clicked, this, [this]() { yerel_disk_getir(); });
        connect(yerel_imaj_btn_, &QPushButton::clicked, this, [this]() { yerel_imaj_baslat(); });

        return sayfa;
#endif
    }

    QWidget* olustur_windows_yerel_disk_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Windows Yerel Disk Imaji");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(8);

#ifdef _WIN32
        QLabel* bilgi = new QLabel("Windows yerel ham disk imaji almak icin uygulamayi Yonetici olarak calistirin.");
        bilgi->setWordWrap(true);
        icerik->addWidget(bilgi);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Kaynak Disk:"));
            yerel_disk_secim_ = new QComboBox();
            satir->addWidget(yerel_disk_secim_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Cikti Klasoru:"));
            yerel_cikti_klasor_giris_ = new QLineEdit();
            const QString ev = QString::fromUtf8(g_get_home_dir());
            yerel_cikti_klasor_giris_->setText(ev + "/Worm/Ciktilar");
            satir->addWidget(yerel_cikti_klasor_giris_, 1);

            QPushButton* klasor_sec_btn = new QPushButton(QString::fromUtf8(metin_dil("Klasor Sec", "Browse Folder")));
            satir->addWidget(klasor_sec_btn);
            connect(klasor_sec_btn, &QPushButton::clicked, this, [this]() {
                klasor_sec_ve_ata(
                    yerel_cikti_klasor_giris_,
                    QString::fromUtf8(metin_dil("Cikti Klasoru Sec", "Select Output Folder"))
                );
            });

            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            yerel_disk_getir_btn_ = new QPushButton("Yerel Diskleri Getir");
            yerel_imaj_btn_ = new QPushButton("Yerel Imaj Al");
            satir->addWidget(yerel_disk_getir_btn_);
            satir->addWidget(yerel_imaj_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        yerel_ilerleme_ = new QProgressBar();
        yerel_ilerleme_->setRange(0, 100);
        yerel_ilerleme_->setValue(0);
        icerik->addWidget(yerel_ilerleme_);

        yerel_durum_label_ = new QLabel("Hazir");
        icerik->addWidget(yerel_durum_label_);

        connect(yerel_disk_getir_btn_, &QPushButton::clicked, this, [this]() { yerel_disk_getir(); });
        connect(yerel_imaj_btn_, &QPushButton::clicked, this, [this]() { yerel_imaj_baslat(); });
#else
        QLabel* bilgi = new QLabel("Bu ozellik sadece Windows'ta calisir.");
        bilgi->setWordWrap(true);
        icerik->addWidget(bilgi);
#endif

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_windows_uzak_ram_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Windows Uzak RAM Edinimi (WinPMEM)");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(8);

        QLabel* bilgi = new QLabel(
            "WinPMEM ile canli sistemden fiziksel RAM imaji alin.\n"
            "Bu islem yonetici yetkisi gerektirir ve .raw formatinda kaydedilir.\n"
            "Cikti dosyasi Volatility3 ile analiz edilebilir."
        );
        bilgi->setWordWrap(true);
        icerik->addWidget(bilgi);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("IP Adresi:"));
            winpmem_ip_giris_ = new QLineEdit();
            winpmem_ip_giris_->setPlaceholderText("192.168.1.100");
            satir->addWidget(winpmem_ip_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Port:"));
            winpmem_port_giris_ = new QLineEdit("4444");
            satir->addWidget(winpmem_port_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Token (opsiyonel):"));
            winpmem_token_giris_ = new QLineEdit();
            winpmem_token_giris_->setPlaceholderText("Guvenlik anahtari");
            satir->addWidget(winpmem_token_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            winpmem_baglan_btn_ = new QPushButton("RAM Sunucusuna Baglan");
            satir->addWidget(winpmem_baglan_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            winpmem_vpn_kullan_secim_ = new QCheckBox("WireGuard VPN Kullan");
            winpmem_vpn_yapilandir_btn_ = new QPushButton("VPN Yapilandir");
            satir->addWidget(winpmem_vpn_kullan_secim_);
            satir->addWidget(winpmem_vpn_yapilandir_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            winpmem_kontrol_btn_ = new QPushButton("WinPMEM Kontrol Et");
            winpmem_bilgi_label_ = new QLabel("Kontrol edilmedi");
            winpmem_bilgi_label_->setWordWrap(true);
            satir->addWidget(winpmem_kontrol_btn_);
            satir->addWidget(winpmem_bilgi_label_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Cikti Dosyasi:"));
            winpmem_cikti_giris_ = new QLineEdit();
            winpmem_cikti_giris_->setText("memory_dump.raw");
            winpmem_cikti_giris_->setPlaceholderText("memory_dump.raw");
            satir->addWidget(winpmem_cikti_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            winpmem_baslat_btn_ = new QPushButton("RAM Edinimini Baslat");
            winpmem_indir_btn_ = new QPushButton("RAM Indir");
            winpmem_baslat_btn_->setEnabled(false);
            winpmem_indir_btn_->setEnabled(false);
            satir->addWidget(winpmem_baslat_btn_);
            satir->addWidget(winpmem_indir_btn_);
            icerik->addLayout(satir);
        }

        winpmem_ilerleme_ = new QProgressBar();
        winpmem_ilerleme_->setRange(0, 100);
        winpmem_ilerleme_->setValue(0);
        icerik->addWidget(winpmem_ilerleme_);

        winpmem_durum_label_ = new QLabel("Hazir - once baglanti yapin ve kontrol edin");
        icerik->addWidget(winpmem_durum_label_);

        duzen->addWidget(cerceve);
        duzen->addStretch();

        connect(winpmem_baglan_btn_, &QPushButton::clicked, this, [this]() {
            if (!winpmem_ip_giris_ || !winpmem_port_giris_) {
                return;
            }

            const QString ip = winpmem_ip_giris_->text().trimmed();
            const QString port = winpmem_port_giris_->text().trimmed();
            const QString token = winpmem_token_giris_ ? winpmem_token_giris_->text().trimmed() : QString();

            if (ip_giris_) {
                ip_giris_->setText(ip);
            }
            if (port_giris_) {
                port_giris_->setText(port);
            }

            if (token_giris_) {
                token_giris_->setText(token);
                if (token.isEmpty()) {
                    onayli_guvenlik_anahtari_.clear();
                    token_giris_->setReadOnly(false);
                    if (guvenlik_durum_label_) {
                        guvenlik_durum_label_->setText("Guvenlik anahtari: Kapali");
                    }
                } else {
                    onayli_guvenlik_anahtari_ = token;
                    token_giris_->setReadOnly(true);
                    if (guvenlik_durum_label_) {
                        guvenlik_durum_label_->setText("Guvenlik anahtari: Aktif");
                    }
                }
            }

            if (winpmem_durum_label_) {
                winpmem_durum_label_->setText("Baglanti deneniyor...");
            }

            if (vpn_kullan_secim_ && winpmem_vpn_kullan_secim_) {
                vpn_kullan_secim_->setChecked(winpmem_vpn_kullan_secim_->isChecked());
            }
            uzak_baglan();
        });

        connect(winpmem_vpn_yapilandir_btn_, &QPushButton::clicked, this, [this]() { vpn_yapilandir(); });

        connect(winpmem_kontrol_btn_, &QPushButton::clicked, this, [this]() { winpmem_kontrol_yap(); });
        connect(winpmem_baslat_btn_, &QPushButton::clicked, this, [this]() { winpmem_baslat(); });
        connect(winpmem_indir_btn_, &QPushButton::clicked, this, [this]() { winpmem_indir(); });

        return sayfa;
    }

    QWidget* olustur_linux_uzak_ram_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Linux Uzak RAM Edinimi (AVML)");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(8);

        QLabel* bilgi = new QLabel(
            "Linux ajaninda AVML ile RAM imaji olusturulur ve bu makineye indirilebilir."
        );
        bilgi->setWordWrap(true);
        icerik->addWidget(bilgi);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("IP Adresi:"));
            linux_uzak_ram_ip_giris_ = new QLineEdit();
            linux_uzak_ram_ip_giris_->setPlaceholderText("192.168.1.110");
            satir->addWidget(linux_uzak_ram_ip_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Port:"));
            linux_uzak_ram_port_giris_ = new QLineEdit("4444");
            satir->addWidget(linux_uzak_ram_port_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Token (opsiyonel):"));
            linux_uzak_ram_token_giris_ = new QLineEdit();
            linux_uzak_ram_token_giris_->setPlaceholderText("Guvenlik anahtari");
            satir->addWidget(linux_uzak_ram_token_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            linux_uzak_ram_vpn_kullan_secim_ = new QCheckBox("WireGuard VPN Kullan");
            linux_uzak_ram_vpn_yapilandir_btn_ = new QPushButton("VPN Yapilandir");
            satir->addWidget(linux_uzak_ram_vpn_kullan_secim_);
            satir->addWidget(linux_uzak_ram_vpn_yapilandir_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            linux_uzak_ram_baglan_btn_ = new QPushButton("Baglan");
            linux_uzak_ram_kontrol_btn_ = new QPushButton("AVML Kontrol Et");
            satir->addWidget(linux_uzak_ram_baglan_btn_);
            satir->addWidget(linux_uzak_ram_kontrol_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        linux_uzak_ram_bilgi_label_ = new QLabel("Kontrol edilmedi");
        linux_uzak_ram_bilgi_label_->setWordWrap(true);
        icerik->addWidget(linux_uzak_ram_bilgi_label_);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Cikti Dosyasi:"));
            linux_uzak_ram_cikti_giris_ = new QLineEdit("memory_dump_linux_remote.raw");
            satir->addWidget(linux_uzak_ram_cikti_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            linux_uzak_ram_baslat_btn_ = new QPushButton("RAM Edinimini Baslat");
            linux_uzak_ram_indir_btn_ = new QPushButton("RAM Indir");
            linux_uzak_ram_baslat_btn_->setEnabled(false);
            linux_uzak_ram_indir_btn_->setEnabled(false);
            satir->addWidget(linux_uzak_ram_baslat_btn_);
            satir->addWidget(linux_uzak_ram_indir_btn_);
            icerik->addLayout(satir);
        }

        linux_uzak_ram_ilerleme_ = new QProgressBar();
        linux_uzak_ram_ilerleme_->setRange(0, 100);
        linux_uzak_ram_ilerleme_->setValue(0);
        icerik->addWidget(linux_uzak_ram_ilerleme_);

        linux_uzak_ram_durum_label_ = new QLabel("Hazir");
        icerik->addWidget(linux_uzak_ram_durum_label_);

        duzen->addWidget(cerceve);
        duzen->addStretch();

        connect(linux_uzak_ram_vpn_yapilandir_btn_, &QPushButton::clicked, this, [this]() { vpn_yapilandir(); });
        connect(linux_uzak_ram_baglan_btn_, &QPushButton::clicked, this, [this]() { linux_uzak_ram_baglan(); });
        connect(linux_uzak_ram_kontrol_btn_, &QPushButton::clicked, this, [this]() { linux_uzak_ram_kontrol_yap(); });
        connect(linux_uzak_ram_baslat_btn_, &QPushButton::clicked, this, [this]() { linux_uzak_ram_baslat(); });
        connect(linux_uzak_ram_indir_btn_, &QPushButton::clicked, this, [this]() { linux_uzak_ram_indir(); });

        return sayfa;
    }

    QWidget* olustur_windows_yerel_ram_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Windows Yerel RAM Edinimi (WinPMEM)");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(8);

#ifdef _WIN32
        QLabel* bilgi = new QLabel(
            "Bu ozellik sadece Windows'ta calisir.\n"
            "WinPMEM binary yoksa otomatik indirilebilir."
        );
#else
        QLabel* bilgi = new QLabel("Bu ozellik sadece Windows'ta calisir.");
#endif
        bilgi->setWordWrap(true);
        icerik->addWidget(bilgi);

#ifdef _WIN32
        {
            QHBoxLayout* satir = new QHBoxLayout();
            winpmem_yerel_kontrol_btn_ = new QPushButton("WinPMEM Kontrol Et");
            winpmem_yerel_indir_btn_ = new QPushButton("WinPMEM Indir");
            winpmem_yerel_indir_btn_->setEnabled(false);
            satir->addWidget(winpmem_yerel_kontrol_btn_);
            satir->addWidget(winpmem_yerel_indir_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        winpmem_yerel_bilgi_label_ = new QLabel("Kontrol edilmedi");
        winpmem_yerel_bilgi_label_->setWordWrap(true);
        icerik->addWidget(winpmem_yerel_bilgi_label_);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Cikti Dosyasi:"));
            winpmem_yerel_cikti_giris_ = new QLineEdit("memory_dump_local.raw");
            satir->addWidget(winpmem_yerel_cikti_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            winpmem_yerel_baslat_btn_ = new QPushButton("Yerel RAM Edinimini Baslat");
            winpmem_yerel_baslat_btn_->setEnabled(false);
            satir->addWidget(winpmem_yerel_baslat_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        winpmem_yerel_ilerleme_ = new QProgressBar();
        winpmem_yerel_ilerleme_->setRange(0, 100);
        winpmem_yerel_ilerleme_->setValue(0);
        icerik->addWidget(winpmem_yerel_ilerleme_);

        winpmem_yerel_durum_label_ = new QLabel("Hazir");
        icerik->addWidget(winpmem_yerel_durum_label_);

        connect(winpmem_yerel_kontrol_btn_, &QPushButton::clicked, this, [this]() { winpmem_yerel_kontrol_yap(); });
        connect(winpmem_yerel_indir_btn_, &QPushButton::clicked, this, [this]() { winpmem_yerel_indir(); });
        connect(winpmem_yerel_baslat_btn_, &QPushButton::clicked, this, [this]() { winpmem_yerel_baslat(); });
#endif

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_linux_yerel_ram_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Linux Yerel RAM Edinimi (AVML)");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(8);

#ifdef _WIN32
        QLabel* bilgi = new QLabel("Bu ozellik sadece Linux'ta calisir.");
        bilgi->setWordWrap(true);
        icerik->addWidget(bilgi);
#else
        QLabel* bilgi = new QLabel(
            "Bu sekme AVML ile yerel RAM imaji alir.\n"
            "Islem icin root yetkisi gerekir."
        );
        bilgi->setWordWrap(true);
        icerik->addWidget(bilgi);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            avml_kontrol_btn_ = new QPushButton("AVML Kontrol Et");
            avml_bilgi_label_ = new QLabel("Kontrol edilmedi");
            avml_bilgi_label_->setWordWrap(true);
            satir->addWidget(avml_kontrol_btn_);
            satir->addWidget(avml_bilgi_label_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Cikti Dosyasi:"));
            avml_cikti_giris_ = new QLineEdit("linux_memory_dump.raw");
            satir->addWidget(avml_cikti_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            avml_baslat_btn_ = new QPushButton("Linux Yerel RAM Edinimini Baslat");
            avml_baslat_btn_->setEnabled(false);
            satir->addWidget(avml_baslat_btn_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        avml_ilerleme_ = new QProgressBar();
        avml_ilerleme_->setRange(0, 100);
        avml_ilerleme_->setValue(0);
        icerik->addWidget(avml_ilerleme_);

        avml_durum_label_ = new QLabel("Hazir");
        icerik->addWidget(avml_durum_label_);

        connect(avml_kontrol_btn_, &QPushButton::clicked, this, [this]() { avml_kontrol_yap(); });
        connect(avml_baslat_btn_, &QPushButton::clicked, this, [this]() { avml_baslat(); });
#endif

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_sistem_bilgisi_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(12);

        QGroupBox* cerceve = new QGroupBox("Sistem Bilgisi");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);
        icerik->setSpacing(10);

        QLabel* ozet_baslik = new QLabel("Calisma Ortami Ozeti");
        QFont ozet_baslik_font = ozet_baslik->font();
        ozet_baslik_font.setPointSize(15);
        ozet_baslik_font.setBold(true);
        ozet_baslik->setFont(ozet_baslik_font);
        icerik->addWidget(ozet_baslik);

        QGroupBox* ozet_kutu = new QGroupBox("Genel");
        QFormLayout* ozet_form = new QFormLayout(ozet_kutu);
        QLabel* os_deger = new QLabel();
        QLabel* surum_deger = new QLabel();
        QLabel* mimari_deger = new QLabel();
        QLabel* host_deger = new QLabel();
        QLabel* kernel_deger = new QLabel();
        QLabel* kullanici_deger = new QLabel();
        QLabel* zaman_deger = new QLabel();

        ozet_form->addRow("Isletim Sistemi:", os_deger);
        ozet_form->addRow("Surum:", surum_deger);
        ozet_form->addRow("Mimari:", mimari_deger);
        ozet_form->addRow("Hostname:", host_deger);
        ozet_form->addRow("Kernel:", kernel_deger);
        ozet_form->addRow("Kullanici:", kullanici_deger);
        ozet_form->addRow("Olusturma Zamani:", zaman_deger);

        icerik->addWidget(ozet_kutu);

        QTextEdit* metin = new QTextEdit();
        metin->setReadOnly(true);
        metin->setMinimumHeight(250);
        icerik->addWidget(metin);

        auto bilgi_olustur = [=]() -> QString {
            const QString kullanici = qEnvironmentVariable("USERNAME", qEnvironmentVariable("USER", "bilinmiyor"));
            const QString zaman = QDateTime::currentDateTime().toString("yyyy-MM-dd HH:mm:ss");

            os_deger->setText(QSysInfo::productType());
            surum_deger->setText(QSysInfo::productVersion());
            mimari_deger->setText(QSysInfo::currentCpuArchitecture());
            host_deger->setText(QSysInfo::machineHostName());
            kernel_deger->setText(QSysInfo::kernelType() + " " + QSysInfo::kernelVersion());
            kullanici_deger->setText(kullanici);
            zaman_deger->setText(zaman);

            QString bilgi;
            bilgi += "Worm Sistem Bilgisi\n";
            bilgi += "-------------------\n";
            bilgi += "Olusturma Zamani: " + zaman + "\n";
            bilgi += "Isletim Sistemi : " + QSysInfo::productType() + "\n";
            bilgi += "Surum          : " + QSysInfo::productVersion() + "\n";
            bilgi += "Mimari         : " + QSysInfo::currentCpuArchitecture() + "\n";
            bilgi += "Hostname       : " + QSysInfo::machineHostName() + "\n";
            bilgi += "Kernel         : " + QSysInfo::kernelType() + " " + QSysInfo::kernelVersion() + "\n";
            bilgi += "Kullanici      : " + kullanici + "\n";
            return bilgi;
        };

        metin->setPlainText(bilgi_olustur());

        QHBoxLayout* buton_satiri = new QHBoxLayout();
        QPushButton* yenile_btn = new QPushButton("Yenile");
        QPushButton* kaydet_btn = new QPushButton("Bilgiyi Kaydet");
        buton_satiri->addWidget(yenile_btn);
        buton_satiri->addWidget(kaydet_btn);
        buton_satiri->addStretch();
        icerik->addLayout(buton_satiri);

        connect(yenile_btn, &QPushButton::clicked, this, [=]() {
            metin->setPlainText(bilgi_olustur());
            log_ekle("Sistem bilgisi yenilendi", GUNLUK_SEVIYE_INFO);
        });

        connect(kaydet_btn, &QPushButton::clicked, this, [this, metin, bilgi_olustur]() {
            const QString ev = QString::fromUtf8(g_get_home_dir());
            const QString rapor_klasoru = ev + "/Worm/raporlar";
            QDir().mkpath(rapor_klasoru);

            const QString dosya_adi = "sistem_bilgisi_" + QDateTime::currentDateTime().toString("yyyyMMdd_HHmmss") + ".txt";
            const QString tam_yol = rapor_klasoru + "/" + dosya_adi;

            QFile dosya(tam_yol);
            if (!dosya.open(QIODevice::WriteOnly | QIODevice::Text)) {
                QMessageBox::warning(this, "Hata", "Sistem bilgisi kaydedilemedi.");
                log_ekle("Sistem bilgisi kaydetme basarisiz", GUNLUK_SEVIYE_ERROR);
                return;
            }

            const QString icerik = bilgi_olustur();
            dosya.write(icerik.toUtf8());
            dosya.close();

            metin->setPlainText(icerik);
            QMessageBox::information(this, "Bilgi", "Sistem bilgisi kaydedildi:\n" + tam_yol);
            log_ekle("Sistem bilgisi kaydedildi: " + tam_yol, GUNLUK_SEVIYE_INFO);
        });

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_hash_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Hash Hesaplayici");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Dosya:"));
            hash_dosya_giris_ = new QLineEdit();
            satir->addWidget(hash_dosya_giris_, 1);
            icerik->addLayout(satir);
        }

        QPushButton* hesapla_btn = new QPushButton("Hesapla");
        icerik->addWidget(hesapla_btn);

        md5_label_ = new QLabel("MD5: -");
        sha1_label_ = new QLabel("SHA1: -");
        sha256_label_ = new QLabel("SHA256: -");
        sha512_label_ = new QLabel("SHA512: -");
        icerik->addWidget(md5_label_);
        icerik->addWidget(sha1_label_);
        icerik->addWidget(sha256_label_);
        icerik->addWidget(sha512_label_);

        QGroupBox* karsilastir = new QGroupBox("Hash Karsilastir");
        QVBoxLayout* karsilastir_duzen = new QVBoxLayout(karsilastir);
        hash_karsilastir_giris_ = new QLineEdit();
        hash_karsilastir_giris_->setPlaceholderText("Hash degeri girin");
        QPushButton* karsilastir_btn = new QPushButton("Karsilastir");
        hash_sonuc_label_ = new QLabel();
        karsilastir_duzen->addWidget(hash_karsilastir_giris_);
        karsilastir_duzen->addWidget(karsilastir_btn);
        karsilastir_duzen->addWidget(hash_sonuc_label_);
        icerik->addWidget(karsilastir);

        connect(hesapla_btn, &QPushButton::clicked, this, [this]() {
            hash_hesapla();
        });
        connect(karsilastir_btn, &QPushButton::clicked, this, [this]() {
            hash_karsilastir();
        });

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_kanit_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Vaka Yonetimi");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Vaka Adi:"));
            vaka_giris_ = new QLineEdit();
            satir->addWidget(vaka_giris_, 1);
            icerik->addLayout(satir);
        }

        QPushButton* vaka_olustur_btn = new QPushButton("Vaka Olustur");
        icerik->addWidget(vaka_olustur_btn);

        vaka_durum_label_ = new QLabel("Vaka olusturulmadi");
        icerik->addWidget(vaka_durum_label_);

        QGroupBox* dosyalar = new QGroupBox("Dosyalar");
        QVBoxLayout* dosya_duzen = new QVBoxLayout(dosyalar);

        {
            QHBoxLayout* secim_satiri = new QHBoxLayout();
            secim_satiri->addWidget(new QLabel("Klasor:"));
            klasor_secim_ = new QComboBox();
            klasor_secim_->addItem("Ciktilar", "ciktilar");
            klasor_secim_->addItem("Disk Imajlari", "disk_imajlari");
            klasor_secim_->addItem("RAM", "ram");
            klasor_secim_->addItem("Raporlar", "raporlar");
            klasor_secim_->addItem("Hash", "hash");
            klasor_secim_->addItem("Notlar", "notlar");
            klasor_secim_->addItem("Gunlukler", "gunlukler");
            secim_satiri->addWidget(klasor_secim_, 1);
            dosya_duzen->addLayout(secim_satiri);
        }

        dosya_listesi_ = new QComboBox();
        QPushButton* dosya_listele_btn = new QPushButton("Dosyalari Listele");
        dosya_duzen->addWidget(dosya_listesi_);
        dosya_duzen->addWidget(dosya_listele_btn);
        icerik->addWidget(dosyalar);

        connect(vaka_olustur_btn, &QPushButton::clicked, this, [this]() {
            vaka_olustur();
        });
        connect(dosya_listele_btn, &QPushButton::clicked, this, [this]() {
            dosyalari_listele();
        });

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_rapor_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Rapor Olustur");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);

        icerik->addWidget(new QLabel("Rapor olusturmak icin once vaka olusturun ve islem tamamlayin."));

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Rapor Basligi:"));
            rapor_baslik_giris_ = new QLineEdit("Adli Bilisim Teknik Raporu");
            satir->addWidget(rapor_baslik_giris_, 1);
            icerik->addLayout(satir);
        }

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Format:"));
            rapor_format_secim_ = new QComboBox();
            rapor_format_secim_->addItem("TXT", "txt");
            rapor_format_secim_->addItem("JSON", "json");
            satir->addWidget(rapor_format_secim_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        rapor_not_giris_ = new QTextEdit();
        rapor_not_giris_->setPlaceholderText("Not veya rapor aciklamasi girin");
        rapor_not_giris_->setMinimumHeight(160);
        icerik->addWidget(rapor_not_giris_);

        QPushButton* not_ekle_btn = new QPushButton("Not Ekle");
        QPushButton* rapor_olustur_btn = new QPushButton("Rapor Olustur");
        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(not_ekle_btn);
            satir->addWidget(rapor_olustur_btn);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        rapor_durum_label_ = new QLabel("Hazir");
        icerik->addWidget(rapor_durum_label_);

        connect(not_ekle_btn, &QPushButton::clicked, this, [this]() {
            not_ekle();
        });
        connect(rapor_olustur_btn, &QPushButton::clicked, this, [this]() {
            rapor_olustur();
        });

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_gunluk_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QLabel* bilgi = new QLabel("Canli gunluk burada da goruntulenir.");
        duzen->addWidget(bilgi);

        gunluk_sekme_metin_ = new QTextEdit();
        gunluk_sekme_metin_->setReadOnly(true);
        duzen->addWidget(gunluk_sekme_metin_, 1);

        QPushButton* yenile_btn = new QPushButton("Dosyadan Gunlugu Yenile");
        duzen->addWidget(yenile_btn);
        connect(yenile_btn, &QPushButton::clicked, this, [this]() {
            gunluk_dosyadan_yenile();
        });

        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_ayarlar_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(10);

        QGroupBox* cerceve = new QGroupBox("Uygulama Ayarlari");
        QVBoxLayout* icerik = new QVBoxLayout(cerceve);

        ayar_karanlik_tema_secim_ = new QCheckBox("Karanlik Tema");
        ayar_karanlik_tema_secim_->setChecked(ayarlar_ ? ayarlar_->karanlik_tema : false);
        icerik->addWidget(ayar_karanlik_tema_secim_);

        {
            QHBoxLayout* satir = new QHBoxLayout();
            satir->addWidget(new QLabel("Dil:"));
            ayar_dil_secim_ = new QComboBox();
            ayar_dil_secim_->addItem("Turkce", "tr");
            ayar_dil_secim_->addItem("English", "en");
            int idx = ayar_dil_secim_->findData(ayarlar_ ? QString::fromUtf8(ayarlar_->dil) : QString("tr"));
            ayar_dil_secim_->setCurrentIndex(idx >= 0 ? idx : 0);
            satir->addWidget(ayar_dil_secim_);
            satir->addStretch();
            icerik->addLayout(satir);
        }

        QPushButton* kaydet_btn = new QPushButton("Ayarlari Kaydet");
        icerik->addWidget(kaydet_btn);

        ayar_durum_label_ = new QLabel("Hazir");
        icerik->addWidget(ayar_durum_label_);

        connect(ayar_karanlik_tema_secim_, &QCheckBox::toggled, this, [this](bool aktif) {
            tema_uygula(aktif);
        });

        connect(ayar_dil_secim_, &QComboBox::currentIndexChanged, this, [this](int) {
            if (!ayar_dil_secim_) return;
            dil_uygula(ayar_dil_secim_->currentData().toString());
        });

        connect(kaydet_btn, &QPushButton::clicked, this, [this]() {
            if (!ayarlar_ || !ayar_dil_secim_ || !ayar_karanlik_tema_secim_) {
                return;
            }

            ayarlar_->karanlik_tema = ayar_karanlik_tema_secim_->isChecked();
            const QString dil = ayar_dil_secim_->currentData().toString();
            strncpy(ayarlar_->dil, dil.toUtf8().constData(), sizeof(ayarlar_->dil) - 1);
            ayarlar_->dil[sizeof(ayarlar_->dil) - 1] = '\0';

            if (ayar_dosya_yolu_) {
                ayarlar_kaydet(ayarlar_, ayar_dosya_yolu_);
            }

            dil_uygula(dil);
            if (ayar_durum_label_) {
                ayar_durum_label_->setText(QString::fromUtf8(metin_dil("Ayarlar kaydedildi", "Settings saved")));
            }
            log_ekle(QString::fromUtf8(metin_dil("Ayarlar guncellendi", "Settings updated")), GUNLUK_SEVIYE_INFO);
        });

        duzen->addWidget(cerceve);
        duzen->addStretch();
        return sayfa;
    }

    QWidget* olustur_hakkinda_sekmesi() {
        QWidget* sayfa = new QWidget();
        QVBoxLayout* duzen = new QVBoxLayout(sayfa);
        duzen->setContentsMargins(24, 24, 24, 24);
        duzen->setSpacing(12);

        QFrame* ust_kart = new QFrame();
        ust_kart->setObjectName("AboutHeaderCard");
        QVBoxLayout* ust_icerik = new QVBoxLayout(ust_kart);
        ust_icerik->setContentsMargins(18, 16, 18, 16);

        QLabel* baslik = new QLabel("Worm Forensic Tool");
        QFont baslik_font = baslik->font();
        baslik_font.setPointSize(20);
        baslik_font.setBold(true);
        baslik->setFont(baslik_font);
        ust_icerik->addWidget(baslik);

        QLabel* surum = new QLabel("Surum 0.0.1 alpha");
        ust_icerik->addWidget(surum);

        QLabel* ozet = new QLabel(
            "Worm, yetkili adli bilisim sureclerinde disk ve RAM edinimi, dogrulama ve raporlama adimlarini tek bir merkezde birlestiren bir denetim aracidir."
        );
        ozet->setWordWrap(true);
        ust_icerik->addWidget(ozet);

        duzen->addWidget(ust_kart);

        QGroupBox* ozellikler = new QGroupBox("Temel Kabiliyetler");
        QVBoxLayout* ozellik_icerik = new QVBoxLayout(ozellikler);
        QLabel* ozellik_metin = new QLabel(
            "- Windows ve Linux disk imaji edinimi\n"
            "- Windows ve Linux RAM edinimi (WinPMEM / AVML)\n"
            "- Hash hesaplama ve dogrulama\n"
            "- Vaka klasoru, kanit kasasi ve rapor akisi"
        );
        ozellik_metin->setWordWrap(true);
        ozellik_icerik->addWidget(ozellik_metin);
        duzen->addWidget(ozellikler);

        QGroupBox* ilke = new QGroupBox("Kullanim Ilkesi");
        QVBoxLayout* ilke_icerik = new QVBoxLayout(ilke);
        QLabel* ilke_metin = new QLabel(
            "Bu uygulama yalnizca acik yetkili incelemelerde kullanilmalidir. Tum edinim ve kayit adimlari gorunur, denetlenebilir ve loglanabilir olacak sekilde tasarlanmistir."
        );
        ilke_metin->setWordWrap(true);
        ilke_icerik->addWidget(ilke_metin);
        duzen->addWidget(ilke);

        duzen->addStretch();
        return sayfa;
    }

    void log_ekle(const QString& mesaj, GunlukSeviye seviye) {
        const QString zaman = QDateTime::currentDateTime().toString("HH:mm:ss");
        const QString satir = "[" + zaman + "] " + mesaj;
        if (gunluk_metin_) {
            gunluk_metin_->append(satir);
        }
        if (gunluk_sekme_metin_) {
            gunluk_sekme_metin_->append(satir);
        }
        if (gunluk_) {
            gunluk_yaz(gunluk_, seviye, "%s", mesaj.toUtf8().constData());
        }
    }

    void durum_guncelle(const QString& mesaj) {
        if (durum_label_) {
            durum_label_->setText(mesaj);
        }
    }

    void klasor_sec_ve_ata(QLineEdit* hedef, const QString& baslik) {
        if (!hedef) {
            return;
        }

        QString baslangic = hedef->text().trimmed();
        if (baslangic.isEmpty()) {
            baslangic = QString::fromUtf8(g_get_home_dir());
        }

        const QString secilen = QFileDialog::getExistingDirectory(
            this,
            baslik,
            baslangic,
            QFileDialog::ShowDirsOnly | QFileDialog::DontResolveSymlinks
        );

        if (!secilen.isEmpty()) {
            hedef->setText(secilen);
        }
    }

    QString worm_ana_klasor() const {
        return QString::fromUtf8(g_get_home_dir()) + "/Worm";
    }

    QString vaka_cikti_klasoru_hazirla(const QString& alt_klasor) {
        if (!kasa_) {
            return QString();
        }

        QString hedef = QString::fromUtf8(kasa_->ciktilar_klasoru);
        if (!alt_klasor.trimmed().isEmpty()) {
            hedef += "/" + alt_klasor.trimmed();
        }

        if (g_mkdir_with_parents(hedef.toUtf8().constData(), 0755) != 0) {
            return QString();
        }
        return hedef;
    }

    bool vaka_sec_veya_olustur(const QString& islem_adi) {
        const QString ana_klasor = worm_ana_klasor();
        g_mkdir_with_parents(ana_klasor.toUtf8().constData(), 0755);

        QDir dir(ana_klasor);
        QStringList vakalar = dir.entryList(QDir::Dirs | QDir::NoDotAndDotDot, QDir::Name);

        QStringList secenekler;
        secenekler << "Yeni vaka olustur...";
        secenekler.append(vakalar);

        QString varsayilan = "Yeni vaka olustur...";
        if (kasa_ && strlen(kasa_->vaka_adi) > 0) {
            QString aktif = QString::fromUtf8(kasa_->vaka_adi);
            if (secenekler.contains(aktif)) {
                varsayilan = aktif;
            }
        }

        bool ok = false;
        QString secim = QInputDialog::getItem(
            this,
            "Vaka Secimi",
            islem_adi + " icin vaka secin:",
            secenekler,
            secenekler.indexOf(varsayilan),
            false,
            &ok
        );

        if (!ok || secim.isEmpty()) {
            return false;
        }


        QString hedef_vaka = secim;
        if (secim == "Yeni vaka olustur...") {
            QString vars_adi = vaka_giris_ ? vaka_giris_->text().trimmed() : QString();
            hedef_vaka = QInputDialog::getText(
                this,
                "Yeni Vaka",
                "Vaka adi:",
                QLineEdit::Normal,
                vars_adi,
                &ok
            ).trimmed();

            if (!ok || hedef_vaka.isEmpty()) {
                return false;
            }
        }

        if (kasa_ && QString::fromUtf8(kasa_->vaka_adi) == hedef_vaka) {
            if (vaka_giris_) {
                vaka_giris_->setText(hedef_vaka);
            }
            if (vaka_durum_label_) {
                vaka_durum_label_->setText("Vaka secildi: " + hedef_vaka);
            }
            return true;
        }

        if (kasa_) {
            kanit_kasasi_kapat(kasa_);
            kasa_ = nullptr;
        }

        kasa_ = kanit_kasasi_olustur(ana_klasor.toUtf8().constData(), hedef_vaka.toUtf8().constData());
        if (!kasa_) {
            QMessageBox::critical(this, "Hata", "Vaka acilamadi/olusturulamadi!");
            return false;
        }

        if (vaka_giris_) {
            vaka_giris_->setText(hedef_vaka);
        }
        if (vaka_durum_label_) {
            vaka_durum_label_->setText("Vaka secildi: " + hedef_vaka);
        }
        log_ekle("Vaka secildi: " + hedef_vaka, GUNLUK_SEVIYE_INFO);
        return true;
    }

    void ilerleme_guncelle_is_parcacigi(int64_t okunan, int64_t toplam) {
        if (toplam <= 0) {
            return;
        }
        const int oran = static_cast<int>((okunan * 100) / toplam);
        QMetaObject::invokeMethod(this, [this, oran]() {
            if (aktif_uzak_ilerleme_) {
                aktif_uzak_ilerleme_->setValue(oran);
            } else if (uzak_ilerleme_) {
                uzak_ilerleme_->setValue(oran);
            }
            if (genel_ilerleme_) {
                genel_ilerleme_->setValue(oran);
            }
        }, Qt::QueuedConnection);
    }

    void guvenlik_anahtari_onayla() {
        if (!token_giris_) {
            return;
        }

        const QString anahtar = token_giris_->text().trimmed();
        if (anahtar.isEmpty()) {
            QMessageBox::warning(this, "Uyari", "Onaylamak icin guvenlik anahtari girin!");
            return;
        }

        onayli_guvenlik_anahtari_ = anahtar;
        token_giris_->setReadOnly(true);
        if (guvenlik_durum_label_) {
            guvenlik_durum_label_->setText("Guvenlik anahtari: Aktif");
        }
        log_ekle("Guvenlik anahtari onaylandi", GUNLUK_SEVIYE_INFO);
    }

    void guvenlik_anahtari_sifirla() {
        onayli_guvenlik_anahtari_.clear();
        if (token_giris_) {
            token_giris_->clear();
            token_giris_->setReadOnly(false);
        }
        if (guvenlik_durum_label_) {
            guvenlik_durum_label_->setText("Guvenlik anahtari: Kapali");
        }
        log_ekle("Guvenlik anahtari sifirlandi", GUNLUK_SEVIYE_INFO);
    }

    void uzak_baglan() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma surerken yeniden baglanamazsiniz.");
            return;
        }

        if (baglaniyor_.load()) {
            QMessageBox::warning(this, "Bilgi", "Baglanti islemi devam ediyor...");
            return;
        }

        const QString ip = ip_giris_->text().trimmed();
        const QString port_metin = port_giris_->text().trimmed();
        const QString token = onayli_guvenlik_anahtari_;
        const QString token_guncel = token_giris_ ? token_giris_->text().trimmed() : QString();

        if (ip.isEmpty() || port_metin.isEmpty()) {
            QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("IP ve port girin!"));
            return;
        }

        bool port_ok = false;
        const int port = port_metin.toInt(&port_ok);
        if (!port_ok || port <= 0 || port > 65535) {
            QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("Gecersiz port!"));
            return;
        }

        if (!token_guncel.isEmpty() && token.isEmpty()) {
            QMessageBox::warning(this, "Uyari", "Guvenlik anahtarini kullanmak icin once 'Anahtari Onayla' butonuna basin.");
            return;
        }

        if (!token.isEmpty() && token_guncel != token) {
            QMessageBox::warning(this, "Uyari", "Guvenlik anahtari degisti. Lutfen yeniden 'Anahtari Onayla' yapin veya 'Anahtari Sifirla' kullanin.");
            return;
        }

        // Butonlari devre disi birak
        baglan_btn_->setEnabled(false);
        baglaniyor_ = true;
        uzak_durum_label_->setText("Baglaniyor... (Zaman asimi: 10sn)");
        log_ekle("Baglanti baslatiliyor: " + ip + ":" + port_metin, GUNLUK_SEVIYE_INFO);

        // Eski baglantiyi temizle
        if (baglanti_) {
            uzak_disk_baglanti_kapat(baglanti_);
            baglanti_ = nullptr;
        }

        // VPN baslat (gerekirse)
        if (vpn_kullan_secim_ && vpn_kullan_secim_->isChecked()) {
            if (!vpn_yonetici_) {
                vpn_yonetici_ = wireguard_yonetici_olustur();
            }

            if (!vpn_yonetici_) {
                QMessageBox::critical(this, "Hata", "VPN yoneticisi olusturulamadi!");
                baglan_btn_->setEnabled(true);
                baglaniyor_ = false;
                return;
            }

            if (wireguard_durum(vpn_yonetici_) == 0) {
                if (wireguard_baslat(vpn_yonetici_, vpn_config_yolu_.toUtf8().constData()) != 0) {
                    QMessageBox::critical(this, "Hata", "VPN baslatilamadi! Once VPN Yapilandir ile ayarlari kontrol edin.");
                    baglan_btn_->setEnabled(true);
                    baglaniyor_ = false;
                    return;
                }
                log_ekle("WireGuard VPN baslatildi", GUNLUK_SEVIYE_INFO);
            }
        }

        // Baglanti bilgilerini kopyala
        QString ip_copy = ip;
        QString token_copy = token;

        // Asenkron baglanti icin thread baslat
        std::thread([this, ip_copy, port, token_copy]() {
            UzakDiskBaglanti* yeni_baglanti = uzak_disk_baglanti_olustur(
                ip_copy.toUtf8().constData(), 
                port, 
                token_copy.isEmpty() ? nullptr : token_copy.toUtf8().constData()
            );

            bool basarili = false;
            if (yeni_baglanti) {
                // Zaman asimi ile baglan (10 saniye)
                // Note: g_socket_client_connect_to_host biraz uzun surer
                // Bu yuzden simdilik dogrudan cagiriyoruz, 
                // ileride daha gelismis zaman asimi eklenebilir
                basarili = uzak_disk_baglan(yeni_baglanti);
            }

            QMetaObject::invokeMethod(this, [this, yeni_baglanti, basarili, ip_copy]() {
                baglaniyor_ = false;
                
                if (basarili) {
                    baglanti_ = yeni_baglanti;
                    uzak_durum_label_->setText("Baglandi - " + ip_copy);
                    log_ekle("Uzak sunucuya baglandi: " + ip_copy, GUNLUK_SEVIYE_INFO);
                    
                    // Baglanti sonrasi disk listesini otomatik almayi teklif et
                    QMessageBox::StandardButton reply = QMessageBox::question(
                        this, 
                        "Baglanti Basarili", 
                        "Sunucuya baglanildi. Disk listesi alinsin mi?",
                        QMessageBox::Yes | QMessageBox::No
                    );
                    if (reply == QMessageBox::Yes) {
                        uzak_disk_getir();
                    }
                } else {
                    QString detay;
                    if (yeni_baglanti && yeni_baglanti->son_hata[0] != '\0') {
                        detay = QString::fromUtf8(yeni_baglanti->son_hata);
                    }
                    if (yeni_baglanti) {
                        uzak_disk_baglanti_kapat(yeni_baglanti);
                    }
                    uzak_durum_label_->setText("Baglanti basarisiz!");
                    QString mesaj =
                        "Sunucuya baglanilamadi!\n\n"
                        "Kontrol edin:\n"
                        "- IP adresi ve port dogru mu?\n"
                        "- Windows Agent calisiyor mu?\n"
                        "- Guvenlik duvari engellemiyor mu?\n"
                        "- Ag baglantisi var mi?";
                    if (!detay.isEmpty()) {
                        mesaj += "\n\nAjan detayi: " + detay;
                    }
                    QMessageBox::critical(this, "Hata", mesaj);
                    log_ekle("Baglanti basarisiz: " + ip_copy, GUNLUK_SEVIYE_ERROR);
                }
                
                baglan_btn_->setEnabled(true);
            }, Qt::QueuedConnection);
        }).detach();
    }

    void vpn_yapilandir() {
        QDialog pencere(this);
        pencere.setWindowTitle("VPN Yapilandir");
        QVBoxLayout* ana = new QVBoxLayout(&pencere);
        QFormLayout* form = new QFormLayout();

        QLineEdit* config_yol = new QLineEdit(vpn_config_yolu_);
        QLineEdit* private_key = new QLineEdit();
        private_key->setPlaceholderText("WireGuard ozel anahtari");
        QLineEdit* public_key = new QLineEdit();
        public_key->setPlaceholderText("Sunucu acik anahtari");
        QLineEdit* endpoint = new QLineEdit("1.2.3.4:51820");
        QLineEdit* address = new QLineEdit("10.0.0.2/24");
        QLineEdit* dns = new QLineEdit("1.1.1.1");
        QLineEdit* allowed_ips = new QLineEdit("0.0.0.0/0, ::/0");
        QSpinBox* keepalive = new QSpinBox();
        keepalive->setRange(0, 120);
        keepalive->setValue(25);

        form->addRow("Config Dosyasi:", config_yol);
        form->addRow("Private Key:", private_key);
        form->addRow("Public Key:", public_key);
        form->addRow("Endpoint:", endpoint);
        form->addRow("Address:", address);
        form->addRow("DNS:", dns);
        form->addRow("Allowed IPs:", allowed_ips);
        form->addRow("Keepalive:", keepalive);
        ana->addLayout(form);

        QDialogButtonBox* butonlar = new QDialogButtonBox(QDialogButtonBox::Ok | QDialogButtonBox::Cancel);
        ana->addWidget(butonlar);

        connect(butonlar, &QDialogButtonBox::accepted, &pencere, &QDialog::accept);
        connect(butonlar, &QDialogButtonBox::rejected, &pencere, &QDialog::reject);

        if (pencere.exec() != QDialog::Accepted) {
            return;
        }

        const QString yol = config_yol->text().trimmed();
        if (yol.isEmpty()) {
            QMessageBox::critical(this, "Hata", "Config dosya yolu bos olamaz!");
            return;
        }

        if (private_key->text().trimmed().isEmpty() ||
            public_key->text().trimmed().isEmpty() ||
            endpoint->text().trimmed().isEmpty() ||
            address->text().trimmed().isEmpty()) {
            QMessageBox::critical(this, "Hata", "Private Key, Public Key, Endpoint ve Address alanlari zorunludur!");
            return;
        }

        const QString klasor = QFileInfo(yol).absolutePath();
        g_mkdir_with_parents(klasor.toUtf8().constData(), 0755);

        int sonuc = wireguard_config_olustur(
            yol.toUtf8().constData(),
            private_key->text().trimmed().toUtf8().constData(),
            public_key->text().trimmed().toUtf8().constData(),
            endpoint->text().trimmed().toUtf8().constData(),
            allowed_ips->text().trimmed().toUtf8().constData(),
            address->text().trimmed().toUtf8().constData(),
            dns->text().trimmed().toUtf8().constData(),
            keepalive->value()
        );

        if (sonuc != 0) {
            QMessageBox::critical(this, "Hata", "VPN config olusturulamadi!");
            return;
        }

        vpn_config_yolu_ = yol;
        log_ekle("VPN config olusturuldu", GUNLUK_SEVIYE_INFO);
        QMessageBox::information(this, "Bilgi", "VPN ayarlari kaydedildi.");
    }

    void uzak_disk_getir() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma surerken disk listesi yenilenemez.");
            return;
        }

        if (!baglanti_) {
            QMessageBox::critical(this, "Hata", "Once baglanin!");
            return;
        }

        if (baglaniyor_.load()) {
            QMessageBox::warning(this, "Bilgi", "Baglanti islemi devam ediyor...");
            return;
        }

        // Butonlari devre disi birak ve durum guncelle
        disk_getir_btn_->setEnabled(false);
        uzak_durum_label_->setText("Diskler getiriliyor...");
        log_ekle("Disk listesi aliniyor...", GUNLUK_SEVIYE_INFO);

        // Asenkron disk listesi alma
        std::thread([this]() {
            GList* diskler = uzak_disk_listele(baglanti_);
            bool yanit_ok = baglanti_ ? baglanti_->son_yanit_ok : false;
            
            QMetaObject::invokeMethod(this, [this, diskler, yanit_ok]() {
                disk_getir_btn_->setEnabled(true);
                
                if (!diskler) {
                    if (yanit_ok) {
                        uzak_durum_label_->setText("Disk bulunamadi veya yetki yok");
                        log_ekle("Disk listesi alindi ancak disk yok", GUNLUK_SEVIYE_WARN);
                        QMessageBox::information(this, "Bilgi",
                            "Baglanti kuruldu ancak listelenecek disk bulunamadi.\n\n"
                            "Olasi nedenler:\n"
                            "- Windows ajan Yonetici olarak calismiyor\n"
                            "- Guvenlik urunleri ham disk erisimini engelliyor\n"
                            "- Test ortami diskleri erisime kapali");
                    } else {
                        QString detay;
                        if (baglanti_ && baglanti_->son_hata[0] != '\0') {
                            detay = QString::fromUtf8(baglanti_->son_hata);
                        }

                        uzak_durum_label_->setText("Diskler alinamadi - baglanti kopmus olabilir");
                        log_ekle("Disk listesi alinamadi", GUNLUK_SEVIYE_ERROR);
                        QString mesaj = "Disk listesi alinamadi!\n"
                                        "Baglanti kopmus olabilir veya zaman asimina ugradi.";
                        if (!detay.isEmpty()) {
                            mesaj += "\n\nAjan detayi: " + detay;
                        }
                        QMessageBox::critical(this, "Hata", mesaj);
                    }
                    return;
                }

                disk_secim_->clear();
                uzak_diskler_.clear();

                for (GList* l = diskler; l != nullptr; l = l->next) {
                    UzakDisk* d = static_cast<UzakDisk*>(l->data);
                    if (!d) continue;
                    
                    uzak_diskler_.push_back(*d);
                    const double gb = static_cast<double>(d->boyut) / (1024.0 * 1024.0 * 1024.0);
                    const QString etiket = QString("%1 (%2, %3 GB)")
                        .arg(QString::fromUtf8(d->id))
                        .arg(QString::fromUtf8(d->ad))
                        .arg(QString::number(gb, 'f', 1));
                    disk_secim_->addItem(etiket);
                }

                g_list_free_full(diskler, g_free);

                uzak_durum_label_->setText(QString::number(uzak_diskler_.size()) + " disk bulundu");
                log_ekle("Uzak diskler listelendi: " + QString::number(uzak_diskler_.size()) + " disk", GUNLUK_SEVIYE_INFO);
            }, Qt::QueuedConnection);
        }).detach();
    }

    void uzak_imaj_baslat() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma islemi zaten calisiyor.");
            return;
        }

        if (!baglanti_) {
            QMessageBox::critical(this, "Hata", "Once baglanin!");
            return;
        }

        if (!vaka_sec_veya_olustur("Uzak disk imaji")) {
            return;
        }

        const int secili = disk_secim_->currentIndex();
        if (secili < 0 || secili >= static_cast<int>(uzak_diskler_.size())) {
            QMessageBox::critical(this, "Hata", "Disk secilmedi!");
            return;
        }

        const QString cikti_klasoru = vaka_cikti_klasoru_hazirla("disk_imajlari");
        if (cikti_klasoru.isEmpty()) {
            QMessageBox::critical(this, "Hata", "Vaka cikti klasoru olusturulamadi!");
            return;
        }
        if (cikti_klasor_giris_) {
            cikti_klasor_giris_->setText(cikti_klasoru);
        }

        UzakDisk& disk = uzak_diskler_[static_cast<size_t>(secili)];
        if (!::uzak_imaj_baslat(baglanti_, disk.id, cikti_klasoru.toUtf8().constData(), kasa_->vaka_adi)) {
            uzak_durum_label_->setText("Imaj baslatilamadi");
            QMessageBox::critical(this, "Hata", "Imaj alma baslatilamadi!");
            return;
        }

        log_ekle("Uzak imaj baslatildi", GUNLUK_SEVIYE_INFO);
        uzak_durum_label_->setText("Imaj aliniyor...");
        durum_guncelle("Uzak imaj alma calisiyor");
        uzak_ilerleme_->setValue(0);
        genel_ilerleme_->setValue(0);

        imaj_btn_->setEnabled(false);
        if (baglan_btn_) {
            baglan_btn_->setEnabled(false);
        }
        if (disk_getir_btn_) {
            disk_getir_btn_->setEnabled(false);
        }
        imaj_calisiyor_ = true;
        aktif_uzak_ilerleme_ = uzak_ilerleme_;

        std::thread([this]() {
            IsGorevi* is = is_olustur(IS_TIPI_DISK_EDINIM, "Uzak Disk Imaji");
            bool basarili = uzak_imaj_stream_al(baglanti_, is, &AnaPencere::ilerleme_kopru, this);

            QMetaObject::invokeMethod(this, [this, basarili]() {
                imaj_calisiyor_ = false;
                imaj_btn_->setEnabled(true);
                if (baglan_btn_) {
                    baglan_btn_->setEnabled(true);
                }
                if (disk_getir_btn_) {
                    disk_getir_btn_->setEnabled(true);
                }
                if (basarili) {
                    uzak_durum_label_->setText("Imaj alma tamamlandi");
                    durum_guncelle("Hazir");
                    log_ekle("Uzak imaj alma tamamlandi", GUNLUK_SEVIYE_INFO);
                } else {
                    uzak_durum_label_->setText("Imaj alma basarisiz");
                    durum_guncelle("Hata");
                    log_ekle("Uzak imaj alma basarisiz", GUNLUK_SEVIYE_ERROR);
                }
                aktif_uzak_ilerleme_ = nullptr;
            }, Qt::QueuedConnection);

            is_temizle(is);
        }).detach();
    }

    void linux_uzak_baglan() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma surerken yeniden baglanamazsiniz.");
            return;
        }
        if (baglaniyor_.load()) {
            QMessageBox::warning(this, "Bilgi", "Baglanti islemi devam ediyor...");
            return;
        }
        if (!linux_uzak_ip_giris_ || !linux_uzak_port_giris_) {
            return;
        }

        const QString ip = linux_uzak_ip_giris_->text().trimmed();
        const QString port_metin = linux_uzak_port_giris_->text().trimmed();
        const QString token = linux_uzak_token_giris_ ? linux_uzak_token_giris_->text().trimmed() : QString();

        if (ip.isEmpty() || port_metin.isEmpty()) {
            QMessageBox::critical(this, "Hata", "IP ve port girin!");
            return;
        }

        bool port_ok = false;
        const int port = port_metin.toInt(&port_ok);
        if (!port_ok || port <= 0 || port > 65535) {
            QMessageBox::critical(this, "Hata", "Gecersiz port!");
            return;
        }

        linux_uzak_baglan_btn_->setEnabled(false);
        baglaniyor_ = true;
        if (linux_uzak_durum_label_) {
            linux_uzak_durum_label_->setText("Baglaniyor... (Zaman asimi: 10sn)");
        }
        log_ekle("Linux uzak baglanti baslatiliyor: " + ip + ":" + port_metin, GUNLUK_SEVIYE_INFO);

        if (baglanti_) {
            uzak_disk_baglanti_kapat(baglanti_);
            baglanti_ = nullptr;
        }

        if (linux_uzak_vpn_kullan_secim_ && linux_uzak_vpn_kullan_secim_->isChecked()) {
            if (!vpn_yonetici_) {
                vpn_yonetici_ = wireguard_yonetici_olustur();
            }
            if (!vpn_yonetici_) {
                QMessageBox::critical(this, "Hata", "VPN yoneticisi olusturulamadi!");
                linux_uzak_baglan_btn_->setEnabled(true);
                baglaniyor_ = false;
                return;
            }
            if (wireguard_durum(vpn_yonetici_) == 0) {
                if (wireguard_baslat(vpn_yonetici_, vpn_config_yolu_.toUtf8().constData()) != 0) {
                    QMessageBox::critical(this, "Hata", "VPN baslatilamadi! Once VPN Yapilandir ile ayarlari kontrol edin.");
                    linux_uzak_baglan_btn_->setEnabled(true);
                    baglaniyor_ = false;
                    return;
                }
                log_ekle("WireGuard VPN baslatildi", GUNLUK_SEVIYE_INFO);
            }
        }

        std::thread([this, ip, port, token]() {
            UzakDiskBaglanti* yeni_baglanti = uzak_disk_baglanti_olustur(
                ip.toUtf8().constData(),
                port,
                token.isEmpty() ? nullptr : token.toUtf8().constData()
            );
            bool basarili = false;
            if (yeni_baglanti) {
                basarili = uzak_disk_baglan(yeni_baglanti);
            }

            QMetaObject::invokeMethod(this, [this, yeni_baglanti, basarili, ip]() {
                baglaniyor_ = false;
                linux_uzak_baglan_btn_->setEnabled(true);

                if (basarili) {
                    baglanti_ = yeni_baglanti;
                    if (linux_uzak_durum_label_) {
                        linux_uzak_durum_label_->setText("Baglandi - " + ip);
                    }
                    log_ekle("Linux uzak sunucuya baglandi: " + ip, GUNLUK_SEVIYE_INFO);
                } else {
                    QString detay;
                    if (yeni_baglanti && yeni_baglanti->son_hata[0] != '\0') {
                        detay = QString::fromUtf8(yeni_baglanti->son_hata);
                    }
                    if (yeni_baglanti) {
                        uzak_disk_baglanti_kapat(yeni_baglanti);
                    }
                    if (linux_uzak_durum_label_) {
                        linux_uzak_durum_label_->setText("Baglanti basarisiz!");
                    }
                    QString mesaj =
                        "Sunucuya baglanilamadi!\n\n"
                        "Kontrol edin:\n"
                        "- IP adresi ve port dogru mu?\n"
                        "- Linux ajan calisiyor mu?\n"
                        "- Guvenlik duvari engellemiyor mu?\n"
                        "- Ag baglantisi var mi?";
                    if (!detay.isEmpty()) {
                        mesaj += "\n\nAjan detayi: " + detay;
                    }
                    QMessageBox::critical(this, "Hata", mesaj);
                    log_ekle("Linux uzak baglanti basarisiz: " + ip, GUNLUK_SEVIYE_ERROR);
                }
            }, Qt::QueuedConnection);
        }).detach();
    }

    void linux_uzak_disk_getir() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma surerken disk listesi yenilenemez.");
            return;
        }
        if (!baglanti_) {
            QMessageBox::critical(this, "Hata", "Once baglanin!");
            return;
        }
        if (baglaniyor_.load()) {
            QMessageBox::warning(this, "Bilgi", "Baglanti islemi devam ediyor...");
            return;
        }
        if (!linux_uzak_disk_getir_btn_ || !linux_uzak_disk_secim_) {
            return;
        }

        linux_uzak_disk_getir_btn_->setEnabled(false);
        if (linux_uzak_durum_label_) {
            linux_uzak_durum_label_->setText("Diskler getiriliyor...");
        }
        log_ekle("Linux uzak disk listesi aliniyor...", GUNLUK_SEVIYE_INFO);

        std::thread([this]() {
            GList* diskler = uzak_disk_listele(baglanti_);
            bool yanit_ok = baglanti_ ? baglanti_->son_yanit_ok : false;

            QMetaObject::invokeMethod(this, [this, diskler, yanit_ok]() {
                linux_uzak_disk_getir_btn_->setEnabled(true);

                if (!diskler) {
                    if (yanit_ok) {
                        if (linux_uzak_durum_label_) {
                            linux_uzak_durum_label_->setText("Disk bulunamadi veya yetki yok");
                        }
                        log_ekle("Linux uzak disk listesi alindi ancak disk yok", GUNLUK_SEVIYE_WARN);
                    } else {
                        QString detay;
                        if (baglanti_ && baglanti_->son_hata[0] != '\0') {
                            detay = QString::fromUtf8(baglanti_->son_hata);
                        }
                        if (linux_uzak_durum_label_) {
                            linux_uzak_durum_label_->setText("Diskler alinamadi - baglanti kopmus olabilir");
                        }
                        log_ekle("Linux uzak disk listesi alinamadi", GUNLUK_SEVIYE_ERROR);
                        QString mesaj = "Disk listesi alinamadi!\n"
                                        "Baglanti kopmus olabilir veya zaman asimina ugradi.";
                        if (!detay.isEmpty()) {
                            mesaj += "\n\nAjan detayi: " + detay;
                        }
                        QMessageBox::critical(this, "Hata", mesaj);
                    }
                    return;
                }

                linux_uzak_disk_secim_->clear();
                uzak_diskler_.clear();

                for (GList* l = diskler; l != nullptr; l = l->next) {
                    UzakDisk* d = static_cast<UzakDisk*>(l->data);
                    if (!d) continue;

                    uzak_diskler_.push_back(*d);
                    const double gb = static_cast<double>(d->boyut) / (1024.0 * 1024.0 * 1024.0);
                    const QString etiket = QString("%1 (%2, %3 GB)")
                        .arg(QString::fromUtf8(d->id))
                        .arg(QString::fromUtf8(d->ad))
                        .arg(QString::number(gb, 'f', 1));
                    linux_uzak_disk_secim_->addItem(etiket);
                }

                g_list_free_full(diskler, g_free);
                if (linux_uzak_durum_label_) {
                    linux_uzak_durum_label_->setText(QString::number(uzak_diskler_.size()) + " disk bulundu");
                }
                log_ekle("Linux uzak diskler listelendi: " + QString::number(uzak_diskler_.size()) + " disk", GUNLUK_SEVIYE_INFO);
            }, Qt::QueuedConnection);
        }).detach();
    }

    void linux_uzak_imaj_baslat() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma islemi zaten calisiyor.");
            return;
        }
        if (!baglanti_) {
            QMessageBox::critical(this, "Hata", "Once baglanin!");
            return;
        }
        if (!linux_uzak_disk_secim_ || !linux_uzak_imaj_btn_) {
            return;
        }

        if (!vaka_sec_veya_olustur("Linux uzak disk imaji")) {
            return;
        }

        const int secili = linux_uzak_disk_secim_->currentIndex();
        if (secili < 0 || secili >= static_cast<int>(uzak_diskler_.size())) {
            QMessageBox::critical(this, "Hata", "Disk secilmedi!");
            return;
        }

        const QString cikti_klasoru = vaka_cikti_klasoru_hazirla("disk_imajlari");
        if (cikti_klasoru.isEmpty()) {
            QMessageBox::critical(this, "Hata", "Vaka cikti klasoru olusturulamadi!");
            return;
        }
        if (linux_uzak_cikti_klasor_giris_) {
            linux_uzak_cikti_klasor_giris_->setText(cikti_klasoru);
        }

        UzakDisk& disk = uzak_diskler_[static_cast<size_t>(secili)];
        if (!::uzak_imaj_baslat(baglanti_, disk.id, cikti_klasoru.toUtf8().constData(), kasa_->vaka_adi)) {
            if (linux_uzak_durum_label_) {
                linux_uzak_durum_label_->setText("Imaj baslatilamadi");
            }
            QMessageBox::critical(this, "Hata", "Imaj alma baslatilamadi!");
            return;
        }

        log_ekle("Linux uzak imaj baslatildi", GUNLUK_SEVIYE_INFO);
        if (linux_uzak_durum_label_) {
            linux_uzak_durum_label_->setText("Imaj aliniyor...");
        }
        durum_guncelle("Linux uzak imaj alma calisiyor");
        if (linux_uzak_ilerleme_) {
            linux_uzak_ilerleme_->setValue(0);
        }
        if (genel_ilerleme_) {
            genel_ilerleme_->setValue(0);
        }

        linux_uzak_imaj_btn_->setEnabled(false);
        if (linux_uzak_baglan_btn_) {
            linux_uzak_baglan_btn_->setEnabled(false);
        }
        if (linux_uzak_disk_getir_btn_) {
            linux_uzak_disk_getir_btn_->setEnabled(false);
        }
        imaj_calisiyor_ = true;
        aktif_uzak_ilerleme_ = linux_uzak_ilerleme_;

        std::thread([this]() {
            IsGorevi* is = is_olustur(IS_TIPI_DISK_EDINIM, "Linux Uzak Disk Imaji");
            bool basarili = uzak_imaj_stream_al(baglanti_, is, &AnaPencere::ilerleme_kopru, this);

            QMetaObject::invokeMethod(this, [this, basarili]() {
                imaj_calisiyor_ = false;
                linux_uzak_imaj_btn_->setEnabled(true);
                if (linux_uzak_baglan_btn_) {
                    linux_uzak_baglan_btn_->setEnabled(true);
                }
                if (linux_uzak_disk_getir_btn_) {
                    linux_uzak_disk_getir_btn_->setEnabled(true);
                }

                if (basarili) {
                    if (linux_uzak_durum_label_) {
                        linux_uzak_durum_label_->setText("Imaj alma tamamlandi");
                    }
                    durum_guncelle("Hazir");
                    log_ekle("Linux uzak imaj alma tamamlandi", GUNLUK_SEVIYE_INFO);
                } else {
                    if (linux_uzak_durum_label_) {
                        linux_uzak_durum_label_->setText("Imaj alma basarisiz");
                    }
                    durum_guncelle("Hata");
                    log_ekle("Linux uzak imaj alma basarisiz", GUNLUK_SEVIYE_ERROR);
                }
                aktif_uzak_ilerleme_ = nullptr;
            }, Qt::QueuedConnection);

            is_temizle(is);
        }).detach();
    }

    void yerel_disk_getir() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma islemi surerken yerel diskler yenilenemez.");
            return;
        }

        if (!yerel_disk_getir_btn_ || !yerel_disk_secim_) {
            return;
        }

        yerel_disk_getir_btn_->setEnabled(false);
        if (yerel_durum_label_) {
            yerel_durum_label_->setText("Yerel diskler aranıyor...");
        }

        std::thread([this]() {
            DiskBilgisi* diskler = nullptr;
            int disk_sayisi = 0;
            bool ok = disk_listele(&diskler, &disk_sayisi);

            QMetaObject::invokeMethod(this, [this, ok, diskler, disk_sayisi]() {
                yerel_disk_getir_btn_->setEnabled(true);
                yerel_disk_secim_->clear();
                yerel_diskler_.clear();

                if (!ok || !diskler || disk_sayisi <= 0) {
                    if (yerel_durum_label_) {
                        yerel_durum_label_->setText("Yerel disk bulunamadi veya erisim yok");
                    }
                    log_ekle("Yerel disk listesi alinamadi", GUNLUK_SEVIYE_ERROR);
                    if (diskler) {
                        free(diskler);
                    }
                    return;
                }

                yerel_diskler_.reserve((size_t)disk_sayisi);
                for (int i = 0; i < disk_sayisi; i++) {
                    yerel_diskler_.push_back(diskler[i]);

                    const double gb = (double)diskler[i].toplam_boyut / (1024.0 * 1024.0 * 1024.0);
                    QString etiket = QString("%1 (%2 GB)%3")
                        .arg(QString::fromUtf8(diskler[i].cihaz))
                        .arg(QString::number(gb, 'f', 1))
                        .arg(diskler[i].erisilebilir ? "" : " [erisim yok]");

                    yerel_disk_secim_->addItem(etiket, QString::fromUtf8(diskler[i].cihaz));
                }

                free(diskler);

                if (yerel_durum_label_) {
                    yerel_durum_label_->setText(QString::number(disk_sayisi) + " yerel disk bulundu");
                }
                log_ekle("Yerel diskler listelendi: " + QString::number(disk_sayisi), GUNLUK_SEVIYE_INFO);
            }, Qt::QueuedConnection);
        }).detach();
    }

    void yerel_imaj_baslat() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma islemi zaten calisiyor.");
            return;
        }

        if (!vaka_sec_veya_olustur("Yerel disk imaji")) {
            return;
        }

        if (!yerel_disk_secim_ || yerel_disk_secim_->currentIndex() < 0) {
            QMessageBox::critical(this, "Hata", "Yerel disk secilmedi!");
            return;
        }

        const QString kaynak = yerel_disk_secim_->currentData().toString().trimmed();
        if (kaynak.isEmpty()) {
            QMessageBox::critical(this, "Hata", "Gecerli kaynak secilemedi!");
            return;
        }

        const QString cikti_klasoru = vaka_cikti_klasoru_hazirla("disk_imajlari");
        if (cikti_klasoru.isEmpty()) {
            QMessageBox::critical(this, "Hata", "Vaka cikti klasoru olusturulamadi!");
            return;
        }
        if (yerel_cikti_klasor_giris_) {
            yerel_cikti_klasor_giris_->setText(cikti_klasoru);
        }

        QString kaynak_etiket = kaynak;
        kaynak_etiket.replace("/", "_");
        kaynak_etiket.replace("\\", "_");
        kaynak_etiket.replace(":", "_");
        kaynak_etiket.replace(".", "_");

        const QString zaman = QDateTime::currentDateTime().toString("yyyyMMdd_HHmmss");
        const QString hedef = QString("%1/yerel_%2_%3.img")
            .arg(cikti_klasoru)
            .arg(kaynak_etiket)
            .arg(zaman);

        if (yerel_ilerleme_) {
            yerel_ilerleme_->setValue(0);
        }
        if (genel_ilerleme_) {
            genel_ilerleme_->setValue(0);
        }

        if (yerel_durum_label_) {
            yerel_durum_label_->setText("Yerel imaj aliniyor...");
        }
        durum_guncelle("Yerel imaj alma calisiyor");
        log_ekle("Yerel imaj baslatildi: " + kaynak, GUNLUK_SEVIYE_INFO);

        imaj_calisiyor_ = true;
        if (yerel_imaj_btn_) {
            yerel_imaj_btn_->setEnabled(false);
        }
        if (yerel_disk_getir_btn_) {
            yerel_disk_getir_btn_->setEnabled(false);
        }
        if (imaj_btn_) {
            imaj_btn_->setEnabled(false);
        }

        std::thread([this, kaynak, hedef]() {
            IsGorevi* is = is_olustur(IS_TIPI_DISK_EDINIM, "Yerel Disk Imaji");

            std::atomic<bool> ilerleme_izleme_aktif = true;
            std::thread ilerleme_izleyici([this, is, &ilerleme_izleme_aktif]() {
                while (ilerleme_izleme_aktif.load()) {
                    int oran = 0;
                    if (is) {
                        oran = is->ilerleme_yuzde;
                    }
                    if (oran < 0) {
                        oran = 0;
                    }
                    if (oran > 100) {
                        oran = 100;
                    }

                    QMetaObject::invokeMethod(this, [this, oran]() {
                        if (yerel_ilerleme_) {
                            yerel_ilerleme_->setValue(oran);
                        }
                        if (genel_ilerleme_) {
                            genel_ilerleme_->setValue(oran);
                        }
                    }, Qt::QueuedConnection);

                    std::this_thread::sleep_for(std::chrono::milliseconds(300));
                }
            });

            DiskEdinimGorevi gorev;
            memset(&gorev, 0, sizeof(gorev));
            gorev.is = is;
            strncpy(gorev.kaynak, kaynak.toUtf8().constData(), sizeof(gorev.kaynak) - 1);
            strncpy(gorev.hedef, hedef.toUtf8().constData(), sizeof(gorev.hedef) - 1);
            gorev.parca_boyutu = 4 * 1024 * 1024;
            gorev.hash_hesapla = true;
            gorev.tam_disk = true;

            bool basarili = disk_edinim_gorevi_calistir(&gorev);

            ilerleme_izleme_aktif = false;
            if (ilerleme_izleyici.joinable()) {
                ilerleme_izleyici.join();
            }

            const QString hata = (is && is->hata_mesaji) ? QString::fromUtf8(is->hata_mesaji) : QString();

            QMetaObject::invokeMethod(this, [this, basarili, hedef, hata]() {
                imaj_calisiyor_ = false;

                if (yerel_imaj_btn_) {
                    yerel_imaj_btn_->setEnabled(true);
                }
                if (yerel_disk_getir_btn_) {
                    yerel_disk_getir_btn_->setEnabled(true);
                }
                if (imaj_btn_) {
                    imaj_btn_->setEnabled(true);
                }

                if (basarili) {
                    if (yerel_ilerleme_) {
                        yerel_ilerleme_->setValue(100);
                    }
                    if (genel_ilerleme_) {
                        genel_ilerleme_->setValue(100);
                    }
                    if (yerel_durum_label_) {
                        yerel_durum_label_->setText("Yerel imaj alma tamamlandi");
                    }
                    durum_guncelle("Hazir");
                    log_ekle("Yerel imaj tamamlandi: " + hedef, GUNLUK_SEVIYE_INFO);
                } else {
                    if (yerel_durum_label_) {
                        yerel_durum_label_->setText("Yerel imaj alma basarisiz");
                    }
                    durum_guncelle("Hata");
                    log_ekle("Yerel imaj alma basarisiz", GUNLUK_SEVIYE_ERROR);

                    QString detay = "Yerel imaj alma basarisiz oldu.";
                    if (!hata.isEmpty()) {
                        detay += "\n\nDetay: " + hata;
                    }
                    QMessageBox::critical(this, "Hata", detay);
                }
            }, Qt::QueuedConnection);

            is_temizle(is);
        }).detach();
    }

    void hash_hesapla() {
        const QString dosya = hash_dosya_giris_->text().trimmed();
        if (dosya.isEmpty()) {
            QMessageBox::critical(this, "Hata", "Dosya secilmedi!");
            return;
        }
        if (!QFileInfo::exists(dosya)) {
            QMessageBox::critical(this, "Hata", "Dosya bulunamadi!");
            return;
        }

        // Dosya boyutunu kontrol et - buyuk dosyalar icin uyar
        QFileInfo info(dosya);
        const qint64 boyut = info.size();
        const double gb = boyut / (1024.0 * 1024.0 * 1024.0);
        
        log_ekle("Hash hesaplaniyor: " + dosya + " (" + QString::number(gb, 'f', 2) + " GB)", GUNLUK_SEVIYE_INFO);
        
        // Hash etiketlerini temizle ve durum goster
        md5_label_->setText("MD5: Hesaplaniyor...");
        sha1_label_->setText("SHA1: Hesaplaniyor...");
        sha256_label_->setText("SHA256: Hesaplaniyor...");
        sha512_label_->setText("SHA512: Hesaplaniyor...");
        
        // Asenkron hash hesaplama
        std::thread([this, dosya]() {
            char hash[129] = {0};
            QString md5, sha1, sha256, sha512;
            
            // MD5
            if (hash_dosya_hesapla(dosya.toUtf8().constData(), HASH_MD5, hash, sizeof(hash))) {
                md5 = QString::fromUtf8(hash);
            }
            
            // SHA1
            if (hash_dosya_hesapla(dosya.toUtf8().constData(), HASH_SHA1, hash, sizeof(hash))) {
                sha1 = QString::fromUtf8(hash);
            }
            
            // SHA256
            if (hash_dosya_hesapla(dosya.toUtf8().constData(), HASH_SHA256, hash, sizeof(hash))) {
                sha256 = QString::fromUtf8(hash);
            }
            
            // SHA512
            if (hash_dosya_hesapla(dosya.toUtf8().constData(), HASH_SHA512, hash, sizeof(hash))) {
                sha512 = QString::fromUtf8(hash);
            }
            
            QMetaObject::invokeMethod(this, [this, md5, sha1, sha256, sha512]() {
                if (!md5.isEmpty()) md5_label_->setText("MD5: " + md5);
                else md5_label_->setText("MD5: Hata!");
                
                if (!sha1.isEmpty()) sha1_label_->setText("SHA1: " + sha1);
                else sha1_label_->setText("SHA1: Hata!");
                
                if (!sha256.isEmpty()) sha256_label_->setText("SHA256: " + sha256);
                else sha256_label_->setText("SHA256: Hata!");
                
                if (!sha512.isEmpty()) sha512_label_->setText("SHA512: " + sha512);
                else sha512_label_->setText("SHA512: Hata!");
                
                log_ekle("Hash hesaplama tamamlandi", GUNLUK_SEVIYE_INFO);
            }, Qt::QueuedConnection);
        }).detach();
    }

    void hash_karsilastir() {
        const QString beklenen = hash_karsilastir_giris_->text().trimmed();
        if (beklenen.isEmpty()) {
            hash_sonuc_label_->setText("Hash girin!");
            return;
        }

        const QString md5 = md5_label_->text().section(':', 1).trimmed();
        const QString sha1 = sha1_label_->text().section(':', 1).trimmed();
        const QString sha256 = sha256_label_->text().section(':', 1).trimmed();
        const QString sha512 = sha512_label_->text().section(':', 1).trimmed();

        if (::hash_karsilastir(beklenen.toUtf8().constData(), md5.toUtf8().constData())) {
            hash_sonuc_label_->setText("MD5 eslesti!");
        } else if (::hash_karsilastir(beklenen.toUtf8().constData(), sha1.toUtf8().constData())) {
            hash_sonuc_label_->setText("SHA1 eslesti!");
        } else if (::hash_karsilastir(beklenen.toUtf8().constData(), sha256.toUtf8().constData())) {
            hash_sonuc_label_->setText("SHA256 eslesti!");
        } else if (::hash_karsilastir(beklenen.toUtf8().constData(), sha512.toUtf8().constData())) {
            hash_sonuc_label_->setText("SHA512 eslesti!");
        } else {
            hash_sonuc_label_->setText("Eslesme bulunamadi!");
        }
    }

    void vaka_olustur() {
        const QString vaka_adi = vaka_giris_->text().trimmed();
        if (vaka_adi.isEmpty()) {
            QMessageBox::critical(this, "Hata", "Vaka adi girin!");
            return;
        }

        if (kasa_) {
            kanit_kasasi_kapat(kasa_);
            kasa_ = nullptr;
        }

        char* ana_klasor = g_build_filename(g_get_home_dir(), "Worm", nullptr);
        kasa_ = kanit_kasasi_olustur(ana_klasor, vaka_adi.toUtf8().constData());
        g_free(ana_klasor);

        if (kasa_) {
            vaka_durum_label_->setText("Vaka olusturuldu");
            log_ekle("Yeni vaka olusturuldu", GUNLUK_SEVIYE_INFO);
        } else {
            vaka_durum_label_->setText("Vaka olusturulamadi!");
        }
    }

    void dosyalari_listele() {
        if (!kasa_) {
            QMessageBox::critical(this, "Hata", "Once vaka olusturun!");
            return;
        }

        dosya_listesi_->clear();

        const QString secim = klasor_secim_ ? klasor_secim_->currentData().toString() : "ciktilar";

        auto dizinden_ekle = [this](const QString& yol, const QString& on_ek = QString()) {
            GDir* dir = g_dir_open(yol.toUtf8().constData(), 0, nullptr);
            if (!dir) {
                return;
            }
            const char* isim = nullptr;
            while ((isim = g_dir_read_name(dir)) != nullptr) {
                QString gorunen = on_ek.isEmpty()
                    ? QString::fromUtf8(isim)
                    : (on_ek + "/" + QString::fromUtf8(isim));
                dosya_listesi_->addItem(gorunen);
            }
            g_dir_close(dir);
        };

        if (secim == "disk_imajlari") {
            dizinden_ekle(QString::fromUtf8(kasa_->ciktilar_klasoru) + "/disk_imajlari");
        } else if (secim == "ram") {
            dizinden_ekle(QString::fromUtf8(kasa_->ciktilar_klasoru) + "/ram");
        } else if (secim == "ciktilar") {
            dizinden_ekle(QString::fromUtf8(kasa_->ciktilar_klasoru));
            dizinden_ekle(QString::fromUtf8(kasa_->ciktilar_klasoru) + "/disk_imajlari", "disk_imajlari");
            dizinden_ekle(QString::fromUtf8(kasa_->ciktilar_klasoru) + "/ram", "ram");
        } else {
            GList* dosyalar = kanit_kasasi_dosyalari_listele(kasa_, secim.toUtf8().constData());
            for (GList* l = dosyalar; l != nullptr; l = l->next) {
                char* tam_yol = static_cast<char*>(l->data);
                char* isim = g_path_get_basename(tam_yol);
                if (isim) {
                    dosya_listesi_->addItem(QString::fromUtf8(isim));
                    g_free(isim);
                }
            }
            g_list_free_full(dosyalar, g_free);
        }

        log_ekle("Dosyalar listelendi", GUNLUK_SEVIYE_INFO);
    }

    void not_ekle() {
        if (!kasa_) {
            QMessageBox::critical(this, "Hata", "Once vaka olusturun!");
            return;
        }
        if (!rapor_not_giris_) {
            return;
        }

        const QString not_metin = rapor_not_giris_->toPlainText().trimmed();
        if (not_metin.isEmpty()) {
            QMessageBox::critical(this, "Hata", "Not metni bos olamaz!");
            return;
        }

        if (kanit_kasasi_not_ekle(kasa_, not_metin.toUtf8().constData())) {
            if (rapor_durum_label_) {
                rapor_durum_label_->setText("Not eklendi");
            }
            log_ekle("Kanit kasasina not eklendi", GUNLUK_SEVIYE_INFO);
        } else {
            if (rapor_durum_label_) {
                rapor_durum_label_->setText("Not eklenemedi");
            }
            QMessageBox::critical(this, "Hata", "Not eklenemedi!");
        }
    }

    void rapor_olustur() {
        if (!kasa_) {
            QMessageBox::critical(this, "Hata", "Once vaka olusturun!");
            return;
        }

        RaporBilgisi bilgi;
        memset(&bilgi, 0, sizeof(bilgi));

        QString baslik = rapor_baslik_giris_ ? rapor_baslik_giris_->text().trimmed() : QString();
        if (baslik.isEmpty()) {
            baslik = "Adli Bilisim Teknik Raporu";
        }
        strncpy(bilgi.baslik, baslik.toUtf8().constData(), sizeof(bilgi.baslik) - 1);

        QString aciklama = rapor_not_giris_ ? rapor_not_giris_->toPlainText().trimmed() : QString();
        if (aciklama.isEmpty()) {
            aciklama = "Rapor arayuz uzerinden olusturuldu.";
        }
        strncpy(bilgi.aciklama, aciklama.toUtf8().constData(), sizeof(bilgi.aciklama) - 1);

        const QString olusturan = qEnvironmentVariable("USERNAME", qEnvironmentVariable("USER", "bilinmiyor"));
        strncpy(bilgi.olusturan, olusturan.toUtf8().constData(), sizeof(bilgi.olusturan) - 1);
        strncpy(bilgi.kaynak, "Worm", sizeof(bilgi.kaynak) - 1);

        const QString tarih = QDateTime::currentDateTime().toString("yyyy-MM-dd HH:mm:ss");
        strncpy(bilgi.tarih, tarih.toUtf8().constData(), sizeof(bilgi.tarih) - 1);

        QString sha256 = sha256_label_ ? sha256_label_->text().section(':', 1).trimmed() : QString();
        if (sha256 != "-" && !sha256.isEmpty()) {
            strncpy(bilgi.hash, sha256.toUtf8().constData(), sizeof(bilgi.hash) - 1);
        }

        const QString format_metin = rapor_format_secim_ ? rapor_format_secim_->currentData().toString() : "txt";
        RaporFormat format = (format_metin == "json") ? RAPOR_FORMAT_JSON : RAPOR_FORMAT_TXT;

        char* dosya_adi = rapor_yeni_dosya_adi(kasa_->vaka_adi, format);
        if (!dosya_adi) {
            QMessageBox::critical(this, "Hata", "Rapor dosya adi olusturulamadi!");
            return;
        }

        char* tam_yol = kanit_kasasi_yeni_dosya(kasa_, "raporlar", dosya_adi);
        g_free(dosya_adi);
        if (!tam_yol) {
            QMessageBox::critical(this, "Hata", "Rapor yolu olusturulamadi!");
            return;
        }

        const bool ok = ::rapor_olustur(&bilgi, format, tam_yol, kasa_);
        if (ok) {
            if (rapor_durum_label_) {
                rapor_durum_label_->setText(QString("Rapor olusturuldu: %1").arg(QString::fromUtf8(tam_yol)));
            }
            log_ekle("Rapor olusturuldu: " + QString(tam_yol), GUNLUK_SEVIYE_INFO);
        }
        g_free(tam_yol);
    }

    // WinPMEM RAM yardimci fonksiyonlar
    void winpmem_kontrol_yap() {
        if (!baglanti_) {
            QMessageBox::critical(this, "Hata", "Once Uzak Disk sekmesinden baglanti yapin!");
            return;
        }

        // Butonlari devre disi birak
        winpmem_kontrol_btn_->setEnabled(false);
        winpmem_durum_label_->setText("WinPMEM kontrol ediliyor...");
        
        log_ekle("WinPMEM kontrolu baslatildi", GUNLUK_SEVIYE_INFO);

        // Asenkron kontrol
        std::thread([this]() {
            bool binary_mevcut = false;
            bool yetki = false;
            int64_t ram_boyut = 0;
            char mesaj[256] = {0};

            bool protokol_ok = uzak_winpmem_kontrol(
                baglanti_,
                &binary_mevcut,
                &yetki,
                &ram_boyut,
                mesaj,
                sizeof(mesaj)
            );
            
            double gb = ram_boyut / (1024.0 * 1024.0 * 1024.0);
            QString bilgi;
            if (protokol_ok) {
                bilgi = QString("WinPMEM: %1 | Yetki: %2 | RAM: %3 GB | %4")
                    .arg(binary_mevcut ? "Mevcut" : "Bulunamadi")
                    .arg(yetki ? "Yonetici" : "Yetkisiz")
                    .arg(QString::number(gb, 'f', 2))
                    .arg(QString::fromUtf8(mesaj));
            } else {
                bilgi = "Ajanla WinPMEM kontrolu basarisiz";
            }
            
            QMetaObject::invokeMethod(this, [this, bilgi, binary_mevcut, yetki, protokol_ok]() {
                winpmem_kontrol_btn_->setEnabled(true);
                winpmem_bilgi_label_->setText(bilgi);

                if (!protokol_ok) {
                    winpmem_durum_label_->setText("Ajanla kontrol basarisiz");
                    winpmem_baslat_btn_->setEnabled(false);
                    log_ekle("WinPMEM kontrolu basarisiz", GUNLUK_SEVIYE_ERROR);
                    QMessageBox::warning(this, "Uyari", "WinPMEM kontrolu uzak ajan uzerinden yapilamadi.");
                    return;
                }
                
                if (binary_mevcut && yetki) {
                    winpmem_durum_label_->setText("WinPMEM hazir - RAM edinimi yapilabilir");
                    winpmem_baslat_btn_->setEnabled(true);
                    log_ekle("WinPMEM hazir - " + bilgi, GUNLUK_SEVIYE_INFO);
                } else {
                    winpmem_durum_label_->setText("WinPMEM hazir degil");
                    winpmem_baslat_btn_->setEnabled(false);
                    if (!binary_mevcut) {
                        QMessageBox::warning(this, "Uyari", 
                            "WinPMEM binary bulunamadi!\n"
                            "Imzali RC2 exe agent dizininde olmali.");
                    }
                    if (!yetki) {
                        QMessageBox::warning(this, "Uyari", 
                            "Yonetici yetkisi gerekli!\n"
                            "Windows Agent yonetici olarak calistirilmali.");
                    }
                    log_ekle("WinPMEM hazir degil - " + bilgi, GUNLUK_SEVIYE_WARN);
                }
            }, Qt::QueuedConnection);
        }).detach();
    }

    static void winpmem_ilerleme_callback(int64_t okunan, int64_t toplam, void* kullanici_verisi) {
        AnaPencere* pencere = static_cast<AnaPencere*>(kullanici_verisi);
        if (!pencere) return;
        
        if (toplam <= 0) return;
        const int oran = static_cast<int>((okunan * 100) / toplam);
        
        QMetaObject::invokeMethod(pencere, [pencere, oran]() {
            if (pencere->winpmem_ilerleme_) {
                pencere->winpmem_ilerleme_->setValue(oran);
            }
            if (pencere->winpmem_indirme_asamasi_) {
                int kademe = (oran / 10) * 10;
                if (kademe > 100) {
                    kademe = 100;
                }
                if (kademe >= 0 && kademe != pencere->winpmem_son_log_oran_) {
                    pencere->winpmem_son_log_oran_ = kademe;
                    pencere->log_ekle(
                        "RAM indirme ilerleme: %" + QString::number(kademe),
                        GUNLUK_SEVIYE_INFO
                    );
                }
            }
        }, Qt::QueuedConnection);
    }

    void winpmem_baslat() {
        if (!baglanti_) {
            QMessageBox::critical(this, "Hata", "Once baglanti ve WinPMEM kontrolu yapin!");
            return;
        }

        if (!vaka_sec_veya_olustur("RAM imaji")) {
            return;
        }

        QString cikti_dosya = winpmem_cikti_giris_->text().trimmed();
        if (cikti_dosya.isEmpty()) {
            cikti_dosya = "memory_dump.raw";
        }

        const QString ram_klasor = vaka_cikti_klasoru_hazirla("ram");
        if (ram_klasor.isEmpty()) {
            QMessageBox::critical(this, "Hata", "RAM cikti klasoru olusturulamadi!");
            return;
        }

        QString dosya_adi = QFileInfo(cikti_dosya).fileName();
        if (dosya_adi.isEmpty()) {
            dosya_adi = "memory_dump.raw";
        }
        QString tam_cikti = ram_klasor + "/" + dosya_adi;
        winpmem_cikti_giris_->setText(tam_cikti);

        // Butonlari devre disi birak
        winpmem_baslat_btn_->setEnabled(false);
        winpmem_indir_btn_->setEnabled(false);
        winpmem_durum_label_->setText("RAM edinimi baslatildi...");
        winpmem_indirme_asamasi_ = false;
        winpmem_son_log_oran_ = -1;
        
        QString ajan_cikti = QFileInfo(tam_cikti).fileName();
        if (ajan_cikti.isEmpty()) {
            ajan_cikti = "memory_dump.raw";
        }

        log_ekle("WinPMEM RAM edinimi baslatildi (uzak ajan): " + ajan_cikti, GUNLUK_SEVIYE_INFO);

        // Asenkron RAM edinim
        std::thread([this, tam_cikti, ajan_cikti]() {
            char sonuc_metin[256] = {0};
            bool sonuc = uzak_ram_edinim_baslat_ve_takip(
                baglanti_,
                ajan_cikti.toUtf8().constData(),
                winpmem_ilerleme_callback,
                this,
                sonuc_metin,
                sizeof(sonuc_metin)
            );

            bool otomatik_indir = false;
            if (sonuc) {
                QMetaObject::invokeMethod(this, [this, &otomatik_indir, tam_cikti]() {
                    QMessageBox::StandardButton cevap = QMessageBox::question(
                        this,
                        "RAM Edinimi Tamamlandi",
                        "RAM edinimi tamamlandi.\n\n"
                        "RAM dosyasi simdi bu makineye indirilsin mi?\n"
                        "Hayir secerseniz daha sonra 'RAM Indir' ile indirebilirsiniz."
                    );
                    otomatik_indir = (cevap == QMessageBox::Yes);
                    if (otomatik_indir) {
                        winpmem_indirme_asamasi_ = true;
                        winpmem_son_log_oran_ = -1;
                        winpmem_durum_label_->setText("RAM dosyasi otomatik indiriliyor...");
                        log_ekle("Kullanici otomatik RAM indirmeyi onayladi", GUNLUK_SEVIYE_INFO);
                    } else {
                        winpmem_indirme_asamasi_ = false;
                        log_ekle("Kullanici otomatik RAM indirmeyi erteledi", GUNLUK_SEVIYE_INFO);
                    }
                }, Qt::BlockingQueuedConnection);
            }

            bool indir_ok = false;
            char indir_metin[256] = {0};
            if (sonuc && otomatik_indir) {
                indir_ok = uzak_ram_dosya_indir(
                    baglanti_,
                    ajan_cikti.toUtf8().constData(),
                    tam_cikti.toUtf8().constData(),
                    winpmem_ilerleme_callback,
                    this,
                    indir_metin,
                    sizeof(indir_metin)
                );
            }
            
            QMetaObject::invokeMethod(this, [this, sonuc, otomatik_indir, indir_ok, tam_cikti, sonuc_metin, indir_metin]() {
                winpmem_baslat_btn_->setEnabled(true);
                winpmem_indirme_asamasi_ = false;
                if (sonuc && otomatik_indir && indir_ok) {
                    winpmem_durum_label_->setText("RAM edinimi + indirme tamamlandi: " + tam_cikti);
                    winpmem_indir_btn_->setEnabled(true);
                    log_ekle("WinPMEM RAM edinimi ve indirme tamamlandi: " + tam_cikti, GUNLUK_SEVIYE_INFO);
                    QMessageBox::information(this, "Basarili", 
                        "RAM edinimi tamamlandi ve dosya bu makineye indirildi.\n\n"
                        + tam_cikti);
                } else if (sonuc && otomatik_indir && !indir_ok) {
                    winpmem_durum_label_->setText("RAM edinimi tamamlandi ama indirme basarisiz");
                    winpmem_indir_btn_->setEnabled(true);
                    QString hata_mesaj = QString("RAM uzak makinada olustu, indirme hatasi: %1")
                        .arg(QString::fromUtf8(indir_metin));
                    log_ekle(hata_mesaj, GUNLUK_SEVIYE_ERROR);
                    QMessageBox::warning(this, "Uyari", hata_mesaj);
                } else if (sonuc && !otomatik_indir) {
                    winpmem_durum_label_->setText("RAM edinimi tamamlandi. Indirme ertelendi.");
                    winpmem_indir_btn_->setEnabled(true);
                    log_ekle("RAM edinimi tamamlandi; indirme kullanici istegiyle ertelendi", GUNLUK_SEVIYE_INFO);
                    QMessageBox::information(this, "Bilgi",
                        "RAM edinimi tamamlandi.\n\n"
                        "Istediginiz zaman 'RAM Indir' butonuyla indirebilirsiniz.");
                } else {
                    winpmem_durum_label_->setText("RAM edinimi basarisiz!");
                    winpmem_indir_btn_->setEnabled(true);
                    QString hata_mesaj = QString("WinPMEM hatasi: %1").arg(QString::fromUtf8(sonuc_metin));
                    log_ekle(hata_mesaj, GUNLUK_SEVIYE_ERROR);
                    QMessageBox::critical(this, "Hata", hata_mesaj);
                }
            }, Qt::QueuedConnection);
        }).detach();
    }

    void winpmem_indir() {
        if (!baglanti_) {
            QMessageBox::critical(this, "Hata", "Baglanti yok!");
            return;
        }

        QString dosya_adi = winpmem_cikti_giris_->text().trimmed();
        if (dosya_adi.isEmpty()) {
            dosya_adi = "memory_dump.raw";
        }

        winpmem_indir_btn_->setEnabled(false);
        winpmem_durum_label_->setText("RAM dosyasi indiriliyor...");
        winpmem_indirme_asamasi_ = true;
        winpmem_son_log_oran_ = -1;
        
        log_ekle("RAM dosyasi indiriliyor: " + dosya_adi, GUNLUK_SEVIYE_INFO);

        QString yerel_yol = dosya_adi;
        if (QFileInfo(dosya_adi).isRelative() && kasa_) {
            QString ram_klasor = vaka_cikti_klasoru_hazirla("ram");
            if (!ram_klasor.isEmpty()) {
                yerel_yol = ram_klasor + "/" + QFileInfo(dosya_adi).fileName();
            }
        }

        QString ajan_dosya = QFileInfo(dosya_adi).fileName();

        std::thread([this, ajan_dosya, yerel_yol]() {
            char sonuc_metin[256] = {0};
            bool ok = uzak_ram_dosya_indir(
                baglanti_,
                ajan_dosya.toUtf8().constData(),
                yerel_yol.toUtf8().constData(),
                winpmem_ilerleme_callback,
                this,
                sonuc_metin,
                sizeof(sonuc_metin)
            );

            QMetaObject::invokeMethod(this, [this, ok, yerel_yol, sonuc_metin]() {
                winpmem_indirme_asamasi_ = false;
                winpmem_indir_btn_->setEnabled(true);
                if (ok) {
                    winpmem_durum_label_->setText("RAM dosyasi indirildi: " + yerel_yol);
                    log_ekle("RAM dosyasi indirildi: " + yerel_yol, GUNLUK_SEVIYE_INFO);
                    QMessageBox::information(this, "Basarili", "RAM dosyasi indirildi:\n" + yerel_yol);
                } else {
                    winpmem_durum_label_->setText("RAM indirme basarisiz");
                    QString detay = QString::fromUtf8(sonuc_metin);
                    if (detay.trimmed().isEmpty()) {
                        detay = "Ajan dosya indirme hatasi";
                    }
                    log_ekle("RAM indirme hatasi: " + detay, GUNLUK_SEVIYE_ERROR);
                    QMessageBox::critical(this, "Hata", "RAM dosyasi indirilemedi:\n" + detay);
                }
            }, Qt::QueuedConnection);
        }).detach();
    }

    static void linux_uzak_ram_ilerleme_callback(int64_t okunan, int64_t toplam, void* kullanici_verisi) {
        AnaPencere* pencere = static_cast<AnaPencere*>(kullanici_verisi);
        if (!pencere || toplam <= 0) return;

        const int oran = static_cast<int>((okunan * 100) / toplam);
        QMetaObject::invokeMethod(pencere, [pencere, oran]() {
            if (pencere->linux_uzak_ram_ilerleme_) {
                pencere->linux_uzak_ram_ilerleme_->setValue(oran);
            }
        }, Qt::QueuedConnection);
    }

    void linux_uzak_ram_baglan() {
        if (imaj_calisiyor_) {
            QMessageBox::warning(this, "Bilgi", "Imaj alma surerken yeniden baglanamazsiniz.");
            return;
        }
        if (baglaniyor_.load()) {
            QMessageBox::warning(this, "Bilgi", "Baglanti islemi devam ediyor...");
            return;
        }

        if (!linux_uzak_ram_ip_giris_ || !linux_uzak_ram_port_giris_) {
            return;
        }

        const QString ip = linux_uzak_ram_ip_giris_->text().trimmed();
        const QString port_metin = linux_uzak_ram_port_giris_->text().trimmed();
        const QString token = linux_uzak_ram_token_giris_ ? linux_uzak_ram_token_giris_->text().trimmed() : QString();

        if (ip.isEmpty() || port_metin.isEmpty()) {
            QMessageBox::critical(this, "Hata", "IP ve port girin!");
            return;
        }

        bool port_ok = false;
        const int port = port_metin.toInt(&port_ok);
        if (!port_ok || port <= 0 || port > 65535) {
            QMessageBox::critical(this, "Hata", "Gecersiz port!");
            return;
        }

        linux_uzak_ram_baglan_btn_->setEnabled(false);
        baglaniyor_ = true;
        if (linux_uzak_ram_durum_label_) {
            linux_uzak_ram_durum_label_->setText(cevir_metin("Baglaniyor..."));
        }

        if (baglanti_) {
            uzak_disk_baglanti_kapat(baglanti_);
            baglanti_ = nullptr;
        }

        if (linux_uzak_ram_vpn_kullan_secim_ && linux_uzak_ram_vpn_kullan_secim_->isChecked()) {
            if (!vpn_yonetici_) {
                vpn_yonetici_ = wireguard_yonetici_olustur();
            }
            if (!vpn_yonetici_) {
                QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("VPN yoneticisi olusturulamadi!"));
                linux_uzak_ram_baglan_btn_->setEnabled(true);
                baglaniyor_ = false;
                return;
            }
            if (wireguard_durum(vpn_yonetici_) == 0) {
                if (wireguard_baslat(vpn_yonetici_, vpn_config_yolu_.toUtf8().constData()) != 0) {
                    QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("VPN baslatilamadi! Once VPN Yapilandir ile ayarlari kontrol edin."));
                    linux_uzak_ram_baglan_btn_->setEnabled(true);
                    baglaniyor_ = false;
                    return;
                }
                log_ekle("WireGuard VPN baslatildi", GUNLUK_SEVIYE_INFO);
            }
        }

        std::thread([this, ip, port, token]() {
            UzakDiskBaglanti* yeni_baglanti = uzak_disk_baglanti_olustur(
                ip.toUtf8().constData(),
                port,
                token.isEmpty() ? nullptr : token.toUtf8().constData());

            bool basarili = false;
            if (yeni_baglanti) {
                basarili = uzak_disk_baglan(yeni_baglanti);
            }

            QMetaObject::invokeMethod(this, [this, yeni_baglanti, basarili, ip]() {
                baglaniyor_ = false;
                linux_uzak_ram_baglan_btn_->setEnabled(true);

                if (basarili) {
                    baglanti_ = yeni_baglanti;
                    if (linux_uzak_ram_durum_label_) {
                        linux_uzak_ram_durum_label_->setText(cevir_metin("Baglandi - ") + ip);
                    }
                    log_ekle("Linux uzak RAM ajanina baglandi: " + ip, GUNLUK_SEVIYE_INFO);
                } else {
                    QString detay;
                    if (yeni_baglanti && yeni_baglanti->son_hata[0] != '\0') {
                        detay = QString::fromUtf8(yeni_baglanti->son_hata);
                    }
                    if (yeni_baglanti) {
                        uzak_disk_baglanti_kapat(yeni_baglanti);
                    }
                    if (linux_uzak_ram_durum_label_) {
                        linux_uzak_ram_durum_label_->setText(cevir_metin("Baglanti basarisiz!"));
                    }
                    QString mesaj = cevir_metin("Sunucuya baglanilamadi!");
                    if (!detay.isEmpty()) {
                        mesaj += "\n\n" + cevir_metin("Ajan detayi: ") + detay;
                    }
                    QMessageBox::critical(this, cevir_metin("Hata"), mesaj);
                }
            }, Qt::QueuedConnection);
        }).detach();
    }

    void linux_uzak_ram_kontrol_yap() {
        if (!baglanti_) {
            QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("Once baglanin!"));
            return;
        }

        linux_uzak_ram_kontrol_btn_->setEnabled(false);
        if (linux_uzak_ram_durum_label_) {
            linux_uzak_ram_durum_label_->setText(cevir_metin("AVML kontrol ediliyor..."));
        }

        std::thread([this]() {
            bool avml_mevcut = false;
            bool yetki = false;
            int64_t ram_boyut = 0;
            char mesaj[256] = {0};

            bool protokol_ok = uzak_avml_kontrol(
                baglanti_,
                &avml_mevcut,
                &yetki,
                &ram_boyut,
                mesaj,
                sizeof(mesaj)
            );

            double gb = ram_boyut / (1024.0 * 1024.0 * 1024.0);
            QString bilgi;
            if (protokol_ok) {
                bilgi = QString("AVML: %1 | Yetki: %2 | RAM: %3 GB | %4")
                            .arg(avml_mevcut ? "Mevcut" : "Bulunamadi")
                            .arg(yetki ? "root" : "root degil")
                            .arg(QString::number(gb, 'f', 2))
                            .arg(QString::fromUtf8(mesaj));
            } else {
                bilgi = "Ajanla AVML kontrolu basarisiz";
            }

            QMetaObject::invokeMethod(this, [this, protokol_ok, avml_mevcut, yetki, bilgi]() {
                linux_uzak_ram_kontrol_btn_->setEnabled(true);
                if (linux_uzak_ram_bilgi_label_) {
                    linux_uzak_ram_bilgi_label_->setText(bilgi);
                }
                if (!protokol_ok) {
                    if (linux_uzak_ram_durum_label_) {
                        linux_uzak_ram_durum_label_->setText(cevir_metin("Ajanla kontrol basarisiz"));
                    }
                    linux_uzak_ram_baslat_btn_->setEnabled(false);
                    return;
                }

                linux_uzak_ram_baslat_btn_->setEnabled(avml_mevcut && yetki);
                if (linux_uzak_ram_durum_label_) {
                    linux_uzak_ram_durum_label_->setText((avml_mevcut && yetki) ? cevir_metin("AVML hazir") : cevir_metin("AVML hazir degil"));
                }
            }, Qt::QueuedConnection);
        }).detach();
    }

    void linux_uzak_ram_baslat() {
        if (!baglanti_) {
            QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("Once baglanin!"));
            return;
        }

        if (!vaka_sec_veya_olustur("Linux uzak RAM imaji")) {
            return;
        }

        QString cikti_dosya = linux_uzak_ram_cikti_giris_ ? linux_uzak_ram_cikti_giris_->text().trimmed() : QString();
        if (cikti_dosya.isEmpty()) {
            cikti_dosya = "memory_dump_linux_remote.raw";
        }

        const QString ram_klasor = vaka_cikti_klasoru_hazirla("ram");
        if (ram_klasor.isEmpty()) {
            QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("RAM cikti klasoru olusturulamadi!"));
            return;
        }

        QString dosya_adi = QFileInfo(cikti_dosya).fileName();
        if (dosya_adi.isEmpty()) {
            dosya_adi = "memory_dump_linux_remote.raw";
        }
        QString tam_cikti = ram_klasor + "/" + dosya_adi;
        if (linux_uzak_ram_cikti_giris_) {
            linux_uzak_ram_cikti_giris_->setText(tam_cikti);
        }

        linux_uzak_ram_baslat_btn_->setEnabled(false);
        linux_uzak_ram_indir_btn_->setEnabled(false);
        if (linux_uzak_ram_durum_label_) {
            linux_uzak_ram_durum_label_->setText(cevir_metin("RAM edinimi baslatildi..."));
        }

        const QString ajan_cikti = QFileInfo(tam_cikti).fileName();

        std::thread([this, tam_cikti, ajan_cikti]() {
            char sonuc_metin[256] = {0};
            bool sonuc = uzak_ram_edinim_baslat_ve_takip(
                baglanti_,
                ajan_cikti.toUtf8().constData(),
                linux_uzak_ram_ilerleme_callback,
                this,
                sonuc_metin,
                sizeof(sonuc_metin)
            );

            QMetaObject::invokeMethod(this, [this, sonuc, tam_cikti, sonuc_metin]() {
                linux_uzak_ram_baslat_btn_->setEnabled(true);
                linux_uzak_ram_indir_btn_->setEnabled(true);
                if (sonuc) {
                    if (linux_uzak_ram_durum_label_) {
                        linux_uzak_ram_durum_label_->setText(cevir_metin("RAM edinimi tamamlandi. Indirilebilir."));
                    }
                    QMessageBox::information(this, cevir_metin("Bilgi"), cevir_metin("Linux uzak RAM edinimi tamamlandi.\nRAM Indir ile cekebilirsiniz."));
                } else {
                    if (linux_uzak_ram_durum_label_) {
                        linux_uzak_ram_durum_label_->setText(cevir_metin("RAM edinimi basarisiz"));
                    }
                    QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("Linux uzak RAM hatasi: ") + QString::fromUtf8(sonuc_metin));
                }
            }, Qt::QueuedConnection);
        }).detach();
    }

    void linux_uzak_ram_indir() {
        if (!baglanti_) {
            QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("Baglanti yok!"));
            return;
        }

        QString dosya_adi = linux_uzak_ram_cikti_giris_ ? linux_uzak_ram_cikti_giris_->text().trimmed() : QString();
        if (dosya_adi.isEmpty()) {
            dosya_adi = "memory_dump_linux_remote.raw";
        }

        QString yerel_yol = dosya_adi;
        if (QFileInfo(dosya_adi).isRelative() && kasa_) {
            QString ram_klasor = vaka_cikti_klasoru_hazirla("ram");
            if (!ram_klasor.isEmpty()) {
                yerel_yol = ram_klasor + "/" + QFileInfo(dosya_adi).fileName();
            }
        }
        QString ajan_dosya = QFileInfo(dosya_adi).fileName();

        linux_uzak_ram_indir_btn_->setEnabled(false);
        if (linux_uzak_ram_durum_label_) {
            linux_uzak_ram_durum_label_->setText(cevir_metin("RAM dosyasi indiriliyor..."));
        }

        std::thread([this, ajan_dosya, yerel_yol]() {
            char sonuc_metin[256] = {0};
            bool ok = uzak_ram_dosya_indir(
                baglanti_,
                ajan_dosya.toUtf8().constData(),
                yerel_yol.toUtf8().constData(),
                linux_uzak_ram_ilerleme_callback,
                this,
                sonuc_metin,
                sizeof(sonuc_metin)
            );

            QMetaObject::invokeMethod(this, [this, ok, yerel_yol, sonuc_metin]() {
                linux_uzak_ram_indir_btn_->setEnabled(true);
                if (ok) {
                    if (linux_uzak_ram_durum_label_) {
                        linux_uzak_ram_durum_label_->setText(cevir_metin("RAM dosyasi indirildi: ") + yerel_yol);
                    }
                    QMessageBox::information(this, cevir_metin("Basarili"), cevir_metin("RAM dosyasi indirildi:\n") + yerel_yol);
                } else {
                    if (linux_uzak_ram_durum_label_) {
                        linux_uzak_ram_durum_label_->setText(cevir_metin("RAM indirme basarisiz"));
                    }
                    QMessageBox::critical(this, cevir_metin("Hata"), cevir_metin("RAM dosyasi indirilemedi:\n") + QString::fromUtf8(sonuc_metin));
                }
            }, Qt::QueuedConnection);
        }).detach();
    }

    static void winpmem_yerel_ilerleme_callback(int64_t okunan, int64_t toplam, void* kullanici_verisi) {
        AnaPencere* pencere = static_cast<AnaPencere*>(kullanici_verisi);
        if (!pencere || toplam <= 0) {
            return;
        }

        const int oran = static_cast<int>((okunan * 100) / toplam);
        QMetaObject::invokeMethod(pencere, [pencere, oran]() {
            if (pencere->winpmem_yerel_ilerleme_) {
                pencere->winpmem_yerel_ilerleme_->setValue(oran);
            }
            if (pencere->genel_ilerleme_) {
                pencere->genel_ilerleme_->setValue(oran);
            }
        }, Qt::QueuedConnection);
    }

    bool winpmem_yerel_dosya_indir(QString* indirilen_yol, QString* hata) {
#ifdef _WIN32
        const QString hedef = QDir(QCoreApplication::applicationDirPath()).filePath("go-winpmem_amd64_1.0-rc2_signed.exe");
        const QString url = "https://github.com/Velocidex/WinPmem/releases/download/v4.0.rc1/go-winpmem_amd64_1.0-rc2_signed.exe";

        const QString script = QString(
            "$ProgressPreference='SilentlyContinue';"
            "Invoke-WebRequest -UseBasicParsing -Uri '%1' -OutFile '%2';"
        ).arg(url, QDir::toNativeSeparators(hedef));

        QProcess proc;
        proc.start("powershell", QStringList()
            << "-NoProfile"
            << "-ExecutionPolicy" << "Bypass"
            << "-Command" << script);

        if (!proc.waitForFinished(300000)) {
            if (hata) {
                *hata = "PowerShell indirme zaman asimi";
            }
            proc.kill();
            return false;
        }

        if (proc.exitStatus() != QProcess::NormalExit || proc.exitCode() != 0) {
            if (hata) {
                *hata = QString::fromUtf8(proc.readAllStandardError());
                if (hata->trimmed().isEmpty()) {
                    *hata = "PowerShell WinPMEM indirme hatasi";
                }
            }
            return false;
        }

        QFileInfo fi(hedef);
        if (!fi.exists() || fi.size() <= 0) {
            if (hata) {
                *hata = "WinPMEM dosyasi indirildi ama bos/gozukmuyor";
            }
            return false;
        }

        if (indirilen_yol) {
            *indirilen_yol = hedef;
        }
        return true;
#else
        if (hata) {
            *hata = "Bu ozellik sadece Windows'ta calisir";
        }
        (void)indirilen_yol;
        return false;
#endif
    }

    void winpmem_yerel_kontrol_yap() {
#ifndef _WIN32
        QMessageBox::warning(this, "Bilgi", "Bu ozellik sadece Windows'ta calisir.");
        return;
#else
        if (!winpmem_yerel_edinim_) {
            winpmem_yerel_edinim_ = winpmem_edinim_olustur();
        }
        if (!winpmem_yerel_edinim_) {
            QMessageBox::critical(this, "Hata", "WinPMEM edinim nesnesi olusturulamadi.");
            return;
        }

        winpmem_yerel_kontrol_btn_->setEnabled(false);
        winpmem_yerel_durum_label_->setText("WinPMEM kontrol ediliyor...");

        QString bulunan;
        const QString app_dir = QCoreApplication::applicationDirPath();
        const QString adaylar[] = {
            QDir(app_dir).filePath("go-winpmem_amd64_1.0-rc2_signed.exe"),
            "go-winpmem_amd64_1.0-rc2_signed.exe",
            "C:/Forensics/go-winpmem_amd64_1.0-rc2_signed.exe",
            "C:/Tools/go-winpmem_amd64_1.0-rc2_signed.exe"
        };

        bool mevcut = false;
        for (const QString& aday : adaylar) {
            if (winpmem_binary_kontrol(winpmem_yerel_edinim_, aday.toUtf8().constData())) {
                bulunan = aday;
                mevcut = true;
                break;
            }
        }

        // Agent ile ayni davranis: yoksa indirip tekrar kontrol et.
        if (!mevcut) {
            QString indirilen;
            QString hata;
            if (winpmem_yerel_dosya_indir(&indirilen, &hata) &&
                winpmem_binary_kontrol(winpmem_yerel_edinim_, indirilen.toUtf8().constData())) {
                bulunan = indirilen;
                mevcut = true;
                log_ekle("WinPMEM otomatik indirildi: " + indirilen, GUNLUK_SEVIYE_INFO);
            } else {
                log_ekle("WinPMEM otomatik indirme basarisiz: " + hata, GUNLUK_SEVIYE_WARN);
            }
        }

        const bool yetki = winpmem_yonetici_yetkisi_kontrol();
        QString bilgi = QString("WinPMEM: %1 | Yetki: %2")
            .arg(mevcut ? "Mevcut" : "Bulunamadi")
            .arg(yetki ? "Yonetici" : "Yetkisiz");
        if (mevcut) {
            bilgi += " | Yol: " + bulunan;
        }

        winpmem_yerel_bilgi_label_->setText(bilgi);
        winpmem_yerel_indir_btn_->setEnabled(!mevcut);
        winpmem_yerel_baslat_btn_->setEnabled(mevcut && yetki);
        winpmem_yerel_durum_label_->setText((mevcut && yetki) ? "Hazir" : "Hazir degil");
        winpmem_yerel_kontrol_btn_->setEnabled(true);
#endif
    }

    void winpmem_yerel_indir() {
#ifndef _WIN32
        QMessageBox::warning(this, "Bilgi", "Bu ozellik sadece Windows'ta calisir.");
#else
        QString indirilen;
        QString hata;
        winpmem_yerel_durum_label_->setText("WinPMEM indiriliyor...");
        if (winpmem_yerel_dosya_indir(&indirilen, &hata)) {
            log_ekle("WinPMEM manuel indirildi: " + indirilen, GUNLUK_SEVIYE_INFO);
            QMessageBox::information(this, "Bilgi", "WinPMEM indirildi:\n" + indirilen);
            winpmem_yerel_kontrol_yap();
        } else {
            log_ekle("WinPMEM manuel indirme basarisiz: " + hata, GUNLUK_SEVIYE_ERROR);
            QMessageBox::critical(this, "Hata", "WinPMEM indirilemedi:\n" + hata);
            winpmem_yerel_durum_label_->setText("WinPMEM indirilemedi");
        }
#endif
    }

    void winpmem_yerel_baslat() {
#ifndef _WIN32
        QMessageBox::warning(this, "Bilgi", "Bu ozellik sadece Windows'ta calisir.");
        return;
#else
        if (!winpmem_yerel_edinim_) {
            QMessageBox::critical(this, "Hata", "Once WinPMEM kontrolu yapin.");
            return;
        }

        if (!vaka_sec_veya_olustur("Windows yerel RAM imaji")) {
            return;
        }

        QString cikti_dosya = winpmem_yerel_cikti_giris_ ? winpmem_yerel_cikti_giris_->text().trimmed() : QString();
        if (cikti_dosya.isEmpty()) {
            cikti_dosya = "memory_dump_local.raw";
        }

        const QString ram_klasor = vaka_cikti_klasoru_hazirla("ram");
        if (ram_klasor.isEmpty()) {
            QMessageBox::critical(this, "Hata", "RAM cikti klasoru olusturulamadi!");
            return;
        }

        QString dosya_adi = QFileInfo(cikti_dosya).fileName();
        if (dosya_adi.isEmpty()) {
            dosya_adi = "memory_dump_local.raw";
        }
        const QString tam_cikti = ram_klasor + "/" + dosya_adi;
        if (winpmem_yerel_cikti_giris_) {
            winpmem_yerel_cikti_giris_->setText(tam_cikti);
        }

        winpmem_yerel_baslat_btn_->setEnabled(false);
        winpmem_yerel_kontrol_btn_->setEnabled(false);
        if (winpmem_yerel_indir_btn_) {
            winpmem_yerel_indir_btn_->setEnabled(false);
        }
        if (winpmem_yerel_ilerleme_) {
            winpmem_yerel_ilerleme_->setValue(0);
        }
        winpmem_yerel_durum_label_->setText("Windows yerel RAM edinimi baslatildi...");
        log_ekle("Windows yerel RAM edinimi baslatildi: " + tam_cikti, GUNLUK_SEVIYE_INFO);

        std::thread([this, tam_cikti]() {
            HataKodu hata = HATA_OK;
            bool ok = winpmem_ram_al(
                winpmem_yerel_edinim_,
                tam_cikti.toUtf8().constData(),
                winpmem_yerel_ilerleme_callback,
                this,
                &hata
            );

            QMetaObject::invokeMethod(this, [this, ok, tam_cikti, hata]() {
                winpmem_yerel_kontrol_btn_->setEnabled(true);
                winpmem_yerel_baslat_btn_->setEnabled(true);
                if (winpmem_yerel_indir_btn_) {
                    winpmem_yerel_indir_btn_->setEnabled(false);
                }

                if (ok) {
                    if (winpmem_yerel_ilerleme_) {
                        winpmem_yerel_ilerleme_->setValue(100);
                    }
                    winpmem_yerel_durum_label_->setText("Windows yerel RAM edinimi tamamlandi");
                    log_ekle("Windows yerel RAM edinimi tamamlandi: " + tam_cikti, GUNLUK_SEVIYE_INFO);
                    QMessageBox::information(this, "Basarili", "Windows yerel RAM imaji alindi:\n" + tam_cikti);
                } else {
                    winpmem_yerel_durum_label_->setText("Windows yerel RAM edinimi basarisiz");
                    log_ekle("Windows yerel RAM edinimi basarisiz", GUNLUK_SEVIYE_ERROR);
                    QMessageBox::critical(this, "Hata", "Windows yerel RAM edinimi basarisiz. Hata kodu: " + QString::number((int)hata));
                }
            }, Qt::QueuedConnection);
        }).detach();
#endif
    }

    static void avml_ilerleme_callback(int64_t okunan, int64_t toplam, void* kullanici_verisi) {
        AnaPencere* pencere = static_cast<AnaPencere*>(kullanici_verisi);
        if (!pencere || toplam <= 0) {
            return;
        }

        int oran = static_cast<int>((okunan * 100) / toplam);
        if (oran >= 100) {
            oran = 99;
        }
        QMetaObject::invokeMethod(pencere, [pencere, oran]() {
            if (pencere->avml_ilerleme_) {
                pencere->avml_ilerleme_->setValue(oran);
            }
            if (pencere->genel_ilerleme_) {
                pencere->genel_ilerleme_->setValue(oran);
            }
        }, Qt::QueuedConnection);
    }

    void avml_kontrol_yap() {
#ifdef _WIN32
        QMessageBox::warning(this, "Bilgi", "Bu ozellik sadece Linux'ta calisir.");
        return;
#else
        if (!avml_edinim_) {
            avml_edinim_ = avml_edinim_olustur();
        }
        if (!avml_edinim_) {
            QMessageBox::critical(this, "Hata", "AVML edinim nesnesi olusturulamadi.");
            return;
        }

        const bool avml_mevcut = avml_binary_kontrol(avml_edinim_, nullptr);
        const bool root = avml_root_yetkisi_kontrol();
        const double gb = avml_ram_boyut_al() / (1024.0 * 1024.0 * 1024.0);

        QString bilgi = QString("AVML: %1 | Yetki: %2 | RAM: %3 GB")
            .arg(avml_mevcut ? "Mevcut" : "Bulunamadi")
            .arg(root ? "root" : "root degil")
            .arg(QString::number(gb, 'f', 2));
        if (avml_mevcut) {
            bilgi += " | Yol: " + QString::fromUtf8(avml_edinim_->avml_yolu);
        }

        if (avml_bilgi_label_) {
            avml_bilgi_label_->setText(bilgi);
        }
        if (avml_baslat_btn_) {
            avml_baslat_btn_->setEnabled(avml_mevcut && root);
        }
        if (avml_durum_label_) {
            avml_durum_label_->setText((avml_mevcut && root) ? "Hazir" : "Hazir degil");
        }
#endif
    }

    void avml_baslat() {
#ifdef _WIN32
        QMessageBox::warning(this, "Bilgi", "Bu ozellik sadece Linux'ta calisir.");
        return;
#else
        if (!avml_edinim_) {
            QMessageBox::critical(this, "Hata", "Once AVML kontrolu yapin.");
            return;
        }

        if (!vaka_sec_veya_olustur("Linux yerel RAM imaji")) {
            return;
        }

        QString cikti = avml_cikti_giris_ ? avml_cikti_giris_->text().trimmed() : QString();
        if (cikti.isEmpty()) {
            cikti = "linux_memory_dump.raw";
        }

        const QString ram_klasor = vaka_cikti_klasoru_hazirla("ram");
        if (ram_klasor.isEmpty()) {
            QMessageBox::critical(this, "Hata", "RAM cikti klasoru olusturulamadi!");
            return;
        }

        QString tam_cikti = ram_klasor + "/" + QFileInfo(cikti).fileName();
        if (avml_cikti_giris_) {
            avml_cikti_giris_->setText(tam_cikti);
        }

        if (avml_baslat_btn_) {
            avml_baslat_btn_->setEnabled(false);
        }
        if (avml_ilerleme_) {
            avml_ilerleme_->setValue(0);
        }
        if (avml_durum_label_) {
            avml_durum_label_->setText("AVML ile Linux RAM edinimi baslatildi...");
        }
        log_ekle("Linux yerel RAM edinimi baslatildi: " + tam_cikti, GUNLUK_SEVIYE_INFO);

        std::thread([this, tam_cikti]() {
            HataKodu hata = HATA_OK;
            bool ok = avml_ram_al(
                avml_edinim_,
                tam_cikti.toUtf8().constData(),
                avml_ilerleme_callback,
                this,
                &hata
            );

            QMetaObject::invokeMethod(this, [this, ok, tam_cikti, hata]() {
                if (avml_baslat_btn_) {
                    avml_baslat_btn_->setEnabled(true);
                }

                if (ok) {
                    if (avml_ilerleme_) {
                        avml_ilerleme_->setValue(100);
                    }
                    if (avml_durum_label_) {
                        avml_durum_label_->setText("Linux yerel RAM edinimi tamamlandi");
                    }
                    log_ekle("Linux yerel RAM edinimi tamamlandi: " + tam_cikti, GUNLUK_SEVIYE_INFO);
                    QMessageBox::information(this, "Basarili", "Linux yerel RAM imaji alindi:\n" + tam_cikti);
                } else {
                    if (avml_durum_label_) {
                        avml_durum_label_->setText("Linux yerel RAM edinimi basarisiz");
                    }
                    log_ekle("Linux yerel RAM edinimi basarisiz", GUNLUK_SEVIYE_ERROR);
                    QMessageBox::critical(this, "Hata", "Linux yerel RAM edinimi basarisiz. Hata kodu: " + QString::number((int)hata));
                }
            }, Qt::QueuedConnection);
        }).detach();
#endif
    }

    void gunluk_dosyadan_yenile() {
        if (!kasa_ || !gunluk_sekme_metin_) {
            return;
        }

        GList* dosyalar = kanit_kasasi_dosyalari_listele(kasa_, "gunlukler");
        if (!dosyalar) {
            gunluk_sekme_metin_->append("[Bilgi] Gunluk dosyasi bulunamadi.");
            return;
        }

        gchar* en_yeni = nullptr;
        for (GList* l = dosyalar; l != nullptr; l = l->next) {
            const char* yol = static_cast<const char*>(l->data);
            if (!en_yeni || g_strcmp0(yol, en_yeni) > 0) {
                g_free(en_yeni);
                en_yeni = g_strdup(yol);
            }
        }

        if (en_yeni) {
            gchar* icerik = nullptr;
            gsize boyut = 0;
            GError* hata = nullptr;
            if (g_file_get_contents(en_yeni, &icerik, &boyut, &hata)) {
                gunluk_sekme_metin_->setPlainText(QString::fromUtf8(icerik));
                g_free(icerik);
                log_ekle("Gunluk dosyasi yenilendi", GUNLUK_SEVIYE_INFO);
            } else {
                gunluk_sekme_metin_->append("[Hata] Gunluk dosyasi okunamadi.");
                if (hata) {
                    g_error_free(hata);
                }
            }
            g_free(en_yeni);
        }

        g_list_free_full(dosyalar, g_free);
    }

    void temizle() {
        if (temizlendi_) {
            return;
        }
        temizlendi_ = true;

        imaj_baglanti_kaldir(true);

        if (baglanti_) {
            uzak_disk_baglanti_kapat(baglanti_);
            baglanti_ = nullptr;
        }

        if (winpmem_edinim_) {
            winpmem_edinim_yok_et(winpmem_edinim_);
            winpmem_edinim_ = nullptr;
        }

        if (winpmem_yerel_edinim_) {
            winpmem_edinim_yok_et(winpmem_yerel_edinim_);
            winpmem_yerel_edinim_ = nullptr;
        }

        if (avml_edinim_) {
            avml_edinim_yok_et(avml_edinim_);
            avml_edinim_ = nullptr;
        }

        if (kasa_) {
            kanit_kasasi_kapat(kasa_);
            kasa_ = nullptr;
        }

        if (is_kuyrugu_) {
            is_kuyrugu_kapat(is_kuyrugu_);
            is_kuyrugu_ = nullptr;
        }

        if (gunluk_) {
            gunluk_kapat(gunluk_);
            gunluk_ = nullptr;
        }

        if (ayarlar_) {
            ayarlar_temizle(ayarlar_);
            ayarlar_ = nullptr;
        }

        if (ayar_dosya_yolu_) {
            g_free(ayar_dosya_yolu_);
            ayar_dosya_yolu_ = nullptr;
        }

        if (vpn_yonetici_) {
            wireguard_yonetici_yok_et(vpn_yonetici_);
            vpn_yonetici_ = nullptr;
        }
    }
};

int main(int argc, char* argv[]) {
    setlocale(LC_ALL, "");

    QApplication uygulama(argc, argv);
    AnaPencere pencere;
    pencere.show();
    return uygulama.exec();
}
