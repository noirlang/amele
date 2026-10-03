//! Yapay zeka agent keşfi, model listeleme ve Amele SKILL.md kural motoru entegrasyonu.

use crate::server::{Response, json_error, json_ok};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
    pub profile_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteCommandRequest {
    pub command: String,
    pub sudo_password: Option<String>,
    pub windows_confirmed: Option<bool>,
    pub linux_confirmed: Option<bool>,
    pub profile_username: Option<String>,
    pub profile_fullname: Option<String>,
}

pub use crate::api::desktop::{check_binary, find_terminal_command};

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

/// Komuta --quiet bayragi ekler, temiz cikti icin.
// ajan komutlari sessiz calissin diye eklendi.
fn with_quiet(cmd: &str) -> String {
    if cmd.contains("--quiet") || cmd.contains("--no-logo") {
        return cmd.to_string();
    }
    if let Some(rest) = cmd.strip_prefix("sudo amele") {
        format!("sudo amele --quiet{rest}")
    } else if let Some(rest) = cmd.strip_prefix("amele") {
        format!("amele --quiet{rest}")
    } else {
        cmd.to_string()
    }
}

/// Çalıştırılabilir amele ikili dosyasını belirler.
fn resolve_amele_binary() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if exe
            .file_name()
            .map(|n| n.to_string_lossy().starts_with("amele"))
            .unwrap_or(false)
            && exe.is_file()
        {
            return exe.to_string_lossy().to_string();
        }
    }
    check_binary("amele").unwrap_or_else(|| "amele".to_string())
}

/// Arayuzde secili analist profilini CLI tarafinda aktif yapar.
/// CLI profilsiz komut calistirmadigi icin ajan komutlari Profile bulunamadi
/// hatasiyla dusuyordu, bunu onlemek icin eklendi. en fazla ~20sn surer.
fn ensure_cli_profile(username: Option<&str>, full_name: Option<&str>) {
    let u = match username {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => return,
    };
    if !u
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return;
    }
    let amele_bin = resolve_amele_binary();
    if let Some(out) = run_cli_with_timeout(&amele_bin, &["profile", "use", u, "--direct"], 10) {
        if out.status.success() {
            return;
        }
    }
    if let Some(name) = full_name {
        let name = name.trim();
        if !name.is_empty() && !name.chars().any(|c| matches!(c, '\0' | '\n' | '\r')) {
            if let Some(out) =
                run_cli_with_timeout(&amele_bin, &["profile", "create", name, u, "--direct"], 10)
            {
                if out.status.success() {
                    return;
                }
            }
            let _ = run_cli_with_timeout(&amele_bin, &["profile", "use", u, "--direct"], 10);
        }
    }
}

fn fetch_agy_models(binary_path: &str) -> Vec<AgentModel> {
    if let Some(output) = run_cli_with_timeout(binary_path, &["models"], 3) {
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
    if let Some(output) = run_cli_with_timeout(binary_path, &["models"], 3) {
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
    if let Some(output) = run_cli_with_timeout(binary_path, &["login", "status"], 3) {
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
    if let Some(output) = run_cli_with_timeout(binary_path, &["auth", "check"], 3) {
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
    if let Some(output) = run_cli_with_timeout(binary_path, &["auth", "status"], 3) {
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
            models: vec![],
        },
        DiscoveredAgent {
            id: "claude".to_string(),
            name: "Claude Code".to_string(),
            installed: claude_path.is_some(),
            binary_path: claude_path,
            description: "Anthropic Claude Code CLI Asistanı".to_string(),
            models: vec![],
        },
        DiscoveredAgent {
            id: "codex".to_string(),
            name: "Codex".to_string(),
            installed: codex_path.is_some(),
            binary_path: codex_path,
            description: "OpenAI Codex CLI Ajanı".to_string(),
            models: vec![],
        },
        DiscoveredAgent {
            id: "pi".to_string(),
            name: "Pi".to_string(),
            installed: pi_path.is_some(),
            binary_path: pi_path,
            description: "Earendil Works çok sağlayıcılı terminal ajanı".to_string(),
            models: vec![],
        },
        DiscoveredAgent {
            id: "opencode".to_string(),
            name: "OpenCode".to_string(),
            installed: opencode_path.is_some(),
            binary_path: opencode_path,
            description: "Açık kaynak çoklu sağlayıcı CLI ajanı".to_string(),
            models: vec![],
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

/// Agent sorguları için standartlaştırılmış istem talimatı üretir.
fn format_agent_instruction(
    prompt: &str,
    case_name: &str,
    target_scope: &str,
    profile_name: &str,
) -> String {
    format!(
        "Sen Amele Adli Bilişim (Digital Forensics) platformunun yapay zeka asistanısın.\n\n\
        Amele SKILL.md Kural ve Beceri Kılavuzu:\n{}\n\n\
        Analist Profili / Kullanıcı: {}\n\
        Aktif Vaka: {}\n\
        İnceleme Kapsamı: {}\n\n\
        📌 ZORUNLU YANIT KURALI:\n\
        Yanıtına MUTLAKA analist profilini, aktif vakayı ve yetki durumunu özetleyen net bir 'Giriş & Durum Tespiti' (Giriş Kısmı) ile başla.\n\
        Ardından doğrudan çalıştırılabilir Amele CLI komutunu tek satırlık kod bloğu olarak ver.\n\n\
        Analistin İstemi / Kullanıcı Sorusu: {}",
        load_amele_skill_text(),
        profile_name,
        case_name,
        target_scope,
        prompt
    )
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
    let profile_name = req.profile_name.as_deref().unwrap_or("-");

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

    let combined = format_agent_instruction(prompt, case_name, target_scope, profile_name);

    match agent_id {
        "pi" => {
            let bin = check_binary("pi").unwrap_or_else(|| "pi".to_string());
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

#[derive(Debug, Deserialize)]
pub struct LaunchTerminalRequest {
    pub prompt: String,
    pub agent: Option<String>,
    pub model: Option<String>,
    pub case_name: Option<String>,
    pub target_scope: Option<String>,
    pub profile_name: Option<String>,
    pub profile_username: Option<String>,
    pub profile_fullname: Option<String>,
}

/// Yapay zekaya aktarılacak zengin sistem, profil, vaka ve Amele SKILL.md bağlamını üretir.
fn build_agent_full_prompt(
    user_prompt: &str,
    analist_name: &str,
    username: &str,
    case_name: &str,
    target_scope: &str,
    working_dir: &str,
) -> String {
    let os_name = std::env::consts::OS;
    let arch_name = std::env::consts::ARCH;

    let amele_path = check_binary("amele").unwrap_or_else(|| "amele".to_string());
    let adb_path = check_binary("adb").unwrap_or_else(|| "bulunamadı".to_string());
    let avml_path = check_binary("avml").unwrap_or_else(|| "bulunamadı".to_string());
    let docker_path = check_binary("docker").unwrap_or_else(|| "bulunamadı".to_string());

    let profile_display = if !analist_name.is_empty() && !username.is_empty() {
        format!("{analist_name} (@{username})")
    } else if !analist_name.is_empty() {
        analist_name.to_string()
    } else if !username.is_empty() {
        format!("@{username}")
    } else {
        "Bilinmeyen Analist".to_string()
    };

    let skill_text = load_amele_skill_text();

    format!(
        r#"# Amele Adli Bilişim Asistanı Görev Talimatı

Sen Amele Adli Bilişim (Digital Forensics & Incident Response) platformunun uzman yapay zeka asistanısın.
Analiste adli bilişim incelemelerinde, disk/RAM/mobil/docker edinimlerinde ve Amele CLI komutlarında rehberlik et.

## 👤 Analist & Oturum Bilgileri
- Analist / Kullanıcı Adı: {profile_display}
- Aktif Vaka: {case_name}
- İnceleme Kapsamı: {target_scope}
- Çalışma Dizini: {working_dir}

## 💻 Sistem & Araç Bilgileri
- İşletim Sistemi: {os_name} ({arch_name})
- Amele CLI Yolu: {amele_path}
- ADB Durumu: {adb_path}
- AVML / RAM Aracı: {avml_path}
- Docker Durumu: {docker_path}

## 📌 Yanıt Formatı ve Giriş Bölümü Zorunluluğu:
1. Yanıtına MUTLAKA analist profilini ({profile_display}), aktif vakayı ({case_name}) ve inceleme amacını özetleyen net bir **Giriş & Durum Tespiti (Giriş Kısmı)** ile başla.
2. Adli edinim gereksinimlerini (root/sudo ihtiyacı, vaka adı, hedef blok aygıt) açıkça belirt.
3. Ardından doğrudan çalıştırılabilir Amele CLI komutunu ve delil bütünlüğü adımlarını ver.

## 📖 Amele Kural & Beceri Kılavuzu (SKILL.md)
{skill_text}

---
## 🎯 Analistin İstemi:
{user_prompt}
"#
    )
}

/// Yapay zeka ajanını varsayılan sistem terminalinde etkileşimli olarak başlatır.
/// Amele adli bilişim skill kuralları ve kullanıcının prompt'u oturuma aktarılır.
pub fn launch_terminal_endpoint(body: &[u8]) -> Response {
    let req: LaunchTerminalRequest = match serde_json::from_slice(body) {
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

    if !agent_id
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return json_error(400, "Geçersiz ajan tanımlayıcısı.");
    }

    if !model_id.is_empty()
        && !model_id.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':' || c == '/'
        })
    {
        return json_error(400, "Geçersiz model tanımlayıcısı.");
    }

    if case_name.chars().any(|c| {
        matches!(
            c,
            '"' | '\'' | '$' | '`' | '\n' | '\r' | ';' | '&' | '|' | '\\'
        )
    }) {
        return json_error(400, "Vaka adında geçersiz özel karakterler bulunamaz.");
    }

    // Profil kontrolü (kullanıcı adı ve tam ad)
    let (mut username, mut full_name) = (
        req.profile_username
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
        req.profile_fullname
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
    );

    if username.is_none() || full_name.is_none() {
        if let Some(ref p) = req.profile_name {
            if let Some(start) = p.find('(') {
                if let Some(end) = p.rfind(')') {
                    let u = p[start + 1..end].trim();
                    let name = p[..start].trim();
                    if username.is_none() && !u.is_empty() {
                        username = Some(u);
                    }
                    if full_name.is_none() && !name.is_empty() {
                        full_name = Some(name);
                    }
                } else if username.is_none() {
                    username = Some(p.trim());
                }
            } else if username.is_none() {
                username = Some(p.trim());
            }
        }
    }

    if username.is_none() && full_name.is_none() {
        return json_error(
            400,
            "Aktif analist profili bulunamadı. Lütfen önce profil seçin veya yeni bir profil oluşturun.",
        );
    }

    ensure_cli_profile(username, full_name);

    // Çalışma dizini: Mevcut çalışma dizini, yoksa kullanıcı ana dizini
    let working_dir = std::env::current_dir()
        .ok()
        .or_else(|| {
            std::env::var_os("HOME")
                .or_else(|| std::env::var_os("USERPROFILE"))
                .map(std::path::PathBuf::from)
        })
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string());

    // Agent ikili dosyası ve komut satırı
    let (agent_display_name, agent_cmd) = match agent_id {
        "claude" => {
            let bin = check_binary("claude").unwrap_or_else(|| "claude".to_string());
            let cmd = if model_id.is_empty() {
                format!(
                    "\"{bin}\" --append-system-prompt \"You are the Amele Digital Forensics Agent. Follow the amele digital forensics skills and rules.\" \"$PROMPT\""
                )
            } else {
                format!(
                    "\"{bin}\" --model \"{model_id}\" --append-system-prompt \"You are the Amele Digital Forensics Agent. Follow the amele digital forensics skills and rules.\" \"$PROMPT\""
                )
            };
            ("Claude Code", cmd)
        }
        "codex" => {
            let bin = check_binary("codex").unwrap_or_else(|| "codex".to_string());
            let cmd = if model_id.is_empty() {
                format!("\"{bin}\" \"$PROMPT\"")
            } else {
                format!("\"{bin}\" -m \"{model_id}\" \"$PROMPT\"")
            };
            ("Codex", cmd)
        }
        "pi" => {
            let bin = check_binary("pi").unwrap_or_else(|| "pi".to_string());
            let home = std::env::var("HOME").unwrap_or_default();
            let skill_arg = format!("--skill \"{home}/.gemini/config/skills/amele\"");
            let cmd = if model_id.is_empty() {
                format!("\"{bin}\" {skill_arg} \"$PROMPT\"")
            } else {
                format!("\"{bin}\" {skill_arg} --model \"{model_id}\" \"$PROMPT\"")
            };
            ("Pi", cmd)
        }
        "opencode" => {
            let bin = check_binary("opencode").unwrap_or_else(|| "opencode".to_string());
            let cmd = if model_id.is_empty() {
                format!("\"{bin}\" --prompt \"$PROMPT\"")
            } else {
                format!("\"{bin}\" -m \"{model_id}\" --prompt \"$PROMPT\"")
            };
            ("OpenCode", cmd)
        }
        _ => {
            // varsayılan agy (Google Antigravity)
            let bin = check_binary("agy").unwrap_or_else(|| "agy".to_string());
            let cmd = if model_id.is_empty() {
                format!("\"{bin}\" -i \"$PROMPT\"")
            } else {
                format!("\"{bin}\" --model \"{model_id}\" -i \"$PROMPT\"")
            };
            ("Antigravity (AGY)", cmd)
        }
    };

    // Güvenli runtime dizini (XDG_RUNTIME_DIR veya kullanıcıya özel amele klasörü)
    let run_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(|p| std::path::PathBuf::from(p).join("amele"))
        .unwrap_or_else(|_| {
            #[cfg(unix)]
            {
                std::env::temp_dir().join(format!("amele-{}", unsafe { libc::geteuid() }))
            }
            #[cfg(not(unix))]
            {
                std::env::temp_dir().join("amele")
            }
        });
    let _ = std::fs::create_dir_all(&run_dir);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&run_dir) {
            let mut perms = meta.permissions();
            perms.set_mode(0o700);
            let _ = std::fs::set_permissions(&run_dir, perms);
        }
    }

    let session_id = format!(
        "{}_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis(),
        std::process::id()
    );

    let prompt_file = run_dir.join(format!("prompt_{session_id}.txt"));
    let script_file = run_dir.join(format!("agent_run_{session_id}.sh"));

    // Kullanıcının sorusu, Amele SKILL.md, sistem bilgileri, kullanıcı adı ve aktif vaka ile zenginleştirilmiş tam istem üretilir
    let target_scope = req.target_scope.as_deref().unwrap_or("all");
    let full_prompt = build_agent_full_prompt(
        prompt,
        full_name.unwrap_or(""),
        username.unwrap_or(""),
        case_name,
        target_scope,
        &working_dir,
    );

    if let Err(e) = std::fs::write(&prompt_file, &full_prompt) {
        return json_error(500, format!("İstem dosyası oluşturulamadı: {e}"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&prompt_file) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(&prompt_file, perms);
        }
    }

    let model_display = if model_id.is_empty() {
        "Varsayılan"
    } else {
        model_id
    };

    let safe_working_dir = working_dir
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`");

    let script_content = format!(
        r#"#!/usr/bin/env bash
PROMPT_FILE="{prompt_file_path}"
SCRIPT_FILE="{script_file_path}"
trap 'rm -f "$PROMPT_FILE" "$SCRIPT_FILE"' EXIT

export TERM=xterm-256color
export PATH="$HOME/.local/bin:$HOME/.local/share/mise/shims:$HOME/.cargo/bin:/usr/local/bin:/usr/bin:$PATH"
cd "{working_dir}"

clear
echo -e "\033[1;36m═══════════════════════════════════════════════════════════════════════\033[0m"
echo -e "\033[1;32m  AMELE ADLİ BİLİŞİM — YAPAY ZEKA AJAN OTURUMU\033[0m"
echo -e "\033[1;37m  Ajan: {agent_display_name} | Model: {model_display} | Vaka: {case_name}\033[0m"
echo -e "\033[1;36m═══════════════════════════════════════════════════════════════════════\033[0m"
echo

if [ -f "$PROMPT_FILE" ]; then
  PROMPT=$(cat "$PROMPT_FILE")
else
  PROMPT=""
fi

{agent_cmd}
EXIT_CODE=$?

echo
if [ $EXIT_CODE -ne 0 ]; then
  echo -e "\033[1;31mAjan $EXIT_CODE kodu ile kapandı.\033[0m"
fi
read -r -p "Terminali kapatmak için Enter tuşuna basın..." _
"#,
        prompt_file_path = prompt_file.display(),
        script_file_path = script_file.display(),
        working_dir = safe_working_dir,
        agent_display_name = agent_display_name,
        model_display = model_display,
        case_name = case_name,
        agent_cmd = agent_cmd
    );

    if let Err(e) = std::fs::write(&script_file, &script_content) {
        return json_error(500, format!("Çalıştırma betiği yazılamadı: {e}"));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(&script_file) {
            let mut perms = meta.permissions();
            perms.set_mode(0o700);
            let _ = std::fs::set_permissions(&script_file, perms);
        }
    }

    let script_str = script_file.to_string_lossy().to_string();
    let (term_bin, term_args) = match find_terminal_command(&script_str) {
        Some(pair) => pair,
        None => {
            return json_error(
                500,
                "Sistemde desteklenen bir terminal emülatörü bulunamadı (alacritty, kitty, gnome-terminal, konsole vb.).",
            );
        }
    };

    match Command::new(&term_bin).args(&term_args).spawn() {
        Ok(_) => json_ok(json!({
            "ok": true,
            "terminal": term_bin,
            "agent": agent_id,
            "model": model_id,
            "message": format!("Terminal başlatıldı ({term_bin})")
        })),
        Err(err) => json_error(500, format!("Terminal başlatılamadı: {err}")),
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

/// Komut dizgisini güvenli şekilde argüman listesine ayrıştırır.
/// Kabuk metakarakterleri (;, &, |, `, $, >, <, yeni satır vb.) engellenir.
fn parse_cli_args(input: &str) -> Result<Vec<String>, String> {
    for ch in input.chars() {
        if matches!(
            ch,
            ';' | '&' | '|' | '`' | '$' | '>' | '<' | '\n' | '\r' | '\0'
        ) {
            return Err(
                "Güvenlik ihlali: Komutta kabuk kontrol veya yönlendirme karakterleri bulunamaz."
                    .to_string(),
            );
        }
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quote = None;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            '\'' | '"' => {
                if in_quote == Some(c) {
                    in_quote = None;
                } else if in_quote.is_none() {
                    in_quote = Some(c);
                } else {
                    current.push(c);
                }
            }
            c if c.is_whitespace() && in_quote.is_none() => {
                if !current.is_empty() {
                    args.push(std::mem::take(&mut current));
                }
            }
            c => {
                current.push(c);
            }
        }
    }

    if in_quote.is_some() {
        return Err("Geçersiz komut sözdizimi: Kapatılmamış tırnak işareti.".to_string());
    }

    if !current.is_empty() {
        args.push(current);
    }

    if args.is_empty() {
        return Err("Komut boş olamaz.".to_string());
    }

    Ok(args)
}

/// Kullanıcının onayladığı CLI komutunu çalıştırır.
/// Güvenlik: Kabuk çağrısı (sh -c / cmd /C) YAPILMAZ; komut argümanlara ayrıştırılıp
/// doğrudan amele ikili dosyası olarak çalıştırılır.
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

    // Komut güvenli ayrıştırma ve kabuk enjeksiyon kontrolü
    let tokens = match parse_cli_args(cmd_str) {
        Ok(t) => t,
        Err(e) => return json_error(400, e),
    };

    let is_sudo_prefixed = tokens[0] == "sudo";
    let amele_idx = if is_sudo_prefixed { 1 } else { 0 };

    if tokens.get(amele_idx).map(String::as_str) != Some("amele") {
        return json_error(403, "Yalnızca resmi Amele CLI komutları çalıştırılabilir.");
    }

    // arayuzdeki aktif profili CLI tarafinda da aktif yap
    ensure_cli_profile(
        req.profile_username.as_deref(),
        req.profile_fullname.as_deref(),
    );

    let amele_bin = resolve_amele_binary();
    let mut amele_args: Vec<String> = tokens[amele_idx + 1..].to_vec();

    // Ajan komutlarının sessiz (başlıksız) çalışması için --quiet ekle
    if !amele_args
        .iter()
        .any(|a| a == "--quiet" || a == "-q" || a == "--no-logo")
    {
        amele_args.insert(0, "--quiet".to_string());
    }

    let is_root_required = is_sudo_prefixed
        || amele_args.iter().any(|a| {
            a == "disk"
                || a == "ram"
                || a == "avml"
                || a == "winpmem"
                || a.contains("/dev/")
                || a.contains("PhysicalDrive")
        });

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
                .arg(&amele_bin)
                .args(&amele_args)
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

            // parola geldiyse sudo -S ile calistir
            if let Some(password) = req.sudo_password.as_deref().filter(|p| !p.is_empty()) {
                let mut child = match Command::new("sudo")
                    .arg("-S")
                    .arg("-p")
                    .arg("")
                    .arg(&amele_bin)
                    .args(&amele_args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                {
                    Ok(c) => c,
                    Err(err) => return json_error(500, format!("Süreç başlatılamadı: {err}")),
                };

                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(format!("{password}\n").as_bytes());
                }

                return match child.wait_with_output() {
                    Ok(output) => cmd_result(output),
                    Err(err) => json_error(500, format!("Komut tamamlanamadı: {err}")),
                };
            }

            // parola yoksa sistem penceresini ac (polkit)
            match Command::new("pkexec")
                .arg(&amele_bin)
                .args(&amele_args)
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
            // Normal çalıştırma (kabuksuz doğrudan yürütme)
            match Command::new(&amele_bin).args(&amele_args).output() {
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

        match Command::new(&amele_bin).args(&amele_args).output() {
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

    if lower.contains("profil")
        || lower.contains("profile")
        || lower.contains("giriş")
        || lower.contains("login")
        || lower.contains("oturum")
    {
        return format!(
            "### 1. Giriş & Profil Yönetimi (Oturum Açma)\n\n\
            Amele adli bilişim incelemelerinde tüm delil zincirinin ve işlemlerin denetlenebilmesi için analist oturumu zorunludur.\n\n\
            **Profil ile Giriş Yapma / Oturum Açma (Login):**\n\
            ```bash\n\
            amele profile use <kullanici_adi> --direct\n\
            ```\n\n\
            **Yeni Profil Oluşturma & Anında Giriş:**\n\
            ```bash\n\
            amele profile create \"<Ad Soyad>\" <kullanici_adi> tr dark --direct\n\
            ```\n\n\
            **Aktif Oturum ve Profilleri Listeleme:**\n\
            ```bash\n\
            amele profile list\n\
            ```\n\n\
            **Çevrimiçi Lisans & Yetki Eşitleme:**\n\
            ```bash\n\
            amele profile sync\n\
            ```\n\n\
            **Oturumu Kapatma (Çıkış):**\n\
            ```bash\n\
            amele profile logout\n\
            ```"
        );
    }

    if lower.contains("ram") || lower.contains("bellek") || lower.contains("memory") {
        return format!(
            "### 1. Giriş & Durum Tespiti\n\n\
            - **Aktif Vaka:** `{}`\n\
            - **İşlem:** Canlı Fiziksel RAM Edinimi (AVML / WinPMEM)\n\
            - **Yetki Seviyesi:** Root / Sudo Yetkisi Gerekir (Çekirdek Bellek Sayfaları)\n\n\
            ### 2. Önerilen Amele CLI Komutu\n\
            ```bash\n\
            sudo amele --quiet linux ram --case \"{}\" --hash sha256\n\
            ```\n\n\
            **Uzak Sunucu Üzerinden (SSH - Agentsız):**\n\
            ```bash\n\
            amele --quiet linux ram --ssh root@192.168.1.50 --case \"{}\"\n\
            ```\n\n\
            > Not: Canlı RAM edinimi tamamlandığında SHA-256 hash bütünlüğü otomatik hesaplanır.",
            case_name, case_name, case_name
        );
    }

    if lower.contains("disk")
        || lower.contains("dd")
        || lower.contains("imaj")
        || lower.contains("raw")
    {
        return format!(
            "### 1. Giriş & Durum Tespiti\n\n\
            - **Aktif Vaka:** `{}`\n\
            - **İşlem:** Fiziksel Blok Disk Edinimi\n\
            - **Yetki Seviyesi:** Root / Sudo Yetkisi Gerekir (Blok aygıt okuma)\n\n\
            ### 2. Önerilen Amele CLI Komutu\n\
            ```bash\n\
            sudo amele --quiet linux disk /dev/nvme0n1 --case \"{}\" --format raw --hash sha256\n\
            ```\n\n\
            **Uzak Agent Üzerinden TCP Akışı:**\n\
            ```bash\n\
            amele --quiet linux disk /dev/sda --agent 10.0.0.15:9000 --token \"GIZLI_TOKEN\" --case \"{}\"\n\
            ```\n\n\
            *Edinim tamamlandığında SHA-256 ve MD5 hash değerleri otomatik hesaplanıp vaka loguna kaydedilir.*",
            case_name, case_name, case_name
        );
    }

    if lower.contains("android") || lower.contains("adb") || lower.contains("telefon") {
        return format!(
            "### 1. Giriş & Durum Tespiti\n\n\
            - **Aktif Vaka:** `{}`\n\
            - **İşlem:** Android Mobil Adli İnceleme\n\
            - **Ön Gereksinim:** USB Hata Ayıklama (ADB) Açık Olmalı\n\n\
            ### 2. Önerilen Amele CLI Komutu\n\
            ```bash\n\
            amele --quiet android acquire --case \"{}\" --all\n\
            ```\n\n\
            **Cihaz Taraması:**\n\
            ```bash\n\
            amele --quiet android scan\n\
            ```",
            case_name, case_name
        );
    }

    if lower.contains("docker") || lower.contains("konteyner") || lower.contains("container") {
        return format!(
            "### 1. Giriş & Durum Tespiti\n\n\
            - **Aktif Vaka:** `{}`\n\
            - **İşlem:** Docker Konteyner Adli İncelemesi & Kaçış Riski\n\
            - **Yetki Seviyesi:** Docker soket erişimi (sudo / docker grubu)\n\n\
            ### 2. Önerilen Amele CLI Komutu\n\
            ```bash\n\
            sudo amele --quiet docker inspect amele-web --case \"{}\" --scan-secrets\n\
            ```\n\n\
            *Gizli anahtarlar, API tokenları ve veri tabanı parolaları otomatik olarak tespit edilir.*",
            case_name, case_name
        );
    }

    if lower.contains("vaka") || lower.contains("case") || lower.contains("paket") {
        return format!(
            "### 1. Giriş & Durum Tespiti\n\n\
            - **Aktif Vaka:** `{}`\n\
            - **İşlem:** Vaka Yönetimi ve Bütünlük Doğrulama\n\n\
            ### 2. Önerilen Amele CLI Komutu\n\
            ```bash\n\
            amele --quiet case verify --case \"{}\"\n\
            ```\n\n\
            **Vakayı Taşınabilir İmzalı Pakete Dönüştürme:**\n\
            ```bash\n\
            amele --quiet case export \"{}\" \"/delil/{}.amelecase\"\n\
            ```",
            case_name, case_name, case_name, case_name
        );
    }

    format!(
        "### 1. Giriş & Durum Tespiti\n\n\
        - **Aktif Vaka:** `{}`\n\
        - **Platform:** Amele Adli Bilişim Asistanı\n\n\
        Amele platformunda disk edinimi, canlı RAM analizi, Android/iOS mobil edinim, Docker konteyner adli bilişimi ve SSH üzerinden disksiz edinim gerçekleştirebilirsiniz.\n\n\
        ### 2. Hızlı Komutlar:\n\
        - `sudo amele --quiet linux disk /dev/nvme0n1 --case \"{}\"`\n\
        - `sudo amele --quiet linux ram --case \"{}\"`\n\
        - `amele --quiet android scan`\n\
        - `amele --quiet case list`\n\n\
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
            "sudo amele --quiet linux ram --case \"test_case\" --hash sha256"
        );
    }

    #[test]
    fn test_chat_endpoint_profile_login() {
        let req = serde_json::json!({
            "prompt": "Giriş yapma ve profil oturumu komutları nelerdir?",
            "agent": "test-mock",
            "case_name": "test_case"
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = chat_endpoint(&body);
        assert_eq!(resp.status, 200);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
        assert!(val["ok"].as_bool().unwrap());
        let resp_text = val["response"].as_str().unwrap();
        assert!(resp_text.contains("Giriş"));
        assert!(resp_text.contains("amele profile use"));
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

    #[test]
    fn test_launch_terminal_empty_prompt() {
        let req = serde_json::json!({
            "prompt": "   "
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = launch_terminal_endpoint(&body);
        assert_eq!(resp.status, 400);
    }

    #[test]
    fn test_launch_terminal_missing_profile() {
        let req = serde_json::json!({
            "prompt": "Test sorgusu"
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = launch_terminal_endpoint(&body);
        assert_eq!(resp.status, 400);
        let val: serde_json::Value = serde_json::from_slice(&resp.body).unwrap();
        assert_eq!(val["ok"], false);
        assert!(
            val["error"]
                .as_str()
                .unwrap()
                .contains("Aktif analist profili bulunamadı")
        );
    }

    #[test]
    fn test_find_terminal_command() {
        // ci kosan kutuda terminal olmayabiliyor, o yuzden sahte $TERMINAL ile bakiyoruz
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let tmp = std::env::temp_dir().join(format!("amele-fake-term-{}", std::process::id()));
            let _ = std::fs::create_dir_all(&tmp);
            let fake = tmp.join("fake-term");
            std::fs::write(&fake, "#!/bin/sh\nexit 0\n").unwrap();
            if let Ok(meta) = std::fs::metadata(&fake) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&fake, perms);
            }

            let old = std::env::var("TERMINAL").ok();
            // rust 2024te env yazmak unsafe, test tek threadde kosuyor o yuzden sorun yok
            unsafe {
                std::env::set_var("TERMINAL", fake.to_string_lossy().to_string());
            }

            let res = find_terminal_command("/tmp/test.sh");
            assert!(res.is_some());
            let (bin, args) = res.unwrap();
            assert!(!bin.is_empty());
            assert!(!args.is_empty());

            unsafe {
                if let Some(v) = old {
                    std::env::set_var("TERMINAL", v);
                } else {
                    std::env::remove_var("TERMINAL");
                }
            }
            let _ = std::fs::remove_file(&fake);

            // gercek ortamda terminal yoksa da paniklemeden None donebilmeli
            let res2 = find_terminal_command("/tmp/test.sh");
            if let Some((bin, args)) = res2 {
                assert!(!bin.is_empty());
                assert!(!args.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_cli_args_valid() {
        let args = parse_cli_args("amele disk acquire /dev/sdb \"case 1\" --format raw").unwrap();
        assert_eq!(
            args,
            vec![
                "amele", "disk", "acquire", "/dev/sdb", "case 1", "--format", "raw"
            ]
        );

        let sudo_args = parse_cli_args("sudo amele ram dump /tmp/dump.raw").unwrap();
        assert_eq!(
            sudo_args,
            vec!["sudo", "amele", "ram", "dump", "/tmp/dump.raw"]
        );
    }

    #[test]
    fn test_parse_cli_args_shell_injection_blocked() {
        assert!(parse_cli_args("amele disk list; rm -rf /").is_err());
        assert!(parse_cli_args("amele disk list && echo 1").is_err());
        assert!(parse_cli_args("amele disk list || echo 1").is_err());
        assert!(parse_cli_args("amele disk list | whoami").is_err());
        assert!(parse_cli_args("amele disk `id`").is_err());
        assert!(parse_cli_args("amele disk $(whoami)").is_err());
        assert!(parse_cli_args("amele disk > out.txt").is_err());
        assert!(parse_cli_args("amele disk < in.txt").is_err());
        assert!(parse_cli_args("amele disk\nid").is_err());
    }

    #[test]
    fn test_execute_command_injection_rejected() {
        let req = serde_json::json!({
            "command": "amele disk list; cat /etc/shadow"
        });
        let body = serde_json::to_vec(&req).unwrap();
        let resp = execute_command_endpoint(&body);
        assert_eq!(resp.status, 400);

        let req_pipe = serde_json::json!({
            "command": "sudo amele ram | rm -rf /"
        });
        let body_pipe = serde_json::to_vec(&req_pipe).unwrap();
        let resp_pipe = execute_command_endpoint(&body_pipe);
        assert_eq!(resp_pipe.status, 400);
    }

    #[test]
    fn test_launch_terminal_invalid_inputs() {
        // Geçersiz model kimliği (enjeksiyon denemesi)
        let req_model = serde_json::json!({
            "prompt": "Test",
            "model": "model\"; rm -rf / #",
            "profile_username": "analyst"
        });
        let resp_model = launch_terminal_endpoint(&serde_json::to_vec(&req_model).unwrap());
        assert_eq!(resp_model.status, 400);

        // Geçersiz vaka adı
        let req_case = serde_json::json!({
            "prompt": "Test",
            "case_name": "vaka; id",
            "profile_username": "analyst"
        });
        let resp_case = launch_terminal_endpoint(&serde_json::to_vec(&req_case).unwrap());
        assert_eq!(resp_case.status, 400);
    }
}
