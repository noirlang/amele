#include "guvenlik.h"
#include <string.h>
#include <stdlib.h>
#include <time.h>
#include <openssl/sha.h>
#include <openssl/rand.h>
#include <regex.h>

GuvenlikYonetici* guvenlik_yonetici_olustur(void) {
    GuvenlikYonetici* guvenlik = calloc(1, sizeof(GuvenlikYonetici));
    if (!guvenlik) return NULL;
    
    g_mutex_init(&guvenlik->kilit);
    guvenlik->aktif = false;
    guvenlik->yanlis_deneme = 0;
    guvenlik->kilitli_zaman = 0;
    
    return guvenlik;
}

void guvenlik_yonetici_yok_et(GuvenlikYonetici* guvenlik) {
    if (!guvenlik) return;
    
    memset(guvenlik->beklenen_token, 0, sizeof(guvenlik->beklenen_token));
    guvenlik->aktif = false;
    
    g_mutex_clear(&guvenlik->kilit);
    free(guvenlik);
}

bool guvenlik_token_ayarla(GuvenlikYonetici* guvenlik, const char* token) {
    if (!guvenlik || !token) return false;
    
    g_mutex_lock(&guvenlik->kilit);
    
    if (strlen(token) == 0) {
        guvenlik->aktif = false;
        guvenlik->beklenen_token[0] = '\0';
    } else {
        strncpy(guvenlik->beklenen_token, token, TOKEN_UZUNLUK);
        guvenlik->beklenen_token[TOKEN_UZUNLUK] = '\0';
        guvenlik->aktif = true;
    }
    
    guvenlik->yanlis_deneme = 0;
    guvenlik->kilitli_zaman = 0;
    
    g_mutex_unlock(&guvenlik->kilit);
    return true;
}

bool guvenlik_token_dogrula(GuvenlikYonetici* guvenlik, const char* token) {
    if (!guvenlik) return false;
    
    g_mutex_lock(&guvenlik->kilit);
    
    // Kilitleme kontrolu
    if (guvenlik_kilitli_mi(guvenlik)) {
        g_mutex_unlock(&guvenlik->kilit);
        return false;
    }
    
    // Token gerekli degilse
    if (!guvenlik->aktif) {
        g_mutex_unlock(&guvenlik->kilit);
        return true;
    }
    
    // Token dogrulama
    bool gecerli = false;
    if (token && strcmp(guvenlik->beklenen_token, token) == 0) {
        gecerli = true;
        guvenlik->yanlis_deneme = 0;
    } else {
        guvenlik->yanlis_deneme++;
        if (guvenlik->yanlis_deneme >= MAX_YANLIS_DENEME) {
            guvenlik->kilitli_zaman = time(NULL);
        }
    }
    
    g_mutex_unlock(&guvenlik->kilit);
    return gecerli;
}

bool guvenlik_token_gerekli_mi(GuvenlikYonetici* guvenlik) {
    if (!guvenlik) return false;
    return guvenlik->aktif;
}

bool guvenlik_kilitli_mi(GuvenlikYonetici* guvenlik) {
    if (!guvenlik || guvenlik->yanlis_deneme < MAX_YANLIS_DENEME) {
        return false;
    }
    
    time_t simdi = time(NULL);
    return (simdi - guvenlik->kilitli_zaman) < KILIT_SURESI;
}

int guvenlik_kalan_kilit_suresi(GuvenlikYonetici* guvenlik) {
    if (!guvenlik || !guvenlik_kilitli_mi(guvenlik)) {
        return 0;
    }
    
    time_t simdi = time(NULL);
    return KILIT_SURESI - (simdi - guvenlik->kilitli_zaman);
}

void guvenlik_yanlis_deneme_sifirla(GuvenlikYonetici* guvenlik) {
    if (!guvenlik) return;
    
    g_mutex_lock(&guvenlik->kilit);
    guvenlik->yanlis_deneme = 0;
    guvenlik->kilitli_zaman = 0;
    g_mutex_unlock(&guvenlik->kilit);
}

char* guvenlik_sha256(const char* girdi) {
    if (!girdi) return NULL;
    
    unsigned char hash[SHA256_DIGEST_LENGTH];
    SHA256((unsigned char*)girdi, strlen(girdi), hash);
    
    char* cikti = malloc(SHA256_DIGEST_LENGTH * 2 + 1);
    if (!cikti) return NULL;
    
    for (int i = 0; i < SHA256_DIGEST_LENGTH; i++) {
        sprintf(cikti + (i * 2), "%02x", hash[i]);
    }
    cikti[SHA256_DIGEST_LENGTH * 2] = '\0';
    
    return cikti;
}

bool guvenlik_token_uret(char* cikti, size_t boyut) {
    if (!cikti || boyut < 33) return false;
    
    const char karakterler[] = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    unsigned char rastgele[32];
    
    if (RAND_bytes(rastgele, sizeof(rastgele)) != 1) {
        // RAND_bytes basarisizsa, time-based fallback
        srand(time(NULL));
        for (int i = 0; i < 32; i++) {
            rastgele[i] = rand() % 256;
        }
    }
    
    for (int i = 0; i < 32 && i < (int)boyut - 1; i++) {
        cikti[i] = karakterler[rastgele[i] % (sizeof(karakterler) - 1)];
    }
    cikti[32] = '\0';
    
    return true;
}

// ==================== INPUT VALIDASYONU ====================

bool input_ip_dogrula(const char* ip) {
    if (!ip || strlen(ip) == 0 || strlen(ip) > 45) return false;
    
    // IPv4 veya IPv6 regex
    regex_t regex;
    const char* ipv4_pattern = "^([0-9]{1,3}\\.){3}[0-9]{1,3}$";
    const char* ipv6_pattern = "^([0-9a-fA-F:]+)$";
    
    // IPv4 kontrol
    if (regcomp(&regex, ipv4_pattern, REG_EXTENDED) == 0) {
        if (regexec(&regex, ip, 0, NULL, 0) == 0) {
            regfree(&regex);
            // Octet degerlerini kontrol et
            int a, b, c, d;
            if (sscanf(ip, "%d.%d.%d.%d", &a, &b, &c, &d) == 4) {
                return (a >= 0 && a <= 255) &&
                       (b >= 0 && b <= 255) &&
                       (c >= 0 && c <= 255) &&
                       (d >= 0 && d <= 255);
            }
            return false;
        }
        regfree(&regex);
    }
    
    // IPv6 kontrol (basit)
    if (regcomp(&regex, ipv6_pattern, REG_EXTENDED) == 0) {
        int ret = regexec(&regex, ip, 0, NULL, 0);
        regfree(&regex);
        return ret == 0;
    }
    
    return false;
}

bool input_port_dogrula(int port) {
    return port > 0 && port <= 65535;
}

bool input_dosya_yolu_guvenli_mi(const char* yol) {
    if (!yol || strlen(yol) == 0 || strlen(yol) > 4096) return false;
    
    // Tehlikeli karakterler
    const char* tehlikeli[] = {"..", ";", "|", "&", "$", "`", "\\"};
    for (size_t i = 0; i < sizeof(tehlikeli)/sizeof(tehlikeli[0]); i++) {
        if (strstr(yol, tehlikeli[i]) != NULL) {
            return false;
        }
    }
    
    // Gizli dosyalar (nokta ile baslayan)
    const char* base = strrchr(yol, '/');
    if (!base) base = yol;
    else base++;
    
    if (base[0] == '.' && base[1] != '\0' && base[1] != '/') {
        return false;
    }
    
    return true;
}

bool input_vaka_adi_dogrula(const char* ad) {
    if (!ad || strlen(ad) == 0 || strlen(ad) > 128) return false;
    
    // Sadece alfanumerik, tire, alt tire ve bosluk
    regex_t regex;
    const char* pattern = "^[a-zA-Z0-9_\\- ]+$";
    
    if (regcomp(&regex, pattern, REG_EXTENDED) != 0) {
        return false;
    }
    
    int ret = regexec(&regex, ad, 0, NULL, 0);
    regfree(&regex);
    
    return ret == 0;
}

bool input_disk_id_dogrula(const char* disk_id) {
    if (!disk_id) return false;
    
    // Sayi veya "PhysicalDriveX" formati
    char* endptr;
    long id = strtol(disk_id, &endptr, 10);
    
    if (*endptr == '\0') {
        return id >= 0 && id < 100;
    }
    
    // PhysicalDriveX formati
    regex_t regex;
    const char* pattern = "^PhysicalDrive[0-9]{1,2}$";
    
    if (regcomp(&regex, pattern, REG_EXTENDED) != 0) {
        return false;
    }
    
    int ret = regexec(&regex, disk_id, 0, NULL, 0);
    regfree(&regex);
    
    return ret == 0;
}

bool input_dogrula(InputTipi tip, const char* deger) {
    switch (tip) {
        case INPUT_TIP_IP:
            return input_ip_dogrula(deger);
        case INPUT_TIP_PORT: {
            char* endptr;
            long port = strtol(deger, &endptr, 10);
            return *endptr == '\0' && input_port_dogrula((int)port);
        }
        case INPUT_TIP_DOSYA_YOLU:
            return input_dosya_yolu_guvenli_mi(deger);
        case INPUT_TIP_VAKA_ADI:
            return input_vaka_adi_dogrula(deger);
        case INPUT_TIP_TOKEN:
            return deger && strlen(deger) <= TOKEN_UZUNLUK;
        case INPUT_TIP_DISK_ID:
            return input_disk_id_dogrula(deger);
        default:
            return false;
    }
}
