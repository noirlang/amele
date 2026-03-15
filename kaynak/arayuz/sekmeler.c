#include "sekmeler.h"
#include <string.h>
#include <stdlib.h>
#include <sys/utsname.h>
#include "disk_edinim.h"
#include "hash.h"
#include "rapor.h"
#include "kanit_kasasi.h"
#include "uzak_disk_edinim.h"
#include "wireguard.h"

typedef struct {
    ArayuzYonetici* arayuz;
    GtkWidget* disk_secim;
    GtkWidget* hedef_giris;
    GtkWidget* boyut_giris;
    GtkWidget* tam_disk_secim;
    GtkWidget* hash_secim;
    DiskBilgisi* diskler;
    int disk_sayisi;
} GenelSekmeVerisi;

static void disk_listesi_guncelle(GenelSekmeVerisi* veri) {
    if (veri->diskler) {
        free(veri->diskler);
        veri->diskler = NULL;
    }
    veri->disk_sayisi = 0;
    
    disk_listele(&veri->diskler, &veri->disk_sayisi);
    
    gtk_drop_down_set_model(GTK_DROP_DOWN(veri->disk_secim), NULL);
    
    GtkStringList* str_list = gtk_string_list_new(NULL);
    for (int i = 0; i < veri->disk_sayisi; i++) {
        char label[512];
        snprintf(label, sizeof(label), "%s (%.1f GB)", 
                 veri->diskler[i].cihaz,
                 veri->diskler[i].toplam_boyut / (1024.0 * 1024.0 * 1024.0));
        gtk_string_list_append(str_list, label);
    }
    
    gtk_drop_down_set_model(GTK_DROP_DOWN(veri->disk_secim), G_LIST_MODEL(str_list));
}

static void imaj_baslat_tikla(GtkButton* btn, GenelSekmeVerisi* veri) {
    if (!veri->arayuz->kasa) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Once vaka olusturun!");
        return;
    }
    
    guint secili = gtk_drop_down_get_selected(GTK_DROP_DOWN(veri->disk_secim));
    if (secili >= (guint)veri->disk_sayisi) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Disk secilmedi!");
        return;
    }
    
    const char* hedef = gtk_editable_get_text(GTK_EDITABLE(veri->hedef_giris));
    if (!hedef || strlen(hedef) == 0) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Hedef dosya belirtilmedi!");
        return;
    }
    
    arayuz_log_ekle(veri->arayuz, "Disk edinim baslatiliyor...", GUNLUK_SEVIYE_INFO);
    arayuz_durum_guncelle(veri->arayuz, "Disk edinim calisiyor");
    
    DiskEdinimGorevi* gorev = g_new0(DiskEdinimGorevi, 1);
    gorev->is = is_olustur(IS_TIPI_DISK_EDINIM, "Disk Edinim");
    strncpy(gorev->kaynak, veri->diskler[secili].cihaz, sizeof(gorev->kaynak) - 1);
    strncpy(gorev->hedef, hedef, sizeof(gorev->hedef) - 1);
    
    if (gtk_check_button_get_active(GTK_CHECK_BUTTON(veri->tam_disk_secim))) {
        gorev->tam_disk = TRUE;
        gorev->boyut = veri->diskler[secili].toplam_boyut;
    } else {
        gorev->tam_disk = FALSE;
        const char* boyut_str = gtk_editable_get_text(GTK_EDITABLE(veri->boyut_giris));
        gorev->boyut = atoi(boyut_str) * 1024 * 1024;
    }
    
    gorev->hash_hesapla = gtk_check_button_get_active(GTK_CHECK_BUTTON(veri->hash_secim));
    gorev->parca_boyutu = 4 * 1024 * 1024;
    
    is_ekle(veri->arayuz->is_kuyrugu, gorev->is);
    
    arayuz_log_ekle(veri->arayuz, "Edinim gorevi kuyruga eklendi", GUNLUK_SEVIYE_INFO);
}

GtkWidget* sekme_genel_olustur(ArayuzYonetici* arayuz) {
    GtkWidget* kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(kutu, 24);
    gtk_widget_set_margin_end(kutu, 24);
    gtk_widget_set_margin_top(kutu, 24);
    gtk_widget_set_margin_bottom(kutu, 24);
    
    GtkWidget* cerceve = gtk_frame_new("Disk Edinim");
    gtk_box_append(GTK_BOX(kutu), cerceve);
    
    GtkWidget* icerik = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(icerik, 12);
    gtk_widget_set_margin_end(icerik, 12);
    gtk_widget_set_margin_top(icerik, 12);
    gtk_widget_set_margin_bottom(icerik, 12);
    gtk_frame_set_child(GTK_FRAME(cerceve), icerik);
    
    GenelSekmeVerisi* veri = g_new0(GenelSekmeVerisi, 1);
    veri->arayuz = arayuz;
    
    GtkWidget* disk_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), disk_kutu);
    
    GtkWidget* disk_label = gtk_label_new("Kaynak Disk:");
    gtk_box_append(GTK_BOX(disk_kutu), disk_label);
    
    veri->disk_secim = gtk_drop_down_new(NULL, NULL);
    gtk_widget_set_hexpand(veri->disk_secim, TRUE);
    gtk_box_append(GTK_BOX(disk_kutu), veri->disk_secim);
    
    GtkWidget* yenile_btn = gtk_button_new_with_label("Yenile");
    gtk_box_append(GTK_BOX(disk_kutu), yenile_btn);
    
    GtkWidget* hedef_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), hedef_kutu);
    
    GtkWidget* hedef_label = gtk_label_new("Hedef Dosya:");
    gtk_box_append(GTK_BOX(hedef_kutu), hedef_label);
    
    veri->hedef_giris = gtk_entry_new();
    gtk_widget_set_hexpand(veri->hedef_giris, TRUE);
    gtk_box_append(GTK_BOX(hedef_kutu), veri->hedef_giris);
    
    GtkWidget* boyut_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), boyut_kutu);
    
    GtkWidget* boyut_label = gtk_label_new("Boyut (MB):");
    gtk_box_append(GTK_BOX(boyut_kutu), boyut_label);
    
    veri->boyut_giris = gtk_entry_new();
    gtk_editable_set_text(GTK_EDITABLE(veri->boyut_giris), "100");
    gtk_box_append(GTK_BOX(boyut_kutu), veri->boyut_giris);
    
    veri->tam_disk_secim = gtk_check_button_new_with_label("Tam Diski Kopyala");
    gtk_box_append(GTK_BOX(icerik), veri->tam_disk_secim);
    
    veri->hash_secim = gtk_check_button_new_with_label("Hash Hesapla (SHA-256)");
    gtk_check_button_set_active(GTK_CHECK_BUTTON(veri->hash_secim), TRUE);
    gtk_box_append(GTK_BOX(icerik), veri->hash_secim);
    
    GtkWidget* btn_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), btn_kutu);
    
    GtkWidget* baslat_btn = gtk_button_new_with_label("Imaji Baslat");
    gtk_widget_add_css_class(baslat_btn, "suggested-action");
    gtk_box_append(GTK_BOX(btn_kutu), baslat_btn);
    
    GtkWidget* dogrula_btn = gtk_button_new_with_label("Imaji Dogrula");
    gtk_box_append(GTK_BOX(btn_kutu), dogrula_btn);
    
    g_signal_connect(yenile_btn, "clicked", G_CALLBACK(disk_listesi_guncelle), veri);
    g_signal_connect(baslat_btn, "clicked", G_CALLBACK(imaj_baslat_tikla), veri);
    
    disk_listesi_guncelle(veri);
    
    return kutu;
}

GtkWidget* sekme_sistem_bilgisi_olustur(ArayuzYonetici* arayuz) {
    GtkWidget* kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(kutu, 24);
    gtk_widget_set_margin_end(kutu, 24);
    gtk_widget_set_margin_top(kutu, 24);
    gtk_widget_set_margin_bottom(kutu, 24);
    
    GtkWidget* cerceve = gtk_frame_new("Sistem Bilgisi");
    gtk_box_append(GTK_BOX(kutu), cerceve);
    
    GtkWidget* icerik = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    gtk_widget_set_margin_start(icerik, 12);
    gtk_widget_set_margin_end(icerik, 12);
    gtk_widget_set_margin_top(icerik, 12);
    gtk_widget_set_margin_bottom(icerik, 12);
    gtk_frame_set_child(GTK_FRAME(cerceve), icerik);
    
    GtkWidget* metin = gtk_text_view_new();
    gtk_text_view_set_editable(GTK_TEXT_VIEW(metin), FALSE);
    gtk_text_view_set_cursor_visible(GTK_TEXT_VIEW(metin), FALSE);
    gtk_text_view_set_monospace(GTK_TEXT_VIEW(metin), TRUE);
    gtk_widget_set_size_request(metin, -1, 400);
    gtk_box_append(GTK_BOX(icerik), metin);
    
    GtkTextBuffer* buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(metin));
    
    struct utsname uts;
    if (uname(&uts) == 0) {
        char* bilgi = g_strdup_printf(
            "Isletim Sistemi: %s\n"
            "Surum: %s\n"
            "Makine: %s\n"
            "Hostname: %s\n"
            "\n"
            "Kernel: %s\n"
            "Kullanici: %s\n",
            uts.sysname, uts.release, uts.machine, uts.nodename,
            uts.version,
            getlogin() ? getlogin() : "bilinmiyor"
        );
        gtk_text_buffer_set_text(buffer, bilgi, -1);
        g_free(bilgi);
    } else {
        gtk_text_buffer_set_text(buffer, "Sistem bilgisi alinamadi.", -1);
    }
    
    GtkWidget* kaydet_btn = gtk_button_new_with_label("Bilgiyi Kaydet");
    gtk_box_append(GTK_BOX(icerik), kaydet_btn);
    
    return kutu;
}

typedef struct {
    ArayuzYonetici* arayuz;
    GtkWidget* dosya_secim;
    GtkWidget* md5_label;
    GtkWidget* sha1_label;
    GtkWidget* sha256_label;
    GtkWidget* sha512_label;
    GtkWidget* karsilastir_giris;
    GtkWidget* sonuc_label;
} HashSekmeVerisi;

static void hash_hesapla_tikla(GtkButton* btn, HashSekmeVerisi* veri) {
    const char* dosya = gtk_editable_get_text(GTK_EDITABLE(veri->dosya_secim));
    if (!dosya || strlen(dosya) == 0) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Dosya secilmedi!");
        return;
    }
    
    arayuz_log_ekle(veri->arayuz, "Hash hesaplaniyor...", GUNLUK_SEVIYE_INFO);
    
    char hash_str[129];
    
    if (hash_dosya_hesapla(dosya, HASH_MD5, hash_str, sizeof(hash_str))) {
        gtk_label_set_text(GTK_LABEL(veri->md5_label), hash_str);
    }
    if (hash_dosya_hesapla(dosya, HASH_SHA1, hash_str, sizeof(hash_str))) {
        gtk_label_set_text(GTK_LABEL(veri->sha1_label), hash_str);
    }
    if (hash_dosya_hesapla(dosya, HASH_SHA256, hash_str, sizeof(hash_str))) {
        gtk_label_set_text(GTK_LABEL(veri->sha256_label), hash_str);
    }
    if (hash_dosya_hesapla(dosya, HASH_SHA512, hash_str, sizeof(hash_str))) {
        gtk_label_set_text(GTK_LABEL(veri->sha512_label), hash_str);
    }
    
    arayuz_log_ekle(veri->arayuz, "Hash hesaplama tamamlandi", GUNLUK_SEVIYE_INFO);
}

static void hash_karsilastir_tikla(GtkButton* btn, HashSekmeVerisi* veri) {
    const char* beklenen = gtk_editable_get_text(GTK_EDITABLE(veri->karsilastir_giris));
    if (!beklenen || strlen(beklenen) == 0) {
        gtk_label_set_text(GTK_LABEL(veri->sonuc_label), "Hash girin!");
        return;
    }
    
    const char* md5 = gtk_label_get_text(GTK_LABEL(veri->md5_label));
    const char* sha1 = gtk_label_get_text(GTK_LABEL(veri->sha1_label));
    const char* sha256 = gtk_label_get_text(GTK_LABEL(veri->sha256_label));
    const char* sha512 = gtk_label_get_text(GTK_LABEL(veri->sha512_label));
    
    if (strcasecmp(beklenen, md5) == 0) {
        gtk_label_set_text(GTK_LABEL(veri->sonuc_label), "MD5 eslesti!");
    } else if (strcasecmp(beklenen, sha1) == 0) {
        gtk_label_set_text(GTK_LABEL(veri->sonuc_label), "SHA1 eslesti!");
    } else if (strcasecmp(beklenen, sha256) == 0) {
        gtk_label_set_text(GTK_LABEL(veri->sonuc_label), "SHA256 eslesti!");
    } else if (strcasecmp(beklenen, sha512) == 0) {
        gtk_label_set_text(GTK_LABEL(veri->sonuc_label), "SHA512 eslesti!");
    } else {
        gtk_label_set_text(GTK_LABEL(veri->sonuc_label), "Eslesme bulunamadi!");
    }
}

GtkWidget* sekme_hash_islemleri_olustur(ArayuzYonetici* arayuz) {
    GtkWidget* kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(kutu, 24);
    gtk_widget_set_margin_end(kutu, 24);
    gtk_widget_set_margin_top(kutu, 24);
    gtk_widget_set_margin_bottom(kutu, 24);
    
    GtkWidget* cerceve = gtk_frame_new("Hash Hesaplayici");
    gtk_box_append(GTK_BOX(kutu), cerceve);
    
    GtkWidget* icerik = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(icerik, 12);
    gtk_widget_set_margin_end(icerik, 12);
    gtk_widget_set_margin_top(icerik, 12);
    gtk_widget_set_margin_bottom(icerik, 12);
    gtk_frame_set_child(GTK_FRAME(cerceve), icerik);
    
    HashSekmeVerisi* veri = g_new0(HashSekmeVerisi, 1);
    veri->arayuz = arayuz;
    
    GtkWidget* dosya_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), dosya_kutu);
    
    GtkWidget* dosya_label = gtk_label_new("Dosya:");
    gtk_box_append(GTK_BOX(dosya_kutu), dosya_label);
    
    veri->dosya_secim = gtk_entry_new();
    gtk_widget_set_hexpand(veri->dosya_secim, TRUE);
    gtk_box_append(GTK_BOX(dosya_kutu), veri->dosya_secim);
    
    GtkWidget* hesapla_btn = gtk_button_new_with_label("Hesapla");
    gtk_widget_add_css_class(hesapla_btn, "suggested-action");
    gtk_box_append(GTK_BOX(icerik), hesapla_btn);
    
    GtkWidget* sonuclar_kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    gtk_box_append(GTK_BOX(icerik), sonuclar_kutu);
    
    veri->md5_label = gtk_label_new("MD5: -");
    gtk_label_set_selectable(GTK_LABEL(veri->md5_label), TRUE);
    gtk_box_append(GTK_BOX(sonuclar_kutu), veri->md5_label);
    
    veri->sha1_label = gtk_label_new("SHA1: -");
    gtk_label_set_selectable(GTK_LABEL(veri->sha1_label), TRUE);
    gtk_box_append(GTK_BOX(sonuclar_kutu), veri->sha1_label);
    
    veri->sha256_label = gtk_label_new("SHA256: -");
    gtk_label_set_selectable(GTK_LABEL(veri->sha256_label), TRUE);
    gtk_box_append(GTK_BOX(sonuclar_kutu), veri->sha256_label);
    
    veri->sha512_label = gtk_label_new("SHA512: -");
    gtk_label_set_selectable(GTK_LABEL(veri->sha512_label), TRUE);
    gtk_box_append(GTK_BOX(sonuclar_kutu), veri->sha512_label);
    
    GtkWidget* karsilastir_cerceve = gtk_frame_new("Hash Karsilastir");
    gtk_box_append(GTK_BOX(icerik), karsilastir_cerceve);
    
    GtkWidget* karsilastir_icerik = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    gtk_widget_set_margin_start(karsilastir_icerik, 12);
    gtk_widget_set_margin_end(karsilastir_icerik, 12);
    gtk_widget_set_margin_top(karsilastir_icerik, 12);
    gtk_widget_set_margin_bottom(karsilastir_icerik, 12);
    gtk_frame_set_child(GTK_FRAME(karsilastir_cerceve), karsilastir_icerik);
    
    veri->karsilastir_giris = gtk_entry_new();
    gtk_entry_set_placeholder_text(GTK_ENTRY(veri->karsilastir_giris), "Hash degeri girin");
    gtk_box_append(GTK_BOX(karsilastir_icerik), veri->karsilastir_giris);
    
    GtkWidget* karsilastir_btn = gtk_button_new_with_label("Karsilastir");
    gtk_box_append(GTK_BOX(karsilastir_icerik), karsilastir_btn);
    
    veri->sonuc_label = gtk_label_new("");
    gtk_box_append(GTK_BOX(karsilastir_icerik), veri->sonuc_label);
    
    g_signal_connect(hesapla_btn, "clicked", G_CALLBACK(hash_hesapla_tikla), veri);
    g_signal_connect(karsilastir_btn, "clicked", G_CALLBACK(hash_karsilastir_tikla), veri);
    
    return kutu;
}

typedef struct {
    ArayuzYonetici* arayuz;
    GtkWidget* vaka_giris;
    GtkWidget* durum_label;
    GtkWidget* dosya_listesi;
} KanitKasasiVerisi;

static void vaka_olustur_tikla(GtkButton* btn, KanitKasasiVerisi* veri) {
    const char* vaka_adi = gtk_editable_get_text(GTK_EDITABLE(veri->vaka_giris));
    if (!vaka_adi || strlen(vaka_adi) == 0) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Vaka adi girin!");
        return;
    }
    
    if (veri->arayuz->kasa) {
        kanit_kasasi_kapat(veri->arayuz->kasa);
    }
    
    const char* home = g_get_home_dir();
    char* ana_klasor = g_build_filename(home, "Worm", NULL);
    
    veri->arayuz->kasa = kanit_kasasi_olustur(ana_klasor, vaka_adi);
    g_free(ana_klasor);
    
    if (veri->arayuz->kasa) {
        gtk_label_set_text(GTK_LABEL(veri->durum_label), "Vaka olusturuldu");
        arayuz_log_ekle(veri->arayuz, "Yeni vaka olusturuldu", GUNLUK_SEVIYE_INFO);
    } else {
        gtk_label_set_text(GTK_LABEL(veri->durum_label), "Vaka olusturulamadi!");
    }
}

static void dosyalari_listele_tikla(GtkButton* btn, KanitKasasiVerisi* veri) {
    if (!veri->arayuz->kasa) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Once vaka olusturun!");
        return;
    }
    
    GListModel* list_model = gtk_drop_down_get_model(GTK_DROP_DOWN(veri->dosya_listesi));
    while (g_list_model_get_n_items(list_model) > 0) {
        gtk_string_list_remove(GTK_STRING_LIST(list_model), 0);
    }
    
    GList* dosyalar = kanit_kasasi_dosyalari_listele(veri->arayuz->kasa, "ciktilar");
    for (GList* l = dosyalar; l; l = l->next) {
        char* isim = g_path_get_basename((char*)l->data);
        gtk_string_list_append(GTK_STRING_LIST(list_model), isim);
        g_free(isim);
    }
    g_list_free_full(dosyalar, g_free);
    
    arayuz_log_ekle(veri->arayuz, "Dosyalar listelendi", GUNLUK_SEVIYE_INFO);
}

GtkWidget* sekme_kanit_kasasi_olustur(ArayuzYonetici* arayuz) {
    GtkWidget* kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(kutu, 24);
    gtk_widget_set_margin_end(kutu, 24);
    gtk_widget_set_margin_top(kutu, 24);
    gtk_widget_set_margin_bottom(kutu, 24);
    
    GtkWidget* cerceve = gtk_frame_new("Vaka Yonetimi");
    gtk_box_append(GTK_BOX(kutu), cerceve);
    
    GtkWidget* icerik = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(icerik, 12);
    gtk_widget_set_margin_end(icerik, 12);
    gtk_widget_set_margin_top(icerik, 12);
    gtk_widget_set_margin_bottom(icerik, 12);
    gtk_frame_set_child(GTK_FRAME(cerceve), icerik);
    
    KanitKasasiVerisi* veri = g_new0(KanitKasasiVerisi, 1);
    veri->arayuz = arayuz;
    
    GtkWidget* vaka_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), vaka_kutu);
    
    GtkWidget* vaka_label = gtk_label_new("Vaka Adi:");
    gtk_box_append(GTK_BOX(vaka_kutu), vaka_label);
    
    veri->vaka_giris = gtk_entry_new();
    gtk_widget_set_hexpand(veri->vaka_giris, TRUE);
    gtk_box_append(GTK_BOX(vaka_kutu), veri->vaka_giris);
    
    GtkWidget* olustur_btn = gtk_button_new_with_label("Vaka Olustur");
    gtk_widget_add_css_class(olustur_btn, "suggested-action");
    gtk_box_append(GTK_BOX(icerik), olustur_btn);
    
    veri->durum_label = gtk_label_new("Vaka olusturulmadi");
    gtk_box_append(GTK_BOX(icerik), veri->durum_label);
    
    GtkWidget* dosyalar_cerceve = gtk_frame_new("Dosyalar");
    gtk_box_append(GTK_BOX(icerik), dosyalar_cerceve);
    
    GtkWidget* dosyalar_icerik = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    gtk_widget_set_margin_start(dosyalar_icerik, 12);
    gtk_widget_set_margin_end(dosyalar_icerik, 12);
    gtk_widget_set_margin_top(dosyalar_icerik, 12);
    gtk_widget_set_margin_bottom(dosyalar_icerik, 12);
    gtk_frame_set_child(GTK_FRAME(dosyalar_cerceve), dosyalar_icerik);
    
    veri->dosya_listesi = gtk_drop_down_new(G_LIST_MODEL(gtk_string_list_new(NULL)), NULL);
    gtk_box_append(GTK_BOX(dosyalar_icerik), veri->dosya_listesi);
    
    GtkWidget* listele_btn = gtk_button_new_with_label("Dosyalari Listele");
    gtk_box_append(GTK_BOX(dosyalar_icerik), listele_btn);
    
    g_signal_connect(olustur_btn, "clicked", G_CALLBACK(vaka_olustur_tikla), veri);
    g_signal_connect(listele_btn, "clicked", G_CALLBACK(dosyalari_listele_tikla), veri);
    
    return kutu;
}

GtkWidget* sekme_raporlar_olustur(ArayuzYonetici* arayuz) {
    GtkWidget* kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(kutu, 24);
    gtk_widget_set_margin_end(kutu, 24);
    gtk_widget_set_margin_top(kutu, 24);
    gtk_widget_set_margin_bottom(kutu, 24);
    
    GtkWidget* cerceve = gtk_frame_new("Rapor Olustur");
    gtk_box_append(GTK_BOX(kutu), cerceve);
    
    GtkWidget* icerik = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(icerik, 12);
    gtk_widget_set_margin_end(icerik, 12);
    gtk_widget_set_margin_top(icerik, 12);
    gtk_widget_set_margin_bottom(icerik, 12);
    gtk_frame_set_child(GTK_FRAME(cerceve), icerik);
    
    GtkWidget* bilgi_label = gtk_label_new("Rapor olusturmak icin once vaka olusturun ve islem tamamlayin.");
    gtk_box_append(GTK_BOX(icerik), bilgi_label);
    
    GtkWidget* not_btn = gtk_button_new_with_label("Not Ekle");
    gtk_box_append(GTK_BOX(icerik), not_btn);
    
    GtkWidget* rapor_btn = gtk_button_new_with_label("Rapor Olustur");
    gtk_widget_add_css_class(rapor_btn, "suggested-action");
    gtk_box_append(GTK_BOX(icerik), rapor_btn);
    
    return kutu;
}

typedef struct {
    ArayuzYonetici* arayuz;
    GtkWidget* ip_giris;
    GtkWidget* port_giris;
    GtkWidget* token_giris;
    GtkWidget* baglan_btn;
    GtkWidget* disk_getir_btn;
    GtkWidget* imaj_btn;
    GtkWidget* disk_secim;
    GtkWidget* cikti_klasor;
    GtkWidget* ilerleme;
    GtkWidget* durum_label;
    // WireGuard VPN
    GtkWidget* vpn_check;
    GtkWidget* vpn_config_btn;
    WireGuardYonetici* vpn_yonetici;
    UzakDiskBaglanti* baglanti;
    GList* diskler;
} UzakDiskVerisi;

static void uzak_baglan_tikla(GtkButton* btn, UzakDiskVerisi* veri) {
    const char* ip = gtk_editable_get_text(GTK_EDITABLE(veri->ip_giris));
    const char* port_str = gtk_editable_get_text(GTK_EDITABLE(veri->port_giris));
    const char* token = gtk_editable_get_text(GTK_EDITABLE(veri->token_giris));
    
    if (!ip || strlen(ip) == 0 || !port_str || strlen(port_str) == 0) {
        arayuz_hata_goster(veri->arayuz, "Hata", "IP ve port girin!");
        return;
    }
    
    int port = atoi(port_str);
    if (port <= 0 || port > 65535) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Gecersiz port!");
        return;
    }
    
    if (veri->baglanti) {
        uzak_disk_baglanti_kapat(veri->baglanti);
    }
    
    veri->baglanti = uzak_disk_baglanti_olustur(ip, port, token);
    if (!veri->baglanti) {
        gtk_label_set_text(GTK_LABEL(veri->durum_label), "Baglanti olusturulamadi");
        return;
    }
    
    gtk_label_set_text(GTK_LABEL(veri->durum_label), "Baglaniyor...");
    
    if (uzak_disk_baglan(veri->baglanti)) {
        gtk_label_set_text(GTK_LABEL(veri->durum_label), "Baglandi");
        arayuz_log_ekle(veri->arayuz, "Uzak sunucuya baglandi", GUNLUK_SEVIYE_INFO);
    } else {
        gtk_label_set_text(GTK_LABEL(veri->durum_label), "Baglanti basarisiz");
        arayuz_hata_goster(veri->arayuz, "Hata", "Sunucuya baglanilamadi!");
    }
}

static void uzak_disk_getir_tikla(GtkButton* btn, UzakDiskVerisi* veri) {
    if (!veri->baglanti) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Once baglanin!");
        return;
    }
    
    g_list_free_full(veri->diskler, g_free);
    veri->diskler = NULL;
    
    GtkStringList* str_list = GTK_STRING_LIST(gtk_drop_down_get_model(GTK_DROP_DOWN(veri->disk_secim)));
    while (g_list_model_get_n_items(G_LIST_MODEL(str_list)) > 0) {
        gtk_string_list_remove(str_list, 0);
    }
    
    veri->diskler = uzak_disk_listele(veri->baglanti);
    if (!veri->diskler) {
        gtk_label_set_text(GTK_LABEL(veri->durum_label), "Diskler alinamadi");
        return;
    }
    
    for (GList* l = veri->diskler; l; l = l->next) {
        UzakDisk* disk = (UzakDisk*)l->data;
        char label[512];
        snprintf(label, sizeof(label), "%s (%s, %.1f GB)", 
                 disk->id, disk->ad, disk->boyut / (1024.0 * 1024.0 * 1024.0));
        gtk_string_list_append(str_list, label);
    }
    
    gtk_label_set_text(GTK_LABEL(veri->durum_label), "Diskler listelendi");
    arayuz_log_ekle(veri->arayuz, "Uzak diskler listelendi", GUNLUK_SEVIYE_INFO);
}

static void uzak_imaj_baslat_tikla(GtkButton* btn, UzakDiskVerisi* veri) {
    if (!veri->baglanti || !veri->arayuz->kasa) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Once baglanin ve vaka olusturun!");
        return;
    }
    
    guint secili = gtk_drop_down_get_selected(GTK_DROP_DOWN(veri->disk_secim));
    if (secili >= g_list_length(veri->diskler)) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Disk secilmedi!");
        return;
    }
    
    UzakDisk* disk = (UzakDisk*)g_list_nth_data(veri->diskler, secili);
    if (!disk) return;
    
    const char* cikti = gtk_editable_get_text(GTK_EDITABLE(veri->cikti_klasor));
    if (!cikti || strlen(cikti) == 0) {
        arayuz_hata_goster(veri->arayuz, "Hata", "Cikti klasoru secilmedi!");
        return;
    }
    
    gtk_label_set_text(GTK_LABEL(veri->durum_label), "Imaj aliniyor...");
    
    if (uzak_imaj_baslat(veri->baglanti, disk->id, cikti, veri->arayuz->kasa->vaka_adi)) {
        arayuz_log_ekle(veri->arayuz, "Uzak imaj baslatildi", GUNLUK_SEVIYE_INFO);
        
        IsGorevi* is = is_olustur(IS_TIPI_DISK_EDINIM, "Uzak Disk Imaji");
        is_ekle(veri->arayuz->is_kuyrugu, is);
        
        gtk_label_set_text(GTK_LABEL(veri->durum_label), "Imaj alma basladi");
    } else {
        gtk_label_set_text(GTK_LABEL(veri->durum_label), "Imaj baslatilamadi");
        arayuz_hata_goster(veri->arayuz, "Hata", "Imaj alma baslatilamadi!");
    }
}

GtkWidget* sekme_uzak_disk_edinim_olustur(ArayuzYonetici* arayuz) {
    GtkWidget* kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(kutu, 24);
    gtk_widget_set_margin_end(kutu, 24);
    gtk_widget_set_margin_top(kutu, 24);
    gtk_widget_set_margin_bottom(kutu, 24);
    
    GtkWidget* cerceve = gtk_frame_new("Uzak Windows Sunucu Baglantisi");
    gtk_box_append(GTK_BOX(kutu), cerceve);
    
    GtkWidget* icerik = gtk_box_new(GTK_ORIENTATION_VERTICAL, 12);
    gtk_widget_set_margin_start(icerik, 12);
    gtk_widget_set_margin_end(icerik, 12);
    gtk_widget_set_margin_top(icerik, 12);
    gtk_widget_set_margin_bottom(icerik, 12);
    gtk_frame_set_child(GTK_FRAME(cerceve), icerik);
    
    UzakDiskVerisi* veri = g_new0(UzakDiskVerisi, 1);
    veri->arayuz = arayuz;
    
    GtkWidget* ip_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), ip_kutu);
    
    GtkWidget* ip_label = gtk_label_new("IP Adresi:");
    gtk_box_append(GTK_BOX(ip_kutu), ip_label);
    
    veri->ip_giris = gtk_entry_new();
    gtk_entry_set_placeholder_text(GTK_ENTRY(veri->ip_giris), "192.168.1.100");
    gtk_widget_set_hexpand(veri->ip_giris, TRUE);
    gtk_box_append(GTK_BOX(ip_kutu), veri->ip_giris);
    
    GtkWidget* port_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), port_kutu);
    
    GtkWidget* port_label = gtk_label_new("Port:");
    gtk_box_append(GTK_BOX(port_kutu), port_label);
    
    veri->port_giris = gtk_entry_new();
    gtk_editable_set_text(GTK_EDITABLE(veri->port_giris), "4444");
    gtk_widget_set_hexpand(veri->port_giris, TRUE);
    gtk_box_append(GTK_BOX(port_kutu), veri->port_giris);
    
    GtkWidget* token_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), token_kutu);
    
    GtkWidget* token_label = gtk_label_new("Token (opsiyonel):");
    gtk_box_append(GTK_BOX(token_kutu), token_label);
    
    veri->token_giris = gtk_entry_new();
    gtk_entry_set_placeholder_text(GTK_ENTRY(veri->token_giris), "Güvenlik anahtari");
    gtk_widget_set_hexpand(veri->token_giris, TRUE);
    gtk_box_append(GTK_BOX(token_kutu), veri->token_giris);
    
    // WireGuard VPN seçimi
    GtkWidget* vpn_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), vpn_kutu);
    
    veri->vpn_check = gtk_check_button_new_with_label("WireGuard VPN Kullan");
    gtk_box_append(GTK_BOX(vpn_kutu), veri->vpn_check);
    
    veri->vpn_config_btn = gtk_button_new_with_label("VPN Yapılandır");
    gtk_box_append(GTK_BOX(vpn_kutu), veri->vpn_config_btn);
    
    veri->baglan_btn = gtk_button_new_with_label("Baglan");
    gtk_widget_add_css_class(veri->baglan_btn, "suggested-action");
    gtk_box_append(GTK_BOX(icerik), veri->baglan_btn);
    
    veri->disk_getir_btn = gtk_button_new_with_label("Diskleri Getir");
    gtk_box_append(GTK_BOX(icerik), veri->disk_getir_btn);
    
    GtkWidget* disk_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), disk_kutu);
    
    GtkWidget* disk_label = gtk_label_new("Disk:");
    gtk_box_append(GTK_BOX(disk_kutu), disk_label);
    
    veri->disk_secim = gtk_drop_down_new(G_LIST_MODEL(gtk_string_list_new(NULL)), NULL);
    gtk_widget_set_hexpand(veri->disk_secim, TRUE);
    gtk_box_append(GTK_BOX(disk_kutu), veri->disk_secim);
    
    GtkWidget* cikti_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 6);
    gtk_box_append(GTK_BOX(icerik), cikti_kutu);
    
    GtkWidget* cikti_label = gtk_label_new("Cikti Klasoru:");
    gtk_box_append(GTK_BOX(cikti_kutu), cikti_label);
    
    veri->cikti_klasor = gtk_entry_new();
    const char* home = g_get_home_dir();
    char* varsayilan = g_strdup_printf("%s/Worm/Ciktilar", home);
    gtk_editable_set_text(GTK_EDITABLE(veri->cikti_klasor), varsayilan);
    g_free(varsayilan);
    gtk_widget_set_hexpand(veri->cikti_klasor, TRUE);
    gtk_box_append(GTK_BOX(cikti_kutu), veri->cikti_klasor);
    
    veri->imaj_btn = gtk_button_new_with_label("Imaj Al");
    gtk_widget_add_css_class(veri->imaj_btn, "suggested-action");
    gtk_box_append(GTK_BOX(icerik), veri->imaj_btn);
    
    veri->ilerleme = gtk_progress_bar_new();
    gtk_box_append(GTK_BOX(icerik), veri->ilerleme);
    
    veri->durum_label = gtk_label_new("Baglanti yok");
    gtk_box_append(GTK_BOX(icerik), veri->durum_label);
    
    g_signal_connect(veri->baglan_btn, "clicked", G_CALLBACK(uzak_baglan_tikla), veri);
    g_signal_connect(veri->disk_getir_btn, "clicked", G_CALLBACK(uzak_disk_getir_tikla), veri);
    g_signal_connect(veri->imaj_btn, "clicked", G_CALLBACK(uzak_imaj_baslat_tikla), veri);
    
    return kutu;
}

GtkWidget* sekme_gunluk_olustur(ArayuzYonetici* arayuz) {
    return gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
}

void sekmeler_olustur(ArayuzYonetici* arayuz) {
    if (!arayuz || !arayuz->sekmeler) return;
    
    GtkWidget* sekme_uzak = sekme_uzak_disk_edinim_olustur(arayuz);
    gtk_notebook_append_page(arayuz->sekmeler, sekme_uzak, gtk_label_new("Uzak Disk"));
    
    GtkWidget* sekme_sistem = sekme_sistem_bilgisi_olustur(arayuz);
    gtk_notebook_append_page(arayuz->sekmeler, sekme_sistem, gtk_label_new("Sistem Bilgisi"));
    
    GtkWidget* sekme_hash = sekme_hash_islemleri_olustur(arayuz);
    gtk_notebook_append_page(arayuz->sekmeler, sekme_hash, gtk_label_new("Hash Islemleri"));
    
    GtkWidget* sekme_kanit = sekme_kanit_kasasi_olustur(arayuz);
    gtk_notebook_append_page(arayuz->sekmeler, sekme_kanit, gtk_label_new("Kanıt Kasasi"));
    
    GtkWidget* sekme_rapor = sekme_raporlar_olustur(arayuz);
    gtk_notebook_append_page(arayuz->sekmeler, sekme_rapor, gtk_label_new("Raporlar"));
    
    GtkWidget* sekme_gunluk = sekme_gunluk_olustur(arayuz);
    gtk_notebook_append_page(arayuz->sekmeler, sekme_gunluk, gtk_label_new("Gunluk"));
}
