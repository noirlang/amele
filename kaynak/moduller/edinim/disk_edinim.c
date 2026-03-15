#include "disk_edinim.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <openssl/evp.h>

#ifdef _WIN32
#include <windows.h>
#include <io.h>
#include <sys/stat.h>
#define strcasecmp _stricmp
#else
#include <fcntl.h>
#include <unistd.h>
#include <sys/stat.h>
#include <sys/ioctl.h>
#include <linux/fs.h>
#include <errno.h>
#include <libgen.h>
#endif

#define OKUMA_PARCA (4 * 1024 * 1024)

static volatile bool iptal_edildi = false;

static const char* dosya_adi_al(const char* yol) {
    if (!yol) return "";

    const char* slash = strrchr(yol, '/');
    const char* backslash = strrchr(yol, '\\');
    const char* son = slash;

    if (!son || (backslash && backslash > son)) {
        son = backslash;
    }

    return son ? son + 1 : yol;
}

static bool partial_yol_uret(char* hedef, size_t hedef_boyut, const char* kaynak) {
    if (!hedef || !kaynak || hedef_boyut == 0) {
        return false;
    }
    int yazilan = snprintf(hedef, hedef_boyut, "%s.partial", kaynak);
    return yazilan >= 0 && (size_t)yazilan < hedef_boyut;
}

static void yari_kalani_partiala_tasi(const char* hedef) {
    if (!hedef || hedef[0] == '\0') {
        return;
    }

    char partial[1024];
    if (!partial_yol_uret(partial, sizeof(partial), hedef)) {
        return;
    }

#ifdef _WIN32
    MoveFileExA(hedef, partial, MOVEFILE_REPLACE_EXISTING);
#else
    rename(hedef, partial);
#endif
}

int64_t disk_boyut_al(const char* cihaz) {
#ifdef _WIN32
    if (!cihaz) return -1;

    if (strncmp(cihaz, "\\\\.\\PhysicalDrive", 17) == 0) {
        HANDLE handle = CreateFileA(cihaz,
                                    GENERIC_READ,
                                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                                    NULL,
                                    OPEN_EXISTING,
                                    0,
                                    NULL);
        if (handle == INVALID_HANDLE_VALUE) {
            return -1;
        }

        GET_LENGTH_INFORMATION info;
        DWORD donen = 0;
        BOOL ok = DeviceIoControl(handle,
                                  IOCTL_DISK_GET_LENGTH_INFO,
                                  NULL,
                                  0,
                                  &info,
                                  sizeof(info),
                                  &donen,
                                  NULL);
        CloseHandle(handle);
        if (!ok) {
            return -1;
        }
        return (int64_t)info.Length.QuadPart;
    }

    struct _stat64 st;
    if (_stat64(cihaz, &st) == 0) {
        return (int64_t)st.st_size;
    }
    return -1;
#else
    int fd = open(cihaz, O_RDONLY);
    if (fd < 0) return -1;

    int64_t boyut = -1;

    struct stat st;
    if (fstat(fd, &st) == 0) {
        if (S_ISREG(st.st_mode)) {
            boyut = st.st_size;
        } else if (S_ISBLK(st.st_mode)) {
            unsigned long long blksize = 0;
            if (ioctl(fd, BLKGETSIZE64, &blksize) == 0) {
                boyut = (int64_t)blksize;
            }
        }
    }

    close(fd);
    return boyut;
#endif
}

bool disk_listele(DiskBilgisi** diskler, int* disk_sayisi) {
    if (!diskler || !disk_sayisi) return false;

    *disk_sayisi = 0;
    *diskler = NULL;

#ifdef _WIN32
    for (int i = 0; i < 32; i++) {
        char cihaz[256];
        snprintf(cihaz, sizeof(cihaz), "\\\\.\\PhysicalDrive%d", i);

        int64_t boyut = disk_boyut_al(cihaz);
        if (boyut > 0) {
            *diskler = realloc(*diskler, (*disk_sayisi + 1) * sizeof(DiskBilgisi));
            if (!*diskler) return false;

            DiskBilgisi* d = &(*diskler)[*disk_sayisi];
            strncpy(d->cihaz, cihaz, sizeof(d->cihaz) - 1);
            d->cihaz[sizeof(d->cihaz) - 1] = '\0';
            d->toplam_boyut = boyut;
            d->kullanilan_boyut = boyut;
            d->erisilebilir = true;
            (*disk_sayisi)++;
        }
    }

    return *disk_sayisi > 0;
#else

    for (int i = 0; i < 16; i++) {
        char cihaz[256];
        snprintf(cihaz, sizeof(cihaz), "/dev/sd%c", 'a' + i);

        int64_t boyut = disk_boyut_al(cihaz);
        if (boyut > 0) {
            *diskler = realloc(*diskler, (*disk_sayisi + 1) * sizeof(DiskBilgisi));
            if (!*diskler) return false;

            DiskBilgisi* d = &(*diskler)[*disk_sayisi];
            strncpy(d->cihaz, cihaz, sizeof(d->cihaz) - 1);
            d->cihaz[sizeof(d->cihaz) - 1] = '\0';
            d->toplam_boyut = boyut;
            d->kullanilan_boyut = boyut;
            d->erisilebilir = (access(cihaz, R_OK) == 0);

            (*disk_sayisi)++;
        }
    }

    for (int i = 0; i < 8; i++) {
        char cihaz[256];
        snprintf(cihaz, sizeof(cihaz), "/dev/nvme%dn1", i);

        int64_t boyut = disk_boyut_al(cihaz);
        if (boyut > 0) {
            *diskler = realloc(*diskler, (*disk_sayisi + 1) * sizeof(DiskBilgisi));
            if (!*diskler) return false;

            DiskBilgisi* d = &(*diskler)[*disk_sayisi];
            strncpy(d->cihaz, cihaz, sizeof(d->cihaz) - 1);
            d->cihaz[sizeof(d->cihaz) - 1] = '\0';
            d->toplam_boyut = boyut;
            d->kullanilan_boyut = boyut;
            d->erisilebilir = (access(cihaz, R_OK) == 0);

            (*disk_sayisi)++;
        }
    }

    for (int i = 0; i < 8; i++) {
        char cihaz[256];
        snprintf(cihaz, sizeof(cihaz), "/dev/vd%c", 'a' + i);

        int64_t boyut = disk_boyut_al(cihaz);
        if (boyut > 0) {
            *diskler = realloc(*diskler, (*disk_sayisi + 1) * sizeof(DiskBilgisi));
            if (!*diskler) return false;

            DiskBilgisi* d = &(*diskler)[*disk_sayisi];
            strncpy(d->cihaz, cihaz, sizeof(d->cihaz) - 1);
            d->cihaz[sizeof(d->cihaz) - 1] = '\0';
            d->toplam_boyut = boyut;
            d->kullanilan_boyut = boyut;
            d->erisilebilir = (access(cihaz, R_OK) == 0);

            (*disk_sayisi)++;
        }
    }

    return *disk_sayisi > 0;
#endif
}

bool disk_edinim_gorevi_calistir(DiskEdinimGorevi* gorev) {
    if (!gorev || !gorev->is) return false;

#ifdef _WIN32
    HANDLE kaynak_h = INVALID_HANDLE_VALUE;
    HANDLE hedef_h = INVALID_HANDLE_VALUE;
    EVP_MD_CTX* hash_ctx = NULL;
    unsigned char* parca = NULL;
    bool sonuc = false;
    int64_t kaynak_boyut = 0;
    int64_t kopyalanacak = 0;
    int64_t kopyalanan = 0;
    unsigned char hash[EVP_MAX_MD_SIZE];
    unsigned int hash_uzunlugu = 0;
    char hash_str[65];
    char hash_dosya[1024];
    FILE* hash_file = NULL;

    iptal_edildi = false;
    is_durum_guncelle(gorev->is, IS_DURUMU_CALISIYOR, 0);

    kaynak_h = CreateFileA(gorev->kaynak,
                           GENERIC_READ,
                           FILE_SHARE_READ | FILE_SHARE_WRITE,
                           NULL,
                           OPEN_EXISTING,
                           FILE_ATTRIBUTE_NORMAL | FILE_FLAG_SEQUENTIAL_SCAN,
                           NULL);
    if (kaynak_h == INVALID_HANDLE_VALUE) {
        is_hata(gorev->is, "Kaynak acilamadi");
        goto temizle_win;
    }

    hedef_h = CreateFileA(gorev->hedef,
                          GENERIC_WRITE,
                          FILE_SHARE_READ,
                          NULL,
                          CREATE_ALWAYS,
                          FILE_ATTRIBUTE_NORMAL,
                          NULL);
    if (hedef_h == INVALID_HANDLE_VALUE) {
        is_hata(gorev->is, "Hedef dosya olusturulamadi");
        goto temizle_win;
    }

    kaynak_boyut = disk_boyut_al(gorev->kaynak);
    if (kaynak_boyut <= 0) {
        is_hata(gorev->is, "Kaynak boyut alinamadi");
        goto temizle_win;
    }

    kopyalanacak = gorev->tam_disk ? kaynak_boyut : gorev->boyut;
    if (kopyalanacak <= 0 || kopyalanacak > kaynak_boyut) {
        kopyalanacak = kaynak_boyut;
    }

    if (gorev->hash_hesapla) {
        hash_ctx = EVP_MD_CTX_new();
        if (hash_ctx) {
            EVP_DigestInit_ex(hash_ctx, EVP_sha256(), NULL);
        }
    }

    parca = (unsigned char*)malloc(OKUMA_PARCA);
    if (!parca) {
        is_hata(gorev->is, "Bellek ayrilamadi");
        goto temizle_win;
    }

    while (kopyalanan < kopyalanacak && !iptal_edildi) {
        DWORD okunacak = (DWORD)((kopyalanacak - kopyalanan) > OKUMA_PARCA ?
                                 OKUMA_PARCA : (kopyalanacak - kopyalanan));
        DWORD okunan = 0;
        DWORD yazilan = 0;

        if (!ReadFile(kaynak_h, parca, okunacak, &okunan, NULL) || okunan == 0) {
            break;
        }

        if (!WriteFile(hedef_h, parca, okunan, &yazilan, NULL) || yazilan != okunan) {
            is_hata(gorev->is, "Yazma hatasi");
            goto temizle_win;
        }

        if (hash_ctx) {
            EVP_DigestUpdate(hash_ctx, parca, okunan);
        }

        kopyalanan += okunan;
        is_durum_guncelle(gorev->is, IS_DURUMU_CALISIYOR,
                          (int)((kopyalanan * 100) / kopyalanacak));
    }

    if (iptal_edildi) {
        is_durum_guncelle(gorev->is, IS_DURUMU_IPTAL_EDILDI, -1);
        yari_kalani_partiala_tasi(gorev->hedef);
        goto temizle_win;
    }

    if (kopyalanan < kopyalanacak) {
        is_hata(gorev->is, "Okuma yarida kesildi");
        yari_kalani_partiala_tasi(gorev->hedef);
        goto temizle_win;
    }

    if (hash_ctx) {
        EVP_DigestFinal_ex(hash_ctx, hash, &hash_uzunlugu);
        for (unsigned int i = 0; i < hash_uzunlugu; i++) {
            snprintf(hash_str + (i * 2), 3, "%02x", hash[i]);
        }
        hash_str[hash_uzunlugu * 2] = '\0';

        snprintf(hash_dosya, sizeof(hash_dosya), "%s.sha256", gorev->hedef);
        hash_file = fopen(hash_dosya, "w");
        if (hash_file) {
            fprintf(hash_file, "%s  %s\n", hash_str, dosya_adi_al(gorev->hedef));
            fclose(hash_file);
        }
    }

    is_durum_guncelle(gorev->is, IS_DURUMU_TAMAMLANDI, 100);
    sonuc = true;

temizle_win:
    if (parca) free(parca);
    if (kaynak_h != INVALID_HANDLE_VALUE) CloseHandle(kaynak_h);
    if (hedef_h != INVALID_HANDLE_VALUE) CloseHandle(hedef_h);
    if (hash_ctx) EVP_MD_CTX_free(hash_ctx);
    if (!sonuc) {
        yari_kalani_partiala_tasi(gorev->hedef);
    }
    return sonuc;
#else

    int kaynak_fd = -1;
    int hedef_fd = -1;
    EVP_MD_CTX* hash_ctx = NULL;
    void* parca = NULL;
    bool sonuc = false;
    int64_t kaynak_boyut, kopyalanacak, kopyalanan, okunacak;
    ssize_t okunan, yazilan;
    unsigned int i;
    int yuzde;
    unsigned char hash[EVP_MAX_MD_SIZE];
    unsigned int hash_uzunlugu;
    char hash_str[65];
    char hash_dosya[1024];
    FILE* hash_file;

    iptal_edildi = false;
    is_durum_guncelle(gorev->is, IS_DURUMU_CALISIYOR, 0);

    kaynak_fd = open(gorev->kaynak, O_RDONLY | O_DIRECT);
    if (kaynak_fd < 0) {
        kaynak_fd = open(gorev->kaynak, O_RDONLY);
        if (kaynak_fd < 0) {
            is_hata(gorev->is, "Kaynak acilamadi");
            goto temizle;
        }
    }

    hedef_fd = open(gorev->hedef, O_WRONLY | O_CREAT | O_TRUNC, 0644);
    if (hedef_fd < 0) {
        is_hata(gorev->is, "Hedef dosya olusturulamadi");
        goto temizle;
    }

    kaynak_boyut = disk_boyut_al(gorev->kaynak);
    if (kaynak_boyut < 0) {
        is_hata(gorev->is, "Kaynak boyut alinamadi");
        goto temizle;
    }

    kopyalanacak = gorev->tam_disk ? kaynak_boyut : gorev->boyut;
    if (kopyalanacak > kaynak_boyut) kopyalanacak = kaynak_boyut;

    if (gorev->hash_hesapla) {
        hash_ctx = EVP_MD_CTX_new();
        if (hash_ctx) {
            EVP_DigestInit_ex(hash_ctx, EVP_sha256(), NULL);
        }
    }

    parca = aligned_alloc(512, OKUMA_PARCA);
    if (!parca) {
        is_hata(gorev->is, "Bellek ayrilamadi");
        goto temizle;
    }

    kopyalanan = 0;
    while (kopyalanan < kopyalanacak && !iptal_edildi) {
        okunacak = OKUMA_PARCA;
        if (kopyalanan + okunacak > kopyalanacak) {
            okunacak = kopyalanacak - kopyalanan;
        }

        okunan = read(kaynak_fd, parca, okunacak);
        if (okunan <= 0) break;

        yazilan = write(hedef_fd, parca, okunan);
        if (yazilan != okunan) {
            is_hata(gorev->is, "Yazma hatasi");
            break;
        }

        if (hash_ctx) {
            EVP_DigestUpdate(hash_ctx, parca, okunan);
        }

        kopyalanan += okunan;
        yuzde = (int)((kopyalanan * 100) / kopyalanacak);
        is_durum_guncelle(gorev->is, IS_DURUMU_CALISIYOR, yuzde);
    }

    if (iptal_edildi) {
        is_durum_guncelle(gorev->is, IS_DURUMU_IPTAL_EDILDI, -1);
        yari_kalani_partiala_tasi(gorev->hedef);
        goto temizle;
    }

    if (kopyalanan < kopyalanacak) {
        is_hata(gorev->is, "Okuma yarida kesildi");
        yari_kalani_partiala_tasi(gorev->hedef);
        goto temizle;
    }

    if (hash_ctx) {
        EVP_DigestFinal_ex(hash_ctx, hash, &hash_uzunlugu);

        for (i = 0; i < hash_uzunlugu; i++) {
            snprintf(hash_str + (i * 2), 3, "%02x", hash[i]);
        }
        hash_str[hash_uzunlugu * 2] = '\0';

        snprintf(hash_dosya, sizeof(hash_dosya), "%s.sha256", gorev->hedef);
        hash_file = fopen(hash_dosya, "w");
        if (hash_file) {
            fprintf(hash_file, "%s  %s\n", hash_str, dosya_adi_al(gorev->hedef));
            fclose(hash_file);
        }
    }

    is_durum_guncelle(gorev->is, IS_DURUMU_TAMAMLANDI, 100);
    sonuc = true;

temizle:
    if (parca) free(parca);
    if (kaynak_fd >= 0) close(kaynak_fd);
    if (hedef_fd >= 0) close(hedef_fd);
    if (hash_ctx) EVP_MD_CTX_free(hash_ctx);
    if (!sonuc) {
        yari_kalani_partiala_tasi(gorev->hedef);
    }
    
    return sonuc;
#endif
}

bool disk_imaj_dogrula(const char* imaj_yolu, const char* beklenen_hash) {
    if (!imaj_yolu || !beklenen_hash) return false;

    EVP_MD_CTX* ctx = EVP_MD_CTX_new();
    if (!ctx) return false;

    if (EVP_DigestInit_ex(ctx, EVP_sha256(), NULL) != 1) {
        EVP_MD_CTX_free(ctx);
        return false;
    }

    FILE* f = fopen(imaj_yolu, "rb");
    if (!f) {
        EVP_MD_CTX_free(ctx);
        return false;
    }

    unsigned char parca[1024 * 1024];
    size_t okunan;
    while ((okunan = fread(parca, 1, sizeof(parca), f)) > 0) {
        EVP_DigestUpdate(ctx, parca, okunan);
    }

    fclose(f);

    unsigned char hash[EVP_MAX_MD_SIZE];
    unsigned int hash_uzunlugu;
    if (EVP_DigestFinal_ex(ctx, hash, &hash_uzunlugu) != 1) {
        EVP_MD_CTX_free(ctx);
        return false;
    }

    EVP_MD_CTX_free(ctx);

    char hesaplanan_hash[65];
    for (unsigned int i = 0; i < hash_uzunlugu; i++) {
        snprintf(hesaplanan_hash + (i * 2), 3, "%02x", hash[i]);
    }
    hesaplanan_hash[hash_uzunlugu * 2] = '\0';

    return strcasecmp(hesaplanan_hash, beklenen_hash) == 0;
}

bool disk_edinim_iptal(void) {
    iptal_edildi = true;
    return true;
}
