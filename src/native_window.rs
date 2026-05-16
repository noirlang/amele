#[cfg(target_os = "linux")]
pub fn prepare_environment() {
    linux::prepare_environment();
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn prepare_environment() {}

#[cfg(target_os = "windows")]
pub fn prepare_environment() {
    windows::prepare_environment();
}

#[cfg(target_os = "linux")]
pub fn run(url: &str) -> Result<(), String> {
    linux::run(url)
}

#[cfg(target_os = "windows")]
pub fn run(url: &str) -> Result<(), String> {
    windows::run(url)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn run(_url: &str) -> Result<(), String> {
    Err("Native UI window is supported on Linux and Windows".to_string())
}

#[cfg(target_os = "linux")]
mod linux {
    use std::ffi::CString;
    use std::os::raw::{c_char, c_int, c_ulong, c_void};
    use std::ptr;

    const GTK_WINDOW_TOPLEVEL: c_int = 0;

    pub fn prepare_environment() {
        set_env_if_missing("GDK_BACKEND", "x11");
        set_env_if_missing("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    fn set_env_if_missing(key: &str, value: &str) {
        if std::env::var_os(key).is_none() {
            unsafe {
                std::env::set_var(key, value);
            }
        }
    }

    #[link(name = "gtk-3")]
    unsafe extern "C" {
        fn gtk_init(argc: *mut c_int, argv: *mut *mut *mut c_char);
        fn gtk_window_new(window_type: c_int) -> *mut c_void;
        fn gtk_window_set_title(window: *mut c_void, title: *const c_char);
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

    pub fn run(url: &str) -> Result<(), String> {
        let title = CString::new("Worm Forensic Tool").map_err(|err| err.to_string())?;
        let destroy = CString::new("destroy").map_err(|err| err.to_string())?;
        let uri = CString::new(url).map_err(|err| err.to_string())?;

        unsafe {
            gtk_init(ptr::null_mut(), ptr::null_mut());

            let window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
            if window.is_null() {
                return Err("GTK window could not be created".to_string());
            }

            let webview = webkit_web_view_new();
            if webview.is_null() {
                return Err("WebKit webview could not be created".to_string());
            }

            gtk_window_set_title(window, title.as_ptr());
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

    unsafe extern "C" fn on_destroy(_widget: *mut c_void, _data: *mut c_void) {
        unsafe {
            gtk_main_quit();
        }
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use winit::application::ApplicationHandler;
    use winit::dpi::LogicalSize;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::window::{Window, WindowId};
    use wry::WebViewBuilder;

    pub fn prepare_environment() {
        if std::env::var_os("WEBVIEW2_USER_DATA_FOLDER").is_none() {
            let dir = std::env::temp_dir().join("worm-webview2");
            unsafe {
                std::env::set_var("WEBVIEW2_USER_DATA_FOLDER", dir);
            }
        }
    }

    pub fn run(url: &str) -> Result<(), String> {
        let event_loop = EventLoop::new().map_err(|err| err.to_string())?;
        let mut app = WindowsApp {
            url: url.to_string(),
            window: None,
            webview: None,
        };
        event_loop.run_app(&mut app).map_err(|err| err.to_string())
    }

    struct WindowsApp {
        url: String,
        window: Option<Window>,
        webview: Option<wry::WebView>,
    }

    impl ApplicationHandler for WindowsApp {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            if self.window.is_some() {
                return;
            }

            let attributes = Window::default_attributes()
                .with_title("Worm Forensic Tool")
                .with_inner_size(LogicalSize::new(1280.0, 820.0));
            let window = event_loop
                .create_window(attributes)
                .expect("Windows native window could not be created");
            let webview = WebViewBuilder::new()
                .with_url(&self.url)
                .build(&window)
                .expect("WebView2 view could not be created");

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
}
