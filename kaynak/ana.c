#include <gtk/gtk.h>
#include <stdio.h>
#include <stdlib.h>
#include <locale.h>
#include "arayuz.h"
#include "sekmeler.h"

static void uygulama_baslat(GtkApplication* app, gpointer kullanici_verisi) {
    (void)kullanici_verisi;
    
    ArayuzYonetici* arayuz = arayuz_olustur(app);
    if (!arayuz) {
        g_printerr("Arayuz olusturulamadi!\n");
        return;
    }
    
    sekmeler_olustur(arayuz);
    
    gtk_window_present(arayuz->pencere);
    
    arayuz_log_ekle(arayuz, "Worm basladi", GUNLUK_SEVIYE_INFO);
    arayuz_log_ekle(arayuz, "Sistem hazir", GUNLUK_SEVIYE_INFO);
}

int main(int argc, char* argv[]) {
    setlocale(LC_ALL, "");
    
    GtkApplication* app = gtk_application_new("org.worm.forensic", G_APPLICATION_DEFAULT_FLAGS);
    
    g_signal_connect(app, "activate", G_CALLBACK(uygulama_baslat), NULL);
    
    int status = g_application_run(G_APPLICATION(app), argc, argv);
    
    g_object_unref(app);
    
    return status;
}
