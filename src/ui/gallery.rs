use super::components::*;
use super::{ComponentGalleryState, Rect, UiContext};
use windows::core::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectWrite::*;

pub fn render_component_gallery(
    target: &ID2D1DCRenderTarget,
    bounds: &Rect,
    state: &mut ComponentGalleryState,
    ui: &mut UiContext,
    card_bg: &ID2D1SolidColorBrush,
    card_border: &ID2D1SolidColorBrush,
    white: &ID2D1SolidColorBrush,
    secondary: &ID2D1SolidColorBrush,
    accent: &ID2D1SolidColorBrush,
    body_fmt: &IDWriteTextFormat,
    title_fmt: &IDWriteTextFormat,
    meta_fmt: &IDWriteTextFormat,
) -> Result<()> {
    // 2-column gallery of all UI toolkit components
    let col_w = (bounds.w - 14.0) / 2.0;

    // Card Left: Buttons, Toggles, Sliders, Segmented Controls
    let left_rect = Rect::new(bounds.x, bounds.y, col_w, bounds.h);
    draw_card(target, &left_rect, card_bg, card_border)?;

    // Title Left
    unsafe {
        let wide: Vec<u16> = "Interactive Controls".encode_utf16().collect();
        let r = Rect::new(left_rect.x + 16.0, left_rect.y + 14.0, left_rect.w - 32.0, 24.0);
        target.DrawText(&wide, title_fmt, &r.to_d2d_rect(), white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }

    // 1. Button component
    let btn_label = format!("Button (Clicked {}x)", state.button_click_count);
    let btn_rect = Rect::new(left_rect.x + 16.0, left_rect.y + 46.0, 160.0, 32.0);
    if draw_button(target, 101, &btn_rect, &btn_label, ui, body_fmt, white, card_border)? {
        state.button_click_count += 1;
    }

    // 2. Toggle Switch component
    let toggle_label_r = Rect::new(left_rect.x + 16.0, left_rect.y + 92.0, 100.0, 24.0);
    unsafe {
        let wide: Vec<u16> = "Toggle Switch".encode_utf16().collect();
        target.DrawText(&wide, body_fmt, &toggle_label_r.to_d2d_rect(), secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }
    let toggle_rect = Rect::new(left_rect.x + 130.0, left_rect.y + 92.0, 44.0, 24.0);
    draw_toggle_switch(target, 102, &toggle_rect, &mut state.toggle_val, ui, accent, white, card_border)?;

    // 3. Slider component
    let slider_label_r = Rect::new(left_rect.x + 16.0, left_rect.y + 130.0, 200.0, 20.0);
    unsafe {
        let text = format!("Slider: {:.0}%", state.slider_val);
        let wide: Vec<u16> = text.encode_utf16().collect();
        target.DrawText(&wide, body_fmt, &slider_label_r.to_d2d_rect(), secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }
    let slider_rect = Rect::new(left_rect.x + 16.0, left_rect.y + 154.0, left_rect.w - 32.0, 20.0);
    draw_slider(target, 103, &slider_rect, &mut state.slider_val, ui, accent, white)?;

    // 4. Segmented Control component
    let seg_rect = Rect::new(left_rect.x + 16.0, left_rect.y + 186.0, left_rect.w - 32.0, 32.0);
    draw_segmented_control(
        target,
        104,
        &seg_rect,
        &["Small", "Default", "Large"],
        &mut state.selected_segment,
        ui,
        body_fmt,
        white,
        secondary,
        card_border,
    )?;

    // Card Right: Inputs, Ring Gauges, Cards & Indicators
    let right_rect = Rect::new(bounds.x + col_w + 14.0, bounds.y, col_w, bounds.h);
    draw_card(target, &right_rect, card_bg, card_border)?;

    // Title Right
    unsafe {
        let wide: Vec<u16> = "Inputs & Data Displays".encode_utf16().collect();
        let r = Rect::new(right_rect.x + 16.0, right_rect.y + 14.0, right_rect.w - 32.0, 24.0);
        target.DrawText(&wide, title_fmt, &r.to_d2d_rect(), white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }

    // 5. TextInput component
    let input_label_r = Rect::new(right_rect.x + 16.0, right_rect.y + 46.0, 200.0, 20.0);
    unsafe {
        let wide: Vec<u16> = "Single-Line Text Input".encode_utf16().collect();
        target.DrawText(&wide, meta_fmt, &input_label_r.to_d2d_rect(), secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }
    let input_rect = Rect::new(right_rect.x + 16.0, right_rect.y + 70.0, right_rect.w - 32.0, 32.0);
    draw_text_input(
        target,
        105,
        &input_rect,
        &mut state.text_input,
        &mut state.caret_pos,
        true, // Focused in gallery
        body_fmt,
        white,
        secondary,
        card_border,
    )?;

    // 6. Ring Gauge preview
    let gauge_center = D2D_POINT_2F { x: right_rect.x + 60.0, y: right_rect.y + 155.0 };
    unsafe {
        let ring_r = 24.0;
        let track_brush = target.CreateSolidColorBrush(&D2D1_COLOR_F { r: 0.16, g: 0.16, b: 0.18, a: 0.8 }, None)?;
        target.DrawEllipse(
            &D2D1_ELLIPSE { point: gauge_center, radiusX: ring_r, radiusY: ring_r },
            &track_brush,
            4.0,
            None,
        );
        let arc_r = D2D1_ROUNDED_RECT {
            rect: D2D_RECT_F {
                left: gauge_center.x - ring_r,
                top: gauge_center.y - ring_r,
                right: gauge_center.x + ring_r,
                bottom: gauge_center.y + ring_r,
            },
            radiusX: 4.0,
            radiusY: 4.0,
        };
        let pct_str = format!("{:.0}%", state.slider_val);
        let wide_p: Vec<u16> = pct_str.encode_utf16().collect();
        target.DrawText(&wide_p, body_fmt, &arc_r.rect, white, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);

        let note = "Linked directly to Slider above";
        let wide_n: Vec<u16> = note.encode_utf16().collect();
        let note_r = Rect::new(right_rect.x + 100.0, right_rect.y + 145.0, right_rect.w - 110.0, 24.0);
        target.DrawText(&wide_n, body_fmt, &note_r.to_d2d_rect(), secondary, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL);
    }

    Ok(())
}

