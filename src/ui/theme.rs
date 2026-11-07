use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeMode {
    PureBlack,
    EmberDark,
    Vivid,
    Silver,
}

#[derive(Clone, Copy, Debug)]
pub struct ThemeTokens {
    pub mode: ThemeMode,
    pub panel_bg: D2D1_COLOR_F,
    pub notch_collapsed_bg: D2D1_COLOR_F,
    pub card_bg: D2D1_COLOR_F,
    pub card_border: D2D1_COLOR_F,
    pub dock_bg: D2D1_COLOR_F,
    pub text_primary: D2D1_COLOR_F,
    pub text_secondary: D2D1_COLOR_F,
    pub text_muted: D2D1_COLOR_F,
    pub accent: D2D1_COLOR_F,
    pub accent_rgb: (f32, f32, f32),
    pub ring_track: D2D1_COLOR_F,
    pub outline: D2D1_COLOR_F,
}

impl ThemeTokens {
    pub fn from_name(name: &str) -> Self {
        match name {
            "Ember Dark" => Self::ember_dark(),
            "Vivid" => Self::vivid(),
            "Silver" => Self::silver(),
            _ => Self::pure_black(),
        }
    }

    pub fn pure_black() -> Self {
        Self {
            mode: ThemeMode::PureBlack,
            panel_bg: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            notch_collapsed_bg: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            card_bg: D2D1_COLOR_F { r: 0.063, g: 0.063, b: 0.063, a: 1.0 }, // #101010
            card_border: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.06 },
            dock_bg: D2D1_COLOR_F { r: 0.039, g: 0.039, b: 0.039, a: 1.0 }, // #0A0A0A
            text_primary: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            text_secondary: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.55 },
            text_muted: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.35 },
            accent: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White accent
            accent_rgb: (1.0, 1.0, 1.0),
            ring_track: D2D1_COLOR_F { r: 0.16, g: 0.16, b: 0.18, a: 0.8 },
            outline: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.08 },
        }
    }

    pub fn ember_dark() -> Self {
        // Orange accent #FF6A2B -> rgb(1.0, 0.416, 0.169)
        Self {
            mode: ThemeMode::EmberDark,
            panel_bg: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            notch_collapsed_bg: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            card_bg: D2D1_COLOR_F { r: 0.063, g: 0.063, b: 0.063, a: 1.0 }, // #101010
            card_border: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.07 },
            dock_bg: D2D1_COLOR_F { r: 0.039, g: 0.039, b: 0.039, a: 1.0 }, // #0A0A0A
            text_primary: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            text_secondary: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.55 },
            text_muted: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.35 },
            accent: D2D1_COLOR_F { r: 1.0, g: 0.416, b: 0.169, a: 1.0 }, // #FF6A2B
            accent_rgb: (1.0, 0.416, 0.169),
            ring_track: D2D1_COLOR_F { r: 0.16, g: 0.14, b: 0.12, a: 0.8 },
            outline: D2D1_COLOR_F { r: 1.0, g: 0.416, b: 0.169, a: 0.14 },
        }
    }

    pub fn vivid() -> Self {
        // Blue accent #2F7BFF -> rgb(0.184, 0.482, 1.0)
        Self {
            mode: ThemeMode::Vivid,
            panel_bg: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            notch_collapsed_bg: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            card_bg: D2D1_COLOR_F { r: 0.063, g: 0.063, b: 0.063, a: 1.0 }, // #101010
            card_border: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.07 },
            dock_bg: D2D1_COLOR_F { r: 0.039, g: 0.039, b: 0.039, a: 1.0 }, // #0A0A0A
            text_primary: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            text_secondary: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.55 },
            text_muted: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 0.35 },
            accent: D2D1_COLOR_F { r: 0.184, g: 0.482, b: 1.0, a: 1.0 }, // #2F7BFF
            accent_rgb: (0.184, 0.482, 1.0),
            ring_track: D2D1_COLOR_F { r: 0.12, g: 0.14, b: 0.18, a: 0.8 },
            outline: D2D1_COLOR_F { r: 0.184, g: 0.482, b: 1.0, a: 0.14 },
        }
    }

    pub fn silver() -> Self {
        // Light panel #F2F2F2, the notch body & collapsed pill stay black
        Self {
            mode: ThemeMode::Silver,
            panel_bg: D2D1_COLOR_F { r: 0.949, g: 0.949, b: 0.949, a: 1.0 }, // #F2F2F2
            notch_collapsed_bg: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }, // Notch stays black
            card_bg: D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }, // White cards
            card_border: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.08 },
            dock_bg: D2D1_COLOR_F { r: 0.88, g: 0.88, b: 0.88, a: 1.0 },
            text_primary: D2D1_COLOR_F { r: 0.08, g: 0.08, b: 0.08, a: 1.0 }, // Dark text
            text_secondary: D2D1_COLOR_F { r: 0.35, g: 0.35, b: 0.35, a: 1.0 },
            text_muted: D2D1_COLOR_F { r: 0.55, g: 0.55, b: 0.55, a: 1.0 },
            accent: D2D1_COLOR_F { r: 0.0, g: 0.48, b: 0.85, a: 1.0 }, // Subtle blue accent
            accent_rgb: (0.0, 0.48, 0.85),
            ring_track: D2D1_COLOR_F { r: 0.85, g: 0.85, b: 0.85, a: 0.8 },
            outline: D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.12 },
        }
    }
}

