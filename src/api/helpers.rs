//! Yetki yükseltme, dosya indirme, JSON yardımcıları ve ortak API araçlarını içerir.
use chrono::Local;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

/// Komut stderr çıktısını yoksa fallback mesajını döndürür.
pub fn command_error_message(output: &std::process::Output, fallback: &str) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        fallback.to_string()
    } else {
        stderr
    }
}

/// Sürecin root/admin yetkisiyle çalışıp çalışmadığını kontrol eder.
pub fn process_is_root() -> bool {
    #[cfg(target_os = "linux")]
    {
        unsafe { libc::geteuid() == 0 }
    }

    #[cfg(windows)]
    {
        crate::ram::is_root_or_admin()
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        false
    }
}

/// Linux pkexec veya Windows UAC ile aynı binary'yi yetkili helper olarak başlatır.
pub fn spawn_elevated_helper(args: &[String]) -> Result<Child, String> {
    #[cfg(target_os = "linux")]
    {
        let exe = elevated_helper_executable()?;
        Command::new("pkexec")
            .arg(exe)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|err| format!("pkexec baslatilamadi: {err}"))
    }

    #[cfg(windows)]
    {
        let exe = std::env::current_exe().map_err(|err| err.to_string())?;
        let exe_str = exe.to_string_lossy().to_string();

        let quoted_args: Vec<String> = args
            .iter()
            .map(|a| {
                let escaped = a.replace('\'', "''");
                format!("'{escaped}'")
            })
            .collect();
        let arg_list = quoted_args.join(",");

        let ps_command = format!(
            "$ErrorActionPreference='Stop'; \
             $process = Start-Process -FilePath '{}' -ArgumentList {} -Verb RunAs -Wait -PassThru; \
             exit $process.ExitCode",
            exe_str.replace('\'', "''"),
            arg_list,
        );

        Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(&ps_command)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|err| format!("UAC baslatilamadi: {err}"))
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = args;
        Err("yetki yükseltme bu platformda desteklenmiyor".to_string())
    }
}

#[cfg(target_os = "linux")]
/// AppImage içindeyken doğru helper binary yolunu döndürür.
pub fn elevated_helper_executable() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("APPIMAGE") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }
    std::env::current_exe().map_err(|err| err.to_string())
}

/// Yetkili helper sürecini başlatır ve tamamlanmasını bekler.
pub fn run_elevated_helper_wait(args: &[String]) -> Result<(), String> {
    let mut child = spawn_elevated_helper(args)?;
    let status = child.wait().map_err(|err| err.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("yetki yükseltme iptal edildi veya başarısız oldu".to_string())
    }
}

/// URL'den dosya indirir; Windows'ta PowerShell, diğerlerinde curl kullanır.
pub fn download_file_to_path(url: &str, target: &Path, fallback: &str) -> Result<(), String> {
    #[cfg(windows)]
    let output = {
        let target_str = target.to_string_lossy();
        let ps_command = format!(
            "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; \
             $ProgressPreference = 'SilentlyContinue'; \
             Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
            url.replace('\'', "''"),
            target_str.replace('\'', "''"),
        );
        Command::new("powershell")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-Command")
            .arg(&ps_command)
            .output()
    };

    #[cfg(not(windows))]
    let output = Command::new("curl")
        .arg("-L")
        .arg("--fail")
        .arg("--silent")
        .arg("--show-error")
        .arg("-o")
        .arg(target)
        .arg(url)
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(command_error_message(&output, fallback)),
        Err(err) => Err(err.to_string()),
    }
}

/// Dosya için SHA-256 hash hesaplar.
pub fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|err| err.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; crate::hash::HASH_BUFFER_SIZE];
    loop {
        let read = file.read(&mut buffer).map_err(|err| err.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(crate::hash::to_hex(&hasher.finalize()))
}

/// Geçici helper dosyaları için çakışmayan zaman damgalı ad kökü üretir.
pub fn helper_file_stem(prefix: &str) -> String {
    format!(
        "{}-{}-{}",
        prefix,
        std::process::id(),
        Local::now().format("%Y%m%d%H%M%S%3f")
    )
}

/// JSON değeri pretty formatla belirtilen dosyaya yazar.
pub fn write_json_file(path: &Path, value: &Value) -> Result<(), String> {
    fs::write(
        path,
        serde_json::to_vec_pretty(value).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())
}

/// Helper kontrol dosyasına running/pause/stop durumunu yazar.
pub fn write_helper_control_state(path: &Path, state: &str) -> Result<(), String> {
    write_json_file(path, &json!({ "state": state }))
}

/// Helper sonucunu JSON olarak okur.
pub fn read_helper_json(path: &Path) -> Result<Value, String> {
    serde_json::from_slice(&fs::read(path).map_err(|err| err.to_string())?)
        .map_err(|err| err.to_string())
}

/// Helper hata dosyasından hata metnini okur.
pub fn read_helper_error(path: &Path) -> Option<String> {
    read_helper_json(path).ok().and_then(|value| {
        value
            .get("error")
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

/// Helper ilerleme dosyasından done/total/message değerlerini okur.
pub fn read_helper_progress(path: &Path) -> Option<(u64, u64, String)> {
    let value = read_helper_json(path).ok()?;
    let done = value
        .get("done")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let total = value
        .get("total")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    let message = value
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("Imaj alma sürüyor")
        .to_string();
    Some((done, total, message))
}

/// Geçici helper dosyalarını sessizce temizler.
pub fn cleanup_helper_files(paths: &[&Path]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}

#[cfg(unix)]
/// Yetkili helper çıktılarının sahibini düzeltmek için çağıran UID değerini döndürür.
pub fn helper_owner_uid() -> Option<u32> {
    Some(unsafe { libc::geteuid() })
}

#[cfg(not(unix))]
/// Unix dışı platformlarda helper UID bilgisi yoktur.
pub fn helper_owner_uid() -> Option<u32> {
    None
}

#[cfg(unix)]
/// Yetkili helper çıktılarının sahibini düzeltmek için çağıran GID değerini döndürür.
pub fn helper_owner_gid() -> Option<u32> {
    Some(unsafe { libc::getegid() })
}

#[cfg(not(unix))]
/// Unix dışı platformlarda helper GID bilgisi yoktur.
pub fn helper_owner_gid() -> Option<u32> {
    None
}

/// Disk listesi için yetkili helper çağırır ve sonucu parse eder.
pub fn elevated_disk_list() -> Result<Vec<crate::disk::DiskInfo>, String> {
    let output_path = std::env::temp_dir().join(format!(
        "worm-disk-list-{}-{}.json",
        std::process::id(),
        Local::now().format("%Y%m%d%H%M%S%3f")
    ));

    let run_result = run_elevated_disk_list_helper(&output_path);
    if let Err(err) = run_result {
        let _ = fs::remove_file(&output_path);
        return Err(err);
    }

    let content = fs::read_to_string(&output_path).map_err(|err| err.to_string())?;
    let _ = fs::remove_file(&output_path);
    let value: Value = serde_json::from_str(&content).map_err(|err| err.to_string())?;
    if value.get("ok").and_then(Value::as_bool) != Some(true) {
        return Err(value
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("elevated disk list failed")
            .to_string());
    }
    serde_json::from_value(
        value
            .get("disks")
            .cloned()
            .unwrap_or(Value::Array(Vec::new())),
    )
    .map_err(|err| err.to_string())
}

#[cfg(target_os = "linux")]
/// Linux disk listeleme helper komutunu çalıştırır.
fn run_elevated_disk_list_helper(output_path: &Path) -> Result<(), String> {
    run_elevated_helper_wait(&[
        "disk-list-helper".to_string(),
        output_path.to_string_lossy().into_owned(),
    ])
}

#[cfg(windows)]
/// Windows disk listeleme helper komutunu çalıştırır.
fn run_elevated_disk_list_helper(output_path: &Path) -> Result<(), String> {
    run_elevated_helper_wait(&[
        "disk-list-helper".to_string(),
        output_path.to_string_lossy().into_owned(),
    ])
}

#[cfg(not(any(target_os = "linux", windows)))]
/// Desteklenmeyen platformlarda yetkili disk listelemeyi hata olarak döndürür.
fn run_elevated_disk_list_helper(_output_path: &Path) -> Result<(), String> {
    Err("yetki yükseltmeli disk listeleme bu platformda desteklenmiyor".to_string())
}
