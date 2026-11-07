pub mod components;
pub mod gallery;
pub mod icons;
pub mod settings;
pub mod theme;

use windows::Win32::Graphics::Direct2D::Common::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn to_d2d_rect(&self) -> D2D_RECT_F {
        D2D_RECT_F {
            left: self.x,
            top: self.y,
            right: self.x + self.w,
            bottom: self.y + self.h,
        }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= (self.x + self.w) && py >= self.y && py <= (self.y + self.h)
    }
}

pub struct UiContext {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_down: bool,
    pub mouse_clicked: bool,
    pub hot_id: Option<u64>,
    pub active_id: Option<u64>,
    pub focused_id: Option<u64>,
    pub dirty: bool,
}

impl UiContext {
    pub fn new() -> Self {
        Self {
            mouse_x: 0.0,
            mouse_y: 0.0,
            mouse_down: false,
            mouse_clicked: false,
            hot_id: None,
            active_id: None,
            focused_id: None,
            dirty: true,
        }
    }
}

// Retained UI Component Gallery & Toolkit state
pub struct ComponentGalleryState {
    pub toggle_val: bool,
    pub slider_val: f32,
    pub selected_segment: usize,
    pub text_input: String,
    pub caret_pos: usize,
    pub button_click_count: u32,
}

impl ComponentGalleryState {
    pub fn new() -> Self {
        Self {
            toggle_val: true,
            slider_val: 65.0,
            selected_segment: 0,
            text_input: "Search modules...".to_string(),
            caret_pos: 17,
            button_click_count: 0,
        }
    }
}

