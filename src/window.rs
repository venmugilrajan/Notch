use crate::config::AppConfig;
use crate::modules::clipboard::ClipboardManager;
use crate::modules::media::{MediaInfo, MediaWatcher};
use crate::modules::power::{PowerNotifier, PowerStatusInfo};
use crate::modules::productivity::ProductivityManager;
use crate::modules::stats::SystemStatsSampler;
use crate::renderer::{D2DRenderer, NotchTab, ALL_TABS};
use crate::tray::*;
use crate::ui::settings::SettingsWindow;
use crate::ui::theme::ThemeTokens;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::System::DataExchange::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

const NOTCH_WINDOW_CLASS: PCWSTR = w!("NotchBarWindowClass");
const NOTCH_HOTKEY_ID: i32 = 1001;
const DIAGNOSTICS_HOTKEY_ID: i32 = 1003;

const WM_MOUSEHOVER: u32 = 0x02A1;
const WM_MOUSELEAVE: u32 = 0x02A3;
const WM_POWERBROADCAST: u32 = 0x0218;
pub const WM_SETTINGS_CHANGED: u32 = WM_USER + 200;

const TIMER_ANIMATION_ID: usize = 1;
const TIMER_AUTOHIDE_ID: usize = 2;
const TIMER_LEAVE_COLLAPSE_ID: usize = 3;
const TIMER_MEDIA_REFRESH_ID: usize = 4;

#[derive(PartialEq, Clone, Copy)]
pub enum NotchState {
    Collapsed,
    Expanding,
    Expanded,
    Collapsing,
}

pub struct WindowContext {
    pub config: Arc<Mutex<AppConfig>>,
    pub renderer: D2DRenderer,
    pub tray: Option<TrayIcon>,
    pub state: NotchState,
    pub animation_progress: f32, // 0.0 -> 1.0
    pub animation_start: Instant,
    pub is_hovered: bool,
    pub is_tracking_mouse: bool,
    pub is_pinned: bool,
    pub active_tab: NotchTab,
    pub hovered_dock_index: Option<usize>,
    pub current_w: i32,
    pub current_h: i32,
    pub screen_w: i32,
    pub screen_h: i32,
    pub theme_tokens: ThemeTokens,
    pub paused_hover: bool,
    pub stats_sampler: SystemStatsSampler,
    pub power_status: PowerStatusInfo,
    pub power_notifier: Option<PowerNotifier>,
    pub media_watcher: MediaWatcher,
    pub media_info: MediaInfo,
    pub prod_mgr: ProductivityManager,
    pub clipboard_mgr: ClipboardManager,
    pub mem_dc: HDC,
    pub dib_bitmap: HBITMAP,
    pub dib_bits: *mut u8,
    pub dib_w: i32,
    pub dib_h: i32,
}

pub fn run_app(config: Arc<Mutex<AppConfig>>) -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.into(),
            hIcon: LoadIconW(None, IDI_APPLICATION)?,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hbrBackground: HBRUSH(GetStockObject(BLACK_BRUSH).0),
            lpszMenuName: PCWSTR::null(),
            lpszClassName: NOTCH_WINDOW_CLASS,
            hIconSm: HICON(std::ptr::null_mut()),
        };

        let atom = RegisterClassExW(&wnd_class);
        if atom == 0 {
            return Err(Error::from_win32());
        }

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        let (initial_w, initial_h, theme_name, hotkey_mods, hotkey_vk, clip_limit) = {
            let cfg = config.lock().unwrap();
            (
                cfg.collapsed_width,
                cfg.collapsed_height,
                cfg.theme.clone(),
                cfg.hotkey_modifiers,
                cfg.hotkey_vk,
                cfg.clipboard_history_limit,
            )
        };
        let initial_x = (screen_w - initial_w) / 2;
        let initial_y = 0; // Anchored flush with top screen edge (y = 0)

        let ex_style = WS_EX_TOPMOST
            | WS_EX_TOOLWINDOW
            | WS_EX_LAYERED
            | WS_EX_NOACTIVATE;

        let hwnd = CreateWindowExW(
            ex_style,
            NOTCH_WINDOW_CLASS,
            w!("NotchBar"),
            WS_POPUP,
            initial_x,
            initial_y,
            initial_w,
            initial_h,
            HWND(std::ptr::null_mut()),
            HMENU(std::ptr::null_mut()),
            instance,
            None,
        )?;

        let theme_tokens = ThemeTokens::from_name(&theme_name);
        let renderer = D2DRenderer::new()?;

        let mut context = Box::new(WindowContext {
            config: config.clone(),
            renderer,
            tray: None,
            state: NotchState::Collapsed,
            animation_progress: 0.0,
            animation_start: Instant::now(),
            is_hovered: false,
            is_tracking_mouse: false,
            is_pinned: false,
            active_tab: NotchTab::Home,
            hovered_dock_index: None,
            current_w: initial_w,
            current_h: initial_h,
            screen_w,
            screen_h,
            theme_tokens,
            paused_hover: false,
            stats_sampler: SystemStatsSampler::new(),
            power_status: PowerStatusInfo::read_current(),
            power_notifier: None,
            media_watcher: MediaWatcher::new(),
            media_info: MediaInfo::default(),
            prod_mgr: ProductivityManager::new(),
            clipboard_mgr: ClipboardManager::new(clip_limit),
            mem_dc: HDC(std::ptr::null_mut()),
            dib_bitmap: HBITMAP(std::ptr::null_mut()),
            dib_bits: std::ptr::null_mut(),
            dib_w: 0,
            dib_h: 0,
        });

        recreate_layered_buffer(&mut context, initial_w, initial_h);

        let tray = TrayIcon::new(hwnd)?;
        context.tray = Some(tray);
        context.power_notifier = Some(PowerNotifier::register(hwnd));

        let _ = AddClipboardFormatListener(hwnd);

        SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);

        let _ = DragAcceptFiles(hwnd, true);

        let _ = RegisterHotKey(
            hwnd,
            NOTCH_HOTKEY_ID,
            HOT_KEY_MODIFIERS(hotkey_mods),
            hotkey_vk,
        );

        let _ = RegisterHotKey(
            hwnd,
            DIAGNOSTICS_HOTKEY_ID,
            MOD_CONTROL | MOD_SHIFT,
            0x44, // 'D'
        );

        let ctx = &mut *(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowContext);
        render_and_update_layered(hwnd, ctx);

        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        let _ = SetTimer(hwnd, TIMER_AUTOHIDE_ID, 2000, None);
        let _ = SetTimer(hwnd, TIMER_MEDIA_REFRESH_ID, 1000, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        let _ = UnregisterHotKey(hwnd, NOTCH_HOTKEY_ID);
        let _ = UnregisterHotKey(hwnd, DIAGNOSTICS_HOTKEY_ID);
    }

    Ok(())
}

fn recreate_layered_buffer(ctx: &mut WindowContext, w: i32, h: i32) {
    unsafe {
        if !ctx.mem_dc.0.is_null() && !ctx.dib_bitmap.0.is_null() {
            let _ = DeleteObject(ctx.dib_bitmap);
            let _ = DeleteDC(ctx.mem_dc);
        }

        let hdc_screen = GetDC(HWND(std::ptr::null_mut()));
        ctx.mem_dc = CreateCompatibleDC(hdc_screen);

        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default()],
        };

        let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
        ctx.dib_bitmap = CreateDIBSection(
            ctx.mem_dc,
            &bmi,
            DIB_RGB_COLORS,
            &mut bits,
            HANDLE(std::ptr::null_mut()),
            0,
        ).unwrap_or(HBITMAP(std::ptr::null_mut()));
        ctx.dib_bits = bits as *mut u8;
        ctx.dib_w = w;
        ctx.dib_h = h;

        let _ = SelectObject(ctx.mem_dc, ctx.dib_bitmap);
        let _ = ReleaseDC(HWND(std::ptr::null_mut()), hdc_screen);
    }
}

fn render_and_update_layered(hwnd: HWND, ctx: &mut WindowContext) {
    if ctx.mem_dc.0.is_null() {
        return;
    }

    let stats_data = ctx.stats_sampler.get_metrics();
    ctx.media_info = ctx.media_watcher.get_info();

    let (enabled_modules, time_format_24h, shrink_mode) = {
        let cfg = ctx.config.lock().unwrap();
        (cfg.enabled_modules.clone(), cfg.time_format_24h, cfg.shrink_mode)
    };

    let _ = ctx.renderer.render_to_dc(
        ctx.mem_dc,
        ctx.current_w as f32,
        ctx.current_h as f32,
        ctx.animation_progress,
        &ctx.theme_tokens,
        ctx.active_tab,
        ctx.hovered_dock_index,
        ctx.is_pinned,
        &stats_data,
        &ctx.power_status,
        &ctx.media_info,
        &ctx.prod_mgr,
        &ctx.clipboard_mgr,
        &enabled_modules,
        time_format_24h,
        shrink_mode,
    );

    unsafe {
        let x = (ctx.screen_w - ctx.current_w) / 2;
        let y = 0;

        let pt_dst = POINT { x, y };
        let size_dst = SIZE { cx: ctx.current_w, cy: ctx.current_h };
        let pt_src = POINT { x: 0, y: 0 };

        let blend = BLENDFUNCTION {
            BlendOp: AC_SRC_OVER as u8,
            BlendFlags: 0,
            SourceConstantAlpha: 255,
            AlphaFormat: AC_SRC_ALPHA as u8,
        };

        let _ = UpdateLayeredWindow(
            hwnd,
            HDC(std::ptr::null_mut()),
            Some(&pt_dst),
            Some(&size_dst),
            ctx.mem_dc,
            Some(&pt_src),
            COLORREF(0),
            Some(&blend),
            ULW_ALPHA,
        );
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    let ctx_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WindowContext;
    if ctx_ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let ctx = &mut *ctx_ptr;

    if msg == WM_DROPFILES {
        let hdrop = HDROP(wparam.0 as *mut _);
        let count = DragQueryFileW(hdrop, 0xFFFFFFFF, None);
        for i in 0..count {
            let mut buf = [0u16; 512];
            let len = DragQueryFileW(hdrop, i, Some(&mut buf));
            if len > 0 {
                let path_str = String::from_utf16_lossy(&buf[..len as usize]);
                ctx.prod_mgr.park_file(std::path::PathBuf::from(path_str));
            }
        }
        DragFinish(hdrop);
        if ctx.state == NotchState::Collapsed || ctx.state == NotchState::Collapsing {
            start_expansion(hwnd, ctx);
        }
        ctx.active_tab = NotchTab::FileShelf;
        render_and_update_layered(hwnd, ctx);
        return LRESULT(0);
    }

    if msg == WM_CLIPBOARDUPDATE {
        if let Some(_captured) = ctx.clipboard_mgr.on_clipboard_change(hwnd) {
            render_and_update_layered(hwnd, ctx);
        }
        return LRESULT(0);
    }

    if msg == WM_MOUSEMOVE {
        let x = (lparam.0 & 0xFFFF) as i32;
        let y = ((lparam.0 >> 16) & 0xFFFF) as i32;

        if !ctx.is_tracking_mouse {
            let hover_delay = ctx.config.lock().unwrap().hover_delay_ms;
            let mut tme = TRACKMOUSEEVENT {
                cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                dwFlags: TME_LEAVE | TME_HOVER,
                hwndTrack: hwnd,
                dwHoverTime: hover_delay,
            };
            let _ = TrackMouseEvent(&mut tme);
            ctx.is_tracking_mouse = true;
        }

        let _ = KillTimer(hwnd, TIMER_LEAVE_COLLAPSE_ID);

        if ctx.state == NotchState::Expanded {
            let dock_top = ctx.current_h - 56;
            if y >= dock_top {
                let enabled_modules = {
                    ctx.config.lock().unwrap().enabled_modules.clone()
                };
                let visible_count = ALL_TABS.iter().filter(|t| enabled_modules.is_enabled(**t)).count();
                if visible_count > 0 {
                    let ear_r = 14.0 + (ctx.animation_progress * 6.0);
                    let dock_left = ear_r + 4.0;
                    let dock_w = ctx.current_w as f32 - (dock_left * 2.0);
                    let slot_w = dock_w / (visible_count as f32);
                    let rel_x = x as f32 - dock_left;
                    if rel_x >= 0.0 {
                        let idx = (rel_x / slot_w) as usize;
                        if idx < visible_count && ctx.hovered_dock_index != Some(idx) {
                            ctx.hovered_dock_index = Some(idx);
                            render_and_update_layered(hwnd, ctx);
                        }
                    }
                }
            } else if ctx.hovered_dock_index.is_some() {
                ctx.hovered_dock_index = None;
                render_and_update_layered(hwnd, ctx);
            }
        }

        return LRESULT(0);
    }

    if msg == WM_MOUSEHOVER {
        ctx.is_hovered = true;
        if !ctx.paused_hover && (ctx.state == NotchState::Collapsed || ctx.state == NotchState::Collapsing) {
            start_expansion(hwnd, ctx);
        }
        return LRESULT(0);
    }

    if msg == WM_MOUSELEAVE {
        ctx.is_tracking_mouse = false;
        ctx.is_hovered = false;
        ctx.hovered_dock_index = None;
        if !ctx.is_pinned && (ctx.state == NotchState::Expanded || ctx.state == NotchState::Expanding) {
            let auto_delay = ctx.config.lock().unwrap().auto_collapse_delay_ms;
            let _ = SetTimer(hwnd, TIMER_LEAVE_COLLAPSE_ID, auto_delay, None);
        }
        return LRESULT(0);
    }

    if msg == WM_LBUTTONDOWN {
        let x = (lparam.0 & 0xFFFF) as i32;
        let y = ((lparam.0 >> 16) & 0xFFFF) as i32;

        if ctx.state == NotchState::Expanded {
            let ear_r = 14.0 + (ctx.animation_progress * 6.0);
            let content_right = ctx.current_w as f32 - ear_r - 14.0;

            if y <= 44 {
                let close_cx = content_right - 16.0;
                let gear_cx = content_right - 44.0;
                let pin_cx = content_right - 72.0;

                // Close button (cx = content_right - 16)
                if (x as f32 - close_cx).abs() <= 14.0 {
                    start_collapse(hwnd, ctx);
                    return LRESULT(0);
                }
                // Gear / Settings button (cx = content_right - 44)
                if (x as f32 - gear_cx).abs() <= 14.0 {
                    let _ = SettingsWindow::open_or_focus(ctx.config.clone());
                    return LRESULT(0);
                }
                // Pin button (cx = content_right - 72)
                if (x as f32 - pin_cx).abs() <= 14.0 {
                    ctx.is_pinned = !ctx.is_pinned;
                    render_and_update_layered(hwnd, ctx);
                    return LRESULT(0);
                }
            }

            // Media control clicks when active_tab == NotchTab::Media
            if ctx.active_tab == NotchTab::Media {
                let ear_r = 14.0 + (ctx.animation_progress * 6.0);
                let content_left = ear_r + 14.0;
                let content_top = 48.0;
                let content_bottom = ctx.current_h as f32 - 56.0;

                let art_size = 140.0;
                let art_left = content_left + 28.0;
                let art_top = content_top + ((content_bottom - content_top - art_size) / 2.0);
                let details_left = art_left + art_size + 28.0;
                let btn_y = art_top + 120.0;

                if ((y as f32) - btn_y).abs() <= 20.0 {
                    let prev_cx = details_left + 30.0;
                    let play_cx = details_left + 80.0;
                    let next_cx = details_left + 130.0;

                    if ((x as f32) - prev_cx).abs() <= 16.0 {
                        MediaWatcher::previous_track();
                        return LRESULT(0);
                    } else if ((x as f32) - play_cx).abs() <= 20.0 {
                        MediaWatcher::toggle_play_pause();
                        return LRESULT(0);
                    } else if ((x as f32) - next_cx).abs() <= 16.0 {
                        MediaWatcher::next_track();
                        return LRESULT(0);
                    }
                }
            }

            // Clipboard controls clicks when active_tab == NotchTab::Clipboard
            if ctx.active_tab == NotchTab::Clipboard {
                let ear_r = 14.0 + (ctx.animation_progress * 6.0);
                let content_left = ear_r + 14.0;
                let content_top = 48.0;
                let left_w = 280.0;
                let right_left = content_left + left_w + 12.0;
                let fx = x as f32;
                let fy = y as f32;

                // Tab clicks in left pane
                let tab_y = content_top + 14.0;
                if fy >= tab_y && fy <= tab_y + 24.0 && fx >= content_left + 16.0 && fx <= content_left + 256.0 {
                    let rel_tab = ((fx - (content_left + 16.0)) / 60.0) as usize;
                    match rel_tab {
                        0 => ctx.clipboard_mgr.active_tab = crate::modules::clipboard::ClipboardFilterTab::All,
                        1 => ctx.clipboard_mgr.active_tab = crate::modules::clipboard::ClipboardFilterTab::Text,
                        2 => ctx.clipboard_mgr.active_tab = crate::modules::clipboard::ClipboardFilterTab::Images,
                        3 => ctx.clipboard_mgr.active_tab = crate::modules::clipboard::ClipboardFilterTab::Links,
                        _ => {}
                    }
                    render_and_update_layered(hwnd, ctx);
                    return LRESULT(0);
                }

                // Item row clicks in left pane
                let list_start_y = content_top + 14.0 + 26.0 + 26.0 + 12.0;
                if fx >= content_left + 10.0 && fx <= content_left + left_w - 10.0 && fy >= list_start_y {
                    let filtered = ctx.clipboard_mgr.filtered_items();
                    let row_idx = ((fy - list_start_y) / 40.0) as usize;
                    if row_idx < filtered.len() {
                        let clicked_id = filtered[row_idx].id;
                        ctx.clipboard_mgr.selected_item_id = Some(clicked_id);
                        render_and_update_layered(hwnd, ctx);
                        return LRESULT(0);
                    }
                }

                // Copy button click in right pane
                let right_right = content_right;
                let copy_btn_left = right_right - 80.0;
                let copy_btn_top = content_top + 12.0;
                if fx >= copy_btn_left && fx <= right_right - 16.0 && fy >= copy_btn_top && fy <= copy_btn_top + 26.0 {
                    if let Some(item) = ctx.clipboard_mgr.items.iter().find(|i| Some(i.id) == ctx.clipboard_mgr.selected_item_id) {
                        ctx.clipboard_mgr.copy_to_clipboard(&item.text);
                        render_and_update_layered(hwnd, ctx);
                        return LRESULT(0);
                    }
                }

                // Pin toggle click in right pane
                let pin_icon_x = right_right - 140.0;
                let pin_icon_y = content_top + 24.0;
                if (fx - pin_icon_x).abs() <= 12.0 && (fy - pin_icon_y).abs() <= 12.0 {
                    if let Some(id) = ctx.clipboard_mgr.selected_item_id {
                        ctx.clipboard_mgr.toggle_pin(id);
                        render_and_update_layered(hwnd, ctx);
                        return LRESULT(0);
                    }
                }
            }

            // Notes page interaction
            if ctx.active_tab == NotchTab::Notes {
                let ear_r = 14.0 + (ctx.animation_progress * 6.0);
                let content_left = ear_r + 14.0;
                let content_top = 48.0;
                let left_w = 260.0;
                let fx = x as f32;
                let fy = y as f32;

                // "+ Note" button click in left pane header
                let add_btn_left = content_left + left_w - 68.0;
                let add_btn_top = content_top + 12.0;
                if fx >= add_btn_left && fx <= content_left + left_w - 14.0 && fy >= add_btn_top && fy <= add_btn_top + 22.0 {
                    let note_count = ctx.prod_mgr.notes.len() + 1;
                    ctx.prod_mgr.add_note(&format!("New task or quick memo #{}", note_count));
                    render_and_update_layered(hwnd, ctx);
                    return LRESULT(0);
                }

                // Checkbox or note selection click
                let mut note_y = content_top + 48.0;
                for note in ctx.prod_mgr.notes.clone().iter().take(5) {
                    if fx >= content_left + 10.0 && fx <= content_left + left_w - 10.0 && fy >= note_y && fy <= note_y + 32.0 {
                        let check_cx = content_left + 10.0 + 14.0;
                        if (fx - check_cx).abs() <= 12.0 {
                            // Toggle checkbox
                            ctx.prod_mgr.toggle_note(note.id);
                        }
                        render_and_update_layered(hwnd, ctx);
                        return LRESULT(0);
                    }
                    note_y += 36.0;
                }
            }

            // File Shelf page interaction
            if ctx.active_tab == NotchTab::FileShelf {
                let ear_r = 14.0 + (ctx.animation_progress * 6.0);
                let content_left = ear_r + 14.0;
                let content_top = 48.0;
                let fx = x as f32;
                let fy = y as f32;

                let mut item_y = content_top + 50.0;
                for (i, p) in ctx.prod_mgr.parked_files.clone().iter().take(5).enumerate() {
                    if fx >= content_left + 24.0 && fx <= content_right && fy >= item_y && fy <= item_y + 24.0 {
                        // Open file on click via ShellExecute
                        let path_w: Vec<u16> = p.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
                        unsafe {
                            windows::Win32::UI::Shell::ShellExecuteW(
                                hwnd,
                                w!("open"),
                                PCWSTR(path_w.as_ptr()),
                                PCWSTR::null(),
                                PCWSTR::null(),
                                windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                            );
                        }
                        return LRESULT(0);
                    }
                    item_y += 28.0;
                }
            }

            // Links page interaction
            if ctx.active_tab == NotchTab::Links {
                let ear_r = 14.0 + (ctx.animation_progress * 6.0);
                let content_left = ear_r + 14.0;
                let fx = x as f32;
                let fy = y as f32;

                let chip_y = 48.0 + 46.0;
                let b_top = chip_y + 42.0;
                let tile_w = (content_right - content_left - 48.0 - 36.0) / 4.0;

                let bookmarks = [
                    "https://github.com",
                    "https://youtube.com",
                    "https://google.com",
                    "https://chatgpt.com",
                ];

                for (i, url) in bookmarks.iter().enumerate() {
                    let bx = content_left + 24.0 + (i as f32 * (tile_w + 12.0));
                    if fx >= bx && fx <= bx + tile_w && fy >= b_top && fy <= b_top + 90.0 {
                        let url_w: Vec<u16> = url.encode_utf16().chain(std::iter::once(0)).collect();
                        unsafe {
                            windows::Win32::UI::Shell::ShellExecuteW(
                                hwnd,
                                w!("open"),
                                PCWSTR(url_w.as_ptr()),
                                PCWSTR::null(),
                                PCWSTR::null(),
                                windows::Win32::UI::WindowsAndMessaging::SW_SHOW,
                            );
                        }
                        return LRESULT(0);
                    }
                }
            }

            let dock_top = ctx.current_h - 56;
            if y >= dock_top {
                let enabled_modules = {
                    ctx.config.lock().unwrap().enabled_modules.clone()
                };
                let visible_tabs: Vec<NotchTab> = ALL_TABS
                    .iter()
                    .filter(|t| enabled_modules.is_enabled(**t))
                    .cloned()
                    .collect();

                let visible_count = visible_tabs.len();
                if visible_count > 0 {
                    let ear_r = 14.0 + (ctx.animation_progress * 6.0);
                    let dock_left = ear_r + 4.0;
                    let dock_w = ctx.current_w as f32 - (dock_left * 2.0);
                    let slot_w = dock_w / (visible_count as f32);
                    let rel_x = x as f32 - dock_left;
                    if rel_x >= 0.0 {
                        let clicked_idx = (rel_x / slot_w) as usize;
                        if clicked_idx < visible_count {
                            ctx.active_tab = visible_tabs[clicked_idx];
                            ctx.stats_sampler.set_active(ctx.active_tab == NotchTab::Home || ctx.active_tab == NotchTab::System);
                            render_and_update_layered(hwnd, ctx);
                        }
                    }
                }
            }
        } else if ctx.state == NotchState::Collapsed {
            start_expansion(hwnd, ctx);
        }
        return LRESULT(0);
    }

    if msg == WM_HOTKEY {
        if wparam.0 == NOTCH_HOTKEY_ID as usize {
            toggle_notch(hwnd, ctx);
        } else if wparam.0 == DIAGNOSTICS_HOTKEY_ID as usize {
            if ctx.state == NotchState::Collapsed || ctx.state == NotchState::Collapsing {
                start_expansion(hwnd, ctx);
            }
            ctx.active_tab = NotchTab::Diagnostics;
            render_and_update_layered(hwnd, ctx);
        }
        return LRESULT(0);
    }

    if msg == WM_KEYDOWN {
        if wparam.0 == VK_ESCAPE.0 as usize {
            if ctx.state == NotchState::Expanded || ctx.state == NotchState::Expanding {
                start_collapse(hwnd, ctx);
            }
        }
        return LRESULT(0);
    }

    if msg == WM_TIMER {
        if wparam.0 == TIMER_ANIMATION_ID {
            handle_animation_tick(hwnd, ctx);
        } else if wparam.0 == TIMER_AUTOHIDE_ID {
            check_fullscreen_state(hwnd);
        } else if wparam.0 == TIMER_MEDIA_REFRESH_ID {
            if ctx.state == NotchState::Expanded && ctx.active_tab == NotchTab::Media {
                render_and_update_layered(hwnd, ctx);
            }
        } else if wparam.0 == TIMER_LEAVE_COLLAPSE_ID {
            let _ = KillTimer(hwnd, TIMER_LEAVE_COLLAPSE_ID);
            if !ctx.is_hovered && !ctx.is_pinned && (ctx.state == NotchState::Expanded || ctx.state == NotchState::Expanding) {
                start_collapse(hwnd, ctx);
            }
        }
        return LRESULT(0);
    }

    if msg == WM_POWERBROADCAST {
        ctx.power_status = PowerStatusInfo::read_current();
        render_and_update_layered(hwnd, ctx);
        return LRESULT(1); // Return TRUE for PBT messages
    }

    if msg == WM_SETTINGS_CHANGED {
        let (theme_name, col_w, col_h, exp_w, exp_h, shrink_mode) = {
            let cfg = ctx.config.lock().unwrap();
            (cfg.theme.clone(), cfg.collapsed_width, cfg.collapsed_height, cfg.expanded_width, cfg.expanded_height, cfg.shrink_mode)
        };
        ctx.theme_tokens = ThemeTokens::from_name(&theme_name);
        if ctx.state == NotchState::Collapsed {
            if shrink_mode {
                ctx.current_w = 48;
                ctx.current_h = 16;
            } else {
                ctx.current_w = col_w;
                ctx.current_h = col_h;
            }
        } else if ctx.state == NotchState::Expanded {
            ctx.current_w = exp_w;
            ctx.current_h = exp_h;
        }
        if ctx.dib_w != ctx.current_w || ctx.dib_h != ctx.current_h {
            recreate_layered_buffer(ctx, ctx.current_w, ctx.current_h);
        }
        render_and_update_layered(hwnd, ctx);
        return LRESULT(0);
    }

    if msg == WM_DISPLAYCHANGE {
        ctx.screen_w = GetSystemMetrics(SM_CXSCREEN);
        ctx.screen_h = GetSystemMetrics(SM_CYSCREEN);
        render_and_update_layered(hwnd, ctx);
        return LRESULT(0);
    }

    if msg == WM_TRAYICON {
        let event = (lparam.0 & 0xFFFF) as u32;
        if event == WM_RBUTTONUP || event == WM_LBUTTONUP {
            if let Some(tray) = &ctx.tray {
                tray.show_menu();
            }
        }
        return LRESULT(0);
    }

    if msg == WM_COMMAND {
        let cmd_id = (wparam.0 & 0xFFFF) as u32;
        match cmd_id {
            IDM_TRAY_OPEN => toggle_notch(hwnd, ctx),
            IDM_TRAY_PAUSE => ctx.paused_hover = !ctx.paused_hover,
            IDM_TRAY_SETTINGS => {
                let _ = SettingsWindow::open_or_focus(ctx.config.clone());
            }
            IDM_TRAY_QUIT => { let _ = DestroyWindow(hwnd); }
            _ => {}
        }
        return LRESULT(0);
    }

    if msg == WM_DESTROY {
        let _ = KillTimer(hwnd, TIMER_ANIMATION_ID);
        let _ = KillTimer(hwnd, TIMER_AUTOHIDE_ID);
        let _ = KillTimer(hwnd, TIMER_MEDIA_REFRESH_ID);
        let _ = KillTimer(hwnd, TIMER_LEAVE_COLLAPSE_ID);
        let _ = RemoveClipboardFormatListener(hwnd);
        let _ = DeleteObject(ctx.dib_bitmap);
        let _ = DeleteDC(ctx.mem_dc);
        let _ = Box::from_raw(ctx_ptr);
        PostQuitMessage(0);
        return LRESULT(0);
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn toggle_notch(hwnd: HWND, ctx: &mut WindowContext) {
    match ctx.state {
        NotchState::Collapsed | NotchState::Collapsing => start_expansion(hwnd, ctx),
        NotchState::Expanded | NotchState::Expanding => start_collapse(hwnd, ctx),
    }
}

fn start_expansion(hwnd: HWND, ctx: &mut WindowContext) {
    ctx.state = NotchState::Expanding;
    ctx.animation_start = Instant::now();
    ctx.stats_sampler.set_active(ctx.active_tab == NotchTab::Home || ctx.active_tab == NotchTab::System);
    unsafe {
        let _ = SetTimer(hwnd, TIMER_ANIMATION_ID, 16, None);
    }
}

fn start_collapse(hwnd: HWND, ctx: &mut WindowContext) {
    ctx.state = NotchState::Collapsing;
    ctx.animation_start = Instant::now();
    ctx.stats_sampler.set_active(false);
    unsafe {
        let _ = SetTimer(hwnd, TIMER_ANIMATION_ID, 16, None);
    }
}

fn handle_animation_tick(hwnd: HWND, ctx: &mut WindowContext) {
    let elapsed = ctx.animation_start.elapsed().as_secs_f32();
    let duration = if ctx.state == NotchState::Expanding { 0.22 } else { 0.16 };
    let progress = (elapsed / duration).min(1.0);

    let eased = 1.0 - (1.0 - progress).powi(3);

    let target_w;
    let target_h;

    let (col_w, col_h, exp_w, exp_h) = {
        let cfg = ctx.config.lock().unwrap();
        if cfg.shrink_mode {
            (48.0, 16.0, cfg.expanded_width as f32, cfg.expanded_height as f32)
        } else {
            (cfg.collapsed_width as f32, cfg.collapsed_height as f32, cfg.expanded_width as f32, cfg.expanded_height as f32)
        }
    };

    match ctx.state {
        NotchState::Expanding => {
            ctx.animation_progress = eased;
            target_w = col_w + (exp_w - col_w) * eased;
            target_h = col_h + (exp_h - col_h) * eased;

            if progress >= 1.0 {
                ctx.state = NotchState::Expanded;
                ctx.animation_progress = 1.0;
                unsafe {
                    let _ = KillTimer(hwnd, TIMER_ANIMATION_ID);
                }
            }
        }
        NotchState::Collapsing => {
            let inv_eased = 1.0 - eased;
            ctx.animation_progress = inv_eased;
            target_w = col_w + (exp_w - col_w) * inv_eased;
            target_h = col_h + (exp_h - col_h) * inv_eased;

            if progress >= 1.0 {
                ctx.state = NotchState::Collapsed;
                ctx.animation_progress = 0.0;
                unsafe {
                    let _ = KillTimer(hwnd, TIMER_ANIMATION_ID);
                }
                ctx.renderer.release_target();
            }
        }
        _ => return,
    }

    ctx.current_w = target_w as i32;
    ctx.current_h = target_h as i32;

    if ctx.dib_w != ctx.current_w || ctx.dib_h != ctx.current_h {
        recreate_layered_buffer(ctx, ctx.current_w, ctx.current_h);
    }

    render_and_update_layered(hwnd, ctx);
}

fn check_fullscreen_state(hwnd: HWND) {
    unsafe {
        // Ensure window always stays on top (HWND_TOPMOST)
        let _ = SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
    }
}
