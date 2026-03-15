#ifndef WINPMEM_RAM_H
#define WINPMEM_RAM_H

#include <stdbool.h>
#include <stdint.h>
#include <glib.h>
#include "hata_yonetim.h"

#ifdef __cplusplus
extern "C" {
#endif

// WinPMEM yapilandirmasi
typedef struct {
    char winpmem_yolu[512];      // WinPMEM binary yolu
    char cikti_dosyasi[512];     // RAM dump cikti dosyasi
    int64_t toplam_boyut;        // Toplam RAM boyutu
    int64_t okunan_boyut;        // Simdiye kadar okunan
    bool calisiyor;              // Edinim devam ediyor mu
    bool yonetici_yetkisi;       // Yonetici yetkisi kontrolu
    GMutex kilit;
} WinPMEMEdinim;

// Ilerleme callback fonksiyon tipi
typedef void (*WinPMEMIlerlemeCallback)(int64_t okunan, int64_t toplam, void* kullanici_verisi);

// WinPMEM edinim fonksiyonlari
WinPMEMEdinim* winpmem_edinim_olustur(void);
void winpmem_edinim_yok_et(WinPMEMEdinim* edinim);

// WinPMEM binary kontrolu
bool winpmem_binary_kontrol(WinPMEMEdinim* edinim, const char* yol);

// Yonetici yetkisi kontrolu (Windows uzerinde)
bool winpmem_yonetici_yetkisi_kontrol(void);

// RAM boyutunu al
int64_t winpmem_ram_boyut_al(void);

// RAM edinim islemi
bool winpmem_ram_al(WinPMEMEdinim* edinim, 
                    const char* cikti_dosyasi,
                    WinPMEMIlerlemeCallback ilerleme_callback,
                    void* kullanici_verisi,
                    HataKodu* hata_kodu);

// Edinim durumunu kontrol et
bool winpmem_calisiyor_mu(WinPMEMEdinim* edinim);
int64_t winpmem_ilerleme_al(WinPMEMEdinim* edinim);

// Edinimi iptal et
void winpmem_iptal_et(WinPMEMEdinim* edinim);

// Volatility3 ile analiz icin hazirlik
bool winpmem_volatility3_hazirlik(const char* ram_dosyasi, 
                                  const char* sembol_dosyasi,
                                  HataKodu* hata_kodu);

#ifdef __cplusplus
}
#endif

#endif
