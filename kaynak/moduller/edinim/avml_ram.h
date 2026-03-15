#ifndef AVML_RAM_H
#define AVML_RAM_H

#include <stdbool.h>
#include <stdint.h>
#include <glib.h>
#include "hata_yonetim.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    char avml_yolu[512];
    char cikti_dosyasi[512];
    int64_t toplam_boyut;
    int64_t okunan_boyut;
    bool calisiyor;
    GMutex kilit;
} AVMLEdinim;

typedef void (*AVMLIlerlemeCallback)(int64_t okunan, int64_t toplam, void* kullanici_verisi);

AVMLEdinim* avml_edinim_olustur(void);
void avml_edinim_yok_et(AVMLEdinim* edinim);

bool avml_binary_kontrol(AVMLEdinim* edinim, const char* yol);
bool avml_root_yetkisi_kontrol(void);
int64_t avml_ram_boyut_al(void);

bool avml_ram_al(AVMLEdinim* edinim,
                 const char* cikti_dosyasi,
                 AVMLIlerlemeCallback ilerleme_callback,
                 void* kullanici_verisi,
                 HataKodu* hata_kodu);

bool avml_calisiyor_mu(AVMLEdinim* edinim);
void avml_iptal_et(AVMLEdinim* edinim);

#ifdef __cplusplus
}
#endif

#endif
