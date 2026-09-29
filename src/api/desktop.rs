//! tarayıcıda link açma ve masaüstü kontrolleri api rotası.

use serde::Deserialize;
use serde_json::json;
use std::process::{Command, Stdio};

use crate::server::{Response, json_error, json_ok};

/// Güvenli harici URL'yi sistem tarayıcısında açar.
pub fn open_url_endpoint(body: &[u8]) -> Response {
    #[derive(Deserialize)]
    struct OpenUrlRequest {
        url: String,
    }

    let request: OpenUrlRequest = match serde_json::from_slice(body) {
        Ok(request) => request,
        Err(err) => return json_error(400, err.to_string()),
    };

    let url = match validate_external_url(&request.url) {
        Ok(url) => url,
        Err(err) => return json_error(400, err),
    };

    open_external_url(&url)
        .map(|()| json_ok(json!({ "opened": true })))
        .unwrap_or_else(|err| json_error(500, err))
}

/// Native dosya/klasör seçici açar.
pub fn pick_path_endpoint(directory: bool) -> Response {
    match pick_path(directory) {
        Ok(Some(path)) => json_ok(json!({ "path": path })),
        Ok(None) => json_error(499, "selection cancelled"),
        Err(err) => json_error(500, err),
    }
}

/// Platforma göre dosya veya klasör seçici çalıştırır.
fn pick_path(directory: bool) -> Result<Option<String>, String> {
    #[cfg(windows)]
    {
        pick_path_windows(directory)
    }

    #[cfg(not(windows))]
    {
        pick_path_unix(directory)
    }
}

#[cfg(not(windows))]
/// Unix ortamında zenity/kdialog/yad ile dosya seçici açar.
fn pick_path_unix(directory: bool) -> Result<Option<String>, String> {
    let candidates: &[(&str, &[&str])] = if directory {
        &[
            ("zenity", &["--file-selection", "--directory"]),
            ("kdialog", &["--getexistingdirectory"]),
            ("yad", &["--file", "--directory"]),
        ]
    } else {
        &[
            ("zenity", &["--file-selection"]),
            ("kdialog", &["--getopenfilename"]),
            ("yad", &["--file"]),
        ]
    };

    let mut last_error = String::new();
    for (program, args) in candidates {
        match Command::new(program).args(*args).output() {
            Ok(output) if output.status.success() => {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if path.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(path));
            }
            Ok(output) => {
                if output.status.code() == Some(1) || output.status.code() == Some(252) {
                    return Ok(None);
                }
                last_error = String::from_utf8_lossy(&output.stderr).trim().to_string();
            }
            Err(err) => last_error = err.to_string(),
        }
    }

    Err(if last_error.is_empty() {
        "no file picker command found".to_string()
    } else {
        last_error
    })
}

#[cfg(windows)]
/// Windows PowerShell ile dosya veya klasör seçici açar.
fn pick_path_windows(directory: bool) -> Result<Option<String>, String> {
    let script = if directory {
        r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.FolderBrowserDialog
$dialog.ShowNewFolderButton = $true
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
  [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
  Write-Output $dialog.SelectedPath
  exit 0
}
exit 1
"#
    } else {
        r#"
Add-Type -AssemblyName System.Windows.Forms
$dialog = New-Object System.Windows.Forms.OpenFileDialog
$dialog.CheckFileExists = $true
$dialog.Multiselect = $false
$dialog.Filter = 'All files (*.*)|*.*'
if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
  [Console]::OutputEncoding = [System.Text.Encoding]::UTF8
  Write-Output $dialog.FileName
  exit 0
}
exit 1
"#
    };

    let output = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-STA")
        .arg("-Command")
        .arg(script)
        .stdin(Stdio::null())
        .output()
        .map_err(|err| format!("Windows file picker baslatilamadi: {err}"))?;

    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path.is_empty() {
            Ok(None)
        } else {
            Ok(Some(path))
        }
    } else if output.status.code() == Some(1) {
        Ok(None)
    } else {
        let error = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if error.is_empty() {
            "Windows file picker acilamadi".to_string()
        } else {
            error
        })
    }
}

/// Platforma göre harici URL açma komutunu çalıştırır.
fn open_external_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        let openers: &[(&str, &[&str])] = &[("xdg-open", &[url]), ("gio", &["open", url])];
        for (program, args) in openers {
            if Command::new(program)
                .args(*args)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .is_ok()
            {
                return Ok(());
            }
        }
        Err("external link opener could not be started".to_string())
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("rundll32")
            .arg("url.dll,FileProtocolHandler")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|err| format!("external link opener could not be started: {err}"))
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|err| format!("external link opener could not be started: {err}"))
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = url;
        Err("external links are not supported on this platform".to_string())
    }
}

/// Sadece izin verilen URL şemalarını kabul eder.
fn validate_external_url(value: &str) -> Result<String, String> {
    let url = value.trim();
    if url.is_empty() {
        return Err("url is required".to_string());
    }
    if url.chars().any(char::is_control) {
        return Err("url contains invalid characters".to_string());
    }

    let lower = url.to_ascii_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") || lower.starts_with("mailto:")
    {
        Ok(url.to_string())
    } else {
        Err("only http, https and mailto links can be opened".to_string())
    }
}

/// Sistemde kurulu olan CLI araçlarını tespit eder.
pub fn check_binary(binary_name: &str) -> Option<String> {
    let clean = binary_name.trim();
    if clean.is_empty() {
        return None;
    }

    // 1. Doğrudan dosya yolu verilmişse kontrol et
    let direct_path = std::path::Path::new(clean);
    if direct_path.is_file() {
        return Some(clean.to_string());
    }

    // 2. PATH ortam değişkeni üzerinden tara
    if let Some(paths) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&paths) {
            let candidate = dir.join(clean);
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().to_string());
            }
            #[cfg(windows)]
            {
                let candidate_exe = dir.join(format!("{clean}.exe"));
                if candidate_exe.is_file() {
                    return Some(candidate_exe.to_string_lossy().to_string());
                }
            }
        }
    }

    // 3. which / where komutu
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("which").arg(clean).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() && std::path::Path::new(&path).is_file() {
                    return Some(path);
                }
            }
        }
    }
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("where").arg(clean).output() {
            if output.status.success() {
                let first_line = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                if !first_line.is_empty() && std::path::Path::new(&first_line).is_file() {
                    return Some(first_line);
                }
            }
        }
    }

    // 4. Standart kullanıcı ikili dizinleri fallback
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from);

    if let Some(h) = home {
        let fallbacks = [
            h.join(".local/bin").join(clean),
            h.join(".cargo/bin").join(clean),
            h.join(".local/share/mise/shims").join(clean),
            std::path::PathBuf::from("/usr/local/bin").join(clean),
            std::path::PathBuf::from("/usr/bin").join(clean),
        ];
        for fb in fallbacks {
            if fb.is_file() {
                return Some(fb.to_string_lossy().to_string());
            }
        }
    }

    None
}

/// Uygun sistem terminal emülatörünü tespit eder.
pub fn find_terminal_command(script_path: &str) -> Option<(String, Vec<String>)> {
    #[cfg(unix)]
    {
        // 1. $TERMINAL ortam değişkeni
        if let Ok(term) = std::env::var("TERMINAL") {
            let term_clean = term.trim().to_string();
            if !term_clean.is_empty() && check_binary(&term_clean).is_some() {
                let args = if term_clean.contains("xdg-terminal-exec")
                    || term_clean.contains("gnome-terminal")
                    || term_clean.contains("kgx")
                {
                    vec![
                        "--".to_string(),
                        "bash".to_string(),
                        script_path.to_string(),
                    ]
                } else {
                    vec![
                        "-e".to_string(),
                        "bash".to_string(),
                        script_path.to_string(),
                    ]
                };
                return Some((term_clean, args));
            }
        }

        // 2. xdg-terminal-exec (standart masaüstü terminal başlatıcısı)
        if check_binary("xdg-terminal-exec").is_some() {
            return Some((
                "xdg-terminal-exec".to_string(),
                vec![
                    "--".to_string(),
                    "bash".to_string(),
                    script_path.to_string(),
                ],
            ));
        }

        // 3. Bilinen popüler Linux terminal emülatörleri
        let candidates = [
            ("alacritty", vec!["-e", "bash", script_path]),
            ("kitty", vec!["-e", "bash", script_path]),
            ("ghostty", vec!["-e", "bash", script_path]),
            ("foot", vec!["bash", script_path]),
            ("wezterm", vec!["start", "--", "bash", script_path]),
            ("gnome-terminal", vec!["--", "bash", script_path]),
            ("konsole", vec!["-e", "bash", script_path]),
            ("xfce4-terminal", vec!["--", "bash", script_path]),
            ("kgx", vec!["--", "bash", script_path]),
            ("x-terminal-emulator", vec!["-e", "bash", script_path]),
            ("xterm", vec!["-e", "bash", script_path]),
        ];

        for (bin, args) in candidates {
            if check_binary(bin).is_some() {
                return Some((
                    bin.to_string(),
                    args.into_iter().map(String::from).collect(),
                ));
            }
        }
    }

    #[cfg(windows)]
    {
        if check_binary("wt.exe").is_some() || check_binary("wt").is_some() {
            return Some((
                "wt.exe".to_string(),
                vec![
                    "cmd.exe".to_string(),
                    "/k".to_string(),
                    script_path.to_string(),
                ],
            ));
        }
        return Some((
            "cmd.exe".to_string(),
            vec!["/k".to_string(), script_path.to_string()],
        ));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_external_url_valid() {
        assert_eq!(
            validate_external_url("https://github.com/amele-next").unwrap(),
            "https://github.com/amele-next"
        );
        assert_eq!(
            validate_external_url("http://127.0.0.1:8080/test").unwrap(),
            "http://127.0.0.1:8080/test"
        );
        assert_eq!(
            validate_external_url("mailto:support@amele.dev").unwrap(),
            "mailto:support@amele.dev"
        );
    }

    #[test]
    fn test_validate_external_url_invalid() {
        assert!(validate_external_url("").is_err());
        assert!(validate_external_url("   ").is_err());
        assert!(validate_external_url("ftp://example.com").is_err());
        assert!(validate_external_url("javascript:alert(1)").is_err());
        assert!(validate_external_url("file:///etc/passwd").is_err());
        assert!(validate_external_url("https://example.com/test\x00evil").is_err());
    }

    #[test]
    fn test_open_url_endpoint_invalid() {
        let req = serde_json::json!({
            "url": "ftp://invalid-scheme.com"
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = open_url_endpoint(&body);
        assert_eq!(resp.status, 400);

        let req_invalid_json = b"invalid json";
        let resp_err = open_url_endpoint(req_invalid_json);
        assert_eq!(resp_err.status, 400);
    }

    #[test]
    fn test_check_binary() {
        assert!(check_binary("").is_none());
        assert!(check_binary("   ").is_none());
        assert!(check_binary("this_binary_definitely_does_not_exist_xyz123").is_none());

        #[cfg(unix)]
        {
            assert!(check_binary("sh").is_some());
        }
        #[cfg(windows)]
        {
            assert!(check_binary("cmd").is_some() || check_binary("cmd.exe").is_some());
        }
    }

    #[test]
    fn test_find_terminal_command() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let tmp = std::env::temp_dir().join(format!("amele-desktop-fake-term-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&tmp);
            let fake = tmp.join("fake-desktop-term");
            std::fs::write(&fake, "#!/bin/sh\nexit 0\n").unwrap();
            if let Ok(meta) = std::fs::metadata(&fake) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&fake, perms);
            }

            let old = std::env::var("TERMINAL").ok();
            unsafe {
                std::env::set_var("TERMINAL", fake.to_string_lossy().to_string());
            }

            let res = find_terminal_command("/tmp/run.sh");
            assert!(res.is_some());
            let (bin, args) = res.unwrap();
            assert_eq!(bin, fake.to_string_lossy());
            assert!(!args.is_empty());

            unsafe {
                if let Some(v) = old {
                    std::env::set_var("TERMINAL", v);
                } else {
                    std::env::remove_var("TERMINAL");
                }
            }
            let _ = std::fs::remove_file(&fake);
        }
    }
}
