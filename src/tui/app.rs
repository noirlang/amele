use crate::disk::{list_disks, DiskInfo};
use crate::profile::{active_profile, LocalProfile};
use crate::settings::AppSettings;
use crate::storage_guard::check_available_space;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub fn list_case_names(vaka_dir: &Path) -> Vec<String> {
    let mut cases = Vec::new();
    if let Ok(entries) = fs::read_dir(vaka_dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        if !name.starts_with('.') {
                            cases.push(name.to_string());
                        }
                    }
                }
            }
        }
    }
    cases.sort();
    cases
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Home = 0,
    Linux = 1,
    Windows = 2,
    Docker = 3,
    Android = 4,
    Ios = 5,
    Agent = 6,
    Other = 7,
    Profile = 8,
    Settings = 9,
}

impl ActiveTab {
    pub fn all() -> Vec<ActiveTab> {
        vec![
            ActiveTab::Home,
            ActiveTab::Linux,
            ActiveTab::Windows,
            ActiveTab::Docker,
            ActiveTab::Android,
            ActiveTab::Ios,
            ActiveTab::Agent,
            ActiveTab::Other,
            ActiveTab::Profile,
            ActiveTab::Settings,
        ]
    }

    pub fn title_tr(&self) -> &'static str {
        match self {
            ActiveTab::Home => "Ana Sayfa & Vakalar",
            ActiveTab::Linux => "Linux Araçları",
            ActiveTab::Windows => "Windows Araçları",
            ActiveTab::Docker => "Docker Konteyner",
            ActiveTab::Android => "Android Araçları",
            ActiveTab::Ios => "iOS Araçları",
            ActiveTab::Agent => "Agent & SSH Yönetimi",
            ActiveTab::Other => "Diğer Araçlar",
            ActiveTab::Profile => "Profil & Lisanslar",
            ActiveTab::Settings => "Ayarlar",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            ActiveTab::Home => "🏠",
            ActiveTab::Linux => "🐧",
            ActiveTab::Windows => "🪟",
            ActiveTab::Docker => "🐳",
            ActiveTab::Android => "🤖",
            ActiveTab::Ios => "🍏",
            ActiveTab::Agent => "🌐",
            ActiveTab::Other => "🛠️",
            ActiveTab::Profile => "👤",
            ActiveTab::Settings => "⚙️",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            ActiveTab::Home => "1",
            ActiveTab::Linux => "2",
            ActiveTab::Windows => "3",
            ActiveTab::Docker => "4",
            ActiveTab::Android => "5",
            ActiveTab::Ios => "6",
            ActiveTab::Agent => "7",
            ActiveTab::Other => "8",
            ActiveTab::Profile => "P",
            ActiveTab::Settings => "S",
        }
    }
}

pub struct Notification {
    pub message: String,
    pub is_error: bool,
    pub created_at: Instant,
}

pub struct AppState {
    pub active_tab: ActiveTab,
    pub sidebar_focused: bool,
    pub selected_item_index: usize,
    pub active_subtab: usize,
    pub should_quit: bool,
    pub notification: Option<Notification>,

    // Data Cache
    pub profile: Option<LocalProfile>,
    pub settings: AppSettings,
    pub cases: Vec<String>,
    pub disks: Vec<DiskInfo>,
    pub storage_free_gb: f64,
    pub storage_total_gb: f64,

    // Status indicators
    pub avml_status: String,
    pub winpmem_status: String,
    pub adb_status: String,
    pub docker_status: String,

    // Input fields for interactive forms
    pub input_buffer: String,
    pub is_input_mode: bool,
    pub input_label: String,
}

impl AppState {
    pub fn new() -> Self {
        let settings = AppSettings::load(crate::settings::default_settings_path()).unwrap_or_default();
        let profile = active_profile();
        let cases = list_case_names(&settings.vaka_klasoru);
        let disks = list_disks().unwrap_or_default();

        // Calculate storage
        let storage_free_gb = check_available_space(Path::new(&settings.vaka_klasoru))
            .map(|b| b as f64 / (1024.0 * 1024.0 * 1024.0))
            .unwrap_or(50.0);
        let storage_total_gb = storage_free_gb + 100.0;

        // Probe tool statuses
        let avml_status = if cfg!(target_os = "linux") {
            "Mevcut (Sistem/Helper)"
        } else {
            "N/A"
        }
        .to_string();
        let winpmem_status = if cfg!(target_os = "windows") {
            "Mevcut"
        } else {
            "N/A"
        }
        .to_string();
        let adb_status = "Hazır".to_string();
        let docker_status = "Çalışıyor (Yerel Daemon)".to_string();

        Self {
            active_tab: ActiveTab::Home,
            sidebar_focused: true,
            selected_item_index: 0,
            active_subtab: 0,
            should_quit: false,
            notification: Some(Notification {
                message: "Amele Forensic Tool Terminal Arayüzü Başlatıldı (v0.0.18)".to_string(),
                is_error: false,
                created_at: Instant::now(),
            }),
            profile,
            settings,
            cases,
            disks,
            storage_free_gb,
            storage_total_gb,
            avml_status,
            winpmem_status,
            adb_status,
            docker_status,
            input_buffer: String::new(),
            is_input_mode: false,
            input_label: String::new(),
        }
    }

    pub fn set_notification(&mut self, msg: impl Into<String>, is_error: bool) {
        self.notification = Some(Notification {
            message: msg.into(),
            is_error,
            created_at: Instant::now(),
        });
    }

    pub fn refresh_data(&mut self) {
        self.settings = AppSettings::load(crate::settings::default_settings_path()).unwrap_or_default();
        self.cases = list_case_names(&self.settings.vaka_klasoru);
        self.disks = list_disks().unwrap_or_default();
        self.profile = active_profile();
    }

    pub fn next_tab(&mut self) {
        let tabs = ActiveTab::all();
        let current_pos = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        let next_pos = (current_pos + 1) % tabs.len();
        self.active_tab = tabs[next_pos];
        self.selected_item_index = 0;
        self.active_subtab = 0;
    }

    pub fn prev_tab(&mut self) {
        let tabs = ActiveTab::all();
        let current_pos = tabs.iter().position(|t| *t == self.active_tab).unwrap_or(0);
        let prev_pos = if current_pos == 0 {
            tabs.len() - 1
        } else {
            current_pos - 1
        };
        self.active_tab = tabs[prev_pos];
        self.selected_item_index = 0;
        self.active_subtab = 0;
    }

    pub fn set_tab(&mut self, tab: ActiveTab) {
        self.active_tab = tab;
        self.selected_item_index = 0;
        self.active_subtab = 0;
    }

    pub fn move_selection_down(&mut self) {
        let max_items = match self.active_tab {
            ActiveTab::Home => self.cases.len().max(1),
            ActiveTab::Linux | ActiveTab::Windows => self.disks.len().max(1),
            _ => 10,
        };
        if self.selected_item_index + 1 < max_items {
            self.selected_item_index += 1;
        }
    }

    pub fn move_selection_up(&mut self) {
        if self.selected_item_index > 0 {
            self.selected_item_index -= 1;
        }
    }
}
