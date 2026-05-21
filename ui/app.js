import { androidModePage, androidPage, handleAndroidAction, syncAndroidDeviceSelection } from "./android.js";

const icons = {
  home: '<path d="m3 11 9-8 9 8"/><path d="M5 10v10h5v-6h4v6h5V10"/>',
  grid: '<rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/>',
  tiles: '<rect x="4" y="4" width="6" height="6"/><rect x="14" y="4" width="6" height="6"/><rect x="4" y="14" width="6" height="6"/><rect x="14" y="14" width="6" height="6"/>',
  linux: '<circle cx="12" cy="6" r="3"/><path d="M8.2 11.2c.6-1.8 1.9-3.2 3.8-3.2s3.2 1.4 3.8 3.2l1.4 4.3c.5 1.5-.6 3-2.2 3H9c-1.6 0-2.7-1.5-2.2-3l1.4-4.3Z"/><path d="M9 18.5 6.5 21"/><path d="M15 18.5l2.5 2.5"/><path d="M10.2 6h.01"/><path d="M13.8 6h.01"/><path d="M10 13h4"/>',
  android: '<path fill="currentColor" stroke="none" d="M18.4395 5.5586c-.675 1.1664-1.352 2.3318-2.0274 3.498-.0366-.0155-.0742-.0286-.1113-.043-1.8249-.6957-3.484-.8-4.42-.787-1.8551.0185-3.3544.4643-4.2597.8203-.084-.1494-1.7526-3.021-2.0215-3.4864a1.1451 1.1451 0 0 0-.1406-.1914c-.3312-.364-.9054-.4859-1.379-.203-.475.282-.7136.9361-.3886 1.5019 1.9466 3.3696-.0966-.2158 1.9473 3.3593.0172.031-.4946.2642-1.3926 1.0177C2.8987 12.176.452 14.772 0 18.9902h24c-.119-1.1108-.3686-2.099-.7461-3.0683-.7438-1.9118-1.8435-3.2928-2.7402-4.1836a12.1048 12.1048 0 0 0-2.1309-1.6875c.6594-1.122 1.312-2.2559 1.9649-3.3848.2077-.3615.1886-.7956-.0079-1.1191a1.1001 1.1001 0 0 0-.8515-.5332c-.5225-.0536-.9392.3128-1.0488.5449zm-.0391 8.461c.3944.5926.324 1.3306-.1563 1.6503-.4799.3197-1.188.0985-1.582-.4941-.3944-.5927-.324-1.3307.1563-1.6504.4727-.315 1.1812-.1086 1.582.4941zM7.207 13.5273c.4803.3197.5506 1.0577.1563 1.6504-.394.5926-1.1038.8138-1.584.4941-.48-.3197-.5503-1.0577-.1563-1.6504.4008-.6021 1.1087-.8106 1.584-.4941z"/>',
  network: '<circle cx="12" cy="5" r="3"/><circle cx="5" cy="19" r="3"/><circle cx="19" cy="19" r="3"/><path d="M10.5 7.5 6.5 16"/><path d="M13.5 7.5 17.5 16"/><path d="M8 19h8"/>',
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/><path d="M8 11h6"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 10v6"/><path d="M12 7h.01"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .34 1.87l.04.04a2 2 0 1 1-2.83 2.83l-.04-.04A1.7 1.7 0 0 0 15 19.4a1.7 1.7 0 0 0-1 .6 1.7 1.7 0 0 0-.4 1.1V21a2 2 0 0 1-4 0v-.1A1.7 1.7 0 0 0 8.6 19.4a1.7 1.7 0 0 0-1.87.34l-.04.04a2 2 0 1 1-2.83-2.83l.04-.04A1.7 1.7 0 0 0 4.6 15a1.7 1.7 0 0 0-.6-1 1.7 1.7 0 0 0-1.1-.4H3a2 2 0 0 1 0-4h.1A1.7 1.7 0 0 0 4.6 8.6a1.7 1.7 0 0 0-.34-1.87l-.04-.04a2 2 0 1 1 2.83-2.83l.04.04A1.7 1.7 0 0 0 9 4.6a1.7 1.7 0 0 0 1-.6 1.7 1.7 0 0 0 .4-1.1V3a2 2 0 0 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.87-.34l.04-.04a2 2 0 1 1 2.83 2.83l-.04.04A1.7 1.7 0 0 0 19.4 9c.36.12.7.32 1 .6.3.28.5.63.6 1h.1a2 2 0 0 1 0 4H21a1.7 1.7 0 0 0-1.6.4Z"/>',
  menu: '<path d="M4 7h16"/><path d="M4 12h16"/><path d="M4 17h16"/>',
  disk: '<path d="M4 5h16l-2 10H6L4 5Z"/><path d="M7 19h10"/><path d="M9 15v4"/><path d="M15 15v4"/>',
  shield: '<path d="M12 3 5 6v6c0 4.5 3 7.5 7 9 4-1.5 7-4.5 7-9V6l-7-3Z"/><path d="m9 12 2 2 4-5"/>',
  scale: '<path d="M12 3v18"/><path d="M5 7h14"/><path d="M6 7l-3 6h6L6 7Z"/><path d="M18 7l-3 6h6l-3-6Z"/>',
  report: '<path d="M7 3h8l4 4v14H7V3Z"/><path d="M15 3v5h5"/><path d="M10 13h6"/><path d="M10 17h6"/>',
  monitor: '<rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8"/><path d="M12 17v4"/>',
  database: '<ellipse cx="12" cy="5" rx="7" ry="3"/><path d="M5 5v7c0 1.7 3.1 3 7 3s7-1.3 7-3V5"/><path d="M5 12v7c0 1.7 3.1 3 7 3s7-1.3 7-3v-7"/>',
  chip: '<rect x="7" y="7" width="10" height="10" rx="2"/><path d="M9 1v4"/><path d="M15 1v4"/><path d="M9 19v4"/><path d="M15 19v4"/><path d="M1 9h4"/><path d="M1 15h4"/><path d="M19 9h4"/><path d="M19 15h4"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v6l4 2"/>',
  windows: '<path d="M3 5.5 11 4v7H3V5.5Z"/><path d="M13 3.7 21 2v9h-8V3.7Z"/><path d="M3 13h8v7l-8-1.5V13Z"/><path d="M13 13h8v9l-8-1.7V13Z"/>',
  ram: '<rect x="3" y="7" width="18" height="10" rx="2"/><path d="M7 7V4"/><path d="M12 7V4"/><path d="M17 7V4"/><path d="M7 20v-3"/><path d="M12 20v-3"/><path d="M17 20v-3"/>',
  folder: '<path d="M3 6h7l2 2h9v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6Z"/>',
  download: '<path d="M12 3v12"/><path d="m7 10 5 5 5-5"/><path d="M5 21h14"/>',
  globe: '<circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3a14 14 0 0 1 0 18"/><path d="M12 3a14 14 0 0 0 0 18"/>',
  user: '<circle cx="12" cy="8" r="4"/><path d="M4 21c1.8-4 4.4-6 8-6s6.2 2 8 6"/>',
  key: '<circle cx="8" cy="15" r="4"/><path d="m11 12 9-9"/><path d="m15 4 3 3"/><path d="m13 6 3 3"/>',
  refresh: '<path d="M21 12a9 9 0 0 1-15.5 6.2L3 16"/><path d="M3 21v-5h5"/><path d="M3 12A9 9 0 0 1 18.5 5.8L21 8"/><path d="M21 3v5h-5"/>',
  pause: '<path d="M8 5v14"/><path d="M16 5v14"/>',
  play: '<path d="M8 5v14l11-7Z"/>',
  stop: '<rect x="6" y="6" width="12" height="12"/>',
  arrow: '<path d="M5 12h14"/><path d="m13 5 7 7-7 7"/>'
};

const APP_VERSION = "v0.0.7";
const assetPath = "./assets";
const backendAvailable = location.protocol === "http:" || location.protocol === "https:";
const isNativeWebView = new URLSearchParams(window.location.search).get("native") === "1";
if (isNativeWebView) document.documentElement.classList.add("native-webview");
const fontIcons = {
  windows: "",
  linux: "",
  github: "",
  linkedin: "",
  website: "",
  globe: ""
};

const app = document.querySelector("#app");
const view = document.querySelector("#view");
const preferredLanguage = localStorage.getItem("worm-language") || "tr";

const translations = {
  tr: {
    "nav.home": "Ana Sayfa",
    "nav.windows": "Windows Araçları",
    "nav.linux": "Linux Araçları",
    "nav.android": "Android Araçları",
    "nav.agent": "Agent",
    "nav.analysis": "Analiz",
    "nav.other": "Diğer",
    ready: "Hazır",
    settingsSaved: "Ayarlar kaydedildi.",
    fileRequired: "Önce dosya seçin.",
    platformBlocked: "Bu yerel işlem yalnızca {platform} üzerinde çalışır.",
    unknown: "Bilinmiyor",
    select: "Seç",
    open: "Aç",
    localUnsupported: "Bu sistemde yerel çalışmaz",
    remoteAgent: "Uzak agent",
    localOperation: "Yerel işlem",
    targetNotSelected: "Hedef seçilmedi",
    localCheckWaiting: "Yerel kontrol bekleniyor",
    notConnected: "Henüz bağlanmadı",
    lastActionReady: "Hazır",
    diskNotSelected: "Disk seçilmedi",
    scanDisksFirst: "Diskleri tara ile listeleyin",
    imageAcquisition: "İmaj alma",
    ramAcquisition: "RAM edinimi",
    acquisitionFailed: "Edinim başarısız",
    "log.appReady": "Uygulama tam modda açıldı.",
    "log.previewMode": "Önizleme modunda yerel işlemler kullanılamaz.",
    "log.agentProtocol": "Agent protokolü hazır.",
    "log.workflowsReady": "Disk ve RAM işlemleri hazır.",
    "home.acquire.title": "Edinim",
    "home.acquire.desc": "Windows ve Linux için disk/RAM toplama akışları.",
    "home.integrity.title": "Bütünlük",
    "home.integrity.desc": "MD5, SHA ailesi ve karşılaştırma adımları.",
    "home.evidence.title": "Kanıt",
    "home.evidence.desc": "Vaka klasörü ve kanıt kasası yönetimi.",
    "home.output.title": "Çıktı",
    "home.output.desc": "İnceleme notları ve rapor üretimi.",
    "hub.detected": "Algılanan sistem: {platform}. Yerel disk/RAM işlemleri sadece aynı işletim sisteminde açılır; uzak agent akışları platform bağımsızdır.",
    "hub.windows.title": "Windows Araçları",
    "hub.windows.desc": "Windows yerel/uzak disk ve RAM edinim akışlarını seçin.",
    "hub.linux.title": "Linux Araçları",
    "hub.linux.desc": "Linux yerel/uzak disk ve RAM edinim akışlarını seçin.",
    "hub.android.title": "Android Araçları",
    "hub.android.desc": "Mobil adli bilişim araçları için çalışma alanı.",
    "hub.android.empty": "Android modülleri sonraki adımda eklenecek.",
    "android.mode.physical.title": "Fiziksel İmaj",
    "android.mode.physical.desc": "Cihaz bootloader veya EDL modundayken, üretici ya da çipset seviyesindeki erişimle mümkün olan en düşük seviyede imaj alınır.",
    "android.mode.physical.badge": "Bootloader / EDL",
    "android.mode.logical.title": "Mantıksal İmaj",
    "android.mode.logical.desc": "ADB ve USB hata ayıklama açıkken rehber, mesajlar, arama kayıtları, medya dosyaları ve erişilebilir kullanıcı verileri toplanır.",
    "android.mode.logical.badge": "ADB",
    "android.mode.filesystem.title": "Dosya Sistemi İmajı",
    "android.mode.filesystem.desc": "Mantıksal imajdan daha derin, fiziksel imajdan daha yüzeyseldir. Root erişimi veya özel açıklarla protected alanlar dahil dosya sistemi alınır.",
    "android.mode.filesystem.badge": "Root / exploit",
    "android.back": "Android Araçları",
    "android.appModeRequired": "Android araçları uygulama modunda çalışır.",
    "android.adb.title": "ADB Kontrol",
    "android.adb.unknown": "Henüz kontrol edilmedi",
    "android.adb.installed": "ADB kurulu",
    "android.adb.missing": "ADB bulunamadı",
    "android.adb.check": "ADB Kontrol Et",
    "android.adb.checkHint": "ADB durumunu kontrol edin.",
    "android.adb.checkFirst": "Önce ADB kontrolü yapın.",
    "android.adb.checkFailed": "ADB kontrolü başarısız: {message}",
    "android.devices.title": "Cihazlar",
    "android.devices.list": "Cihazları Listele",
    "android.devices.select": "Cihaz Seç",
    "android.devices.none": "Cihaz bulunamadı",
    "android.devices.listed": "{count} cihaz listelendi.",
    "android.devices.selected": "Seçili cihaz: {serial}",
    "android.devices.listFailed": "Cihaz listesi alınamadı: {message}",
    "android.logical.caseTitle": "Vaka",
    "android.logical.caseHint": "Android imaj çıktısı seçilen vakanın android klasörüne yazılır. Vaka yoksa yeni vaka adıyla otomatik oluşturulur.",
    "android.logical.acquisitionTitle": "İmaj Alma",
    "android.logical.start": "Mantıksal İmaj Al",
    "android.logical.stop": "Durdur",
    "android.logical.stopped": "İmaj alma durduruldu.",
    "android.logical.starting": "İmaj alma başlatılıyor...",
    "android.logical.progress": "İlerleme",
    "android.logical.waiting": "İmaj alma bekleniyor.",
    "android.logical.done": "Mantıksal imaj alma tamamlandı.",
    "android.logical.failed": "İmaj alma başarısız: {message}",
    "android.logical.deviceRequired": "Önce cihaz seçin.",
    "android.side.status": "Durum",
    "android.side.adb": "ADB",
    "android.side.device": "Cihaz",
    "android.side.lastAction": "Son İşlem",
    "android.side.totalBytes": "Toplam Boyut",
    "workflow.ip": "IP Adresi",
    "workflow.ipPlaceholder": "IP adresi",
    "workflow.port": "Port",
    "workflow.token": "Token",
    "workflow.tokenPlaceholder": "Güvenlik anahtarı (Onayla ile aktif olur)",
    "workflow.approveKey": "Anahtarı Onayla",
    "workflow.reset": "Sıfırla",
    "workflow.networkVpn": "2. Ağ ve VPN",
    "workflow.useVpn": "WireGuard VPN Kullan",
    "workflow.configureVpn": "VPN Yapılandır",
    "workflow.server": "Sunucu",
    "workflow.allowedIps": "İzinli IP'ler",
    "workflow.vpnPrivateKey": "Private Key",
    "workflow.vpnPublicKey": "Server Public Key",
    "workflow.vpnAddress": "Adres",
    "workflow.vpnDns": "DNS",
    "workflow.vpnKeepalive": "Keepalive",
    "workflow.configFile": "Config Dosyası",
    "workflow.saveVpn": "VPN Kaydet",
    "workflow.startVpn": "VPN Başlat",
    "workflow.stopVpn": "VPN Durdur",
    "workflow.connectionOps": "3. Bağlantı İşlemleri",
    "workflow.connect": "Bağlan",
    "workflow.scanDisks": "Diskleri Tara",
    "workflow.check": "Kontrol",
    "workflow.checkTool": "{tool} Kontrol",
    "workflow.localCheck": "1. Yerel Kontrol",
    "workflow.localHint": "{platform} yerel akışında yönetici/root yetkisi gerekebilir. İşlem başlamadan önce araç ve erişim kontrolü yapılır.",
    "workflow.checkToolAction": "{tool} Kontrol Et",
    "workflow.scanLocalDisks": "Yerel Diskleri Tara",
    "workflow.downloadWinpmem": "WinPMEM İndir",
    "workflow.winpmemInstalling": "WinPMEM indiriliyor ve C:\\Tools altına kuruluyor",
    "workflow.winpmemInstalled": "WinPMEM kuruldu: {path}",
    "workflow.winpmemInstallFailed": "WinPMEM kurulamadı: {message}",
    "workflow.winpmemUnsupported": "WinPMEM otomatik kurulumu sadece Windows yerel RAM akışında çalışır.",
    "workflow.downloadAvml": "AVML İndir ve Kur",
    "workflow.avmlInstalling": "AVML indiriliyor ve /usr/bin/avml olarak kuruluyor",
    "workflow.avmlInstalled": "AVML kuruldu: {path}",
    "workflow.avmlInstallFailed": "AVML kurulamadı: {message}",
    "workflow.avmlUnsupported": "AVML otomatik kurulumu sadece Linux yerel RAM akışında çalışır.",
    "workflow.appModeRequired": "Bu işlem uygulama modunda çalışır.",
    "workflow.ramOutput": "RAM ve Çıktı",
    "workflow.diskOutput": "Disk ve Çıktı",
    "workflow.caseSection": "4. Vaka",
    "workflow.case": "Vaka",
    "workflow.caseHint": "İmaj çıktısı seçilen vakanın ciktilar klasörüne yazılır. Vaka yoksa yeni vaka adıyla otomatik oluşturulur.",
    "workflow.ramCaseHint": "RAM çıktısı seçilen vakanın ram klasörüne yazılır. Vaka yoksa yeni vaka adıyla otomatik oluşturulur.",
    "workflow.newCase": "Yeni vaka oluştur",
    "workflow.newCaseName": "Yeni Vaka Adı",
    "workflow.caseOutput": "Vaka çıktı klasörü",
    "workflow.outputFile": "Çıktı Dosyası",
    "workflow.outputFileName": "Çıktı Dosya Adı",
    "workflow.outputFolder": "Çıktı Klasörü",
    "workflow.tool": "Araç",
    "workflow.disk": "Disk",
    "workflow.startRam": "RAM Edinimini Başlat",
    "workflow.startImage": "İmaj Al",
    "workflow.controls": "5. İşlem Kontrolleri",
    "workflow.pause": "Duraklat",
    "workflow.resume": "Devam",
    "workflow.stop": "Durdur",
    "workflow.progress": "6. İlerleme Durumu",
    "workflow.status": "İşlem Durumu",
    "workflow.platform": "Platform",
    "workflow.connection": "Bağlantı",
    "workflow.target": "Hedef",
    "workflow.lastAction": "Son işlem",
    "workflow.operationRunning": "{operation} çalışıyor",
    "workflow.operationStarted": "{operation} başlatıldı.",
    "workflow.operationCompleted": "{operation} tamamlandı",
    "workflow.operationCompletedPath": "{operation} tamamlandı: {path}",
    "workflow.operationFailed": "{operation} başarısız",
    "workflow.operationFailedDetail": "{operation} başarısız: {message}",
    "workflow.outputRequired": "Çıktı konumu seçin.",
    "workflow.diskRequired": "Önce hedef disk seçin.",
    "workflow.jobIdMissing": "İş kimliği alınamadı.",
    "workflow.activeJobMissing": "Aktif edinim işi yok.",
    "workflow.pauseFailed": "Duraklatma gönderilemedi: {message}",
    "workflow.resumeFailed": "Devam komutu gönderilemedi: {message}",
    "workflow.stopFailed": "Durdurma gönderilemedi: {message}",
    "workflow.controlApplied": "{label} komutu uygulandı.",
    "workflow.controlSent": "{label} komutu agent'a gönderildi.",
    "workflow.pauseLabel": "Duraklatma",
    "workflow.resumeLabel": "Devam",
    "workflow.stopLabel": "Durdurma",
    "workflow.selectFile": "Dosya seçildi: {path}",
    "workflow.filePickerFailed": "Dosya seçimi açılamadı: {message}",
    "workflow.filePickerFailedShort": "Dosya seçimi açılamadı.",
    "workflow.selectFolder": "Klasör seçildi: {path}",
    "workflow.folderPickerFailed": "Klasör seçimi açılamadı: {message}",
    "workflow.folderPickerFailedShort": "Klasör seçimi açılamadı.",
    "connection.keyApproveFirst": "Güvenlik anahtarını kullanmak için önce Anahtarı Onayla butonuna basın.",
    "connection.keyChanged": "Güvenlik anahtarı değişti. Yeniden Anahtarı Onayla yapın veya Sıfırla kullanın.",
    "connection.connectFirst": "Önce bağlanın.",
    "connection.none": "Bağlantı yok",
    "connection.required": "Önce geçerli agent bağlantısı kurulmalı.",
    "connection.ipPortRequired": "IP ve port girin.",
    "connection.invalidPort": "Geçersiz port.",
    "connection.remoteOnly": "Bu akış uzak bağlantı kullanmıyor.",
    "connection.connecting": "Bağlanıyor... (Zaman aşımı: 10sn)",
    "connection.starting": "Bağlantı başlatılıyor: {host}",
    "connection.connected": "Bağlandı - {ip}",
    "connection.connectedLog": "Uzak sunucuya bağlandı: {ip}",
    "connection.success": "Bağlantı başarılı. Disk/RAM kontrolünü şimdi çalıştırabilirsiniz.",
    "connection.failed": "Bağlantı başarısız.",
    "connection.failedLog": "Bağlantı başarısız: {ip} - {message}",
    "connection.cannotConnect": "Sunucuya bağlanılamadı: {message}",
    "connection.checked": "{host} kontrol edildi",
    "connection.alive": "{host} bağlı",
    "connection.toolFailed": "Ajanla kontrol başarısız",
    "connection.disksFailed": "Diskler alınamadı - bağlantı kopmuş olabilir",
    "scan.toolDoneLog": "{target} kontrolü: {message}",
    "scan.done": "Kontrol tamamlandı.",
    "scan.failed": "Kontrol başarısız: {message}",
    "scan.ramFailedLog": "RAM kontrolü başarısız: {message}",
    "scan.toolListUpdated": "Araç listesi güncellendi.",
    "scan.noDisk": "Disk bulunamadı",
    "scan.accessDenied": "erişim yok",
    "scan.elevated": "Disk listesi yetki yükseltilerek alındı.",
    "scan.elevationFailed": "Yetki yükseltme başarısız: {message}",
    "scan.noDiskLog": "Disk bulunamadı veya erişim izni yok.",
    "scan.diskDoneLog": "Disk listesi güncellendi.",
    "scan.diskDone": "Disk taraması tamamlandı.",
    "scan.diskFailed": "Disk taraması başarısız: {message}",
    "scan.diskFailedShort": "Disk taraması başarısız oldu.",
    "scan.waiting": "Disk listesi için uygulama modu bekleniyor",
    "scan.appModeRequired": "Disk taraması uygulama modunda çalışır.",
    "scan.completed": "Tarama tamamlandı.",
    "vpn.enabled": "VPN kullanımı açıldı.",
    "vpn.disabled": "VPN kullanımı kapatıldı.",
    "vpn.waiting": "VPN yapılandırması bekleniyor",
    "vpn.off": "VPN kapalı",
    "vpn.opened": "VPN yapılandırma alanı açıldı.",
    "vpn.endpointRequired": "VPN sunucu bilgisini girin.",
    "vpn.configRequired": "Önce VPN config dosyasını seçin.",
    "vpn.configured": "VPN yapılandırıldı: {endpoint}",
    "vpn.ready": "VPN hazır",
    "vpn.saved": "VPN yapılandırması kaydedildi.",
    "vpn.started": "VPN başlatıldı.",
    "vpn.stopped": "VPN durduruldu.",
    "vpn.failed": "VPN işlemi başarısız: {message}",
    "key.required": "Onaylamak için güvenlik anahtarı girin.",
    "key.approved": "Güvenlik anahtarı onaylandı.",
    "key.active": "Güvenlik anahtarı aktif.",
    "key.reset": "Güvenlik anahtarı sıfırlandı.",
    "analysis.title": "İmaj Görüntüleme",
    "analysis.desc": "Seçilen disk imajını salt-okunur olarak bağlar ve içeriğini klasör ağacında gösterir.",
    "analysis.hint": "İmaj dosyasını seçin, salt-okunur bağlayın ve içerik ağacını bu ekrandan inceleyin.",
    "analysis.imageFile": "İmaj Dosyası",
    "analysis.mount": "Salt-Okunur Bağla",
    "analysis.unmount": "Bağlantıyı Kaldır",
    "analysis.status": "Durum",
    "analysis.noImage": "İmaj seçilmedi",
    "analysis.outputWaiting": "Klasör ağacı ve bağlama çıktısı burada görüntülenecek.",
    "analysis.imageRequired": "Önce imaj dosyası seçin.",
    "analysis.mounting": "İmaj bağlanıyor...",
    "analysis.mounted": "Bağlandı: {path}",
    "analysis.mountedLog": "İmaj salt-okunur bağlandı. İçerik ağacı aşağıda gösteriliyor.",
    "analysis.mountPrepared": "İmaj bağlama işlemi hazırlandı.",
    "analysis.mountFailed": "İmaj bağlanamadı: {message}",
    "analysis.unmounted": "Bağlantı kaldırıldı",
    "analysis.noActiveMount": "Aktif imaj bağlantısı yok.",
    "analysis.unmountFailed": "Bağlantı kaldırılamadı: {message}",
    "other.title": "Diğer",
    "other.desc": "Hash işlemleri, kanıt kasası, rapor üretimi ve canlı günlük modülleri.",
    "other.hash.title": "Hash İşlemleri",
    "other.hash.desc": "MD5, SHA1, SHA256 ve SHA512 hesaplama.",
    "other.evidence.title": "Kanıt Kasası",
    "other.evidence.desc": "Vaka klasörü ve kanıt kasası yönetimi.",
    "other.reports.title": "Raporlar",
    "other.reports.desc": "İnceleme notları ve rapor üretimi.",
    "other.logs.title": "Günlük",
    "other.logs.desc": "Canlı günlük ve dosyadan yenileme akışı.",
    "hash.calculator": "Hash Hesaplayıcı",
    "hash.file": "Dosya",
    "hash.selectFile": "Dosya seçin",
    "hash.calculate": "Hesapla",
    "hash.compare": "Hash Karşılaştır",
    "hash.value": "Hash Değeri",
    "hash.placeholder": "Hash değeri girin",
    "hash.result": "Sonuç",
    "hash.waiting": "Karşılaştırma bekleniyor",
    "hash.done": "Hash hesaplama tamamlandı.",
    "hash.failed": "Hash hesaplama başarısız: {message}",
    "hash.fullAppRequired": "Uygulama modu gerekli",
    "hash.compareRequired": "Karşılaştırılacak hash değerini girin.",
    "hash.matched": "Eşleşti",
    "hash.notMatched": "Eşleşmedi",
    "hash.matchedToast": "Hash eşleşti.",
    "hash.notMatchedToast": "Hash eşleşmedi.",
    "settings.title": "Ayarlar",
    "settings.desc": "Tema, dil ve güncelleme kontrolleri.",
    "settings.appearance": "Görünüm",
    "settings.appSettings": "Uygulama Ayarları",
    "settings.persisted": "Tema ve dil tercihi kaydedilir; sayfa yenilendiğinde korunur.",
    "settings.darkTheme": "Karanlık Tema",
    "settings.darkHint": "Adli bilişim çalışma ekranları için düşük parlaklık.",
    "settings.language": "Dil",
    "settings.languageHint": "Menü dili ve uygulama mesajları.",
    "settings.detectedSystem": "Algılanan Sistem",
    "settings.detectedHint": "Yerel işlem filtreleri buna göre çalışır.",
    "settings.save": "Ayarları Kaydet",
    "settings.version": "Sürüm",
    "settings.update": "Güncelleme",
    "settings.updateDesc": "Kurulum dosyasını platforma göre seçer, indirme ilerlemesini ve release notlarını burada gösterir.",
    "settings.installed": "Kurulu",
    "settings.checkUpdate": "Güncellemeyi Kontrol Et",
    "settings.downloadInstall": "İndir ve Kur",
    "settings.releaseNotes": "Release notları ve indirme durumu burada görüntülenecek.",
    "settings.updateChecked": "Güncelleme kontrol edildi",
    "settings.updateLog": "Kurulu sürüm: {version}<br />Son sürüm bilgisi burada gösterilecek.",
    "settings.updateDone": "Güncelleme kontrolü tamamlandı.",
    "settings.updateFailed": "Güncelleme kontrolü başarısız: {message}",
    "settings.latestVersion": "Son sürüm: {version}",
    "settings.noAsset": "Bu platform için indirme paketi bulunamadı.",
    "settings.downloading": "İndiriliyor",
    "settings.downloadReady": "İndirme hazır",
    "settings.packageReady": "Güncelleme paketi hazır.",
    "settings.downloadFailed": "İndirme başarısız: {message}",
    "settings.downloaded": "İndirildi: {path}",
    "settings.sha256": "SHA256: {hash}",
    "settings.installing": "Kurulum başlatılıyor",
    "settings.installStarted": "Kurulum başlatıldı.",
    "settings.installFailed": "Kurulum başlatılamadı: {message}",
    "about.version": "Sürüm {version}",
    "about.desc": "Worm, yetkili adli bilişim süreçlerinde disk ve RAM edinimi, doğrulama ve raporlama adımlarını tek bir merkezde birleştiren bir denetim aracıdır.",
    "about.capabilities": "Temel Kabiliyetler",
    "about.collect.title": "Disk ve RAM",
    "about.collect.desc": "Windows ve Linux için imaj ve bellek edinimi.",
    "about.prove.title": "Doğrulama",
    "about.prove.desc": "Hash üretimi, karşılaştırma ve denetlenebilir loglar.",
    "about.package.title": "Raporlama",
    "about.package.desc": "Vaka notları, kanıt kasası ve rapor çıktıları.",
    "about.usage": "Kullanım İlkesi",
    "about.usageDesc": "Bu araç yalnızca yetkili adli bilişim süreçlerinde kullanılmalıdır. Edinim, doğrulama ve günlük adımları görünür, denetlenebilir ve raporlanabilir tutulur.",
    "about.maintainers": "Proje Sorumluları",
    "about.role.lead": "Lider Geliştirici",
    "about.role.windows": "Windows Sorumlusu",
    "about.role.linux": "Linux Sorumlusu",
    "agent.desc": "Windows ve Linux agent kullanım özetleri.",
    "agent.windowsNote": "Windows Agent kullanım özeti. Dosyayı Windows üzerinde yönetici olarak çalıştırın ve ana uygulamadaki IP/Port bilgisiyle eşleştirin.",
    "agent.linuxNote": "Linux Agent kullanım özeti. Çalıştırılabilir izin verin, agentı başlatın ve ana uygulamadaki IP/Port ile bağlanın.",
    "agent.downloadWin": "Agent indirin: wget -O worm-win.exe https://worm.noirlang.tr/worm-win.exe",
    "agent.runWin": "Windows'ta worm-win.exe dosyasını yönetici olarak çalıştırın.",
    "agent.match": "Ana uygulamadaki IP/Port bilgisi ile eşleştirin.",
    "agent.downloadLinux": "Agent indirin: wget -O worm-linux https://worm.noirlang.tr/worm-linux",
    "agent.chmod": "Yetki verin: chmod +x worm-linux",
    "agent.runLinux": "Çalıştırın: ./worm-linux",
    "agent.connect": "Ana uygulamadaki IP/Port ile bağlantı kurun.",
    "case.management": "Vaka Yönetimi",
    "case.name": "Vaka Adı",
    "case.baseDir": "Vaka Kök Klasörü",
    "case.location": "Vaka Konumu",
    "case.fixedLocation": "Vakalar otomatik olarak Worm/Vakalar altında oluşturulur.",
    "case.select": "Vaka Seç",
    "case.noCases": "Kayıtlı vaka yok",
    "case.refresh": "Vakaları Yenile",
    "case.loaded": "{count} vaka yüklendi.",
    "case.create": "Vaka Oluştur",
    "case.notCreated": "Vaka oluşturulmadı",
    "case.required": "Önce vaka oluşturun.",
    "case.created": "Vaka oluşturuldu: {path}",
    "case.createFailed": "Vaka oluşturulamadı: {message}",
    "case.files": "Dosyalar",
    "case.folder": "Klasör",
    "case.file": "Dosya",
    "case.outputs": "Çıktılar / ciktilar",
    "case.diskImages": "Disk İmajları / disk_imajlari",
    "case.ram": "RAM / ram",
    "case.reports": "Raporlar / raporlar",
    "case.hash": "Hash / hash",
    "case.notes": "Notlar / notlar",
    "case.logs": "Günlükler / gunlukler",
    "case.listFilesPlaceholder": "Dosyaları listeleyin...",
    "case.listFiles": "Dosyaları Listele",
    "case.filesListed": "{count} dosya listelendi.",
    "case.empty": "Bu klasör boş.",
    "case.listFailed": "Dosyalar listelenemedi: {message}",
    "report.createTitle": "Rapor Oluştur",
    "report.hint": "Vaka seçin; hiç vaka yoksa rapor oluştururken otomatik vaka açılır.",
    "report.case": "Rapor Vakası",
    "report.autoCase": "Otomatik açılacak vaka",
    "report.title": "Rapor Başlığı",
    "report.defaultTitle": "Adli Bilişim Teknik Raporu",
    "report.format": "Format",
    "report.note": "Not",
    "report.notePlaceholder": "Not veya rapor açıklaması girin",
    "report.addNote": "Not Ekle",
    "report.noteRequired": "Not alanı boş olamaz.",
    "report.noteAdded": "Not eklendi: {path}",
    "report.noteFailed": "Not eklenemedi: {message}",
    "report.created": "Rapor oluşturuldu: {path}",
    "report.failed": "Rapor oluşturulamadı: {message}",
    "log.live": "Canlı günlük burada da görüntülenir.",
    "log.refreshFromFile": "Dosyadan Günlüğü Yenile"
  },
  en: {
    "nav.home": "Home",
    "nav.windows": "Windows Tools",
    "nav.linux": "Linux Tools",
    "nav.android": "Android Tools",
    "nav.agent": "Agent",
    "nav.analysis": "Analysis",
    "nav.other": "Other",
    ready: "Ready",
    settingsSaved: "Settings saved.",
    fileRequired: "Select a file first.",
    platformBlocked: "This local workflow only runs on {platform}.",
    unknown: "Unknown",
    select: "Select",
    open: "Open",
    localUnsupported: "Local workflow is unavailable on this system",
    remoteAgent: "Remote agent",
    localOperation: "Local operation",
    targetNotSelected: "No target selected",
    localCheckWaiting: "Waiting for local check",
    notConnected: "Not connected",
    lastActionReady: "Ready",
    diskNotSelected: "No disk selected",
    scanDisksFirst: "Scan disks to list targets",
    imageAcquisition: "Image acquisition",
    ramAcquisition: "RAM acquisition",
    acquisitionFailed: "Acquisition failed",
    "log.appReady": "Application is running in full mode.",
    "log.previewMode": "Local operations are unavailable in preview mode.",
    "log.agentProtocol": "Agent protocol is ready.",
    "log.workflowsReady": "Disk and RAM workflows are ready.",
    "home.acquire.title": "Acquisition",
    "home.acquire.desc": "Disk/RAM collection workflows for Windows and Linux.",
    "home.integrity.title": "Integrity",
    "home.integrity.desc": "MD5, SHA family, and comparison steps.",
    "home.evidence.title": "Evidence",
    "home.evidence.desc": "Case folder and evidence vault management.",
    "home.output.title": "Output",
    "home.output.desc": "Review notes and report generation.",
    "hub.detected": "Detected system: {platform}. Local disk/RAM workflows open only on the matching operating system; remote agent workflows are platform independent.",
    "hub.windows.title": "Windows Tools",
    "hub.windows.desc": "Select local/remote disk and RAM acquisition workflows for Windows.",
    "hub.linux.title": "Linux Tools",
    "hub.linux.desc": "Select local/remote disk and RAM acquisition workflows for Linux.",
    "hub.android.title": "Android Tools",
    "hub.android.desc": "Workspace for mobile forensic tools.",
    "hub.android.empty": "Android modules will be added in the next step.",
    "android.mode.physical.title": "Physical Image",
    "android.mode.physical.desc": "Acquires the lowest-level image available while the device is in bootloader or EDL mode through vendor or chipset-level access.",
    "android.mode.physical.badge": "Bootloader / EDL",
    "android.mode.logical.title": "Logical Image",
    "android.mode.logical.desc": "Collects contacts, messages, call history, media files, and accessible user data through ADB when USB debugging is enabled.",
    "android.mode.logical.badge": "ADB",
    "android.mode.filesystem.title": "File System Image",
    "android.mode.filesystem.desc": "Deeper than a logical image and shallower than a physical image. Uses root access or specific exploits to capture the file system, including protected areas.",
    "android.mode.filesystem.badge": "Root / exploit",
    "android.back": "Android Tools",
    "android.appModeRequired": "Android tools require application mode.",
    "android.adb.title": "ADB Check",
    "android.adb.unknown": "Not checked yet",
    "android.adb.installed": "ADB installed",
    "android.adb.missing": "ADB not found",
    "android.adb.check": "Check ADB",
    "android.adb.checkHint": "Check ADB status.",
    "android.adb.checkFirst": "Check ADB first.",
    "android.adb.checkFailed": "ADB check failed: {message}",
    "android.devices.title": "Devices",
    "android.devices.list": "List Devices",
    "android.devices.select": "Select Device",
    "android.devices.none": "No device found",
    "android.devices.listed": "{count} devices listed.",
    "android.devices.selected": "Selected device: {serial}",
    "android.devices.listFailed": "Device list failed: {message}",
    "android.logical.caseTitle": "Case",
    "android.logical.caseHint": "Android image output is written to the selected case's android folder. If no case exists, a new one is created automatically.",
    "android.logical.acquisitionTitle": "Acquisition",
    "android.logical.start": "Acquire Logical Image",
    "android.logical.stop": "Stop",
    "android.logical.stopped": "Acquisition stopped.",
    "android.logical.starting": "Starting acquisition...",
    "android.logical.progress": "Progress",
    "android.logical.waiting": "Waiting for acquisition.",
    "android.logical.done": "Logical image acquisition completed.",
    "android.logical.failed": "Acquisition failed: {message}",
    "android.logical.deviceRequired": "Select a device first.",
    "android.side.status": "Status",
    "android.side.adb": "ADB",
    "android.side.device": "Device",
    "android.side.lastAction": "Last Action",
    "android.side.totalBytes": "Total Size",
    "workflow.ip": "IP Address",
    "workflow.ipPlaceholder": "IP address",
    "workflow.port": "Port",
    "workflow.token": "Token",
    "workflow.tokenPlaceholder": "Security key (approve to activate)",
    "workflow.approveKey": "Approve Key",
    "workflow.reset": "Reset",
    "workflow.networkVpn": "2. Network and VPN",
    "workflow.useVpn": "Use WireGuard VPN",
    "workflow.configureVpn": "Configure VPN",
    "workflow.server": "Server",
    "workflow.allowedIps": "Allowed IPs",
    "workflow.vpnPrivateKey": "Private Key",
    "workflow.vpnPublicKey": "Server Public Key",
    "workflow.vpnAddress": "Address",
    "workflow.vpnDns": "DNS",
    "workflow.vpnKeepalive": "Keepalive",
    "workflow.configFile": "Config File",
    "workflow.saveVpn": "Save VPN",
    "workflow.startVpn": "Start VPN",
    "workflow.stopVpn": "Stop VPN",
    "workflow.connectionOps": "3. Connection Actions",
    "workflow.connect": "Connect",
    "workflow.scanDisks": "Scan Disks",
    "workflow.check": "Check",
    "workflow.checkTool": "Check {tool}",
    "workflow.localCheck": "1. Local Check",
    "workflow.localHint": "{platform} local workflows may require administrator/root privileges. Tool and access checks run before acquisition starts.",
    "workflow.checkToolAction": "Check {tool}",
    "workflow.scanLocalDisks": "Scan Local Disks",
    "workflow.downloadWinpmem": "Download WinPMEM",
    "workflow.winpmemInstalling": "Downloading WinPMEM and installing it under C:\\Tools",
    "workflow.winpmemInstalled": "WinPMEM installed: {path}",
    "workflow.winpmemInstallFailed": "WinPMEM install failed: {message}",
    "workflow.winpmemUnsupported": "Automatic WinPMEM install only works in the Windows local RAM workflow.",
    "workflow.downloadAvml": "Download and Install AVML",
    "workflow.avmlInstalling": "Downloading AVML and installing it as /usr/bin/avml",
    "workflow.avmlInstalled": "AVML installed: {path}",
    "workflow.avmlInstallFailed": "AVML install failed: {message}",
    "workflow.avmlUnsupported": "Automatic AVML install only works in the Linux local RAM workflow.",
    "workflow.appModeRequired": "This action requires application mode.",
    "workflow.ramOutput": "RAM and Output",
    "workflow.diskOutput": "Disk and Output",
    "workflow.caseSection": "4. Case",
    "workflow.case": "Case",
    "workflow.caseHint": "The image output is written to the selected case's ciktilar folder. If no case exists, a new one is created automatically.",
    "workflow.ramCaseHint": "The RAM output is written to the selected case's ram folder. If no case exists, a new one is created automatically.",
    "workflow.newCase": "Create new case",
    "workflow.newCaseName": "New Case Name",
    "workflow.caseOutput": "Case output folder",
    "workflow.outputFile": "Output File",
    "workflow.outputFileName": "Output File Name",
    "workflow.outputFolder": "Output Folder",
    "workflow.tool": "Tool",
    "workflow.disk": "Disk",
    "workflow.startRam": "Start RAM Acquisition",
    "workflow.startImage": "Acquire Image",
    "workflow.controls": "5. Operation Controls",
    "workflow.pause": "Pause",
    "workflow.resume": "Resume",
    "workflow.stop": "Stop",
    "workflow.progress": "6. Progress",
    "workflow.status": "Operation Status",
    "workflow.platform": "Platform",
    "workflow.connection": "Connection",
    "workflow.target": "Target",
    "workflow.lastAction": "Last action",
    "workflow.operationRunning": "{operation} running",
    "workflow.operationStarted": "{operation} started.",
    "workflow.operationCompleted": "{operation} completed",
    "workflow.operationCompletedPath": "{operation} completed: {path}",
    "workflow.operationFailed": "{operation} failed",
    "workflow.operationFailedDetail": "{operation} failed: {message}",
    "workflow.outputRequired": "Select an output location.",
    "workflow.diskRequired": "Select a target disk first.",
    "workflow.jobIdMissing": "Could not get job id.",
    "workflow.activeJobMissing": "No active acquisition job.",
    "workflow.pauseFailed": "Could not send pause command: {message}",
    "workflow.resumeFailed": "Could not send resume command: {message}",
    "workflow.stopFailed": "Could not send stop command: {message}",
    "workflow.controlApplied": "{label} command applied.",
    "workflow.controlSent": "{label} command sent to agent.",
    "workflow.pauseLabel": "Pause",
    "workflow.resumeLabel": "Resume",
    "workflow.stopLabel": "Stop",
    "workflow.selectFile": "File selected: {path}",
    "workflow.filePickerFailed": "File picker could not be opened: {message}",
    "workflow.filePickerFailedShort": "File picker could not be opened.",
    "workflow.selectFolder": "Folder selected: {path}",
    "workflow.folderPickerFailed": "Folder picker could not be opened: {message}",
    "workflow.folderPickerFailedShort": "Folder picker could not be opened.",
    "connection.keyApproveFirst": "Approve the security key before using it.",
    "connection.keyChanged": "The security key changed. Approve it again or reset it.",
    "connection.connectFirst": "Connect first.",
    "connection.none": "No connection",
    "connection.required": "A valid agent connection is required first.",
    "connection.ipPortRequired": "Enter IP and port.",
    "connection.invalidPort": "Invalid port.",
    "connection.remoteOnly": "This workflow does not use a remote connection.",
    "connection.connecting": "Connecting... (timeout: 10s)",
    "connection.starting": "Starting connection: {host}",
    "connection.connected": "Connected - {ip}",
    "connection.connectedLog": "Connected to remote server: {ip}",
    "connection.success": "Connection successful. You can now run disk/RAM checks.",
    "connection.failed": "Connection failed.",
    "connection.failedLog": "Connection failed: {ip} - {message}",
    "connection.cannotConnect": "Could not connect to server: {message}",
    "connection.checked": "{host} checked",
    "connection.alive": "{host} connected",
    "connection.toolFailed": "Agent check failed",
    "connection.disksFailed": "Could not load disks - connection may be lost",
    "scan.toolDoneLog": "{target} check: {message}",
    "scan.done": "Check completed.",
    "scan.failed": "Check failed: {message}",
    "scan.ramFailedLog": "RAM check failed: {message}",
    "scan.toolListUpdated": "Tool list updated.",
    "scan.noDisk": "No disk found",
    "scan.accessDenied": "no access",
    "scan.elevated": "Disk list loaded with elevated permission.",
    "scan.elevationFailed": "Elevation failed: {message}",
    "scan.noDiskLog": "No disk found or access is denied.",
    "scan.diskDoneLog": "Disk list updated.",
    "scan.diskDone": "Disk scan completed.",
    "scan.diskFailed": "Disk scan failed: {message}",
    "scan.diskFailedShort": "Disk scan failed.",
    "scan.waiting": "Application mode is required to list disks",
    "scan.appModeRequired": "Disk scan works in application mode.",
    "scan.completed": "Scan completed.",
    "vpn.enabled": "VPN enabled.",
    "vpn.disabled": "VPN disabled.",
    "vpn.waiting": "VPN configuration pending",
    "vpn.off": "VPN off",
    "vpn.opened": "VPN configuration panel opened.",
    "vpn.endpointRequired": "Enter the VPN server.",
    "vpn.configRequired": "Select the VPN config file first.",
    "vpn.configured": "VPN configured: {endpoint}",
    "vpn.ready": "VPN ready",
    "vpn.saved": "VPN configuration saved.",
    "vpn.started": "VPN started.",
    "vpn.stopped": "VPN stopped.",
    "vpn.failed": "VPN action failed: {message}",
    "key.required": "Enter the security key to approve it.",
    "key.approved": "Security key approved.",
    "key.active": "Security key active.",
    "key.reset": "Security key reset.",
    "analysis.title": "Image Viewer",
    "analysis.desc": "Mounts the selected disk image read-only and shows its contents as a folder tree.",
    "analysis.hint": "Select an image file, mount it read-only, and inspect the content tree here.",
    "analysis.imageFile": "Image File",
    "analysis.mount": "Mount Read-Only",
    "analysis.unmount": "Remove Mount",
    "analysis.status": "Status",
    "analysis.noImage": "No image selected",
    "analysis.outputWaiting": "Folder tree and mount output will appear here.",
    "analysis.imageRequired": "Select an image file first.",
    "analysis.mounting": "Mounting image...",
    "analysis.mounted": "Mounted: {path}",
    "analysis.mountedLog": "Image mounted read-only. The content tree is shown below.",
    "analysis.mountPrepared": "Image mount prepared.",
    "analysis.mountFailed": "Image could not be mounted: {message}",
    "analysis.unmounted": "Mount removed",
    "analysis.noActiveMount": "No active image mount.",
    "analysis.unmountFailed": "Mount could not be removed: {message}",
    "other.title": "Other",
    "other.desc": "Hash operations, evidence vault, report generation, and live log modules.",
    "other.hash.title": "Hash Operations",
    "other.hash.desc": "Calculate MD5, SHA1, SHA256, and SHA512.",
    "other.evidence.title": "Evidence Vault",
    "other.evidence.desc": "Case folder and evidence vault management.",
    "other.reports.title": "Reports",
    "other.reports.desc": "Review notes and report generation.",
    "other.logs.title": "Log",
    "other.logs.desc": "Live log and file refresh flow.",
    "hash.calculator": "Hash Calculator",
    "hash.file": "File",
    "hash.selectFile": "Select a file",
    "hash.calculate": "Calculate",
    "hash.compare": "Compare Hash",
    "hash.value": "Hash Value",
    "hash.placeholder": "Enter hash value",
    "hash.result": "Result",
    "hash.waiting": "Waiting for comparison",
    "hash.done": "Hash calculation completed.",
    "hash.failed": "Hash calculation failed: {message}",
    "hash.fullAppRequired": "Application mode required",
    "hash.compareRequired": "Enter the hash value to compare.",
    "hash.matched": "Matched",
    "hash.notMatched": "Not matched",
    "hash.matchedToast": "Hash matched.",
    "hash.notMatchedToast": "Hash did not match.",
    "settings.title": "Settings",
    "settings.desc": "Theme, language, and update controls.",
    "settings.appearance": "Appearance",
    "settings.appSettings": "Application Settings",
    "settings.persisted": "Theme and language preferences are saved and restored on reload.",
    "settings.darkTheme": "Dark Theme",
    "settings.darkHint": "Lower brightness for forensic work screens.",
    "settings.language": "Language",
    "settings.languageHint": "Menu language and application messages.",
    "settings.detectedSystem": "Detected System",
    "settings.detectedHint": "Local workflow filters use this value.",
    "settings.save": "Save Settings",
    "settings.version": "Version",
    "settings.update": "Update",
    "settings.updateDesc": "Selects the installer for the platform and shows download progress and release notes here.",
    "settings.installed": "Installed",
    "settings.checkUpdate": "Check for Updates",
    "settings.downloadInstall": "Download and Install",
    "settings.releaseNotes": "Release notes and download status will appear here.",
    "settings.updateChecked": "Update checked",
    "settings.updateLog": "Installed version: {version}<br />Latest version information will appear here.",
    "settings.updateDone": "Update check completed.",
    "settings.updateFailed": "Update check failed: {message}",
    "settings.latestVersion": "Latest version: {version}",
    "settings.noAsset": "No download package was found for this platform.",
    "settings.downloading": "Downloading",
    "settings.downloadReady": "Download ready",
    "settings.packageReady": "Update package ready.",
    "settings.downloadFailed": "Download failed: {message}",
    "settings.downloaded": "Downloaded: {path}",
    "settings.sha256": "SHA256: {hash}",
    "settings.installing": "Starting installer",
    "settings.installStarted": "Installer started.",
    "settings.installFailed": "Installer could not be started: {message}",
    "about.version": "Version {version}",
    "about.desc": "Worm is an audit tool that brings disk/RAM acquisition, verification, and reporting steps into one place for authorized forensic workflows.",
    "about.capabilities": "Core Capabilities",
    "about.collect.title": "Disk and RAM",
    "about.collect.desc": "Image and memory acquisition for Windows and Linux.",
    "about.prove.title": "Verification",
    "about.prove.desc": "Hash generation, comparison, and auditable logs.",
    "about.package.title": "Reporting",
    "about.package.desc": "Case notes, evidence vault, and report outputs.",
    "about.usage": "Use Policy",
    "about.usageDesc": "This tool should be used only in authorized forensic workflows. Acquisition, verification, and log steps remain visible, auditable, and reportable.",
    "about.maintainers": "Maintainers",
    "about.role.lead": "Lead Maintainer",
    "about.role.windows": "Windows Maintainer",
    "about.role.linux": "Linux Maintainer",
    "agent.desc": "Windows and Linux agent usage summaries.",
    "agent.windowsNote": "Windows Agent summary. Run the file as Administrator on Windows and match it with the IP/Port in the main application.",
    "agent.linuxNote": "Linux Agent summary. Make it executable, start the agent, and connect from the main application using IP/Port.",
    "agent.downloadWin": "Download agent: wget -O worm-win.exe https://worm.noirlang.tr/worm-win.exe",
    "agent.runWin": "Run worm-win.exe as Administrator on Windows.",
    "agent.match": "Match it with the IP/Port shown in the main application.",
    "agent.downloadLinux": "Download agent: wget -O worm-linux https://worm.noirlang.tr/worm-linux",
    "agent.chmod": "Make it executable: chmod +x worm-linux",
    "agent.runLinux": "Run: ./worm-linux",
    "agent.connect": "Connect using the IP/Port from the main application.",
    "case.management": "Case Management",
    "case.name": "Case Name",
    "case.baseDir": "Case Root Folder",
    "case.location": "Case Location",
    "case.fixedLocation": "Cases are created automatically under Worm/Vakalar.",
    "case.select": "Select Case",
    "case.noCases": "No saved cases",
    "case.refresh": "Refresh Cases",
    "case.loaded": "{count} cases loaded.",
    "case.create": "Create Case",
    "case.notCreated": "No case created",
    "case.required": "Create a case first.",
    "case.created": "Case created: {path}",
    "case.createFailed": "Case could not be created: {message}",
    "case.files": "Files",
    "case.folder": "Folder",
    "case.file": "File",
    "case.outputs": "Outputs / ciktilar",
    "case.diskImages": "Disk Images / disk_imajlari",
    "case.ram": "RAM / ram",
    "case.reports": "Reports / raporlar",
    "case.hash": "Hash / hash",
    "case.notes": "Notes / notlar",
    "case.logs": "Logs / gunlukler",
    "case.listFilesPlaceholder": "List files...",
    "case.listFiles": "List Files",
    "case.filesListed": "{count} files listed.",
    "case.empty": "This folder is empty.",
    "case.listFailed": "Files could not be listed: {message}",
    "report.createTitle": "Create Report",
    "report.hint": "Select a case; if none exists, a case is created automatically when the report is generated.",
    "report.case": "Report Case",
    "report.autoCase": "Case to create automatically",
    "report.title": "Report Title",
    "report.defaultTitle": "Forensic Technical Report",
    "report.format": "Format",
    "report.note": "Note",
    "report.notePlaceholder": "Enter a note or report description",
    "report.addNote": "Add Note",
    "report.noteRequired": "Note cannot be empty.",
    "report.noteAdded": "Note added: {path}",
    "report.noteFailed": "Note could not be added: {message}",
    "report.created": "Report created: {path}",
    "report.failed": "Report could not be created: {message}",
    "log.live": "Live log is also shown here.",
    "log.refreshFromFile": "Refresh Log From File"
  }
};

function translate(language, key, vars = {}) {
  let value = translations[language]?.[key] || translations.tr[key] || key;
  for (const [name, replacement] of Object.entries(vars)) {
    value = value.replaceAll(`{${name}}`, replacement);
  }
  return value;
}

function initialLogMessages(language) {
  return [
    translate(language, backendAvailable ? "log.appReady" : "log.previewMode"),
    translate(language, "log.agentProtocol"),
    translate(language, "log.workflowsReady")
  ];
}

const state = {
  route: new URLSearchParams(window.location.search).get("route") || "home",
  theme: localStorage.getItem("worm-theme") || "dark",
  language: preferredLanguage,
  platform: detectPlatform(),
  files: {},
  activeTab: "hash",
  approvedSecurityKey: "",
  remoteConnections: {},
  activeAcquisition: null,
  activeCase: null,
  cases: [],
  caseBaseDir: "",
  imageMount: null,
  latestUpdate: null,
  android: {
    adbStatus: null,
    devices: [],
    selectedDevice: ""
  },
  jobs: {},
  cachedDefaultCaseName: "",
  lastLog: initialLogMessages(preferredLanguage)
};

function detectPlatform() {
  const override = new URLSearchParams(window.location.search).get("platform");
  if (["windows", "linux", "android", "mac"].includes(override || "")) return override;
  const text = `${navigator.userAgent} ${navigator.platform}`.toLowerCase();
  if (text.includes("android")) return "android";
  if (text.includes("win")) return "windows";
  if (text.includes("linux")) return "linux";
  if (text.includes("mac")) return "mac";
  return "unknown";
}

function t(key, vars = {}) {
  return translate(state.language, key, vars);
}

function L(tr, en) {
  return { tr, en };
}

function localText(value) {
  if (value && typeof value === "object" && "tr" in value) {
    return value[state.language] || value.tr;
  }
  return value;
}

const toolCards = {
  windows: [
    {
      id: "windows-remote-disk",
      title: L("Uzak Disk İmajı", "Remote Disk Image"),
      desc: L("Windows agent üzerinden PhysicalDrive imajı alın.", "Acquire a PhysicalDrive image through the Windows agent."),
      icon: "disk",
      accent: "var(--green)",
      badge: "Agent + raw stream"
    },
    {
      id: "windows-local-disk",
      title: L("Yerel Disk İmajı", "Local Disk Image"),
      desc: L("Bu makinedeki Windows disklerinden ham imaj üretin.", "Create a raw image from Windows disks on this machine."),
      icon: "windows",
      accent: "var(--blue)",
      badge: "PhysicalDrive"
    },
    {
      id: "windows-remote-ram",
      title: L("Uzak RAM", "Remote RAM"),
      desc: L("WinPMEM ile uzak Windows RAM edinimi başlatın ve indirin.", "Start and download remote Windows RAM acquisition with WinPMEM."),
      icon: "ram",
      accent: "var(--purple)",
      badge: "WinPMEM remote"
    },
    {
      id: "windows-local-ram",
      title: L("Yerel RAM", "Local RAM"),
      desc: L("Yerel WinPMEM kontrolü, indirme ve RAM imajı alma.", "Check local WinPMEM, download if needed, and acquire RAM."),
      icon: "chip",
      accent: "var(--amber)",
      badge: L("Yönetici gerekli", "Admin required")
    }
  ],
  linux: [
    {
      id: "linux-remote-disk",
      title: L("Uzak Disk İmajı", "Remote Disk Image"),
      desc: L("Linux agent üzerinden /dev disklerinden ham imaj alın.", "Acquire raw images from /dev disks through the Linux agent."),
      icon: "disk",
      accent: "var(--green)",
      badge: "Agent + /dev"
    },
    {
      id: "linux-local-disk",
      title: L("Yerel Disk İmajı", "Local Disk Image"),
      desc: L("Yerel Linux diskleri için root yetkili imaj alma akışı.", "Root-level acquisition workflow for local Linux disks."),
      icon: "linux",
      accent: "var(--blue)",
      badge: "BLKGETSIZE64"
    },
    {
      id: "linux-remote-ram",
      title: L("Uzak RAM", "Remote RAM"),
      desc: L("AVML ile uzak Linux RAM edinimi ve dosya indirme.", "Acquire remote Linux RAM with AVML and download the dump file."),
      icon: "ram",
      accent: "var(--purple)",
      badge: "AVML remote"
    },
    {
      id: "linux-local-ram",
      title: L("Yerel RAM", "Local RAM"),
      desc: L("AVML varlık/yetki kontrolü ve yerel RAM dump üretimi.", "Check AVML availability/privileges and create a local RAM dump."),
      icon: "chip",
      accent: "var(--amber)",
      badge: L("Root gerekli", "Root required")
    }
  ]
};

const workflows = {
  "windows-remote-disk": {
    platform: "Windows",
    icon: "windows",
    title: L("Uzak Windows Sunucu Bağlantısı", "Remote Windows Server Connection"),
    desc: L("Uzak Windows sistemlerine güvenli bağlantı kurun ve disk imajı alın.", "Connect securely to remote Windows systems and acquire disk images."),
    mode: "remote-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: L("Disk seçilmedi", "No disk selected")
  },
  "linux-remote-disk": {
    platform: "Linux",
    icon: "linux",
    title: L("Uzak Linux Disk Bağlantısı", "Remote Linux Disk Connection"),
    desc: L("Linux agent ile uzak /dev disklerini listeleyin ve raw imaj alın.", "List remote /dev disks through the Linux agent and acquire raw images."),
    mode: "remote-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: L("Disk seçilmedi", "No disk selected")
  },
  "windows-local-disk": {
    platform: "Windows",
    icon: "windows",
    title: L("Windows Yerel Disk İmajı", "Windows Local Disk Image"),
    desc: L("Yerel PhysicalDrive kaynaklarından ham imaj alma akışı.", "Raw image acquisition workflow for local PhysicalDrive sources."),
    mode: "local-disk",
    output: "C:\\Worm\\Ciktilar",
    diskLabel: L("Disk seçilmedi", "No disk selected")
  },
  "linux-local-disk": {
    platform: "Linux",
    icon: "linux",
    title: L("Linux Yerel Disk İmajı", "Linux Local Disk Image"),
    desc: L("Yerel Linux blok cihazlarından imaj alma akışı.", "Image acquisition workflow for local Linux block devices."),
    mode: "local-disk",
    output: "/home/raodrin/Worm/Ciktilar",
    diskLabel: L("Disk seçilmedi", "No disk selected")
  },
  "windows-remote-ram": {
    platform: "Windows",
    icon: "ram",
    title: L("Windows Uzak RAM Edinimi", "Windows Remote RAM Acquisition"),
    desc: L("WinPMEM durumunu kontrol edin, uzak RAM edinimini başlatın ve dump dosyasını indirin.", "Check WinPMEM, start remote RAM acquisition, and download the dump file."),
    mode: "remote-ram",
    output: "memory_dump.raw",
    diskLabel: "WinPMEM"
  },
  "linux-remote-ram": {
    platform: "Linux",
    icon: "ram",
    title: L("Linux Uzak RAM Edinimi", "Linux Remote RAM Acquisition"),
    desc: L("AVML durumunu kontrol edin, uzak RAM edinimini başlatın ve dump dosyasını indirin.", "Check AVML, start remote RAM acquisition, and download the dump file."),
    mode: "remote-ram",
    output: "memory_dump_linux.raw",
    diskLabel: "AVML"
  },
  "windows-local-ram": {
    platform: "Windows",
    icon: "chip",
    title: L("Windows Yerel RAM Edinimi", "Windows Local RAM Acquisition"),
    desc: L("Yerel WinPMEM kontrolü, gerekirse indirme ve RAM imajı alma.", "Check local WinPMEM, download if needed, and acquire a RAM image."),
    mode: "local-ram",
    output: "memory_dump_local.raw",
    diskLabel: L("WinPMEM local", "Local WinPMEM")
  },
  "linux-local-ram": {
    platform: "Linux",
    icon: "chip",
    title: L("Linux Yerel RAM Edinimi", "Linux Local RAM Acquisition"),
    desc: L("Yerel AVML kontrolü ve root yetkili RAM imajı alma.", "Check local AVML and acquire RAM with root privileges."),
    mode: "local-ram",
    output: "linux_memory_dump.raw",
    diskLabel: L("AVML local", "Local AVML")
  }
};

function icon(name) {
  if (fontIcons[name]) {
    return `<span class="fa-icon" aria-hidden="true">${fontIcons[name]}</span>`;
  }
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${icons[name] || icons.info}</svg>`;
}

function hydrateIcons(root = document) {
  root.querySelectorAll("[data-icon]").forEach((node) => {
    node.innerHTML = icon(node.dataset.icon);
  });
}

function setRoute(route) {
  if (route.startsWith("workflow:")) {
    const workflow = workflows[route.split(":")[1]];
    if (workflow && isLocalWorkflowBlocked(workflow)) {
      showToast(t("platformBlocked", { platform: workflow.platform }), "error");
      return;
    }
  }
  state.route = route;
  render();
}

function isLocalWorkflowBlocked(workflow) {
  if (!workflow.mode.startsWith("local")) return false;
  return workflow.platform.toLowerCase() !== state.platform;
}

function setTheme(theme) {
  state.theme = theme;
  localStorage.setItem("worm-theme", theme);
  app.classList.toggle("theme-light", theme === "light");
  app.classList.toggle("theme-dark", theme !== "light");
}

function setLanguage(language) {
  state.language = language;
  localStorage.setItem("worm-language", language);
  document.documentElement.lang = language;
  document.querySelectorAll("[data-i18n]").forEach((node) => {
    node.textContent = t(node.dataset.i18n);
  });
}

function render() {
  const activeGroup = routeGroup(state.route);
  document.querySelectorAll("[data-route]").forEach((button) => {
    button.classList.toggle("active", button.dataset.route === activeGroup);
  });

  if (state.route.startsWith("workflow:")) {
    view.innerHTML = workflowPage(state.route.split(":")[1]);
  } else if (state.route.startsWith("android:")) {
    view.innerHTML = androidModePage({
      modeId: state.route.split(":")[1],
      t,
      icon,
      pageTitle,
      state,
      escapeHtml,
      backendReady,
      casePanel,
      field
    });
  } else {
    view.innerHTML = routes[state.route]?.() || homePage();
  }

  hydrateIcons(view);
  if (state.route === "other" && ["evidence", "reports"].includes(state.activeTab)) {
    loadEvidenceCases();
  }
  if (state.route.startsWith("workflow:")) {
    const workflow = workflows[state.route.split(":")[1]];
    if (workflow && workflow.mode.includes("disk")) loadEvidenceCases();
  }
  if (state.route === "android:logical") loadEvidenceCases();
  view.focus({ preventScroll: true });
}

function routeGroup(route) {
  if (route.startsWith("android:")) return "android";
  if (!route.startsWith("workflow:")) return route;
  const workflowId = route.split(":")[1] || "";
  if (workflowId.startsWith("windows")) return "windows";
  if (workflowId.startsWith("linux")) return "linux";
  return route;
}

function homePage() {
  return `
    <section class="page">
      <div class="hero home-hero">
        <div class="worm-art">
          <img src="${assetPath}/logo/logo.png" alt="Worm logo" />
        </div>
      </div>

      <div class="home-grid">
        ${homeTile(t("home.acquire.title"), t("home.acquire.desc"), "ACQUIRE", "disk", "windows", "var(--green)")}
        ${homeTile(t("home.integrity.title"), t("home.integrity.desc"), "VERIFY", "shield", "other", "var(--green)")}
        ${homeTile(t("home.evidence.title"), t("home.evidence.desc"), "CASE", "scale", "other", "var(--purple)")}
        ${homeTile(t("home.output.title"), t("home.output.desc"), "REPORT", "report", "other", "var(--blue)")}
      </div>
    </section>
  `;
}

function homeTile(title, desc, label, iconName, route, accent) {
  return `
    <button class="action-tile" data-route="${route}" style="--accent:${accent}">
      <span class="tile-icon">${icon(iconName)}</span>
      <span>
        <span class="eyebrow">${label}</span>
        <h3>${title}</h3>
        <p>${desc}</p>
      </span>
      <span class="tile-arrow">→</span>
    </button>
  `;
}

function metric(label, value, iconName, accent) {
  return `
    <div class="metric" style="--accent:${accent}">
      <span class="metric-icon">${icon(iconName)}</span>
      <span><small>${label}</small><strong>${value}</strong></span>
    </div>
  `;
}

function toolHub(platform) {
  const cards = toolCards[platform]
    .map(
      (card) => {
        const workflow = workflows[card.id];
        const blocked = workflow && isLocalWorkflowBlocked(workflow);
        return `
        <button class="forensic-card ${blocked ? "is-disabled" : ""}" data-route="workflow:${card.id}" style="--accent:${card.accent}" ${blocked ? `aria-disabled="true" data-disabled-reason="${workflow.platform}"` : ""}>
          <span class="card-icon">${icon(card.icon)}</span>
          <h3>${localText(card.title)}</h3>
          <p>${localText(card.desc)}</p>
          <span class="meta">${blocked ? t("localUnsupported") : localText(card.badge)}</span>
        </button>
      `;
      }
    )
    .join("");

  const isWindows = platform === "windows";
  return `
    <section class="page">
      <div class="platform-note">
        ${icon("monitor")} ${t("hub.detected", { platform: `<strong>${platformLabel(state.platform)}</strong>` })}
      </div>
      ${pageTitle(
        t(isWindows ? "hub.windows.title" : "hub.linux.title"),
        t(isWindows ? "hub.windows.desc" : "hub.linux.desc"),
        isWindows ? "windows" : "linux"
      )}
      <div class="tool-grid">${cards}</div>
    </section>
  `;
}

function platformLabel(platform) {
  if (platform === "windows") return "Windows";
  if (platform === "linux") return "Linux";
  if (platform === "android") return "Android";
  if (platform === "mac") return "macOS";
  return t("unknown");
}

function workflowPage(id) {
  const data = workflows[id] || workflows["windows-remote-disk"];
  const isRemote = data.mode.startsWith("remote");
  const isRam = data.mode.includes("ram");
  const toolCheck = data.platform === "Windows" ? "WinPMEM" : "AVML";
  const initialTarget = isRam ? localText(data.diskLabel) : "";
  const initialTargetLabel = isRam ? localText(data.diskLabel) : t("scanDisksFirst");
  const outputField = isRam ? ramCasePanel() : imageCasePanel();

  return `
    <section class="page">
      <div class="workflow-layout">
        <div class="workflow-panel">
          ${pageTitle(localText(data.title), localText(data.desc), data.icon)}
          <div class="form-grid">
            ${
              isRemote
                ? `
                  ${field(t("workflow.ip"), `<input class="input" data-field="ip" placeholder="${t("workflow.ipPlaceholder")}" value="" />`)}
                  ${field(t("workflow.port"), '<input class="input" data-field="port" value="4444" />')}
                  ${field(t("workflow.token"), `<input class="input" data-field="token" placeholder="${t("workflow.tokenPlaceholder")}" />`)}
                  <div class="button-row">
                    <button class="secondary-button" data-action="approve-key">${icon("key")} ${t("workflow.approveKey")}</button>
                    <button class="secondary-button" data-action="reset-key">${icon("refresh")} ${t("workflow.reset")}</button>
                  </div>
                  <div class="section-divider"></div>
                  <p class="section-label">${t("workflow.networkVpn")}</p>
                  <div class="toggle-row">
                    <span>${t("workflow.useVpn")}</span>
                    <button class="switch" data-action="toggle-vpn" aria-label="${t("workflow.useVpn")}"></button>
                  </div>
                  <button class="secondary-button" data-action="vpn-config">${icon("settings")} ${t("workflow.configureVpn")}</button>
                  <div class="vpn-panel" hidden>
                    ${field(t("workflow.server"), '<input class="input" data-field="vpn-endpoint" placeholder="10.0.0.1:51820" />')}
                    ${field(t("workflow.vpnPrivateKey"), '<input class="input" data-field="vpn-private-key" placeholder="YOUR_PRIVATE_KEY" />')}
                    ${field(t("workflow.vpnPublicKey"), '<input class="input" data-field="vpn-public-key" placeholder="SERVER_PUBLIC_KEY" />')}
                    ${field(t("workflow.allowedIps"), '<input class="input" data-field="vpn-allowed" value="0.0.0.0/0" />')}
                    ${field(t("workflow.vpnAddress"), '<input class="input" data-field="vpn-address" value="10.0.0.2/24" />')}
                    ${field(t("workflow.vpnDns"), '<input class="input" data-field="vpn-dns" value="1.1.1.1" />')}
                    ${field(t("workflow.vpnKeepalive"), '<input class="input" data-field="vpn-keepalive" value="25" />')}
                    ${pickerField(t("workflow.configFile"), "vpn-config-file", "wireguard.conf", "file")}
                    <div class="button-row">
                      <button class="primary-button" data-action="save-vpn">${icon("shield")} ${t("workflow.saveVpn")}</button>
                      <button class="secondary-button" data-action="start-vpn">${icon("play")} ${t("workflow.startVpn")}</button>
                      <button class="danger-button" data-action="stop-vpn">${icon("stop")} ${t("workflow.stopVpn")}</button>
                    </div>
                  </div>
                  <div class="section-divider"></div>
                  <p class="section-label">${t("workflow.connectionOps")}</p>
                  <div class="button-row">
                    <button class="primary-button" data-action="connect">${icon("network")} ${t("workflow.connect")}</button>
                    <button class="secondary-button" data-action="scan">${icon(isRam ? "chip" : "disk")} ${isRam ? t("workflow.checkTool", { tool: toolCheck }) : t("workflow.scanDisks")}</button>
                  </div>
                `
                : `
                  <p class="section-label">${t("workflow.localCheck")}</p>
                  <p class="field-hint">${t("workflow.localHint", { platform: data.platform })}</p>
                  <div class="button-row">
                    <button class="primary-button" data-action="scan">${icon(isRam ? "chip" : "disk")} ${isRam ? t("workflow.checkToolAction", { tool: toolCheck }) : t("workflow.scanLocalDisks")}</button>
                    ${isRam && data.platform === "Windows" ? `<button class="secondary-button" data-action="download">${icon("refresh")} ${t("workflow.downloadWinpmem")}</button>` : ""}
                    ${isRam && data.platform === "Linux" ? `<button class="secondary-button" data-action="install-avml">${icon("download")} ${t("workflow.downloadAvml")}</button>` : ""}
                  </div>
                `
            }

            <div class="section-divider"></div>
            ${
              isRam
                ? `
                  <p class="section-label">4. ${t("workflow.ramOutput")}</p>
                  ${field(t("workflow.tool"), `<select class="select" data-field="target"><option value="${initialTarget}">${initialTargetLabel}</option></select>`)}
                  ${outputField}
                `
                : `
                  <p class="section-label">${t("workflow.caseSection")}</p>
                  ${outputField}
                  <div class="section-divider"></div>
                  <p class="section-label">5. ${t("workflow.diskOutput")}</p>
                  ${field(t("workflow.disk"), `<select class="select" data-field="target"><option value="" disabled selected>${initialTargetLabel}</option></select>`)}
                `
            }
            <button class="primary-button" data-action="start">${icon(isRam ? "ram" : "disk")} ${isRam ? t("workflow.startRam") : t("workflow.startImage")}</button>

            <div class="section-divider"></div>
            <p class="section-label">${t("workflow.controls")}</p>
            <div class="button-row">
              <button class="secondary-button" data-action="pause">${icon("pause")} ${t("workflow.pause")}</button>
              <button class="secondary-button" data-action="resume">${icon("play")} ${t("workflow.resume")}</button>
              <button class="danger-button" data-action="stop">${icon("stop")} ${t("workflow.stop")}</button>
            </div>

            <div class="section-divider"></div>
            <p class="section-label">${t("workflow.progress")}</p>
            <div class="progress-bar" data-progress style="--value:0%"><span></span><b>0%</b></div>
            <div class="log-box" id="workflow-log">${state.lastLog.map((line) => `• ${line}`).join("<br />")}</div>
          </div>
        </div>

        <aside class="side-panel">
          <h3>${t("workflow.status")}</h3>
          ${sideInfo(t("workflow.platform"), `${data.platform} • ${isRemote ? t("remoteAgent") : t("localOperation")}`, data.icon)}
          ${sideInfo(t("workflow.connection"), isRemote ? t("notConnected") : t("localCheckWaiting"), "monitor", "connection")}
          ${sideInfo(isRam ? t("workflow.tool") : t("workflow.target"), initialTarget || t("targetNotSelected"), isRam ? "chip" : "disk", "target")}
          ${sideInfo(t("workflow.lastAction"), t("lastActionReady"), "clock", "last-action")}
        </aside>
      </div>
    </section>
  `;
}

function pageTitle(title, desc, iconName) {
  return `
    <div class="page-title">
      <span class="card-icon">${icon(iconName)}</span>
      <span>
        <h1>${title}</h1>
        <p>${desc}</p>
      </span>
    </div>
  `;
}

function field(label, control) {
  return `
    <div class="field">
      <label>${label}</label>
      ${control}
    </div>
  `;
}

function pickerField(label, id, value, type = "file") {
  const action = type === "folder" ? "pick-folder" : "pick-file";
  const placeholderOnly = value.startsWith(".") || value.toLowerCase().includes("seç") || value.toLowerCase().includes("select");
  const valueAttr = placeholderOnly ? `placeholder="${value}" value=""` : `value="${value}"`;
  return field(
    label,
    `<div class="input-action"><input id="${id}" class="input" ${valueAttr} data-picker-target /><button class="secondary-button" data-action="${action}" data-target="#${id}">${icon(type === "folder" ? "folder" : "search")} ${t("select")}</button></div>`
  );
}

function imageCasePanel() {
  return casePanel("ciktilar", t("workflow.caseHint"));
}

function ramCasePanel() {
  return `
    ${casePanel("ram", t("workflow.ramCaseHint"))}
    ${field(t("workflow.outputFileName"), `<input id="workflow-output" class="input" value="${escapeHtml(canonicalRamFileName())}" readonly />`)}
  `;
}

function casePanel(subdir, hint) {
  const selected = state.activeCase?.case_name || (state.cases.length ? state.cases[0].case_name : "__new__");
  const output = caseOutputLabel(selected, subdir);
  return `
    <p class="field-hint">${hint}</p>
    ${field(t("workflow.case"), `<select id="workflow-case" class="select" data-case-select data-allow-new-case="1">${caseSelectOptions(selected, { allowNew: true })}</select>`)}
    ${field(t("workflow.newCaseName"), `<input id="workflow-case-name" class="input" value="${stableDefaultCaseName()}" />`)}
    <div class="button-row">
      <button class="secondary-button" data-action="refresh-cases">${icon("refresh")} ${t("case.refresh")}</button>
    </div>
    <div class="side-info">
      <span class="metric-icon">${icon("folder")}</span>
      <span><strong>${t("workflow.caseOutput")}</strong><small data-case-output data-case-output-subdir="${subdir}">${escapeHtml(output)}</small></span>
    </div>
  `;
}

function sideInfo(title, body, iconName, key = "") {
  return `
    <div class="side-info" ${key ? `data-side="${key}"` : ""}>
      <span class="metric-icon">${icon(iconName)}</span>
      <span><strong>${title}</strong><small>${body}</small></span>
    </div>
  `;
}

function agentPage() {
  return `
    <section class="page">
      ${pageTitle("Agent", t("agent.desc"), "network")}
      <div class="doc-grid">
        ${agentDoc({
          title: "Windows Agent",
          repo: "https://github.com/noirlang/worm-win",
          binary: "worm-win.exe",
          url: "https://worm.noirlang.tr/worm-win.exe",
          note: t("agent.windowsNote"),
          iconName: "windows",
          stepsTr: [
            t("agent.downloadWin"),
            t("agent.runWin"),
            t("agent.match")
          ],
          stepsEn: [
            t("agent.downloadWin"),
            t("agent.runWin"),
            t("agent.match")
          ]
        })}
        ${agentDoc({
          title: "Linux Agent",
          repo: "https://github.com/noirlang/worm-linux",
          binary: "worm-linux",
          url: "https://worm.noirlang.tr/worm-linux",
          note: t("agent.linuxNote"),
          iconName: "linux",
          stepsTr: [
            t("agent.downloadLinux"),
            t("agent.chmod"),
            t("agent.runLinux"),
            t("agent.connect")
          ],
          stepsEn: [
            t("agent.downloadLinux"),
            t("agent.chmod"),
            t("agent.runLinux"),
            t("agent.connect")
          ]
        })}
      </div>
    </section>
  `;
}

function agentDoc({ title, repo, binary, url, note, iconName, stepsTr, stepsEn }) {
  const commands = iconName === "linux"
    ? `wget -O ${binary} ${url}\nchmod +x ${binary}\n./${binary}`
    : `wget -O ${binary} ${url}\n${state.language === "en" ? `Run ${binary} as Administrator.` : `${binary} dosyasını yönetici olarak çalıştırın.`}`;
  const steps = state.language === "en" ? stepsEn : stepsTr;
  return `
    <article class="doc-card">
      <span class="card-icon">${icon(iconName)}</span>
      <h3>${title}</h3>
      <p>${note}</p>
      <div class="link-row">
        <a href="${repo}">${repo}</a>
        <a href="${url}">${url}</a>
      </div>
      <ol class="step-list">
        ${steps.map((step, index) => `<li><b>${index + 1}</b><span>${step}</span></li>`).join("")}
      </ol>
      <div class="code-box">${commands}</div>
    </article>
  `;
}

function analysisPage() {
  return `
    <section class="page">
      ${pageTitle(t("analysis.title"), t("analysis.desc"), "search")}
      <div class="workflow-panel">
        <p class="section-label">${t("analysis.title")}</p>
        <p class="field-hint">${t("analysis.hint")}</p>
        ${pickerField(t("analysis.imageFile"), "image-path", ".img, .dd, .raw, .iso ...", "file")}
        <div class="button-row">
          <button class="primary-button" data-action="mount-readonly">${icon("disk")} ${t("analysis.mount")}</button>
          <button class="danger-button" data-action="unmount-image">${icon("stop")} ${t("analysis.unmount")}</button>
        </div>
        <div class="section-divider"></div>
        <div class="side-info">
          <span class="metric-icon">${icon("info")}</span>
          <span><strong>${t("analysis.status")}</strong><small data-analysis-status>${state.imageMount?.mountDir || t("analysis.noImage")}</small></span>
        </div>
        <div class="log-box" data-analysis-log>${t("analysis.outputWaiting")}</div>
      </div>
    </section>
  `;
}

function otherPage() {
  return `
    <section class="page">
      ${pageTitle(t("other.title"), t("other.desc"), "tiles")}
      <div class="other-grid">
        ${simpleCard(t("other.hash.title"), t("other.hash.desc"), "shield", "hash")}
        ${simpleCard(t("other.evidence.title"), t("other.evidence.desc"), "scale", "evidence")}
        ${simpleCard(t("other.reports.title"), t("other.reports.desc"), "report", "reports")}
        ${simpleCard(t("other.logs.title"), t("other.logs.desc"), "clock", "logs")}
      </div>
      <div id="other-detail" class="workflow-panel" style="margin-top:16px">${detailPanel(state.activeTab)}</div>
    </section>
  `;
}

function simpleCard(title, desc, iconName, tab) {
  return `
    <button class="forensic-card" data-tab="${tab}">
      <span class="card-icon">${icon(iconName)}</span>
      <h3>${title}</h3>
      <p>${desc}</p>
      <span class="meta">${t("open")}</span>
    </button>
  `;
}

function hashPanel() {
  return `
    <p class="section-label">${t("hash.calculator")}</p>
    ${pickerField(t("hash.file"), "hash-file", t("hash.selectFile"), "file")}
    <div class="button-row">
      <button class="primary-button" data-action="hash">${icon("shield")} ${t("hash.calculate")}</button>
    </div>
    <div class="hash-grid">
      ${hashResult("MD5", "md5")}
      ${hashResult("SHA1", "sha1")}
      ${hashResult("SHA256", "sha256")}
      ${hashResult("SHA512", "sha512")}
    </div>
    <div class="section-divider"></div>
    <p class="section-label">${t("hash.compare")}</p>
    ${field(t("hash.value"), `<input class="input" data-hash-expected placeholder="${t("hash.placeholder")}" />`)}
    <div class="button-row">
      <button class="secondary-button" data-action="compare">${icon("search")} ${t("hash.compare")}</button>
    </div>
    <div class="side-info" data-hash-compare-result>
      <span class="metric-icon">${icon("info")}</span>
      <span><strong>${t("hash.result")}</strong><small>${t("hash.waiting")}</small></span>
    </div>
  `;
}

function hashResult(label, key) {
  return `
    <div class="hash-result" data-hash-result="${key}">
      <small>${label}</small>
      <strong>-</strong>
    </div>
  `;
}

function settingsPage() {
  return `
    <section class="page">
      <div class="settings-header">
        <h1>${t("settings.title")}</h1>
        <p>${t("settings.desc")}</p>
      </div>
      <div class="settings-layout">
        <article class="settings-card settings-primary">
          <span class="settings-kicker">${t("settings.appearance")}</span>
          <h3>${t("settings.appSettings")}</h3>
          <p>${t("settings.persisted")}</p>
          <div class="settings-row">
            <span>
              <strong>${t("settings.darkTheme")}</strong>
              <small>${t("settings.darkHint")}</small>
            </span>
            <button class="switch ${state.theme === "dark" ? "on" : ""}" data-action="theme-toggle" aria-label="${t("settings.darkTheme")}"></button>
          </div>
          <div class="settings-row">
            <span>
              <strong>${t("settings.language")}</strong>
              <small>${t("settings.languageHint")}</small>
            </span>
            <select class="select compact-select" data-action="language-select" aria-label="${t("settings.language")}">
              <option value="tr" ${state.language === "tr" ? "selected" : ""}>Türkçe</option>
              <option value="en" ${state.language === "en" ? "selected" : ""}>English</option>
            </select>
          </div>
          <div class="settings-row">
            <span>
              <strong>${t("settings.detectedSystem")}</strong>
              <small>${t("settings.detectedHint")}</small>
            </span>
            <span class="status-badge">${icon(state.platform === "windows" ? "windows" : state.platform === "linux" ? "linux" : "monitor")} ${platformLabel(state.platform)}</span>
          </div>
          <div class="button-row">
            <button class="primary-button" data-action="save-settings">${t("settings.save")}</button>
          </div>
          <div class="status-badge" data-settings-status>${icon("info")} ${t("ready")}</div>
        </article>

        <article class="settings-card">
          <span class="settings-kicker">${t("settings.version")}</span>
          <h3>${t("settings.update")}</h3>
          <p>${t("settings.updateDesc")}</p>
          <div class="settings-meta">
            <span>${t("settings.installed")}: ${APP_VERSION}</span>
            <span>Asset: ${state.platform === "windows" ? "worm-windows-x64.msi" : "worm-linux-x64.AppImage"}</span>
          </div>
          <div class="progress-bar" data-update-progress style="--value:0%"><span></span><b>0%</b></div>
          <div class="button-row">
            <button class="primary-button" data-action="check-update">${icon("refresh")} ${t("settings.checkUpdate")}</button>
            <button class="secondary-button" data-action="download-update">${icon("download")} ${t("settings.downloadInstall")}</button>
          </div>
          <div class="status-badge" data-update-status>${icon("info")} ${t("ready")}</div>
          <div class="log-box compact-log" data-update-log>${t("settings.releaseNotes")}</div>
        </article>
      </div>
    </section>
  `;
}

function aboutPage() {
  return `
    <section class="page">
      <div class="about-hero">
        <span class="about-logo"><img src="${assetPath}/logo/logo.png" alt="Worm logo" /></span>
        <div>
          <p class="eyebrow">Worm Forensic Tool</p>
          <h1>Worm Forensic Tool</h1>
          <span class="status-badge">${t("about.version", { version: APP_VERSION })}</span>
          <p>${t("about.desc")}</p>
        </div>
      </div>

      <h2 class="section-heading">${t("about.capabilities")}</h2>
      <div class="capability-grid">
        ${capabilityCard("COLLECT", t("about.collect.title"), t("about.collect.desc"), "disk", "var(--green)")}
        ${capabilityCard("PROVE", t("about.prove.title"), t("about.prove.desc"), "shield", "var(--blue)")}
        ${capabilityCard("PACKAGE", t("about.package.title"), t("about.package.desc"), "report", "var(--purple)")}
      </div>

      <div class="doc-card usage-card">
        <h3>${t("about.usage")}</h3>
        <p>${t("about.usageDesc")}</p>
      </div>

      <h2 class="section-heading">${t("about.maintainers")}</h2>
      <div class="contributor-grid">
        ${contributorCard("ME", "Melih Emik", t("about.role.lead"), "melih-emik.jpg", [
          ["GitHub", "https://github.com/favilances"],
          ["LinkedIn", "https://www.linkedin.com/in/melihemik/"],
          ["Website", "https://melihemik.com.tr"]
        ])}
        ${contributorCard("YT", "Yusuf Tuncel", t("about.role.windows"), "yusuf-tuncel.jpg", [
          ["GitHub", "https://github.com/yetece1"],
          ["LinkedIn", "https://www.linkedin.com/in/yusuf-tuncel/"]
        ])}
        ${contributorCard("MG", "Muhammet Ali Güner", t("about.role.linux"), "muhammet-ali-guner.jpg", [
          ["GitHub", "https://github.com/kafkaskrtl"],
          ["LinkedIn", "https://www.linkedin.com/in/muhammetali-g%C3%BCner/"]
        ])}
      </div>

      <div class="company-logo-card">
        <img src="${assetPath}/logo/sirket.png" alt="Şirket logosu" />
      </div>
    </section>
  `;
}

function capabilityCard(kicker, title, desc, iconName, accent) {
  return `
    <article class="doc-card capability-card" style="--accent:${accent}">
      <span class="card-icon">${icon(iconName)}</span>
      <p class="eyebrow">${kicker}</p>
      <h3>${title}</h3>
      <p>${desc}</p>
    </article>
  `;
}

function contributorCard(initials, name, role, photo, links) {
  return `
    <article class="contributor-card">
      <img class="avatar" src="${assetPath}/contributors/${photo}" alt="${name}" />
      <h3>${name}</h3>
      <p>${role}</p>
      <div class="social-row" aria-label="${name} bağlantıları">
        ${links.map(([label, url]) => socialLink(label, url)).join("")}
      </div>
    </article>
  `;
}

function socialLink(label, url) {
  const key = label === "LinkedIn" ? "linkedin" : label === "Website" ? "website" : "github";
  return `<a class="social-button" href="${url}" target="_blank" rel="noopener noreferrer" aria-label="${label}">${icon(key)}</a>`;
}

const routes = {
  home: homePage,
  windows: () => toolHub("windows"),
  linux: () => toolHub("linux"),
  android: () => androidPage({ t, icon, pageTitle, state, escapeHtml, backendReady }),
  agent: agentPage,
  analysis: analysisPage,
  other: otherPage,
  settings: settingsPage,
  about: aboutPage
};

async function apiRequest(path, options = {}) {
  const headers = new Headers(options.headers || {});
  if (options.body && !headers.has("content-type")) {
    headers.set("content-type", "application/json");
  }
  const response = await fetch(path, { ...options, headers });
  const text = await response.text();
  const data = text ? JSON.parse(text) : {};
  if (!response.ok) {
    throw new Error(data.error || response.statusText);
  }
  return data;
}

function isExternalUrl(url) {
  try {
    const parsed = new URL(url, window.location.href);
    return ["http:", "https:", "mailto:"].includes(parsed.protocol);
  } catch {
    return false;
  }
}

async function openExternalUrl(url) {
  try {
    await apiRequest("/api/open-url", {
      method: "POST",
      body: JSON.stringify({ url })
    });
    return;
  } catch (error) {
    console.warn("External link could not be opened by backend", error);
  }
  window.open(url, "_blank", "noopener,noreferrer");
}

async function loadEvidenceCases({ silent = true } = {}) {
  if (!backendReady()) return;
  try {
    const result = await apiRequest("/api/evidence-cases");
    state.caseBaseDir = result.base_dir || "";
    state.cases = Array.isArray(result.cases) ? result.cases : [];
    if (result.current_case) state.activeCase = result.current_case;
    updateCaseControls();
    if (!silent) showToast(t("case.loaded", { count: String(state.cases.length) }));
  } catch (error) {
    if (!silent) showToast(t("case.listFailed", { message: error.message }), "error");
  }
}

function updateCaseControls() {
  document.querySelectorAll("[data-case-base]").forEach((node) => {
    node.textContent = state.caseBaseDir || "~/Worm/Vakalar";
  });

  document.querySelectorAll("[data-case-select]").forEach((select) => {
    const allowNew = select.dataset.allowNewCase === "1";
    const selected = select.value || state.activeCase?.case_name || "";
    select.innerHTML = caseSelectOptions(selected, { allowNew });
    toggleCaseCreateInput(select);
  });
}

function caseSelectOptions(selected = "", { allowNew = false } = {}) {
  const effectiveSelected = selected || (allowNew && !state.cases.length ? "__new__" : "");
  if (!state.cases.length && !allowNew) {
    return `<option value="">${t("case.noCases")}</option>`;
  }
  const options = state.cases
    .map((item) => {
      const name = escapeHtml(item.case_name || "");
      const isSelected = item.case_name === effectiveSelected ? " selected" : "";
      return `<option value="${name}"${isSelected}>${name}</option>`;
    })
    .join("");
  const newSelected = effectiveSelected === "__new__" || (allowNew && !state.cases.length) ? " selected" : "";
  const newOption = allowNew ? `<option value="__new__"${newSelected}>${t("workflow.newCase")}</option>` : "";
  return `${options}${newOption}`;
}

function toggleCaseCreateInput(select) {
  const input = document.querySelector("#workflow-case-name");
  if (input) input.closest(".field").hidden = select.value !== "__new__";
  document.querySelectorAll("[data-case-output]").forEach((output) => {
    output.textContent = caseOutputLabel(select.value, output.dataset.caseOutputSubdir || "ciktilar");
  });
}

function imageCaseOutputLabel(caseName) {
  return caseOutputLabel(caseName, "ciktilar");
}

function caseOutputLabel(caseName, subdir = "ciktilar") {
  const selected = state.cases.find((item) => item.case_name === caseName)
    || (state.activeCase?.case_name === caseName ? state.activeCase : null);
  const key = subdir === "ram" ? "ram_dir" : "output_dir";
  if (caseName !== "__new__" && selected?.[key]) return selected[key];
  const folder = subdir === "ram" ? "ram" : "ciktilar";
  if (state.caseBaseDir) return `${state.caseBaseDir}/${document.querySelector("#workflow-case-name")?.value.trim() || defaultCaseName()}/${folder}`;
  return `~/Worm/Vakalar/<vaka>/${folder}`;
}

function reportCaseName() {
  const selected = document.querySelector("#report-case")?.value.trim() || "";
  if (selected && state.cases.length) return selected;
  return document.querySelector("#report-case-name")?.value.trim() || defaultCaseName();
}

async function ensureImageCase() {
  const select = document.querySelector("#workflow-case");
  const selected = select?.value || "";
  if (selected && selected !== "__new__") {
    const existing = state.cases.find((item) => item.case_name === selected);
    if (existing) return existing;
  }

  const caseName = document.querySelector("#workflow-case-name")?.value.trim() || defaultCaseName();
  const created = await apiRequest("/api/evidence-create", {
    method: "POST",
    body: JSON.stringify({ case_name: caseName })
  });
  state.activeCase = created;
  state.cachedDefaultCaseName = "";
  await loadEvidenceCases();
  return created;
}

function defaultCaseName() {
  const now = new Date();
  const pad = (value) => String(value).padStart(2, "0");
  return `Case_${now.getFullYear()}${pad(now.getMonth() + 1)}${pad(now.getDate())}_${pad(now.getHours())}${pad(now.getMinutes())}${pad(now.getSeconds())}`;
}

function stableDefaultCaseName() {
  if (!state.cachedDefaultCaseName) {
    state.cachedDefaultCaseName = defaultCaseName();
  }
  return state.cachedDefaultCaseName;
}

function timestampForFileName(date = new Date()) {
  const pad = (value) => String(value).padStart(2, "0");
  return `${date.getFullYear()}${pad(date.getMonth() + 1)}${pad(date.getDate())}_${pad(date.getHours())}${pad(date.getMinutes())}${pad(date.getSeconds())}`;
}

function sanitizeFileStem(value) {
  return String(value || "")
    .trim()
    .replace(/[<>:"/\\|?*\x00-\x1F\s]+/g, "_")
    .replace(/^_+|_+$/g, "");
}

function canonicalRamFileName(remoteIp = "", date = new Date()) {
  const ip = sanitizeFileStem(remoteIp);
  return `${ip ? `${ip}_` : ""}ram_${timestampForFileName(date)}.raw`;
}

function selectedTargetName() {
  const select = document.querySelector("[data-field='target']");
  const option = select?.selectedOptions?.[0];
  return option?.dataset.diskName || option?.textContent?.split("·")[0]?.trim() || "";
}

function backendReady() {
  return backendAvailable;
}

function connectionPayload() {
  const tokenText = document.querySelector("[data-field='token']")?.value.trim() || "";
  if (tokenText && !state.approvedSecurityKey) {
    throw new Error(t("connection.keyApproveFirst"));
  }
  if (tokenText && tokenText !== state.approvedSecurityKey) {
    throw new Error(t("connection.keyChanged"));
  }
  return {
    ip: document.querySelector("[data-field='ip']")?.value.trim() || "",
    port: Number(document.querySelector("[data-field='port']")?.value.trim() || 0),
    token: tokenText ? state.approvedSecurityKey : null
  };
}

function vpnPayload() {
  const endpoint = document.querySelector("[data-field='vpn-endpoint']")?.value.trim() || "";
  const configFile = document.querySelector("#vpn-config-file")?.value.trim() || "";
  if (!endpoint) throw new Error(t("vpn.endpointRequired"));
  if (!configFile) throw new Error(t("vpn.configRequired"));
  return {
    config_file: configFile,
    private_key: document.querySelector("[data-field='vpn-private-key']")?.value.trim() || "",
    public_key: document.querySelector("[data-field='vpn-public-key']")?.value.trim() || "",
    endpoint,
    allowed_ips: document.querySelector("[data-field='vpn-allowed']")?.value.trim() || "0.0.0.0/0",
    address: document.querySelector("[data-field='vpn-address']")?.value.trim() || "10.0.0.2/24",
    dns: document.querySelector("[data-field='vpn-dns']")?.value.trim() || "1.1.1.1",
    keepalive: Number(document.querySelector("[data-field='vpn-keepalive']")?.value.trim() || 25)
  };
}

function currentWorkflowId() {
  return state.route.startsWith("workflow:") ? state.route.split(":")[1] : "";
}

function currentWorkflow() {
  return workflows[currentWorkflowId()];
}

function rememberConnection(workflowId, payload, details) {
  state.remoteConnections[workflowId] = {
    ip: payload.ip,
    port: payload.port,
    token: payload.token || "",
    serverName: details.server_name || "",
    serverVersion: details.server_version || "",
    features: details.features || []
  };
}

function forgetConnection(workflowId = currentWorkflowId()) {
  if (workflowId) delete state.remoteConnections[workflowId];
}

function requireActiveConnection(workflow, payload) {
  if (!workflow?.mode.startsWith("remote")) return true;
  const connection = state.remoteConnections[currentWorkflowId()];
  const matches = connection
    && connection.ip === payload.ip
    && Number(connection.port) === Number(payload.port)
    && (connection.token || "") === (payload.token || "");
  if (!matches) {
    showToast(t("connection.connectFirst"), "error");
    updateSide("connection", t("connection.none"));
    writeWorkflowLog(t("connection.required"));
    return false;
  }
  return true;
}

function formatBytes(bytes) {
  const value = Number(bytes || 0);
  if (!Number.isFinite(value) || value <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  let size = value;
  let unit = 0;
  while (size >= 1024 && unit < units.length - 1) {
    size /= 1024;
    unit += 1;
  }
  return `${size.toFixed(size >= 10 || unit === 0 ? 0 : 1)} ${units[unit]}`;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#39;");
}

document.addEventListener("click", (event) => {
  const externalLink = event.target.closest("a[href]");
  if (externalLink && isExternalUrl(externalLink.href)) {
    event.preventDefault();
    openExternalUrl(externalLink.href);
    return;
  }

  const routeButton = event.target.closest("[data-route]");
  if (routeButton) {
    setRoute(routeButton.dataset.route);
    return;
  }

  const actionButton = event.target.closest("[data-action]");
  if (actionButton) {
    handleAction(actionButton);
    return;
  }

  const tabButton = event.target.closest("[data-tab]");
  if (tabButton) {
    state.activeTab = tabButton.dataset.tab;
    const detail = document.querySelector("#other-detail");
    if (detail) detail.innerHTML = detailPanel(state.activeTab);
    hydrateIcons(detail);
    if (["evidence", "reports"].includes(state.activeTab)) loadEvidenceCases();
  }
});

document.addEventListener("change", (event) => {
  const select = event.target.closest("[data-action='language-select']");
  if (select) {
    setLanguage(select.value);
    showToast(t("settingsSaved"));
    render();
  }

  const target = event.target.closest("[data-field='target']");
  if (target) {
    updateSide("target", target.value || t("targetNotSelected"));
  }

  const caseSelect = event.target.closest("[data-case-select]");
  if (caseSelect) {
    toggleCaseCreateInput(caseSelect);
  }

  const androidDeviceSelect = event.target.closest("[data-android-device-select]");
  if (androidDeviceSelect) {
    syncAndroidDeviceSelection(androidDeviceSelect, { state, t, showToast });
  }

  if (event.target.closest("#workflow-case-name")) {
    const select = document.querySelector("#workflow-case");
    if (select?.value === "__new__") {
      document.querySelectorAll("[data-case-output]").forEach((output) => {
        output.textContent = caseOutputLabel("__new__", output.dataset.caseOutputSubdir || "ciktilar");
      });
    }
  }
});

document.addEventListener("input", (event) => {
  if (!event.target.closest("#workflow-case-name")) return;
  const select = document.querySelector("#workflow-case");
  if (select?.value === "__new__") {
    document.querySelectorAll("[data-case-output]").forEach((output) => {
      output.textContent = caseOutputLabel("__new__", output.dataset.caseOutputSubdir || "ciktilar");
    });
  }
});

async function handleAction(button) {
  const action = button.dataset.action;
  if (action?.startsWith("android-")) {
    const handled = await handleAndroidAction(button, {
      apiRequest,
      backendReady,
      state,
      t,
      showToast,
      render,
      resolveCase() {
        const select = document.querySelector("#workflow-case");
        const selected = select?.value || "";
        if (selected && selected !== "__new__") return selected;
        return document.querySelector("#workflow-case-name")?.value.trim() || null;
      }
    });
    if (handled) return;
  }

  if (action === "theme-toggle") {
    setTheme(state.theme === "dark" ? "light" : "dark");
    render();
    return;
  }

  if (action === "pick-file") {
    await pickFile(button.dataset.target);
    return;
  }

  if (action === "pick-folder") {
    await pickFolder(button.dataset.target);
    return;
  }

  if (action === "toggle-vpn") {
    button.classList.toggle("on");
    const panel = document.querySelector(".vpn-panel");
    if (panel) panel.hidden = !button.classList.contains("on");
    writeWorkflowLog(button.classList.contains("on") ? t("vpn.enabled") : t("vpn.disabled"));
    updateSide("connection", button.classList.contains("on") ? t("vpn.waiting") : t("vpn.off"));
    return;
  }

  if (action === "vpn-config") {
    const panel = document.querySelector(".vpn-panel");
    if (panel) panel.hidden = false;
    document.querySelector("[data-action='toggle-vpn']")?.classList.add("on");
    writeWorkflowLog(t("vpn.opened"));
    return;
  }

  if (action === "save-vpn") {
    try {
      const payload = vpnPayload();
      const result = await apiRequest("/api/wireguard-config", {
        method: "POST",
        body: JSON.stringify(payload)
      });
      writeWorkflowLog(t("vpn.configured", { endpoint: payload.endpoint }));
      updateSide("connection", t("vpn.ready"));
      showToast(t("vpn.saved"));
      if (result.path) document.querySelector("#vpn-config-file").value = result.path;
    } catch (error) {
      showToast(t("vpn.failed", { message: error.message }), "error");
      writeWorkflowLog(t("vpn.failed", { message: error.message }));
    }
    return;
  }

  if (action === "start-vpn") {
    const configFile = document.querySelector("#vpn-config-file")?.value.trim();
    if (!configFile) {
      showToast(t("vpn.configRequired"), "error");
      return;
    }
    try {
      await apiRequest("/api/wireguard-start", {
        method: "POST",
        body: JSON.stringify({ config_file: configFile })
      });
      writeWorkflowLog(t("vpn.started"));
      updateSide("connection", t("vpn.ready"));
      showToast(t("vpn.started"));
    } catch (error) {
      showToast(t("vpn.failed", { message: error.message }), "error");
      writeWorkflowLog(t("vpn.failed", { message: error.message }));
    }
    return;
  }

  if (action === "stop-vpn") {
    try {
      await apiRequest("/api/wireguard-stop", { method: "POST" });
      writeWorkflowLog(t("vpn.stopped"));
      updateSide("connection", t("vpn.off"));
      showToast(t("vpn.stopped"));
    } catch (error) {
      showToast(t("vpn.failed", { message: error.message }), "error");
      writeWorkflowLog(t("vpn.failed", { message: error.message }));
    }
    return;
  }

  if (action === "approve-key") {
    const token = document.querySelector("[data-field='token']");
    const value = token?.value.trim() || "";
    if (!value) {
      showToast(t("key.required"), "error");
      return;
    }
    state.approvedSecurityKey = value;
    if (token) token.readOnly = true;
    writeWorkflowLog(t("key.approved"));
    showToast(t("key.active"));
    return;
  }

  if (action === "reset-key") {
    const token = document.querySelector("[data-field='token']");
    state.approvedSecurityKey = "";
    if (token) {
      token.value = "";
      token.readOnly = false;
    }
    forgetConnection();
    writeWorkflowLog(t("key.reset"));
    return;
  }

  if (action === "connect") {
    const workflowId = currentWorkflowId();
    const workflow = currentWorkflow();
    let payload;
    try {
      payload = connectionPayload();
    } catch (error) {
      showToast(error.message, "error");
      return;
    }
    if (!payload.ip || !payload.port) {
      showToast(t("connection.ipPortRequired"), "error");
      return;
    }
    if (payload.port <= 0 || payload.port > 65535) {
      showToast(t("connection.invalidPort"), "error");
      return;
    }
    if (!workflow?.mode.startsWith("remote")) {
      showToast(t("connection.remoteOnly"), "error");
      return;
    }

    forgetConnection(workflowId);
    button.disabled = true;
    updateSide("connection", t("connection.connecting"));
    writeWorkflowLog(t("connection.starting", { host: `${payload.ip}:${payload.port}` }));
    try {
      const result = await apiRequest("/api/connect", {
        method: "POST",
        body: JSON.stringify(payload)
      });
      rememberConnection(workflowId, payload, result);
      updateSide("connection", t("connection.connected", { ip: payload.ip }));
      writeWorkflowLog(t("connection.connectedLog", { ip: payload.ip }));
      showToast(t("connection.success"));
    } catch (error) {
      forgetConnection(workflowId);
      updateSide("connection", t("connection.failed"));
      writeWorkflowLog(t("connection.failedLog", { ip: payload.ip, message: error.message }));
      showToast(t("connection.cannotConnect", { message: error.message }), "error");
    } finally {
      button.disabled = false;
    }
    return;
  }

  if (action === "scan") {
    await scanTargets();
    return;
  }

  if (action === "download") {
    await installWinpmem(button);
    return;
  }

  if (action === "install-avml") {
    await installAvml(button);
    return;
  }

  if (action === "start") {
    await startAcquisition(button);
    return;
  }

  if (action === "pause") {
    try {
      await sendAcquisitionControl("pause");
    } catch (error) {
      showToast(t("workflow.pauseFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "resume") {
    try {
      await sendAcquisitionControl("resume");
    } catch (error) {
      showToast(t("workflow.resumeFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "stop") {
    try {
      await sendAcquisitionControl("stop");
    } catch (error) {
      showToast(t("workflow.stopFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "mount-readonly") {
    const imagePath = document.querySelector("#image-path")?.value.trim();
    if (!imagePath || imagePath.startsWith(".")) {
      showToast(t("analysis.imageRequired"), "error");
      return;
    }
    setAnalysisStatus(t("analysis.mounting"), t("analysis.mounting"));
    try {
      const result = await apiRequest("/api/image-mount-readonly", {
        method: "POST",
        body: JSON.stringify({ path: imagePath })
      });
      state.imageMount = {
        imagePath: result.image_path,
        mountDir: result.mount_dir
      };
      setAnalysisStatus(
        t("analysis.mounted", { path: result.mount_dir }),
        renderTree(result.tree)
      );
      showToast(t("analysis.mountPrepared"));
    } catch (error) {
      setAnalysisStatus(t("analysis.noImage"), t("analysis.mountFailed", { message: error.message }));
      showToast(t("analysis.mountFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "unmount-image") {
    try {
      await apiRequest("/api/image-unmount", { method: "POST" });
      state.imageMount = null;
      setAnalysisStatus(t("analysis.unmounted"), t("analysis.noActiveMount"));
      showToast(t("analysis.unmounted"));
    } catch (error) {
      showToast(t("analysis.unmountFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "hash") {
    await calculateHashes();
    return;
  }

  if (action === "compare") {
    compareHash();
    return;
  }

  if (action === "save-settings") {
    setStatus("[data-settings-status]", `${icon("info")} ${t("settingsSaved")}`);
    showToast(t("settingsSaved"));
    return;
  }

  if (action === "check-update") {
    try {
      setStatus("[data-update-status]", `${icon("refresh")} ${t("settings.updateChecked")}`);
      const result = await apiRequest("/api/update-check");
      state.latestUpdate = result;
      const asset = result.platform_asset || {};
      const assetLine = asset.name ? `<br />Asset: ${escapeHtml(asset.name)} (${formatBytes(asset.size)})` : `<br />${t("settings.noAsset")}`;
      setStatus("[data-update-status]", `${icon("info")} ${t("settings.latestVersion", { version: result.tag_name || result.name || "-" })}`);
      setStatus("[data-update-log]", `${escapeHtml(result.body || t("settings.releaseNotes")).replaceAll("\n", "<br />")}${assetLine}`);
      showToast(t("settings.updateDone"));
    } catch (error) {
      setStatus("[data-update-status]", `${icon("info")} ${t("settings.updateFailed", { message: escapeHtml(error.message) })}`);
      showToast(t("settings.updateFailed", { message: error.message }), "error");
    }
    return;
  }

  if (action === "download-update") {
    await downloadUpdatePackage();
    return;
  }

  if (action === "list-files") {
    await listEvidenceFiles();
    return;
  }

  if (action === "refresh-cases") {
    await loadEvidenceCases({ silent: false });
    return;
  }

  if (action === "create-case") {
    await createEvidenceCase();
    return;
  }

  if (action === "add-note") {
    await addEvidenceNote();
    return;
  }

  if (action === "create-report") {
    await createEvidenceReport();
    return;
  }

  const label = button.textContent.trim().replace(/\s+/g, " ");
  writeWorkflowLog(`${label}: ${t("ready")}`);
  showToast(`${label}: ${t("ready")}`);
}

function showToast(message, type = "success") {
  let toast = document.querySelector(".toast");
  if (!toast) {
    toast = document.createElement("div");
    toast.className = "toast";
    document.body.appendChild(toast);
  }
  toast.textContent = message;
  toast.dataset.type = type;
  toast.classList.add("visible");
  window.clearTimeout(showToast.timer);
  showToast.timer = window.setTimeout(() => toast.classList.remove("visible"), 3200);
}

async function pickFile(targetSelector) {
  const target = targetSelector ? document.querySelector(targetSelector) : null;
  if (backendReady()) {
    try {
      const result = await apiRequest("/api/pick-file", { method: "POST" });
      if (target) {
        target.value = result.path;
        delete state.files[targetSelector];
      }
      showToast(t("workflow.selectFile", { path: result.path }));
      return result.path;
    } catch (error) {
      if (String(error?.message || "").includes("cancelled")) return null;
      showToast(t("workflow.filePickerFailed", { message: error.message }), "error");
      return null;
    }
  }

  try {
    if (window.showOpenFilePicker) {
      const [handle] = await window.showOpenFilePicker({ multiple: false });
      const file = await handle.getFile();
      if (target) {
        target.value = file.name;
        state.files[targetSelector] = file;
      }
      showToast(t("workflow.selectFile", { path: file.name }));
      return file;
    }
  } catch (error) {
    if (error?.name === "AbortError") return null;
    showToast(t("workflow.filePickerFailedShort"), "error");
    return null;
  }

  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.style.position = "fixed";
    input.style.opacity = "0";
    input.addEventListener("change", () => {
      const file = input.files?.[0] || null;
      if (file && target) {
        target.value = file.name;
        state.files[targetSelector] = file;
        showToast(t("workflow.selectFile", { path: file.name }));
      }
      input.remove();
      resolve(file);
    });
    document.body.appendChild(input);
    input.click();
  });
}

async function pickFolder(targetSelector) {
  const target = targetSelector ? document.querySelector(targetSelector) : null;
  if (backendReady()) {
    try {
      const result = await apiRequest("/api/pick-folder", { method: "POST" });
      if (target) target.value = result.path;
      showToast(t("workflow.selectFolder", { path: result.path }));
      return result.path;
    } catch (error) {
      if (String(error?.message || "").includes("cancelled")) return null;
      showToast(t("workflow.folderPickerFailed", { message: error.message }), "error");
      return null;
    }
  }

  try {
    if (window.showDirectoryPicker) {
      const handle = await window.showDirectoryPicker();
      if (target) target.value = handle.name;
      showToast(t("workflow.selectFolder", { path: handle.name }));
      return handle;
    }
  } catch (error) {
    if (error?.name === "AbortError") return null;
    showToast(t("workflow.folderPickerFailedShort"), "error");
    return null;
  }

  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.webkitdirectory = true;
    input.style.position = "fixed";
    input.style.opacity = "0";
    input.addEventListener("change", () => {
      const first = input.files?.[0];
      const folder = first?.webkitRelativePath?.split("/")?.[0] || first?.name || "";
      if (folder && target) target.value = folder;
      if (folder) showToast(t("workflow.selectFolder", { path: folder }));
      input.remove();
      resolve(folder);
    });
    document.body.appendChild(input);
    input.click();
  });
}

function writeWorkflowLog(message) {
  state.lastLog.unshift(message);
  state.lastLog = state.lastLog.slice(0, 8);
  const log = document.querySelector("#workflow-log");
  if (log) log.innerHTML = state.lastLog.map((line) => `• ${line}`).join("<br />");
  updateSide("last-action", message);
}

function updateSide(key, value) {
  const item = document.querySelector(`[data-side="${key}"] small`);
  if (item) item.innerHTML = value;
}

async function scanTargets() {
  const select = document.querySelector("[data-field='target']");
  if (!select) return;
  const routeId = state.route.split(":")[1];
  const workflow = workflows[routeId];
  const isRam = workflow?.mode.includes("ram");

  if (isRam) {
    if (backendReady()) {
      try {
        const toolKey = workflow.platform === "Windows" ? "winpmem" : "avml";
        let status;
        if (workflow.mode.startsWith("remote")) {
          const payload = connectionPayload();
          if (!payload.ip || !payload.port) {
            showToast(t("connection.ipPortRequired"), "error");
            return;
          }
          if (!requireActiveConnection(workflow, payload)) return;
          const result = await apiRequest("/api/remote-tool-check", {
            method: "POST",
            body: JSON.stringify({ ...payload, tool: toolKey })
          });
          status = result.status;
          updateSide("connection", t("connection.checked", { host: `${payload.ip}:${payload.port}` }));
        } else {
          const result = await apiRequest("/api/ram-status");
          status = result[toolKey];
        }
        const label = status?.tool_path || status?.message || localText(workflow.diskLabel);
        const targets = [localText(workflow.diskLabel), label].filter(Boolean);
        select.innerHTML = targets.map((target) => `<option value="${target}">${target}</option>`).join("");
        updateSide("target", targets[0]);
        writeWorkflowLog(t("scan.toolDoneLog", { target: localText(workflow.diskLabel), message: status?.message || t("ready") }));
        showToast(t("scan.done"));
      } catch (error) {
        if (workflow?.mode.startsWith("remote")) {
          forgetConnection();
          updateSide("connection", t("connection.toolFailed"));
        }
        showToast(t("scan.failed", { message: error.message }), "error");
        writeWorkflowLog(t("scan.ramFailedLog", { message: error.message }));
      }
      return;
    }

    const fallbackTargets = [localText(workflow.diskLabel), workflow.platform === "Windows" ? "WinPMEM portable" : "AVML local"];
    select.innerHTML = fallbackTargets.map((target) => `<option value="${target}">${target}</option>`).join("");
    updateSide("target", fallbackTargets[0]);
    writeWorkflowLog(t("scan.toolListUpdated"));
    showToast(t("scan.done"));
    return;
  }

  if (backendReady()) {
    try {
      let disks = [];
      if (workflow.mode.startsWith("remote")) {
        const payload = connectionPayload();
        if (!payload.ip || !payload.port) {
          showToast(t("connection.ipPortRequired"), "error");
          return;
        }
        if (!requireActiveConnection(workflow, payload)) return;
        const result = await apiRequest("/api/remote-disks", {
          method: "POST",
          body: JSON.stringify(payload)
        });
        disks = result.disks || [];
        updateSide("connection", t("connection.alive", { host: `${payload.ip}:${payload.port}` }));
      } else {
        const result = await apiRequest("/api/disk-list");
        disks = result.disks || [];
        if (result.elevated) {
          writeWorkflowLog(t("scan.elevated"));
        } else if (result.elevation_error) {
          writeWorkflowLog(t("scan.elevationFailed", { message: result.elevation_error }));
          showToast(t("scan.elevationFailed", { message: result.elevation_error }), "error");
        }
      }

      const options = disks
        .map((disk) => {
          const value = disk.id || disk.device || disk.name || disk.path || "";
          if (!value) return "";
          const size = disk.boyut || disk.total_size || 0;
          const name = disk.ad || disk.device || disk.name || value;
          const access = disk.accessible === false ? ` ${t("scan.accessDenied")}` : "";
          return `<option value="${escapeHtml(value)}" data-disk-name="${escapeHtml(name)}">${escapeHtml(name)} · ${formatBytes(size)}${access}</option>`;
        })
        .filter(Boolean);

      if (options.length === 0) {
        select.innerHTML = `<option value="" disabled selected>${t("scan.noDisk")}</option>`;
        updateSide("target", t("targetNotSelected"));
        writeWorkflowLog(t("scan.noDiskLog"));
        showToast(t("scan.noDisk"), "error");
        return;
      }

      select.innerHTML = options.join("");
      updateSide("target", select.value);
      writeWorkflowLog(t("scan.diskDoneLog"));
      showToast(t("scan.diskDone"));
      return;
    } catch (error) {
      if (workflow?.mode.startsWith("remote")) {
        forgetConnection();
        updateSide("connection", t("connection.disksFailed"));
      }
      showToast(t("scan.diskFailed", { message: error.message }), "error");
      writeWorkflowLog(t("scan.diskFailed", { message: error.message }));
      return;
    }
  }

  const tauriInvoke = window.__TAURI__?.core?.invoke || window.__TAURI__?.tauri?.invoke;
  if (tauriInvoke) {
    try {
      const disks = await tauriInvoke(workflow.mode.startsWith("remote") ? "remote_disk_list" : "local_disk_list", {
        platform: workflow.platform.toLowerCase()
      });
      const targets = Array.isArray(disks) ? disks.map((disk) => disk.id || disk.name || disk.path || disk).filter(Boolean) : [];
      if (targets.length > 0) {
        select.innerHTML = targets.map((target) => `<option value="${escapeHtml(target)}" data-disk-name="${escapeHtml(target)}">${escapeHtml(target)}</option>`).join("");
        updateSide("target", targets[0]);
        writeWorkflowLog(t("scan.diskDoneLog"));
        showToast(t("scan.diskDone"));
        return;
      }
    } catch (error) {
      showToast(t("scan.diskFailedShort"), "error");
      writeWorkflowLog(t("scan.diskFailed", { message: error?.message || error }));
      return;
    }
  }

  select.innerHTML = `<option value="" disabled selected>${t("scan.waiting")}</option>`;
  updateSide("target", t("targetNotSelected"));
  writeWorkflowLog(t("scan.appModeRequired"));
  showToast(t("scan.completed"));
}

async function installAvml(button) {
  const workflow = currentWorkflow();
  if (!workflow || workflow.platform !== "Linux" || workflow.mode !== "local-ram") {
    showToast(t("workflow.avmlUnsupported"), "error");
    return;
  }
  if (!backendReady()) {
    showToast(t("workflow.appModeRequired"), "error");
    return;
  }

  button.disabled = true;
  writeWorkflowLog(t("workflow.avmlInstalling"));
  updateSide("last-action", t("workflow.avmlInstalling"));
  try {
    const result = await apiRequest("/api/avml-install", { method: "POST" });
    const status = result.status || {};
    const path = status.tool_path || result.path || "/usr/bin/avml";
    const label = status.message || result.message || "AVML ready";
    const select = document.querySelector("[data-field='target']");
    if (select) {
      const localLabel = localText(workflow.diskLabel);
      select.innerHTML = [
        `<option value="${escapeHtml(localLabel)}">${escapeHtml(localLabel)}</option>`,
        `<option value="${escapeHtml(path)}">${escapeHtml(path)}</option>`
      ].join("");
      select.value = path;
    }
    updateSide("target", escapeHtml(path));
    writeWorkflowLog(t("scan.toolDoneLog", { target: "AVML", message: escapeHtml(label) }));
    writeWorkflowLog(t("workflow.avmlInstalled", { path: escapeHtml(path) }));
    showToast(t("workflow.avmlInstalled", { path }));
  } catch (error) {
    writeWorkflowLog(t("workflow.avmlInstallFailed", { message: escapeHtml(error.message) }));
    showToast(t("workflow.avmlInstallFailed", { message: error.message }), "error");
  } finally {
    button.disabled = false;
  }
}

async function installWinpmem(button) {
  const workflow = currentWorkflow();
  if (!workflow || workflow.platform !== "Windows" || workflow.mode !== "local-ram") {
    showToast(t("workflow.winpmemUnsupported"), "error");
    return;
  }
  if (!backendReady()) {
    showToast(t("workflow.appModeRequired"), "error");
    return;
  }

  button.disabled = true;
  writeWorkflowLog(t("workflow.winpmemInstalling"));
  updateSide("last-action", t("workflow.winpmemInstalling"));
  setProgress(0, "0%");
  try {
    const start = await apiRequest("/api/winpmem-install", { method: "POST" });
    if (!start.job_id) throw new Error(t("workflow.jobIdMissing"));

    // Wait for the download/install job to finish
    const result = await waitForAcquisitionJob(start.job_id);

    const status = result.status || {};
    const path = status.tool_path || result.path || "C:\\Tools\\go-winpmem_amd64_1.0-rc2_signed.exe";
    const label = status.message || result.message || "WinPMEM ready";
    const select = document.querySelector("[data-field='target']");
    if (select) {
      const localLabel = localText(workflow.diskLabel);
      select.innerHTML = [
        `<option value="${escapeHtml(localLabel)}">${escapeHtml(localLabel)}</option>`,
        `<option value="${escapeHtml(path)}">${escapeHtml(path)}</option>`
      ].join("");
      select.value = path;
    }
    updateSide("target", escapeHtml(path));
    writeWorkflowLog(t("scan.toolDoneLog", { target: "WinPMEM", message: escapeHtml(label) }));
    writeWorkflowLog(t("workflow.winpmemInstalled", { path: escapeHtml(path) }));
    showToast(t("workflow.winpmemInstalled", { path }));
  } catch (error) {
    setProgress(0);
    writeWorkflowLog(t("workflow.winpmemInstallFailed", { message: escapeHtml(error.message) }));
    showToast(t("workflow.winpmemInstallFailed", { message: error.message }), "error");
  } finally {
    button.disabled = false;
  }
}

function setProgress(value, labelText = `${value}%`) {
  const progress = document.querySelector("[data-progress]");
  if (!progress) return;
  const next = `${value}%`;
  progress.style.setProperty("--value", next);
  const label = progress.querySelector("b");
  if (label) label.textContent = labelText;
}

function acquisitionPercent(job) {
  const done = Number(job?.done || 0);
  const total = Number(job?.total || 0);
  if (!Number.isFinite(done) || !Number.isFinite(total) || total <= 0) return 0;
  return Math.max(0, Math.min(100, Math.floor((done * 100) / total)));
}

async function waitForAcquisitionJob(jobId) {
  while (true) {
    const job = await apiRequest("/api/acquisition-status", {
      method: "POST",
      body: JSON.stringify({ job_id: jobId })
    });
    const percent = acquisitionPercent(job);
    setProgress(percent, `${percent}%`);
    if (job.message) updateSide("last-action", job.message);

    if (job.status === "completed") {
      setProgress(100, "100%");
      return job.result || {};
    }
    if (job.status === "failed") {
      throw new Error(job.error || job.message || t("acquisitionFailed"));
    }

    await new Promise((resolve) => window.setTimeout(resolve, 500));
  }
}

async function sendAcquisitionControl(action) {
  const active = state.activeAcquisition;
  if (!active || !active.jobId || !active.workflowId) {
    showToast(t("workflow.activeJobMissing"), "error");
    return;
  }
  const workflow = workflows[active.workflowId];
  const body = {
    job_id: active.jobId,
    action
  };
  if (workflow?.mode.startsWith("remote")) {
    Object.assign(body, active.payload || {});
  }

  await apiRequest("/api/acquisition-control", {
    method: "POST",
    body: JSON.stringify(body)
  });
  const label = action === "stop" ? t("workflow.stopLabel") : action === "pause" ? t("workflow.pauseLabel") : t("workflow.resumeLabel");
  const message = workflow?.mode.startsWith("remote")
    ? t("workflow.controlSent", { label })
    : t("workflow.controlApplied", { label });
  writeWorkflowLog(message);
  updateSide("last-action", message);
  showToast(message);
}

async function startAcquisition(button) {
  const routeId = state.route.split(":")[1];
  const workflow = workflows[routeId];
  const isRam = workflow?.mode.includes("ram");
  let payload = null;
  if (workflow?.mode.startsWith("remote")) {
    try {
      payload = connectionPayload();
    } catch (error) {
      showToast(error.message, "error");
      return;
    }
    if (!requireActiveConnection(workflow, payload)) return;
  }
  const target = document.querySelector("[data-field='target']")?.value.trim();
  if (workflow && !workflow.mode.includes("ram") && !target) {
    showToast(t("workflow.diskRequired"), "error");
    return;
  }
  let output = document.querySelector("#workflow-output")?.value.trim() || "";
  const diskName = isRam ? "" : selectedTargetName();
  let caseName = null;
  button.disabled = true;
  window.clearInterval(state.jobs.workflow);
  setProgress(0, "0%");
  const operation = isRam ? t("ramAcquisition") : t("imageAcquisition");

  try {
    await loadEvidenceCases();
    const evidenceCase = await ensureImageCase();
    caseName = evidenceCase.case_name;
    if (isRam) {
      const remoteIp = workflow?.mode.startsWith("remote") ? payload?.ip : "";
      const fileName = canonicalRamFileName(remoteIp);
      const outputInput = document.querySelector("#workflow-output");
      if (outputInput) outputInput.value = fileName;
      const ramDir = evidenceCase.ram_dir || `${evidenceCase.case_dir}/ram`;
      output = `${ramDir}/${fileName}`;
    } else {
      output = evidenceCase.output_dir || `${evidenceCase.case_dir}/ciktilar`;
    }
    document.querySelectorAll("[data-case-output]").forEach((outputNode) => {
      outputNode.textContent = outputNode.dataset.caseOutputSubdir === "ram"
        ? (evidenceCase.ram_dir || `${evidenceCase.case_dir}/ram`)
        : (evidenceCase.output_dir || `${evidenceCase.case_dir}/ciktilar`);
    });

    writeWorkflowLog(t("workflow.operationStarted", { operation }));
    updateSide("last-action", t("workflow.operationRunning", { operation }));
    if (workflow?.mode.startsWith("remote")) updateSide("connection", t("workflow.operationRunning", { operation }));

    const start = workflow?.mode.startsWith("remote")
      ? await apiRequest(isRam ? "/api/remote-ram" : "/api/remote-image", {
          method: "POST",
          body: JSON.stringify(isRam
            ? {
                ...payload,
                output,
                case_name: caseName
              }
            : {
                ...payload,
                disk_id: target,
                disk_name: diskName,
                output,
                case_name: caseName
              })
        })
      : await apiRequest(isRam ? "/api/local-ram" : "/api/local-image", {
          method: "POST",
          body: JSON.stringify(isRam
            ? {
                output,
                tool: workflow.platform === "Windows" ? "winpmem" : "avml",
                tool_path: target,
                case_name: caseName
            }
            : {
                source: target,
                disk_name: diskName,
                output,
                case_name: caseName
              })
        });
    if (!start.job_id) throw new Error(t("workflow.jobIdMissing"));
    state.activeAcquisition = {
      jobId: start.job_id,
      workflowId: routeId,
      payload
    };
    const result = await waitForAcquisitionJob(start.job_id);

    setProgress(100);
    const targetPath = result.target_path || result.target || output;
    writeWorkflowLog(t("workflow.operationCompletedPath", { operation, path: targetPath }));
    updateSide("last-action", t("workflow.operationCompleted", { operation }));
    if (workflow?.mode.startsWith("remote") && payload) {
      updateSide("connection", t("connection.connected", { ip: payload.ip }));
    }
    showToast(t("workflow.operationCompleted", { operation }));
  } catch (error) {
    setProgress(0);
    writeWorkflowLog(t("workflow.operationFailedDetail", { operation, message: error.message }));
    updateSide("last-action", t("workflow.operationFailed", { operation }));
    if (workflow?.mode.startsWith("remote")) {
      updateSide("connection", t("workflow.operationFailed", { operation }));
    }
    showToast(t("workflow.operationFailedDetail", { operation, message: error.message }), "error");
  } finally {
    state.activeAcquisition = null;
    button.disabled = false;
  }
}

function setAnalysisStatus(status, log) {
  const statusNode = document.querySelector("[data-analysis-status]");
  const logNode = document.querySelector("[data-analysis-log]");
  if (statusNode) statusNode.textContent = status;
  if (logNode) logNode.innerHTML = log;
}

function renderTree(node, depth = 0) {
  if (!node) return escapeHtml(t("analysis.outputWaiting"));
  const indent = "&nbsp;".repeat(depth * 2);
  const marker = node.is_dir ? "▸" : "•";
  const size = node.is_dir ? "" : ` <small>${formatBytes(node.size)}</small>`;
  const current = `${indent}${marker} ${escapeHtml(node.name || node.path || "")}${size}`;
  const children = Array.isArray(node.children)
    ? node.children.map((child) => renderTree(child, depth + 1)).join("<br />")
    : "";
  return children ? `${current}<br />${children}` : current;
}

async function calculateHashes() {
  const inputPath = document.querySelector("#hash-file")?.value.trim();
  if (backendReady() && inputPath) {
    try {
      const hashes = await apiRequest("/api/hash", {
        method: "POST",
        body: JSON.stringify({
          path: inputPath,
          algorithms: ["md5", "sha1", "sha256", "sha512"]
        })
      });
      setHashResult("md5", hashes.md5 || "-");
      setHashResult("sha1", hashes.sha1 || "-");
      setHashResult("sha256", hashes.sha256 || "-");
      setHashResult("sha512", hashes.sha512 || "-");
      showToast(t("hash.done"));
      return;
    } catch (error) {
      showToast(t("hash.failed", { message: error.message }), "error");
      return;
    }
  }

  const file = state.files["#hash-file"];
  if (!file) {
    showToast(t("fileRequired"), "error");
    return;
  }
  const buffer = await file.arrayBuffer();
  setHashResult("md5", t("hash.fullAppRequired"));
  setHashResult("sha1", await digestHex("SHA-1", buffer));
  setHashResult("sha256", await digestHex("SHA-256", buffer));
  setHashResult("sha512", await digestHex("SHA-512", buffer));
  showToast(t("hash.done"));
}

async function digestHex(algorithm, buffer) {
  const hash = await crypto.subtle.digest(algorithm, buffer.slice(0));
  return [...new Uint8Array(hash)].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function setHashResult(key, value) {
  const node = document.querySelector(`[data-hash-result="${key}"] strong`);
  if (node) node.textContent = value;
}

function compareHash() {
  const expected = document.querySelector("[data-hash-expected]")?.value.trim().toLowerCase();
  const values = [...document.querySelectorAll("[data-hash-result] strong")].map((node) => node.textContent.trim().toLowerCase());
  const result = document.querySelector("[data-hash-compare-result] small");
  if (!expected) {
    showToast(t("hash.compareRequired"), "error");
    return;
  }
  const matched = values.includes(expected);
  if (result) result.textContent = matched ? t("hash.matched") : t("hash.notMatched");
  showToast(matched ? t("hash.matchedToast") : t("hash.notMatchedToast"), matched ? "success" : "error");
}

function setStatus(selector, html) {
  const node = document.querySelector(selector);
  if (node) node.innerHTML = html;
}

async function createEvidenceCase() {
  const caseName = document.querySelector("#case-name")?.value.trim();
  if (!caseName) {
    showToast(t("case.required"), "error");
    return;
  }
  try {
    const result = await apiRequest("/api/evidence-create", {
      method: "POST",
      body: JSON.stringify({ case_name: caseName })
    });
    state.activeCase = result;
    await loadEvidenceCases();
    setStatus("[data-case-status]", `${icon("info")} ${t("case.created", { path: escapeHtml(result.case_dir) })}`);
    showToast(t("case.created", { path: result.case_dir }));
  } catch (error) {
    setStatus("[data-case-status]", `${icon("info")} ${t("case.createFailed", { message: escapeHtml(error.message) })}`);
    showToast(t("case.createFailed", { message: error.message }), "error");
  }
}

async function listEvidenceFiles() {
  if (!state.activeCase) {
    showToast(t("case.required"), "error");
    return;
  }
  const subdir = document.querySelector("#case-folder")?.value || "ciktilar";
  try {
    const result = await apiRequest("/api/evidence-list-files", {
      method: "POST",
      body: JSON.stringify({ subdir })
    });
    const files = result.files || [];
    const select = document.querySelector("#case-file-list");
    if (select) {
      select.innerHTML = files.length
        ? files.map((file) => `<option value="${escapeHtml(file.path)}">${escapeHtml(file.name)} · ${formatBytes(file.size)}</option>`).join("")
        : `<option>${t("case.empty")}</option>`;
    }
    setStatus("[data-case-status]", `${icon("info")} ${t("case.filesListed", { count: String(files.length) })}`);
    showToast(t("case.filesListed", { count: String(files.length) }));
  } catch (error) {
    showToast(t("case.listFailed", { message: error.message }), "error");
  }
}

async function addEvidenceNote() {
  const note = document.querySelector("#report-note")?.value.trim();
  if (!note) {
    showToast(t("report.noteRequired"), "error");
    return;
  }
  const caseName = reportCaseName();
  try {
    const result = await apiRequest("/api/evidence-add-note", {
      method: "POST",
      body: JSON.stringify({ note, case_name: caseName })
    });
    await loadEvidenceCases();
    setStatus("[data-report-status]", `${icon("info")} ${t("report.noteAdded", { path: escapeHtml(result.path) })}`);
    showToast(t("report.noteAdded", { path: result.path }));
  } catch (error) {
    setStatus("[data-report-status]", `${icon("info")} ${t("report.noteFailed", { message: escapeHtml(error.message) })}`);
    showToast(t("report.noteFailed", { message: error.message }), "error");
  }
}

async function createEvidenceReport() {
  const caseName = reportCaseName();
  const title = document.querySelector("#report-title")?.value.trim() || t("report.defaultTitle");
  const format = document.querySelector("#report-format")?.value || "txt";
  const description = document.querySelector("#report-note")?.value.trim() || "";
  try {
    const result = await apiRequest("/api/report-create", {
      method: "POST",
      body: JSON.stringify({ case_name: caseName, title, description, format })
    });
    await loadEvidenceCases();
    setStatus("[data-report-status]", `${icon("info")} ${t("report.created", { path: escapeHtml(result.path) })}`);
    showToast(t("report.created", { path: result.path }));
  } catch (error) {
    setStatus("[data-report-status]", `${icon("info")} ${t("report.failed", { message: escapeHtml(error.message) })}`);
    showToast(t("report.failed", { message: error.message }), "error");
  }
}

async function downloadUpdatePackage() {
  const progress = document.querySelector("[data-update-progress]");
  const status = document.querySelector("[data-update-status]");
  const update = state.latestUpdate || await apiRequest("/api/update-check");
  state.latestUpdate = update;
  const asset = update.platform_asset || {};
  if (!asset.download_url) {
    showToast(t("settings.noAsset"), "error");
    return;
  }
  if (progress) {
    progress.style.setProperty("--value", "35%");
    const label = progress.querySelector("b");
    if (label) label.textContent = "35%";
  }
  if (status) status.innerHTML = `${icon("download")} ${t("settings.downloading")}`;
  try {
    const result = await apiRequest("/api/update-download", {
      method: "POST",
      body: JSON.stringify({
        url: asset.download_url,
        name: asset.name,
        expected_sha256: asset.digest || ""
      })
    });
    if (progress) {
      progress.style.setProperty("--value", "75%");
      const label = progress.querySelector("b");
      if (label) label.textContent = "75%";
    }
    if (status) status.innerHTML = `${icon("download")} ${t("settings.installing")}`;
    const install = await apiRequest("/api/update-install", {
      method: "POST",
      body: JSON.stringify({ path: result.path })
    });
    if (progress) {
      progress.style.setProperty("--value", "100%");
      const label = progress.querySelector("b");
      if (label) label.textContent = "100%";
    }
    if (status) status.innerHTML = `${icon("shield")} ${t("settings.installStarted")}`;
    setStatus(
      "[data-update-log]",
      `${t("settings.downloaded", { path: escapeHtml(result.path) })}<br />${t("settings.sha256", { hash: escapeHtml(result.sha256) })}<br />${escapeHtml(install.message || t("settings.installStarted"))}`
    );
    showToast(t("settings.installStarted"));
  } catch (error) {
    if (progress) {
      progress.style.setProperty("--value", "0%");
      const label = progress.querySelector("b");
      if (label) label.textContent = "0%";
    }
    const failedKey = String(error.message || "").toLowerCase().includes("installer")
      ? "settings.installFailed"
      : "settings.downloadFailed";
    if (status) status.innerHTML = `${icon("info")} ${t(failedKey, { message: escapeHtml(error.message) })}`;
    showToast(t(failedKey, { message: error.message }), "error");
  }
}

function detailPanel(tab) {
  if (tab === "evidence") {
    return `
      <p class="section-label">${t("case.management")}</p>
      <div class="side-info">
        <span class="metric-icon">${icon("folder")}</span>
        <span><strong>${t("case.location")}</strong><small data-case-base>${escapeHtml(state.caseBaseDir || "~/Worm/Vakalar")}</small></span>
      </div>
      <p class="field-hint">${t("case.fixedLocation")}</p>
      ${field(t("case.name"), '<input id="case-name" class="input" placeholder="Case_2026_001" />')}
      <div class="button-row">
        <button class="primary-button" data-action="create-case">${icon("folder")} ${t("case.create")}</button>
        <button class="secondary-button" data-action="refresh-cases">${icon("refresh")} ${t("case.refresh")}</button>
      </div>
      <div class="status-badge" data-case-status>${icon("info")} ${state.activeCase ? t("case.created", { path: state.activeCase.case_dir }) : t("case.notCreated")}</div>
      <div class="section-divider"></div>
      <p class="section-label">${t("case.files")}</p>
      ${field(t("case.folder"), `<select id="case-folder" class="select"><option value="ciktilar">${t("case.outputs")}</option><option value="disk_imajlari">${t("case.diskImages")}</option><option value="ram">${t("case.ram")}</option><option value="raporlar">${t("case.reports")}</option><option value="hash">${t("case.hash")}</option><option value="notlar">${t("case.notes")}</option><option value="gunlukler">${t("case.logs")}</option></select>`)}
      ${field(t("case.file"), `<select id="case-file-list" class="select"><option>${t("case.listFilesPlaceholder")}</option></select>`)}
      <div class="button-row">
        <button class="secondary-button" data-action="list-files">${icon("search")} ${t("case.listFiles")}</button>
      </div>
    `;
  }
  if (tab === "reports") {
    const autoCaseField = state.cases.length
      ? ""
      : field(t("report.autoCase"), `<input id="report-case-name" class="input" value="${defaultCaseName()}" />`);
    return `
      <p class="section-label">${t("report.createTitle")}</p>
      <p class="field-hint">${t("report.hint")}</p>
      ${field(t("report.case"), `<select id="report-case" class="select" data-case-select>${caseSelectOptions(state.activeCase?.case_name)}</select>`)}
      ${autoCaseField}
      ${field(t("report.title"), `<input id="report-title" class="input" value="${t("report.defaultTitle")}" />`)}
      ${field(t("report.format"), '<select id="report-format" class="select"><option value="txt">TXT</option><option value="json">JSON</option></select>')}
      ${field(t("report.note"), `<textarea id="report-note" class="textarea" placeholder="${t("report.notePlaceholder")}"></textarea>`)}
      <div class="button-row">
        <button class="secondary-button" data-action="refresh-cases">${icon("refresh")} ${t("case.refresh")}</button>
        <button class="secondary-button" data-action="add-note">${icon("report")} ${t("report.addNote")}</button>
        <button class="primary-button" data-action="create-report">${icon("report")} ${t("report.createTitle")}</button>
      </div>
      <div class="status-badge" data-report-status>${icon("info")} ${t("ready")}</div>
    `;
  }
  if (tab === "logs") {
    return `
      <p class="section-label">${t("other.logs.title")}</p>
      <p class="field-hint">${t("log.live")}</p>
      <div class="log-box">${state.lastLog.map((line) => `• ${line}`).join("<br />")}</div>
      <div class="button-row" style="margin-top:12px">
        <button class="secondary-button" data-action="refresh-log">${icon("refresh")} ${t("log.refreshFromFile")}</button>
      </div>
    `;
  }
  return hashPanel();
}

setLanguage(state.language);
setTheme(state.theme);
hydrateIcons();
render();
