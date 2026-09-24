//! Yapay zeka agent keşfi, model listeleme ve Amele SKILL.md kural motoru entegrasyonu.

use crate::server::{Response, json_error, json_ok};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentModel {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscoveredAgent {
    pub id: String,
    pub name: String,
    pub installed: bool,
    pub binary_path: Option<String>,
    pub description: String,
    pub models: Vec<AgentModel>,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub prompt: String,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub case_name: Option<String>,
    pub target_scope: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteCommandRequest {
    pub command: String,
    pub sudo_password: Option<String>,
    pub windows_confirmed: Option<bool>,
    pub linux_confirmed: Option<bool>,
}

/// Sistemde kurulu olan yapay zeka CLI araçlarını tespit eder.
fn check_binary(binary_name: &str) -> Option<String> {
    if let Ok(output) = Command::new("which").arg(binary_name).output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
    }

    if let Ok(home) = std::env::var("HOME") {
        let fallbacks = [
            format!("{home}/.local/bin/{binary_name}"),
            format!("{home}/.local/share/mise/shims/{binary_name}"),
            format!("{home}/.cargo/bin/{binary_name}"),
            format!("/usr/bin/{binary_name}"),
            format!("/usr/local/bin/{binary_name}"),
            format!("{home}/.local/share/mise/installs/{binary_name}/latest/{binary_name}"),
            format!("{home}/.local/share/mise/installs/{binary_name}/latest/bin/{binary_name}"),
        ];
        for fb in fallbacks {
            if std::path::Path::new(&fb).exists() {
                return Some(fb);
            }
        }

        // Mise versiyonlu dizinlerini tara (~/.local/share/mise/installs/<binary>/<versiyon>/...)
        let mise_dir = format!("{home}/.local/share/mise/installs/{binary_name}");
        if let Ok(entries) = std::fs::read_dir(mise_dir) {
            for entry in entries.flatten() {
                let dir_path = entry.path();
                if dir_path.is_dir() {
                    let candidates = [
                        dir_path.join(binary_name),
                        dir_path.join("bin").join(binary_name),
                        dir_path.join(binary_name).join(binary_name),
                    ];
                    for c in candidates {
                        if c.exists() {
                            return Some(c.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// CLI komutunu timeout ile çalıştırır, takılırsa öldürüp None döner.
// ajan cli takılıp backendi kilitlemesin diye eklendi.
fn run_cli_with_timeout(
    program: &str,
    args: &[&str],
    timeout_secs: u64,
) -> Option<std::process::Output> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let timeout = std::time::Duration::from_secs(timeout_secs);
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return child.wait_with_output().ok(),
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return None;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(_) => return None,
        }
    }
}

fn fetch_agy_models(binary_path: &str) -> Vec<AgentModel> {
    if let Some(output) = run_cli_with_timeout(binary_path, &["models"], 10) {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut models = Vec::new();
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with("Fetching") || trimmed.contains("...")
                {
                    continue;
                }
                let mut parts = trimmed.split_whitespace();
                if let Some(id) = parts.next() {
                    let name = parts.collect::<Vec<_>>().join(" ");
                    let display_name = if name.is_empty() {
                        id.to_string()
                    } else {
                        name
                    };
                    models.push(AgentModel {
                        id: id.to_string(),
                        name: display_name.clone(),
                        description: format!("AGY {display_name} modeli"),
                    });
                }
            }
            if !models.is_empty() {
                return models;
            }
        }
    }
    vec![]
}

fn fetch_opencode_models(binary_path: &str) -> Vec<AgentModel> {
    if let Some(output) = run_cli_with_timeout(binary_path, &["models"], 10) {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let mut models = Vec::new();
            for line in stdout.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                models.push(AgentModel {
                    id: trimmed.to_string(),
                    name: trimmed.to_string(),
                    description: format!("OpenCode {trimmed} modeli"),
                });
            }
            if !models.is_empty() {
                return models;
            }
        }
    }
    vec![]
}

fn fetch_codex_models(binary_path: &str) -> Vec<AgentModel> {
    // OpenAI / Codex oturum durumunu kontrol et
    if let Some(output) = run_cli_with_timeout(binary_path, &["login", "status"], 10) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout} {stderr}");
        // Giriş yapılmamışsa sahte model listeleme
        if combined.contains("Not logged in") || !output.status.success() {
            return vec![];
        }
    } else {
        return vec![];
    }

    vec![
        AgentModel {
            id: "gpt-4o".into(),
            name: "GPT-4o".into(),
            description: "OpenAI GPT-4o".into(),
        },
        AgentModel {
            id: "o3-mini".into(),
            name: "o3-mini".into(),
            description: "OpenAI o3-mini".into(),
        },
    ]
}

fn fetch_pi_models(binary_path: &str) -> Vec<AgentModel> {
    // Pi sağlayıcı auth durumunu sorgula
    if let Some(output) = run_cli_with_timeout(binary_path, &["auth", "check"], 10) {
        let combined = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if combined.contains("not_ready") || !output.status.success() {
            return vec![];
        }
    } else {
        return vec![];
    }
    vec![]
}

fn fetch_claude_models(binary_path: &str) -> Vec<AgentModel> {
    if let Some(output) = run_cli_with_timeout(binary_path, &["auth", "status"], 10) {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("\"loggedIn\":true") {
            return vec![
                AgentModel {
                    id: "claude-3-7-sonnet".into(),
                    name: "Claude 3.7 Sonnet".into(),
                    description: "Anthropic Claude 3.7 Sonnet".into(),
                },
                AgentModel {
                    id: "claude-3-5-sonnet".into(),
                    name: "Claude 3.5 Sonnet".into(),
                    description: "Anthropic Claude 3.5 Sonnet".into(),
                },
                AgentModel {
                    id: "claude-3-5-haiku".into(),
                    name: "Claude 3.5 Haiku".into(),
                    description: "Anthropic Claude 3.5 Haiku".into(),
                },
            ];
        }
    }
    vec![]
}

pub fn fetch_live_models_for_agent(agent_id: &str) -> Vec<AgentModel> {
    match agent_id {
        "agy" => {
            let bin = check_binary("agy").unwrap_or_else(|| "agy".to_string());
            fetch_agy_models(&bin)
        }
        "opencode" => {
            let bin = check_binary("opencode").unwrap_or_else(|| "opencode".to_string());
            fetch_opencode_models(&bin)
        }
        "claude" => {
            let bin = check_binary("claude").unwrap_or_else(|| "claude".to_string());
            fetch_claude_models(&bin)
        }
        "codex" => {
            let bin = check_binary("codex").unwrap_or_else(|| "codex".to_string());
            fetch_codex_models(&bin)
        }
        "pi" => {
            let bin = check_binary("pi").unwrap_or_else(|| "pi".to_string());
            fetch_pi_models(&bin)
        }
        _ => vec![],
    }
}

/// Tüm desteklenen gerçek CLI agent'larını anında listeler.
pub fn get_agents_endpoint() -> Response {
    let pi_path = check_binary("pi");
    let agy_path = check_binary("agy");
    let claude_path = check_binary("claude");
    let codex_path = check_binary("codex");
    let opencode_path = check_binary("opencode");

    let agents = vec![
        DiscoveredAgent {
            id: "agy".to_string(),
            name: "Antigravity (AGY)".to_string(),
            installed: agy_path.is_some(),
            binary_path: agy_path,
            description: "Google Antigravity Agentic Coding CLI".to_string(),
            models: fetch_live_models_for_agent("agy"),
        },
        DiscoveredAgent {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            installed: claude_path.is_some(),
            binary_path: claude_path,
            description: "Anthropic Claude Code CLI Asistanı".to_string(),
            models: fetch_live_models_for_agent("claude"),
        },
        DiscoveredAgent {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            installed: codex_path.is_some(),
            binary_path: codex_path,
            description: "OpenAI Codex CLI Ajanı".to_string(),
            models: fetch_live_models_for_agent("codex"),
        },
        DiscoveredAgent {
            id: "pi".to_string(),
            name: "Pi".to_string(),
            installed: pi_path.is_some(),
            binary_path: pi_path,
            description: "Earendil Works çok sağlayıcılı terminal ajanı".to_string(),
            models: fetch_live_models_for_agent("pi"),
        },
        DiscoveredAgent {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            installed: opencode_path.is_some(),
            binary_path: opencode_path,
            description: "Açık kaynak çoklu sağlayıcı CLI ajanı".to_string(),
            models: fetch_live_models_for_agent("opencode"),
        },
    ];

    json_ok(json!({
        "ok": true,
        "agents": agents
    }))
}

/// Seçili agent için canlı modelleri CLI'dan sorgulayan endpoint
pub fn get_models_endpoint(path: &str) -> Response {
    let agent_id = if let Some(query) = path.split('?').nth(1) {
        query
            .split('&')
            .find_map(|pair| {
                let mut parts = pair.split('=');
                if parts.next() == Some("agent") {
                    parts.next()
                } else {
                    None
                }
            })
            .unwrap_or("agy")
    } else {
        "agy"
    };

    let models = fetch_live_models_for_agent(agent_id);
    json_ok(json!({
        "ok": true,
        "agent": agent_id,
        "models": models
    }))
}

/// Amele SKILL.md içeriğini yükler.
fn load_amele_skill_text() -> &'static str {
    include_str!("../../SKILL.md")
}

/// Agent ile sohbet sorgusunu çalıştırır.
pub fn chat_endpoint(body: &[u8]) -> Response {
    let req: ChatRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(e) => return json_error(400, format!("Geçersiz JSON isteği: {e}")),
    };

    let prompt = req.prompt.trim();
    if prompt.is_empty() {
        return json_error(400, "Sorgu metni boş olamaz.");
    }

    let agent_id = req.agent.as_deref().unwrap_or("agy");
    let model_id = req.model.as_deref().unwrap_or("");
    let case_name = req.case_name.as_deref().unwrap_or("varsayilan_vaka");
    let target_scope = req.target_scope.as_deref().unwrap_or("all");

    // cli çalıştır, boş/takılma durumunda kural motoruna düş
    // timeout 60sn, takılırsa öldürüp fallback dönüyoruz
    fn run_or_fallback(
        agent_label: &str,
        bin: &str,
        args: &[&str],
        prompt: &str,
        case_name: &str,
        model_id: &str,
    ) -> Response {
        if let Some(output) = run_cli_with_timeout(bin, args, 60) {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if output.status.success() && !text.is_empty() {
                let suggested_command = extract_suggested_command(&text);
                return json_ok(json!({
                    "ok": true,
                    "agent": agent_label,
                    "model": model_id,
                    "response": text,
                    "suggested_command": suggested_command
                }));
            }
        }
        let fallback = run_amele_rule_engine(prompt, case_name);
        let suggested_command = extract_suggested_command(&fallback);
        json_ok(json!({
            "ok": true,
            "agent": agent_label,
            "model": model_id,
            "response": fallback,
            "suggested_command": suggested_command
        }))
    }

    match agent_id {
        "pi" => {
            let bin = check_binary("pi").unwrap_or_else(|| "pi".to_string());
            let combined = format!(
                "Amele Adli Bilişim Kuralları:\n{}\nAktif Vaka: {}\nKapsam: {}\nKullanıcı Sorusu: {}",
                load_amele_skill_text(),
                case_name,
                target_scope,
                prompt
            );
            if model_id.is_empty() {
                let args = ["-p", combined.as_str()];
                run_or_fallback("pi", &bin, &args, prompt, case_name, model_id)
            } else {
                let args = ["-p", combined.as_str(), "--model", model_id];
                run_or_fallback("pi", &bin, &args, prompt, case_name, model_id)
            }
        }
        "agy" => {
            let bin = check_binary("agy").unwrap_or_else(|| "agy".to_string());
            let combined = format!(
                "You are the Amele Digital Forensics Agent. Amele CLI Skill and Rules:\n{}\n\nActive Case: {}\nScope: {}\n\nUser Question: {}",
                load_amele_skill_text(),
                case_name,
                target_scope,
                prompt
            );
            if model_id.is_empty() {
                let args = ["--print", "--print-timeout", "50s", combined.as_str()];
                run_or_fallback("agy", &bin, &args, prompt, case_name, model_id)
            } else {
                let args = [
                    "--print",
                    "--print-timeout",
                    "50s",
                    combined.as_str(),
                    "--model",
                    model_id,
                ];
                run_or_fallback("agy", &bin, &args, prompt, case_name, model_id)
            }
        }
        "claude" => {
            let bin = check_binary("claude").unwrap_or_else(|| "claude".to_string());
            let combined = format!(
                "Amele Adli Bilişim Kuralları:\n{}\nAktif Vaka: {}\nKapsam: {}\nKullanıcı Sorusu: {}",
                load_amele_skill_text(),
                case_name,
                target_scope,
                prompt
            );
            if model_id.is_empty() {
                let args = ["-p", combined.as_str()];
                run_or_fallback("claude", &bin, &args, prompt, case_name, model_id)
            } else {
                let args = ["-p", combined.as_str(), "--model", model_id];
                run_or_fallback("claude", &bin, &args, prompt, case_name, model_id)
            }
        }
        "codex" => {
            let bin = check_binary("codex").unwrap_or_else(|| "codex".to_string());
            let combined = format!(
                "Amele Adli Bilişim Kuralları:\n{}\nAktif Vaka: {}\nKapsam: {}\nKullanıcı Sorusu: {}",
                load_amele_skill_text(),
                case_name,
                target_scope,
                prompt
            );
            if model_id.is_empty() {
                let args = ["exec", combined.as_str()];
                run_or_fallback("codex", &bin, &args, prompt, case_name, model_id)
            } else {
                let args = ["exec", combined.as_str(), "-m", model_id];
                run_or_fallback("codex", &bin, &args, prompt, case_name, model_id)
            }
        }
        "opencode" => {
            let bin = check_binary("opencode").unwrap_or_else(|| "opencode".to_string());
            let combined = format!(
                "Amele Adli Bilişim Kuralları:\n{}\nAktif Vaka: {}\nKapsam: {}\nKullanıcı Sorusu: {}",
                load_amele_skill_text(),
                case_name,
                target_scope,
                prompt
            );
            if model_id.is_empty() {
                let args = ["run", combined.as_str()];
                run_or_fallback("opencode", &bin, &args, prompt, case_name, model_id)
            } else {
                let args = ["run", combined.as_str(), "-m", model_id];
                run_or_fallback("opencode", &bin, &args, prompt, case_name, model_id)
            }
        }
        _ => {
            let response_text = run_amele_rule_engine(prompt, case_name);
            let suggested_command = extract_suggested_command(&response_text);
            json_ok(json!({
                "ok": true,
                "agent": agent_id,
                "model": model_id,
                "response": response_text,
                "suggested_command": suggested_command
            }))
        }
    }
}

/// Yanıt içerisinden çalıştırılabilir ilk Amele CLI komutunu ayıklar.
fn extract_suggested_command(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("amele ") || trimmed.starts_with("sudo amele ") {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Kullanıcının onayladığı CLI komutunu çalıştırır.
/// Linux'ta onay sonrası once sudo yetkisi denenir, yoksa pkexec sistem
/// penceresi acilir. Windows'ta ise yönetici onayıyla yürütür.
pub fn execute_command_endpoint(body: &[u8]) -> Response {
    let req: ExecuteCommandRequest = match serde_json::from_slice(body) {
        Ok(r) => r,
        Err(e) => return json_error(400, format!("Geçersiz JSON verisi: {e}")),
    };

    let cmd_str = req.command.trim();
    if cmd_str.is_empty() {
        return json_error(400, "Komut boş olamaz.");
    }

    // Güvenlik kontrolü: Yalnızca amele ile başlayan komutlara izin ver
    let is_amele = cmd_str.starts_with("amele") || cmd_str.starts_with("sudo amele");
    if !is_amele {
        return json_error(403, "Yalnızca resmi Amele CLI komutları çalıştırılabilir.");
    }

    let is_root_required = cmd_str.contains("sudo")
        || cmd_str.contains("disk")
        || cmd_str.contains("ram")
        || cmd_str.contains("/dev/")
        || cmd_str.contains("PhysicalDrive")
        || cmd_str.contains("avml")
        || cmd_str.contains("winpmem");

    #[cfg(unix)]
    {
        let is_root = unsafe { libc::geteuid() == 0 };

        // sonuc jsonu kuran kisa yardimci
        fn cmd_result(output: std::process::Output) -> Response {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let success = output.status.success();
            json_ok(json!({
                "ok": success,
                "stdout": stdout,
                "stderr": stderr,
                "exit_code": output.status.code().unwrap_or(-1)
            }))
        }

        if is_root_required && !is_root {
            let clean_cmd = if cmd_str.starts_with("sudo ") {
                cmd_str.strip_prefix("sudo ").unwrap()
            } else {
                cmd_str
            };

            // eski arayuzden parola geldiyse uyumluluk icin sudo -S ile calistir
            if let Some(password) = req.sudo_password.as_deref().filter(|p| !p.is_empty()) {
                let mut child = match Command::new("sudo")
                    .arg("-S")
                    .arg("sh")
                    .arg("-c")
                    .arg(clean_cmd)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(c) => c,
                    Err(err) => return json_error(500, format!("Süreç başlatılamadı: {err}")),
                };

                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(format!("{password}\n").as_bytes());
                }

                return match child.wait_with_output() {
                    Ok(output) => cmd_result(output),
                    Err(err) => json_error(500, format!("Komut tamamlanamadı: {err}")),
                };
            }

            // onay yoksa windowstaki gibi evet/hayir penceresini ac
            if req.linux_confirmed != Some(true) {
                return json_ok(json!({
                    "ok": false,
                    "needs_elevation": true,
                    "os": "linux",
                    "reason": "Bu adli edinim komutu blok aygıtlara veya belleğe erişim için root (sudo) yetkisi gerektirir."
                }));
            }

            // onay var: once sudo yetkisi onbellekte mi diye sormadan dene
            if let Ok(output) = Command::new("sudo")
                .arg("-n")
                .arg("sh")
                .arg("-c")
                .arg(clean_cmd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
            {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let needs_pass = stderr.contains("a password is required")
                    || stderr.contains("no tty present")
                    || stderr.contains("a terminal is required");
                if !needs_pass {
                    return cmd_result(output);
                }
            }

            // onbellek yoksa sistem penceresini ac (polkit)
            match Command::new("pkexec")
                .arg("sh")
                .arg("-c")
                .arg(clean_cmd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
            {
                Ok(output) => cmd_result(output),
                Err(err) => json_error(
                    500,
                    format!(
                        "pkexec başlatılamadı: {err}. Sistem yetki penceresi için polkit kurulu olmalı ya da uygulamayı root ile çalıştırın."
                    ),
                ),
            }
        } else {
            // Normal çalıştırma
            match Command::new("sh").arg("-c").arg(cmd_str).output() {
                Ok(output) => cmd_result(output),
                Err(err) => json_error(500, format!("Komut çalıştırılamadı: {err}")),
            }
        }
    }

    #[cfg(windows)]
    {
        if is_root_required && req.windows_confirmed != Some(true) {
            return json_ok(json!({
                "ok": false,
                "needs_elevation": true,
                "os": "windows",
                "reason": "Bu adli edinim işlemi fiziksel disk ve bellek bloklarına erişim için Yönetici (Administrator) yetkisi gerektirmektedir. İzin verilsin mi?"
            }));
        }

        match Command::new("cmd").arg("/C").arg(cmd_str).output() {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                json_ok(json!({
                    "ok": output.status.success(),
                    "stdout": stdout,
                    "stderr": stderr,
                    "exit_code": output.status.code().unwrap_or(-1)
                }))
            }
            Err(err) => json_error(500, format!("Komut çalıştırılamadı: {err}")),
        }
    }

    #[cfg(not(any(unix, windows)))]
    {
        json_error(501, "Desteklenmeyen işletim sistemi.")
    }
}

/// Amele SKILL.md kurallarına dayalı yerel adli bilişim yanıt motoru.
fn run_amele_rule_engine(prompt: &str, case_name: &str) -> String {
    let lower = prompt.to_lowercase();

    if lower.contains("ram") || lower.contains("bellek") || lower.contains("memory") {
        return format!(
            "### Fiziksel RAM Adli Edinimi\n\n\
            Amele kurallarına göre canlı sistem RAM edinimi **AVML** (Linux) veya **WinPMEM** (Windows) ile gerçekleştirilir.\n\n\
            **Önerilen Amele CLI Komutu:**\n\
            ```bash\n\
            sudo amele linux ram --case \"{}\" --hash sha256\n\
            ```\n\n\
            **Uzak Sunucu Üzerinden (SSH - Agentsız):**\n\
            ```bash\n\
            amele linux ram --ssh root@192.168.1.50 --case \"{}\"\n\
            ```\n\n\
            > Not: RAM edinimi çekirdek bellek sayfalarına doğrudan eriştiği için root yetkisi gerektirir.",
            case_name, case_name
        );
    }

    if lower.contains("disk")
        || lower.contains("dd")
        || lower.contains("imaj")
        || lower.contains("raw")
    {
        return format!(
            "### Fiziksel Blok Disk Edinimi\n\n\
            Amele, hedef diskleri bit-bit raw imaj veya AFF4 adli formatında paketleyerek hash bütünlüğünü anında doğrular.\n\n\
            **Yerel Disk Edinimi:**\n\
            ```bash\n\
            sudo amele linux disk /dev/nvme0n1 --case \"{}\" --format raw --hash sha256\n\
            ```\n\n\
            **Uzak Agent Üzerinden TCP Akışı:**\n\
            ```bash\n\
            amele linux disk /dev/sda --agent 10.0.0.15:9000 --token \"GIZLI_TOKEN\" --case \"{}\"\n\
            ```\n\n\
            *Edinim tamamlandığında SHA-256 ve MD5 hash değerleri otomatik hesaplanıp vaka loguna kaydedilir.*",
            case_name, case_name
        );
    }

    if lower.contains("android") || lower.contains("adb") || lower.contains("telefon") {
        return format!(
            "### Android Adli Bilişimi\n\n\
            Amele Android modülü, ADB veya Wi-Fi üzerinden mantıksal veri, APK dökümleri, çağrı kayıtları ve MFT benzeri dosya sistemini paketler.\n\n\
            **Android Cihaz Taraması:**\n\
            ```bash\n\
            amele android scan\n\
            ```\n\n\
            **Tam Mantıksal Edinim:**\n\
            ```bash\n\
            amele android acquire --case \"{}\" --all\n\
            ```",
            case_name
        );
    }

    if lower.contains("docker") || lower.contains("konteyner") || lower.contains("container") {
        return format!(
            "### Docker Konteyner Adli Bilişimi\n\n\
            Çalışan veya durdurulmuş Docker konteynerlerinden katman diff, ortam değişkenleri (secret scan) ve bellek dökümü alır.\n\n\
            **Konteyner Adli Analizi:**\n\
            ```bash\n\
            sudo amele docker inspect amele-web --case \"{}\" --scan-secrets\n\
            ```\n\n\
            *Gizli anahtarlar, API tokenları ve veri tabanı parolaları otomatik olarak tespit edilir.*",
            case_name
        );
    }

    if lower.contains("vaka") || lower.contains("case") || lower.contains("paket") {
        return format!(
            "### Vaka Yönetimi ve Bütünlük Doğrulama\n\n\
            **Aktif Vaka:** `{}`\n\n\
            **Vaka Bütünlüğünü Doğrulama:**\n\
            ```bash\n\
            amele case verify --case \"{}\"\n\
            ```\n\n\
            **Vakayı Şifreli Arşiv Olarak Paketleme:**\n\
            ```bash\n\
            amele case pack --case \"{}\" --output \"{}.tar.gz\" --encrypt\n\
            ```",
            case_name, case_name, case_name, case_name
        );
    }

    format!(
        "### Amele Adli Bilişim Asistanı\n\n\
        Aktif Vaka: **{}**\n\n\
        Amele platformunda disk edinimi, canlı RAM analizi, Android/iOS mobil edinim, Docker konteyner adli bilişimi ve SSH üzerinden disksiz edinim gerçekleştirebilirsiniz.\n\n\
        **Örnek Hızlı Komutlar:**\n\
        - `sudo amele linux disk /dev/nvme0n1 --case \"{}\"`\n\
        - `sudo amele linux ram --case \"{}\"`\n\
        - `amele android scan`\n\
        - `amele case list`\n\n\
        Herhangi bir soru sorabilir veya doğrudan komut ürettirebilirsiniz.",
        case_name, case_name, case_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agents_endpoint() {
        let resp = get_agents_endpoint();
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
        assert!(val["ok"].as_bool().unwrap());
        let agents = val["agents"].as_array().unwrap();
        assert!(!agents.is_empty());
        let pi = agents.iter().find(|a| a["id"] == "pi").unwrap();
        assert!(pi["models"].is_array());
        let agy = agents.iter().find(|a| a["id"] == "agy").unwrap();
        assert!(agy["models"].is_array());
    }

    #[test]
    fn test_chat_endpoint_empty_prompt() {
        let req = serde_json::json!({
            "prompt": "   "
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = chat_endpoint(&body);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_chat_endpoint_ram() {
        let req = serde_json::json!({
            "prompt": "RAM edinimi nasıl yapılır?",
            "agent": "test-mock",
            "case_name": "test_case"
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = chat_endpoint(&body);
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
        assert!(val["ok"].as_bool().unwrap());
        let resp_text = val["response"].as_str().unwrap();
        assert!(resp_text.contains("RAM"));
        assert_eq!(
            val["suggested_command"].as_str().unwrap(),
            "sudo amele linux ram --case \"test_case\" --hash sha256"
        );
    }

    #[test]
    fn test_execute_command_security_check() {
        let req = serde_json::json!({
            "command": "rm -rf /"
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = execute_command_endpoint(&body);
        assert_eq!(resp.status, 403);
    }

    #[test]
    fn test_execute_command_elevation_needed() {
        let req = serde_json::json!({
            "command": "sudo amele linux ram --case \"test\""
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = execute_command_endpoint(&body);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
        // If not root, must ask for elevation
        #[cfg(unix)]
        {
            let is_root = unsafe { libc::geteuid() == 0 };
            if !is_root {
                assert!(val["needs_elevation"].as_bool().unwrap());
                assert_eq!(val["os"].as_str().unwrap(), "linux");
            }
        }
    }
}
