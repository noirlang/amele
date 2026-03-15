#include "rapor.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <sys/stat.h>
#include <json-glib/json-glib.h>

#ifdef _WIN32
#include <windows.h>
#else
#include <sys/utsname.h>
#include <unistd.h>
#endif

static const char* rapor_kullanici_adi(void) {
    const char* kullanici = g_get_user_name();
    return (kullanici && *kullanici) ? kullanici : "bilinmiyor";
}

static int rapor_pid_al(void) {
#ifdef _WIN32
    return (int)GetCurrentProcessId();
#else
    return (int)getpid();
#endif
}

char* rapor_yeni_dosya_adi(const char* vaka_adi, RaporFormat format) {
    if (!vaka_adi) return NULL;

    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);

    const char* uzanti = (format == RAPOR_FORMAT_JSON) ? "json" : "txt";
    char* dosya_adi = g_strdup_printf("rapor_%s_%04d%02d%02d_%02d%02d%02d.%s",
                                       vaka_adi,
                                       tm_info->tm_year + 1900,
                                       tm_info->tm_mon + 1,
                                       tm_info->tm_mday,
                                       tm_info->tm_hour,
                                       tm_info->tm_min,
                                       tm_info->tm_sec,
                                       uzanti);
    return dosya_adi;
}

bool rapor_dosya_ozet(const char* dosya_yolu, char* ozet, size_t ozet_boyutu) {
    if (!dosya_yolu || !ozet || ozet_boyutu == 0) return false;

    struct stat st;
    if (stat(dosya_yolu, &st) != 0) {
        snprintf(ozet, ozet_boyutu, "Dosya bulunamadi: %s", dosya_yolu);
        return false;
    }

    const char* dosya_adi = g_path_get_basename(dosya_yolu);

    char zaman_str[64];
    strftime(zaman_str, sizeof(zaman_str), "%Y-%m-%d %H:%M:%S",
             localtime(&st.st_mtime));

    const char* boyut_birim;
    double boyut_deger;
    if (st.st_size >= 1024 * 1024 * 1024) {
        boyut_deger = (double)st.st_size / (1024 * 1024 * 1024);
        boyut_birim = "GB";
    } else if (st.st_size >= 1024 * 1024) {
        boyut_deger = (double)st.st_size / (1024 * 1024);
        boyut_birim = "MB";
    } else if (st.st_size >= 1024) {
        boyut_deger = (double)st.st_size / 1024;
        boyut_birim = "KB";
    } else {
        boyut_deger = (double)st.st_size;
        boyut_birim = "B";
    }

    snprintf(ozet, ozet_boyutu,
             "Dosya: %s\n"
             "Boyut: %.2f %s (%ld bayt)\n"
             "Degistirme: %s\n",
             dosya_adi, boyut_deger, boyut_birim, (long)st.st_size, zaman_str);

    g_free((char*)dosya_adi);
    return true;
}

bool rapor_sistem_bilgisi_ekle(const char* hedef_dosya) {
    if (!hedef_dosya) return false;

    FILE* f = fopen(hedef_dosya, "a");
    if (!f) return false;

    fprintf(f, "\n========================================\n");
    fprintf(f, "SISTEM BILGISI\n");
    fprintf(f, "========================================\n");
#ifdef _WIN32
    fprintf(f, "Isletim Sistemi: Windows\n");
    fprintf(f, "Hostname: %s\n", g_get_host_name());
#else
    struct utsname uts;
    if (uname(&uts) == 0) {
        fprintf(f, "Isletim Sistemi: %s\n", uts.sysname);
        fprintf(f, "Surum: %s\n", uts.release);
        fprintf(f, "Makine: %s\n", uts.machine);
        fprintf(f, "Hostname: %s\n", uts.nodename);
    }
#endif

    fprintf(f, "\nKullanici: %s\n", rapor_kullanici_adi());
    fprintf(f, "PID: %d\n", rapor_pid_al());

    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    char zaman_str[64];
    strftime(zaman_str, sizeof(zaman_str), "%Y-%m-%d %H:%M:%S", tm_info);
    fprintf(f, "Rapor Tarihi: %s\n", zaman_str);

    fclose(f);
    return true;
}

static bool rapor_txt_olustur(RaporBilgisi* bilgi, const char* hedef_dosya) {
    FILE* f = fopen(hedef_dosya, "w");
    if (!f) return false;

    fprintf(f, "========================================\n");
    fprintf(f, "    ADLI BILISIM TEKNIK RAPORU\n");
    fprintf(f, "========================================\n\n");

    fprintf(f, "BASLIK: %s\n", bilgi->baslik);
    fprintf(f, "ACIKLAMA: %s\n", bilgi->aciklama);
    fprintf(f, "OLUSTURAN: %s\n", bilgi->olusturan);
    fprintf(f, "KAYNAK: %s\n", bilgi->kaynak);
    fprintf(f, "TARIH: %s\n", bilgi->tarih);

    if (strlen(bilgi->hash) > 0) {
        fprintf(f, "\n----------------------------------------\n");
        fprintf(f, "HASH DEGERI (SHA-256):\n");
        fprintf(f, "%s\n", bilgi->hash);
        fprintf(f, "----------------------------------------\n");
    }

    fprintf(f, "\n========================================\n");
    fprintf(f, "Sistem tarafindan olusturulmustur.\n");
    fprintf(f, "========================================\n");

    fclose(f);
    return true;
}

static bool rapor_json_olustur(RaporBilgisi* bilgi, const char* hedef_dosya) {
    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);

    json_builder_set_member_name(builder, "tur");
    json_builder_add_string_value(builder, "adli_bilisim_raporu");
    json_builder_set_member_name(builder, "versiyon");
    json_builder_add_string_value(builder, "1.0");

    json_builder_set_member_name(builder, "baslik");
    json_builder_add_string_value(builder, bilgi->baslik);
    json_builder_set_member_name(builder, "aciklama");
    json_builder_add_string_value(builder, bilgi->aciklama);
    json_builder_set_member_name(builder, "olusturan");
    json_builder_add_string_value(builder, bilgi->olusturan);
    json_builder_set_member_name(builder, "kaynak");
    json_builder_add_string_value(builder, bilgi->kaynak);
    json_builder_set_member_name(builder, "tarih");
    json_builder_add_string_value(builder, bilgi->tarih);
    json_builder_set_member_name(builder, "hash_sha256");
    json_builder_add_string_value(builder, bilgi->hash);

    json_builder_set_member_name(builder, "sistem");
    json_builder_begin_object(builder);
#ifdef _WIN32
    json_builder_set_member_name(builder, "isletim_sistemi");
    json_builder_add_string_value(builder, "Windows");
    json_builder_set_member_name(builder, "hostname");
    json_builder_add_string_value(builder, g_get_host_name());
#else
    struct utsname uts;
    if (uname(&uts) == 0) {
        json_builder_set_member_name(builder, "isletim_sistemi");
        json_builder_add_string_value(builder, uts.sysname);
        json_builder_set_member_name(builder, "surum");
        json_builder_add_string_value(builder, uts.release);
        json_builder_set_member_name(builder, "makine");
        json_builder_add_string_value(builder, uts.machine);
        json_builder_set_member_name(builder, "hostname");
        json_builder_add_string_value(builder, uts.nodename);
    }
#endif
    json_builder_end_object(builder);

    json_builder_end_object(builder);

    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, TRUE);

    bool basarili = g_file_set_contents(hedef_dosya, json_str, -1, NULL);

    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);

    return basarili;
}

bool rapor_olustur(RaporBilgisi* bilgi, RaporFormat format, const char* hedef_dosya, KanitKasasi* kasa) {
    if (!bilgi || !hedef_dosya) return false;

    bool sonuc;
    if (format == RAPOR_FORMAT_JSON) {
        sonuc = rapor_json_olustur(bilgi, hedef_dosya);
    } else {
        sonuc = rapor_txt_olustur(bilgi, hedef_dosya);
    }

    if (sonuc) {
        rapor_sistem_bilgisi_ekle(hedef_dosya);
        if (kasa) {
            gunluk_info(kasa->gunluk, "Rapor olusturuldu: %s", hedef_dosya);
        }
    }

    return sonuc;
}
