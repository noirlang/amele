#ifndef AYARLAR_H
#define AYARLAR_H

#include <stdbool.h>
#include <glib.h>
#include <json-glib/json-glib.h>

typedef struct {
    int varsayilan_port;
    int varsayilan_boyut_mb;
    int disk_algilama_araligi_ms;
    char* cikti_klasoru;
    char* vaka_klasoru;
    bool otomatik_rapor;
    bool karanlik_tema;
    char dil[16];
    char hash_algoritmasi[16];
    int parca_boyutu;
} UygulamaAyarlar;

UygulamaAyarlar* ayarlar_yukle(const char* dosya_yolu);
void ayarlar_kaydet(UygulamaAyarlar* ayarlar, const char* dosya_yolu);
void ayarlar_varsayilan(UygulamaAyarlar* ayarlar);
void ayarlar_temizle(UygulamaAyarlar* ayarlar);
const char* ayarlar_vaka_klasoru_al(UygulamaAyarlar* ayarlar);

#endif
