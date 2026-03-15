#ifndef WIREGUARD_H
#define WIREGUARD_H

#ifdef __cplusplus
extern "C" {
#endif

// WireGuard yonetici yapisi
typedef struct WireGuardYonetici WireGuardYonetici;

// Yonetici olusturma/yok etme
WireGuardYonetici* wireguard_yonetici_olustur(void);
void wireguard_yonetici_yok_et(WireGuardYonetici* yonetici);

// WireGuard islemleri
int wireguard_baslat(WireGuardYonetici* yonetici, const char* config_dosya);
int wireguard_durdur(WireGuardYonetici* yonetici);
int wireguard_durum(WireGuardYonetici* yonetici);

// Config olusturma
int wireguard_config_olustur(const char* dosya_yolu, const char* private_key,
                              const char* public_key, const char* endpoint,
                              const char* allowed_ips, const char* address,
                              const char* dns, int keepalive);

#ifdef __cplusplus
}
#endif

#endif // WIREGUARD_H
