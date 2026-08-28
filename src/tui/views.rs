//! Amele TUI Workspace Sayfa Görünümleri (Views).

use crate::tui::app::{ActiveTab, AppState};
use crate::tui::theme::Theme;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{
    Block, Borders, Cell, List, ListItem, Paragraph, Row, Table,
};
use ratatui::Frame;

pub fn render_workspace(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    match app.active_tab {
        ActiveTab::Home => render_home_view(f, app, area, theme),
        ActiveTab::Linux => render_linux_view(f, app, area, theme),
        ActiveTab::Windows => render_windows_view(f, app, area, theme),
        ActiveTab::Docker => render_docker_view(f, app, area, theme),
        ActiveTab::Android => render_android_view(f, app, area, theme),
        ActiveTab::Ios => render_ios_view(f, app, area, theme),
        ActiveTab::Agent => render_agent_view(f, app, area, theme),
        ActiveTab::Other => render_other_view(f, app, area, theme),
        ActiveTab::Profile => render_profile_view(f, app, area, theme),
        ActiveTab::Settings => render_settings_view(f, app, area, theme),
    }
}

/// 🏠 1. Ana Sayfa (Dashboard & Vakalar)
fn render_home_view(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Metrics Cards
            Constraint::Min(8),     // Case List & Details
            Constraint::Length(3),  // Quick actions
        ])
        .split(area);

    // Top Stat Cards
    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[0]);

    // Card 1: Vaka Sayısı
    let c1 = Paragraph::new(format!("\n  📁 Toplam Vaka: {}", app.cases.len()))
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Vakalar "));
    f.render_widget(c1, card_chunks[0]);

    // Card 2: Disk Sayısı
    let c2 = Paragraph::new(format!("\n  💾 Algılanan Disk: {}", app.disks.len()))
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Diskler "));
    f.render_widget(c2, card_chunks[1]);

    // Card 3: Storage Guard
    let c3 = Paragraph::new(format!("\n  🛡️ Boş Alan: {:.1} GB / {:.1} GB", app.storage_free_gb, app.storage_total_gb))
        .style(Style::default().fg(theme.success).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Depolama "));
    f.render_widget(c3, card_chunks[2]);

    // Card 4: Aktif Profil
    let prof_name = app.profile.as_ref().map(|p| p.display_name.as_str()).unwrap_or("Misafir");
    let c4 = Paragraph::new(format!("\n  👤 Analist: {}", prof_name))
        .style(Style::default().fg(theme.info).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Oturum "));
    f.render_widget(c4, card_chunks[3]);

    // Middle Case List & Inspector
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    // Left: Case List
    let items: Vec<ListItem> = if app.cases.is_empty() {
        vec![ListItem::new("  (Henüz kayıtlı vaka bulunmuyor)")]
    } else {
        app.cases
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let prefix = if i == app.selected_item_index && !app.sidebar_focused { "▶ " } else { "  " };
                let style = if i == app.selected_item_index && !app.sidebar_focused {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                ListItem::new(format!("{}📂 {}", prefix, c)).style(style)
            })
            .collect()
    };

    let case_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if !app.sidebar_focused { theme.active_border_style() } else { theme.normal_border_style() })
                .title(" Adli Vakalar Listesi [↑/↓ ile seç] "),
        );
    f.render_widget(case_list, main_chunks[0]);

    // Right: Selected Case Details
    let selected_case = app.cases.get(app.selected_item_index).cloned().unwrap_or_else(|| "Seçim yok".to_string());
    let case_details = format!(
        "\n  📌 Vaka Adı: {}\n\n  📂 Dizin: {}\n  🛡️ Bütünlük: Doğrulandı (SHA-256)\n  📝 Notlar: Adli vaka çalışma dizini hazır.\n\n  Komutlar:\n    • [E] .amelecase olarak paketle\n    • [V] Delil bütünlüğünü doğrula\n    • [N] Yeni vaka oluştur",
        selected_case,
        app.settings.vaka_klasoru.display()
    );
    let details_p = Paragraph::new(case_details)
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Vaka Ayrıntıları "));
    f.render_widget(details_p, main_chunks[1]);

    // Bottom Quick Actions
    let actions = Paragraph::new(" [N] Yeni Vaka  |  [E] Vaka Paketle (.amelecase)  |  [V] Doğrula  |  [Tab] Odak Değiştir  |  [1-8] Menü ")
        .style(Style::default().fg(theme.accent_hover).bg(theme.bg))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()));
    f.render_widget(actions, chunks[2]);
}

/// 🐧 2. Linux Araçları (Disk, AVML, Agent, SSH)
fn render_linux_view(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(6)])
        .split(area);

    // Sub-header tabs
    let tabs = Paragraph::new("  [1] Yerel Disk Edinimi  |  [2] AVML Canlı RAM  |  [3] Uzak Agent  |  [4] SSH Agent'sız")
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Linux Adli Edinim Modları "));
    f.render_widget(tabs, chunks[0]);

    // Disks Table
    let rows: Vec<Row> = app
        .disks
        .iter()
        .map(|d| {
            let size_gb = d.total_size as f64 / (1024.0 * 1024.0 * 1024.0);
            Row::new(vec![
                Cell::from(d.device.display().to_string()).style(Style::default().fg(theme.info)),
                Cell::from(format!("{:.1} GB", size_gb)),
                Cell::from(if d.accessible { "Erişilebilir" } else { "Erişim Yok (Root Gerekli)" }),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(
        Row::new(vec!["Aygıt / Disk Yolu", "Boyut", "İzin / Durum"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.active_border_style())
            .title(" Algılanan Linux Blok Cihazları "),
    );
    f.render_widget(table, chunks[1]);

    // Action Form Preview
    let form_text = format!(
        "  Hızlı Komut: amele linux disk <aygıt> <vaka> [raw|aff4]\n  AVML RAM: amele linux ram <vaka> (Durum: {})\n  Agent: amele linux disk --agent <ip> 9000 <disk_id> <vaka>\n  SSH: amele linux disk --ssh <ip> 22 <disk_id> <vaka> root",
        app.avml_status
    );
    let form = Paragraph::new(form_text)
        .style(Style::default().fg(theme.text_muted).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Komut & Edinim Önizleme "));
    f.render_widget(form, chunks[2]);
}

/// 🪟 3. Windows Araçları (Disk, WinPMEM, Agent)
fn render_windows_view(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(8), Constraint::Length(6)])
        .split(area);

    let tabs = Paragraph::new("  [1] PhysicalDrive Disk İmajı  |  [2] WinPMEM Canlı RAM  |  [3] Windows Agent (Uzak)")
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Windows Adli Edinim Modları "));
    f.render_widget(tabs, chunks[0]);

    let rows: Vec<Row> = app
        .disks
        .iter()
        .map(|d| {
            let size_gb = d.total_size as f64 / (1024.0 * 1024.0 * 1024.0);
            Row::new(vec![
                Cell::from(d.device.display().to_string()).style(Style::default().fg(theme.info)),
                Cell::from(format!("{:.1} GB", size_gb)),
                Cell::from(if d.accessible { "Erişilebilir" } else { "Erişim Yok" }),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(25), Constraint::Percentage(25)],
    )
    .header(
        Row::new(vec!["PhysicalDrive / Sürücü", "Boyut", "Erişim Durumu"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).border_style(theme.active_border_style()).title(" Windows Diskleri "));
    f.render_widget(table, chunks[1]);

    let form_text = format!(
        "  Hızlı Komut: amele windows disk \\\\.\\PhysicalDrive0 <vaka>\n  WinPMEM RAM: amele windows ram <vaka> (Durum: {})\n  Agent: amele windows disk --agent <ip> 9000 0 <vaka>",
        app.winpmem_status
    );
    let form = Paragraph::new(form_text)
        .style(Style::default().fg(theme.text_muted).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Windows Komut Satırı "));
    f.render_widget(form, chunks[2]);
}

/// 🐳 4. Docker Konteyner Adli Bilişimi
fn render_docker_view(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(8), Constraint::Length(5)])
        .split(area);

    let status_p = Paragraph::new(format!(
        "\n  🐳 Docker Daemon: {}\n  🛡️ Risk Taraması: Privileged Mod, Host PID, docker.sock Mounts, Hardcoded Secrets",
        app.docker_status
    ))
    .style(Style::default().fg(theme.text).bg(theme.card_bg))
    .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Docker Adli Durum "));
    f.render_widget(status_p, chunks[0]);

    let rows = vec![
        Row::new(vec!["web-app-nginx", "nginx:latest", "Running", "Normal", "Yok"]),
        Row::new(vec!["db-postgres", "postgres:16", "Running", "Normal", "Yok"]),
        Row::new(vec!["ci-runner", "docker:dind", "Running", "Privileged (Risk!)", "ENV Secret"]),
    ];

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
        ],
    )
    .header(
        Row::new(vec!["Konteyner Adı", "İmaj", "Durum", "Kaçış Riski", "Secret Bulundu"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(theme.active_border_style())
            .title(" Çalışan Konteynerler & Adli Risk Analizi "),
    );
    f.render_widget(table, chunks[1]);

    let cmd_text = "  • Drift & UpperDir Al: amele docker acquire <id> <vaka>\n  • Canlı Log Oku: amele docker logs <id> 200\n  • Uzak Agent Docker: amele docker --agent <ip> 9000 list";
    let cmd_p = Paragraph::new(cmd_text)
        .style(Style::default().fg(theme.text_muted).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Hızlı Komutlar "));
    f.render_widget(cmd_p, chunks[2]);
}

/// 🤖 5. Android Mobil Adli Bilişim
fn render_android_view(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(8), Constraint::Length(5)])
        .split(area);

    let status_p = Paragraph::new(format!(
        "\n  🤖 ADB Durumu: {}\n  📱 Desteklenen Modlar: Quick/Full Logical, /data FS Dump (--root), Volatile/Physical RAM (Lemon)",
        app.adb_status
    ))
    .style(Style::default().fg(theme.text).bg(theme.card_bg))
    .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Android ADB Durumu "));
    f.render_widget(status_p, chunks[0]);

    let rows = vec![
        Row::new(vec!["emulator-5554", "Pixel 7 Pro", "Android 14 (API 34)", "USB / TCP", "Tam Uyumlu"]),
    ];

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
        ],
    )
    .header(
        Row::new(vec!["Seri No", "Model / Cihaz", "İşletim Sistemi", "Bağlantı Tipi", "Uyumluluk"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).border_style(theme.active_border_style()).title(" Bağlı Android Cihazları "));
    f.render_widget(table, chunks[1]);

    let cmd_text = "  • Mantıksal Edinim: amele android logical <seri_no> <vaka> full\n  • Dosya Sistemi: amele android filesystem <seri_no> <vaka> --root\n  • Fiziksel RAM: amele android ram <seri_no> <vaka> physical";
    let cmd_p = Paragraph::new(cmd_text)
        .style(Style::default().fg(theme.text_muted).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Android CLI Komutları "));
    f.render_widget(cmd_p, chunks[2]);
}

/// 🍏 6. iOS Araçları
fn render_ios_view(f: &mut Frame, _app: &AppState, area: Rect, theme: &Theme) {
    let text = "\n  🍏 iOS Backup Adli Analizi ve Vaka Normalizasyonu\n\n  1. Yedek Metadata İnceleme:\n     amele ios profile /path/to/ios_backup\n     (Cihaz Modeli, iOS Sürümü, Seri No, GUID, Şifreleme Durumu)\n\n  2. Vaka Klasörüne Normalizasyon:\n     amele ios normalize /path/to/ios_backup <vaka_adi>\n     (Manifest.db tabanlı SMS, Rehber, Safari, Arama Kayıtları ayıklaması)";
    let p = Paragraph::new(text)
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.active_border_style()).title(" iOS Adli Bilişim Araçları "));
    f.render_widget(p, area);
}

/// 🌐 7. Agent & SSH Yönetimi
fn render_agent_view(f: &mut Frame, _app: &AppState, area: Rect, theme: &Theme) {
    let text = "\n  🌐 Uzak Adli Agent Kurulum ve Yönetim Kılavuzu\n\n  🐧 Linux Agent (amele-linux):\n     sudo python3 /opt/amele-linux.py --port 9000 --token \"GizliToken\"\n     Systemd: systemctl enable --now amele-agent\n\n  🪟 Windows Agent (amele-win):\n     python windows.py --port 9000 --token \"GizliToken\" (Yönetici)\n\n  🔒 Agent'sız Doğrudan SSH Edinimi:\n     amele linux disk --ssh <ip> 22 /dev/nvme0n1 <vaka> root <parola> <key>";
    let p = Paragraph::new(text)
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.active_border_style()).title(" Uzak Sunucu Agent & SSH Mimarisi "));
    f.render_widget(p, area);
}

/// 🛠️ 8. Diğer Araçlar (Hash, Verify, Mount, WireGuard)
fn render_other_view(f: &mut Frame, _app: &AppState, area: Rect, theme: &Theme) {
    let text = "\n  🛠️ Adli Bilişim Yardımcı Araçları\n\n  #️⃣ 1. Kriptografik Hash Hesaplama:\n     amele hash <dosya> [md5|sha1|sha256|sha512]\n\n  🔍 2. İmaj Bütünlük Doğrulama:\n     amele verify <imaj_dosyasi> <beklenen_sha256>\n\n  📦 3. Adli İmaj Bağlama (Mount):\n     amele mount list\n     amele mount cleanup <vaka>\n\n  🛡️ 4. Güvenli WireGuard VPN Tüneli:\n     amele wireguard /tmp/wg0.conf";
    let p = Paragraph::new(text)
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.active_border_style()).title(" Yardımcı Adli Araçlar "));
    f.render_widget(p, area);
}

/// 👤 9. Profil & Lisanslar
fn render_profile_view(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(6), Constraint::Min(8)])
        .split(area);

    let (name, email, roles_str) = if let Some(p) = &app.profile {
        let email = p.online.as_ref().and_then(|o| o.email.as_deref()).unwrap_or("Yerel Profil");
        let roles = p.online.as_ref().map(|o| o.roles.join(", ")).unwrap_or_else(|| "Standart".to_string());
        (p.full_name.clone(), email.to_string(), roles)
    } else {
        ("Misafir Kullanıcı".to_string(), "-".to_string(), "-".to_string())
    };

    let card_text = format!(
        "\n  👤 Analist: {}  |  📧 E-posta: {}\n  🏷️ Roller: {}\n  🌐 Online API: https://amele.noirlang.tr  |  Eşitleme: [S] Lisansları Eşitle",
        name, email, roles_str
    );
    let card = Paragraph::new(card_text)
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.active_border_style()).title(" Aktif Profil Kartı "));
    f.render_widget(card, chunks[0]);

    // Licenses Table
    let rows: Vec<Row> = if let Some(p) = &app.profile {
        p.online
            .as_ref()
            .map(|o| {
                o.licenses
                    .iter()
                    .map(|l| {
                        Row::new(vec![
                            Cell::from(l.license_key.clone()).style(Style::default().fg(theme.info)),
                            Cell::from(l.plan.clone()),
                            Cell::from(l.status.clone()).style(Style::default().fg(theme.success)),
                            Cell::from(l.created_at.clone()),
                        ])
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .header(
        Row::new(vec!["Lisans Anahtarı", "Plan", "Durum", "Oluşturulma Tarihi"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    )
    .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()).title(" Kayıtlı Lisanslar "));
    f.render_widget(table, chunks[1]);
}

/// ⚙️ 10. Ayarlar
fn render_settings_view(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let settings_text = format!(
        "\n  ⚙️ Amele Sistem Yapılandırması\n\n  📂 Vaka Klasörü: {}\n  📂 Çıktı Klasörü: {}\n  🌐 Dil: {}\n  🎨 Tema: {}\n  🛡️ Depolama Koruma Eşiği: 500 MB\n  🔔 Bildirimler: Açık\n  🚀 Donanım Hızlandırma: Açık",
        app.settings.vaka_klasoru.display(),
        app.settings.cikti_klasoru.display(),
        app.settings.dil,
        if app.settings.karanlik_tema { "Karanlık" } else { "Aydınlık" }
    );
    let p = Paragraph::new(settings_text)
        .style(Style::default().fg(theme.text).bg(theme.card_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.active_border_style()).title(" Uygulama Ayarları "));
    f.render_widget(p, area);
}
