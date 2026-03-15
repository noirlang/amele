#include "gunluk.h"
#include <stdlib.h>
#include <errno.h>

static const char* seviye_str(GunlukSeviye s) {
    switch (s) {
        case GUNLUK_SEVIYE_INFO:  return "INFO";
        case GUNLUK_SEVIYE_WARN:  return "WARN";
        case GUNLUK_SEVIYE_ERROR: return "ERROR";
        case GUNLUK_SEVIYE_DEBUG: return "DEBUG";
        default: return "UNKNOWN";
    }
}

GunlukYonetici* gunluk_baslat(const char* vaka_adi, const char* gunluk_klasor) {
    GunlukYonetici* y = calloc(1, sizeof(GunlukYonetici));
    if (!y) return NULL;

    y->vaka_adi = g_strdup(vaka_adi);
    y->gunluk_klasor = g_strdup(gunluk_klasor);
    y->min_seviye = GUNLUK_SEVIYE_DEBUG;
    g_mutex_init(&y->kilit);

    if (g_mkdir_with_parents(gunluk_klasor, 0755) != 0) {
        g_free(y->vaka_adi);
        g_free(y->gunluk_klasor);
        free(y);
        return NULL;
    }

    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    char dosya_adi[512];
    snprintf(dosya_adi, sizeof(dosya_adi), "%s/%s_%04d%02d%02d_%02d%02d%02d.log",
             gunluk_klasor, vaka_adi,
             tm_info->tm_year + 1900, tm_info->tm_mon + 1, tm_info->tm_mday,
             tm_info->tm_hour, tm_info->tm_min, tm_info->tm_sec);

    y->aktif_dosya = fopen(dosya_adi, "a");
    if (!y->aktif_dosya) {
        g_free(y->vaka_adi);
        g_free(y->gunluk_klasor);
        free(y);
        return NULL;
    }

    gunluk_yaz(y, GUNLUK_SEVIYE_INFO, "Gunluk sistemi baslatildi: %s", dosya_adi);
    return y;
}

void gunluk_kapat(GunlukYonetici* y) {
    if (!y) return;
    gunluk_yaz(y, GUNLUK_SEVIYE_INFO, "Gunluk sistemi kapatiliyor");
    if (y->aktif_dosya) fclose(y->aktif_dosya);
    g_free(y->vaka_adi);
    g_free(y->gunluk_klasor);
    g_mutex_clear(&y->kilit);
    free(y);
}

void gunluk_yaz(GunlukYonetici* y, GunlukSeviye seviye, const char* format, ...) {
    if (!y || seviye < y->min_seviye) return;

    g_mutex_lock(&y->kilit);

    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    char zaman[32];
    strftime(zaman, sizeof(zaman), "%Y-%m-%d %H:%M:%S", tm_info);

    va_list args;
    va_start(args, format);
    char mesaj[4096];
    vsnprintf(mesaj, sizeof(mesaj), format, args);
    va_end(args);

    if (y->aktif_dosya) {
        fprintf(y->aktif_dosya, "[%s] | %s | %s\n", zaman, seviye_str(seviye), mesaj);
        fflush(y->aktif_dosya);
    }

    g_mutex_unlock(&y->kilit);
}

void gunluk_sistem(GunlukYonetici* y, GunlukSeviye seviye, const char* mesaj) {
    gunluk_yaz(y, seviye, "[SISTEM] %s", mesaj);
}
