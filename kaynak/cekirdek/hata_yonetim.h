#ifndef HATA_YONETIM_H
#define HATA_YONETIM_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// Hata kodlari
typedef enum {
    HATA_OK = 0,
    HATA_GENEL = -1,
    HATA_BAGLANTI = -100,
    HATA_BAGLANTI_ZAMAN_ASIMI = -101,
    HATA_BAGLANTI_KESILDI = -102,
    HATA_BAGLANTI_TEKRAR_DENE = -103,
    HATA_DOSYA = -200,
    HATA_DOSYA_ACILAMADI = -201,
    HATA_DOSYA_YAZMA = -202,
    HATA_DOSYA_OKUMA = -203,
    HATA_DISK = -300,
    HATA_DISK_ERISIM = -301,
    HATA_DISK_OKUMA = -302,
    HATA_DISK_BOYUT = -303,
    HATA_AG = -400,
    HATA_AG_GONDERME = -401,
    HATA_AG_ALMA = -402,
    HATA_PROTOKOL = -500,
    HATA_PROTOKOL_JSON = -501,
    HATA_PROTOKOL_VERSIYON = -502,
    HATA_GUVENLIK = -600,
    HATA_TOKEN_GECERSIZ = -601,
    HATA_YETKISIZ_ERISIM = -602,
    HATA_ICERIK = -700,
    HATA_ICERIK_GECERSIZ = -701,
    HATA_ICERIK_BUYUK = -702,
    HATA_HYPERV = -800,
    HATA_HYPERV_BULUNAMADI = -801,
    HATA_HYPERV_VM_YOK = -802,
    HATA_HYPERV_DOSYA_YOK = -803,
} HataKodu;

typedef struct {
    HataKodu kod;
    char mesaj[512];
    char detay[1024];
    char zaman[64];
    char kaynak_dosya[256];
    int kaynak_satir;
} HataBilgisi;

// Hata yonetimi fonksiyonlari
void hata_kaydet(HataKodu kod, const char* mesaj, const char* detay, 
                 const char* dosya, int satir);
void hata_son_al(HataBilgisi* cikti);
const char* hata_kod_metin(HataKodu kod);
bool hata_ciddi_mi(HataKodu kod);
void hata_gunluk_yaz(HataKodu kod, const char* mesaj);

// Makrolar
#define HATA_KAYDET(kod, mesaj) hata_kaydet(kod, mesaj, NULL, __FILE__, __LINE__)
#define HATA_KAYDET_DETAY(kod, mesaj, detay) hata_kaydet(kod, mesaj, detay, __FILE__, __LINE__)

#ifdef __cplusplus
}
#endif

#endif
