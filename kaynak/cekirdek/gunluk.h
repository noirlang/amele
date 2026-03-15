#ifndef GUNLUK_H
#define GUNLUK_H

#include <stdio.h>
#include <stdarg.h>
#include <time.h>
#include <string.h>
#include <stdbool.h>
#include <glib.h>

typedef enum {
    GUNLUK_SEVIYE_INFO,
    GUNLUK_SEVIYE_WARN,
    GUNLUK_SEVIYE_ERROR,
    GUNLUK_SEVIYE_DEBUG
} GunlukSeviye;

typedef struct {
    char* gunluk_klasor;
    char* vaka_adi;
    FILE* aktif_dosya;
    GMutex kilit;
    GunlukSeviye min_seviye;
} GunlukYonetici;

GunlukYonetici* gunluk_baslat(const char* vaka_adi, const char* gunluk_klasor);
void gunluk_kapat(GunlukYonetici* yonetici);
void gunluk_yaz(GunlukYonetici* yonetici, GunlukSeviye seviye, const char* format, ...);
void gunluk_sistem(GunlukYonetici* yonetici, GunlukSeviye seviye, const char* mesaj);

#define gunluk_info(y, ...) gunluk_yaz(y, GUNLUK_SEVIYE_INFO, __VA_ARGS__)
#define gunluk_warn(y, ...) gunluk_yaz(y, GUNLUK_SEVIYE_WARN, __VA_ARGS__)
#define gunluk_error(y, ...) gunluk_yaz(y, GUNLUK_SEVIYE_ERROR, __VA_ARGS__)
#define gunluk_debug(y, ...) gunluk_yaz(y, GUNLUK_SEVIYE_DEBUG, __VA_ARGS__)

#endif
