//! Platforma göre native pencere açma ve WebView ortam hazırlığını yapar.
#[cfg(target_os = "linux")]
/// Linux WebKit/GTK ortam değişkenlerini hazırlar.
pub fn prepare_environment() -> Result<(), String> {
    linux::prepare_environment()
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
/// Desteklenmeyen platformlarda ek ortam hazırlığı yapmadan döner.
pub fn prepare_environment() -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "windows")]
/// Windows WebView2 kullanıcı veri klasörünü hazırlar.
pub fn prepare_environment() -> Result<(), String> {
    windows::prepare_environment()
}

#[cfg(target_os = "linux")]
/// Linux GTK/WebKit penceresini verilen URL ile açar.
pub fn run(url: &str) -> Result<(), String> {
    linux::run(url)
}

#[cfg(target_os = "windows")]
/// Windows WRY/WebView2 penceresini verilen URL ile açar.
pub fn run(url: &str) -> Result<(), String> {
    windows::run(url)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
/// Native pencere desteği olmayan platformlarda hata döndürür.
pub fn run(_url: &str) -> Result<(), String> {
    Err("Native UI window is supported on Linux and Windows".to_string())
}

#[cfg(target_os = "linux")]
mod linux {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int, c_ulong, c_void};
    use std::path::{Path, PathBuf};
    use std::ptr;

    const GTK_WINDOW_TOPLEVEL: c_int = 0;

    /// GTK/WebKit render ayarlarını güvenli varsayılanlara çeker.
    pub fn prepare_environment() -> Result<(), String> {
        set_env_if_missing("GDK_BACKEND", "x11");
        set_env_if_missing("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        Ok(())
    }

    /// Ortam değişkeni yoksa varsayılan değer atar.
    fn set_env_if_missing(key: &str, value: &str) {
        if std::env::var_os(key).is_none() {
            unsafe {
                std::env::set_var(key, value);
            }
        }
    }

    #[link(name = "gtk-3")]
    unsafe extern "C" {
        fn gtk_init_check(argc: *mut c_int, argv: *mut *mut *mut c_char) -> c_int;
        fn gtk_window_new(window_type: c_int) -> *mut c_void;
        fn gtk_window_set_title(window: *mut c_void, title: *const c_char);
        fn gtk_window_set_icon_from_file(
            window: *mut c_void,
            filename: *const c_char,
            error: *mut *mut c_void,
        ) -> c_int;
        fn gtk_window_set_default_size(window: *mut c_void, width: c_int, height: c_int);
        fn gtk_container_add(container: *mut c_void, widget: *mut c_void);
        fn gtk_widget_show_all(widget: *mut c_void);
        fn gtk_main();
        fn gtk_main_quit();
    }

    #[link(name = "webkit2gtk-4.1")]
    unsafe extern "C" {
        fn webkit_web_view_new() -> *mut c_void;
        fn webkit_web_view_load_uri(web_view: *mut c_void, uri: *const c_char);
    }

    #[link(name = "gobject-2.0")]
    unsafe extern "C" {
        fn g_signal_connect_data(
            instance: *mut c_void,
            detailed_signal: *const c_char,
            c_handler: Option<unsafe extern "C" fn(*mut c_void, *mut c_void)>,
            data: *mut c_void,
            destroy_data: Option<unsafe extern "C" fn(*mut c_void)>,
            connect_flags: c_int,
        ) -> c_ulong;
    }

    /// GTK penceresi ve WebKit view oluşturup UI URL'ini yükler.
    pub fn run(url: &str) -> Result<(), String> {
        ensure_webkit_helper_available()?;

        let title = CString::new("Worm Forensic Tool").map_err(|err| err.to_string())?;
        let destroy = CString::new("destroy").map_err(|err| err.to_string())?;
        let uri = CString::new(url).map_err(|err| err.to_string())?;

        unsafe {
            if gtk_init_check(ptr::null_mut(), ptr::null_mut()) == 0 {
                return Err(crate::diagnostics::startup_error(
                    "GTK ekrana baglanamadi.",
                    "DISPLAY/WAYLAND_DISPLAY ayari yok veya grafik oturum erisimi reddedildi.",
                ));
            }

            let window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
            if window.is_null() {
                return Err(crate::diagnostics::startup_error(
                    "GTK penceresi olusturulamadi.",
                    "gtk_window_new null dondu.",
                ));
            }

            let webview = webkit_web_view_new();
            if webview.is_null() {
                return Err(crate::diagnostics::startup_error(
                    "WebKit webview olusturulamadi.",
                    "webkit_web_view_new null dondu. WebKitGTK runtime veya grafik bagimliliklari eksik olabilir.",
                ));
            }

            gtk_window_set_title(window, title.as_ptr());
            if let Some(icon_path) = app_icon_path()
                && let Ok(icon_path) = CString::new(icon_path.to_string_lossy().as_bytes())
            {
                gtk_window_set_icon_from_file(window, icon_path.as_ptr(), ptr::null_mut());
            }
            gtk_window_set_default_size(window, 1280, 820);
            gtk_container_add(window, webview);
            g_signal_connect_data(
                window,
                destroy.as_ptr(),
                Some(on_destroy),
                ptr::null_mut(),
                None,
                0,
            );
            webkit_web_view_load_uri(webview, uri.as_ptr());
            gtk_widget_show_all(window);
            gtk_main();
        }

        Ok(())
    }

    /// GTK pencere kapanınca ana döngüyü sonlandırır.
    unsafe extern "C" fn on_destroy(_widget: *mut c_void, _data: *mut c_void) {
        unsafe {
            gtk_main_quit();
        }
    }

    /// WebKitNetworkProcess binary'si bulunabiliyor mu diye başlangıç kontrolü yapar.
    fn ensure_webkit_helper_available() -> Result<(), String> {
        if std::env::var_os("WORM_SKIP_WEBKIT_HELPER_CHECK").is_some() {
            return Ok(());
        }

        let candidates = webkit_helper_candidates();
        if candidates.iter().any(|path| path.is_file()) {
            return Ok(());
        }

        let searched = candidates
            .iter()
            .map(|path| format!("  - {}", path.display()))
            .collect::<Vec<_>>()
            .join("\n");
        Err(crate::diagnostics::startup_error(
            "WebKit helper process bulunamadi.",
            &format!(
                "WebKitNetworkProcess su yollarda bulunamadi:\n{searched}\nDebian/Ubuntu: sudo apt install libwebkit2gtk-4.1-0\nFedora: sudo dnf install webkit2gtk4.1\nAppImage paketinde bu yardimci binary de paketlenmeli."
            ),
        ))
    }

    /// Paket içi ve sistem WebKit helper aday yollarını üretir.
    fn webkit_helper_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        push_helper_from_env(&mut candidates, "WEBKIT_EXEC_PATH");

        if let Some(appdir) = std::env::var_os("APPDIR") {
            push_helper_dirs(&mut candidates, PathBuf::from(appdir).join("usr"));
        }

        if let Ok(exe) = std::env::current_exe()
            && let Some(bin_dir) = exe.parent()
            && let Some(prefix) = bin_dir.parent()
        {
            push_helper_dirs(&mut candidates, prefix);
        }

        candidates.push(PathBuf::from(
            "/usr/lib/x86_64-linux-gnu/webkit2gtk-4.1/WebKitNetworkProcess",
        ));
        candidates.push(PathBuf::from(
            "/usr/libexec/webkit2gtk-4.1/WebKitNetworkProcess",
        ));
        candidates.push(PathBuf::from(
            "/usr/lib/webkit2gtk-4.1/WebKitNetworkProcess",
        ));
        candidates.push(PathBuf::from(
            "/usr/lib64/webkit2gtk-4.1/WebKitNetworkProcess",
        ));

        candidates.sort();
        candidates.dedup();
        candidates
    }

    /// Ortam değişkeninden gelen WebKit helper klasörünü adaylara ekler.
    fn push_helper_from_env(candidates: &mut Vec<PathBuf>, key: &str) {
        if let Some(path) = std::env::var_os(key) {
            candidates.push(PathBuf::from(path).join("WebKitNetworkProcess"));
        }
    }

    /// Prefix altındaki yaygın WebKit helper yollarını adaylara ekler.
    fn push_helper_dirs(candidates: &mut Vec<PathBuf>, prefix: impl AsRef<Path>) {
        let prefix = prefix.as_ref();
        candidates.push(
            prefix
                .join("lib")
                .join("webkit2gtk-4.1")
                .join("WebKitNetworkProcess"),
        );
        candidates.push(
            prefix
                .join("libexec")
                .join("webkit2gtk-4.1")
                .join("WebKitNetworkProcess"),
        );
        candidates.push(
            prefix
                .join("lib")
                .join("x86_64-linux-gnu")
                .join("webkit2gtk-4.1")
                .join("WebKitNetworkProcess"),
        );
    }

    /// Native pencere ikon yolunu ortam değişkeni veya geliştirme assetinden bulur.
    fn app_icon_path() -> Option<std::path::PathBuf> {
        if let Some(path) = std::env::var_os("WORM_APP_ICON") {
            let path = std::path::PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        let dev_icon =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/assets/logo/icon.png");
        dev_icon.exists().then_some(dev_icon)
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::path::{Path, PathBuf};

    use winit::application::ApplicationHandler;
    use winit::dpi::LogicalSize;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window, WindowId};
    use wry::WebViewBuilder;

    /// WebView2 kullanıcı veri klasörü yoksa geçici klasörde hazırlar.
    pub fn prepare_environment() -> Result<(), String> {
        if std::env::var_os("WEBVIEW2_USER_DATA_FOLDER").is_none() {
            let dir = std::env::temp_dir().join("worm-webview2");
            std::fs::create_dir_all(&dir).map_err(|err| {
                crate::diagnostics::startup_error(
                    "WebView2 kullanici veri klasoru hazirlanamadi.",
                    &format!("{}: {err}", dir.display()),
                )
            })?;
            unsafe {
                std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", dir);
            }
        }
        Ok(())
    }

    /// Windows olay döngüsünü başlatıp WebView penceresini çalıştırır.
    pub fn run(url: &str) -> Result<(), String> {
        let event_loop = EventLoop::new().map_err(|err| {
            crate::diagnostics::startup_error(
                "Windows olay dongusu olusturulamadi.",
                &err.to_string(),
            )
        })?;
        let mut app = WindowsApp {
            url: url.to_string(),
            window: None,
            webview: None,
            startup_error: None,
        };
        let result = event_loop.run_app(&mut app).map_err(|err| err.to_string());
        if let Some(err) = app.startup_error.take() {
            return Err(err);
        }
        result
    }

    /// Winit uygulama yaşam döngüsü boyunca pencere ve WebView nesnelerini tutar.
    struct WindowsApp {
        url: String,
        window: Option<Window>,
        webview: Option<wry::WebView>,
        startup_error: Option<String>,
    }

    impl ApplicationHandler for WindowsApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }

            let attributes = Window::default_attributes()
                .with_title("Worm Forensic Tool")
                .with_inner_size(LogicalSize::new(1280.0, 820.0));
            let window = match event_loop.create_window(attributes) {
                Ok(window) => window,
                Err(err) => {
                    self.fail_startup(
                        event_loop,
                        crate::diagnostics::startup_error(
                            "Windows native pencere olusturulamadi.",
                            &err.to_string(),
                        ),
                    );
                    return;
                }
            };
            let webview = match WebViewBuilder::new().with_url(&self.url).build(&window) {
                Ok(webview) => webview,
                Err(err) => {
                    self.fail_startup(event_loop, webview2_error_message(&err.to_string()));
                    return;
                }
            };

            self.webview = Some(webview);
            self.window = Some(window);
        }

        fn window_event(
            &mut self,
            event_loop: &ActiveEventLoop,
            _window_id: WindowId,
            event: WindowEvent,
        ) {
            if matches!(event, WindowEvent::CloseRequested) {
                event_loop.exit();
            }
        }
    }

    impl WindowsApp {
        /// Başlangıç hatasını saklayıp olay döngüsünü kapatır.
        fn fail_startup(&mut self, event_loop: &ActiveEventLoop, message: String) {
            self.startup_error = Some(message);
            event_loop.exit();
        }
    }

    /// WebView2 hatasını runtime kontrolüyle birlikte açıklayıcı mesaja çevirir.
    fn webview2_error_message(err: &str) -> String {
        let mut message = format!(
            "WebView2 view could not be created: {err}\nInstall Microsoft Edge WebView2 Evergreen Runtime and start Worm again."
        );
        let candidates = webview2_candidates();
        if !candidates.iter().any(|path| path.exists()) {
            let searched = candidates
                .iter()
                .map(|path| format!("  - {}", path.display()))
                .collect::<Vec<_>>()
                .join("\n");
            message.push_str(&format!(
                "\nRuntime klasoru bilinen yollarda bulunamadi:\n{searched}"
            ));
        }
        crate::diagnostics::startup_error("Windows WebView2 baslatilamadi.", &message)
    }

    /// WebView2 runtime için bilinen klasör adaylarını üretir.
    fn webview2_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        if let Some(path) = std::env::var_os("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER") {
            candidates.push(PathBuf::from(path));
        }
        push_program_files_candidate(&mut candidates, "PROGRAMFILES");
        push_program_files_candidate(&mut candidates, "PROGRAMFILES(X86)");
        candidates.sort();
        candidates.dedup();
        candidates
    }

    /// Program Files tabanlı WebView2 aday yolunu ekler.
    fn push_program_files_candidate(candidates: &mut Vec<PathBuf>, key: &str) {
        if let Some(base) = std::env::var_os(key) {
            candidates.push(
                Path::new(&base)
                    .join("Microsoft")
                    .join("EdgeWebView")
                    .join("Application"),
            );
        }
    }
}
