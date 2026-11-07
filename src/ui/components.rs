use super::{Rect, UiContext};
use windows::core::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;

pub fn draw_card(
    target: &ID2D1DCRenderTarget,
    rect: &Rect,
    card_bg: &ID2D1SolidColorBrush,
    card_border: &ID2D1SolidColorBrush,
) -> Result<()> {
    unsafe {
        let round = D2D1_ROUNDED_RECT {
            rect: rect.to_d2d_rect(),
            radiusX: 16.0,
            radiusY: 16.0,
        };
        target.FillRoundedRectangle(&round, card_bg);
        target.DrawRoundedRectangle(&round, card_border, 1.0, None);
    }
    Ok(())
}

pub fn draw_button(
    target: &ID2D1DCRenderTarget,
    id: u64,
    rect: &Rect,
    label: &str,
    ui: &mut UiContext,
    fmt: &IDWriteTextFormat,
    white: &ID2D1SolidColorBrush,
    border: &ID2D1SolidColorBrush,
) -> Result<bool> {
    let hovered = rect.contains(ui.mouse_x, ui.mouse_y);
    let mut clicked = false;

    if hovered {
        ui.hot_id = Some(id);
        if ui.mouse_clicked {
            clicked = true;
        }
    }

    let bg_color = if hovered && ui.mouse_down {
        D2D1_COLOR_F { r: 0.22, g: 0.22, b: 0.24, a: 1.0 }
    } else if hovered {
        D2D1_COLOR_F { r: 0.16, g: 0.16, b: 0.18, a: 1.0 }
    } else {
        D2D1_COLOR_F { r: 0.10, g: 0.10, b: 0.12, a: 1.0 }
    };

    unsafe {
        let bg_brush = target.CreateSolidColorBrush(&bg_color, None)?;
        let round = D2D1_ROUNDED_RECT {
            rect: rect.to_d2d_rect(),
            radiusX: 8.0,
            radiusY: 8.0,
        };
        target.FillRoundedRectangle(&round, &bg_brush);
        target.DrawRoundedRectangle(&round, border, 1.0, None);

        let wide: Vec<u16> = label.encode_utf16().collect();
        target.DrawText(&wide, fmt, &rect.to_d2d_rect(), white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }

    Ok(clicked)
}

pub fn draw_toggle_switch(
    target: &ID2D1DCRenderTarget,
    _id: u64,
    rect: &Rect,
    val: &mut bool,
    ui: &mut UiContext,
    accent: &ID2D1SolidColorBrush,
    white: &ID2D1SolidColorBrush,
    border: &ID2D1SolidColorBrush,
) -> Result<bool> {
    let hovered = rect.contains(ui.mouse_x, ui.mouse_y);
    let mut toggled = false;

    if hovered && ui.mouse_clicked {
        *val = !*val;
        toggled = true;
        ui.dirty = true;
    }

    unsafe {
        let round = D2D1_ROUNDED_RECT {
            rect: rect.to_d2d_rect(),
            radiusX: rect.h / 2.0,
            radiusY: rect.h / 2.0,
        };

        if *val {
            target.FillRoundedRectangle(&round, accent);
        } else {
            let off_brush = target.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.18, g: 0.18, b: 0.20, a: 1.0 }, None)?;
            target.FillRoundedRectangle(&round, &off_brush);
            target.DrawRoundedRectangle(&round, border, 1.0, None);
        }

        // Knob
        let knob_r = (rect.h - 6.0) / 2.0;
        let knob_x = if *val {
            rect.x + rect.w - knob_r - 3.0
        } else {
            rect.x + knob_r + 3.0
        };
        let knob_center = D2D_POINT_2F { x: knob_x, y: rect.y + (rect.h / 2.0) };
        target.FillEllipse(&D2D1_ELLIPSE { point: knob_center, radiusX: knob_r, radiusY: knob_r }, white);
    }

    Ok(toggled)
}

pub fn draw_slider(
    target: &ID2D1DCRenderTarget,
    _id: u64,
    rect: &Rect,
    val: &mut f32, // 0.0 - 100.0
    ui: &mut UiContext,
    accent: &ID2D1SolidColorBrush,
    white: &ID2D1SolidColorBrush,
) -> Result<bool> {
    let hovered = rect.contains(ui.mouse_x, ui.mouse_y);
    let mut changed = false;

    if hovered && ui.mouse_down {
        let pct = ((ui.mouse_x - rect.x) / rect.w).clamp(0.0, 1.0);
        *val = pct * 100.0;
        changed = true;
        ui.dirty = true;
    }

    unsafe {
        let track_y = rect.y + (rect.h / 2.0);
        let track_h = 4.0;
        let track_round = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: rect.x,
                top: track_y - (track_h / 2.0),
                right: rect.x + rect.w,
                bottom: track_y + (track_h / 2.0),
            },
            radiusX: 2.0,
            radiusY: 2.0,
        };
        let track_brush = target.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.16, g: 0.16, b: 0.18, a: 1.0 }, None)?;
        target.FillRoundedRectangle(&track_round, &track_brush);

        // Filled active track
        let active_w = rect.w * (*val / 100.0);
        let active_round = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: rect.x,
                top: track_y - (track_h / 2.0),
                right: rect.x + active_w,
                bottom: track_y + (track_h / 2.0),
            },
            radiusX: 2.0,
            radiusY: 2.0,
        };
        target.FillRoundedRectangle(&active_round, accent);

        // Knob
        let knob_center = D2D_POINT_2F { x: rect.x + active_w, y: track_y };
        target.FillEllipse(&D2D1_ELLIPSE { point: knob_center, radiusX: 7.0, radiusY: 7.0 }, white);
    }

    Ok(changed)
}

pub fn draw_segmented_control(
    target: &ID2D1DCRenderTarget,
    _id: u64,
    rect: &Rect,
    items: &[&str],
    selected: &mut usize,
    ui: &mut UiContext,
    fmt: &IDWriteTextFormat,
    white: &ID2D1SolidColorBrush,
    secondary: &ID2D1SolidColorBrush,
    border: &ID2D1SolidColorBrush,
) -> Result<bool> {
    let mut changed = false;
    let seg_w = rect.w / items.len() as f32;

    unsafe {
        let bg_brush = target.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.08, g: 0.08, b: 0.10, a: 1.0 }, None)?;
        let container_round = D2D1_ROUNDED_RECT {
            rect: rect.to_d2d_rect(),
            radiusX: 8.0,
            radiusY: 8.0,
        };
        target.FillRoundedRectangle(&container_round, &bg_brush);
        target.DrawRoundedRectangle(&container_round, border, 1.0, None);

        let active_brush = target.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.20, g: 0.20, b: 0.24, a: 1.0 }, None)?;

        for (i, label) in items.iter().enumerate() {
            let item_x = rect.x + (i as f32 * seg_w);
            let item_rect = Rect::new(item_x, rect.y, seg_w, rect.h);

            if item_rect.contains(ui.mouse_x, ui.mouse_y) && ui.mouse_clicked {
                *selected = i;
                changed = true;
                ui.dirty = true;
            }

            if i == *selected {
                let seg_round = D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F {
                        left: item_x + 2.0,
                        top: rect.y + 2.0,
                        right: item_x + seg_w - 2.0,
                        bottom: rect.y + rect.h - 2.0,
                    },
                    radiusX: 6.0,
                    radiusY: 6.0,
                };
                target.FillRoundedRectangle(&seg_round, &active_brush);
            }

            let wide: Vec<u16> = label.encode_utf16().collect();
            target.DrawText(
                &wide,
                fmt,
                &item_rect.to_d2d_rect(),
                if i == *selected { white } else { secondary },
                D2D1_DRAW_TEXT_OPTIONS_NONE,
                DWRITE_MEASURING_MODE_NATURAL,
            );
        }
    }

    Ok(changed)
}

pub fn draw_text_input(
    target: &ID2D1DCRenderTarget,
    _id: u64,
    rect: &Rect,
    text: &mut String,
    _caret: &mut usize,
    focused: bool,
    fmt: &IDWriteTextFormat,
    white: &ID2D1SolidColorBrush,
    _secondary: &ID2D1SolidColorBrush,
    border: &ID2D1SolidColorBrush,
) -> Result<()> {
    unsafe {
        let bg_color = if focused {
            D2D1_COLOR_F { r: 0.12, g: 0.12, b: 0.14, a: 1.0 }
        } else {
            D2D1_COLOR_F { r: 0.08, g: 0.08, b: 0.10, a: 1.0 }
        };
        let bg_brush = target.CreateSolidColorBrush(&bg_color, None)?;
        let round = D2D1_ROUNDED_RECT {
            rect: rect.to_d2d_rect(),
            radiusX: 8.0,
            radiusY: 8.0,
        };
        target.FillRoundedRectangle(&round, &bg_brush);
        target.DrawRoundedRectangle(&round, border, 1.0, None);

        let wide: Vec<u16> = text.encode_utf16().collect();
        let draw_r = D2D_RECT_F {
            left: rect.x + 10.0,
            top: rect.y + 4.0,
            right: rect.x + rect.w - 10.0,
            bottom: rect.y + rect.h - 4.0,
        };
        target.DrawText(&wide, fmt, &draw_r, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);

        if focused {
            let caret_x = rect.x + 10.0 + (text.len() as f32 * 7.0).min(rect.w - 15.0);
            target.DrawLine(
                D2D_POINT_2F { x: caret_x, y: rect.y + 6.0 },
                D2D_POINT_2F { x: caret_x, y: rect.y + rect.h - 6.0 },
                white,
                1.5,
                None,
            );
        }
    }
    Ok(())
}
