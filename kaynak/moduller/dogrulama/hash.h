#ifndef HASH_H
#define HASH_H

#include <stdbool.h>
#include <stdint.h>
#include "is_kuyrugu.h"

typedef enum {
    HASH_MD5,
    HASH_SHA1,
    HASH_SHA256,
    HASH_SHA512
} HashAlgoritma;

typedef struct {
    HashAlgoritma algoritma;
    char deger[129];
} HashSonuc;

typedef struct {
    IsGorevi* is;
    char dosya_yolu[1024];
    HashAlgoritma algoritmalar[4];
    int algoritma_sayisi;
} HashHesaplamaGorevi;

bool hash_dosya_hesapla(const char* dosya_yolu, HashAlgoritma algo, char* cikti, size_t cikti_boyutu);
bool hash_coklu_hesapla(const char* dosya_yolu, HashSonuc* sonuclar, int* sonuc_sayisi);
const char* hash_algoritma_isim(HashAlgoritma algo);
bool hash_karsilastir(const char* hash1, const char* hash2);
void hash_gorevi_calistir(HashHesaplamaGorevi* gorev);

#endif
