project('worm', ['c', 'cpp'],
    version : '0.0.4',
    default_options : ['warning_level=2', 'c_std=c11', 'cpp_std=c++17']
)

# Gerekli kutuphaneleri bul
qt6_dep = dependency('qt6', modules : ['Core', 'Gui', 'Widgets'])
json_glib_dep = dependency('json-glib-1.0')
glib_dep = dependency('glib-2.0')
openssl_dep = dependency('openssl')

# Dependencies listesi
deps = [
    qt6_dep,
    json_glib_dep,
    glib_dep,
    openssl_dep,
]

# Kaynak dosyalari
sources = files(
    'kaynak/qt/ana_qt.cpp',
    'kaynak/cekirdek/gunluk.c',
    'kaynak/cekirdek/ayarlar.c',
    'kaynak/cekirdek/is_kuyrugu.c',
    'kaynak/cekirdek/hata_yonetim.c',
    'kaynak/cekirdek/guvenlik.c',
    'kaynak/moduller/edinim/disk_edinim.c',
    'kaynak/moduller/edinim/uzak_disk_edinim.c',
    'kaynak/moduller/edinim/hyperv_artifact.c',
    'kaynak/moduller/edinim/winpmem_ram.c',
    'kaynak/moduller/edinim/avml_ram.c',
    'kaynak/moduller/dogrulama/hash.c',
    'kaynak/moduller/rapor/rapor.c',
    'kaynak/moduller/kanit/kanit_kasasi.c',
    'kaynak/moduller/vpn/wireguard.c',
)

if host_machine.system() == 'windows'
    sources += files('kaynak/qt/worm.rc')
endif

# Include dizinleri
inc_dirs = include_directories(
    'kaynak',
    'kaynak/cekirdek',
    'kaynak/qt',
    'kaynak/moduller/edinim',
    'kaynak/moduller/dogrulama',
    'kaynak/moduller/rapor',
    'kaynak/moduller/kanit',
    'kaynak/moduller/vpn',
)

# Ana uygulama
target_name = 'worm'

app = executable(target_name,
    sources,
    include_directories : inc_dirs,
    dependencies : deps,
    install : true,
    install_dir : 'bin',
    c_args : [
        '-Wall',
        '-Wextra',
        '-D_GNU_SOURCE',
    ],
    cpp_args : [
        '-DQT_NO_KEYWORDS',
    ],
)

# Varsayilan ayarlari ve UI dosyalarini kur
install_data(
    [
        'veriler/varsayilan_ayarlar.json',
        'veriler/ui_dil.json',
    ],
    install_dir : get_option('datadir') / 'worm'
)

# Logo dosyalarini kur (ana sayfa logosu + uygulama ikonu)
install_subdir(
    'logo',
    install_dir : get_option('datadir') / 'worm'
)

# Masaustu entegrasyonu icin desktop dosyasi ve ikon
if host_machine.system() != 'windows'
    worm_icon = configure_file(
        input : 'logo/icon.png',
        output : 'worm.png',
        copy : true
    )

    install_data('packaging/worm.desktop', install_dir : get_option('datadir') / 'applications')
    install_data(worm_icon, install_dir : get_option('datadir') / 'icons' / 'hicolor' / '256x256' / 'apps')
endif