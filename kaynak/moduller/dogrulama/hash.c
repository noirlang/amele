#include "hash.h"
#include <openssl/evp.h>
#include <openssl/md5.h>
#include <openssl/sha.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define PARCA_BOYUTU (1024 * 1024)

const char* hash_algoritma_isim(HashAlgoritma algo) {
    switch (algo) {
        case HASH_MD5:    return "MD5";
        case HASH_SHA1:   return "SHA1";
        case HASH_SHA256: return "SHA256";
        case HASH_SHA512: return "SHA512";
        default: return "Bilinmiyor";
    }
}

bool hash_dosya_hesapla(const char* dosya_yolu, HashAlgoritma algo, char* cikti, size_t cikti_boyutu) {
    if (!dosya_yolu || !cikti || cikti_boyutu < 65) return false;

    FILE* f = fopen(dosya_yolu, "rb");
    if (!f) return false;

    EVP_MD_CTX* ctx = EVP_MD_CTX_new();
    if (!ctx) {
        fclose(f);
        return false;
    }

    const EVP_MD* md = NULL;
    switch (algo) {
        case HASH_MD5:    md = EVP_md5(); break;
        case HASH_SHA1:   md = EVP_sha1(); break;
        case HASH_SHA256: md = EVP_sha256(); break;
        case HASH_SHA512: md = EVP_sha512(); break;
        default: 
            EVP_MD_CTX_free(ctx);
            fclose(f);
            return false;
    }

    if (EVP_DigestInit_ex(ctx, md, NULL) != 1) {
        EVP_MD_CTX_free(ctx);
        fclose(f);
        return false;
    }

    unsigned char parca[PARCA_BOYUTU];
    size_t okunan;
    while ((okunan = fread(parca, 1, PARCA_BOYUTU, f)) > 0) {
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

    for (unsigned int i = 0; i < hash_uzunlugu; i++) {
        snprintf(cikti + (i * 2), 3, "%02x", hash[i]);
    }
    cikti[hash_uzunlugu * 2] = '\0';

    return true;
}

bool hash_coklu_hesapla(const char* dosya_yolu, HashSonuc* sonuclar, int* sonuc_sayisi) {
    if (!dosya_yolu || !sonuclar || !sonuc_sayisi || *sonuc_sayisi == 0) return false;

    FILE* f = fopen(dosya_yolu, "rb");
    if (!f) return false;

    EVP_MD_CTX* ctxs[4] = {NULL};
    const EVP_MD* mds[4] = {NULL};
    int ctx_sayisi = *sonuc_sayisi;

    for (int i = 0; i < ctx_sayisi && i < 4; i++) {
        ctxs[i] = EVP_MD_CTX_new();
        if (!ctxs[i]) {
            for (int j = 0; j < i; j++) {
                if (ctxs[j]) EVP_MD_CTX_free(ctxs[j]);
            }
            fclose(f);
            return false;
        }

        switch (sonuclar[i].algoritma) {
            case HASH_MD5:    mds[i] = EVP_md5(); break;
            case HASH_SHA1:   mds[i] = EVP_sha1(); break;
            case HASH_SHA256: mds[i] = EVP_sha256(); break;
            case HASH_SHA512: mds[i] = EVP_sha512(); break;
            default: mds[i] = EVP_sha256();
        }

        if (EVP_DigestInit_ex(ctxs[i], mds[i], NULL) != 1) {
            for (int j = 0; j <= i; j++) {
                if (ctxs[j]) EVP_MD_CTX_free(ctxs[j]);
            }
            fclose(f);
            return false;
        }
    }

    unsigned char parca[PARCA_BOYUTU];
    size_t okunan;
    while ((okunan = fread(parca, 1, PARCA_BOYUTU, f)) > 0) {
        for (int i = 0; i < ctx_sayisi; i++) {
            EVP_DigestUpdate(ctxs[i], parca, okunan);
        }
    }

    fclose(f);

    for (int i = 0; i < ctx_sayisi; i++) {
        unsigned char hash[EVP_MAX_MD_SIZE];
        unsigned int hash_uzunlugu;
        if (EVP_DigestFinal_ex(ctxs[i], hash, &hash_uzunlugu) == 1) {
            for (unsigned int j = 0; j < hash_uzunlugu; j++) {
                snprintf(sonuclar[i].deger + (j * 2), 3, "%02x", hash[j]);
            }
            sonuclar[i].deger[hash_uzunlugu * 2] = '\0';
        }
        EVP_MD_CTX_free(ctxs[i]);
    }

    return true;
}

bool hash_karsilastir(const char* hash1, const char* hash2) {
    if (!hash1 || !hash2) return false;
    return strcasecmp(hash1, hash2) == 0;
}

void hash_gorevi_calistir(HashHesaplamaGorevi* gorev) {
    if (!gorev || !gorev->is) return;

    is_durum_guncelle(gorev->is, IS_DURUMU_CALISIYOR, 0);

    HashSonuc sonuclar[4];
    int sonuc_sayisi = gorev->algoritma_sayisi;
    for (int i = 0; i < sonuc_sayisi; i++) {
        sonuclar[i].algoritma = gorev->algoritmalar[i];
        sonuclar[i].deger[0] = '\0';
    }

    if (hash_coklu_hesapla(gorev->dosya_yolu, sonuclar, &sonuc_sayisi)) {
        for (int i = 0; i < sonuc_sayisi; i++) {
            is_urun_dosya_ekle(gorev->is, sonuclar[i].deger);
        }
        is_tamamla(gorev->is, NULL);
    } else {
        is_hata(gorev->is, "Hash hesaplama basarisiz");
    }
}
