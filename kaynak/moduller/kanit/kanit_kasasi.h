#ifndef KANIT_KASASI_H
#define KANIT_KASASI_H

#include <stdbool.h>
#include <glib.h>
#include "gunluk.h"

typedef struct {
    char vaka_adi[256];
    char vaka_klasoru[1024];
    char gunlukler_klasoru[1024];
    char ciktilar_klasoru[1024];
    char raporlar_klasoru[1024];
    char hash_klasoru[1024];
    char notlar_klasoru[1024];
    GunlukYonetici* gunluk;
    GMutex kilit;
} KanitKasasi;

KanitKasasi* kanit_kasasi_olustur(const char* ana_klasor, const char* vaka_adi);
void kanit_kasasi_kapat(KanitKasasi* kasa);
bool kanit_kasasi_dizinler_olustur(KanitKasasi* kasa);
char* kanit_kasasi_yeni_dosya(KanitKasasi* kasa, const char* alt_klasor, const char* dosya_adi);
bool kanit_kasasi_not_ekle(KanitKasasi* kasa, const char* not_icerik);
GList* kanit_kasasi_dosyalari_listele(KanitKasasi* kasa, const char* alt_klasor);
char* kanit_kasasi_vaka_ozet(KanitKasasi* kasa);

#endif
