#include "is_kuyrugu.h"
#include <stdlib.h>
#include <string.h>

static int sonraki_id = 1;

const char* is_durum_metin(IsDurumu durum) {
    switch (durum) {
        case IS_DURUMU_BEKLIYOR:    return "Bekliyor";
        case IS_DURUMU_CALISIYOR:   return "Calisiyor";
        case IS_DURUMU_TAMAMLANDI:  return "Tamamlandi";
        case IS_DURUMU_HATA:        return "Hata";
        case IS_DURUMU_IPTAL_EDILDI:return "Iptal Edildi";
        default: return "Bilinmiyor";
    }
}

const char* is_tip_metin(IsTipi tip) {
    switch (tip) {
        case IS_TIPI_DISK_EDINIM:      return "Disk Edinim";
        case IS_TIPI_HASH_HESAPLA:     return "Hash Hesaplama";
        case IS_TIPI_DOGRULAMA:        return "Dogrulama";
        case IS_TIPI_SISTEM_BILGISI:   return "Sistem Bilgisi";
        case IS_TIPI_AG_TRANSFERI:     return "Ag Transferi";
        case IS_TIPI_RAPOR_OLUSTUR:    return "Rapor Olustur";
        default: return "Bilinmiyor";
    }
}

IsKuyrugu* is_kuyrugu_olustur(GunlukYonetici* gunluk) {
    IsKuyrugu* k = calloc(1, sizeof(IsKuyrugu));
    if (!k) return NULL;

    k->kuyruk = g_queue_new();
    g_mutex_init(&k->kilit);
    g_cond_init(&k->kosul);
    k->calisiyor = TRUE;
    k->gunluk = gunluk;

    if (gunluk) {
        gunluk_info(gunluk, "Is kuyrugu olusturuldu");
    }
    return k;
}

void is_kuyrugu_kapat(IsKuyrugu* k) {
    if (!k) return;

    g_mutex_lock(&k->kilit);
    k->calisiyor = FALSE;
    g_cond_broadcast(&k->kosul);
    g_mutex_unlock(&k->kilit);

    while (!g_queue_is_empty(k->kuyruk)) {
        IsGorevi* is = g_queue_pop_head(k->kuyruk);
        is_temizle(is);
    }

    g_queue_free(k->kuyruk);
    g_mutex_clear(&k->kilit);
    g_cond_clear(&k->kosul);

    if (k->gunluk) {
        gunluk_info(k->gunluk, "Is kuyrugu kapatildi");
    }
    free(k);
}

IsGorevi* is_olustur(IsTipi tip, const char* aciklama) {
    IsGorevi* is = calloc(1, sizeof(IsGorevi));
    if (!is) return NULL;

    is->id = sonraki_id++;
    is->tip = tip;
    is->durum = IS_DURUMU_BEKLIYOR;
    is->aciklama = g_strdup(aciklama);
    is->ilerleme_yuzde = 0;
    is->baslama_zamani = 0;
    is->bitis_zamani = 0;

    return is;
}

void is_ekle(IsKuyrugu* k, IsGorevi* is) {
    if (!k || !is) return;

    g_mutex_lock(&k->kilit);
    g_queue_push_tail(k->kuyruk, is);
    if (k->gunluk) {
        gunluk_info(k->gunluk, "Is eklendi: %s (ID: %d)", is->aciklama, is->id);
    }
    g_cond_signal(&k->kosul);
    g_mutex_unlock(&k->kilit);
}

void is_durum_guncelle(IsGorevi* is, IsDurumu durum, int ilerleme) {
    if (!is) return;

    is->durum = durum;
    if (ilerleme >= 0 && ilerleme <= 100) {
        is->ilerleme_yuzde = ilerleme;
    }

    if (durum == IS_DURUMU_CALISIYOR && is->baslama_zamani == 0) {
        is->baslama_zamani = time(NULL);
    }
    if ((durum == IS_DURUMU_TAMAMLANDI || durum == IS_DURUMU_HATA || durum == IS_DURUMU_IPTAL_EDILDI)
        && is->bitis_zamani == 0) {
        is->bitis_zamani = time(NULL);
    }
}

void is_tamamla(IsGorevi* is, const char* cikti_klasoru) {
    if (!is) return;
    is_durum_guncelle(is, IS_DURUMU_TAMAMLANDI, 100);
    if (cikti_klasoru) {
        is->cikti_klasoru = g_strdup(cikti_klasoru);
    }
    if (is->tamamlandi_callback) {
        is->tamamlandi_callback(is, is->kullanici_verisi);
    }
}

void is_hata(IsGorevi* is, const char* hata_mesaji) {
    if (!is) return;
    is_durum_guncelle(is, IS_DURUMU_HATA, -1);
    is->hata_mesaji = g_strdup(hata_mesaji);
    if (is->tamamlandi_callback) {
        is->tamamlandi_callback(is, is->kullanici_verisi);
    }
}

void is_urun_dosya_ekle(IsGorevi* is, const char* dosya_yolu) {
    if (!is || !dosya_yolu) return;
    is->uretilen_dosyalar = g_list_append(is->uretilen_dosyalar, g_strdup(dosya_yolu));
}

void is_temizle(IsGorevi* is) {
    if (!is) return;
    g_free(is->aciklama);
    g_free(is->cikti_klasoru);
    g_free(is->hata_mesaji);
    g_list_free_full(is->uretilen_dosyalar, g_free);
    free(is);
}
