// Windows adli araçlar bileşeni.
// WinPmem sürücüsünü sisteme yükleme, VSS (Volume Shadow Copy) tarama ve
// sayfa dosyası (pagefile.sys) dökümü araçlarını yönetir.

export function windowsPage({ t, icon, state, pageTitle, toolHub }) {
  return toolHub("windows");
}
