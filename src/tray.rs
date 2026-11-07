use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub const WM_TRAYICON: u32 = WM_USER + 101;
pub const IDM_TRAY_OPEN: u32 = 2001;
pub const IDM_TRAY_PAUSE: u32 = 2002;
pub const IDM_TRAY_STARTUP: u32 = 2003;
pub const IDM_TRAY_SETTINGS: u32 = 2004;
pub const IDM_TRAY_QUIT: u32 = 2005;

pub struct TrayIcon {
    hwnd: HWND,
    nid: NOTIFYICONDATAW,
}

impl TrayIcon {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let mut nid = NOTIFYICONDATAW::default();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;

        unsafe {
            nid.hIcon = LoadIconW(None, IDI_APPLICATION)?;
        }

        let tip = "NotchBar (Ctrl+Alt+N)";
        let tip_u16: Vec<u16> = tip.encode_utf16().chain(std::iter::repeat(0)).take(128).collect();
        nid.szTip[..tip_u16.len()].copy_from_slice(&tip_u16);

        unsafe {
            let _ = Shell_NotifyIconW(NIM_ADD, &nid);
        }

        Ok(Self { hwnd, nid })
    }

    pub fn show_menu(&self) {
        unsafe {
            let mut pt = POINT::default();
            let _ = GetCursorPos(&mut pt);

            let hmenu = CreatePopupMenu().unwrap_or_default();
            if hmenu.0.is_null() {
                return;
            }

            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TRAY_OPEN as usize, w!("Open Notch (Ctrl+Alt+N)"));
            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TRAY_PAUSE as usize, w!("Pause Hover"));
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TRAY_STARTUP as usize, w!("Start with Windows"));
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TRAY_SETTINGS as usize, w!("Settings"));
            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, None);
            let _ = AppendMenuW(hmenu, MF_STRING, IDM_TRAY_QUIT as usize, w!("Quit NotchBar"));

            let _ = SetForegroundWindow(self.hwnd);
            let _ = TrackPopupMenu(
                hmenu,
                TPM_BOTTOMALIGN | TPM_RIGHTALIGN,
                pt.x,
                pt.y,
                0,
                self.hwnd,
                None,
            );
            let _ = DestroyMenu(hmenu);
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        unsafe {
            let _ = Shell_NotifyIconW(NIM_DELETE, &self.nid);
        }
    }
}
