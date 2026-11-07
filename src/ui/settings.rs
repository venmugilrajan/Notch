use crate::config::AppConfig;
use crate::window::WM_SETTINGS_CHANGED;
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, Mutex};
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
use windows::Win32::UI::WindowsAndMessaging::*;

const SETTINGS_WINDOW_CLASS: PCWSTR = w!("NotchBarSettingsWindowClass");

pub struct SettingsWindowContext {
    pub config: Arc<Mutex<AppConfig>>,
    pub active_page: usize,
    pub search_query: String,
    pub width: i32,
    pub height: i32,
}

pub struct SettingsWindow;

static SETTINGS_HWND_RAW: AtomicIsize = AtomicIsize::new(0);

impl SettingsWindow {
    pub fn open_or_focus(config: Arc<Mutex<AppConfig>>) -> Result<()> {
        let raw = SETTINGS_HWND_RAW.load(Ordering::SeqCst);
        if raw != 0 {
            let existing_hwnd = HWND(raw as *mut core::ffi::c_void);
            unsafe {
                if IsWindow(existing_hwnd).as_bool() {
                    let _ = ShowWindow(existing_hwnd, SW_SHOW);
                    let _ = SetForegroundWindow(existing_hwnd);
                    return Ok(());
                }
            }
        }

        unsafe {
            let instance = GetModuleHandleW(None)?;

            let wnd_class = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(settings_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: instance.into(),
                hIcon: LoadIconW(None, IDI_APPLICATION)?,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
                lpszMenuName: PCWSTR::null(),
                lpszClassName: SETTINGS_WINDOW_CLASS,
                hIconSm: HICON(std::ptr::null_mut()),
            };

            let _ = RegisterClassExW(&wnd_class);

            let screen_w = GetSystemMetrics(SM_CXSCREEN);
            let screen_h = GetSystemMetrics(SM_CYSCREEN);
            let win_w = 900;
            let win_h = 560;
            let x = (screen_w - win_w) / 2;
            let y = (screen_h - win_h) / 2;

            let hwnd = CreateWindowExW(
                WS_EX_APPWINDOW,
                SETTINGS_WINDOW_CLASS,
                w!("NotchBar Settings"),
                WS_POPUP | WS_VISIBLE | WS_MINIMIZEBOX,
                x,
                y,
                win_w,
                win_h,
                HWND(std::ptr::null_mut()),
                HMENU(std::ptr::null_mut()),
                instance,
                None,
            )?;

            // Rounded window frame (24px radius)
            let rgn = CreateRoundRectRgn(0, 0, win_w, win_h, 24, 24);
            let _ = SetWindowRgn(hwnd, rgn, true);

            let ctx = Box::new(SettingsWindowContext {
                config,
                active_page: 0,
                search_query: String::new(),
                width: win_w,
                height: win_h,
            });

            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(ctx) as isize);
            SETTINGS_HWND_RAW.store(hwnd.0 as isize, Ordering::SeqCst);

            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetForegroundWindow(hwnd);
        }

        Ok(())
    }
}

unsafe extern "system" fn settings_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let ctx_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut SettingsWindowContext;

    match msg {
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);
            if !ctx_ptr.is_null() {
                let ctx = &mut *ctx_ptr;
                draw_settings_ui(hdc, ctx);
            }
            let _ = EndPaint(hwnd, &ps);
            LRESULT(0)
        }
        WM_LBUTTONDOWN => {
            let x = (lparam.0 & 0xFFFF) as i32;
            let y = ((lparam.0 >> 16) & 0xFFFF) as i32;

            if !ctx_ptr.is_null() {
                let ctx = &mut *ctx_ptr;

                // Close button (top right: 860..890, 10..40)
                if x >= ctx.width - 40 && y <= 40 {
                    let _ = DestroyWindow(hwnd);
                    return LRESULT(0);
                }
                // Minimize button (820..855, 10..40)
                if x >= ctx.width - 80 && x < ctx.width - 40 && y <= 40 {
                    let _ = ShowWindow(hwnd, SW_MINIMIZE);
                    return LRESULT(0);
                }

                // Window Drag on title bar (x > 220, y <= 40)
                if x > 220 && y <= 40 {
                    let _ = ReleaseCapture();
                    let _ = SendMessageW(hwnd, WM_NCLBUTTONDOWN, WPARAM(2), LPARAM(0)); // HTCAPTION = 2
                    return LRESULT(0);
                }

                // Nav sidebar click (x <= 220, 8 pages: General, Notch, Appearance, Modules, Live Activities, Privacy, Updates, About)
                if x <= 220 && y >= 105 && y <= 105 + (8 * 42) {
                    let page_idx = (y - 105) / 42;
                    if page_idx >= 0 && page_idx < 8 {
                        ctx.active_page = page_idx as usize;
                        let _ = InvalidateRect(hwnd, None, false);
                        return LRESULT(0);
                    }
                }

                // Right side interactive elements
                handle_settings_click(hwnd, ctx, x, y);
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            if !ctx_ptr.is_null() {
                let _ = Box::from_raw(ctx_ptr);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            SETTINGS_HWND_RAW.store(0, Ordering::SeqCst);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn handle_settings_click(hwnd: HWND, ctx: &mut SettingsWindowContext, x: i32, y: i32) {
    let mut cfg = ctx.config.lock().unwrap();
    let mut changed = false;

    match ctx.active_page {
        0 => {
            // Page 0: General
            // Row 1: Start with Windows at y: 135..185
            if y >= 135 && y <= 185 && x >= 740 && x <= 830 {
                cfg.start_with_windows = !cfg.start_with_windows;
                let _ = cfg.save();
                changed = true;
            }
            // Row 2: Attach to Primary Monitor at y: 200..250
            if y >= 200 && y <= 250 && x >= 740 && x <= 830 {
                cfg.primary_monitor_only = !cfg.primary_monitor_only;
                let _ = cfg.save();
                changed = true;
            }
        }
        1 => {
            // Page 1: Notch
            // Size chips at y: 145..195 (Small: 200x32, Default: 200x32, Large: 240x36)
            if y >= 145 && y <= 195 {
                let sizes = [
                    (180, 30, 720, 320),
                    (200, 32, 760, 340),
                    (240, 36, 820, 380),
                ];
                let chip_w = 110;
                for (i, (cw, ch, ew, eh)) in sizes.iter().enumerate() {
                    let cx = 256 + (i as i32 * (chip_w + 14));
                    if x >= cx && x <= cx + chip_w {
                        cfg.collapsed_width = *cw;
                        cfg.collapsed_height = *ch;
                        cfg.expanded_width = *ew;
                        cfg.expanded_height = *eh;
                        let _ = cfg.save();
                        changed = true;
                        break;
                    }
                }
            }
        }
        2 => {
            // Page 2: Appearance & Themes (Feature 6 & Feature 7)
            // 2x2 Preview Grid for Themes
            // Row 0: y: 140..220 | Row 1: y: 236..316
            let card_w = 260;
            let card_h = 76;
            let themes = [
                ("Pure Black", 256, 140),
                ("Ember Dark", 532, 140),
                ("Vivid", 256, 230),
                ("Silver", 532, 230),
            ];

            for (theme_name, tx, ty) in themes.iter() {
                if x >= *tx && x <= *tx + card_w && y >= *ty && y <= *ty + card_h {
                    cfg.theme = theme_name.to_string();
                    let _ = cfg.save();
                    changed = true;
                    break;
                }
            }

            // Outline Toggle at y: 360..395, x: 740..830
            if y >= 360 && y <= 395 && x >= 740 && x <= 830 {
                cfg.outline_enabled = !cfg.outline_enabled;
                let _ = cfg.save();
                changed = true;
            }

            // Outline Width Segmented Control (1px, 2px, 3px) at y: 410..445
            if y >= 410 && y <= 445 && cfg.outline_enabled {
                let widths = [1, 2, 3];
                for (i, w) in widths.iter().enumerate() {
                    let bx = 680 + (i as i32 * 46);
                    if x >= bx && x <= bx + 42 {
                        cfg.outline_width = *w;
                        let _ = cfg.save();
                        changed = true;
                        break;
                    }
                }
            }
        }
        3 => {
            // Page 3: Modules Management
            // 8 Modules toggles row by row starting at y: 140
            for i in 0..8 {
                let row_y = 140 + (i * 38);
                if y >= row_y && y <= row_y + 32 && x >= 740 && x <= 830 {
                    match i {
                        0 => cfg.enabled_modules.home = !cfg.enabled_modules.home,
                        1 => cfg.enabled_modules.system = !cfg.enabled_modules.system,
                        2 => cfg.enabled_modules.clock = !cfg.enabled_modules.clock,
                        3 => cfg.enabled_modules.timers = !cfg.enabled_modules.timers,
                        4 => cfg.enabled_modules.calendar = !cfg.enabled_modules.calendar,
                        5 => cfg.enabled_modules.weather = !cfg.enabled_modules.weather,
                        6 => cfg.enabled_modules.clipboard = !cfg.enabled_modules.clipboard,
                        7 => cfg.enabled_modules.notes = !cfg.enabled_modules.notes,
                        _ => {}
                    }
                    let _ = cfg.save();
                    changed = true;
                    break;
                }
            }
        }
        _ => {}
    }

    if changed {
        // 1. Repaint Settings window immediately
        unsafe {
            let _ = InvalidateRect(hwnd, None, false);
        }
        // 2. Notify Notch window to update live theme, sizes, and layout instantly
        unsafe {
            if let Ok(notch_hwnd) = FindWindowW(w!("NotchBarWindowClass"), None) {
                if !notch_hwnd.0.is_null() {
                    let _ = PostMessageW(notch_hwnd, WM_SETTINGS_CHANGED, WPARAM(0), LPARAM(0));
                }
            }
        }
    }
}

fn draw_settings_ui(hdc: HDC, ctx: &SettingsWindowContext) {
    unsafe {
        // Double buffer to eliminate flicker
        let mem_dc = CreateCompatibleDC(hdc);
        let mem_bmp = CreateCompatibleBitmap(hdc, ctx.width, ctx.height);
        let _ = SelectObject(mem_dc, mem_bmp);

        // Fill background with #101010
        let bg_brush = CreateSolidBrush(COLORREF(0x00101010));
        let full_rect = RECT { left: 0, top: 0, right: ctx.width, bottom: ctx.height };
        let _ = FillRect(mem_dc, &full_rect, bg_brush);
        let _ = DeleteObject(bg_brush);

        // Left sidebar background (220px wide, #141414)
        let sidebar_brush = CreateSolidBrush(COLORREF(0x00141414));
        let sidebar_rect = RECT { left: 0, top: 0, right: 220, bottom: ctx.height };
        let _ = FillRect(mem_dc, &sidebar_rect, sidebar_brush);
        let _ = DeleteObject(sidebar_brush);

        // Sidebar 1px right border #242424
        let border_pen = CreatePen(PS_SOLID, 1, COLORREF(0x00242424));
        let old_pen = SelectObject(mem_dc, border_pen);
        let _ = MoveToEx(mem_dc, 220, 0, None);
        let _ = LineTo(mem_dc, 220, ctx.height);

        // Window Controls (Minimize & Close buttons)
        let close_pen = CreatePen(PS_SOLID, 2, COLORREF(0x00888888));
        let _ = SelectObject(mem_dc, close_pen);
        // Minimize horizontal line at (835, 24)
        let _ = MoveToEx(mem_dc, 830, 24, None);
        let _ = LineTo(mem_dc, 846, 24);
        // Close cross at (870, 20)
        let _ = MoveToEx(mem_dc, 866, 18, None);
        let _ = LineTo(mem_dc, 878, 30);
        let _ = MoveToEx(mem_dc, 878, 18, None);
        let _ = LineTo(mem_dc, 866, 30);
        let _ = DeleteObject(close_pen);

        SetBkMode(mem_dc, TRANSPARENT);

        // App Logo & Title in sidebar
        SetTextColor(mem_dc, COLORREF(0x00FFFFFF));
        draw_gdi_text(mem_dc, "NotchBar", 52, 22, 17, true);
        SetTextColor(mem_dc, COLORREF(0x00888888));
        draw_gdi_text(mem_dc, "v0.1.0", 136, 25, 12, false);

        // Sidebar Search Box at (16, 62)
        let search_rect = RECT { left: 16, top: 60, right: 204, bottom: 92 };
        let search_bg = CreateSolidBrush(COLORREF(0x001F1F1F));
        let _ = FillRect(mem_dc, &search_rect, search_bg);
        let _ = DeleteObject(search_bg);
        SetTextColor(mem_dc, COLORREF(0x00777777));
        draw_gdi_text(mem_dc, "Search    Ctrl K", 28, 68, 12, false);

        // 8 Navigation Items per Feature 6 spec:
        // General, Notch, Appearance, Modules, Live Activities, Privacy, Updates, About
        let nav_items = [
            ("General", 0x003B82F6),        // Blue
            ("Notch", 0x0010B981),          // Green
            ("Appearance", 0x00EC4899),     // Pink
            ("Modules", 0x00F59E0B),        // Amber
            ("Live Activities", 0x008B5CF6),// Purple
            ("Privacy", 0x0064748B),        // Slate
            ("Updates", 0x0006B6D4),        // Cyan
            ("About", 0x0094A3B8),          // Grey
        ];

        let mut nav_y = 105;
        for (i, (label, color)) in nav_items.iter().enumerate() {
            let is_active = ctx.active_page == i;
            if is_active {
                let item_rect = RECT { left: 12, top: nav_y, right: 208, bottom: nav_y + 36 };
                let active_brush = CreateSolidBrush(COLORREF(0x00262626));
                let _ = FillRect(mem_dc, &item_rect, active_brush);
                let _ = DeleteObject(active_brush);
            }

            // Rounded color badge
            let dot_brush = CreateSolidBrush(COLORREF(*color));
            let dot_rect = RECT { left: 24, top: nav_y + 11, right: 38, bottom: nav_y + 25 };
            let _ = FillRect(mem_dc, &dot_rect, dot_brush);
            let _ = DeleteObject(dot_brush);

            let txt_color = if is_active { 0x00FFFFFF } else { 0x00AAAAAA };
            SetTextColor(mem_dc, COLORREF(txt_color));
            draw_gdi_text(mem_dc, label, 48, nav_y + 9, 13, is_active);

            nav_y += 42;
        }

        // Sidebar Footer
        SetTextColor(mem_dc, COLORREF(0x00555555));
        draw_gdi_text(mem_dc, "Help  ·  Feedback  ·  v0.1.0", 24, ctx.height - 30, 11, false);

        // Right Content Area
        draw_settings_page_content(mem_dc, ctx);

        let _ = SelectObject(mem_dc, old_pen);
        let _ = DeleteObject(border_pen);

        // BitBlt final buffer
        let _ = BitBlt(hdc, 0, 0, ctx.width, ctx.height, mem_dc, 0, 0, SRCCOPY);
        let _ = DeleteDC(mem_dc);
        let _ = DeleteObject(mem_bmp);
    }
}

fn draw_settings_page_content(hdc: HDC, ctx: &SettingsWindowContext) {
    unsafe {
        let content_x = 256;
        let cfg = ctx.config.lock().unwrap();

        match ctx.active_page {
            0 => {
                // Page 0: General
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                draw_gdi_text(hdc, "General", content_x, 32, 22, true);
                SetTextColor(hdc, COLORREF(0x00888888));
                draw_gdi_text(hdc, "Manage app startup, positioning, and global shortcuts.", content_x, 62, 13, false);

                draw_section_header(hdc, "STARTUP & SHORTCUTS", content_x, 105);
                draw_toggle_row(hdc, "Start with Windows", "Automatically launch NotchBar upon system boot.", cfg.start_with_windows, content_x, 135);
                draw_toggle_row(hdc, "Attach to Primary Monitor", "Pin notch display strictly to the primary desktop screen.", cfg.primary_monitor_only, content_x, 200);

                draw_section_header(hdc, "HOTKEY", content_x, 275);
                draw_info_row(hdc, "Global Hotkey", "Ctrl + Alt + N", content_x, 305);
            }
            1 => {
                // Page 1: Notch
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                draw_gdi_text(hdc, "Notch Layout & Timing", content_x, 32, 22, true);
                SetTextColor(hdc, COLORREF(0x00888888));
                draw_gdi_text(hdc, "Configure physical dimensions, hover responsiveness, and collapse timing.", content_x, 62, 13, false);

                draw_section_header(hdc, "PHYSICAL SIZE PRESETS", content_x, 105);
                let sizes = [
                    ("Small (180 px)", 180),
                    ("Default (200 px)", 200),
                    ("Large (240 px)", 240),
                ];
                let chip_w = 120;
                for (i, (s_label, cw)) in sizes.iter().enumerate() {
                    let chip_x = content_x + (i as i32 * (chip_w + 14));
                    let is_cur = cfg.collapsed_width == *cw;
                    let bg_col = if is_cur { 0x002B2B2B } else { 0x001A1A1A };
                    let border_col = if is_cur { 0x004A90E2 } else { 0x002E2E2E };

                    let brush = CreateSolidBrush(COLORREF(bg_col));
                    let r = RECT { left: chip_x, top: 145, right: chip_x + chip_w, bottom: 185 };
                    let _ = FillRect(hdc, &r, brush);
                    let _ = DeleteObject(brush);

                    let pen = CreatePen(PS_SOLID, 1, COLORREF(border_col));
                    let old_p = SelectObject(hdc, pen);
                    let _ = MoveToEx(hdc, chip_x, 145, None);
                    let _ = LineTo(hdc, chip_x + chip_w, 145);
                    let _ = LineTo(hdc, chip_x + chip_w, 185);
                    let _ = LineTo(hdc, chip_x, 185);
                    let _ = LineTo(hdc, chip_x, 145);
                    let _ = SelectObject(hdc, old_p);
                    let _ = DeleteObject(pen);

                    let txt_col = if is_cur { 0x00FFFFFF } else { 0x00888888 };
                    SetTextColor(hdc, COLORREF(txt_col));
                    draw_gdi_text(hdc, s_label, chip_x + 10, 158, 12, is_cur);
                }

                draw_section_header(hdc, "TIMINGS", content_x, 215);
                draw_info_row(hdc, "Hover Delay", &format!("{} ms", cfg.hover_delay_ms), content_x, 245);
                draw_info_row(hdc, "Auto Collapse Delay", &format!("{} ms", cfg.auto_collapse_delay_ms), content_x, 305);
            }
            2 => {
                // Page 2: Appearance & Themes (Feature 6 & Feature 7)
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                draw_gdi_text(hdc, "Appearance & Themes", content_x, 32, 22, true);
                SetTextColor(hdc, COLORREF(0x00888888));
                draw_gdi_text(hdc, "Switch themes live and configure notch outline styling.", content_x, 62, 13, false);

                draw_section_header(hdc, "THEMES (2×2 PREVIEW GRID)", content_x, 105);

                // 2x2 grid preview cards
                let themes = [
                    ("Pure Black", "Monochrome on true black", 0x00FFFFFF, 256, 140),
                    ("Ember Dark", "Black with orange accent #FF6A2B", 0x002BA2FF, 532, 140),
                    ("Vivid", "Black with vivid blue #2F7BFF", 0x00FF7B2F, 256, 230),
                    ("Silver", "Light panel #F2F2F2, black notch", 0x00D97706, 532, 230),
                ];

                let card_w = 260;
                let card_h = 76;

                for (name, desc, accent_colorref, tx, ty) in themes.iter() {
                    let is_sel = cfg.theme == *name;
                    let card_bg = CreateSolidBrush(COLORREF(if is_sel { 0x00222222 } else { 0x00161616 }));
                    let cr = RECT { left: *tx, top: *ty, right: *tx + card_w, bottom: *ty + card_h };
                    let _ = FillRect(hdc, &cr, card_bg);
                    let _ = DeleteObject(card_bg);

                    // Card border
                    let b_color = if is_sel { 0x00FFFFFF } else { 0x002A2A2A };
                    let b_pen = CreatePen(PS_SOLID, if is_sel { 2 } else { 1 }, COLORREF(b_color));
                    let old_p = SelectObject(hdc, b_pen);
                    let _ = MoveToEx(hdc, *tx, *ty, None);
                    let _ = LineTo(hdc, *tx + card_w, *ty);
                    let _ = LineTo(hdc, *tx + card_w, *ty + card_h);
                    let _ = LineTo(hdc, *tx, *ty + card_h);
                    let _ = LineTo(hdc, *tx, *ty);
                    let _ = SelectObject(hdc, old_p);
                    let _ = DeleteObject(b_pen);

                    // Mini accent dot
                    let dot_b = CreateSolidBrush(COLORREF(*accent_colorref));
                    let dr = RECT { left: *tx + 14, top: *ty + 16, right: *tx + 24, bottom: *ty + 26 };
                    let _ = FillRect(hdc, &dr, dot_b);
                    let _ = DeleteObject(dot_b);

                    SetTextColor(hdc, COLORREF(0x00FFFFFF));
                    draw_gdi_text(hdc, name, *tx + 32, *ty + 13, 14, true);
                    SetTextColor(hdc, COLORREF(0x00888888));
                    draw_gdi_text(hdc, desc, *tx + 14, *ty + 40, 11, false);
                }

                draw_section_header(hdc, "SURFACE & OUTLINE", content_x, 335);
                draw_toggle_row(hdc, "Notch Outline", "A crisp hairline around the notch geometry.", cfg.outline_enabled, content_x, 360);

                // Segmented control for outline width
                draw_segmented_row(hdc, "Outline Width", cfg.outline_width, cfg.outline_enabled, content_x, 415);
            }
            3 => {
                // Page 3: Modules Management
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                draw_gdi_text(hdc, "Modules Management", content_x, 32, 22, true);
                SetTextColor(hdc, COLORREF(0x00888888));
                draw_gdi_text(hdc, "Choose what lives in your notch. Toggling off frees module memory instantly.", content_x, 62, 13, false);

                draw_section_header(hdc, "DOCK MODULES", content_x, 105);
                let modules = [
                    ("Home Hub", "Now playing, quick stats and focus summary", cfg.enabled_modules.home),
                    ("System Performance", "CPU, memory, disk and real battery", cfg.enabled_modules.system),
                    ("World Clock", "Live world clocks with city offsets", cfg.enabled_modules.clock),
                    ("Focus Timers", "Pomodoro, countdown and stopwatches", cfg.enabled_modules.timers),
                    ("Calendar", "Month grid and agenda schedule", cfg.enabled_modules.calendar),
                    ("Local Weather", "Current forecast and conditions", cfg.enabled_modules.weather),
                    ("Clipboard History", "Searchable history with DPAPI encryption", cfg.enabled_modules.clipboard),
                    ("Quick Notes", "Local persistent notes and checkboxes", cfg.enabled_modules.notes),
                ];

                for (i, (name, desc, en)) in modules.iter().enumerate() {
                    let y = 135 + (i as i32 * 38);
                    SetTextColor(hdc, COLORREF(0x00DDDDDD));
                    draw_gdi_text(hdc, name, content_x, y + 2, 13, true);
                    SetTextColor(hdc, COLORREF(0x00777777));
                    draw_gdi_text(hdc, desc, content_x + 180, y + 3, 11, false);

                    // Toggle switch pill at 780..820
                    let switch_bg = if *en { 0x0010B981 } else { 0x002B2B2B };
                    let brush = CreateSolidBrush(COLORREF(switch_bg));
                    let r = RECT { left: 780, top: y + 2, right: 820, bottom: y + 22 };
                    let _ = FillRect(hdc, &r, brush);
                    let _ = DeleteObject(brush);

                    let knob_x = if *en { 802 } else { 782 };
                    let knob_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
                    let kr = RECT { left: knob_x, top: y + 4, right: knob_x + 16, bottom: y + 20 };
                    let _ = FillRect(hdc, &kr, knob_brush);
                    let _ = DeleteObject(knob_brush);
                }
            }
            4 => {
                // Page 4: Live Activities
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                draw_gdi_text(hdc, "Live Activities", content_x, 32, 22, true);
                SetTextColor(hdc, COLORREF(0x00888888));
                draw_gdi_text(hdc, "Configure automatic drop-down notifications in the notch.", content_x, 62, 13, false);

                draw_section_header(hdc, "TRIGGERS", content_x, 105);
                draw_toggle_row(hdc, "Clipboard Copies", "Show notification when text or images are copied.", true, content_x, 135);
                draw_toggle_row(hdc, "Screenshots Saved", "Show preview chip when screenshots are taken.", true, content_x, 200);
                draw_toggle_row(hdc, "Power & Charging", "Notify on charger plug/unplug and low battery alerts.", true, content_x, 265);
            }
            5 => {
                // Page 5: Privacy
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                draw_gdi_text(hdc, "Privacy & Data", content_x, 32, 22, true);
                SetTextColor(hdc, COLORREF(0x00888888));
                draw_gdi_text(hdc, "Everything stays on your PC: no account, no ads, no usage tracking.", content_x, 62, 13, false);

                draw_section_header(hdc, "CLIPBOARD SECURITY", content_x, 105);
                draw_toggle_row(hdc, "Ignore Password Managers", "Respect ExcludeClipboardContentFromMonitorProcessing.", true, content_x, 135);
                draw_toggle_row(hdc, "Encrypt History (DPAPI)", "Encrypt clipboard data at rest using Windows DPAPI.", true, content_x, 200);

                draw_section_header(hdc, "DATA CONTROLS", content_x, 280);
                draw_info_row(hdc, "Clipboard History Limit", "50 items", content_x, 310);
                draw_info_row(hdc, "Storage Location", "%APPDATA%\\NotchBar", content_x, 360);
            }
            6 => {
                // Page 6: Updates
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                draw_gdi_text(hdc, "Updates", content_x, 32, 22, true);
                SetTextColor(hdc, COLORREF(0x00888888));
                draw_gdi_text(hdc, "NotchBar operates strictly offline with zero background tracking.", content_x, 62, 13, false);

                draw_section_header(hdc, "VERSION STATUS", content_x, 105);
                draw_info_row(hdc, "Current Build", "NotchBar v0.1.0 (Release)", content_x, 135);
                draw_info_row(hdc, "Update Channel", "Local Binary · Up to date", content_x, 195);
            }
            _ => {
                // Page 7: About
                SetTextColor(hdc, COLORREF(0x00FFFFFF));
                draw_gdi_text(hdc, "About NotchBar", content_x, 32, 22, true);
                SetTextColor(hdc, COLORREF(0x00888888));
                draw_gdi_text(hdc, "A high-performance macOS-style productivity notch overlay for Windows.", content_x, 62, 13, false);

                draw_section_header(hdc, "SYSTEM ARCHITECTURE", content_x, 105);
                draw_info_row(hdc, "Rendering Engine", "Hardware Direct2D / DirectWrite / GDI Layered", content_x, 135);
                draw_info_row(hdc, "Target Idle RAM", "< 15 MB Private Working Set", content_x, 195);
                draw_info_row(hdc, "Runtime", "Zero Electron / Zero Web runtime · Pure Native Rust", content_x, 255);
            }
        }
    }
}

fn draw_section_header(hdc: HDC, title: &str, x: i32, y: i32) {
    unsafe {
        SetTextColor(hdc, COLORREF(0x00666666));
        draw_gdi_text(hdc, title, x, y, 10, true);
    }
}

fn draw_toggle_row(hdc: HDC, title: &str, desc: &str, is_on: bool, x: i32, y: i32) {
    unsafe {
        SetTextColor(hdc, COLORREF(0x00FFFFFF));
        draw_gdi_text(hdc, title, x, y, 14, true);
        SetTextColor(hdc, COLORREF(0x00888888));
        draw_gdi_text(hdc, desc, x, y + 20, 11, false);

        // Toggle Switch at 780..820
        let switch_bg = if is_on { 0x0010B981 } else { 0x002B2B2B };
        let brush = CreateSolidBrush(COLORREF(switch_bg));
        let r = RECT { left: 780, top: y + 6, right: 820, bottom: y + 26 };
        let _ = FillRect(hdc, &r, brush);
        let _ = DeleteObject(brush);

        let knob_x = if is_on { 802 } else { 782 };
        let knob_brush = CreateSolidBrush(COLORREF(0x00FFFFFF));
        let kr = RECT { left: knob_x, top: y + 8, right: knob_x + 16, bottom: y + 24 };
        let _ = FillRect(hdc, &kr, knob_brush);
        let _ = DeleteObject(knob_brush);
    }
}

fn draw_segmented_row(hdc: HDC, title: &str, current_w: u32, enabled: bool, x: i32, y: i32) {
    unsafe {
        SetTextColor(hdc, COLORREF(if enabled { 0x00FFFFFF } else { 0x00555555 }));
        draw_gdi_text(hdc, title, x, y, 14, true);

        let widths = [1, 2, 3];
        for (i, w) in widths.iter().enumerate() {
            let bx = 680 + (i as i32 * 46);
            let is_cur = *w == current_w && enabled;
            let bg_col = if is_cur { 0x003B82F6 } else { 0x00222222 };

            let brush = CreateSolidBrush(COLORREF(bg_col));
            let r = RECT { left: bx, top: y, right: bx + 40, bottom: y + 28 };
            let _ = FillRect(hdc, &r, brush);
            let _ = DeleteObject(brush);

            let txt = format!("{} px", w);
            SetTextColor(hdc, COLORREF(if is_cur { 0x00FFFFFF } else { 0x00888888 }));
            draw_gdi_text(hdc, &txt, bx + 7, y + 6, 11, is_cur);
        }
    }
}

fn draw_info_row(hdc: HDC, title: &str, value: &str, x: i32, y: i32) {
    unsafe {
        SetTextColor(hdc, COLORREF(0x00FFFFFF));
        draw_gdi_text(hdc, title, x, y, 13, false);
        SetTextColor(hdc, COLORREF(0x00888888));
        draw_gdi_text(hdc, value, 580, y, 12, true);
    }
}

fn draw_gdi_text(hdc: HDC, text: &str, x: i32, y: i32, size: i32, bold: bool) {
    unsafe {
        let weight = if bold { FW_BOLD } else { FW_NORMAL };
        let font = CreateFontW(
            size,
            0,
            0,
            0,
            weight.0 as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET.0 as u32,
            OUT_DEFAULT_PRECIS.0 as u32,
            CLIP_DEFAULT_PRECIS.0 as u32,
            CLEARTYPE_QUALITY.0 as u32,
            DEFAULT_PITCH.0 as u32,
            w!("Segoe UI"),
        );
        let old_font = SelectObject(hdc, font);
        let wide: Vec<u16> = text.encode_utf16().collect();
        let _ = TextOutW(hdc, x, y, &wide);
        let _ = SelectObject(hdc, old_font);
        let _ = DeleteObject(font);
    }
}
