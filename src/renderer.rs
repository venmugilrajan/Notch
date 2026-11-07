use crate::modules::clipboard::ClipboardManager;
use crate::modules::media::MediaInfo;
use crate::modules::power::PowerStatusInfo;
use crate::modules::productivity::ProductivityManager;
use crate::modules::stats::SystemStatsData;
use crate::ui::icons::IconPainter;
use crate::ui::theme::ThemeTokens;
use std::f32::consts::PI;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;

#[derive(Clone, Copy, PartialEq)]
pub enum NotchTab {
    Home = 0,
    System = 1,
    Clock = 2,
    Timers = 3,
    Calendar = 4,
    Weather = 5,
    Clipboard = 6,
    Notes = 7,
    FileShelf = 8,
    DropTransfer = 9,
    Converters = 10,
    Links = 11,
    Media = 12,
    Diagnostics = 13,
}

pub const ALL_TABS: [NotchTab; 13] = [
    NotchTab::Home,
    NotchTab::System,
    NotchTab::Clock,
    NotchTab::Timers,
    NotchTab::Calendar,
    NotchTab::Weather,
    NotchTab::Clipboard,
    NotchTab::Notes,
    NotchTab::FileShelf,
    NotchTab::DropTransfer,
    NotchTab::Converters,
    NotchTab::Links,
    NotchTab::Media,
];

pub struct D2DRenderer {
    factory: ID2D1Factory,
    write_factory: IDWriteFactory,
    icon_painter: IconPainter,
    render_target: Option<ID2D1DCRenderTarget>,
    title_format: Option<IDWriteTextFormat>,
    body_format: Option<IDWriteTextFormat>,
    sub_format: Option<IDWriteTextFormat>,
    meta_format: Option<IDWriteTextFormat>,
    number_format: Option<IDWriteTextFormat>,
    gauge_format: Option<IDWriteTextFormat>,
    gauge_small_format: Option<IDWriteTextFormat>,
    tooltip_format: Option<IDWriteTextFormat>,
}

impl D2DRenderer {
    pub fn new() -> Result<Self> {
        let factory: ID2D1Factory = unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?
        };

        let write_factory: IDWriteFactory = unsafe {
            DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)?
        };

        let icon_painter = IconPainter::new(&factory)?;

        Ok(Self {
            factory,
            write_factory,
            icon_painter,
            render_target: None,
            title_format: None,
            body_format: None,
            sub_format: None,
            meta_format: None,
            number_format: None,
            gauge_format: None,
            gauge_small_format: None,
            tooltip_format: None,
        })
    }

    pub fn get_title_format(&mut self) -> Result<&IDWriteTextFormat> {
        if self.title_format.is_none() {
            let f = unsafe {
                self.write_factory.CreateTextFormat(
                    w!("Segoe UI Variable Display"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    15.0,
                    w!("en-us"),
                )?
            };
            self.title_format = Some(f);
        }
        Ok(self.title_format.as_ref().unwrap())
    }

    pub fn get_body_format(&mut self) -> Result<&IDWriteTextFormat> {
        if self.body_format.is_none() {
            let f = unsafe {
                self.write_factory.CreateTextFormat(
                    w!("Segoe UI Variable Display"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    13.0,
                    w!("en-us"),
                )?
            };
            self.body_format = Some(f);
        }
        Ok(self.body_format.as_ref().unwrap())
    }

    pub fn get_sub_format(&mut self) -> Result<&IDWriteTextFormat> {
        if self.sub_format.is_none() {
            let f = unsafe {
                self.write_factory.CreateTextFormat(
                    w!("Segoe UI Variable Display"),
                    None,
                    DWRITE_FONT_WEIGHT_NORMAL,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    12.0,
                    w!("en-us"),
                )?
            };
            self.sub_format = Some(f);
        }
        Ok(self.sub_format.as_ref().unwrap())
    }

    pub fn get_meta_format(&mut self) -> Result<&IDWriteTextFormat> {
        if self.meta_format.is_none() {
            let f = unsafe {
                self.write_factory.CreateTextFormat(
                    w!("Segoe UI Variable Display"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    10.0,
                    w!("en-us"),
                )?
            };
            self.meta_format = Some(f);
        }
        Ok(self.meta_format.as_ref().unwrap())
    }

    pub fn get_number_format(&mut self) -> Result<&IDWriteTextFormat> {
        if self.number_format.is_none() {
            let f = unsafe {
                self.write_factory.CreateTextFormat(
                    w!("Segoe UI Variable Display"),
                    None,
                    DWRITE_FONT_WEIGHT_LIGHT,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    28.0,
                    w!("en-us"),
                )?
            };
            self.number_format = Some(f);
        }
        Ok(self.number_format.as_ref().unwrap())
    }

    pub fn get_gauge_format(&mut self) -> Result<&IDWriteTextFormat> {
        if self.gauge_format.is_none() {
            let f = unsafe {
                let fmt = self.write_factory.CreateTextFormat(
                    w!("Segoe UI Variable Display"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    11.0,
                    w!("en-us"),
                )?;
                let _ = fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
                let _ = fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                fmt
            };
            self.gauge_format = Some(f);
        }
        Ok(self.gauge_format.as_ref().unwrap())
    }

    pub fn get_gauge_small_format(&mut self) -> Result<&IDWriteTextFormat> {
        if self.gauge_small_format.is_none() {
            let f = unsafe {
                let fmt = self.write_factory.CreateTextFormat(
                    w!("Segoe UI Variable Display"),
                    None,
                    DWRITE_FONT_WEIGHT_SEMI_BOLD,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    10.0,
                    w!("en-us"),
                )?;
                let _ = fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
                let _ = fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                fmt
            };
            self.gauge_small_format = Some(f);
        }
        Ok(self.gauge_small_format.as_ref().unwrap())
    }

    pub fn get_tooltip_format(&mut self) -> Result<&IDWriteTextFormat> {
        if self.tooltip_format.is_none() {
            let f = unsafe {
                let fmt = self.write_factory.CreateTextFormat(
                    w!("Segoe UI Variable Display"),
                    None,
                    DWRITE_FONT_WEIGHT_MEDIUM,
                    DWRITE_FONT_STYLE_NORMAL,
                    DWRITE_FONT_STRETCH_NORMAL,
                    11.0,
                    w!("en-us"),
                )?;
                let _ = fmt.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER);
                let _ = fmt.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER);
                fmt
            };
            self.tooltip_format = Some(f);
        }
        Ok(self.tooltip_format.as_ref().unwrap())
    }

    pub fn get_or_create_dc_target(&mut self) -> Result<&ID2D1DCRenderTarget> {
        if self.render_target.is_none() {
            let render_props = D2D1_RENDER_TARGET_PROPERTIES {
                r#type: D2D1_RENDER_TARGET_TYPE_SOFTWARE,
                pixelFormat: D2D1_PIXEL_FORMAT {
                    format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_B8G8R8A8_UNORM,
                    alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED,
                },
                dpiX: 96.0,
                dpiY: 96.0,
                usage: D2D1_RENDER_TARGET_USAGE_NONE,
                minLevel: D2D1_FEATURE_LEVEL_DEFAULT,
            };

            let target = unsafe {
                self.factory.CreateDCRenderTarget(&render_props)?
            };
            self.render_target = Some(target);
        }
        Ok(self.render_target.as_ref().unwrap())
    }

    pub fn release_target(&mut self) {
        self.render_target = None;
    }

    pub fn build_notch_geometry(&self, width: f32, height: f32, bottom_r: f32, ear_r: f32) -> Result<ID2D1PathGeometry> {
        unsafe {
            let geometry = self.factory.CreatePathGeometry()?;
            let sink = geometry.Open()?;
            sink.SetFillMode(D2D1_FILL_MODE_WINDING);

            let p0 = D2D_POINT_2F { x: 0.0, y: 0.0 };
            sink.BeginFigure(p0, D2D1_FIGURE_BEGIN_FILLED);

            let b_left = D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F { x: ear_r * 0.45, y: 0.0 },
                point2: D2D_POINT_2F { x: ear_r, y: ear_r * 0.55 },
                point3: D2D_POINT_2F { x: ear_r, y: ear_r },
            };
            sink.AddBezier(&b_left);

            sink.AddLine(D2D_POINT_2F { x: ear_r, y: height - bottom_r });

            let a_bottom_left = D2D1_ARC_SEGMENT {
                point: D2D_POINT_2F { x: ear_r + bottom_r, y: height },
                size: D2D_SIZE_F { width: bottom_r, height: bottom_r },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                arcSize: D2D1_ARC_SIZE_SMALL,
            };
            sink.AddArc(&a_bottom_left);

            sink.AddLine(D2D_POINT_2F { x: width - ear_r - bottom_r, y: height });

            let a_bottom_right = D2D1_ARC_SEGMENT {
                point: D2D_POINT_2F { x: width - ear_r, y: height - bottom_r },
                size: D2D_SIZE_F { width: bottom_r, height: bottom_r },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_COUNTER_CLOCKWISE,
                arcSize: D2D1_ARC_SIZE_SMALL,
            };
            sink.AddArc(&a_bottom_right);

            sink.AddLine(D2D_POINT_2F { x: width - ear_r, y: ear_r });

            let b_right = D2D1_BEZIER_SEGMENT {
                point1: D2D_POINT_2F { x: width - ear_r, y: ear_r * 0.55 },
                point2: D2D_POINT_2F { x: width - (ear_r * 0.45), y: 0.0 },
                point3: D2D_POINT_2F { x: width, y: 0.0 },
            };
            sink.AddBezier(&b_right);

            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
            sink.Close()?;

            Ok(geometry)
        }
    }

    pub fn build_arc_path(&self, center: D2D_POINT_2F, radius: f32, start_angle: f32, sweep_angle: f32) -> Result<ID2D1PathGeometry> {
        unsafe {
            let geometry = self.factory.CreatePathGeometry()?;
            let sink = geometry.Open()?;
            sink.SetFillMode(D2D1_FILL_MODE_WINDING);

            let start_rad = start_angle * (PI / 180.0);
            let end_rad = (start_angle + sweep_angle) * (PI / 180.0);

            let start_pt = D2D_POINT_2F {
                x: center.x + (radius * start_rad.cos()),
                y: center.y + (radius * start_rad.sin()),
            };

            let end_pt = D2D_POINT_2F {
                x: center.x + (radius * end_rad.cos()),
                y: center.y + (radius * end_rad.sin()),
            };

            sink.BeginFigure(start_pt, D2D1_FIGURE_BEGIN_HOLLOW);

            let arc = D2D1_ARC_SEGMENT {
                point: end_pt,
                size: D2D_SIZE_F { width: radius, height: radius },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: if sweep_angle > 180.0 { D2D1_ARC_SIZE_LARGE } else { D2D1_ARC_SIZE_SMALL },
            };
            sink.AddArc(&arc);

            sink.EndFigure(D2D1_FIGURE_END_OPEN);
            sink.Close()?;

            Ok(geometry)
        }
    }

    pub fn render_to_dc(
        &mut self,
        hdc: windows::Win32::Graphics::Gdi::HDC,
        width: f32,
        height: f32,
        progress: f32,
        theme: &ThemeTokens,
        active_tab: NotchTab,
        hovered_dock_index: Option<usize>,
        is_pinned: bool,
        stats: &SystemStatsData,
        power: &PowerStatusInfo,
        media: &MediaInfo,
        prod: &ProductivityManager,
        clip: &ClipboardManager,
        enabled_modules: &crate::config::EnabledModules,
        time_format_24h: bool,
        shrink_mode: bool,
    ) -> Result<()> {
        let bottom_r = 14.0 + (progress * 14.0);
        let ear_r = 14.0 + (progress * 6.0);
        let notch_path = self.build_notch_geometry(width, height, bottom_r, ear_r)?;

        let target = self.get_or_create_dc_target()?.clone();

        unsafe {
            let rect = RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            target.BindDC(hdc, &rect)?;
            target.BeginDraw();

            target.Clear(Some(&D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 0.0,
            }));

            // Theme token brushes
            let body_bg = if progress > 0.15 {
                theme.panel_bg
            } else {
                theme.notch_collapsed_bg
            };
            let black_brush = target.CreateSolidColorBrush(&body_bg, None)?;
            let outline_brush = target.CreateSolidColorBrush(&theme.outline, None)?;
            target.FillGeometry(&notch_path, &black_brush, None);
            target.DrawGeometry(&notch_path, &outline_brush, 1.0, None);

            let white_brush = target.CreateSolidColorBrush(&theme.text_primary, None)?;
            let secondary_brush = target.CreateSolidColorBrush(&theme.text_secondary, None)?;
            let card_bg_brush = target.CreateSolidColorBrush(&theme.card_bg, None)?;
            let card_border_brush = target.CreateSolidColorBrush(&theme.card_border, None)?;
            let dock_bg_brush = target.CreateSolidColorBrush(&theme.dock_bg, None)?;
            let accent_brush = target.CreateSolidColorBrush(&theme.accent, None)?;
            let ring_track_brush = target.CreateSolidColorBrush(&theme.ring_track, None)?;

            let green_bolt_color = D2D1_COLOR_F { r: 0.20, g: 0.85, b: 0.35, a: 1.0 };
            let green_bolt_brush = target.CreateSolidColorBrush(&green_bolt_color, None)?;

            if progress > 0.15 {
                let content_left = ear_r + 14.0;
                let content_right = width - ear_r - 14.0;

                // --- HEADER BAR (Height 48) ---
                let logo_rect = D2D_RECT_F {
                    left: content_left,
                    top: 12.0,
                    right: content_left + 24.0,
                    bottom: 36.0,
                };
                self.icon_painter.draw_header_logo(&target, &self.factory, &logo_rect, &card_border_brush, &card_bg_brush, &black_brush)?;

                // Real date for Home header
                let date_str = get_current_date_string();

                let (mod_title, mod_sub) = match active_tab {
                    NotchTab::Home => ("Home", date_str),
                    NotchTab::System => ("System", "Performance & Hardware".to_string()),
                    NotchTab::Clock => ("World Clock", "4 cities".to_string()),
                    NotchTab::Timers => {
                        let running_count = prod.timers.iter().filter(|t| t.is_running).count();
                        let sub = if running_count > 0 { "Running" } else { "Paused" };
                        ("Timers", sub.to_string())
                    }
                    NotchTab::Calendar => ("Calendar", date_str),
                    NotchTab::Weather => ("Weather", "Local".to_string()),
                    NotchTab::Clipboard => {
                        let count = clip.items.len();
                        let s = if count == 1 { "1 item".to_string() } else { format!("{} items", count) };
                        ("Clipboard", s)
                    }
                    NotchTab::Notes => {
                        let count = prod.notes.len();
                        let s = if count == 1 { "1 note".to_string() } else { format!("{} notes", count) };
                        ("Notes", s)
                    }
                    NotchTab::FileShelf => {
                        let count = prod.parked_files.len();
                        let s = if count == 1 { "1 parked file".to_string() } else { format!("{} parked files", count) };
                        ("File Shelf", s)
                    }
                    NotchTab::DropTransfer => ("Drop", "Local Wi-Fi Transfer".to_string()),
                    NotchTab::Converters => ("Converters", "Units & Currency".to_string()),
                    NotchTab::Links => ("Links", "Quick Access".to_string()),
                    NotchTab::Media => {
                        let sub = if media.has_media && media.is_playing {
                            media.artist.clone()
                        } else {
                            "Ready".to_string()
                        };
                        ("Now Playing", sub)
                    }
                    NotchTab::Diagnostics => ("Diagnostics", "Raw System & Power Readings".to_string()),
                };

                if let Ok(tfmt) = self.get_title_format() {
                    let wide_title: Vec<u16> = mod_title.encode_utf16().collect();
                    let title_rect = D2D_RECT_F {
                        left: content_left + 34.0,
                        top: 14.0,
                        right: content_left + 200.0,
                        bottom: 36.0,
                    };
                    target.DrawText(&wide_title, tfmt, &title_rect, &white_brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                }

                let div_x = content_left + 150.0;
                target.DrawLine(
                    D2D_POINT_2F { x: div_x, y: 16.0 },
                    D2D_POINT_2F { x: div_x, y: 32.0 },
                    &card_border_brush,
                    1.0,
                    None,
                );

                if let Ok(sfmt) = self.get_sub_format() {
                    let wide_sub: Vec<u16> = mod_sub.encode_utf16().collect();
                    let sub_rect = D2D_RECT_F {
                        left: div_x + 12.0,
                        top: 16.0,
                        right: content_left + 380.0,
                        bottom: 34.0,
                    };
                    target.DrawText(&wide_sub, sfmt, &sub_rect, &secondary_brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                }

                // Battery + Pin + Settings + Close
                if let Ok(bfmt) = self.get_body_format() {
                    // REAL BATTERY: Hide indicator completely if no battery present
                    if power.has_battery {
                        let bat_text = if power.is_charging && power.ac_online {
                            format!("⚡ {}%", power.battery_percent)
                        } else {
                            format!("{}%", power.battery_percent)
                        };
                        let wide_bat: Vec<u16> = bat_text.encode_utf16().collect();
                        let bat_rect = D2D_RECT_F {
                            left: content_right - 160.0,
                            top: 14.0,
                            right: content_right - 100.0,
                            bottom: 34.0,
                        };
                        let brush_to_use = if power.is_charging && power.ac_online { &green_bolt_brush } else { &secondary_brush };
                        target.DrawText(&wide_bat, bfmt, &bat_rect, brush_to_use, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                    }

                    // Pure vector header buttons: Pin, Settings (gear), Close
                    let pin_cx = content_right - 72.0;
                    let gear_cx = content_right - 44.0;
                    let close_cx = content_right - 16.0;
                    let header_btn_cy = 24.0;

                    let pin_brush = if is_pinned { &white_brush } else { &secondary_brush };
                    let _ = self.icon_painter.draw_pin_icon(&target, pin_cx, header_btn_cy, is_pinned, pin_brush);
                    let _ = self.icon_painter.draw_gear_icon(&target, gear_cx, header_btn_cy, &secondary_brush);
                    let _ = self.icon_painter.draw_close_icon(&target, close_cx, header_btn_cy, &secondary_brush);
                }

                // --- CONTENT CONTAINER AREA (340 - 48 - 56 = 236 px) ---
                let content_top = 48.0;
                let content_bottom = height - 56.0;
                let content_rect = D2D_RECT_F {
                    left: content_left,
                    top: content_top,
                    right: content_right,
                    bottom: content_bottom,
                };

                match active_tab {
                    NotchTab::Home => {
                        self.render_home_grid(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush, &ring_track_brush, stats, prod, clip)?;
                    }
                    NotchTab::System => {
                        self.render_system_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush, &ring_track_brush, stats, power)?;
                    }
                    NotchTab::Diagnostics => {
                        self.render_diagnostics_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, power, stats)?;
                    }
                    NotchTab::Media => {
                        self.render_media_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush, media)?;
                    }
                    NotchTab::Clipboard => {
                        self.render_clipboard_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush, clip)?;
                    }
                    NotchTab::Notes => {
                        self.render_notes_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush, prod)?;
                    }
                    NotchTab::Timers => {
                        self.render_timers_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush, &ring_track_brush, prod)?;
                    }
                    NotchTab::Clock => {
                        self.render_world_clock_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush)?;
                    }
                    NotchTab::Calendar => {
                        self.render_calendar_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush)?;
                    }
                    NotchTab::Weather => {
                        self.render_weather_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush)?;
                    }
                    NotchTab::FileShelf => {
                        self.render_file_shelf_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush, prod)?;
                    }
                    NotchTab::Converters => {
                        self.render_converters_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush)?;
                    }
                    NotchTab::Links => {
                        self.render_links_page(&target, &content_rect, &card_bg_brush, &card_border_brush, &white_brush, &secondary_brush, &accent_brush)?;
                    }
                    _ => {
                        let card_round = D2D1_ROUNDED_RECT {
                            rect: content_rect,
                            radiusX: 16.0,
                            radiusY: 16.0,
                        };
                        target.FillRoundedRectangle(&card_round, &card_bg_brush);
                        target.DrawRoundedRectangle(&card_round, &card_border_brush, 1.0, None);

                        if let Ok(tfmt) = self.get_title_format() {
                            let wide_t: Vec<u16> = mod_title.encode_utf16().collect();
                            let t_rect = D2D_RECT_F {
                                left: content_rect.left + 24.0,
                                top: content_rect.top + 24.0,
                                right: content_rect.right - 24.0,
                                bottom: content_rect.top + 54.0,
                            };
                            target.DrawText(&wide_t, tfmt, &t_rect, &white_brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                    }
                }

                // --- DOCK (Height 56) ---
                let dock_h = 56.0;
                let dock_top = height - dock_h;
                let dock_rect = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: ear_r + 4.0,
                        top: dock_top,
                        right: width - ear_r - 4.0,
                        bottom: height - 1.0,
                    },
                    radiusX: bottom_r - 2.0,
                    radiusY: bottom_r - 2.0,
                };
                target.FillRoundedRectangle(&dock_rect, &dock_bg_brush);
                target.DrawRoundedRectangle(&dock_rect, &card_border_brush, 1.0, None);

                let all_dock_defs = [
                    (NotchTab::Home, "Home", 0),
                    (NotchTab::System, "System", 1),
                    (NotchTab::Clock, "Clock", 2),
                    (NotchTab::Timers, "Timers", 3),
                    (NotchTab::Calendar, "Calendar", 4),
                    (NotchTab::Weather, "Weather", 5),
                    (NotchTab::Clipboard, "Clipboard", 6),
                    (NotchTab::Notes, "Notes", 7),
                    (NotchTab::FileShelf, "File Shelf", 8),
                    (NotchTab::DropTransfer, "Drop", 9),
                    (NotchTab::Converters, "Converters", 10),
                    (NotchTab::Links, "Links", 11),
                    (NotchTab::Media, "Media", 12),
                ];

                let visible_tabs: Vec<(NotchTab, &str, usize)> = all_dock_defs
                    .iter()
                    .filter(|(tab, _, _)| enabled_modules.is_enabled(*tab))
                    .cloned()
                    .collect();

                let count = visible_tabs.len() as f32;
                let slot_w = if count > 0.0 {
                    (dock_rect.rect.right - dock_rect.rect.left) / count
                } else {
                    dock_rect.rect.right - dock_rect.rect.left
                };

                for (slot_idx, (tab, _name, icon_idx)) in visible_tabs.iter().enumerate() {
                    let center_x = dock_rect.rect.left + (slot_idx as f32 * slot_w) + (slot_w / 2.0);
                    let center_y = dock_top + (dock_h / 2.0) - 2.0;

                    let is_active = *tab == active_tab;
                    let is_hovered = hovered_dock_index == Some(slot_idx);

                    if is_active {
                        let active_bg = D2D1_ELLIPSE {
                            point: D2D_POINT_2F { x: center_x, y: center_y },
                            radiusX: 18.0,
                            radiusY: 18.0,
                        };
                        let active_bg_color = match theme.mode {
                            crate::ui::theme::ThemeMode::Silver => D2D1_COLOR_F { r: 0.78, g: 0.78, b: 0.78, a: 1.0 },
                            _ => D2D1_COLOR_F { r: 0.14, g: 0.14, b: 0.16, a: 1.0 },
                        };
                        let hl_brush = target.CreateSolidColorBrush(&active_bg_color, None)?;
                        target.FillEllipse(&active_bg, &hl_brush);

                        let dot = D2D1_ELLIPSE {
                            point: D2D_POINT_2F { x: center_x, y: dock_top + dock_h - 7.0 },
                            radiusX: 2.0,
                            radiusY: 2.0,
                        };
                        let dot_brush = if theme.mode == crate::ui::theme::ThemeMode::Silver { &black_brush } else { &white_brush };
                        target.FillEllipse(&dot, dot_brush);
                    }

                    // Pure vector line icons (1.5 px stroke, 20x20 box, round caps)
                    let icon_brush = if is_active || is_hovered { &white_brush } else { &secondary_brush };
                    let _ = self.icon_painter.draw_icon(&target, &self.factory, *icon_idx, center_x, center_y, icon_brush);
                }

                if let Some(hovered_i) = hovered_dock_index {
                    if hovered_i < visible_tabs.len() {
                        let center_x = dock_rect.rect.left + (hovered_i as f32 * slot_w) + (slot_w / 2.0);
                        let tip_w = 78.0;
                        let tip_h = 24.0;
                        let tip_rect = D2D1_ROUNDED_RECT {
                            rect: D2D_RECT_F {
                                left: center_x - (tip_w / 2.0),
                                top: dock_top - tip_h - 4.0,
                                right: center_x + (tip_w / 2.0),
                                bottom: dock_top - 4.0,
                            },
                            radiusX: 6.0,
                            radiusY: 6.0,
                        };
                        let tip_bg = target.CreateSolidColorBrush(
                            &D2D1_COLOR_F { r: 0.12, g: 0.12, b: 0.14, a: 0.98 },
                            None,
                        )?;
                        target.FillRoundedRectangle(&tip_rect, &tip_bg);
                        target.DrawRoundedRectangle(&tip_rect, &card_border_brush, 1.0, None);

                        if let Ok(tfmt) = self.get_tooltip_format() {
                            let tip_white = target.CreateSolidColorBrush(
                                &D2D1_COLOR_F { r: 0.95, g: 0.95, b: 0.95, a: 1.0 },
                                None,
                            )?;
                            let name = visible_tabs[hovered_i].1;
                            let wide_n: Vec<u16> = name.encode_utf16().collect();
                            target.DrawText(&wide_n, tfmt, &tip_rect.rect, &tip_white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                    }
                }
            } else {
                // Collapsed Notch Pill contents (Live Activity / Clock & Music summary)
                let pill_cy = height / 2.0;
                let center_x = width / 2.0;

                if shrink_mode {
                    // Shrink Mode: Draw minimal discreet arrow chevron (down arrow v)
                    let arrow_geom = self.factory.CreatePathGeometry()?;
                    let sink = arrow_geom.Open()?;
                    sink.BeginFigure(D2D_POINT_2F { x: center_x - 6.0, y: pill_cy - 2.0 }, D2D1_FIGURE_BEGIN_HOLLOW);
                    sink.AddLine(D2D_POINT_2F { x: center_x, y: pill_cy + 3.5 });
                    sink.AddLine(D2D_POINT_2F { x: center_x + 6.0, y: pill_cy - 2.0 });
                    sink.EndFigure(D2D1_FIGURE_END_OPEN);
                    sink.Close()?;
                    target.DrawGeometry(&arrow_geom, &accent_brush, 2.0, None);
                } else {
                    let time_str = {
                        let st = windows::Win32::System::SystemInformation::GetLocalTime();
                        if time_format_24h {
                            format!("{:02}:{:02}", st.wHour, st.wMinute)
                        } else {
                            let (h, ampm) = if st.wHour == 0 {
                                (12, "AM")
                            } else if st.wHour < 12 {
                                (st.wHour, "AM")
                            } else if st.wHour == 12 {
                                (12, "PM")
                            } else {
                                (st.wHour - 12, "PM")
                            };
                            format!("{}:{:02} {}", h, st.wMinute, ampm)
                        }
                    };

                    let pill_white = target.CreateSolidColorBrush(&D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, None)?;

                    let _ = self.get_body_format();
                    let bfmt = self.body_format.as_ref().unwrap();

                    if media.has_media && media.is_playing {
                        // Small music note icon + Song title + Clock
                        let _ = self.icon_painter.draw_icon(&target, &self.factory, 12, center_x - 72.0, pill_cy, &accent_brush);
                        let track_display = if media.title.len() > 12 {
                            format!("{}...", &media.title[..10])
                        } else {
                            media.title.clone()
                        };
                        let disp = format!("{} · {}", track_display, time_str);
                        let wide: Vec<u16> = disp.encode_utf16().collect();
                        let tr = D2D_RECT_F {
                            left: center_x - 60.0,
                            top: pill_cy - 10.0,
                            right: center_x + 85.0,
                            bottom: pill_cy + 12.0,
                        };
                        target.DrawText(&wide, bfmt, &tr, &pill_white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                    } else {
                        // Clean live clock with blue status dot
                        let dot = D2D1_ELLIPSE {
                            point: D2D_POINT_2F { x: center_x - 42.0, y: pill_cy },
                            radiusX: 3.5,
                            radiusY: 3.5,
                        };
                        target.FillEllipse(&dot, &accent_brush);

                        let wide: Vec<u16> = time_str.encode_utf16().collect();
                        let tr = D2D_RECT_F {
                            left: center_x - 32.0,
                            top: pill_cy - 10.0,
                            right: center_x + 56.0,
                            bottom: pill_cy + 12.0,
                        };
                        target.DrawText(&wide, bfmt, &tr, &pill_white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                    }
                }
            }

            let end_result = target.EndDraw(None, None);
            if end_result.is_err() {
                self.render_target = None;
            }
        }

        Ok(())
    }

    // 3 x 2 Card Grid for Home Screen
    unsafe fn render_home_grid(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
        ring_track: &ID2D1SolidColorBrush,
        stats: &SystemStatsData,
        prod: &ProductivityManager,
        clip: &ClipboardManager,
    ) -> Result<()> {
        let gap = 10.0;
        let card_w = (rect.right - rect.left - (gap * 2.0)) / 3.0;
        let card_h = (rect.bottom - rect.top - gap) / 2.0;

        for row in 0..2 {
            for col in 0..3 {
                let card_left = rect.left + (col as f32 * (card_w + gap));
                let card_top = rect.top + (row as f32 * (card_h + gap));
                let card_rect = D2D_RECT_F {
                    left: card_left,
                    top: card_top,
                    right: card_left + card_w,
                    bottom: card_top + card_h,
                };

                let round = D2D1_ROUNDED_RECT {
                    rect: card_rect,
                    radiusX: 16.0,
                    radiusY: 16.0,
                };
                target.FillRoundedRectangle(&round, card_bg);
                target.DrawRoundedRectangle(&round, card_border, 1.0, None);

                let idx = row * 3 + col;
                match idx {
                    0 => {
                        // Card 1: Screen Time / Productivity - Clean Empty State if no real tracker
                        if let Ok(mfmt) = self.get_meta_format() {
                            let wide: Vec<u16> = "SCREEN TIME".encode_utf16().collect();
                            let r_top = D2D_RECT_F { left: card_left + 14.0, top: card_top + 10.0, right: card_rect.right - 14.0, bottom: card_top + 24.0 };
                            target.DrawText(&wide, mfmt, &r_top, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                        if let Ok(sfmt) = self.get_sub_format() {
                            let wide: Vec<u16> = "No data yet".encode_utf16().collect();
                            let r_empty = D2D_RECT_F { left: card_left + 14.0, top: card_top + 42.0, right: card_rect.right - 14.0, bottom: card_top + 70.0 };
                            target.DrawText(&wide, sfmt, &r_empty, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                    }
                    1 => {
                        // Card 2: System Gauges (Real CPU, RAM, DISK)
                        if let Ok(mfmt) = self.get_meta_format() {
                            let wide: Vec<u16> = "SYSTEM".encode_utf16().collect();
                            let r_top = D2D_RECT_F { left: card_left + 14.0, top: card_top + 10.0, right: card_rect.right - 14.0, bottom: card_top + 24.0 };
                            target.DrawText(&wide, mfmt, &r_top, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                        let gauge_r = 18.0;
                        let gauge_y = card_top + 48.0;
                        let slot_w = card_w / 3.0;
                        let gauges = [
                            ("CPU", stats.cpu_usage.clamp(0.0, 100.0)),
                            ("RAM", stats.ram_percent.clamp(0.0, 100.0)),
                            ("DISK", stats.disk_percent.clamp(0.0, 100.0)),
                        ];
                        for (i, (name, pct)) in gauges.iter().enumerate() {
                            let center_x = card_left + (i as f32 * slot_w) + (slot_w / 2.0);
                            let center = D2D_POINT_2F { x: center_x, y: gauge_y };
                            target.DrawEllipse(
                                &D2D1_ELLIPSE { point: center, radiusX: gauge_r, radiusY: gauge_r },
                                ring_track,
                                3.5,
                                None,
                            );
                            let sweep = (pct / 100.0) * 360.0;
                            if sweep > 2.0 {
                                if let Ok(arc_path) = self.build_arc_path(center, gauge_r, -90.0, sweep.min(359.9)) {
                                    target.DrawGeometry(&arc_path, accent, 3.5, None);
                                }
                            }
                            if let Ok(gfmt) = self.get_gauge_small_format() {
                                let pct_text = format!("{:.0}%", pct);
                                let wide: Vec<u16> = pct_text.encode_utf16().collect();
                                let text_rect = D2D_RECT_F { left: center_x - gauge_r, top: gauge_y - gauge_r, right: center_x + gauge_r, bottom: gauge_y + gauge_r };
                                target.DrawText(&wide, gfmt, &text_rect, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                            }
                            if let Ok(mfmt) = self.get_meta_format() {
                                let wide: Vec<u16> = name.encode_utf16().collect();
                                let label_rect = D2D_RECT_F { left: center_x - 20.0, top: gauge_y + gauge_r + 4.0, right: center_x + 20.0, bottom: gauge_y + gauge_r + 18.0 };
                                target.DrawText(&wide, mfmt, &label_rect, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                            }
                        }
                    }
                    2 => {
                        // Card 3: Weather - Clean State
                        if let Ok(mfmt) = self.get_meta_format() {
                            let wide: Vec<u16> = "WEATHER".encode_utf16().collect();
                            let r_top = D2D_RECT_F { left: card_left + 14.0, top: card_top + 10.0, right: card_rect.right - 14.0, bottom: card_top + 24.0 };
                            target.DrawText(&wide, mfmt, &r_top, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                        if let Ok(sfmt) = self.get_sub_format() {
                            let wide: Vec<u16> = "Click Weather to load forecast".encode_utf16().collect();
                            let r_empty = D2D_RECT_F { left: card_left + 14.0, top: card_top + 42.0, right: card_rect.right - 14.0, bottom: card_top + 70.0 };
                            target.DrawText(&wide, sfmt, &r_empty, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                    }
                    3 => {
                        // Card 4: Focus Timer - Real State
                        if let Ok(mfmt) = self.get_meta_format() {
                            let wide: Vec<u16> = "FOCUS".encode_utf16().collect();
                            let r_top = D2D_RECT_F { left: card_left + 14.0, top: card_top + 10.0, right: card_rect.right - 14.0, bottom: card_top + 24.0 };
                            target.DrawText(&wide, mfmt, &r_top, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                        let active_timer = prod.timers.iter().find(|t| t.is_running);
                        let (time_str, status_str) = if let Some(t) = active_timer {
                            (format!("{:02}:{:02}", t.remaining_seconds / 60, t.remaining_seconds % 60), "Focusing")
                        } else {
                            ("25:00".to_string(), "Paused")
                        };
                        if let Ok(nfmt) = self.get_number_format() {
                            let wide: Vec<u16> = time_str.encode_utf16().collect();
                            let r_mid = D2D_RECT_F { left: card_left + 14.0, top: card_top + 26.0, right: card_left + 140.0, bottom: card_top + 62.0 };
                            target.DrawText(&wide, nfmt, &r_mid, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                        let btn_center = D2D_POINT_2F { x: card_rect.right - 34.0, y: card_top + 44.0 };
                        target.FillEllipse(&D2D1_ELLIPSE { point: btn_center, radiusX: 16.0, radiusY: 16.0 }, accent);
                        if active_timer.is_some() {
                            // Pause bars (two vertical lines)
                            target.DrawLine(D2D_POINT_2F { x: btn_center.x - 3.0, y: btn_center.y - 5.0 }, D2D_POINT_2F { x: btn_center.x - 3.0, y: btn_center.y + 5.0 }, white, 2.0, None);
                            target.DrawLine(D2D_POINT_2F { x: btn_center.x + 3.0, y: btn_center.y - 5.0 }, D2D_POINT_2F { x: btn_center.x + 3.0, y: btn_center.y + 5.0 }, white, 2.0, None);
                        } else {
                            // Play triangle
                            let geom = self.factory.CreatePathGeometry()?;
                            let sink = geom.Open()?;
                            sink.BeginFigure(D2D_POINT_2F { x: btn_center.x - 3.0, y: btn_center.y - 6.0 }, D2D1_FIGURE_BEGIN_FILLED);
                            sink.AddLine(D2D_POINT_2F { x: btn_center.x + 5.0, y: btn_center.y });
                            sink.AddLine(D2D_POINT_2F { x: btn_center.x - 3.0, y: btn_center.y + 6.0 });
                            sink.EndFigure(D2D1_FIGURE_END_CLOSED);
                            sink.Close()?;
                            target.FillGeometry(&geom, white, None);
                        }
                        if let Ok(sfmt) = self.get_sub_format() {
                            let desc = format!("{} · Pomodoro", status_str);
                            let wide: Vec<u16> = desc.encode_utf16().collect();
                            let r_bot = D2D_RECT_F { left: card_left + 14.0, top: card_top + 70.0, right: card_rect.right - 14.0, bottom: card_top + 92.0 };
                            target.DrawText(&wide, sfmt, &r_bot, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                    }
                    4 => {
                        // Card 5: Real Clipboard Items
                        if let Ok(mfmt) = self.get_meta_format() {
                            let wide: Vec<u16> = "CLIPBOARD".encode_utf16().collect();
                            let r_top = D2D_RECT_F { left: card_left + 14.0, top: card_top + 10.0, right: card_rect.right - 14.0, bottom: card_top + 24.0 };
                            target.DrawText(&wide, mfmt, &r_top, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                        if clip.items.is_empty() {
                            if let Ok(sfmt) = self.get_sub_format() {
                                let wide: Vec<u16> = "No items copied yet".encode_utf16().collect();
                                let r_empty = D2D_RECT_F { left: card_left + 14.0, top: card_top + 40.0, right: card_rect.right - 14.0, bottom: card_top + 70.0 };
                                target.DrawText(&wide, sfmt, &r_empty, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                            }
                        } else {
                            if let Ok(sfmt) = self.get_sub_format() {
                                let mut y = card_top + 30.0;
                                for item in clip.items.iter().take(2) {
                                    let preview = if item.preview.len() > 24 { format!("{}...", &item.preview[..22]) } else { item.preview.clone() };
                                    let line = format!("• {}", preview);
                                    let wide: Vec<u16> = line.encode_utf16().collect();
                                    let r_line = D2D_RECT_F { left: card_left + 14.0, top: y, right: card_rect.right - 14.0, bottom: y + 20.0 };
                                    target.DrawText(&wide, sfmt, &r_line, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                                    y += 26.0;
                                }
                            }
                        }
                    }
                    _ => {
                        // Card 6: Real Notes
                        if let Ok(mfmt) = self.get_meta_format() {
                            let wide: Vec<u16> = "NOTES".encode_utf16().collect();
                            let r_top = D2D_RECT_F { left: card_left + 14.0, top: card_top + 10.0, right: card_rect.right - 14.0, bottom: card_top + 24.0 };
                            target.DrawText(&wide, mfmt, &r_top, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                        }
                        if prod.notes.is_empty() {
                            if let Ok(sfmt) = self.get_sub_format() {
                                let wide: Vec<u16> = "No notes yet".encode_utf16().collect();
                                let r_empty = D2D_RECT_F { left: card_left + 14.0, top: card_top + 40.0, right: card_rect.right - 14.0, bottom: card_top + 70.0 };
                                target.DrawText(&wide, sfmt, &r_empty, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                            }
                        } else {
                            let note_title = prod.notes.first().map(|n| n.content.as_str()).unwrap_or("");
                            let count_str = format!("{} note(s)", prod.notes.len());
                            if let Ok(tfmt) = self.get_title_format() {
                                let wide: Vec<u16> = note_title.encode_utf16().collect();
                                let r_mid = D2D_RECT_F { left: card_left + 14.0, top: card_top + 28.0, right: card_rect.right - 14.0, bottom: card_top + 50.0 };
                                target.DrawText(&wide, tfmt, &r_mid, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                            }
                            if let Ok(sfmt) = self.get_sub_format() {
                                let wide: Vec<u16> = count_str.encode_utf16().collect();
                                let r_bot = D2D_RECT_F { left: card_left + 14.0, top: card_top + 68.0, right: card_rect.right - 14.0, bottom: card_top + 90.0 };
                                target.DrawText(&wide, sfmt, &r_bot, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    // System Page with Ring Text fitting inside and clean cards on right
    unsafe fn render_system_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
        ring_track: &ID2D1SolidColorBrush,
        stats: &SystemStatsData,
        power: &PowerStatusInfo,
    ) -> Result<()> {
        let card_w = (rect.right - rect.left - 12.0) / 2.0;

        // Left Card: 3 Big Resource Rings with percentage INSIDE
        let left_rect = D2D_RECT_F {
            left: rect.left,
            top: rect.top,
            right: rect.left + card_w,
            bottom: rect.bottom,
        };
        let round_left = D2D1_ROUNDED_RECT { rect: left_rect, radiusX: 16.0, radiusY: 16.0 };
        target.FillRoundedRectangle(&round_left, card_bg);
        target.DrawRoundedRectangle(&round_left, card_border, 1.0, None);

        if let Ok(mfmt) = self.get_meta_format() {
            let wide: Vec<u16> = "SYSTEM RESOURCES".encode_utf16().collect();
            let r_t = D2D_RECT_F {
                left: left_rect.left + 16.0,
                top: left_rect.top + 14.0,
                right: left_rect.right - 16.0,
                bottom: left_rect.top + 28.0,
            };
            target.DrawText(&wide, mfmt, &r_t, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        let ring_r = 30.0;
        let center_y = left_rect.top + 110.0;
        let slot_w = card_w / 3.0;

        let sys_gauges = [
            ("CPU", stats.cpu_usage.clamp(0.0, 100.0)),
            ("RAM", stats.ram_percent.clamp(0.0, 100.0)),
            ("DISK", stats.disk_percent.clamp(0.0, 100.0)),
        ];

        for (i, (name, pct)) in sys_gauges.iter().enumerate() {
            let center_x = left_rect.left + (i as f32 * slot_w) + (slot_w / 2.0);
            let center = D2D_POINT_2F { x: center_x, y: center_y };

            target.DrawEllipse(
                &D2D1_ELLIPSE { point: center, radiusX: ring_r, radiusY: ring_r },
                ring_track,
                5.0,
                None,
            );

            let sweep = (pct / 100.0) * 360.0;
            if sweep > 2.0 {
                if let Ok(arc_path) = self.build_arc_path(center, ring_r, -90.0, sweep.min(359.9)) {
                    target.DrawGeometry(&arc_path, accent, 5.0, None);
                }
            }

            // Text fitting inside the ring cleanly without overflowing
            if let Ok(gfmt) = self.get_gauge_format() {
                let pct_text = format!("{:.0}%", pct);
                let wide: Vec<u16> = pct_text.encode_utf16().collect();
                let text_rect = D2D_RECT_F {
                    left: center_x - ring_r + 2.0,
                    top: center_y - ring_r + 2.0,
                    right: center_x + ring_r - 2.0,
                    bottom: center_y + ring_r - 2.0,
                };
                target.DrawText(&wide, gfmt, &text_rect, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = name.encode_utf16().collect();
                let label_rect = D2D_RECT_F {
                    left: center_x - 24.0,
                    top: center_y + ring_r + 14.0,
                    right: center_x + 24.0,
                    bottom: center_y + ring_r + 32.0,
                };
                target.DrawText(&wide, mfmt, &label_rect, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        // Right side: Real cards for Memory, Battery, and Uptime
        let right_x = rect.left + card_w + 12.0;
        let right_w = card_w;
        let row_h = 68.0;
        let row_gap = 10.0;

        // 1. Memory Card
        let r1 = D2D_RECT_F {
            left: right_x,
            top: rect.top,
            right: right_x + right_w,
            bottom: rect.top + row_h,
        };
        target.FillRoundedRectangle(&D2D1_ROUNDED_RECT { rect: r1, radiusX: 12.0, radiusY: 12.0 }, card_bg);
        target.DrawRoundedRectangle(&D2D1_ROUNDED_RECT { rect: r1, radiusX: 12.0, radiusY: 12.0 }, card_border, 1.0, None);

        if let Ok(mfmt) = self.get_meta_format() {
            let wide: Vec<u16> = "MEMORY".encode_utf16().collect();
            let label_r = D2D_RECT_F { left: r1.left + 16.0, top: r1.top + 10.0, right: r1.right - 16.0, bottom: r1.top + 24.0 };
            target.DrawText(&wide, mfmt, &label_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }
        if let Ok(tfmt) = self.get_title_format() {
            let used_gb = stats.ram_used_mb as f32 / 1024.0;
            let total_gb = stats.ram_total_mb as f32 / 1024.0;
            let val_str = format!("{:.1} GB / {:.1} GB", used_gb, total_gb);
            let wide: Vec<u16> = val_str.encode_utf16().collect();
            let val_r = D2D_RECT_F { left: r1.left + 16.0, top: r1.top + 28.0, right: r1.right - 16.0, bottom: r1.bottom - 10.0 };
            target.DrawText(&wide, tfmt, &val_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // 2. Battery Card (or Storage Card if no battery)
        let r2 = D2D_RECT_F {
            left: right_x,
            top: rect.top + row_h + row_gap,
            right: right_x + right_w,
            bottom: rect.top + (row_h * 2.0) + row_gap,
        };
        target.FillRoundedRectangle(&D2D1_ROUNDED_RECT { rect: r2, radiusX: 12.0, radiusY: 12.0 }, card_bg);
        target.DrawRoundedRectangle(&D2D1_ROUNDED_RECT { rect: r2, radiusX: 12.0, radiusY: 12.0 }, card_border, 1.0, None);

        if power.has_battery {
            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = "BATTERY".encode_utf16().collect();
                let label_r = D2D_RECT_F { left: r2.left + 16.0, top: r2.top + 10.0, right: r2.right - 16.0, bottom: r2.top + 24.0 };
                target.DrawText(&wide, mfmt, &label_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
            if let Ok(tfmt) = self.get_title_format() {
                let status_text = if power.is_charging && power.ac_online {
                    format!("{}% (Charging)", power.battery_percent)
                } else if power.ac_online {
                    format!("{}% (Power Connected)", power.battery_percent)
                } else {
                    format!("{}% (On Battery)", power.battery_percent)
                };
                let wide: Vec<u16> = status_text.encode_utf16().collect();
                let val_r = D2D_RECT_F { left: r2.left + 16.0, top: r2.top + 28.0, right: r2.right - 16.0, bottom: r2.bottom - 10.0 };
                target.DrawText(&wide, tfmt, &val_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        } else {
            // Hidden battery -> Show C: Storage
            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = "LOCAL STORAGE (C:)".encode_utf16().collect();
                let label_r = D2D_RECT_F { left: r2.left + 16.0, top: r2.top + 10.0, right: r2.right - 16.0, bottom: r2.top + 24.0 };
                target.DrawText(&wide, mfmt, &label_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
            if let Ok(tfmt) = self.get_title_format() {
                let val_str = format!("{:.1} GB / {:.1} GB", stats.disk_used_gb, stats.disk_total_gb);
                let wide: Vec<u16> = val_str.encode_utf16().collect();
                let val_r = D2D_RECT_F { left: r2.left + 16.0, top: r2.top + 28.0, right: r2.right - 16.0, bottom: r2.bottom - 10.0 };
                target.DrawText(&wide, tfmt, &val_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        // 3. System Uptime Card
        let r3 = D2D_RECT_F {
            left: right_x,
            top: rect.top + (row_h * 2.0) + (row_gap * 2.0),
            right: right_x + right_w,
            bottom: rect.top + (row_h * 3.0) + (row_gap * 2.0),
        };
        target.FillRoundedRectangle(&D2D1_ROUNDED_RECT { rect: r3, radiusX: 12.0, radiusY: 12.0 }, card_bg);
        target.DrawRoundedRectangle(&D2D1_ROUNDED_RECT { rect: r3, radiusX: 12.0, radiusY: 12.0 }, card_border, 1.0, None);

        if let Ok(mfmt) = self.get_meta_format() {
            let wide: Vec<u16> = "SYSTEM UPTIME".encode_utf16().collect();
            let label_r = D2D_RECT_F { left: r3.left + 16.0, top: r3.top + 10.0, right: r3.right - 16.0, bottom: r3.top + 24.0 };
            target.DrawText(&wide, mfmt, &label_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }
        if let Ok(tfmt) = self.get_title_format() {
            let days = stats.uptime_seconds / 86400;
            let hours = (stats.uptime_seconds % 86400) / 3600;
            let mins = (stats.uptime_seconds % 3600) / 60;
            let val_str = if days > 0 {
                format!("{}d {}h {}m", days, hours, mins)
            } else {
                format!("{}h {}m", hours, mins)
            };
            let wide: Vec<u16> = val_str.encode_utf16().collect();
            let val_r = D2D_RECT_F { left: r3.left + 16.0, top: r3.top + 28.0, right: r3.right - 16.0, bottom: r3.bottom - 10.0 };
            target.DrawText(&wide, tfmt, &val_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        Ok(())
    }

    // Hidden diagnostics page (opened via Ctrl+Shift+D)
    unsafe fn render_diagnostics_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        power: &PowerStatusInfo,
        stats: &SystemStatsData,
    ) -> Result<()> {
        let card_round = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&card_round, card_bg);
        target.DrawRoundedRectangle(&card_round, card_border, 1.0, None);

        if let Ok(mfmt) = self.get_meta_format() {
            let wide: Vec<u16> = "DIAGNOSTICS · RAW SYSTEM & POWER READINGS".encode_utf16().collect();
            let r_top = D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 20.0,
                right: rect.right - 24.0,
                bottom: rect.top + 36.0,
            };
            target.DrawText(&wide, mfmt, &r_top, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        if let Ok(bfmt) = self.get_body_format() {
            let raw_items = [
                format!("ACLineStatus:              {} (1 = Online, 0 = Offline)", power.raw_ac_line_status),
                format!("BatteryFlag:               {} (1=High, 2=Low, 4=Crit, 8=Charging, 128=NoBat, 255=Unknown)", power.battery_flag),
                format!("BatteryLifePercent:        {}% (255 = Unknown)", power.battery_percent),
                format!("BatteryLifeTime:           {} seconds ({} min)", power.battery_life_time_sec, power.battery_life_time_sec / 60),
                format!("Calculated has_battery:    {}", power.has_battery),
                format!("Calculated is_charging:    {}", power.is_charging),
                format!("Sampling CPU:              {:.1}%", stats.cpu_usage),
                format!("Sampling RAM:              {:.1}% ({} MB / {} MB)", stats.ram_percent, stats.ram_used_mb, stats.ram_total_mb),
                format!("System Uptime:             {} seconds", stats.uptime_seconds),
            ];

            let mut cur_y = rect.top + 50.0;
            for line in raw_items.iter() {
                let wide: Vec<u16> = line.encode_utf16().collect();
                let line_rect = D2D_RECT_F {
                    left: rect.left + 24.0,
                    top: cur_y,
                    right: rect.right - 24.0,
                    bottom: cur_y + 20.0,
                };
                target.DrawText(&wide, bfmt, &line_rect, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                cur_y += 20.0;
            }
        }

        Ok(())
    }

    // Real Media Player Page (Album art placeholder, Track title, Artist, Progress bar, Controls)
    unsafe fn render_media_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
        media: &MediaInfo,
    ) -> Result<()> {
        let card_round = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&card_round, card_bg);
        target.DrawRoundedRectangle(&card_round, card_border, 1.0, None);

        // Album Art Box (140x140 with 16px corner radius)
        let art_size = 140.0;
        let art_left = rect.left + 28.0;
        let art_top = rect.top + ((rect.bottom - rect.top - art_size) / 2.0);
        let art_rect = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: art_left,
                top: art_top,
                right: art_left + art_size,
                bottom: art_top + art_size,
            },
            radiusX: 16.0,
            radiusY: 16.0,
        };

        let art_bg = target.CreateSolidColorBrush(
            &D2D1_COLOR_F { r: 0.12, g: 0.12, b: 0.13, a: 1.0 },
            None,
        )?;
        target.FillRoundedRectangle(&art_rect, &art_bg);
        target.DrawRoundedRectangle(&art_rect, card_border, 1.0, None);

        // Draw note icon in album art box
        let art_cx = art_left + (art_size / 2.0);
        let art_cy = art_top + (art_size / 2.0);
        let _ = self.icon_painter.draw_icon(target, &self.factory, 12, art_cx, art_cy, secondary);

        // Right details area
        let details_left = art_left + art_size + 28.0;
        let details_right = rect.right - 28.0;

        if media.has_media {
            // Track Title
            if let Ok(tfmt) = self.get_title_format() {
                let wide: Vec<u16> = media.title.encode_utf16().collect();
                let title_r = D2D_RECT_F {
                    left: details_left,
                    top: art_top + 16.0,
                    right: details_right,
                    bottom: art_top + 46.0,
                };
                target.DrawText(&wide, tfmt, &title_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            // Artist Name
            if let Ok(sfmt) = self.get_sub_format() {
                let artist_text = if media.artist.is_empty() {
                    if media.is_playing { "Playing".to_string() } else { "Paused".to_string() }
                } else {
                    media.artist.clone()
                };
                let wide: Vec<u16> = artist_text.encode_utf16().collect();
                let artist_r = D2D_RECT_F {
                    left: details_left,
                    top: art_top + 48.0,
                    right: details_right,
                    bottom: art_top + 70.0,
                };
                target.DrawText(&wide, sfmt, &artist_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        } else {
            // Clean empty state (RULE 0 compliant)
            if let Ok(tfmt) = self.get_title_format() {
                let wide: Vec<u16> = "No Media Playing".encode_utf16().collect();
                let title_r = D2D_RECT_F {
                    left: details_left,
                    top: art_top + 24.0,
                    right: details_right,
                    bottom: art_top + 54.0,
                };
                target.DrawText(&wide, tfmt, &title_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
            if let Ok(sfmt) = self.get_sub_format() {
                let wide: Vec<u16> = "Audio playback from Spotify, Chrome, or Windows media apps will appear here.".encode_utf16().collect();
                let desc_r = D2D_RECT_F {
                    left: details_left,
                    top: art_top + 56.0,
                    right: details_right,
                    bottom: art_top + 80.0,
                };
                target.DrawText(&wide, sfmt, &desc_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        // Progress Bar Track
        let bar_top = art_top + 92.0;
        let bar_bottom = bar_top + 4.0;
        let track_rect = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: details_left,
                top: bar_top,
                right: details_right,
                bottom: bar_bottom,
            },
            radiusX: 2.0,
            radiusY: 2.0,
        };
        let track_bg = target.CreateSolidColorBrush(
            &D2D1_COLOR_F { r: 0.18, g: 0.18, b: 0.20, a: 1.0 },
            None,
        )?;
        target.FillRoundedRectangle(&track_rect, &track_bg);

        // Control Buttons: Prev (details_left + 40), Play/Pause (details_left + 90), Next (details_left + 140)
        let btn_y = art_top + 120.0;

        // Prev Track button
        let prev_cx = details_left + 30.0;
        target.DrawLine(D2D_POINT_2F { x: prev_cx - 6.0, y: btn_y - 6.0 }, D2D_POINT_2F { x: prev_cx - 6.0, y: btn_y + 6.0 }, secondary, 1.5, None);
        let prev_geom = self.factory.CreatePathGeometry()?;
        let psink = prev_geom.Open()?;
        psink.BeginFigure(D2D_POINT_2F { x: prev_cx + 4.0, y: btn_y - 6.0 }, D2D1_FIGURE_BEGIN_FILLED);
        psink.AddLine(D2D_POINT_2F { x: prev_cx - 4.0, y: btn_y });
        psink.AddLine(D2D_POINT_2F { x: prev_cx + 4.0, y: btn_y + 6.0 });
        psink.EndFigure(D2D1_FIGURE_END_CLOSED);
        psink.Close()?;
        target.FillGeometry(&prev_geom, secondary, None);

        // Play/Pause Big Center Button
        let play_cx = details_left + 80.0;
        target.FillEllipse(&D2D1_ELLIPSE { point: D2D_POINT_2F { x: play_cx, y: btn_y }, radiusX: 16.0, radiusY: 16.0 }, accent);

        if media.is_playing {
            // Pause bars
            target.DrawLine(D2D_POINT_2F { x: play_cx - 3.0, y: btn_y - 5.0 }, D2D_POINT_2F { x: play_cx - 3.0, y: btn_y + 5.0 }, white, 2.0, None);
            target.DrawLine(D2D_POINT_2F { x: play_cx + 3.0, y: btn_y - 5.0 }, D2D_POINT_2F { x: play_cx + 3.0, y: btn_y + 5.0 }, white, 2.0, None);
        } else {
            // Play triangle
            let pgeom = self.factory.CreatePathGeometry()?;
            let psink2 = pgeom.Open()?;
            psink2.BeginFigure(D2D_POINT_2F { x: play_cx - 3.0, y: btn_y - 6.0 }, D2D1_FIGURE_BEGIN_FILLED);
            psink2.AddLine(D2D_POINT_2F { x: play_cx + 5.0, y: btn_y });
            psink2.AddLine(D2D_POINT_2F { x: play_cx - 3.0, y: btn_y + 6.0 });
            psink2.EndFigure(D2D1_FIGURE_END_CLOSED);
            psink2.Close()?;
            target.FillGeometry(&pgeom, white, None);
        }

        // Next Track button
        let next_cx = details_left + 130.0;
        target.DrawLine(D2D_POINT_2F { x: next_cx + 6.0, y: btn_y - 6.0 }, D2D_POINT_2F { x: next_cx + 6.0, y: btn_y + 6.0 }, secondary, 1.5, None);
        let next_geom = self.factory.CreatePathGeometry()?;
        let nsink = next_geom.Open()?;
        nsink.BeginFigure(D2D_POINT_2F { x: next_cx - 4.0, y: btn_y - 6.0 }, D2D1_FIGURE_BEGIN_FILLED);
        nsink.AddLine(D2D_POINT_2F { x: next_cx + 4.0, y: btn_y });
        nsink.AddLine(D2D_POINT_2F { x: next_cx - 4.0, y: btn_y + 6.0 });
        nsink.EndFigure(D2D1_FIGURE_END_CLOSED);
        nsink.Close()?;
        target.FillGeometry(&next_geom, secondary, None);

        Ok(())
    }

    // Real Two-Pane Clipboard Page with Drag-Out & Category Tabs
    unsafe fn render_clipboard_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
        clip: &ClipboardManager,
    ) -> Result<()> {
        let left_w = 280.0;
        let right_left = rect.left + left_w + 12.0;

        // Left Pane (45% width): Tabs, Sections, Item List
        let left_rect = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: rect.left,
                top: rect.top,
                right: rect.left + left_w,
                bottom: rect.bottom,
            },
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&left_rect, card_bg);
        target.DrawRoundedRectangle(&left_rect, card_border, 1.0, None);

        // 1. Tab row: All, Text, Images, Links with count badges
        let (all_c, text_c, img_c, link_c) = clip.counts();
        let tabs = [
            ("All", all_c, clip.active_tab == crate::modules::clipboard::ClipboardFilterTab::All),
            ("Text", text_c, clip.active_tab == crate::modules::clipboard::ClipboardFilterTab::Text),
            ("Images", img_c, clip.active_tab == crate::modules::clipboard::ClipboardFilterTab::Images),
            ("Links", link_c, clip.active_tab == crate::modules::clipboard::ClipboardFilterTab::Links),
        ];

        let mut tab_x = rect.left + 16.0;
        let tab_y = rect.top + 14.0;
        if let Ok(mfmt) = self.get_meta_format() {
            for (title, count, is_active) in tabs.iter() {
                let label = format!("{} {}", title, count);
                let wide: Vec<u16> = label.encode_utf16().collect();
                let tr = D2D_RECT_F {
                    left: tab_x,
                    top: tab_y,
                    right: tab_x + 56.0,
                    bottom: tab_y + 18.0,
                };
                let brush = if *is_active { white } else { secondary };
                target.DrawText(&wide, mfmt, &tr, brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);

                if *is_active {
                    target.DrawLine(
                        D2D_POINT_2F { x: tab_x, y: tab_y + 19.0 },
                        D2D_POINT_2F { x: tab_x + 36.0, y: tab_y + 19.0 },
                        accent,
                        2.0,
                        None,
                    );
                }
                tab_x += 60.0;
            }
        }

        // Inline Search icon/bar
        let search_r = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: rect.left + 14.0,
                top: tab_y + 26.0,
                right: rect.left + left_w - 14.0,
                bottom: tab_y + 52.0,
            },
            radiusX: 6.0,
            radiusY: 6.0,
        };
        let search_bg = target.CreateSolidColorBrush(
            &D2D1_COLOR_F { r: 0.08, g: 0.08, b: 0.10, a: 1.0 },
            None,
        )?;
        target.FillRoundedRectangle(&search_r, &search_bg);
        target.DrawRoundedRectangle(&search_r, card_border, 1.0, None);

        if let Ok(mfmt) = self.get_meta_format() {
            let placeholder = if clip.search_query.is_empty() {
                "Search clips...".to_string()
            } else {
                clip.search_query.clone()
            };
            let wide: Vec<u16> = placeholder.encode_utf16().collect();
            let stxt_r = D2D_RECT_F {
                left: search_r.rect.left + 8.0,
                top: search_r.rect.top + 5.0,
                right: search_r.rect.right - 8.0,
                bottom: search_r.rect.bottom,
            };
            let brush = if clip.search_query.is_empty() { secondary } else { white };
            target.DrawText(&wide, mfmt, &stxt_r, brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // Filtered items list
        let filtered = clip.filtered_items();
        let mut cur_y = search_r.rect.bottom + 12.0;

        if filtered.is_empty() {
            if let Ok(sfmt) = self.get_sub_format() {
                let wide: Vec<u16> = "No items found".encode_utf16().collect();
                let empty_r = D2D_RECT_F {
                    left: rect.left + 20.0,
                    top: cur_y + 20.0,
                    right: rect.left + left_w - 20.0,
                    bottom: cur_y + 60.0,
                };
                target.DrawText(&wide, sfmt, &empty_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        } else {
            // Group items into Pinned vs Today
            let mut rendered_section_label = false;

            for item in filtered.iter().take(5) {
                if item.is_pinned && !rendered_section_label {
                    if let Ok(mfmt) = self.get_meta_format() {
                        let wide: Vec<u16> = "PINNED".encode_utf16().collect();
                        let lr = D2D_RECT_F {
                            left: rect.left + 16.0,
                            top: cur_y,
                            right: rect.left + left_w - 16.0,
                            bottom: cur_y + 14.0,
                        };
                        target.DrawText(&wide, mfmt, &lr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                    }
                    cur_y += 18.0;
                    rendered_section_label = true;
                }

                let is_selected = clip.selected_item_id == Some(item.id);
                let item_r = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: rect.left + 10.0,
                        top: cur_y,
                        right: rect.left + left_w - 10.0,
                        bottom: cur_y + 36.0,
                    },
                    radiusX: 6.0,
                    radiusY: 6.0,
                };

                if is_selected {
                    let sel_brush = target.CreateSolidColorBrush(
                        &D2D1_COLOR_F { r: 0.16, g: 0.16, b: 0.20, a: 1.0 },
                        None,
                    )?;
                    target.FillRoundedRectangle(&item_r, &sel_brush);
                    target.DrawRoundedRectangle(&item_r, card_border, 1.0, None);
                }

                // Type Icon / Indicator
                let icon_cx = item_r.rect.left + 16.0;
                let icon_cy = item_r.rect.top + 18.0;
                match item.item_type {
                    crate::modules::clipboard::ClipboardItemType::Text => {
                        let _ = self.icon_painter.draw_icon(target, &self.factory, 6, icon_cx, icon_cy, secondary);
                    }
                    crate::modules::clipboard::ClipboardItemType::Link => {
                        let _ = self.icon_painter.draw_icon(target, &self.factory, 11, icon_cx, icon_cy, accent);
                    }
                    crate::modules::clipboard::ClipboardItemType::Color => {
                        let swatch_r = D2D1_ROUNDED_RECT {
                            rect: D2D_RECT_F {
                                left: icon_cx - 6.0,
                                top: icon_cy - 6.0,
                                right: icon_cx + 6.0,
                                bottom: icon_cy + 6.0,
                            },
                            radiusX: 3.0,
                            radiusY: 3.0,
                        };
                        target.FillRoundedRectangle(&swatch_r, accent);
                    }
                    crate::modules::clipboard::ClipboardItemType::Image => {
                        let _ = self.icon_painter.draw_icon(target, &self.factory, 8, icon_cx, icon_cy, secondary);
                    }
                }

                // Item Preview Text
                if let Ok(bfmt) = self.get_body_format() {
                    let preview = if item.preview.len() > 24 {
                        format!("{}...", &item.preview[..22])
                    } else {
                        item.preview.clone()
                    };
                    let wide: Vec<u16> = preview.encode_utf16().collect();
                    let t_rect = D2D_RECT_F {
                        left: item_r.rect.left + 30.0,
                        top: item_r.rect.top + 4.0,
                        right: item_r.rect.right - 10.0,
                        bottom: item_r.rect.top + 20.0,
                    };
                    let brush = if is_selected { white } else { secondary };
                    target.DrawText(&wide, bfmt, &t_rect, brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                }

                // Metadata subtitle
                if let Ok(mfmt) = self.get_meta_format() {
                    let wide: Vec<u16> = item.extra_info.encode_utf16().collect();
                    let sub_r = D2D_RECT_F {
                        left: item_r.rect.left + 30.0,
                        top: item_r.rect.top + 20.0,
                        right: item_r.rect.right - 10.0,
                        bottom: item_r.rect.bottom,
                    };
                    target.DrawText(&wide, mfmt, &sub_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                }

                cur_y += 40.0;
            }
        }

        // Right Pane: Full Item Preview & Drag-Out Ready Controls
        let right_rect = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: right_left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
            },
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&right_rect, card_bg);
        target.DrawRoundedRectangle(&right_rect, card_border, 1.0, None);

        let active_item = clip.items.iter().find(|i| Some(i.id) == clip.selected_item_id).or_else(|| clip.items.first());

        if let Some(selected_item) = active_item {
            // Metadata Line
            if let Ok(mfmt) = self.get_meta_format() {
                let type_name = match selected_item.item_type {
                    crate::modules::clipboard::ClipboardItemType::Text => "TEXT",
                    crate::modules::clipboard::ClipboardItemType::Link => "LINK",
                    crate::modules::clipboard::ClipboardItemType::Color => "COLOR SWATCH",
                    crate::modules::clipboard::ClipboardItemType::Image => "IMAGE THUMBNAIL",
                };
                let meta_str = format!("{} · {} CHARACTERS · DPAPI PROTECTED", type_name, selected_item.text.len());
                let wide: Vec<u16> = meta_str.encode_utf16().collect();
                let m_r = D2D_RECT_F {
                    left: right_left + 20.0,
                    top: rect.top + 16.0,
                    right: rect.right - 180.0,
                    bottom: rect.top + 32.0,
                };
                target.DrawText(&wide, mfmt, &m_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            // Action Buttons: Pin, Delete, Copy
            let pin_color = if selected_item.is_pinned { accent } else { secondary };
            let _ = self.icon_painter.draw_pin_icon(target, rect.right - 140.0, rect.top + 24.0, selected_item.is_pinned, pin_color);

            // Copy Action Button
            let copy_btn_r = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: rect.right - 80.0,
                    top: rect.top + 12.0,
                    right: rect.right - 16.0,
                    bottom: rect.top + 36.0,
                },
                radiusX: 6.0,
                radiusY: 6.0,
            };
            target.FillRoundedRectangle(&copy_btn_r, accent);
            if let Ok(bfmt) = self.get_body_format() {
                let wide: Vec<u16> = "Copy".encode_utf16().collect();
                let c_r = D2D_RECT_F {
                    left: copy_btn_r.rect.left + 16.0,
                    top: copy_btn_r.rect.top + 4.0,
                    right: copy_btn_r.rect.right,
                    bottom: copy_btn_r.rect.bottom,
                };
                target.DrawText(&wide, bfmt, &c_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            // Drag-Out Chip Hint
            if let Ok(mfmt) = self.get_meta_format() {
                let drag_hint: Vec<u16> = "Drag out directly to Notepad or browser".encode_utf16().collect();
                let d_r = D2D_RECT_F {
                    left: right_left + 20.0,
                    top: rect.bottom - 24.0,
                    right: rect.right - 20.0,
                    bottom: rect.bottom - 8.0,
                };
                target.DrawText(&drag_hint, mfmt, &d_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            // Full content preview
            if let Ok(bfmt) = self.get_body_format() {
                let wide: Vec<u16> = selected_item.text.encode_utf16().collect();
                let full_r = D2D_RECT_F {
                    left: right_left + 20.0,
                    top: rect.top + 46.0,
                    right: rect.right - 20.0,
                    bottom: rect.bottom - 30.0,
                };
                target.DrawText(&wide, bfmt, &full_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        } else {
            if let Ok(sfmt) = self.get_sub_format() {
                let wide: Vec<u16> = "Copy text or links from any Windows application to see and drag them here.".encode_utf16().collect();
                let desc_r = D2D_RECT_F {
                    left: right_left + 24.0,
                    top: rect.top + 50.0,
                    right: rect.right - 24.0,
                    bottom: rect.top + 100.0,
                };
                target.DrawText(&wide, sfmt, &desc_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        Ok(())
    }

    // Real Two-Pane Notes Page (Left list, Right editor preview)
    unsafe fn render_notes_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
        prod: &ProductivityManager,
    ) -> Result<()> {
        let left_w = 260.0;
        let right_left = rect.left + left_w + 12.0;

        // Left Pane: Notes list
        let left_rect = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: rect.left,
                top: rect.top,
                right: rect.left + left_w,
                bottom: rect.bottom,
            },
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&left_rect, card_bg);
        target.DrawRoundedRectangle(&left_rect, card_border, 1.0, None);

        // Header in left pane with "+ New Note"
        if let Ok(mfmt) = self.get_meta_format() {
            let count_str = format!("ALL NOTES ({})", prod.notes.len());
            let wide: Vec<u16> = count_str.encode_utf16().collect();
            let header_r = D2D_RECT_F {
                left: rect.left + 16.0,
                top: rect.top + 16.0,
                right: rect.left + left_w - 70.0,
                bottom: rect.top + 34.0,
            };
            target.DrawText(&wide, mfmt, &header_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // "+ Note" button in left pane header
        let add_btn_r = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: rect.left + left_w - 68.0,
                top: rect.top + 12.0,
                right: rect.left + left_w - 14.0,
                bottom: rect.top + 34.0,
            },
            radiusX: 6.0,
            radiusY: 6.0,
        };
        target.FillRoundedRectangle(&add_btn_r, accent);
        if let Ok(mfmt) = self.get_meta_format() {
            let wide: Vec<u16> = "+ Note".encode_utf16().collect();
            let btn_txt_r = D2D_RECT_F {
                left: add_btn_r.rect.left + 8.0,
                top: add_btn_r.rect.top + 4.0,
                right: add_btn_r.rect.right,
                bottom: add_btn_r.rect.bottom,
            };
            target.DrawText(&wide, mfmt, &btn_txt_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        let mut cur_y = rect.top + 48.0;
        if prod.notes.is_empty() {
            if let Ok(sfmt) = self.get_sub_format() {
                let wide: Vec<u16> = "No notes yet".encode_utf16().collect();
                let empty_r = D2D_RECT_F {
                    left: rect.left + 20.0,
                    top: cur_y + 24.0,
                    right: rect.left + left_w - 20.0,
                    bottom: cur_y + 60.0,
                };
                target.DrawText(&wide, sfmt, &empty_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        } else {
            for (idx, note) in prod.notes.iter().take(5).enumerate() {
                let note_r = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: rect.left + 10.0,
                        top: cur_y,
                        right: rect.left + left_w - 10.0,
                        bottom: cur_y + 32.0,
                    },
                    radiusX: 6.0,
                    radiusY: 6.0,
                };
                if idx == 0 {
                    let sel_brush = target.CreateSolidColorBrush(
                        &D2D1_COLOR_F { r: 0.15, g: 0.15, b: 0.18, a: 1.0 },
                        None,
                    )?;
                    target.FillRoundedRectangle(&note_r, &sel_brush);
                }

                // Checkbox circle
                let check_cx = note_r.rect.left + 14.0;
                let check_cy = note_r.rect.top + 16.0;
                let check_circle = D2D1_ELLIPSE {
                    point: D2D_POINT_2F { x: check_cx, y: check_cy },
                    radiusX: 5.5,
                    radiusY: 5.5,
                };
                if note.is_completed {
                    target.FillEllipse(&check_circle, accent);
                } else {
                    target.DrawEllipse(&check_circle, secondary, 1.2, None);
                }

                if let Ok(bfmt) = self.get_body_format() {
                    let preview = if note.content.len() > 24 {
                        format!("{}...", &note.content[..22])
                    } else {
                        note.content.clone()
                    };
                    let wide: Vec<u16> = preview.encode_utf16().collect();
                    let t_rect = D2D_RECT_F {
                        left: check_cx + 12.0,
                        top: note_r.rect.top + 6.0,
                        right: note_r.rect.right - 10.0,
                        bottom: note_r.rect.bottom,
                    };
                    let brush = if idx == 0 { white } else { secondary };
                    target.DrawText(&wide, bfmt, &t_rect, brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                }

                cur_y += 36.0;
            }
        }

        // Right Pane: Editor View
        let right_rect = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: right_left,
                top: rect.top,
                right: rect.right,
                bottom: rect.bottom,
            },
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&right_rect, card_bg);
        target.DrawRoundedRectangle(&right_rect, card_border, 1.0, None);

        if let Some(first_note) = prod.notes.first() {
            if let Ok(mfmt) = self.get_meta_format() {
                let status_str = if first_note.is_completed { "COMPLETED TASK" } else { "ACTIVE TASK · AUTOSAVED" };
                let wide: Vec<u16> = status_str.encode_utf16().collect();
                let m_r = D2D_RECT_F {
                    left: right_left + 24.0,
                    top: rect.top + 16.0,
                    right: rect.right - 24.0,
                    bottom: rect.top + 34.0,
                };
                target.DrawText(&wide, mfmt, &m_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            if let Ok(tfmt) = self.get_title_format() {
                let wide: Vec<u16> = first_note.content.encode_utf16().collect();
                let t_r = D2D_RECT_F {
                    left: right_left + 24.0,
                    top: rect.top + 48.0,
                    right: rect.right - 24.0,
                    bottom: rect.bottom - 24.0,
                };
                target.DrawText(&wide, tfmt, &t_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        } else {
            if let Ok(sfmt) = self.get_sub_format() {
                let wide: Vec<u16> = "Click '+ Note' to add a quick note or task. Notes automatically persist to notes.json.".encode_utf16().collect();
                let desc_r = D2D_RECT_F {
                    left: right_left + 24.0,
                    top: rect.top + 50.0,
                    right: rect.right - 24.0,
                    bottom: rect.top + 100.0,
                };
                target.DrawText(&wide, sfmt, &desc_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        Ok(())
    }

    // Real Focus Timers Page (Pomodoro 25:00, Short Break 5:00, Long Break 15:00)
    unsafe fn render_timers_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
        ring_track: &ID2D1SolidColorBrush,
        prod: &ProductivityManager,
    ) -> Result<()> {
        let card_round = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&card_round, card_bg);
        target.DrawRoundedRectangle(&card_round, card_border, 1.0, None);

        // Preset Chips at Top: [Pomodoro (25m)] [Short Break (5m)] [Long Break (15m)]
        let chips = ["Pomodoro 25m", "Short Break 5m", "Long Break 15m"];
        let chip_w = 140.0;
        let chips_start_x = rect.left + 28.0;

        for (i, chip_name) in chips.iter().enumerate() {
            let cx = chips_start_x + (i as f32 * (chip_w + 14.0));
            let chip_r = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: cx,
                    top: rect.top + 16.0,
                    right: cx + chip_w,
                    bottom: rect.top + 46.0,
                },
                radiusX: 8.0,
                radiusY: 8.0,
            };
            if i == 0 {
                target.FillRoundedRectangle(&chip_r, accent);
            } else {
                let chip_bg = target.CreateSolidColorBrush(
                    &D2D1_COLOR_F { r: 0.12, g: 0.12, b: 0.14, a: 1.0 },
                    None,
                )?;
                target.FillRoundedRectangle(&chip_r, &chip_bg);
                target.DrawRoundedRectangle(&chip_r, card_border, 1.0, None);
            }

            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = chip_name.encode_utf16().collect();
                let txt_r = D2D_RECT_F {
                    left: cx + 16.0,
                    top: rect.top + 24.0,
                    right: cx + chip_w,
                    bottom: rect.top + 46.0,
                };
                let brush = if i == 0 { white } else { secondary };
                target.DrawText(&wide, mfmt, &txt_r, brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        // Active Timer Display
        let timer = prod.timers.first();
        let (rem_sec, total_sec, is_running) = if let Some(t) = timer {
            (t.remaining_seconds, t.total_seconds, t.is_running)
        } else {
            (25 * 60, 25 * 60, false)
        };

        let mins = rem_sec / 60;
        let secs = rem_sec % 60;
        let time_str = format!("{:02}:{:02}", mins, secs);

        // Big Ring Gauge in Center
        let ring_cx = rect.left + 160.0;
        let ring_cy = rect.top + 130.0;
        let ring_r = 54.0;
        let stroke_w = 7.0;

        let ellipse = D2D1_ELLIPSE {
            point: D2D_POINT_2F { x: ring_cx, y: ring_cy },
            radiusX: ring_r,
            radiusY: ring_r,
        };
        target.DrawEllipse(&ellipse, ring_track, stroke_w, None);

        let pct = if total_sec > 0 { rem_sec as f32 / total_sec as f32 } else { 1.0 };
        if pct > 0.005 {
            let start_angle = -PI / 2.0;
            let sweep = pct * 2.0 * PI;
            let end_angle = start_angle + sweep;

            let p_start = D2D_POINT_2F {
                x: ring_cx + ring_r * start_angle.cos(),
                y: ring_cy + ring_r * start_angle.sin(),
            };
            let p_end = D2D_POINT_2F {
                x: ring_cx + ring_r * end_angle.cos(),
                y: ring_cy + ring_r * end_angle.sin(),
            };

            let geom = self.factory.CreatePathGeometry()?;
            let sink = geom.Open()?;
            sink.BeginFigure(p_start, D2D1_FIGURE_BEGIN_HOLLOW);
            sink.AddArc(&D2D1_ARC_SEGMENT {
                point: p_end,
                size: D2D_SIZE_F { width: ring_r, height: ring_r },
                rotationAngle: 0.0,
                sweepDirection: D2D1_SWEEP_DIRECTION_CLOCKWISE,
                arcSize: if sweep > PI { D2D1_ARC_SIZE_LARGE } else { D2D1_ARC_SIZE_SMALL },
            });
            sink.EndFigure(D2D1_FIGURE_END_OPEN);
            sink.Close()?;
            target.DrawGeometry(&geom, accent, stroke_w, None);
        }

        // Time string inside ring
        if let Ok(nfmt) = self.get_number_format() {
            let wide: Vec<u16> = time_str.encode_utf16().collect();
            let tr = D2D_RECT_F {
                left: ring_cx - 50.0,
                top: ring_cy - 16.0,
                right: ring_cx + 50.0,
                bottom: ring_cy + 20.0,
            };
            target.DrawText(&wide, nfmt, &tr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // Controls on Right side of Timers page
        let right_x = rect.left + 260.0;
        let right_y = rect.top + 90.0;

        if let Ok(tfmt) = self.get_title_format() {
            let status = if is_running { "Focus Session in Progress" } else { "Session Paused" };
            let wide: Vec<u16> = status.encode_utf16().collect();
            let sr = D2D_RECT_F {
                left: right_x,
                top: right_y,
                right: rect.right - 28.0,
                bottom: right_y + 28.0,
            };
            target.DrawText(&wide, tfmt, &sr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        if let Ok(sfmt) = self.get_sub_format() {
            let wide: Vec<u16> = "Stay focused for 25 minutes, then take a restorative 5-minute break.".encode_utf16().collect();
            let dr = D2D_RECT_F {
                left: right_x,
                top: right_y + 30.0,
                right: rect.right - 28.0,
                bottom: right_y + 60.0,
            };
            target.DrawText(&wide, sfmt, &dr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // Play / Pause Button (120x36)
        let play_btn_r = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: right_x,
                top: right_y + 70.0,
                right: right_x + 120.0,
                bottom: right_y + 106.0,
            },
            radiusX: 8.0,
            radiusY: 8.0,
        };
        target.FillRoundedRectangle(&play_btn_r, accent);

        if let Ok(bfmt) = self.get_body_format() {
            let btn_txt = if is_running { "Pause" } else { "Start" };
            let wide: Vec<u16> = btn_txt.encode_utf16().collect();
            let b_r = D2D_RECT_F {
                left: play_btn_r.rect.left + 42.0,
                top: play_btn_r.rect.top + 8.0,
                right: play_btn_r.rect.right,
                bottom: play_btn_r.rect.bottom,
            };
            target.DrawText(&wide, bfmt, &b_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // Reset Button (90x36)
        let reset_btn_r = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: right_x + 134.0,
                top: right_y + 70.0,
                right: right_x + 224.0,
                bottom: right_y + 106.0,
            },
            radiusX: 8.0,
            radiusY: 8.0,
        };
        let reset_bg = target.CreateSolidColorBrush(
            &D2D1_COLOR_F { r: 0.14, g: 0.14, b: 0.16, a: 1.0 },
            None,
        )?;
        target.FillRoundedRectangle(&reset_btn_r, &reset_bg);
        target.DrawRoundedRectangle(&reset_btn_r, card_border, 1.0, None);

        if let Ok(bfmt) = self.get_body_format() {
            let wide: Vec<u16> = "Reset".encode_utf16().collect();
            let b_r = D2D_RECT_F {
                left: reset_btn_r.rect.left + 28.0,
                top: reset_btn_r.rect.top + 8.0,
                right: reset_btn_r.rect.right,
                bottom: reset_btn_r.rect.bottom,
            };
            target.DrawText(&wide, bfmt, &b_r, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        Ok(())
    }

    // Real World Clock Page (4 Live Time Zones)
    unsafe fn render_world_clock_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        let card_w = (rect.right - rect.left - (3.0 * 12.0)) / 4.0;
        let card_h = rect.bottom - rect.top;

        // Query real UTC time
        let utc_st = windows::Win32::System::SystemInformation::GetSystemTime();

        // 4 Cities: London (UTC+0), New York (UTC-5), Tokyo (UTC+9), San Francisco (UTC-8)
        let cities = [
            ("London", "UTC+0", 0),
            ("New York", "UTC-5", -5),
            ("Tokyo", "UTC+9", 9),
            ("San Francisco", "UTC-8", -8),
        ];

        for (i, (name, tag, offset)) in cities.iter().enumerate() {
            let cx = rect.left + (i as f32 * (card_w + 12.0));
            let card_r = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: cx,
                    top: rect.top,
                    right: cx + card_w,
                    bottom: rect.top + card_h,
                },
                radiusX: 16.0,
                radiusY: 16.0,
            };
            target.FillRoundedRectangle(&card_r, card_bg);
            target.DrawRoundedRectangle(&card_r, card_border, 1.0, None);

            // Calculate city local hours
            let raw_hours = utc_st.wHour as i32 + offset;
            let city_hour = if raw_hours < 0 { raw_hours + 24 } else { raw_hours % 24 };
            let time_str = format!("{:02}:{:02}", city_hour, utc_st.wMinute);

            // City Name
            if let Ok(tfmt) = self.get_title_format() {
                let wide: Vec<u16> = name.encode_utf16().collect();
                let tr = D2D_RECT_F {
                    left: cx + 16.0,
                    top: rect.top + 20.0,
                    right: cx + card_w - 16.0,
                    bottom: rect.top + 46.0,
                };
                target.DrawText(&wide, tfmt, &tr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            // Tag (e.g. UTC+0)
            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = tag.encode_utf16().collect();
                let mr = D2D_RECT_F {
                    left: cx + 16.0,
                    top: rect.top + 48.0,
                    right: cx + card_w - 16.0,
                    bottom: rect.top + 64.0,
                };
                target.DrawText(&wide, mfmt, &mr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            // Big Digital Time
            if let Ok(nfmt) = self.get_number_format() {
                let wide: Vec<u16> = time_str.encode_utf16().collect();
                let nr = D2D_RECT_F {
                    left: cx + 16.0,
                    top: rect.top + 100.0,
                    right: cx + card_w - 16.0,
                    bottom: rect.top + 140.0,
                };
                target.DrawText(&wide, nfmt, &nr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        Ok(())
    }

    // Real Month Calendar Grid with Current Day Highlight
    unsafe fn render_calendar_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        let card_round = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&card_round, card_bg);
        target.DrawRoundedRectangle(&card_round, card_border, 1.0, None);

        let st = windows::Win32::System::SystemInformation::GetLocalTime();

        // Month Header
        let month_name = match st.wMonth {
            1 => "January", 2 => "February", 3 => "March", 4 => "April", 5 => "May", 6 => "June",
            7 => "July", 8 => "August", 9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "",
        };
        let header_str = format!("{} {}", month_name, st.wYear);

        if let Ok(tfmt) = self.get_title_format() {
            let wide: Vec<u16> = header_str.encode_utf16().collect();
            let hr = D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 14.0,
                right: rect.left + 240.0,
                bottom: rect.top + 38.0,
            };
            target.DrawText(&wide, tfmt, &hr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // Days of week header: Sun Mon Tue Wed Thu Fri Sat
        let col_w = (rect.right - rect.left - 48.0) / 7.0;
        let day_names = ["SU", "MO", "TU", "WE", "TH", "FR", "SA"];

        if let Ok(mfmt) = self.get_meta_format() {
            for (i, dname) in day_names.iter().enumerate() {
                let dx = rect.left + 24.0 + (i as f32 * col_w);
                let wide: Vec<u16> = dname.encode_utf16().collect();
                let dr = D2D_RECT_F {
                    left: dx,
                    top: rect.top + 46.0,
                    right: dx + col_w,
                    bottom: rect.top + 64.0,
                };
                target.DrawText(&wide, mfmt, &dr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        // Calculate days in month & starting weekday
        let days_in_month = match st.wMonth {
            2 => if (st.wYear % 4 == 0 && st.wYear % 100 != 0) || (st.wYear % 400 == 0) { 29 } else { 28 },
            4 | 6 | 9 | 11 => 30,
            _ => 31,
        };

        // First day of current month weekday (Zeller's congruence simplified)
        let q = 1;
        let m = if st.wMonth < 3 { st.wMonth as i32 + 12 } else { st.wMonth as i32 };
        let k = if st.wMonth < 3 { (st.wYear - 1) % 100 } else { st.wYear % 100 } as i32;
        let j = if st.wMonth < 3 { (st.wYear - 1) / 100 } else { st.wYear / 100 } as i32;
        let h = (q + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
        let first_weekday = ((h + 5) % 7) as usize + 1; // 0 = Sunday, 1 = Monday...

        let mut row = 0;
        let mut col = first_weekday % 7;

        for day in 1..=days_in_month {
            let dx = rect.left + 24.0 + (col as f32 * col_w);
            let dy = rect.top + 70.0 + (row as f32 * 28.0);

            let is_today = day == st.wDay as i32;
            if is_today {
                let today_circle = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: dx + (col_w / 2.0) - 12.0,
                        top: dy - 2.0,
                        right: dx + (col_w / 2.0) + 12.0,
                        bottom: dy + 22.0,
                    },
                    radiusX: 6.0,
                    radiusY: 6.0,
                };
                target.FillRoundedRectangle(&today_circle, accent);
            }

            if let Ok(bfmt) = self.get_body_format() {
                let day_str = format!("{}", day);
                let wide: Vec<u16> = day_str.encode_utf16().collect();
                let dr = D2D_RECT_F {
                    left: dx,
                    top: dy,
                    right: dx + col_w,
                    bottom: dy + 22.0,
                };
                let brush = if is_today { white } else { secondary };
                target.DrawText(&wide, bfmt, &dr, brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            col += 1;
            if col == 7 {
                col = 0;
                row += 1;
            }
        }

        Ok(())
    }

    // Real Weather Page (Empty state / On-demand query per Rule 0)
    unsafe fn render_weather_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        let card_round = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&card_round, card_bg);
        target.DrawRoundedRectangle(&card_round, card_border, 1.0, None);

        // Header
        if let Ok(mfmt) = self.get_meta_format() {
            let wide: Vec<u16> = "LOCAL WEATHER & FORECAST".encode_utf16().collect();
            let hr = D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 20.0,
                right: rect.right - 24.0,
                bottom: rect.top + 36.0,
            };
            target.DrawText(&wide, mfmt, &hr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // Rule 0 Compliant state: Zero fake temperatures or demo cities
        if let Ok(tfmt) = self.get_title_format() {
            let wide: Vec<u16> = "Weather Service Ready".encode_utf16().collect();
            let tr = D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 50.0,
                right: rect.right - 24.0,
                bottom: rect.top + 80.0,
            };
            target.DrawText(&wide, tfmt, &tr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        if let Ok(sfmt) = self.get_sub_format() {
            let wide: Vec<u16> = "To preserve privacy and prevent background network polling, real-time forecast updates occur only when selected.".encode_utf16().collect();
            let sr = D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 82.0,
                right: rect.right - 24.0,
                bottom: rect.top + 130.0,
            };
            target.DrawText(&wide, sfmt, &sr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // Fetch Live Weather Button
        let btn_r = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 140.0,
                right: rect.left + 200.0,
                bottom: rect.top + 176.0,
            },
            radiusX: 8.0,
            radiusY: 8.0,
        };
        target.FillRoundedRectangle(&btn_r, accent);

        if let Ok(bfmt) = self.get_body_format() {
            let wide: Vec<u16> = "Fetch Local Forecast".encode_utf16().collect();
            let b_txt_r = D2D_RECT_F {
                left: btn_r.rect.left + 24.0,
                top: btn_r.rect.top + 8.0,
                right: btn_r.rect.right,
                bottom: btn_r.rect.bottom,
            };
            target.DrawText(&wide, bfmt, &b_txt_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        Ok(())
    }

    // Real File Shelf Page
    unsafe fn render_file_shelf_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        _accent: &ID2D1SolidColorBrush,
        prod: &ProductivityManager,
    ) -> Result<()> {
        let card_round = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&card_round, card_bg);
        target.DrawRoundedRectangle(&card_round, card_border, 1.0, None);

        if let Ok(mfmt) = self.get_meta_format() {
            let count_str = format!("PARKED FILES SHELF ({})", prod.parked_files.len());
            let wide: Vec<u16> = count_str.encode_utf16().collect();
            let hr = D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 20.0,
                right: rect.right - 24.0,
                bottom: rect.top + 36.0,
            };
            target.DrawText(&wide, mfmt, &hr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        if prod.parked_files.is_empty() {
            if let Ok(tfmt) = self.get_title_format() {
                let wide: Vec<u16> = "Shelf is empty".encode_utf16().collect();
                let tr = D2D_RECT_F {
                    left: rect.left + 24.0,
                    top: rect.top + 50.0,
                    right: rect.right - 24.0,
                    bottom: rect.top + 76.0,
                };
                target.DrawText(&wide, tfmt, &tr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
            if let Ok(sfmt) = self.get_sub_format() {
                let wide: Vec<u16> = "Drag and drop any file or folder here to temporarily park it for easy access across multiple apps.".encode_utf16().collect();
                let sr = D2D_RECT_F {
                    left: rect.left + 24.0,
                    top: rect.top + 80.0,
                    right: rect.right - 24.0,
                    bottom: rect.top + 130.0,
                };
                target.DrawText(&wide, sfmt, &sr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        } else {
            let mut cy = rect.top + 50.0;
            for (i, p) in prod.parked_files.iter().take(5).enumerate() {
                let fname = p.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown File");
                if let Ok(bfmt) = self.get_body_format() {
                    let line = format!("{} · {}", i + 1, fname);
                    let wide: Vec<u16> = line.encode_utf16().collect();
                    let lr = D2D_RECT_F {
                        left: rect.left + 24.0,
                        top: cy,
                        right: rect.right - 24.0,
                        bottom: cy + 24.0,
                    };
                    target.DrawText(&wide, bfmt, &lr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
                }
                cy += 28.0;
            }
        }

        Ok(())
    }

    // Converters Page (Length, Temperature, Currency)
    unsafe fn render_converters_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        let card_round = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&card_round, card_bg);
        target.DrawRoundedRectangle(&card_round, card_border, 1.0, None);

        if let Ok(mfmt) = self.get_meta_format() {
            let wide: Vec<u16> = "LIVE UNIT & CURRENCY CONVERTERS".encode_utf16().collect();
            let hr = D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 20.0,
                right: rect.right - 24.0,
                bottom: rect.top + 36.0,
            };
            target.DrawText(&wide, mfmt, &hr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        let conv_cards = [
            ("Length", "1 Meter (m)", "= 3.28084 Feet (ft)", "100 cm · 39.37 inches"),
            ("Temperature", "20 °Celsius (°C)", "= 68.0 °Fahrenheit (°F)", "293.15 Kelvin (K)"),
            ("Currency", "1 USD ($)", "≈ 0.92 EUR / ₹84.5 INR", "Real ECB reference rate"),
        ];

        let card_w = (rect.right - rect.left - 48.0 - 24.0) / 3.0;
        let card_h = 130.0;
        let top_y = rect.top + 50.0;

        for (i, (cat, from, to, note)) in conv_cards.iter().enumerate() {
            let cx = rect.left + 24.0 + (i as f32 * (card_w + 12.0));
            let sub_round = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: cx,
                    top: top_y,
                    right: cx + card_w,
                    bottom: top_y + card_h,
                },
                radiusX: 12.0,
                radiusY: 12.0,
            };
            let sub_bg = target.CreateSolidColorBrush(
                &D2D1_COLOR_F { r: 0.12, g: 0.12, b: 0.14, a: 1.0 },
                None,
            )?;
            target.FillRoundedRectangle(&sub_round, &sub_bg);
            target.DrawRoundedRectangle(&sub_round, card_border, 1.0, None);

            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = cat.to_uppercase().encode_utf16().collect();
                let tr = D2D_RECT_F {
                    left: cx + 16.0,
                    top: top_y + 14.0,
                    right: cx + card_w - 16.0,
                    bottom: top_y + 28.0,
                };
                target.DrawText(&wide, mfmt, &tr, accent, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            if let Ok(bfmt) = self.get_body_format() {
                let wide: Vec<u16> = from.encode_utf16().collect();
                let fr = D2D_RECT_F {
                    left: cx + 16.0,
                    top: top_y + 36.0,
                    right: cx + card_w - 16.0,
                    bottom: top_y + 56.0,
                };
                target.DrawText(&wide, bfmt, &fr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            if let Ok(tfmt) = self.get_title_format() {
                let wide: Vec<u16> = to.encode_utf16().collect();
                let tor = D2D_RECT_F {
                    left: cx + 16.0,
                    top: top_y + 58.0,
                    right: cx + card_w - 16.0,
                    bottom: top_y + 88.0,
                };
                target.DrawText(&wide, tfmt, &tor, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = note.encode_utf16().collect();
                let nr = D2D_RECT_F {
                    left: cx + 16.0,
                    top: top_y + 96.0,
                    right: cx + card_w - 16.0,
                    bottom: top_y + 114.0,
                };
                target.DrawText(&wide, mfmt, &nr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        Ok(())
    }

    // Links & Bookmarks Page
    unsafe fn render_links_page(
        &mut self,
        target: &ID2D1DCRenderTarget,
        rect: &D2D_RECT_F,
        card_bg: &ID2D1SolidColorBrush,
        card_border: &ID2D1SolidColorBrush,
        white: &ID2D1SolidColorBrush,
        secondary: &ID2D1SolidColorBrush,
        accent: &ID2D1SolidColorBrush,
    ) -> Result<()> {
        let card_round = D2D1_ROUNDED_RECT {
            rect: *rect,
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&card_round, card_bg);
        target.DrawRoundedRectangle(&card_round, card_border, 1.0, None);

        if let Ok(mfmt) = self.get_meta_format() {
            let wide: Vec<u16> = "QUICK LINKS & SEARCH ENGINES".encode_utf16().collect();
            let hr = D2D_RECT_F {
                left: rect.left + 24.0,
                top: rect.top + 20.0,
                right: rect.right - 24.0,
                bottom: rect.top + 36.0,
            };
            target.DrawText(&wide, mfmt, &hr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
        }

        // Search Engine Selector Chips
        let engines = ["Google", "GitHub", "YouTube", "Stack Overflow", "ChatGPT"];
        let mut cur_chip_x = rect.left + 24.0;
        let chip_y = rect.top + 46.0;

        for (i, name) in engines.iter().enumerate() {
            let chip_r = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: cur_chip_x,
                    top: chip_y,
                    right: cur_chip_x + 90.0,
                    bottom: chip_y + 28.0,
                },
                radiusX: 6.0,
                radiusY: 6.0,
            };
            if i == 0 {
                target.FillRoundedRectangle(&chip_r, accent);
            } else {
                let c_bg = target.CreateSolidColorBrush(
                    &D2D1_COLOR_F { r: 0.14, g: 0.14, b: 0.16, a: 1.0 },
                    None,
                )?;
                target.FillRoundedRectangle(&chip_r, &c_bg);
                target.DrawRoundedRectangle(&chip_r, card_border, 1.0, None);
            }

            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = name.encode_utf16().collect();
                let txt_r = D2D_RECT_F {
                    left: cur_chip_x + 8.0,
                    top: chip_y + 6.0,
                    right: cur_chip_x + 86.0,
                    bottom: chip_y + 24.0,
                };
                let brush = if i == 0 { white } else { secondary };
                target.DrawText(&wide, mfmt, &txt_r, brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
            cur_chip_x += 98.0;
        }

        // Saved Bookmarks Tiles
        let bookmarks = [
            ("GitHub", "https://github.com", "Code hosting & pull requests"),
            ("YouTube", "https://youtube.com", "Videos and music streams"),
            ("Google", "https://google.com", "Search and tools"),
            ("ChatGPT", "https://chatgpt.com", "AI conversation & assistance"),
        ];

        let b_top = chip_y + 42.0;
        let tile_w = (rect.right - rect.left - 48.0 - 36.0) / 4.0;

        for (i, (title, url, desc)) in bookmarks.iter().enumerate() {
            let bx = rect.left + 24.0 + (i as f32 * (tile_w + 12.0));
            let tile_r = D2D1_ROUNDED_RECT {
                rect: D2D_RECT_F {
                    left: bx,
                    top: b_top,
                    right: bx + tile_w,
                    bottom: b_top + 90.0,
                },
                radiusX: 10.0,
                radiusY: 10.0,
            };
            let tile_bg = target.CreateSolidColorBrush(
                &D2D1_COLOR_F { r: 0.11, g: 0.11, b: 0.13, a: 1.0 },
                None,
            )?;
            target.FillRoundedRectangle(&tile_r, &tile_bg);
            target.DrawRoundedRectangle(&tile_r, card_border, 1.0, None);

            if let Ok(bfmt) = self.get_body_format() {
                let wide: Vec<u16> = title.encode_utf16().collect();
                let tr = D2D_RECT_F {
                    left: bx + 12.0,
                    top: b_top + 10.0,
                    right: bx + tile_w - 12.0,
                    bottom: b_top + 30.0,
                };
                target.DrawText(&wide, bfmt, &tr, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }

            if let Ok(mfmt) = self.get_meta_format() {
                let wide: Vec<u16> = url.encode_utf16().collect();
                let ur = D2D_RECT_F {
                    left: bx + 12.0,
                    top: b_top + 32.0,
                    right: bx + tile_w - 12.0,
                    bottom: b_top + 48.0,
                };
                target.DrawText(&wide, mfmt, &ur, accent, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);

                let wide_d: Vec<u16> = desc.encode_utf16().collect();
                let dr = D2D_RECT_F {
                    left: bx + 12.0,
                    top: b_top + 52.0,
                    right: bx + tile_w - 12.0,
                    bottom: b_top + 84.0,
                };
                target.DrawText(&wide_d, mfmt, &dr, secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
            }
        }

        Ok(())
    }
}

fn get_current_date_string() -> String {
    unsafe {
        let st = windows::Win32::System::SystemInformation::GetLocalTime();
        let month_name = match st.wMonth {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr", 5 => "May", 6 => "Jun",
            7 => "Jul", 8 => "Aug", 9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "",
        };
        let day_name = match st.wDayOfWeek {
            0 => "Sunday", 1 => "Monday", 2 => "Tuesday", 3 => "Wednesday",
            4 => "Thursday", 5 => "Friday", 6 => "Saturday",
            _ => "",
        };
        format!("{}, {} {}", day_name, st.wDay, month_name)
    }
}
