use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub collapsed_width: i32,
    pub collapsed_height: i32,
    pub expanded_width: i32,
    pub expanded_height: i32,
    pub hover_delay_ms: u32,
    pub auto_collapse_delay_ms: u32,
    pub hotkey_modifiers: u32, // MOD_CONTROL | MOD_ALT
    pub hotkey_vk: u32,        // VK_N
    pub theme: String,         // "Pure Black", "Ember Dark", "Vivid", "Silver"
    pub outline_enabled: bool,
    pub outline_width: u32,
    pub user_name: String,
    pub start_with_windows: bool,
    pub primary_monitor_only: bool,
    pub clipboard_history_limit: usize,
    pub clipboard_images_limit: usize,
    pub enabled_modules: EnabledModules,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledModules {
    pub home: bool,
    pub system: bool,
    pub clock: bool,
    pub timers: bool,
    pub calendar: bool,
    pub weather: bool,
    pub clipboard: bool,
    pub notes: bool,
    pub file_shelf: bool,
    pub drop_transfer: bool,
    pub converters: bool,
    pub links: bool,
    pub media: bool,
}

impl EnabledModules {
    pub fn is_enabled(&self, tab: crate::renderer::NotchTab) -> bool {
        match tab {
            crate::renderer::NotchTab::Home => self.home,
            crate::renderer::NotchTab::System => self.system,
            crate::renderer::NotchTab::Clock => self.clock,
            crate::renderer::NotchTab::Timers => self.timers,
            crate::renderer::NotchTab::Calendar => self.calendar,
            crate::renderer::NotchTab::Weather => self.weather,
            crate::renderer::NotchTab::Clipboard => self.clipboard,
            crate::renderer::NotchTab::Notes => self.notes,
            crate::renderer::NotchTab::FileShelf => self.file_shelf,
            crate::renderer::NotchTab::DropTransfer => self.drop_transfer,
            crate::renderer::NotchTab::Converters => self.converters,
            crate::renderer::NotchTab::Links => self.links,
            crate::renderer::NotchTab::Media => self.media,
            crate::renderer::NotchTab::Diagnostics => true,
        }
    }
}
impl Default for EnabledModules {
    fn default() -> Self {
        Self {
            home: true,
            system: true,
            clock: true,
            timers: true,
            calendar: true,
            weather: true,
            clipboard: true,
            notes: true,
            file_shelf: true,
            drop_transfer: true,
            converters: true,
            links: true,
            media: true,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            collapsed_width: 200,
            collapsed_height: 32,
            expanded_width: 760,
            expanded_height: 340,
            hover_delay_ms: 150,
            auto_collapse_delay_ms: 400,
            hotkey_modifiers: 0x0002 | 0x0001, // MOD_CONTROL | MOD_ALT
            hotkey_vk: 0x4E,                  // 'N'
            theme: "Pure Black".to_string(),
            outline_enabled: true,
            outline_width: 1,
            user_name: "User".to_string(),
            start_with_windows: false,
            primary_monitor_only: true,
            clipboard_history_limit: 50,
            clipboard_images_limit: 10,
            enabled_modules: EnabledModules::default(),
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let appdata = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
        let dir = PathBuf::from(appdata).join("NotchBar");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str(&data) {
                return cfg;
            }
        }
        let cfg = Self::default();
        let _ = cfg.save();
        cfg
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}
