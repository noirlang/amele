#ifndef IS_KUYRUGU_H
#define IS_KUYRUGU_H

#include <stdbool.h>
#include <time.h>
#include <glib.h>
#include "gunluk.h"

typedef enum {
    IS_DURUMU_BEKLIYOR,
    IS_DURUMU_CALISIYOR,
    IS_DURUMU_TAMAMLANDI,
    IS_DURUMU_HATA,
    IS_DURUMU_IPTAL_EDILDI
} IsDurumu;

typedef enum {
    IS_TIPI_DISK_EDINIM,
    IS_TIPI_HASH_HESAPLA,
    IS_TIPI_DOGRULAMA,
    IS_TIPI_SISTEM_BILGISI,
    IS_TIPI_AG_TRANSFERI,
    IS_TIPI_RAPOR_OLUSTUR
} IsTipi;

typedef struct IsGorevi {
    int id;
    IsTipi tip;
    IsDurumu durum;
    char* aciklama;
    time_t baslama_zamani;
    time_t bitis_zamani;
    int ilerleme_yuzde;
    char* cikti_klasoru;
    GList* uretilen_dosyalar;
    char* hata_mesaji;
    void* kullanici_verisi;
    void (*tamamlandi_callback)(struct IsGorevi* is, void* kullanici_verisi);
} IsGorevi;

typedef struct {
    GQueue* kuyruk;
    GMutex kilit;
    GCond kosul;
    gboolean calisiyor;
    GunlukYonetici* gunluk;
} IsKuyrugu;

IsKuyrugu* is_kuyrugu_olustur(GunlukYonetici* gunluk);
void is_kuyrugu_kapat(IsKuyrugu* kuyruk);
IsGorevi* is_olustur(IsTipi tip, const char* aciklama);
void is_ekle(IsKuyrugu* kuyruk, IsGorevi* is);
void is_durum_guncelle(IsGorevi* is, IsDurumu durum, int ilerleme);
void is_tamamla(IsGorevi* is, const char* cikti_klasoru);
void is_hata(IsGorevi* is, const char* hata_mesaji);
void is_urun_dosya_ekle(IsGorevi* is, const char* dosya_yolu);
void is_temizle(IsGorevi* is);
const char* is_durum_metin(IsDurumu durum);
const char* is_tip_metin(IsTipi tip);

#endif
