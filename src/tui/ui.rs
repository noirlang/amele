//! Amele TUI Ana Arayüz Düzeni ve Render Motoru.

use crate::tui::app::{ActiveTab, AppState};
use crate::tui::theme::Theme;
use crate::tui::views::render_workspace;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;

pub fn render(f: &mut Frame, app: &AppState) {
    let theme = Theme::default();
    let size = f.area();

    // Split into Sidebar (Left) and Content Area (Right)
    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28), // Sidebar width
            Constraint::Min(60),    // Workspace width
        ])
        .split(size);

    render_sidebar(f, app, main_layout[0], &theme);
    render_main_panel(f, app, main_layout[1], &theme);

    // Render Toast Notification if present and young (< 4s)
    if let Some(notif) = &app.notification {
        if notif.created_at.elapsed().as_secs() < 4 {
            render_notification(f, notif, size, &theme);
        }
    }
}

/// Sol Menü Çubuğu (Sidebar)
fn render_sidebar(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Brand Logo & Title
            Constraint::Min(10),    // Navigation Items
            Constraint::Length(4),  // Profile & Version Footer
        ])
        .split(area);

    // 1. Logo & Brand
    let logo_text = "          ⣠⣧⡀\n         ⣰⠃⡏⢳⡀\n        ⡴⠁ ⡇ ⠳⡀\n       ⡼⠁ ⡼⠹⡄ ⠹⡄\n     ⢀⡜⠁⢀⡜⠁ ⠘⣆ ⠙⣆\n    ⢀⡞ ⢀⣀⣙⡦⠦⣞⣁⣀ ⠘⣆\n   ⢠⠎⠑⣤⣏⣉⣉⠑⡖⢉⣉⣉⣳⡔⠉⢆\n  ⢠⠿⣄⡰⠋⠳⣤⣤⣤⢧⣤⣤⡴⠋⠳⣀⡼⢧\n ⣰⠋ ⡼⠛⢦⡞      ⠘⣦⠞⠻⡄⠈⢳⡀\n⣰⣇⣀⣼⣁⣠⠞        ⠘⢦⣀⣹⣄⣀⣷⡀";

    let brand_p = Paragraph::new(format!("{}\n       AMELE FORENSIC", logo_text))
        .style(Style::default().fg(theme.accent).bg(theme.sidebar_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()));
    f.render_widget(brand_p, chunks[0]);

    // 2. Navigation Menu
    let items: Vec<ListItem> = ActiveTab::all()
        .into_iter()
        .map(|tab| {
            let is_active = tab == app.active_tab;
            let style = if is_active {
                Style::default()
                    .fg(Color::White)
                    .bg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };

            let line = format!(" [{}] {} {}", tab.shortcut(), tab.icon(), tab.title_tr());
            ListItem::new(line).style(style)
        })
        .collect();

    let nav_border = if app.sidebar_focused {
        theme.active_border_style()
    } else {
        theme.normal_border_style()
    };

    let menu = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(nav_border)
                .title(" Ana Menü [1-8, P, S] "),
        )
        .style(Style::default().bg(theme.sidebar_bg));
    f.render_widget(menu, chunks[1]);

    // 3. Profile & Version Footer
    let prof_display = app
        .profile
        .as_ref()
        .map(|p| format!("👤 {}", p.display_name))
        .unwrap_or_else(|| "👤 Misafir".to_string());

    let footer_text = format!(" {}\n v0.0.18 | Termina UI", prof_display);
    let footer_p = Paragraph::new(footer_text)
        .style(Style::default().fg(theme.text_muted).bg(theme.sidebar_bg))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()));
    f.render_widget(footer_p, chunks[2]);
}

/// Sağ İçerik Paneli (Top Bar + Workspace + Bottom Help)
fn render_main_panel(f: &mut Frame, app: &AppState, area: Rect, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Topbar Header
            Constraint::Min(10),   // Active Workspace View
            Constraint::Length(3), // Bottom Keybindings Footer
        ])
        .split(area);

    // 1. Topbar
    let header_text = format!(
        " 📂 {} > {}    |    🛡️ Depolama: {:.1} GB Boş    |    🕒 Canlı Adli Oturum",
        "Amele",
        app.active_tab.title_tr(),
        app.storage_free_gb
    );
    let topbar = Paragraph::new(header_text)
        .style(Style::default().fg(theme.text).bg(theme.card_bg).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()));
    f.render_widget(topbar, chunks[0]);

    // 2. Workspace View
    render_workspace(f, app, chunks[1], theme);

    // 3. Bottom Hotkeys
    let help_text = " [Tab] Odak Değiştir | [↑/↓] Gezin | [1-8, P, S] Sekme Seç | [Enter] İşlem | [Esc] Geri | [q] Çıkış ";
    let help_bar = Paragraph::new(help_text)
        .style(Style::default().fg(theme.text_muted).bg(theme.bg))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_style(theme.normal_border_style()));
    f.render_widget(help_bar, chunks[2]);
}

/// Toast Bildirimi (Overlay)
fn render_notification(
    f: &mut Frame,
    notif: &crate::tui::app::Notification,
    screen_area: Rect,
    theme: &Theme,
) {
    let width = 60.min(screen_area.width.saturating_sub(4));
    let height = 3;
    let x = (screen_area.width.saturating_sub(width)) / 2;
    let y = 2;
    let rect = Rect::new(x, y, width, height);

    f.render_widget(Clear, rect);

    let style = if notif.is_error {
        Style::default().fg(Color::White).bg(theme.danger).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White).bg(theme.accent).add_modifier(Modifier::BOLD)
    };

    let p = Paragraph::new(format!("  ℹ️ {}", notif.message))
        .style(style)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::White)));
    f.render_widget(p, rect);
}
