#include "hyperv_artifact.h"
#include "uzak_disk_edinim.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <json-glib/json-glib.h>
#include <time.h>

HyperVEdinim* hyperv_edinim_olustur(void) {
    HyperVEdinim* edinim = calloc(1, sizeof(HyperVEdinim));
    return edinim;
}

void hyperv_edinim_yok_et(HyperVEdinim* edinim) {
    if (!edinim) return;
    hyperv_baglanti_kapat(edinim);
    free(edinim);
}

bool hyperv_baglan(HyperVEdinim* edinim, UzakDiskBaglanti* uzak_baglanti) {
    if (!edinim || !uzak_baglanti) return false;
    
    // Uzaki baglantinin akislarini kullan
    if (uzak_baglanti->baglanti) {
        edinim->baglanti = g_object_ref(uzak_baglanti->baglanti);
        edinim->girdi = g_object_ref(uzak_baglanti->girdi);
        edinim->cikti = g_object_ref(uzak_baglanti->cikti);
        return true;
    }
    return false;
}

void hyperv_baglanti_kapat(HyperVEdinim* edinim) {
    if (!edinim) return;
    
    if (edinim->girdi) {
        g_object_unref(edinim->girdi);
        edinim->girdi = NULL;
    }
    if (edinim->cikti) {
        g_object_unref(edinim->cikti);
        edinim->cikti = NULL;
    }
    if (edinim->baglanti) {
        g_object_unref(edinim->baglanti);
        edinim->baglanti = NULL;
    }
}

static bool ciktiya_tam_yaz(GOutputStream* cikti, const char* veri, gsize uzunluk, GError** hata) {
    gsize yazilan = 0;
    return g_output_stream_write_all(cikti, veri, uzunluk, &yazilan, NULL, hata) && yazilan == uzunluk;
}

GList* hyperv_vm_listele(HyperVEdinim* edinim) {
    if (!edinim || !edinim->girdi) return NULL;
    
    GError* hata = NULL;
    
    // VM listele komutu gonder
    const char* komut = "{\"komut\":\"hyperv_vm_listele\"}\n";
    ciktiya_tam_yaz(edinim->cikti, komut, strlen(komut), &hata);
    if (!hata) {
        g_output_stream_flush(edinim->cikti, NULL, &hata);
    }
    
    if (hata) {
        g_warning("Hyper-V VM listeleme komutu gonderilemedi: %s", hata->message);
        g_error_free(hata);
        return NULL;
    }
    
    // Yaniti bekle
    gchar* satir = g_data_input_stream_read_line(edinim->girdi, NULL, NULL, &hata);
    if (!satir) {
        g_warning("Hyper-V VM yaniti alinamadi");
        return NULL;
    }
    
    JsonParser* parser = json_parser_new();
    if (!json_parser_load_from_data(parser, satir, -1, &hata)) {
        g_warning("JSON parse hatasi: %s", hata->message);
        g_error_free(hata);
        g_free(satir);
        g_object_unref(parser);
        return NULL;
    }
    
    JsonNode* root = json_parser_get_root(parser);
    JsonObject* obj = json_node_get_object(root);
    
    GList* vm_listesi = NULL;
    
    if (json_object_has_member(obj, "durum") && 
        strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
        
        if (json_object_has_member(obj, "vm_listesi")) {
            JsonArray* arr = json_object_get_array_member(obj, "vm_listesi");
            guint len = json_array_get_length(arr);
            
            for (guint i = 0; i < len; i++) {
                JsonObject* vm_obj = json_array_get_object_element(arr, i);
                char* vm_adi = g_strdup(json_object_get_string_member(vm_obj, "ad"));
                vm_listesi = g_list_append(vm_listesi, vm_adi);
            }
        }
    }
    
    g_free(satir);
    g_object_unref(parser);
    
    return vm_listesi;
}

GList* hyperv_bellek_dosyalari_listele(HyperVEdinim* edinim, const char* vm_adi) {
    if (!edinim || !edinim->girdi || !vm_adi) return NULL;
    
    GError* hata = NULL;
    
    // Bellek dosyalari listele komutu gonder
    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);
    json_builder_set_member_name(builder, "komut");
    json_builder_add_string_value(builder, "hyperv_bellek_listele");
    json_builder_set_member_name(builder, "vm_adi");
    json_builder_add_string_value(builder, vm_adi);
    json_builder_end_object(builder);
    
    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, FALSE);
    gchar* mesaj = g_strdup_printf("%s\n", json_str);
    
    ciktiya_tam_yaz(edinim->cikti, mesaj, strlen(mesaj), &hata);
    if (!hata) {
        g_output_stream_flush(edinim->cikti, NULL, &hata);
    }
    
    g_free(mesaj);
    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);
    
    if (hata) {
        g_warning("Hyper-V bellek listeleme komutu gonderilemedi: %s", hata->message);
        g_error_free(hata);
        return NULL;
    }
    
    // Yaniti bekle
    gchar* satir = g_data_input_stream_read_line(edinim->girdi, NULL, NULL, &hata);
    if (!satir) {
        g_warning("Hyper-V bellek yaniti alinamadi");
        return NULL;
    }
    
    JsonParser* parser = json_parser_new();
    if (!json_parser_load_from_data(parser, satir, -1, &hata)) {
        g_warning("JSON parse hatasi: %s", hata->message);
        g_error_free(hata);
        g_free(satir);
        g_object_unref(parser);
        return NULL;
    }
    
    JsonNode* root2 = json_parser_get_root(parser);
    JsonObject* obj = json_node_get_object(root2);
    
    GList* dosya_listesi = NULL;
    
    if (json_object_has_member(obj, "durum") && 
        strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
        
        if (json_object_has_member(obj, "dosyalar")) {
            JsonArray* arr = json_object_get_array_member(obj, "dosyalar");
            guint len = json_array_get_length(arr);
            
            for (guint i = 0; i < len; i++) {
                JsonObject* dosya_obj = json_array_get_object_element(arr, i);
                HyperVBellekDosyasi* dosya = g_new0(HyperVBellekDosyasi, 1);
                
                strncpy(dosya->dosya_adi, 
                        json_object_get_string_member(dosya_obj, "ad"), 255);
                strncpy(dosya->tam_yol, 
                        json_object_get_string_member(dosya_obj, "yol"), 1023);
                dosya->boyut = json_object_get_int_member(dosya_obj, "boyut");
                strncpy(dosya->vm_adi, vm_adi, 127);
                strncpy(dosya->tur, 
                        hyperv_dosya_turu_al(dosya->dosya_adi), 31);
                
                dosya_listesi = g_list_append(dosya_listesi, dosya);
            }
        }
    }
    
    g_free(satir);
    g_object_unref(parser);
    
    return dosya_listesi;
}

bool hyperv_dosya_indir(HyperVEdinim* edinim, const char* dosya_yolu,
                         const char* hedef_klasor, IsGorevi* is,
                         void (*ilerleme_cb)(int64_t okunan, int64_t toplam, void* veri),
                         void* kullanici_verisi) {
    if (!edinim || !edinim->girdi || !dosya_yolu || !hedef_klasor) return false;
    
    GError* hata = NULL;
    
    // Hedef dosya adini olustur
    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    char dosya_adi[512];
    const char* base_name = strrchr(dosya_yolu, '\\');
    if (!base_name) base_name = strrchr(dosya_yolu, '/');
    if (!base_name) base_name = dosya_yolu;
    else base_name++;
    
    snprintf(dosya_adi, sizeof(dosya_adi), "%s_%04d%02d%02d_%02d%02d%02d_%s",
             "hyperv",
             tm_info->tm_year + 1900, tm_info->tm_mon + 1, tm_info->tm_mday,
             tm_info->tm_hour, tm_info->tm_min, tm_info->tm_sec,
             base_name);
    
    char hedef_dosya_yolu[1024];
    snprintf(hedef_dosya_yolu, sizeof(hedef_dosya_yolu), "%s/%s", hedef_klasor, dosya_adi);
    
    FILE* hedef_dosya = fopen(hedef_dosya_yolu, "wb");
    if (!hedef_dosya) {
        g_warning("Hedef dosya acilamadi: %s", hedef_dosya_yolu);
        return false;
    }
    
    // Indirme komutu gonder
    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);
    json_builder_set_member_name(builder, "komut");
    json_builder_add_string_value(builder, "hyperv_dosya_indir");
    json_builder_set_member_name(builder, "dosya_yolu");
    json_builder_add_string_value(builder, dosya_yolu);
    json_builder_end_object(builder);
    
    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, FALSE);
    gchar* mesaj = g_strdup_printf("%s\n", json_str);
    
    ciktiya_tam_yaz(edinim->cikti, mesaj, strlen(mesaj), &hata);
    if (!hata) {
        g_output_stream_flush(edinim->cikti, NULL, &hata);
    }
    
    g_free(mesaj);
    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);
    
    if (hata) {
        g_warning("Hyper-V indirme komutu gonderilemedi: %s", hata->message);
        g_error_free(hata);
        fclose(hedef_dosya);
        return false;
    }
    
    // Yaniti bekle - metadata
    gchar* satir = g_data_input_stream_read_line(edinim->girdi, NULL, NULL, &hata);
    if (!satir) {
        g_warning("Hyper-V indirme yaniti alinamadi");
        fclose(hedef_dosya);
        return false;
    }
    
    JsonParser* parser = json_parser_new();
    if (!json_parser_load_from_data(parser, satir, -1, &hata)) {
        g_warning("JSON parse hatasi: %s", hata->message);
        g_error_free(hata);
        g_free(satir);
        g_object_unref(parser);
        fclose(hedef_dosya);
        return false;
    }
    
    JsonNode* root2 = json_parser_get_root(parser);
    JsonObject* obj = json_node_get_object(root2);
    
    bool basarili = false;
    int64_t toplam_boyut = 0;
    
    if (json_object_has_member(obj, "durum") && 
        strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
        toplam_boyut = json_object_get_int_member(obj, "boyut");
        basarili = true;
    }
    
    g_free(satir);
    g_object_unref(parser);
    
    if (!basarili) {
        fclose(hedef_dosya);
        return false;
    }
    
    // Veri akisini bekle
    satir = g_data_input_stream_read_line(edinim->girdi, NULL, NULL, &hata);
    if (!satir) {
        fclose(hedef_dosya);
        return false;
    }
    
    parser = json_parser_new();
    if (!json_parser_load_from_data(parser, satir, -1, NULL)) {
        g_free(satir);
        g_object_unref(parser);
        fclose(hedef_dosya);
        return false;
    }
    
    root2 = json_parser_get_root(parser);
    obj = json_node_get_object(root2);
    
    if (!json_object_has_member(obj, "tur") ||
        strcmp(json_object_get_string_member(obj, "tur"), "veri_basliyor") != 0) {
        g_object_unref(parser);
        g_free(satir);
        fclose(hedef_dosya);
        return false;
    }
    
    g_object_unref(parser);
    g_free(satir);
    
    // Binary veri akisi
    GInputStream* raw_girdi = G_INPUT_STREAM(edinim->girdi);
    guchar buffer[1024 * 1024];
    int64_t okunan = 0;
    bool hata_var = false;
    
    if (is) {
        is_durum_guncelle(is, IS_DURUMU_CALISIYOR, 0);
    }
    
    while (okunan < toplam_boyut) {
        gsize kalan = (gsize)(toplam_boyut - okunan);
        gsize okunacak = kalan < sizeof(buffer) ? kalan : sizeof(buffer);
        
        gssize okunan_bayt = g_input_stream_read(raw_girdi, buffer, okunacak, NULL, &hata);
        if (okunan_bayt <= 0) {
            if (hata) {
                g_warning("Veri okuma hatasi: %s", hata->message);
                g_error_free(hata);
            }
            hata_var = true;
            break;
        }
        
        if (fwrite(buffer, 1, (size_t)okunan_bayt, hedef_dosya) != (size_t)okunan_bayt) {
            hata_var = true;
            break;
        }
        
        okunan += okunan_bayt;
        
        if (ilerleme_cb) {
            ilerleme_cb(okunan, toplam_boyut, kullanici_verisi);
        }
        if (is && toplam_boyut > 0) {
            is_durum_guncelle(is, IS_DURUMU_CALISIYOR,
                              (int)((okunan * 100) / toplam_boyut));
        }
    }
    
    fclose(hedef_dosya);
    
    // Bitis mesaji
    if (!hata_var) {
        gchar* bitis_satiri = g_data_input_stream_read_line(edinim->girdi, NULL, NULL, &hata);
        if (!bitis_satiri) {
            hata_var = true;
        } else {
            parser = json_parser_new();
            if (!json_parser_load_from_data(parser, bitis_satiri, -1, NULL)) {
                hata_var = true;
            } else {
                JsonNode* root3 = json_parser_get_root(parser);
                JsonObject* obj3 = json_node_get_object(root3);
                if (!json_object_has_member(obj3, "tur") ||
                    strcmp(json_object_get_string_member(obj3, "tur"), "bitti") != 0) {
                    hata_var = true;
                }
            }
            g_object_unref(parser);
            g_free(bitis_satiri);
        }
    }
    
    if (hata_var) {
        if (is) {
            is_hata(is, "Hyper-V dosya indirme yarida kesildi");
        }
        return false;
    }
    
    if (is) {
        is_tamamla(is, hedef_klasor);
    }
    return true;
}

bool hyperv_varlik_mi(const char* dosya_yolu) {
    if (!dosya_yolu) return false;
    
    const char* ext = strrchr(dosya_yolu, '.');
    if (!ext) return false;
    
    return (strcmp(ext, ".vmem") == 0 ||
            strcmp(ext, ".bin") == 0 ||
            strcmp(ext, ".vhdx") == 0 ||
            strcmp(ext, ".avhdx") == 0);
}

const char* hyperv_dosya_turu_al(const char* dosya_adi) {
    if (!dosya_adi) return "bilinmiyor";
    
    const char* ext = strrchr(dosya_adi, '.');
    if (!ext) return "bilinmiyor";
    
    if (strcmp(ext, ".vmem") == 0) return "vmem";
    if (strcmp(ext, ".bin") == 0) return "bin";
    if (strcmp(ext, ".vhdx") == 0) return "vhdx";
    if (strcmp(ext, ".avhdx") == 0) return "avhdx";
    
    return "diger";
}
