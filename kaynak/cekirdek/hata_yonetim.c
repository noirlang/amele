#include "hata_yonetim.h"
#include "gunluk.h"
#include <string.h>
#include <stdio.h>
#include <time.h>

static HataBilgisi son_hata = {0};
static GMutex hata_kilit;
static gboolean kilit_baslatildi = FALSE;

static void kilit_baslat(void) {
    if (!kilit_baslatildi) {
        g_mutex_init(&hata_kilit);
        kilit_baslatildi = TRUE;
    }
}

void hata_kaydet(HataKodu kod, const char* mesaj, const char* detay, 
                 const char* dosya, int satir) {
    kilit_baslat();
    g_mutex_lock(&hata_kilit);
    
    son_hata.kod = kod;
    
    if (mesaj) {
        strncpy(son_hata.mesaj, mesaj, sizeof(son_hata.mesaj) - 1);
    } else {
        strncpy(son_hata.mesaj, hata_kod_metin(kod), sizeof(son_hata.mesaj) - 1);
    }
    
    if (detay) {
        strncpy(son_hata.detay, detay, sizeof(son_hata.detay) - 1);
    } else {
        son_hata.detay[0] = '\0';
    }
    
    if (dosya) {
        strncpy(son_hata.kaynak_dosya, dosya, sizeof(son_hata.kaynak_dosya) - 1);
    } else {
        son_hata.kaynak_dosya[0] = '\0';
    }

    son_hata.kaynak_satir = satir;
    
    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    strftime(son_hata.zaman, sizeof(son_hata.zaman), "%Y-%m-%d %H:%M:%S", tm_info);
    
    g_mutex_unlock(&hata_kilit);
}

void hata_son_al(HataBilgisi* cikti) {
    if (!cikti) return;
    
    kilit_baslat();
    g_mutex_lock(&hata_kilit);
    memcpy(cikti, &son_hata, sizeof(HataBilgisi));
    g_mutex_unlock(&hata_kilit);
}

const char* hata_kod_metin(HataKodu kod) {
    switch (kod) {
        case HATA_OK: return "Basarili";
        case HATA_GENEL: return "Genel hata";
        case HATA_BAGLANTI: return "Connection error";
        case HATA_BAGLANTI_ZAMAN_ASIMI: return "Connection timeout";
        case HATA_BAGLANTI_KESILDI: return "Connection lost";
        case HATA_BAGLANTI_TEKRAR_DENE: return "Connection should be retried";
        case HATA_DOSYA: return "Dosya hatasi";
        case HATA_DOSYA_ACILAMADI: return "Dosya acilamadi";
        case HATA_DOSYA_YAZMA: return "Dosya yazma hatasi";
        case HATA_DOSYA_OKUMA: return "Dosya okuma hatasi";
        case HATA_DISK: return "Disk error";
        case HATA_DISK_ERISIM: return "Disk access error";
        case HATA_DISK_OKUMA: return "Disk read error";
        case HATA_DISK_BOYUT: return "Disk size error";
        case HATA_AG: return "Ag hatasi";
        case HATA_AG_GONDERME: return "Ag gonderme hatasi";
        case HATA_AG_ALMA: return "Ag alma hatasi";
        case HATA_PROTOKOL: return "Protokol hatasi";
        case HATA_PROTOKOL_JSON: return "JSON protokol hatasi";
        case HATA_PROTOKOL_VERSIYON: return "Protokol versiyon uyumsuzlugu";
        case HATA_GUVENLIK: return "Guvenlik hatasi";
        case HATA_TOKEN_GECERSIZ: return "Gecersiz token";
        case HATA_YETKISIZ_ERISIM: return "Yetkisiz erisim";
        case HATA_ICERIK: return "Icerik hatasi";
        case HATA_ICERIK_GECERSIZ: return "Gecersiz icerik";
        case HATA_ICERIK_BUYUK: return "Icerik boyutu cok buyuk";
        case HATA_HYPERV: return "Hyper-V hatasi";
        case HATA_HYPERV_BULUNAMADI: return "Hyper-V not found";
        case HATA_HYPERV_VM_YOK: return "VM not found";
        case HATA_HYPERV_DOSYA_YOK: return "Hyper-V file not found";
        default: return "Bilinmeyen hata";
    }
}

bool hata_ciddi_mi(HataKodu kod) {
    return kod <= HATA_DISK_ERISIM;
}

void hata_gunluk_yaz(HataKodu kod, const char* mesaj) {
    // gunluk modulu yuklenmeden once stdout'a yaz
    fprintf(stderr, "[HATA %d] %s: %s\n", kod, hata_kod_metin(kod), mesaj ? mesaj : "");
}
