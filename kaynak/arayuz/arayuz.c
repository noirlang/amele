#include "arayuz.h"
#include <string.h>
#include <stdlib.h>

typedef struct {
    GtkTextBuffer* buffer;
    char* mesaj;
} LogEkleVerisi;

static gboolean log_ekle_cb(gpointer veri) {
    LogEkleVerisi* log_veri = (LogEkleVerisi*)veri;
    if (!log_veri || !log_veri->buffer || !log_veri->mesaj) {
        g_free(log_veri->mesaj);
        g_free(log_veri);
        return G_SOURCE_REMOVE;
    }
    
    GtkTextIter iter;
    gtk_text_buffer_get_end_iter(log_veri->buffer, &iter);
    gtk_text_buffer_insert(log_veri->buffer, &iter, log_veri->mesaj, -1);
    gtk_text_buffer_insert(log_veri->buffer, &iter, "\n", -1);
    
    GtkTextMark* mark = gtk_text_buffer_get_mark(log_veri->buffer, "insert");
    if (mark) {
        gtk_text_view_scroll_to_mark(GTK_TEXT_VIEW(g_object_get_data(G_OBJECT(log_veri->buffer), "text_view")),
                                      mark, 0.0, FALSE, 0.0, 0.0);
    }
    
    g_free(log_veri->mesaj);
    g_free(log_veri);
    return G_SOURCE_REMOVE;
}

void arayuz_log_ekle(ArayuzYonetici* arayuz, const char* mesaj, GunlukSeviye seviye) {
    if (!arayuz || !mesaj) return;
    
    g_mutex_lock(&arayuz->kilit);
    
    char zaman[32];
    time_t now = time(NULL);
    strftime(zaman, sizeof(zaman), "%H:%M:%S", localtime(&now));
    
    LogEkleVerisi* log_veri = g_new0(LogEkleVerisi, 1);
    log_veri->buffer = arayuz->log_buffer;
    log_veri->mesaj = g_strdup_printf("[%s] %s", zaman, mesaj);
    
    g_idle_add(log_ekle_cb, log_veri);
    
    if (arayuz->gunluk) {
        gunluk_yaz(arayuz->gunluk, seviye, "%s", mesaj);
    }
    
    g_mutex_unlock(&arayuz->kilit);
}

void arayuz_ilerleme_guncelle(ArayuzYonetici* arayuz, double oran) {
    if (!arayuz) return;
    gtk_progress_bar_set_fraction(arayuz->ilerleme, oran / 100.0);
}

void arayuz_durum_guncelle(ArayuzYonetici* arayuz, const char* mesaj) {
    if (!arayuz || !mesaj) return;
    gtk_label_set_text(arayuz->durum_label, mesaj);
}

void arayuz_hata_goster(ArayuzYonetici* arayuz, const char* baslik, const char* mesaj) {
    if (!arayuz || !mesaj) return;
    
    // Modern GTK4 AlertDialog kullan
    GtkAlertDialog* dialog = gtk_alert_dialog_new("%s", mesaj);
    gtk_alert_dialog_set_modal(dialog, TRUE);
    gtk_alert_dialog_set_detail(dialog, baslik ? baslik : "Hata");
    
    GtkWindow* parent = GTK_WINDOW(arayuz->pencere);
    gtk_alert_dialog_show(dialog, parent);
}

static void pencere_kapat(GtkWindow* window, ArayuzYonetici* arayuz) {
    arayuz->calisiyor = FALSE;
    
    if (arayuz->is_kuyrugu) {
        is_kuyrugu_kapat(arayuz->is_kuyrugu);
        arayuz->is_kuyrugu = NULL;
    }
    
    if (arayuz->kasa) {
        kanit_kasasi_kapat(arayuz->kasa);
        arayuz->kasa = NULL;
    }
    
    if (arayuz->gunluk) {
        gunluk_kapat(arayuz->gunluk);
        arayuz->gunluk = NULL;
    }
    
    if (arayuz->ayarlar) {
        ayarlar_temizle(arayuz->ayarlar);
        arayuz->ayarlar = NULL;
    }
    
    g_mutex_clear(&arayuz->kilit);
    g_free(arayuz);
    
    gtk_window_destroy(window);
}

ArayuzYonetici* arayuz_olustur(GtkApplication* app) {
    ArayuzYonetici* arayuz = g_new0(ArayuzYonetici, 1);
    if (!arayuz) return NULL;
    
    g_mutex_init(&arayuz->kilit);
    arayuz->calisiyor = TRUE;
    arayuz->uygulama = app;
    
    arayuz->ayarlar = ayarlar_yukle(NULL);
    if (!arayuz->ayarlar) {
        arayuz->ayarlar = g_new0(UygulamaAyarlar, 1);
        ayarlar_varsayilan(arayuz->ayarlar);
    }
    
    const char* home = g_get_home_dir();
    char* gunluk_klasor = g_build_filename(home, "Worm", "gunlukler", NULL);
    arayuz->gunluk = gunluk_baslat("Genel", gunluk_klasor);
    g_free(gunluk_klasor);
    
    arayuz->is_kuyrugu = is_kuyrugu_olustur(arayuz->gunluk);
    
    arayuz->pencere = GTK_WINDOW(gtk_application_window_new(app));
    gtk_window_set_title(arayuz->pencere, "Worm Forensic Tool");
    gtk_window_set_default_size(arayuz->pencere, 1100, 800);
    gtk_window_set_resizable(arayuz->pencere, TRUE);
    
    GtkWidget* ana_kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 0);
    gtk_window_set_child(arayuz->pencere, ana_kutu);
    
    GtkWidget* baslik_kutu = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 12);
    gtk_widget_set_margin_start(baslik_kutu, 12);
    gtk_widget_set_margin_end(baslik_kutu, 12);
    gtk_widget_set_margin_top(baslik_kutu, 12);
    gtk_widget_set_margin_bottom(baslik_kutu, 12);
    gtk_box_append(GTK_BOX(ana_kutu), baslik_kutu);
    
    GtkWidget* baslik_label = gtk_label_new("Worm");
    gtk_widget_add_css_class(baslik_label, "title-1");
    gtk_box_append(GTK_BOX(baslik_kutu), baslik_label);
    
    GtkWidget* spacer = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    gtk_widget_set_hexpand(spacer, TRUE);
    gtk_box_append(GTK_BOX(baslik_kutu), spacer);
    
    arayuz->durum_label = GTK_LABEL(gtk_label_new("Hazir"));
    gtk_box_append(GTK_BOX(baslik_kutu), GTK_WIDGET(arayuz->durum_label));
    
    GtkWidget* ayrac1 = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
    gtk_box_append(GTK_BOX(ana_kutu), ayrac1);
    
    arayuz->sekmeler = GTK_NOTEBOOK(gtk_notebook_new());
    gtk_notebook_set_tab_pos(arayuz->sekmeler, GTK_POS_LEFT);
    gtk_widget_set_vexpand(GTK_WIDGET(arayuz->sekmeler), TRUE);
    gtk_box_append(GTK_BOX(ana_kutu), GTK_WIDGET(arayuz->sekmeler));
    
    GtkWidget* ayrac2 = gtk_separator_new(GTK_ORIENTATION_HORIZONTAL);
    gtk_box_append(GTK_BOX(ana_kutu), ayrac2);
    
    GtkWidget* alt_kutu = gtk_box_new(GTK_ORIENTATION_VERTICAL, 6);
    gtk_widget_set_margin_start(alt_kutu, 12);
    gtk_widget_set_margin_end(alt_kutu, 12);
    gtk_widget_set_margin_top(alt_kutu, 6);
    gtk_widget_set_margin_bottom(alt_kutu, 12);
    gtk_box_append(GTK_BOX(ana_kutu), alt_kutu);
    
    arayuz->ilerleme = GTK_PROGRESS_BAR(gtk_progress_bar_new());
    gtk_box_append(GTK_BOX(alt_kutu), GTK_WIDGET(arayuz->ilerleme));
    
    GtkWidget* log_frame = gtk_frame_new("Sistem Gunlugu");
    gtk_widget_set_size_request(log_frame, -1, 150);
    gtk_box_append(GTK_BOX(alt_kutu), log_frame);
    
    GtkWidget* scrolled = gtk_scrolled_window_new();
    gtk_scrolled_window_set_policy(GTK_SCROLLED_WINDOW(scrolled),
                                   GTK_POLICY_AUTOMATIC,
                                   GTK_POLICY_AUTOMATIC);
    gtk_frame_set_child(GTK_FRAME(log_frame), scrolled);
    
    GtkWidget* log_text = gtk_text_view_new();
    gtk_text_view_set_editable(GTK_TEXT_VIEW(log_text), FALSE);
    gtk_text_view_set_cursor_visible(GTK_TEXT_VIEW(log_text), FALSE);
    gtk_text_view_set_wrap_mode(GTK_TEXT_VIEW(log_text), GTK_WRAP_WORD_CHAR);
    gtk_scrolled_window_set_child(GTK_SCROLLED_WINDOW(scrolled), log_text);
    
    arayuz->log_buffer = gtk_text_view_get_buffer(GTK_TEXT_VIEW(log_text));
    g_object_set_data(G_OBJECT(arayuz->log_buffer), "text_view", log_text);
    
    g_signal_connect(arayuz->pencere, "close-request", G_CALLBACK(pencere_kapat), arayuz);
    
    arayuz_log_ekle(arayuz, "Arayuz olusturuldu", GUNLUK_SEVIYE_INFO);
    
    return arayuz;
}

void arayuz_kapat(ArayuzYonetici* arayuz) {
    if (!arayuz) return;
    pencere_kapat(arayuz->pencere, arayuz);
}
