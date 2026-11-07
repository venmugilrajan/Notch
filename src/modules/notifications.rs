use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use std::time::Instant;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Storage::FileSystem::*;
use windows::Win32::System::Power::*;

#[derive(Clone, Debug)]
pub struct ToastNotification {
    pub title: String,
    pub subtitle: String,
    pub icon: &'static str,
    pub created_at: Instant,
    pub duration_secs: f32,
}

pub struct NotificationCenter {
    pub active_toast: Option<ToastNotification>,
    screenshot_rx: Receiver<String>,
    battery_rx: Receiver<String>,
}

impl NotificationCenter {
    pub fn new() -> Self {
        let (stx, srx) = channel();
        let (btx, brx) = channel();

        // 1. Screenshots folder watcher using ReadDirectoryChangesW
        thread::spawn(move || {
            watch_screenshots_folder(stx);
        });

        // 2. Power & Battery watcher
        thread::spawn(move || {
            watch_battery_status(btx);
        });

        Self {
            active_toast: None,
            screenshot_rx: srx,
            battery_rx: brx,
        }
    }

    pub fn push(&mut self, title: String, subtitle: String, icon: &'static str) {
        self.active_toast = Some(ToastNotification {
            title,
            subtitle,
            icon,
            created_at: Instant::now(),
            duration_secs: 2.2,
        });
    }

    pub fn poll_updates(&mut self) -> bool {
        let mut updated = false;

        while let Ok(path) = self.screenshot_rx.try_recv() {
            let filename = PathBuf::from(path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Image.png".to_string());
            self.push("Screenshot Saved".to_string(), filename, "📸");
            updated = true;
        }

        while let Ok(msg) = self.battery_rx.try_recv() {
            self.push("Power Status".to_string(), msg, "⚡");
            updated = true;
        }

        if let Some(toast) = &self.active_toast {
            if toast.created_at.elapsed().as_secs_f32() > toast.duration_secs {
                self.active_toast = None;
                updated = true;
            }
        }

        updated
    }
}

fn watch_screenshots_folder(tx: std::sync::mpsc::Sender<String>) {
    let pictures_dir = std::env::var("USERPROFILE").unwrap_or_default();
    let screenshots_dir = PathBuf::from(pictures_dir).join("Pictures").join("Screenshots");
    if !screenshots_dir.exists() {
        return;
    }

    let wide_path: Vec<u16> = screenshots_dir
        .to_str()
        .unwrap_or_default()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let hdir = CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            FILE_LIST_DIRECTORY.0,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            None,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            HANDLE(std::ptr::null_mut()),
        );

        if let Ok(handle) = hdir {
            let mut buffer = [0u8; 1024];
            let mut bytes_returned = 0u32;

            loop {
                let success = ReadDirectoryChangesW(
                    handle,
                    buffer.as_mut_ptr() as *mut _,
                    buffer.len() as u32,
                    false,
                    FILE_NOTIFY_CHANGE_FILE_NAME,
                    Some(&mut bytes_returned),
                    None,
                    None,
                );

                if success.is_ok() && bytes_returned > 0 {
                    let _ = tx.send("New Screenshot".to_string());
                }

                thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
}

fn watch_battery_status(tx: std::sync::mpsc::Sender<String>) {
    let mut last_charging = false;
    let mut last_low = false;

    loop {
        unsafe {
            let mut status = SYSTEM_POWER_STATUS::default();
            if GetSystemPowerStatus(&mut status).is_ok() {
                let is_charging = status.ACLineStatus == 1;
                let battery_pct = status.BatteryLifePercent;

                if is_charging && !last_charging {
                    let _ = tx.send(format!("Charging started ({}%)", battery_pct));
                }
                last_charging = is_charging;

                let is_low = battery_pct < 20 && !is_charging;
                if is_low && !last_low {
                    let _ = tx.send(format!("Battery low ({}%)", battery_pct));
                }
                last_low = is_low;
            }
        }
        thread::sleep(std::time::Duration::from_secs(5));
    }
}
