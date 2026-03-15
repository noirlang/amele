#include "ayarlar.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#define VARSAYILAN_PORT 4444
#define VARSAYILAN_BOYUT_MB 100
#define VARSAYILAN_ALGILAMA_ARALIGI 3000
#define VARSAYILAN_PARCA_BOYUTU (4 * 1024 * 1024)

void ayarlar_varsayilan(UygulamaAyarlar* ayarlar) {
    if (!ayarlar) return;
    ayarlar->varsayilan_port = VARSAYILAN_PORT;
    ayarlar->varsayilan_boyut_mb = VARSAYILAN_BOYUT_MB;
    ayarlar->disk_algilama_araligi_ms = VARSAYILAN_ALGILAMA_ARALIGI;
    ayarlar->otomatik_rapor = true;
    ayarlar->karanlik_tema = false;
    strncpy(ayarlar->dil, "tr", sizeof(ayarlar->dil) - 1);
    ayarlar->dil[sizeof(ayarlar->dil) - 1] = '\0';
    strncpy(ayarlar->hash_algoritmasi, "sha256", sizeof(ayarlar->hash_algoritmasi) - 1);
    ayarlar->parca_boyutu = VARSAYILAN_PARCA_BOYUTU;

    const char* home = g_get_home_dir();
    if (!home) home = ".";
    
    ayarlar->vaka_klasoru = g_strdup_printf("%s/Worm", home);
    ayarlar->cikti_klasoru = g_strdup_printf("%s/Worm/Ciktilar", home);
}

UygulamaAyarlar* ayarlar_yukle(const char* dosya_yolu) {
    UygulamaAyarlar* ayarlar = calloc(1, sizeof(UygulamaAyarlar));
    if (!ayarlar) return NULL;

    ayarlar_varsayilan(ayarlar);

    if (!dosya_yolu || !g_file_test(dosya_yolu, G_FILE_TEST_IS_REGULAR)) {
        return ayarlar;
    }

    gchar* icerik = NULL;
    gsize uzunluk = 0;
    GError* hata = NULL;

    if (!g_file_get_contents(dosya_yolu, &icerik, &uzunluk, &hata)) {
        g_warning("Ayar dosyasi okunamadi: %s", hata->message);
        g_error_free(hata);
        return ayarlar;
    }

    JsonParser* parser = json_parser_new();
    if (json_parser_load_from_data(parser, icerik, uzunluk, &hata)) {
        JsonNode* root = json_parser_get_root(parser);
        if (root && JSON_NODE_HOLDS_OBJECT(root)) {
            JsonObject* obj = json_node_get_object(root);
            if (json_object_has_member(obj, "varsayilan_port"))
                ayarlar->varsayilan_port = json_object_get_int_member(obj, "varsayilan_port");
            if (json_object_has_member(obj, "varsayilan_boyut_mb"))
                ayarlar->varsayilan_boyut_mb = json_object_get_int_member(obj, "varsayilan_boyut_mb");
            if (json_object_has_member(obj, "disk_algilama_araligi_ms"))
                ayarlar->disk_algilama_araligi_ms = json_object_get_int_member(obj, "disk_algilama_araligi_ms");
            if (json_object_has_member(obj, "otomatik_rapor"))
                ayarlar->otomatik_rapor = json_object_get_boolean_member(obj, "otomatik_rapor");
            if (json_object_has_member(obj, "karanlik_tema"))
                ayarlar->karanlik_tema = json_object_get_boolean_member(obj, "karanlik_tema");
            if (json_object_has_member(obj, "dil")) {
                const gchar* dil = json_object_get_string_member(obj, "dil");
                strncpy(ayarlar->dil, dil, sizeof(ayarlar->dil) - 1);
                ayarlar->dil[sizeof(ayarlar->dil) - 1] = '\0';
            }
            if (json_object_has_member(obj, "hash_algoritmasi")) {
                const gchar* alg = json_object_get_string_member(obj, "hash_algoritmasi");
                strncpy(ayarlar->hash_algoritmasi, alg, sizeof(ayarlar->hash_algoritmasi) - 1);
            }
            if (json_object_has_member(obj, "vaka_klasoru")) {
                const gchar* vaka = json_object_get_string_member(obj, "vaka_klasoru");
                g_free(ayarlar->vaka_klasoru);
                ayarlar->vaka_klasoru = g_strdup(vaka);
            }
            if (json_object_has_member(obj, "cikti_klasoru")) {
                const gchar* cikti = json_object_get_string_member(obj, "cikti_klasoru");
                g_free(ayarlar->cikti_klasoru);
                ayarlar->cikti_klasoru = g_strdup(cikti);
            }
        }
    } else {
        g_warning("Ayar dosyasi parse edilemedi: %s", hata->message);
        g_error_free(hata);
    }

    g_object_unref(parser);
    g_free(icerik);

    return ayarlar;
}

void ayarlar_kaydet(UygulamaAyarlar* ayarlar, const char* dosya_yolu) {
    if (!ayarlar || !dosya_yolu) return;

    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);

    json_builder_set_member_name(builder, "varsayilan_port");
    json_builder_add_int_value(builder, ayarlar->varsayilan_port);
    json_builder_set_member_name(builder, "varsayilan_boyut_mb");
    json_builder_add_int_value(builder, ayarlar->varsayilan_boyut_mb);
    json_builder_set_member_name(builder, "disk_algilama_araligi_ms");
    json_builder_add_int_value(builder, ayarlar->disk_algilama_araligi_ms);
    json_builder_set_member_name(builder, "otomatik_rapor");
    json_builder_add_boolean_value(builder, ayarlar->otomatik_rapor);
    json_builder_set_member_name(builder, "karanlik_tema");
    json_builder_add_boolean_value(builder, ayarlar->karanlik_tema);
    json_builder_set_member_name(builder, "dil");
    json_builder_add_string_value(builder, ayarlar->dil);
    json_builder_set_member_name(builder, "hash_algoritmasi");
    json_builder_add_string_value(builder, ayarlar->hash_algoritmasi);
    json_builder_set_member_name(builder, "vaka_klasoru");
    json_builder_add_string_value(builder, ayarlar->vaka_klasoru);
    json_builder_set_member_name(builder, "cikti_klasoru");
    json_builder_add_string_value(builder, ayarlar->cikti_klasoru);

    json_builder_end_object(builder);

    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, TRUE);

    g_file_set_contents(dosya_yolu, json_str, -1, NULL);

    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);
}

void ayarlar_temizle(UygulamaAyarlar* ayarlar) {
    if (!ayarlar) return;
    g_free(ayarlar->vaka_klasoru);
    g_free(ayarlar->cikti_klasoru);
    free(ayarlar);
}

const char* ayarlar_vaka_klasoru_al(UygulamaAyarlar* ayarlar) {
    return ayarlar ? ayarlar->vaka_klasoru : NULL;
}
