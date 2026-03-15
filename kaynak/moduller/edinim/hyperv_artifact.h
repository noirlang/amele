#ifndef HYPERV_ARTIFACT_H
#define HYPERV_ARTIFACT_H

#include <stdbool.h>
#include <gio/gio.h>
#include "is_kuyrugu.h"
#include "uzak_disk_edinim.h"

#ifdef __cplusplus
extern "C" {
#endif

// Hyper-V bellek dosyasi bilgisi
typedef struct {
    char dosya_adi[256];
    char tam_yol[1024];
    int64_t boyut;
    char vm_adi[128];
    char tur[32];  // vmem, bin, vhdx, avhdx
} HyperVBellekDosyasi;

// Hyper-V edinim yonetici
typedef struct {
    GSocketConnection* baglanti;
    GDataInputStream* girdi;
    GOutputStream* cikti;
    char hedef_klasor[1024];
} HyperVEdinim;

// Yonetici olusturma/yok etme
HyperVEdinim* hyperv_edinim_olustur(void);
void hyperv_edinim_yok_et(HyperVEdinim* edinim);

// Baglanti
bool hyperv_baglan(HyperVEdinim* edinim, UzakDiskBaglanti* uzak_baglanti);
void hyperv_baglanti_kapat(HyperVEdinim* edinim);

// VM ve bellek dosyalari islemleri
GList* hyperv_vm_listele(HyperVEdinim* edinim);
GList* hyperv_bellek_dosyalari_listele(HyperVEdinim* edinim, const char* vm_adi);

// Dosya indirme
bool hyperv_dosya_indir(HyperVEdinim* edinim, const char* dosya_yolu,
                         const char* hedef_klasor, IsGorevi* is,
                         void (*ilerleme_cb)(int64_t okunan, int64_t toplam, void* veri),
                         void* kullanici_verisi);

// Yardimci fonksiyonlar
bool hyperv_varlik_mi(const char* dosya_yolu);
const char* hyperv_dosya_turu_al(const char* dosya_adi);

#ifdef __cplusplus
}
#endif

#endif // HYPERV_ARTIFACT_H
