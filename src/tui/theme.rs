//! Amele TUI Tema ve Renk Paleti (Desktop Dark UI ile birebir uyumlu).

use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub bg: Color,
    pub sidebar_bg: Color,
    pub card_bg: Color,
    pub border: Color,
    pub border_active: Color,
    pub text: Color,
    pub text_muted: Color,
    pub accent: Color,
    pub accent_hover: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub info: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Color::Rgb(15, 17, 23),
            sidebar_bg: Color::Rgb(20, 23, 31),
            card_bg: Color::Rgb(26, 30, 41),
            border: Color::Rgb(45, 52, 70),
            border_active: Color::Rgb(99, 102, 241),
            text: Color::Rgb(241, 245, 249),
            text_muted: Color::Rgb(148, 163, 184),
            accent: Color::Rgb(99, 102, 241), // Indigo
            accent_hover: Color::Rgb(129, 140, 248),
            success: Color::Rgb(34, 197, 94), // Green
            warning: Color::Rgb(234, 179, 8), // Yellow/Amber
            danger: Color::Rgb(239, 68, 68),  // Red
            info: Color::Rgb(56, 189, 248),   // Sky blue
        }
    }
}

impl Theme {
    pub fn title_style(&self) -> Style {
        Style::default()
            .fg(self.text)
            .add_modifier(Modifier::BOLD)
    }

    pub fn active_tab_style(&self) -> Style {
        Style::default()
            .fg(Color::Rgb(255, 255, 255))
            .bg(self.accent)
            .add_modifier(Modifier::BOLD)
    }

    pub fn inactive_tab_style(&self) -> Style {
        Style::default().fg(self.text_muted)
    }

    pub fn active_border_style(&self) -> Style {
        Style::default().fg(self.border_active)
    }

    pub fn normal_border_style(&self) -> Style {
        Style::default().fg(self.border)
    }

    pub fn badge_success(&self) -> Style {
        Style::default().fg(self.success).add_modifier(Modifier::BOLD)
    }

    pub fn badge_warning(&self) -> Style {
        Style::default().fg(self.warning).add_modifier(Modifier::BOLD)
    }

    pub fn badge_danger(&self) -> Style {
        Style::default().fg(self.danger).add_modifier(Modifier::BOLD)
    }

    pub fn badge_info(&self) -> Style {
        Style::default().fg(self.info).add_modifier(Modifier::BOLD)
    }
}
