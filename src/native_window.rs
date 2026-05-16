#[cfg(target_os = "linux")]
pub fn prepare_environment() {
    linux::prepare_environment();
}

#[cfg(not(target_os = "linux"))]
pub fn prepare_environment() {}

#[cfg(target_os = "linux")]
pub fn run(url: &str) -> Result<(), String> {
    linux::run(url)
}

#[cfg(not(target_os = "linux"))]
pub fn run(_url: &str) -> Result<(), String> {
    Err("Native UI window is currently implemented for Linux only".to_string())
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
