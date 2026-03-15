#include "wireguard.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#ifdef _WIN32
#include <windows.h>
#else
#include <unistd.h>
#include <sys/wait.h>
#include <signal.h>
#endif

// WireGuard arayuz yonetici
struct WireGuardYonetici {
    char interface_adi[32];
    char config_dosya[512];
    long wg_quick_pid;
    int aktif;
};

WireGuardYonetici* wireguard_yonetici_olustur(void) {
    WireGuardYonetici* yonetici = (WireGuardYonetici*)malloc(sizeof(WireGuardYonetici));
    if (!yonetici) return NULL;
    
    memset(yonetici, 0, sizeof(WireGuardYonetici));
    strcpy(yonetici->interface_adi, "wg0");
    yonetici->config_dosya[0] = '\0';
    yonetici->wg_quick_pid = -1;
    yonetici->aktif = 0;
    
    return yonetici;
}

void wireguard_yonetici_yok_et(WireGuardYonetici* yonetici) {
    if (!yonetici) return;
    
    if (yonetici->aktif) {
        wireguard_durdur(yonetici);
    }
    
    free(yonetici);
}

int wireguard_baslat(WireGuardYonetici* yonetici, const char* config_dosya) {
    if (!yonetici || !config_dosya) return -1;
    
    if (yonetici->aktif) {
        fprintf(stderr, "WireGuard zaten aktif\n");
        return 0;
    }
    
    strncpy(yonetici->config_dosya, config_dosya, sizeof(yonetici->config_dosya) - 1);
    yonetici->config_dosya[sizeof(yonetici->config_dosya) - 1] = '\0';
    
    // wg-quick ile baslat
#ifdef _WIN32
    (void)config_dosya;
    fprintf(stderr, "Windows surumunde WireGuard otomatik baslatma su an desteklenmiyor\n");
    return -1;
#else
    pid_t pid = fork();
    if (pid == -1) {
        perror("fork");
        return -1;
    }
    
    if (pid == 0) {
        // Cocuk surec
        execlp("wg-quick", "wg-quick", "up", yonetici->config_dosya, NULL);
        perror("execlp wg-quick");
        exit(1);
    }
    
    // Ana surec - bekle
    int status;
    if (waitpid(pid, &status, 0) == -1) {
        perror("waitpid");
        return -1;
    }
    
    if (WIFEXITED(status) && WEXITSTATUS(status) == 0) {
        yonetici->aktif = 1;
        yonetici->wg_quick_pid = pid;
        printf("WireGuard baslatildi: %s\n", yonetici->config_dosya);
        return 0;
    }
    
    fprintf(stderr, "WireGuard baslatilamadi (exit code: %d)\n", WEXITSTATUS(status));
    return -1;
#endif
}

int wireguard_durdur(WireGuardYonetici* yonetici) {
    if (!yonetici) return -1;
    
    if (!yonetici->aktif) {
        return 0;
    }
    
    
#ifdef _WIN32
    fprintf(stderr, "Windows surumunde WireGuard otomatik durdurma su an desteklenmiyor\n");
    yonetici->aktif = 0;
    yonetici->wg_quick_pid = -1;
    return -1;
#else
    pid_t pid = fork();
    if (pid == -1) {
        perror("fork");
        return -1;
    }
    
    if (pid == 0) {
        execlp("wg-quick", "wg-quick", "down", yonetici->config_dosya, NULL);
        perror("execlp wg-quick");
        exit(1);
    }
    
    int status;
    if (waitpid(pid, &status, 0) == -1) {
        perror("waitpid");
        return -1;
    }
    
    yonetici->aktif = 0;
    yonetici->wg_quick_pid = -1;
    printf("WireGuard durduruldu\n");
    
    return 0;
#endif
}

int wireguard_durum(WireGuardYonetici* yonetici) {
    if (!yonetici) return 0;
    return yonetici->aktif;
}

int wireguard_config_olustur(const char* dosya_yolu, const char* private_key,
                              const char* public_key, const char* endpoint,
                              const char* allowed_ips, const char* address,
                              const char* dns, int keepalive) {
    FILE* f = fopen(dosya_yolu, "w");
    if (!f) {
        perror("fopen");
        return -1;
    }
    
    fprintf(f, "[Interface]\n");
    fprintf(f, "PrivateKey = %s\n", private_key ? private_key : "YOUR_PRIVATE_KEY");
    fprintf(f, "Address = %s\n", (address && *address) ? address : "10.0.0.2/24");
    fprintf(f, "DNS = %s\n", (dns && *dns) ? dns : "1.1.1.1");
    fprintf(f, "\n[Peer]\n");
    fprintf(f, "PublicKey = %s\n", public_key ? public_key : "SERVER_PUBLIC_KEY");
    fprintf(f, "Endpoint = %s\n", endpoint ? endpoint : "192.168.1.1:51820");
    fprintf(f, "AllowedIPs = %s\n", allowed_ips ? allowed_ips : "0.0.0.0/0, ::/0");
    fprintf(f, "PersistentKeepalive = %d\n", keepalive > 0 ? keepalive : 25);
    
    fclose(f);
    printf("WireGuard config olusturuldu: %s\n", dosya_yolu);
    return 0;
}
