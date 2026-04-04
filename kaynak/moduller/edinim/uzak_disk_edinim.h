#ifndef UZAK_DISK_EDINIM_H
#define UZAK_DISK_EDINIM_H

#include <stdbool.h>
#include <gio/gio.h>
#include "is_kuyrugu.h"

typedef enum {
    UZAK_DURUM_BAGLI,
    UZAK_DURUM_BAGLANTI_YOK,
    UZAK_DURUM_IMAJ_ALINAMADI
} UzakDiskDurum;

typedef enum {
    UZAK_PROTO_JSON,
    UZAK_PROTO_VERI
} UzakProtokolDurum;

typedef struct {
    char ip[64];
    int port;
    char token[128];
    GSocketConnection* baglanti;
    GDataInputStream* girdi;
    GOutputStream* cikti;
    UzakProtokolDurum proto_durum;
    char is_id[32];
    int64_t toplam_boyut;
    int64_t okunan;
    bool son_yanit_ok;
    char son_hata[256];
    FILE* hedef_dosya;
    char hedef_yol[1024];
} UzakDiskBaglanti;

typedef struct {
    char id[32];
    char ad[256];
    int64_t boyut;
} UzakDisk;

UzakDiskBaglanti* uzak_disk_baglanti_olustur(const char* ip, int port, const char* token);
void uzak_disk_baglanti_kapat(UzakDiskBaglanti* baglanti);
bool uzak_disk_baglan(UzakDiskBaglanti* baglanti);
GList* uzak_disk_listele(UzakDiskBaglanti* baglanti);
bool uzak_imaj_baslat(UzakDiskBaglanti* baglanti, const char* disk_id, 
                       const char* hedef_klasor, const char* vaka_adi);
bool uzak_imaj_stream_al(UzakDiskBaglanti* baglanti, IsGorevi* is,
                          void (*ilerleme_cb)(int64_t okunan, int64_t toplam, void* veri),
                          void* kullanici_verisi);

bool uzak_winpmem_kontrol(UzakDiskBaglanti* baglanti,
                          bool* winpmem_mevcut,
                          bool* yonetici_yetkisi,
                          int64_t* ram_boyut,
                          char* mesaj,
                          size_t mesaj_boyut);

bool uzak_avml_kontrol(UzakDiskBaglanti* baglanti,
                       bool* avml_mevcut,
                       bool* yonetici_yetkisi,
                       int64_t* ram_boyut,
                       char* mesaj,
                       size_t mesaj_boyut);

bool uzak_ram_edinim_baslat_ve_takip(UzakDiskBaglanti* baglanti,
                                     const char* cikti_dosya,
                                     const char* is_id_istek,
                                     void (*ilerleme_cb)(int64_t okunan, int64_t toplam, void* veri),
                                     void* kullanici_verisi,
                                     char* sonuc_metin,
                                     size_t sonuc_metin_boyut,
                                     char* is_id_cikti,
                                     size_t is_id_cikti_boyut);

bool uzak_edinim_kontrol_gonder(UzakDiskBaglanti* baglanti,
                                const char* is_id,
                                const char* eylem,
                                char* sonuc_metin,
                                size_t sonuc_metin_boyut);

bool uzak_ram_dosya_indir(UzakDiskBaglanti* baglanti,
                          const char* uzak_dosya,
                          const char* yerel_yol,
                          void (*ilerleme_cb)(int64_t okunan, int64_t toplam, void* veri),
                          void* kullanici_verisi,
                          char* sonuc_metin,
                          size_t sonuc_metin_boyut);

#endif
