#include "avml_ram.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <signal.h>
#include <unistd.h>
#include <sys/stat.h>
#include <sys/sysinfo.h>
#include <sys/types.h>
#include <sys/wait.h>
#endif

AVMLEdinim* avml_edinim_olustur(void) {
    AVMLEdinim* edinim = calloc(1, sizeof(AVMLEdinim));
    if (!edinim) {
        return NULL;
    }

    g_mutex_init(&edinim->kilit);
    strncpy(edinim->avml_yolu, "avml", sizeof(edinim->avml_yolu) - 1);
    return edinim;
}

void avml_edinim_yok_et(AVMLEdinim* edinim) {
    if (!edinim) {
        return;
    }

    g_mutex_lock(&edinim->kilit);
    edinim->calisiyor = false;
    g_mutex_unlock(&edinim->kilit);

    g_mutex_clear(&edinim->kilit);
    free(edinim);
}

bool avml_binary_kontrol(AVMLEdinim* edinim, const char* yol) {
    if (!edinim) {
        return false;
    }

#ifdef _WIN32
    (void)yol;
    return false;
#else
    if (yol && yol[0] != '\0') {
        if (access(yol, X_OK) == 0 || access(yol, F_OK) == 0) {
            strncpy(edinim->avml_yolu, yol, sizeof(edinim->avml_yolu) - 1);
            return true;
        }
    }

    char* bulunan = g_find_program_in_path("avml");
    if (bulunan) {
        strncpy(edinim->avml_yolu, bulunan, sizeof(edinim->avml_yolu) - 1);
        g_free(bulunan);
        return true;
    }

    const char* adaylar[] = {
        "/usr/bin/avml",
        "/usr/local/bin/avml",
        NULL,
    };

    for (int i = 0; adaylar[i] != NULL; ++i) {
        if (access(adaylar[i], X_OK) == 0 || access(adaylar[i], F_OK) == 0) {
            strncpy(edinim->avml_yolu, adaylar[i], sizeof(edinim->avml_yolu) - 1);
            return true;
        }
    }

    return false;
#endif
}

bool avml_root_yetkisi_kontrol(void) {
#ifdef _WIN32
    return false;
#else
    return getuid() == 0;
#endif
}

int64_t avml_ram_boyut_al(void) {
#ifdef _WIN32
    return 0;
#else
    struct sysinfo info;
    if (sysinfo(&info) != 0) {
        return 0;
    }
    return (int64_t)info.totalram;
#endif
}

bool avml_ram_al(AVMLEdinim* edinim,
                 const char* cikti_dosyasi,
                 AVMLIlerlemeCallback ilerleme_callback,
                 void* kullanici_verisi,
                 HataKodu* hata_kodu) {
    if (!edinim || !cikti_dosyasi) {
        if (hata_kodu) {
            *hata_kodu = HATA_GENEL;
        }
        return false;
    }

#ifdef _WIN32
    (void)ilerleme_callback;
    (void)kullanici_verisi;
    if (hata_kodu) {
        *hata_kodu = HATA_YETKISIZ_ERISIM;
    }
    return false;
#else
    if (!avml_root_yetkisi_kontrol()) {
        if (hata_kodu) {
            *hata_kodu = HATA_YETKISIZ_ERISIM;
        }
        return false;
    }

    if (!avml_binary_kontrol(edinim, edinim->avml_yolu)) {
        if (hata_kodu) {
            *hata_kodu = HATA_DOSYA_ACILAMADI;
        }
        return false;
    }

    g_mutex_lock(&edinim->kilit);
    if (edinim->calisiyor) {
        g_mutex_unlock(&edinim->kilit);
        if (hata_kodu) {
            *hata_kodu = HATA_GENEL;
        }
        return false;
    }

    strncpy(edinim->cikti_dosyasi, cikti_dosyasi, sizeof(edinim->cikti_dosyasi) - 1);
    edinim->toplam_boyut = avml_ram_boyut_al();
    edinim->okunan_boyut = 0;
    edinim->calisiyor = true;
    g_mutex_unlock(&edinim->kilit);

    pid_t pid = fork();
    if (pid < 0) {
        g_mutex_lock(&edinim->kilit);
        edinim->calisiyor = false;
        g_mutex_unlock(&edinim->kilit);
        if (hata_kodu) {
            *hata_kodu = HATA_GENEL;
        }
        return false;
    }

    if (pid == 0) {
        execl(edinim->avml_yolu, edinim->avml_yolu, cikti_dosyasi, (char*)NULL);
        _exit(127);
    }

    time_t baslangic = time(NULL);
    const int timeout_saniye = 7200;
    int process_status = 0;
    bool process_bitti = false;

    while (true) {
        g_mutex_lock(&edinim->kilit);
        bool aktif = edinim->calisiyor;
        g_mutex_unlock(&edinim->kilit);
        if (!aktif) {
            kill(pid, SIGTERM);
            waitpid(pid, &process_status, 0);
            break;
        }

        struct stat st;
        if (stat(cikti_dosyasi, &st) == 0) {
            g_mutex_lock(&edinim->kilit);
            edinim->okunan_boyut = st.st_size;
            int64_t toplam = edinim->toplam_boyut;
            g_mutex_unlock(&edinim->kilit);

            if (ilerleme_callback && toplam > 0) {
                ilerleme_callback(st.st_size, toplam, kullanici_verisi);
            }
        }

        pid_t w = waitpid(pid, &process_status, WNOHANG);
        if (w == pid) {
            process_bitti = true;
            break;
        }

        if (time(NULL) - baslangic > timeout_saniye) {
            kill(pid, SIGTERM);
            waitpid(pid, &process_status, 0);
            break;
        }

        g_usleep(1000000);
    }

    g_mutex_lock(&edinim->kilit);
    edinim->calisiyor = false;
    g_mutex_unlock(&edinim->kilit);

    struct stat st;
    if (process_bitti && WIFEXITED(process_status) && WEXITSTATUS(process_status) == 0 &&
        stat(cikti_dosyasi, &st) == 0 && st.st_size > 0) {
        if (hata_kodu) {
            *hata_kodu = HATA_OK;
        }
        return true;
    }

    if (hata_kodu) {
        *hata_kodu = HATA_DOSYA_YAZMA;
    }
    return false;
#endif
}

bool avml_calisiyor_mu(AVMLEdinim* edinim) {
    if (!edinim) {
        return false;
    }

    g_mutex_lock(&edinim->kilit);
    bool sonuc = edinim->calisiyor;
    g_mutex_unlock(&edinim->kilit);
    return sonuc;
}

void avml_iptal_et(AVMLEdinim* edinim) {
    if (!edinim) {
        return;
    }

    g_mutex_lock(&edinim->kilit);
    edinim->calisiyor = false;
    g_mutex_unlock(&edinim->kilit);
}
