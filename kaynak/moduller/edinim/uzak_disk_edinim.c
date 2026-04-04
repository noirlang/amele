#include "uzak_disk_edinim.h"
#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <json-glib/json-glib.h>
#include <time.h>

static bool partial_yol_uret(char* hedef, size_t hedef_boyutu, const char* kaynak) {
    if (!hedef || !kaynak || hedef_boyutu == 0) {
        return false;
    }

    int yazilan = snprintf(hedef, hedef_boyutu, "%s.partial", kaynak);
    return yazilan >= 0 && (size_t)yazilan < hedef_boyutu;
}

static bool ciktiya_tam_yaz(GOutputStream* cikti, const char* veri, gsize uzunluk, GError** hata) {
    gsize yazilan = 0;
    return g_output_stream_write_all(cikti, veri, uzunluk, &yazilan, NULL, hata) && yazilan == uzunluk;
}

static bool json_satir_yaniti_ok_mu(const char* satir, char* mesaj, size_t mesaj_boyut) {
    if (mesaj && mesaj_boyut > 0) {
        mesaj[0] = '\0';
    }
    if (!satir) {
        return false;
    }

    JsonParser* parser = json_parser_new();
    bool ok = false;
    if (json_parser_load_from_data(parser, satir, -1, NULL)) {
        JsonObject* obj = json_node_get_object(json_parser_get_root(parser));
        if (obj && json_object_has_member(obj, "durum") &&
            strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
            ok = true;
            if (mesaj && mesaj_boyut > 0 && json_object_has_member(obj, "mesaj")) {
                const char* m = json_object_get_string_member(obj, "mesaj");
                if (m) {
                    strncpy(mesaj, m, mesaj_boyut - 1);
                    mesaj[mesaj_boyut - 1] = '\0';
                }
            }
        } else if (obj && mesaj && mesaj_boyut > 0 && json_object_has_member(obj, "mesaj")) {
            const char* m = json_object_get_string_member(obj, "mesaj");
            if (m) {
                strncpy(mesaj, m, mesaj_boyut - 1);
                mesaj[mesaj_boyut - 1] = '\0';
            }
        }
    }
    g_object_unref(parser);
    return ok;
}

UzakDiskBaglanti* uzak_disk_baglanti_olustur(const char* ip, int port, const char* token) {
    if (!ip || port <= 0) return NULL;
    
    UzakDiskBaglanti* baglanti = calloc(1, sizeof(UzakDiskBaglanti));
    if (!baglanti) return NULL;
    
    strncpy(baglanti->ip, ip, sizeof(baglanti->ip) - 1);
    baglanti->port = port;
    if (token) {
        strncpy(baglanti->token, token, sizeof(baglanti->token) - 1);
    }
    baglanti->proto_durum = UZAK_PROTO_JSON;
    baglanti->son_yanit_ok = false;
    baglanti->son_hata[0] = '\0';
    
    return baglanti;
}

void uzak_disk_baglanti_kapat(UzakDiskBaglanti* baglanti) {
    if (!baglanti) return;

    if (baglanti->girdi) {
        g_object_unref(baglanti->girdi);
    }
    if (baglanti->cikti) {
        g_object_unref(baglanti->cikti);
    }
    if (baglanti->baglanti) {
        g_object_unref(baglanti->baglanti);
    }
    if (baglanti->hedef_dosya) {
        fclose(baglanti->hedef_dosya);
    }
    
    free(baglanti);
}

bool uzak_disk_baglan(UzakDiskBaglanti* baglanti) {
    if (!baglanti) return false;
    
    GError* hata = NULL;
    GSocketClient* istemci = g_socket_client_new();
    
    baglanti->baglanti = g_socket_client_connect_to_host(istemci,
                                                          baglanti->ip,
                                                          baglanti->port,
                                                          NULL,
                                                          &hata);
    g_object_unref(istemci);
    
    if (!baglanti->baglanti) {
        g_warning("Baglanti basarisiz: %s", hata->message);
        g_error_free(hata);
        return false;
    }
    
    GInputStream* girdi = g_io_stream_get_input_stream(G_IO_STREAM(baglanti->baglanti));
    baglanti->girdi = g_data_input_stream_new(girdi);
    g_data_input_stream_set_newline_type(baglanti->girdi, G_DATA_STREAM_NEWLINE_TYPE_LF);
    
    // Cikti streami baglanti omru disinda da guvenli yonetebilmek icin ref al.
    baglanti->cikti = g_object_ref(g_io_stream_get_output_stream(G_IO_STREAM(baglanti->baglanti)));

    // Okuma islemlerinde uzun sure takilmamasi icin zaman asimi uygula.
    GSocket* soket = g_socket_connection_get_socket(baglanti->baglanti);
    if (soket) {
        g_socket_set_timeout(soket, 10);
    }
    
    // Merhaba mesaji gonder
    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);
    json_builder_set_member_name(builder, "komut");
    json_builder_add_string_value(builder, "merhaba");
    json_builder_set_member_name(builder, "istemci");
    json_builder_add_string_value(builder, "worm");
    json_builder_set_member_name(builder, "surum");
    json_builder_add_string_value(builder, "0.1");
    if (strlen(baglanti->token) > 0) {
        gchar* anahtar_b64 = g_base64_encode((const guchar*)baglanti->token, strlen(baglanti->token));
        json_builder_set_member_name(builder, "token");
        json_builder_add_string_value(builder, baglanti->token);
        json_builder_set_member_name(builder, "guvenlik_anahtar_b64");
        json_builder_add_string_value(builder, anahtar_b64 ? anahtar_b64 : "");
        g_free(anahtar_b64);
    }
    json_builder_end_object(builder);
    
    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, FALSE);
    gchar* mesaj = g_strdup_printf("%s\n", json_str);
    
    ciktiya_tam_yaz(baglanti->cikti, mesaj, strlen(mesaj), &hata);
    if (!hata) {
        g_output_stream_flush(baglanti->cikti, NULL, &hata);
    }
    
    g_free(mesaj);
    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);
    
    if (hata) {
        g_warning("Merhaba gonderilemedi: %s", hata->message);
        g_error_free(hata);
        return false;
    }

    // Merhaba yanitini dogrula; aksi halde baglandi gorunup ilk komutta kopma yasanabilir.
    gchar* satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
    if (!satir) {
        if (hata) {
            g_warning("Merhaba yaniti alinamadi: %s", hata->message);
            g_error_free(hata);
        } else {
            g_warning("Merhaba yaniti alinamadi");
        }
        return false;
    }

    JsonParser* parser = json_parser_new();
    bool baglanti_ok = false;

    if (json_parser_load_from_data(parser, satir, -1, &hata)) {
        JsonNode* root2 = json_parser_get_root(parser);
        JsonObject* obj = json_node_get_object(root2);
        if (obj && json_object_has_member(obj, "durum") &&
            strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
            baglanti_ok = true;
        } else if (obj && json_object_has_member(obj, "mesaj")) {
            const char* mesaj = json_object_get_string_member(obj, "mesaj");
            if (mesaj) {
                strncpy(baglanti->son_hata, mesaj, sizeof(baglanti->son_hata) - 1);
                baglanti->son_hata[sizeof(baglanti->son_hata) - 1] = '\0';
            }
        }
    }

    if (hata) {
        g_warning("Merhaba parse hatasi: %s", hata->message);
        g_error_free(hata);
    }

    g_object_unref(parser);
    g_free(satir);

    if (!baglanti_ok) {
        if (baglanti->son_hata[0] != '\0') {
            g_warning("Uzak uc beklenen ajan yaniti vermedi: %s", baglanti->son_hata);
        } else {
            g_warning("Uzak uc beklenen ajan yaniti vermedi");
        }
        return false;
    }
    
    return true;
}

bool uzak_edinim_kontrol_gonder(UzakDiskBaglanti* baglanti,
                                const char* is_id,
                                const char* eylem,
                                char* sonuc_metin,
                                size_t sonuc_metin_boyut) {
    if (sonuc_metin && sonuc_metin_boyut > 0) {
        sonuc_metin[0] = '\0';
    }
    if (!baglanti || !is_id || !eylem || is_id[0] == '\0' || eylem[0] == '\0') {
        return false;
    }

    UzakDiskBaglanti* kontrol = uzak_disk_baglanti_olustur(
        baglanti->ip,
        baglanti->port,
        baglanti->token[0] ? baglanti->token : NULL
    );
    if (!kontrol) {
        if (sonuc_metin && sonuc_metin_boyut > 0) {
            strncpy(sonuc_metin, "Kontrol baglantisi olusturulamadi", sonuc_metin_boyut - 1);
            sonuc_metin[sonuc_metin_boyut - 1] = '\0';
        }
        return false;
    }

    if (!uzak_disk_baglan(kontrol)) {
        if (sonuc_metin && sonuc_metin_boyut > 0) {
            const char* m = kontrol->son_hata[0] ? kontrol->son_hata : "Kontrol baglantisi kurulamadi";
            strncpy(sonuc_metin, m, sonuc_metin_boyut - 1);
            sonuc_metin[sonuc_metin_boyut - 1] = '\0';
        }
        uzak_disk_baglanti_kapat(kontrol);
        return false;
    }

    GError* hata = NULL;
    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);
    json_builder_set_member_name(builder, "komut");
    json_builder_add_string_value(builder, "edinim_kontrol");
    json_builder_set_member_name(builder, "is_id");
    json_builder_add_string_value(builder, is_id);
    json_builder_set_member_name(builder, "eylem");
    json_builder_add_string_value(builder, eylem);
    json_builder_end_object(builder);

    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, FALSE);
    gchar* mesaj = g_strdup_printf("%s\n", json_str);

    bool sonuc = false;
    if (ciktiya_tam_yaz(kontrol->cikti, mesaj, strlen(mesaj), &hata) &&
        g_output_stream_flush(kontrol->cikti, NULL, &hata)) {
        gchar* satir = g_data_input_stream_read_line(kontrol->girdi, NULL, NULL, &hata);
        if (satir) {
            sonuc = json_satir_yaniti_ok_mu(satir, sonuc_metin, sonuc_metin_boyut);
            g_free(satir);
        }
    }

    if (hata && sonuc_metin && sonuc_metin_boyut > 0 && sonuc_metin[0] == '\0') {
        strncpy(sonuc_metin, hata->message ? hata->message : "Kontrol komutu hatasi", sonuc_metin_boyut - 1);
        sonuc_metin[sonuc_metin_boyut - 1] = '\0';
    }

    if (hata) {
        g_error_free(hata);
    }
    g_free(mesaj);
    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);
    uzak_disk_baglanti_kapat(kontrol);
    return sonuc;
}

GList* uzak_disk_listele(UzakDiskBaglanti* baglanti) {
    if (!baglanti || !baglanti->girdi) return NULL;
    
    baglanti->son_yanit_ok = false;
    baglanti->son_hata[0] = '\0';

    GError* hata = NULL;
    
    // Disk listele komutu gonder
    const char* komut = "{\"komut\":\"disk_listele\"}\n";
    ciktiya_tam_yaz(baglanti->cikti, komut, strlen(komut), &hata);
    if (!hata) {
        g_output_stream_flush(baglanti->cikti, NULL, &hata);
    }
    
    if (hata) {
        g_warning("Komut gonderilemedi: %s", hata->message);
        strncpy(baglanti->son_hata, "Komut gonderilemedi", sizeof(baglanti->son_hata) - 1);
        g_error_free(hata);
        return NULL;
    }
    
    // Yaniti bekle
    gchar* satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
    if (!satir) {
        g_warning("Yanit alinamadi");
        strncpy(baglanti->son_hata, "Yanit alinamadi", sizeof(baglanti->son_hata) - 1);
        return NULL;
    }
    
    JsonParser* parser = json_parser_new();
    if (!json_parser_load_from_data(parser, satir, -1, &hata)) {
        g_warning("JSON parse hatasi: %s", hata->message);
        strncpy(baglanti->son_hata, "Gecersiz JSON yaniti", sizeof(baglanti->son_hata) - 1);
        g_error_free(hata);
        g_free(satir);
        g_object_unref(parser);
        return NULL;
    }
    
    JsonNode* root = json_parser_get_root(parser);
    JsonObject* obj = json_node_get_object(root);
    
    GList* diskler = NULL;
    
    if (json_object_has_member(obj, "durum") && 
        strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
        baglanti->son_yanit_ok = true;
        
        if (json_object_has_member(obj, "diskler")) {
            JsonArray* arr = json_object_get_array_member(obj, "diskler");
            guint len = json_array_get_length(arr);
            
            for (guint i = 0; i < len; i++) {
                JsonObject* disk_obj = json_array_get_object_element(arr, i);
                UzakDisk* disk = g_new0(UzakDisk, 1);
                
                strncpy(disk->id, json_object_get_string_member(disk_obj, "id"), 31);
                strncpy(disk->ad, json_object_get_string_member(disk_obj, "ad"), 255);
                disk->boyut = json_object_get_int_member(disk_obj, "boyut");
                
                diskler = g_list_append(diskler, disk);
            }
        }
    } else if (json_object_has_member(obj, "durum") &&
               strcmp(json_object_get_string_member(obj, "durum"), "hata") == 0) {
        if (json_object_has_member(obj, "mesaj")) {
            const char* mesaj = json_object_get_string_member(obj, "mesaj");
            if (mesaj) {
                strncpy(baglanti->son_hata, mesaj, sizeof(baglanti->son_hata) - 1);
            }
        } else {
            strncpy(baglanti->son_hata, "Uzak ajan hata dondurdu", sizeof(baglanti->son_hata) - 1);
        }
    } else {
        strncpy(baglanti->son_hata, "Beklenmeyen yanit", sizeof(baglanti->son_hata) - 1);
    }
    
    g_free(satir);
    g_object_unref(parser);
    
    return diskler;
}

bool uzak_imaj_baslat(UzakDiskBaglanti* baglanti, const char* disk_id,
                       const char* hedef_klasor, const char* vaka_adi) {
    (void)vaka_adi;
    if (!baglanti || !baglanti->girdi || !disk_id || !hedef_klasor) return false;
    
    GError* hata = NULL;
    
    // Hedef dosya olustur
    time_t now = time(NULL);
    struct tm* tm_info = localtime(&now);
    char dosya_adi[512];
    snprintf(dosya_adi, sizeof(dosya_adi), "%s_%s_%04d%02d%02d_%02d%02d%02d.img",
             baglanti->ip, disk_id,
             tm_info->tm_year + 1900, tm_info->tm_mon + 1, tm_info->tm_mday,
             tm_info->tm_hour, tm_info->tm_min, tm_info->tm_sec);
    
    snprintf(baglanti->hedef_yol, sizeof(baglanti->hedef_yol), 
             "%s/%s", hedef_klasor, dosya_adi);

    // Hedef klasor yoksa olustur; aksi halde fopen basarisiz olur.
    if (g_mkdir_with_parents(hedef_klasor, 0755) != 0) {
        g_warning("Hedef klasor olusturulamadi: %s", hedef_klasor);
        return false;
    }
    
    baglanti->hedef_dosya = fopen(baglanti->hedef_yol, "wb");
    if (!baglanti->hedef_dosya) {
        g_warning("Hedef dosya acilamadi: %s", baglanti->hedef_yol);
        return false;
    }
    
    // Imaj baslat komutu gonder
    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);
    json_builder_set_member_name(builder, "komut");
    json_builder_add_string_value(builder, "imaj_baslat");
    json_builder_set_member_name(builder, "disk_id");
    json_builder_add_string_value(builder, disk_id);
    json_builder_set_member_name(builder, "format");
    json_builder_add_string_value(builder, "raw");
    json_builder_set_member_name(builder, "parca_boyutu");
    json_builder_add_int_value(builder, 4 * 1024 * 1024);
    json_builder_end_object(builder);
    
    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, FALSE);
    gchar* mesaj = g_strdup_printf("%s\n", json_str);
    
    ciktiya_tam_yaz(baglanti->cikti, mesaj, strlen(mesaj), &hata);
    if (!hata) {
        g_output_stream_flush(baglanti->cikti, NULL, &hata);
    }
    
    g_free(mesaj);
    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);
    
    if (hata) {
        g_warning("Imaj baslat komutu gonderilemedi: %s", hata->message);
        g_error_free(hata);
        fclose(baglanti->hedef_dosya);
        baglanti->hedef_dosya = NULL;
        return false;
    }
    
    // Yaniti bekle
    gchar* satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
    if (!satir) {
        g_warning("Imaj baslat yaniti alinamadi");
        fclose(baglanti->hedef_dosya);
        baglanti->hedef_dosya = NULL;
        return false;
    }
    
    JsonParser* parser = json_parser_new();
    if (!json_parser_load_from_data(parser, satir, -1, &hata)) {
        g_warning("JSON parse hatasi: %s", hata->message);
        g_error_free(hata);
        g_free(satir);
        g_object_unref(parser);
        fclose(baglanti->hedef_dosya);
        baglanti->hedef_dosya = NULL;
        return false;
    }
    
    JsonNode* root2 = json_parser_get_root(parser);
    JsonObject* obj = json_node_get_object(root2);
    
    bool basarili = false;
    if (json_object_has_member(obj, "durum") && 
        strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
        
        strncpy(baglanti->is_id, json_object_get_string_member(obj, "is_id"), 31);
        baglanti->toplam_boyut = json_object_get_int_member(obj, "tahmini_boyut");
        baglanti->okunan = 0;
        basarili = true;
    }
    
    g_free(satir);
    g_object_unref(parser);
    
    if (!basarili) {
        fclose(baglanti->hedef_dosya);
        baglanti->hedef_dosya = NULL;
    }
    
    return basarili;
}

bool uzak_imaj_stream_al(UzakDiskBaglanti* baglanti, IsGorevi* is,
                          void (*ilerleme_cb)(int64_t okunan, int64_t toplam, void* veri),
                          void* kullanici_verisi) {
    if (!baglanti || !baglanti->girdi || !baglanti->hedef_dosya) return false;
    
    GError* hata = NULL;
    bool tamamlandi = false;
    bool hata_var = false;
    bool kullanici_durdurdu = false;
    baglanti->son_hata[0] = '\0';

    GSocket* soket = g_socket_connection_get_socket(baglanti->baglanti);
    guint onceki_timeout = 0;
    if (soket) {
        onceki_timeout = g_socket_get_timeout(soket);
        // Duraklatma sirasinda akista veri akmayabilecegi icin timeout kapatilir.
        g_socket_set_timeout(soket, 0);
    }
    
    while (!tamamlandi && !hata_var) {
        gchar* satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);

        if (!satir) {
            if (hata) {
                g_warning("Veri alinamadi: %s", hata->message);
                g_error_free(hata);
            }
            hata_var = true;
            break;
        }

        JsonParser* parser = json_parser_new();
        if (!json_parser_load_from_data(parser, satir, -1, NULL)) {
            g_object_unref(parser);
            g_free(satir);
            hata_var = true;
            break;
        }

        JsonNode* root = json_parser_get_root(parser);
        JsonObject* obj = json_node_get_object(root);

        if (!json_object_has_member(obj, "tur")) {
            g_object_unref(parser);
            g_free(satir);
            hata_var = true;
            break;
        }

        const char* tur = json_object_get_string_member(obj, "tur");

        if (strcmp(tur, "hata") == 0) {
            if (json_object_has_member(obj, "mesaj")) {
                const char* m = json_object_get_string_member(obj, "mesaj");
                if (m) {
                    strncpy(baglanti->son_hata, m, sizeof(baglanti->son_hata) - 1);
                    baglanti->son_hata[sizeof(baglanti->son_hata) - 1] = '\0';
                }
            }
            if (json_object_has_member(obj, "kod")) {
                const char* kod = json_object_get_string_member(obj, "kod");
                if (kod && strcmp(kod, "STOPPED_BY_USER") == 0) {
                    kullanici_durdurdu = true;
                }
            }
            hata_var = true;
            g_object_unref(parser);
            g_free(satir);
            break;
        }

        if (strcmp(tur, "ilerleme") == 0) {
            baglanti->okunan = json_object_get_int_member(obj, "okunan");
            if (ilerleme_cb) {
                ilerleme_cb(baglanti->okunan, baglanti->toplam_boyut, kullanici_verisi);
            }
            if (is && baglanti->toplam_boyut > 0) {
                is_durum_guncelle(is, IS_DURUMU_CALISIYOR,
                                  (int)((baglanti->okunan * 100) / baglanti->toplam_boyut));
            }
            g_object_unref(parser);
            g_free(satir);
            continue;
        }

        if (strcmp(tur, "veri_basliyor") == 0) {
            baglanti->proto_durum = UZAK_PROTO_VERI;
            g_object_unref(parser);
            g_free(satir);
            break;
        }

        g_object_unref(parser);
        g_free(satir);
    }

    if (!hata_var && baglanti->proto_durum == UZAK_PROTO_VERI) {
        // GDataInputStream satir okurken tamponlayabildigi icin ham veriyi ayni akistan okuyarak
        // olasi tampon kaybini engelleriz.
        GInputStream* raw_girdi = G_INPUT_STREAM(baglanti->girdi);
        guchar buffer[1024 * 1024];

        if (is) {
            is_durum_guncelle(is, IS_DURUMU_CALISIYOR, 0);
        }

        while (baglanti->okunan < baglanti->toplam_boyut) {
            gsize kalan = (gsize)(baglanti->toplam_boyut - baglanti->okunan);
            gsize okunacak = kalan < sizeof(buffer) ? kalan : sizeof(buffer);

            gssize okunan = g_input_stream_read(raw_girdi, buffer, okunacak, NULL, &hata);
            if (okunan <= 0) {
                if (hata) {
                    g_warning("Veri okuma hatasi: %s", hata->message);
                    g_error_free(hata);
                }
                hata_var = true;
                break;
            }

            if (fwrite(buffer, 1, (size_t)okunan, baglanti->hedef_dosya) != (size_t)okunan) {
                hata_var = true;
                break;
            }

            baglanti->okunan += okunan;

            if (ilerleme_cb) {
                ilerleme_cb(baglanti->okunan, baglanti->toplam_boyut, kullanici_verisi);
            }
            if (is && baglanti->toplam_boyut > 0) {
                is_durum_guncelle(is, IS_DURUMU_CALISIYOR,
                                  (int)((baglanti->okunan * 100) / baglanti->toplam_boyut));
            }
        }

        if (!hata_var) {
            gchar* bitis_satiri = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
            if (!bitis_satiri) {
                if (hata) {
                    g_warning("Bitis mesaji alinamadi: %s", hata->message);
                    g_error_free(hata);
                }
                hata_var = true;
            } else {
                JsonParser* parser = json_parser_new();
                if (!json_parser_load_from_data(parser, bitis_satiri, -1, NULL)) {
                    hata_var = true;
                } else {
                    JsonNode* root = json_parser_get_root(parser);
                    JsonObject* obj = json_node_get_object(root);
                    if (!json_object_has_member(obj, "tur")) {
                        hata_var = true;
                    } else {
                        const char* tur = json_object_get_string_member(obj, "tur");
                        if (strcmp(tur, "bitti") == 0) {
                            tamamlandi = true;
                        } else if (strcmp(tur, "hata") == 0) {
                            if (json_object_has_member(obj, "mesaj")) {
                                const char* m = json_object_get_string_member(obj, "mesaj");
                                if (m) {
                                    strncpy(baglanti->son_hata, m, sizeof(baglanti->son_hata) - 1);
                                    baglanti->son_hata[sizeof(baglanti->son_hata) - 1] = '\0';
                                }
                            }
                            if (json_object_has_member(obj, "kod")) {
                                const char* kod = json_object_get_string_member(obj, "kod");
                                if (kod && strcmp(kod, "STOPPED_BY_USER") == 0) {
                                    kullanici_durdurdu = true;
                                }
                            }
                            hata_var = true;
                        } else {
                            hata_var = true;
                        }
                    }
                }
                g_object_unref(parser);
                g_free(bitis_satiri);
            }
        }
    }

    baglanti->proto_durum = UZAK_PROTO_JSON;
    
    if (hata_var) {
        // Yari kalmiş dosyaya .partial ekle
        fclose(baglanti->hedef_dosya);
        baglanti->hedef_dosya = NULL;
        
        char partial_yol[1024];
        if (partial_yol_uret(partial_yol, sizeof(partial_yol), baglanti->hedef_yol)) {
            rename(baglanti->hedef_yol, partial_yol);
        }

        if (kullanici_durdurdu) {
            is_durum_guncelle(is, IS_DURUMU_IPTAL_EDILDI, -1);
            if (baglanti->son_hata[0] == '\0') {
                strncpy(baglanti->son_hata, "Imaj alma kullanici tarafindan durduruldu", sizeof(baglanti->son_hata) - 1);
                baglanti->son_hata[sizeof(baglanti->son_hata) - 1] = '\0';
            }
        } else {
            is_hata(is, "Imaj alma yarida kesildi");
        }

        if (soket) {
            g_socket_set_timeout(soket, onceki_timeout);
        }
        return false;
    }
    
    fclose(baglanti->hedef_dosya);
    baglanti->hedef_dosya = NULL;
    
    if (is) {
        is_tamamla(is, NULL);
    }

    if (soket) {
        g_socket_set_timeout(soket, onceki_timeout);
    }
    return true;
}

bool uzak_winpmem_kontrol(UzakDiskBaglanti* baglanti,
                          bool* winpmem_mevcut,
                          bool* yonetici_yetkisi,
                          int64_t* ram_boyut,
                          char* mesaj,
                          size_t mesaj_boyut) {
    if (!baglanti || !baglanti->girdi || !baglanti->cikti) {
        return false;
    }

    if (winpmem_mevcut) *winpmem_mevcut = false;
    if (yonetici_yetkisi) *yonetici_yetkisi = false;
    if (ram_boyut) *ram_boyut = 0;
    if (mesaj && mesaj_boyut > 0) mesaj[0] = '\0';

    GError* hata = NULL;
    const char* komut = "{\"komut\":\"winpmem_kontrol\"}\n";
    ciktiya_tam_yaz(baglanti->cikti, komut, strlen(komut), &hata);
    if (!hata) {
        g_output_stream_flush(baglanti->cikti, NULL, &hata);
    }
    if (hata) {
        g_error_free(hata);
        return false;
    }

    gchar* satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
    if (!satir) {
        if (hata) g_error_free(hata);
        return false;
    }

    JsonParser* parser = json_parser_new();
    bool ok = false;
    if (json_parser_load_from_data(parser, satir, -1, &hata)) {
        JsonNode* root = json_parser_get_root(parser);
        JsonObject* obj = json_node_get_object(root);
        if (obj && json_object_has_member(obj, "durum") &&
            strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
            ok = true;
            if (winpmem_mevcut && json_object_has_member(obj, "winpmem_mevcut")) {
                *winpmem_mevcut = json_object_get_boolean_member(obj, "winpmem_mevcut");
            }
            if (yonetici_yetkisi && json_object_has_member(obj, "yonetici_yetkisi")) {
                *yonetici_yetkisi = json_object_get_boolean_member(obj, "yonetici_yetkisi");
            }
            if (ram_boyut && json_object_has_member(obj, "ram_boyut")) {
                *ram_boyut = json_object_get_int_member(obj, "ram_boyut");
            }
            if (mesaj && mesaj_boyut > 0 && json_object_has_member(obj, "mesaj")) {
                const char* m = json_object_get_string_member(obj, "mesaj");
                if (m) {
                    strncpy(mesaj, m, mesaj_boyut - 1);
                    mesaj[mesaj_boyut - 1] = '\0';
                }
            }
        }
    }

    if (hata) {
        g_error_free(hata);
    }
    g_object_unref(parser);
    g_free(satir);
    return ok;
}

bool uzak_avml_kontrol(UzakDiskBaglanti* baglanti,
                       bool* avml_mevcut,
                       bool* yonetici_yetkisi,
                       int64_t* ram_boyut,
                       char* mesaj,
                       size_t mesaj_boyut) {
    if (!baglanti || !baglanti->girdi || !baglanti->cikti) {
        return false;
    }

    if (avml_mevcut) *avml_mevcut = false;
    if (yonetici_yetkisi) *yonetici_yetkisi = false;
    if (ram_boyut) *ram_boyut = 0;
    if (mesaj && mesaj_boyut > 0) mesaj[0] = '\0';

    GError* hata = NULL;
    const char* komut = "{\"komut\":\"avml_kontrol\"}\n";
    ciktiya_tam_yaz(baglanti->cikti, komut, strlen(komut), &hata);
    if (!hata) {
        g_output_stream_flush(baglanti->cikti, NULL, &hata);
    }
    if (hata) {
        g_error_free(hata);
        return false;
    }

    gchar* satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
    if (!satir) {
        if (hata) g_error_free(hata);
        return false;
    }

    JsonParser* parser = json_parser_new();
    bool ok = false;
    if (json_parser_load_from_data(parser, satir, -1, &hata)) {
        JsonNode* root = json_parser_get_root(parser);
        JsonObject* obj = json_node_get_object(root);
        if (obj && json_object_has_member(obj, "durum") &&
            strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
            ok = true;
            if (avml_mevcut && json_object_has_member(obj, "avml_mevcut")) {
                *avml_mevcut = json_object_get_boolean_member(obj, "avml_mevcut");
            }
            if (yonetici_yetkisi && json_object_has_member(obj, "yonetici_yetkisi")) {
                *yonetici_yetkisi = json_object_get_boolean_member(obj, "yonetici_yetkisi");
            }
            if (ram_boyut && json_object_has_member(obj, "ram_boyut")) {
                *ram_boyut = json_object_get_int_member(obj, "ram_boyut");
            }
            if (mesaj && mesaj_boyut > 0 && json_object_has_member(obj, "mesaj")) {
                const char* m = json_object_get_string_member(obj, "mesaj");
                if (m) {
                    strncpy(mesaj, m, mesaj_boyut - 1);
                    mesaj[mesaj_boyut - 1] = '\0';
                }
            }
        }
    }

    if (hata) {
        g_error_free(hata);
    }
    g_object_unref(parser);
    g_free(satir);
    return ok;
}

bool uzak_ram_edinim_baslat_ve_takip(UzakDiskBaglanti* baglanti,
                                     const char* cikti_dosya,
                                     const char* is_id_istek,
                                     void (*ilerleme_cb)(int64_t okunan, int64_t toplam, void* veri),
                                     void* kullanici_verisi,
                                     char* sonuc_metin,
                                     size_t sonuc_metin_boyut,
                                     char* is_id_cikti,
                                     size_t is_id_cikti_boyut) {
    if (!baglanti || !baglanti->girdi || !baglanti->cikti || !cikti_dosya) {
        return false;
    }

    if (sonuc_metin && sonuc_metin_boyut > 0) {
        sonuc_metin[0] = '\0';
    }
    if (is_id_cikti && is_id_cikti_boyut > 0) {
        is_id_cikti[0] = '\0';
    }

    GError* hata = NULL;
    GSocket* soket = g_socket_connection_get_socket(baglanti->baglanti);
    guint onceki_timeout = 0;
    if (soket) {
        onceki_timeout = g_socket_get_timeout(soket);
        // RAM ediniminde ajan tarafi bitiste hash hesapladigi icin bir sure sessiz kalabilir.
        // Bu asamada kisa socket timeout yalanci "hata" uretebilir.
        g_socket_set_timeout(soket, 0);
    }
    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);
    json_builder_set_member_name(builder, "komut");
    json_builder_add_string_value(builder, "ram_edinim_baslat");
    json_builder_set_member_name(builder, "cikti_dosya");
    json_builder_add_string_value(builder, cikti_dosya);
    if (is_id_istek && is_id_istek[0] != '\0') {
        json_builder_set_member_name(builder, "is_id");
        json_builder_add_string_value(builder, is_id_istek);
        if (is_id_cikti && is_id_cikti_boyut > 0) {
            strncpy(is_id_cikti, is_id_istek, is_id_cikti_boyut - 1);
            is_id_cikti[is_id_cikti_boyut - 1] = '\0';
        }
    }
    json_builder_end_object(builder);

    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, FALSE);
    gchar* mesaj = g_strdup_printf("%s\n", json_str);

    ciktiya_tam_yaz(baglanti->cikti, mesaj, strlen(mesaj), &hata);
    if (!hata) {
        g_output_stream_flush(baglanti->cikti, NULL, &hata);
    }

    g_free(mesaj);
    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);

    if (hata) {
        if (sonuc_metin && sonuc_metin_boyut > 0) {
            const char* m = hata->message ? hata->message : "Komut gonderim hatasi";
            strncpy(sonuc_metin, m, sonuc_metin_boyut - 1);
            sonuc_metin[sonuc_metin_boyut - 1] = '\0';
        }
        g_error_free(hata);
        if (soket) {
            g_socket_set_timeout(soket, onceki_timeout);
        }
        return false;
    }

    int64_t toplam = 0;
    bool veri_basladi = false;

    while (true) {
        gchar* satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
        if (!satir) {
            if (sonuc_metin && sonuc_metin_boyut > 0) {
                if (hata && hata->message) {
                    strncpy(sonuc_metin, hata->message, sonuc_metin_boyut - 1);
                } else {
                    strncpy(sonuc_metin, "Ajan baglantisi kesildi", sonuc_metin_boyut - 1);
                }
                sonuc_metin[sonuc_metin_boyut - 1] = '\0';
            }
            if (hata) g_error_free(hata);
            if (soket) {
                g_socket_set_timeout(soket, onceki_timeout);
            }
            return false;
        }

        JsonParser* parser = json_parser_new();
        bool parse_ok = json_parser_load_from_data(parser, satir, -1, NULL);
        if (!parse_ok) {
            if (sonuc_metin && sonuc_metin_boyut > 0) {
                strncpy(sonuc_metin, "Ajan yaniti gecersiz JSON", sonuc_metin_boyut - 1);
                sonuc_metin[sonuc_metin_boyut - 1] = '\0';
            }
            g_object_unref(parser);
            g_free(satir);
            if (soket) {
                g_socket_set_timeout(soket, onceki_timeout);
            }
            return false;
        }

        JsonNode* r = json_parser_get_root(parser);
        JsonObject* obj = json_node_get_object(r);

        if (json_object_has_member(obj, "durum") &&
            strcmp(json_object_get_string_member(obj, "durum"), "hata") == 0) {
            if (sonuc_metin && sonuc_metin_boyut > 0) {
                const char* m = json_object_has_member(obj, "mesaj") ?
                                json_object_get_string_member(obj, "mesaj") :
                                "Uzak ajan hata dondurdu";
                strncpy(sonuc_metin, m ? m : "Uzak ajan hata dondurdu", sonuc_metin_boyut - 1);
                sonuc_metin[sonuc_metin_boyut - 1] = '\0';
            }
            g_object_unref(parser);
            g_free(satir);
            if (soket) {
                g_socket_set_timeout(soket, onceki_timeout);
            }
            return false;
        }

        if (json_object_has_member(obj, "durum") &&
            strcmp(json_object_get_string_member(obj, "durum"), "ok") == 0) {
            if (json_object_has_member(obj, "toplam_boyut")) {
                toplam = json_object_get_int_member(obj, "toplam_boyut");
            }
            if (is_id_cikti && is_id_cikti_boyut > 0 && json_object_has_member(obj, "is_id")) {
                const char* is_id = json_object_get_string_member(obj, "is_id");
                if (is_id) {
                    strncpy(is_id_cikti, is_id, is_id_cikti_boyut - 1);
                    is_id_cikti[is_id_cikti_boyut - 1] = '\0';
                }
            }
            g_object_unref(parser);
            g_free(satir);
            continue;
        }

        if (!json_object_has_member(obj, "tur")) {
            g_object_unref(parser);
            g_free(satir);
            continue;
        }

        const char* tur = json_object_get_string_member(obj, "tur");

        if (strcmp(tur, "veri_basliyor") == 0) {
            veri_basladi = true;
            if (json_object_has_member(obj, "toplam")) {
                toplam = json_object_get_int_member(obj, "toplam");
            }
            if (is_id_cikti && is_id_cikti_boyut > 0 && is_id_cikti[0] == '\0' &&
                json_object_has_member(obj, "is_id")) {
                const char* is_id = json_object_get_string_member(obj, "is_id");
                if (is_id) {
                    strncpy(is_id_cikti, is_id, is_id_cikti_boyut - 1);
                    is_id_cikti[is_id_cikti_boyut - 1] = '\0';
                }
            }
            g_object_unref(parser);
            g_free(satir);
            continue;
        }

        if (strcmp(tur, "ilerleme") == 0 && veri_basladi) {
            int64_t okunan = json_object_has_member(obj, "okunan") ?
                             json_object_get_int_member(obj, "okunan") : 0;
            int64_t t = json_object_has_member(obj, "toplam") ?
                        json_object_get_int_member(obj, "toplam") : toplam;
            if (ilerleme_cb && t > 0) {
                ilerleme_cb(okunan, t, kullanici_verisi);
            }
            g_object_unref(parser);
            g_free(satir);
            continue;
        }

        if (strcmp(tur, "bitti") == 0) {
            if (ilerleme_cb && toplam > 0) {
                ilerleme_cb(toplam, toplam, kullanici_verisi);
            }
            if (sonuc_metin && sonuc_metin_boyut > 0) {
                const char* m = json_object_has_member(obj, "mesaj") ?
                                json_object_get_string_member(obj, "mesaj") :
                                "RAM edinimi tamamlandi";
                strncpy(sonuc_metin, m, sonuc_metin_boyut - 1);
                sonuc_metin[sonuc_metin_boyut - 1] = '\0';
            }
            g_object_unref(parser);
            g_free(satir);
            if (soket) {
                g_socket_set_timeout(soket, onceki_timeout);
            }
            return true;
        }

        if (strcmp(tur, "hata") == 0) {
            if (sonuc_metin && sonuc_metin_boyut > 0) {
                const char* m = json_object_has_member(obj, "mesaj") ?
                                json_object_get_string_member(obj, "mesaj") :
                                "Uzak ajan RAM edinim hatasi";
                strncpy(sonuc_metin, m ? m : "Uzak ajan RAM edinim hatasi", sonuc_metin_boyut - 1);
                sonuc_metin[sonuc_metin_boyut - 1] = '\0';
            }
            g_object_unref(parser);
            g_free(satir);
            if (soket) {
                g_socket_set_timeout(soket, onceki_timeout);
            }
            return false;
        }

        g_object_unref(parser);
        g_free(satir);
    }

    if (soket) {
        g_socket_set_timeout(soket, onceki_timeout);
    }
}

bool uzak_ram_dosya_indir(UzakDiskBaglanti* baglanti,
                          const char* uzak_dosya,
                          const char* yerel_yol,
                          void (*ilerleme_cb)(int64_t okunan, int64_t toplam, void* veri),
                          void* kullanici_verisi,
                          char* sonuc_metin,
                          size_t sonuc_metin_boyut) {
    if (!baglanti || !baglanti->girdi || !baglanti->cikti || !uzak_dosya || !yerel_yol) {
        return false;
    }

    if (sonuc_metin && sonuc_metin_boyut > 0) {
        sonuc_metin[0] = '\0';
    }

    GError* hata = NULL;
    JsonBuilder* builder = json_builder_new();
    json_builder_begin_object(builder);
    json_builder_set_member_name(builder, "komut");
    json_builder_add_string_value(builder, "ram_dosya_indir");
    json_builder_set_member_name(builder, "dosya");
    json_builder_add_string_value(builder, uzak_dosya);
    json_builder_end_object(builder);

    JsonNode* root = json_builder_get_root(builder);
    gchar* json_str = json_to_string(root, FALSE);
    gchar* mesaj = g_strdup_printf("%s\n", json_str);

    ciktiya_tam_yaz(baglanti->cikti, mesaj, strlen(mesaj), &hata);
    if (!hata) {
        g_output_stream_flush(baglanti->cikti, NULL, &hata);
    }

    g_free(mesaj);
    g_free(json_str);
    json_node_free(root);
    g_object_unref(builder);

    if (hata) {
        if (hata) g_error_free(hata);
        return false;
    }

    int64_t toplam = 0;

    // 1) Baslangic JSON: durum ok/hata
    gchar* satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
    if (!satir) {
        if (hata) g_error_free(hata);
        return false;
    }

    JsonParser* parser = json_parser_new();
    if (!json_parser_load_from_data(parser, satir, -1, NULL)) {
        g_object_unref(parser);
        g_free(satir);
        return false;
    }

    JsonObject* obj = json_node_get_object(json_parser_get_root(parser));
    if (!obj || !json_object_has_member(obj, "durum")) {
        g_object_unref(parser);
        g_free(satir);
        return false;
    }

    const char* durum = json_object_get_string_member(obj, "durum");
    if (strcmp(durum, "ok") != 0) {
        if (sonuc_metin && sonuc_metin_boyut > 0) {
            const char* m = json_object_has_member(obj, "mesaj") ?
                            json_object_get_string_member(obj, "mesaj") :
                            "RAM dosya indirilemedi";
            strncpy(sonuc_metin, m ? m : "RAM dosya indirilemedi", sonuc_metin_boyut - 1);
            sonuc_metin[sonuc_metin_boyut - 1] = '\0';
        }
        g_object_unref(parser);
        g_free(satir);
        return false;
    }

    if (json_object_has_member(obj, "tahmini_boyut")) {
        toplam = json_object_get_int_member(obj, "tahmini_boyut");
    }

    g_object_unref(parser);
    g_free(satir);

    // 2) veri_basliyor JSON
    satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
    if (!satir) {
        if (hata) g_error_free(hata);
        return false;
    }

    parser = json_parser_new();
    if (!json_parser_load_from_data(parser, satir, -1, NULL)) {
        g_object_unref(parser);
        g_free(satir);
        return false;
    }

    obj = json_node_get_object(json_parser_get_root(parser));
    if (!obj || !json_object_has_member(obj, "tur") ||
        strcmp(json_object_get_string_member(obj, "tur"), "veri_basliyor") != 0) {
        g_object_unref(parser);
        g_free(satir);
        return false;
    }
    if (json_object_has_member(obj, "toplam")) {
        toplam = json_object_get_int_member(obj, "toplam");
    }

    g_object_unref(parser);
    g_free(satir);

    FILE* out = fopen(yerel_yol, "wb");
    if (!out) {
        if (sonuc_metin && sonuc_metin_boyut > 0) {
            strncpy(sonuc_metin, "Yerel dosya acilamadi", sonuc_metin_boyut - 1);
            sonuc_metin[sonuc_metin_boyut - 1] = '\0';
        }
        return false;
    }

    int64_t okunan_toplam = 0;
    guchar buffer[1024 * 1024];
    GInputStream* raw = G_INPUT_STREAM(baglanti->girdi);

    while (okunan_toplam < toplam) {
        gsize kalan = (gsize)(toplam - okunan_toplam);
        gsize okunacak = kalan < sizeof(buffer) ? kalan : sizeof(buffer);
        gssize okunan = g_input_stream_read(raw, buffer, okunacak, NULL, &hata);
        if (okunan <= 0) {
            fclose(out);
            if (hata) g_error_free(hata);
            return false;
        }

        if (fwrite(buffer, 1, (size_t)okunan, out) != (size_t)okunan) {
            fclose(out);
            if (sonuc_metin && sonuc_metin_boyut > 0) {
                strncpy(sonuc_metin, "Yerel yazma hatasi", sonuc_metin_boyut - 1);
                sonuc_metin[sonuc_metin_boyut - 1] = '\0';
            }
            return false;
        }

        okunan_toplam += okunan;
        if (ilerleme_cb && toplam > 0) {
            ilerleme_cb(okunan_toplam, toplam, kullanici_verisi);
        }
    }

    fclose(out);

    // 3) bitis JSON
    satir = g_data_input_stream_read_line(baglanti->girdi, NULL, NULL, &hata);
    if (!satir) {
        if (hata) g_error_free(hata);
        return false;
    }

    parser = json_parser_new();
    bool basarili = false;
    if (json_parser_load_from_data(parser, satir, -1, NULL)) {
        obj = json_node_get_object(json_parser_get_root(parser));
        if (obj && json_object_has_member(obj, "tur") &&
            strcmp(json_object_get_string_member(obj, "tur"), "bitti") == 0) {
            basarili = true;
            if (sonuc_metin && sonuc_metin_boyut > 0) {
                strncpy(sonuc_metin, "RAM dosyasi indirildi", sonuc_metin_boyut - 1);
                sonuc_metin[sonuc_metin_boyut - 1] = '\0';
            }
        } else if (obj && json_object_has_member(obj, "mesaj") && sonuc_metin && sonuc_metin_boyut > 0) {
            const char* m = json_object_get_string_member(obj, "mesaj");
            strncpy(sonuc_metin, m ? m : "RAM dosya indirme hatasi", sonuc_metin_boyut - 1);
            sonuc_metin[sonuc_metin_boyut - 1] = '\0';
        }
    }

    g_object_unref(parser);
    g_free(satir);
    return basarili;
}
