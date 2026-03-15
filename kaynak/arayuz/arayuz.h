#ifndef ARAYUZ_H
#define ARAYUZ_H

#include <gtk/gtk.h>
#include "gunluk.h"
#include "ayarlar.h"
#include "is_kuyrugu.h"
#include "kanit_kasasi.h"

typedef struct {
    GtkApplication* uygulama;
    GtkWindow* pencere;
    GunlukYonetici* gunluk;
    UygulamaAyarlar* ayarlar;
    IsKuyrugu* is_kuyrugu;
    KanitKasasi* kasa;
    
    GtkNotebook* sekmeler;
    GtkProgressBar* ilerleme;
    GtkLabel* durum_label;
    GtkTextBuffer* log_buffer;
    
    gboolean calisiyor;
    GMutex kilit;
} ArayuzYonetici;

ArayuzYonetici* arayuz_olustur(GtkApplication* app);
void arayuz_kapat(ArayuzYonetici* arayuz);
void arayuz_log_ekle(ArayuzYonetici* arayuz, const char* mesaj, GunlukSeviye seviye);
void arayuz_ilerleme_guncelle(ArayuzYonetici* arayuz, double oran);
void arayuz_durum_guncelle(ArayuzYonetici* arayuz, const char* mesaj);
void arayuz_hata_goster(ArayuzYonetici* arayuz, const char* baslik, const char* mesaj);

#endif
