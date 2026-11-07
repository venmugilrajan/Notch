#![windows_subsystem = "windows"]
#![allow(dead_code)]

mod config;
mod modules;
mod renderer;
mod tray;
mod ui;
mod window;

use std::sync::{Arc, Mutex};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex};
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() -> Result<()> {
    // 1. Single Instance Check (Named Mutex)
    let mutex_name = w!("Global\\NotchBar_SingleInstance_Mutex_2026");
    let mutex = unsafe { CreateMutexW(None, true, mutex_name)? };
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe {
            if let Ok(existing_hwnd) = FindWindowW(w!("NotchBarWindowClass"), None) {
                if !existing_hwnd.0.is_null() {
                    let _ = PostMessageW(existing_hwnd, WM_COMMAND, WPARAM(1001), LPARAM(0));
                }
            }
        }
        return Ok(());
    }

    // 2. Set PerMonitorV2 DPI Awareness
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }

    // 3. Load Configuration
    let config = Arc::new(Mutex::new(config::AppConfig::load()));

    // 4. Create and run Window with Lazy Modules
    window::run_app(config)?;

    // Release mutex on clean shutdown
    unsafe {
        ReleaseMutex(mutex).ok();
        CloseHandle(mutex).ok();
    }

    Ok(())
}
