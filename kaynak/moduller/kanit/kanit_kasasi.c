#include "kanit_kasasi.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <time.h>
#include <errno.h>

static bool yol_birlestir(char* hedef, size_t hedef_boyutu, const char* sol, const char* sag) {
    if (!hedef || !sol || !sag || hedef_boyutu == 0) {
        return false;
    }

    int yazilan = snprintf(hedef, hedef_boyutu, "%s/%s", sol, sag);
    return yazilan >= 0 && (size_t)yazilan < hedef_boyutu;
}

static bool dizin_olustur(const char* yol) {
    if (!yol || strlen(yol) == 0) {
        return false;
    }
    if (g_file_test(yol, G_FILE_TEST_IS_DIR)) {
        return true;
    }
    return g_mkdir_with_parents(yol, 0755) == 0;
}

KanitKasasi* kanit_kasasi_olustur(const char* ana_klasor, const char* vaka_adi) {
    if (!ana_klasor || !vaka_adi) return NULL;

    KanitKasasi* kasa = calloc(1, sizeof(KanitKasasi));
    if (!kasa) return NULL;

    strncpy(kasa->vaka_adi, vaka_adi, sizeof(kasa->vaka_adi) - 1);
    kasa->vaka_adi[sizeof(kasa->vaka_adi) - 1] = '\0';

    if (!yol_birlestir(kasa->vaka_klasoru, sizeof(kasa->vaka_klasoru), ana_klasor, vaka_adi) ||
        !yol_birlestir(kasa->gunlukler_klasoru, sizeof(kasa->gunlukler_klasoru), kasa->vaka_klasoru, "gunlukler") ||
        !yol_birlestir(kasa->ciktilar_klasoru, sizeof(kasa->ciktilar_klasoru), kasa->vaka_klasoru, "ciktilar") ||
        !yol_birlestir(kasa->raporlar_klasoru, sizeof(kasa->raporlar_klasoru), kasa->vaka_klasoru, "raporlar") ||
        !yol_birlestir(kasa->hash_klasoru, sizeof(kasa->hash_klasoru), kasa->vaka_klasoru, "hash") ||
        !yol_birlestir(kasa->notlar_klasoru, sizeof(kasa->notlar_klasoru), kasa->vaka_klasoru, "notlar")) {
        free(kasa);
        return NULL;
    }

    g_mutex_init(&kasa->kilit);

    if (!kanit_kasasi_dizinler_olustur(kasa)) {
        g_mutex_clear(&kasa->kilit);
        free(kasa);
        return NULL;
    }

    kasa->gunluk = gunluk_baslat(vaka_adi, kasa->gunlukler_klasoru);

    if (kasa->gunluk) {
        gunluk_info(kasa->gunluk, "Vaka olusturuldu: %s", vaka_adi);
        gunluk_info(kasa->gunluk, "Vaka klasoru: %s", kasa->vaka_klasoru);
    }

    return kasa;
}

void kanit_kasasi_kapat(KanitKasasi* kasa) {
    if (!kasa) return;

    if (kasa->gunluk) {
        gunluk_info(kasa->gunluk, "Vaka kapatiliyor: %s", kasa->vaka_adi);
        gunluk_kapat(kasa->gunluk);
    }

    g_mutex_clear(&kasa->kilit);
    free(kasa);
}

bool kanit_kasasi_dizinler_olustur(KanitKasasi* kasa) {
    if (!kasa) return false;

    bool basarili = true;
    basarili &= dizin_olustur(kasa->vaka_klasoru);
    basarili &= dizin_olustur(kasa->gunlukler_klasoru);
    basarili &= dizin_olustur(kasa->ciktilar_klasoru);
    basarili &= dizin_olustur(kasa->raporlar_klasoru);
    basarili &= dizin_olustur(kasa->hash_klasoru);
    basarili &= dizin_olustur(kasa->notlar_klasoru);

    return basarili;
}

char* kanit_kasasi_yeni_dosya(KanitKasasi* kasa, const char* alt_klasor, const char* dosya_adi) {
    if (!kasa || !alt_klasor || !dosya_adi) return NULL;

    g_mutex_lock(&kasa->kilit);

    const char* hedef_klasor = NULL;
    if (strcmp(alt_klasor, "gunlukler") == 0) hedef_klasor = kasa->gunlukler_klasoru;
    else if (strcmp(alt_klasor, "ciktilar") == 0) hedef_klasor = kasa->ciktilar_klasoru;
    else if (strcmp(alt_klasor, "raporlar") == 0) hedef_klasor = kasa->raporlar_klasoru;
    else if (strcmp(alt_klasor, "hash") == 0) hedef_klasor = kasa->hash_klasoru;
    else if (strcmp(alt_klasor, "notlar") == 0) hedef_klasor = kasa->notlar_klasoru;
    else hedef_klasor = kasa->vaka_klasoru;

    char* tam_yol = g_strdup_printf("%s/%s", hedef_klasor, dosya_adi);

    g_mutex_unlock(&kasa->kilit);

    return tam_yol;
}

bool kanit_kasasi_not_ekle(KanitKasasi* kasa, const char* not_icerik) {
    if (!kasa || !not_icerik) return false;

    g_mutex_lock(&kasa->kilit);

    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    char dosya_adi[256];
    snprintf(dosya_adi, sizeof(dosya_adi), "not_%04d%02d%02d_%02d%02d%02d.txt",
             tm_info->tm_year + 1900, tm_info->tm_mon + 1, tm_info->tm_mday,
             tm_info->tm_hour, tm_info->tm_min, tm_info->tm_sec);

    char tam_yol[1024];
    if (!yol_birlestir(tam_yol, sizeof(tam_yol), kasa->notlar_klasoru, dosya_adi)) {
        g_mutex_unlock(&kasa->kilit);
        return false;
    }

    FILE* f = fopen(tam_yol, "w");
    if (!f) {
        g_mutex_unlock(&kasa->kilit);
        return false;
    }

    fprintf(f, "Vaka: %s\n", kasa->vaka_adi);
    fprintf(f, "Tarih: %04d-%02d-%02d %02d:%02d:%02d\n",
            tm_info->tm_year + 1900, tm_info->tm_mon + 1, tm_info->tm_mday,
            tm_info->tm_hour, tm_info->tm_min, tm_info->tm_sec);
    fprintf(f, "========================================\n\n");
    fprintf(f, "%s\n", not_icerik);

    fclose(f);

    g_mutex_unlock(&kasa->kilit);

    if (kasa->gunluk) {
        gunluk_info(kasa->gunluk, "Not eklendi: %s", dosya_adi);
    }

    return true;
}

GList* kanit_kasasi_dosyalari_listele(KanitKasasi* kasa, const char* alt_klasor) {
    if (!kasa || !alt_klasor) return NULL;

    g_mutex_lock(&kasa->kilit);

    const char* hedef_klasor = NULL;
    if (strcmp(alt_klasor, "gunlukler") == 0) hedef_klasor = kasa->gunlukler_klasoru;
    else if (strcmp(alt_klasor, "ciktilar") == 0) hedef_klasor = kasa->ciktilar_klasoru;
    else if (strcmp(alt_klasor, "raporlar") == 0) hedef_klasor = kasa->raporlar_klasoru;
    else if (strcmp(alt_klasor, "hash") == 0) hedef_klasor = kasa->hash_klasoru;
    else if (strcmp(alt_klasor, "notlar") == 0) hedef_klasor = kasa->notlar_klasoru;
    else hedef_klasor = kasa->vaka_klasoru;

    GList* dosyalar = NULL;
    GDir* dir = g_dir_open(hedef_klasor, 0, NULL);
    if (dir) {
        const char* isim;
        while ((isim = g_dir_read_name(dir)) != NULL) {
            char* tam_yol = g_strdup_printf("%s/%s", hedef_klasor, isim);
            dosyalar = g_list_append(dosyalar, tam_yol);
        }
        g_dir_close(dir);
    }

    g_mutex_unlock(&kasa->kilit);

    return dosyalar;
}

char* kanit_kasasi_vaka_ozet(KanitKasasi* kasa) {
    if (!kasa) return NULL;

    g_mutex_lock(&kasa->kilit);

    GList* ciktilar = kanit_kasasi_dosyalari_listele(kasa, "ciktilar");
    GList* hashler = kanit_kasasi_dosyalari_listele(kasa, "hash");
    GList* raporlar = kanit_kasasi_dosyalari_listele(kasa, "raporlar");

    int cikti_sayisi = g_list_length(ciktilar);
    int hash_sayisi = g_list_length(hashler);
    int rapor_sayisi = g_list_length(raporlar);

    g_list_free_full(ciktilar, g_free);
    g_list_free_full(hashler, g_free);
    g_list_free_full(raporlar, g_free);

    char* ozet = g_strdup_printf(
        "Vaka: %s\n"
        "Klasor: %s\n"
        "Cikti Sayisi: %d\n"
        "Hash Dosyasi Sayisi: %d\n"
        "Rapor Sayisi: %d\n",
        kasa->vaka_adi, kasa->vaka_klasoru,
        cikti_sayisi, hash_sayisi, rapor_sayisi
    );

    g_mutex_unlock(&kasa->kilit);

    return ozet;
}
