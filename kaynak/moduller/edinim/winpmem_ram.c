#include "winpmem_ram.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <time.h>

#ifdef _WIN32
#include <windows.h>
#include <io.h>
#include <sys/stat.h>
#define WORM_ACCESS _access
#define WORM_POPEN _popen
#define WORM_PCLOSE _pclose
typedef struct _stat64 worm_stat_t;
#define WORM_STAT _stat64
#ifndef X_OK
#define X_OK 0
#endif
#ifndef F_OK
#define F_OK 0
#endif
#ifndef R_OK
#define R_OK 4
#endif
#else
#include <unistd.h>
#include <sys/stat.h>
#include <sys/sysinfo.h>
#define WORM_ACCESS access
#define WORM_POPEN popen
#define WORM_PCLOSE pclose
typedef struct stat worm_stat_t;
#define WORM_STAT stat
#endif

WinPMEMEdinim* winpmem_edinim_olustur(void) {
    WinPMEMEdinim* edinim = calloc(1, sizeof(WinPMEMEdinim));
    if (!edinim) return NULL;
    
    g_mutex_init(&edinim->kilit);
    edinim->calisiyor = false;
    edinim->yonetici_yetkisi = false;
    edinim->toplam_boyut = 0;
    edinim->okunan_boyut = 0;
    
    // Varsayilan WinPMEM yolu
    strncpy(edinim->winpmem_yolu, "winpmem_mini_x64.exe", sizeof(edinim->winpmem_yolu) - 1);
    
    return edinim;
}

void winpmem_edinim_yok_et(WinPMEMEdinim* edinim) {
    if (!edinim) return;
    
    g_mutex_lock(&edinim->kilit);
    if (edinim->calisiyor) {
        edinim->calisiyor = false;
    }
    g_mutex_unlock(&edinim->kilit);
    
    g_mutex_clear(&edinim->kilit);
    free(edinim);
}

bool winpmem_binary_kontrol(WinPMEMEdinim* edinim, const char* yol) {
    if (!edinim || !yol) return false;
    
    if (WORM_ACCESS(yol, X_OK) != 0 && WORM_ACCESS(yol, F_OK) != 0) {
        return false;
    }
    
    strncpy(edinim->winpmem_yolu, yol, sizeof(edinim->winpmem_yolu) - 1);
    return true;
}

bool winpmem_yonetici_yetkisi_kontrol(void) {
#ifdef _WIN32
    BOOL isAdmin = FALSE;
    PSID administratorsGroup = NULL;
    SID_IDENTIFIER_AUTHORITY NtAuthority = SECURITY_NT_AUTHORITY;
    
    if (AllocateAndInitializeSid(&NtAuthority, 2, SECURITY_BUILTIN_DOMAIN_RID,
                                 DOMAIN_ALIAS_RID_ADMINS, 0, 0, 0, 0, 0, 0,
                                 &administratorsGroup)) {
        CheckTokenMembership(NULL, administratorsGroup, &isAdmin);
        FreeSid(administratorsGroup);
    }
    
    return isAdmin == TRUE;
#else
    // Linux uzerinde her zaman true dondur (test modu)
    return (getuid() == 0);
#endif
}

int64_t winpmem_ram_boyut_al(void) {
#ifdef _WIN32
    MEMORYSTATUSEX memInfo;
    memInfo.dwLength = sizeof(MEMORYSTATUSEX);
    
    if (GlobalMemoryStatusEx(&memInfo)) {
        return (int64_t)memInfo.ullTotalPhys;
    }
    return 0;
#else
    // Linux uzerinde sysinfo kullan
    struct sysinfo info;
    if (sysinfo(&info) == 0) {
        return (int64_t)info.totalram;
    }
    return 0;
#endif
}

bool winpmem_ram_al(WinPMEMEdinim* edinim, 
                    const char* cikti_dosyasi,
                    WinPMEMIlerlemeCallback ilerleme_callback,
                    void* kullanici_verisi,
                    HataKodu* hata_kodu) {
    if (!edinim || !cikti_dosyasi) {
        if (hata_kodu) *hata_kodu = HATA_GENEL;
        return false;
    }
    
    // Yonetici yetkisi kontrolu
    if (!winpmem_yonetici_yetkisi_kontrol()) {
        if (hata_kodu) *hata_kodu = HATA_YETKISIZ_ERISIM;
        return false;
    }
    
    // WinPMEM binary kontrolu
    if (WORM_ACCESS(edinim->winpmem_yolu, X_OK) != 0 && WORM_ACCESS(edinim->winpmem_yolu, F_OK) != 0) {
        if (hata_kodu) *hata_kodu = HATA_DOSYA_ACILAMADI;
        return false;
    }
    
    g_mutex_lock(&edinim->kilit);
    
    if (edinim->calisiyor) {
        g_mutex_unlock(&edinim->kilit);
        if (hata_kodu) *hata_kodu = HATA_GENEL;
        return false;
    }
    
    strncpy(edinim->cikti_dosyasi, cikti_dosyasi, sizeof(edinim->cikti_dosyasi) - 1);
    edinim->toplam_boyut = winpmem_ram_boyut_al();
    edinim->okunan_boyut = 0;
    edinim->calisiyor = true;
    edinim->yonetici_yetkisi = true;
    
    g_mutex_unlock(&edinim->kilit);
    
    // WinPMEM komutunu olustur
    // Format: winpmem_mini_x64.exe -o output.raw -1
    // -1 flag'i tam bellek imajı alır
    char komut[1024];
    snprintf(komut, sizeof(komut), "\"%s\" -o \"%s\" -1",
             edinim->winpmem_yolu, cikti_dosyasi);
    
    // WinPMEM'i calistir
    FILE* pipe = WORM_POPEN(komut, "r");
    if (!pipe) {
        g_mutex_lock(&edinim->kilit);
        edinim->calisiyor = false;
        g_mutex_unlock(&edinim->kilit);
        if (hata_kodu) *hata_kodu = HATA_GENEL;
        return false;
    }
    
    // Cikti dosyasini izle ve ilerlemeyi guncelle
    // WinPMEM ciktiyi direkt dosyaya yazar, biz dosya boyutunu izleriz
    time_t baslangic = time(NULL);
    int timeout_saniye = 3600; // 1 saat timeout
    
    while (edinim->calisiyor) {
        // Dosya boyutunu kontrol et
        worm_stat_t st;
        if (WORM_STAT(cikti_dosyasi, &st) == 0) {
            g_mutex_lock(&edinim->kilit);
            edinim->okunan_boyut = st.st_size;
            g_mutex_unlock(&edinim->kilit);
            
            if (ilerleme_callback) {
                ilerleme_callback(edinim->okunan_boyut, edinim->toplam_boyut, kullanici_verisi);
            }
            
            // Eger dosya boyutu RAM boyutuna ulastiysa veya daha buyukse bitir
            if (st.st_size >= edinim->toplam_boyut) {
                break;
            }
        }
        
        // Timeout kontrolu
        if (time(NULL) - baslangic > timeout_saniye) {
            break;
        }
        
        // 1 saniye bekle
        g_usleep(1000000);
    }
    
    WORM_PCLOSE(pipe);
    
    g_mutex_lock(&edinim->kilit);
    edinim->calisiyor = false;
    g_mutex_unlock(&edinim->kilit);
    
    // Sonuc kontrolu
    worm_stat_t st;
    if (WORM_STAT(cikti_dosyasi, &st) == 0 && st.st_size > 0) {
        if (hata_kodu) *hata_kodu = HATA_OK;
        return true;
    } else {
        if (hata_kodu) *hata_kodu = HATA_DOSYA_YAZMA;
        return false;
    }
}

bool winpmem_calisiyor_mu(WinPMEMEdinim* edinim) {
    if (!edinim) return false;
    
    bool durum;
    g_mutex_lock(&edinim->kilit);
    durum = edinim->calisiyor;
    g_mutex_unlock(&edinim->kilit);
    
    return durum;
}

int64_t winpmem_ilerleme_al(WinPMEMEdinim* edinim) {
    if (!edinim) return 0;
    
    int64_t ilerleme;
    g_mutex_lock(&edinim->kilit);
    ilerleme = edinim->okunan_boyut;
    g_mutex_unlock(&edinim->kilit);
    
    return ilerleme;
}

void winpmem_iptal_et(WinPMEMEdinim* edinim) {
    if (!edinim) return;
    
    g_mutex_lock(&edinim->kilit);
    edinim->calisiyor = false;
    g_mutex_unlock(&edinim->kilit);
}

bool winpmem_volatility3_hazirlik(const char* ram_dosyasi, 
                                  const char* sembol_dosyasi,
                                  HataKodu* hata_kodu) {
    if (!ram_dosyasi) {
        if (hata_kodu) *hata_kodu = HATA_GENEL;
        return false;
    }
    
    // RAM dosyasinin varligini kontrol et
    if (WORM_ACCESS(ram_dosyasi, R_OK) != 0) {
        if (hata_kodu) *hata_kodu = HATA_DOSYA_ACILAMADI;
        return false;
    }
    
    // Volatility3 konfigurasyon dosyasi olustur (opsiyonel)
    if (sembol_dosyasi) {
        // Sembol dosyasi kontrolu
        if (WORM_ACCESS(sembol_dosyasi, R_OK) != 0) {
            if (hata_kodu) *hata_kodu = HATA_DOSYA_ACILAMADI;
            return false;
        }
    }
    
    if (hata_kodu) *hata_kodu = HATA_OK;
    return true;
}
