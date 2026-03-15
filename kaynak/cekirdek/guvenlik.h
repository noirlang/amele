#ifndef GUVENLIK_H
#define GUVENLIK_H

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>
#include <time.h>
#include <glib.h>

#ifdef __cplusplus
extern "C" {
#endif

#define TOKEN_UZUNLUK 64
#define MAX_YANLIS_DENEME 3
#define KILIT_SURESI 300  // 5 dakika

typedef struct {
    char beklenen_token[TOKEN_UZUNLUK + 1];
    int yanlis_deneme;
    time_t kilitli_zaman;
    GMutex kilit;
    bool aktif;
} GuvenlikYonetici;

// Guvenlik yonetimi
GuvenlikYonetici* guvenlik_yonetici_olustur(void);
void guvenlik_yonetici_yok_et(GuvenlikYonetici* guvenlik);

// Token islemleri
bool guvenlik_token_ayarla(GuvenlikYonetici* guvenlik, const char* token);
bool guvenlik_token_dogrula(GuvenlikYonetici* guvenlik, const char* token);
bool guvenlik_token_gerekli_mi(GuvenlikYonetici* guvenlik);

// Kilitleme
bool guvenlik_kilitli_mi(GuvenlikYonetici* guvenlik);
int guvenlik_kalan_kilit_suresi(GuvenlikYonetici* guvenlik);
void guvenlik_yanlis_deneme_sifirla(GuvenlikYonetici* guvenlik);

// Sifreleme yardimcilari
char* guvenlik_sha256(const char* girdi);
bool guvenlik_token_uret(char* cikti, size_t boyut);

// Input validasyonu
typedef enum {
    INPUT_TIP_IP,
    INPUT_TIP_PORT,
    INPUT_TIP_DOSYA_YOLU,
    INPUT_TIP_VAKA_ADI,
    INPUT_TIP_TOKEN,
    INPUT_TIP_DISK_ID,
} InputTipi;

bool input_dogrula(InputTipi tip, const char* deger);
bool input_ip_dogrula(const char* ip);
bool input_port_dogrula(int port);
bool input_dosya_yolu_guvenli_mi(const char* yol);
bool input_vaka_adi_dogrula(const char* ad);
bool input_disk_id_dogrula(const char* disk_id);

#ifdef __cplusplus
}
#endif

#endif
