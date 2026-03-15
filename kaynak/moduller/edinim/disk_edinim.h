#ifndef DISK_EDINIM_H
#define DISK_EDINIM_H

#include <stdbool.h>
#include <stdint.h>
#include "is_kuyrugu.h"

typedef enum {
    EDINIM_KAYNAK_DISK,
    EDINIM_KAYNAK_DOSYA
} EdinimKaynak;

typedef struct {
    IsGorevi* is;
    char kaynak[512];
    char hedef[512];
    int64_t baslangic;
    int64_t boyut;
    int parca_boyutu;
    bool hash_hesapla;
    bool tam_disk;
} DiskEdinimGorevi;

typedef struct {
    char cihaz[256];
    int64_t toplam_boyut;
    int64_t kullanilan_boyut;
    bool erisilebilir;
} DiskBilgisi;

bool disk_edinim_gorevi_calistir(DiskEdinimGorevi* gorev);
bool disk_listele(DiskBilgisi** diskler, int* disk_sayisi);
int64_t disk_boyut_al(const char* cihaz);
bool disk_imaj_dogrula(const char* imaj_yolu, const char* beklenen_hash);
bool disk_edinim_iptal(void);

#endif
