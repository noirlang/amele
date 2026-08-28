//! Amele Terminal Kullanıcı Arayüzü (TUI) Modülü.

pub mod app;
pub mod theme;
pub mod ui;
pub mod views;

use app::{ActiveTab, AppState};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::Duration;

/// TUI modunu başlatır ve terminal yaşam döngüsünü yönetir.
pub fn run_tui() -> Result<(), String> {
    // 1. Terminali Raw moda geçir
    enable_raw_mode().map_err(|e| format!("Raw mod etkinleştirilemedi: {}", e))?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .map_err(|e| format!("Alternate screen açılamadı: {}", e))?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)
        .map_err(|e| format!("Terminal motoru başlatılamadı: {}", e))?;

    let mut app = AppState::new();

    // 2. Olay ve Çizim Döngüsü
    let res = run_loop(&mut terminal, &mut app);

    // 3. Terminali eski haline geri getir
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal.show_cursor();

    res.map_err(|e| format!("TUI çalışma hatası: {}", e))
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| ui::render(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Global hotkeys
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                    break;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Tab => {
                        app.sidebar_focused = !app.sidebar_focused;
                    }
                    KeyCode::BackTab => {
                        app.sidebar_focused = !app.sidebar_focused;
                    }
                    // Direct Tab Hotkeys
                    KeyCode::Char('1') => app.set_tab(ActiveTab::Home),
                    KeyCode::Char('2') => app.set_tab(ActiveTab::Linux),
                    KeyCode::Char('3') => app.set_tab(ActiveTab::Windows),
                    KeyCode::Char('4') => app.set_tab(ActiveTab::Docker),
                    KeyCode::Char('5') => app.set_tab(ActiveTab::Android),
                    KeyCode::Char('6') => app.set_tab(ActiveTab::Ios),
                    KeyCode::Char('7') => app.set_tab(ActiveTab::Agent),
                    KeyCode::Char('8') => app.set_tab(ActiveTab::Other),
                    KeyCode::Char('p') | KeyCode::Char('P') => app.set_tab(ActiveTab::Profile),
                    KeyCode::Char('s') | KeyCode::Char('S') => app.set_tab(ActiveTab::Settings),
                    KeyCode::Left | KeyCode::Char('h') => {
                        app.prev_tab();
                    }
                    KeyCode::Right | KeyCode::Char('l') => {
                        app.next_tab();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if app.sidebar_focused {
                            app.next_tab();
                        } else {
                            app.move_selection_down();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if app.sidebar_focused {
                            app.prev_tab();
                        } else {
                            app.move_selection_up();
                        }
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        app.refresh_data();
                        app.set_notification("Veriler yenilendi", false);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
