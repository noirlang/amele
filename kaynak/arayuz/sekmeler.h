#ifndef SEKMELER_H
#define SEKMELER_H

#include <gtk/gtk.h>
#include "arayuz.h"

void sekmeler_olustur(ArayuzYonetici* arayuz);
GtkWidget* sekme_genel_olustur(ArayuzYonetici* arayuz);
GtkWidget* sekme_sistem_bilgisi_olustur(ArayuzYonetici* arayuz);
GtkWidget* sekme_hash_islemleri_olustur(ArayuzYonetici* arayuz);
GtkWidget* sekme_kanit_kasasi_olustur(ArayuzYonetici* arayuz);
GtkWidget* sekme_raporlar_olustur(ArayuzYonetici* arayuz);
GtkWidget* sekme_gunluk_olustur(ArayuzYonetici* arayuz);
GtkWidget* sekme_uzak_disk_edinim_olustur(ArayuzYonetici* arayuz);

#endif
