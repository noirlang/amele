use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolatilityProcess {
    pub pid: i64,
    pub ppid: i64,
    pub name: String,
    pub offset: String,
    pub extra_info: String,
}

pub fn locate_vol_py() -> Option<PathBuf> {
    let paths = [
        PathBuf::from("/home/raodrin/Belgeler/forensic/volatility3/vol.py"),
        PathBuf::from("../volatility3/vol.py"),
        PathBuf::from("./volatility3/vol.py"),
    ];

    for path in &paths {
        if path.exists() {
            return Some(path.clone());
        }
    }
    None
}

pub fn run_volatility_plugin(
    file_path: &Path,
    plugin: &str,
    extra_args: &[&str],
) -> Result<Value, String> {
    let Some(vol_py) = locate_vol_py() else {
        return Err("Volatility3 (vol.py) could not be located on the system.".to_string());
    };

    let vol_dir = vol_py.parent().unwrap_or(Path::new("."));

    // Build args: python3 vol.py -f <file_path> -r json <plugin> <extra_args>
    let mut args = vec![
        vol_py.to_string_lossy().into_owned(),
        "-f".to_string(),
        file_path.to_string_lossy().into_owned(),
        "-r".to_string(),
        "json".to_string(),
        plugin.to_string(),
    ];
    for arg in extra_args {
        args.push(arg.to_string());
    }

    let output = Command::new("python3")
        .args(&args)
        .current_dir(vol_dir)
        .output();

    match output {
        Ok(out) => {
            let stdout_str = String::from_utf8_lossy(&out.stdout);
            if out.status.success() {
                // Find where the JSON array starts. Sometimes there are Progress lines or headers
                let clean_json = if let Some(idx) = stdout_str.find('[') {
                    &stdout_str[idx..]
                } else {
                    &stdout_str
                };

                serde_json::from_str(clean_json)
                    .map_err(|e| format!("Failed to parse Volatility3 JSON output: {} (Raw: {})", e, stdout_str))
            } else {
                let stderr_str = String::from_utf8_lossy(&out.stderr);
                Err(format!(
                    "Volatility3 exited with error. Stderr: {}\nStdout: {}",
                    stderr_str.trim(),
                    stdout_str.trim()
                ))
            }
        }
        Err(e) => Err(format!("Failed to execute python3: {}", e)),
    }
}

pub fn get_processes(file_path: &Path, os_type: &str) -> Result<Vec<VolatilityProcess>, String> {
    let plugin = match os_type {
        "windows" => "windows.pslist",
        "linux" => "linux.pslist",
        _ => return Err(format!("Unsupported OS type for Volatility3: {}", os_type)),
    };

    let val = run_volatility_plugin(file_path, plugin, &[])?;
    let arr = val.as_array().ok_or("Volatility3 did not return a JSON array")?;

    let mut procs = Vec::new();
    for item in arr {
        if os_type == "windows" {
            let pid = item.get("PID").and_then(Value::as_i64).unwrap_or(0);
            let ppid = item.get("PPID").and_then(Value::as_i64).unwrap_or(0);
            let name = item.get("ImageFileName")
                .and_then(Value::as_str)
                .unwrap_or("Unknown")
                .to_string();
            
            let offset_val = item.get("Offset(V)");
            let offset = match offset_val {
                Some(Value::Number(n)) => format!("0x{:X}", n.as_u64().unwrap_or(0)),
                Some(Value::String(s)) => s.clone(),
                _ => {
                    if let Some(offset_p) = item.get("Offset(P)") {
                        match offset_p {
                            Value::Number(n) => format!("0x{:X} (P)", n.as_u64().unwrap_or(0)),
                            Value::String(s) => format!("{} (P)", s),
                            _ => "N/A".to_string(),
                        }
                    } else {
                        "N/A".to_string()
                    }
                }
            };

            let threads = item.get("Threads").and_then(Value::as_i64).unwrap_or(0);
            let handles = item.get("Handles").and_then(Value::as_i64).unwrap_or(0);
            let session = item.get("SessionId").and_then(Value::as_i64).unwrap_or(-1);
            let create_time = item.get("CreateTime").and_then(Value::as_str).unwrap_or("-");

            let extra_info = format!(
                "Threads: {} · Handles: {} · Session: {} · Created: {}",
                threads, handles, session, create_time
            );

            procs.push(VolatilityProcess {
                pid,
                ppid,
                name,
                offset,
                extra_info,
            });
        } else {
            let pid = item.get("PID").and_then(Value::as_i64).unwrap_or(0);
            let ppid = item.get("PPID").and_then(Value::as_i64).unwrap_or(0);
            let name = item.get("COMM")
                .and_then(Value::as_str)
                .unwrap_or("Unknown")
                .to_string();
            
            let offset_val = item.get("OFFSET (V)");
            let offset = match offset_val {
                Some(Value::Number(n)) => format!("0x{:X}", n.as_u64().unwrap_or(0)),
                Some(Value::String(s)) => s.clone(),
                _ => "N/A".to_string(),
            };

            let uid = item.get("UID").and_then(Value::as_i64).unwrap_or(-1);
            let gid = item.get("GID").and_then(Value::as_i64).unwrap_or(-1);
            let create_time = item.get("CREATION TIME").and_then(Value::as_str).unwrap_or("-");

            let extra_info = format!(
                "UID: {} · GID: {} · Created: {}",
                uid, gid, create_time
            );

            procs.push(VolatilityProcess {
                pid,
                ppid,
                name,
                offset,
                extra_info,
            });
        }
    }

    Ok(procs)
}

pub fn get_process_details(file_path: &Path, os_type: &str, pid: i64) -> Result<String, String> {
    let pid_str = pid.to_string();
    let (plugin, extra_args) = match os_type {
        "windows" => ("windows.dlllist", vec!["--pid", &pid_str]),
        "linux" => ("linux.lsof", vec!["--pid", &pid_str]),
        _ => return Err(format!("Unsupported OS type for Volatility3 details: {}", os_type)),
    };

    let val = run_volatility_plugin(file_path, plugin, &extra_args)?;
    
    // Format the json array as a readable text block
    let arr = val.as_array().ok_or("Volatility3 details did not return a JSON array")?;
    
    let mut details = String::new();
    if os_type == "windows" {
        details.push_str("LOADED DLL LIST:\n%-12s %-18s %-10s %s\n");
        details = format!("LOADED DLL LIST:\n{:<10} {:<18} {:<10} {}\n", "Base", "Size", "LoadCount", "Path");
        details.push_str("--------------------------------------------------------------------------------\n");
        for item in arr {
            let base = item.get("Base").and_then(Value::as_i64).unwrap_or(0);
            let size = item.get("Size").and_then(Value::as_i64).unwrap_or(0);
            let load_count = item.get("LoadCount").and_then(Value::as_i64).unwrap_or(0);
            let path = item.get("Path").and_then(Value::as_str).unwrap_or("-");
            
            details.push_str(&format!(
                "0x{:<8X} {:<18} {:<10} {}\n",
                base, size, load_count, path
            ));
        }
    } else {
        details = format!("{:<10} {:<10} {:<10} {:<10} {}\n", "PID", "FD", "Type", "Offset", "Path");
        details.push_str("--------------------------------------------------------------------------------\n");
        for item in arr {
            let fd = item.get("FD").and_then(|v| match v {
                Value::Number(n) => Some(n.to_string()),
                Value::String(s) => Some(s.clone()),
                _ => None,
            }).unwrap_or("-".to_string());
            let fd_type = item.get("Type").and_then(Value::as_str).unwrap_or("-");
            let offset_val = item.get("Offset");
            let offset = match offset_val {
                Some(Value::Number(n)) => format!("0x{:X}", n.as_u64().unwrap_or(0)),
                Some(Value::String(s)) => s.clone(),
                _ => "-".to_string(),
            };
            let path = item.get("Path").and_then(Value::as_str).unwrap_or("-");
            
            details.push_str(&format!(
                "{:<10} {:<10} {:<10} {:<10} {}\n",
                pid, fd, fd_type, offset, path
            ));
        }
    }
    
    Ok(details)
}
