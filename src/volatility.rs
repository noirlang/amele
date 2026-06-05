use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityProcess {
    pub pid: i64,
    pub ppid: i64,
    pub name: String,
    pub offset: String,
    pub extra_info: String,
}

pub fn locate_vol_py() -> Option<PathBuf> {
    let mut paths = Vec::new();

    if let Ok(value) = env::var("WORM_VOLATILITY3_PATH") {
        push_volatility_candidate(&mut paths, PathBuf::from(value));
    }
    if let Ok(value) = env::var("VOLATILITY3_PATH") {
        push_volatility_candidate(&mut paths, PathBuf::from(value));
    }

    if let Ok(cwd) = env::current_dir() {
        push_volatility_candidate(&mut paths, cwd.join("volatility3"));
        push_volatility_candidate(&mut paths, cwd.join("../volatility3"));
        push_volatility_candidate(&mut paths, cwd.join("../../volatility3"));
    }

    if let Ok(exe) = env::current_exe()
        && let Some(dir) = exe.parent()
    {
        push_volatility_candidate(&mut paths, dir.join("volatility3"));
        push_volatility_candidate(&mut paths, dir.join("../volatility3"));
        push_volatility_candidate(&mut paths, dir.join("../../volatility3"));
    }

    push_volatility_candidate(
        &mut paths,
        PathBuf::from("/home/raodrin/Belgeler/forensic/volatility3"),
    );

    paths
        .into_iter()
        .find(|path| path.exists())
        .and_then(|path| path.canonicalize().ok().or(Some(path)))
}

fn push_volatility_candidate(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if path.file_name().is_some_and(|name| name == "vol.py") {
        paths.push(path);
    } else {
        paths.push(path.join("vol.py"));
    }
}

pub fn run_volatility_plugin(
    file_path: &Path,
    plugin: &str,
    extra_args: &[&str],
) -> Result<Value, String> {
    run_volatility_plugin_logged(file_path, plugin, extra_args, None)
}

pub fn run_volatility_plugin_logged(
    file_path: &Path,
    plugin: &str,
    extra_args: &[&str],
    log: Option<Arc<dyn Fn(String) + Send + Sync>>,
) -> Result<Value, String> {
    let Some(vol_py) = locate_vol_py() else {
        return Err(
            "Volatility3 vol.py bulunamadı. WORM_VOLATILITY3_PATH ile vol.py veya volatility3 klasörünü belirtin."
                .to_string(),
        );
    };

    let vol_dir = vol_py.parent().unwrap_or(Path::new("."));
    let mut args = vec![
        vol_py.to_string_lossy().into_owned(),
        "-q".to_string(),
        "-f".to_string(),
        file_path.to_string_lossy().into_owned(),
        "-r".to_string(),
        "json".to_string(),
        plugin.to_string(),
    ];
    args.extend(extra_args.iter().map(|arg| arg.to_string()));

    if let Some(log) = &log {
        log(format!("Volatility3 çalışıyor: {plugin}"));
        log(format!("vol.py: {}", vol_py.display()));
        log(format!("RAM imajı: {}", file_path.display()));
    }

    let mut child = Command::new("python3")
        .args(&args)
        .current_dir(vol_dir)
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("python3 çalıştırılamadı: {err}"))?;

    let stdout_buffer = Arc::new(Mutex::new(String::new()));
    let stderr_buffer = Arc::new(Mutex::new(String::new()));
    let stdout_thread = child
        .stdout
        .take()
        .map(|stdout| spawn_reader(stdout, "stdout", stdout_buffer.clone(), log.clone(), false));
    let stderr_thread = child
        .stderr
        .take()
        .map(|stderr| spawn_reader(stderr, "stderr", stderr_buffer.clone(), log.clone(), true));

    let status = child
        .wait()
        .map_err(|err| format!("Volatility3 süreci beklenemedi: {err}"))?;
    if let Some(handle) = stdout_thread {
        let _ = handle.join();
    }
    if let Some(handle) = stderr_thread {
        let _ = handle.join();
    }

    let stdout = stdout_buffer
        .lock()
        .map(|value| value.clone())
        .unwrap_or_default();
    let stderr = stderr_buffer
        .lock()
        .map(|value| value.clone())
        .unwrap_or_default();

    if !status.success() {
        return Err(format_volatility_error(plugin, &stdout, &stderr));
    }

    let clean_json = trim_to_json(&stdout).ok_or_else(|| {
        format!(
            "Volatility3 JSON çıktısı bulunamadı. Plugin: {plugin}. Çıktı: {}",
            stdout.trim()
        )
    })?;

    let parsed = serde_json::from_str(clean_json).map_err(|err| {
        format!(
            "Volatility3 JSON çıktısı parse edilemedi: {err}. Plugin: {plugin}. Ham çıktı: {}",
            stdout.trim()
        )
    })?;
    if let Some(log) = &log {
        log(format!("Volatility3 tamamlandı: {plugin}"));
    }
    Ok(parsed)
}

fn spawn_reader<R>(
    reader: R,
    stream: &'static str,
    buffer: Arc<Mutex<String>>,
    log: Option<Arc<dyn Fn(String) + Send + Sync>>,
    log_all: bool,
) -> thread::JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            if let Ok(mut output) = buffer.lock() {
                output.push_str(&line);
                output.push('\n');
            }
            let clean = line.trim();
            if clean.is_empty() {
                continue;
            }
            let is_json_payload = clean.starts_with('[') || clean.starts_with('{');
            if (log_all || !is_json_payload)
                && let Some(log) = &log
            {
                log(format!("[volatility:{stream}] {clean}"));
            }
        }
    })
}

fn trim_to_json(output: &str) -> Option<&str> {
    let array_idx = output.find('[');
    let object_idx = output.find('{');
    let idx = match (array_idx, object_idx) {
        (Some(a), Some(o)) => a.min(o),
        (Some(a), None) => a,
        (None, Some(o)) => o,
        (None, None) => return None,
    };
    Some(output[idx..].trim())
}

fn format_volatility_error(plugin: &str, stdout: &str, stderr: &str) -> String {
    let mut message = format!(
        "Volatility3 hata verdi. Plugin: {plugin}\nStderr: {}\nStdout: {}",
        stderr.trim(),
        stdout.trim()
    );

    let lower = format!("{stdout}\n{stderr}").to_ascii_lowercase();
    if lower.contains("invalid choice") {
        message.push_str("\n\nSeçilen Volatility3 kurulumunda bu plugin bulunmuyor. volatility3 klasörünü güncelleyin veya WORM_VOLATILITY3_PATH ile doğru vol.py yolunu verin.");
    }
    if lower.contains("unsatisfied requirement")
        || lower.contains("layer_name")
        || lower.contains("symbol_table_name")
        || lower.contains("automagic")
    {
        message.push_str(
            "\n\nOlası nedenler:\n\
             1. RAM imajı seçilen işletim sistemi profiliyle uyumlu değil.\n\
             2. Volatility3 sembol tablosu bulunamadı veya indirilemedi.\n\
             3. Dosya fiziksel RAM imajı değil ya da imaj eksik/bozuk.\n\
             4. Linux imajları için kernel banner/sembol eşleşmesi gerekebilir.",
        );
    }
    message
}

pub fn get_processes(file_path: &Path, os_type: &str) -> Result<Vec<VolatilityProcess>, String> {
    get_processes_logged(file_path, os_type, None)
}

pub fn get_processes_logged(
    file_path: &Path,
    os_type: &str,
    log: Option<Arc<dyn Fn(String) + Send + Sync>>,
) -> Result<Vec<VolatilityProcess>, String> {
    let plugin = match os_type {
        "windows" => "windows.pslist.PsList",
        "linux" => "linux.pslist.PsList",
        _ => return Err(format!("Desteklenmeyen RAM işletim sistemi: {os_type}")),
    };

    let value = run_volatility_plugin_logged(file_path, plugin, &[], log)?;
    let rows = json_rows(&value)?;

    Ok(rows
        .iter()
        .map(|item| {
            if os_type == "linux" {
                linux_process(item)
            } else {
                windows_process(item)
            }
        })
        .filter(|process| process.pid > 0 || process.name != "Unknown")
        .collect())
}

fn windows_process(item: &Value) -> VolatilityProcess {
    let pid = value_i64(item, &["PID"]).unwrap_or(0);
    let ppid = value_i64(item, &["PPID"]).unwrap_or(0);
    let name = value_string(item, &["ImageFileName", "Image File Name", "Process"])
        .unwrap_or_else(|| "Unknown".to_string());
    let offset = value_string(
        item,
        &["Offset(V)", "Offset (V)", "Offset(P)", "Offset (P)"],
    )
    .unwrap_or_else(|| "N/A".to_string());
    let threads = value_string(item, &["Threads"]).unwrap_or_else(|| "0".to_string());
    let handles = value_string(item, &["Handles"]).unwrap_or_else(|| "0".to_string());
    let session = value_string(item, &["SessionId"]).unwrap_or_else(|| "-".to_string());
    let create_time = value_string(item, &["CreateTime"]).unwrap_or_else(|| "-".to_string());

    VolatilityProcess {
        pid,
        ppid,
        name,
        offset,
        extra_info: format!(
            "Threads: {threads} · Handles: {handles} · Session: {session} · Created: {create_time}"
        ),
    }
}

fn linux_process(item: &Value) -> VolatilityProcess {
    let pid = value_i64(item, &["PID"]).unwrap_or(0);
    let ppid = value_i64(item, &["PPID"]).unwrap_or(0);
    let name = value_string(item, &["COMM", "Command", "Process"])
        .unwrap_or_else(|| "Unknown".to_string());
    let offset = value_string(item, &["OFFSET (V)", "Offset(V)", "Offset (V)"])
        .unwrap_or_else(|| "N/A".to_string());
    let uid = value_string(item, &["UID"]).unwrap_or_else(|| "-".to_string());
    let gid = value_string(item, &["GID"]).unwrap_or_else(|| "-".to_string());
    let euid = value_string(item, &["EUID"]).unwrap_or_else(|| "-".to_string());
    let egid = value_string(item, &["EGID"]).unwrap_or_else(|| "-".to_string());
    let create_time =
        value_string(item, &["CREATION TIME", "CreateTime"]).unwrap_or_else(|| "-".to_string());

    VolatilityProcess {
        pid,
        ppid,
        name,
        offset,
        extra_info: format!(
            "UID/GID: {uid}/{gid} · EUID/EGID: {euid}/{egid} · Created: {create_time}"
        ),
    }
}

pub fn get_process_details(file_path: &Path, os_type: &str, pid: i64) -> Result<String, String> {
    get_process_details_logged(file_path, os_type, pid, None)
}

pub fn get_process_details_logged(
    file_path: &Path,
    os_type: &str,
    pid: i64,
    log: Option<Arc<dyn Fn(String) + Send + Sync>>,
) -> Result<String, String> {
    let pid_str = pid.to_string();
    let (plugin, extra_args) = match os_type {
        "windows" => ("windows.dlllist.DllList", vec!["--pid", pid_str.as_str()]),
        "linux" => ("linux.lsof.Lsof", vec!["--pid", pid_str.as_str()]),
        _ => return Err(format!("Desteklenmeyen RAM işletim sistemi: {os_type}")),
    };

    let value = run_volatility_plugin_logged(file_path, plugin, &extra_args, log)?;
    let rows = json_rows(&value)?;

    if os_type == "linux" {
        Ok(format_linux_open_files(&rows))
    } else {
        Ok(format_windows_dlls(&rows))
    }
}

fn format_windows_dlls(rows: &[Value]) -> String {
    let mut details = format!(
        "LOADED DLL LIST\n{:<8} {:<22} {:<14} {:<10} {}\n",
        "PID", "Base", "Size", "LoadCount", "Path"
    );
    details.push_str(
        "--------------------------------------------------------------------------------\n",
    );

    for item in rows {
        let pid = value_string(item, &["PID"]).unwrap_or_else(|| "-".to_string());
        let base = value_string(item, &["Base"]).unwrap_or_else(|| "-".to_string());
        let size = value_string(item, &["Size"]).unwrap_or_else(|| "-".to_string());
        let load_count = value_string(item, &["LoadCount"]).unwrap_or_else(|| "-".to_string());
        let path = value_string(item, &["Path", "Name"]).unwrap_or_else(|| "-".to_string());

        details.push_str(&format!(
            "{:<8} {:<22} {:<14} {:<10} {}\n",
            pid, base, size, load_count, path
        ));
    }

    if rows.is_empty() {
        details.push_str("No DLL rows returned.\n");
    }
    details
}

fn format_linux_open_files(rows: &[Value]) -> String {
    let mut details = format!(
        "OPEN FILES\n{:<8} {:<8} {:<20} {:<8} {:<10} {}\n",
        "PID", "FD", "Process", "Type", "Size", "Path"
    );
    details.push_str(
        "--------------------------------------------------------------------------------\n",
    );

    for item in rows {
        let pid = value_string(item, &["PID"]).unwrap_or_else(|| "-".to_string());
        let fd = value_string(item, &["FD"]).unwrap_or_else(|| "-".to_string());
        let process = value_string(item, &["Process", "COMM"]).unwrap_or_else(|| "-".to_string());
        let inode_type = value_string(item, &["Type"]).unwrap_or_else(|| "-".to_string());
        let size = value_string(item, &["Size"]).unwrap_or_else(|| "-".to_string());
        let path = value_string(item, &["Path"]).unwrap_or_else(|| "-".to_string());

        details.push_str(&format!(
            "{:<8} {:<8} {:<20} {:<8} {:<10} {}\n",
            pid, fd, process, inode_type, size, path
        ));
    }

    if rows.is_empty() {
        details.push_str("No open file rows returned.\n");
    }
    details
}

fn json_rows(value: &Value) -> Result<Vec<Value>, String> {
    if let Some(arr) = value.as_array() {
        return Ok(arr.clone());
    }
    for key in ["rows", "data", "tree"] {
        if let Some(arr) = value.get(key).and_then(Value::as_array) {
            return Ok(arr.clone());
        }
    }
    Err("Volatility3 JSON çıktısı satır listesi içermiyor.".to_string())
}

fn value_i64(item: &Value, keys: &[&str]) -> Option<i64> {
    keys.iter().find_map(|key| {
        let value = item.get(*key)?;
        match value {
            Value::Number(number) => number
                .as_i64()
                .or_else(|| number.as_u64().map(|n| n as i64)),
            Value::String(text) => text.trim().parse::<i64>().ok(),
            _ => None,
        }
    })
}

fn value_string(item: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        let value = item.get(*key)?;
        match value {
            Value::String(text) => Some(text.clone()),
            Value::Number(number) => number
                .as_u64()
                .map(|n| {
                    if key.to_ascii_lowercase().contains("offset")
                        || matches!(*key, "Base" | "Size")
                    {
                        format!("0x{n:X}")
                    } else {
                        n.to_string()
                    }
                })
                .or_else(|| number.as_i64().map(|n| n.to_string())),
            Value::Bool(value) => Some(value.to_string()),
            Value::Null => None,
            other => Some(other.to_string()),
        }
    })
}
