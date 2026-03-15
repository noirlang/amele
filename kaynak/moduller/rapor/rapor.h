#ifndef RAPOR_H
#define RAPOR_H

#include <stdbool.h>
#include "kanit_kasasi.h"

typedef struct {
    char baslik[256];
    char aciklama[1024];
    char olusturan[128];
    char kaynak[512];
    char hash[129];
    char tarih[64];
} RaporBilgisi;

typedef enum {
    RAPOR_FORMAT_TXT,
    RAPOR_FORMAT_JSON
} RaporFormat;

bool rapor_olustur(RaporBilgisi* bilgi, RaporFormat format, const char* hedef_dosya, KanitKasasi* kasa);
bool rapor_dosya_ozet(const char* dosya_yolu, char* ozet, size_t ozet_boyutu);
bool rapor_sistem_bilgisi_ekle(const char* hedef_dosya);
char* rapor_yeni_dosya_adi(const char* vaka_adi, RaporFormat format);

#endif
