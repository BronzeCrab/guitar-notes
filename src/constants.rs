use bevy::prelude::*;

pub const GAP: f32 = 50.0;
pub const GREY: Color = Color::srgba(0.6, 0.6, 0.6, 1.0);
pub const AMOUNT_OF_FRETS: u8 = 22;
/// Open-string labels sit this far left of the nut (`line_start_x`).
pub const OPEN_STRING_OFFSET_X: f32 = GAP / 2.0;
pub const FONT_SIZE: f32 = 22.0;
pub const RECT_SIZE: f32 = 30.0;
pub const GUITAR_OUTLINE_COLOR: Color = Color::srgba(0.15, 0.15, 0.15, 1.0);

/// One color per natural note, matching `NOTE_NAMES` order (A … G).
pub const COLORS: [Color; 7] = [
    Color::srgba(1.0, 0.0, 0.0, 1.0), // A
    Color::srgba(1.0, 0.5, 0.0, 1.0), // B
    Color::srgba(0.8, 0.7, 0.0, 1.0), // C
    Color::srgba(0.0, 1.0, 0.0, 1.0), // D
    Color::srgba(0.0, 0.5, 1.0, 1.0), // E
    Color::srgba(0.0, 0.0, 1.0, 1.0), // F
    Color::srgba(0.5, 0.0, 1.0, 1.0), // G
];

/// Amber highlight for selected notes (matches site accent #feca57).
pub const SELECTED_NOTE_COLOR: Color = Color::srgb(1.0, 0.78, 0.16);
pub const SELECTED_NOTE_TEXT_COLOR: Color = Color::srgb(0.12, 0.1, 0.05);

pub const MAX_SELECTED_NOTES: usize = 6;
pub const NOTE_PLAY_DURATION_MS: u64 = 900;
/// World-space lift so the fretboard sits above the bottom chord panel.
pub const CHORD_PANEL_CLEARANCE_Y: f32 = 170.0;
/// Camera world size: 22 frets + nut + open labels + side padding.
pub const WORLD_WIDTH: f32 = (AMOUNT_OF_FRETS as f32 + 3.0) * GAP;
pub const WORLD_HEIGHT: f32 = 6.0 * GAP + CHORD_PANEL_CLEARANCE_Y;
pub const CHORD_PANEL_MAX_WIDTH_PX: f32 = 480.0;
/// Keep the panel inside the canvas (avoids clipping on rounded/wasm edges).
pub const CHORD_PANEL_INSET_PX: f32 = 20.0;
pub const NARROW_UI_WIDTH_PX: f32 = 700.0;
pub const UI_FONT_SIZE_NARROW: f32 = 16.0;
pub const CHORD_INFO_FONT_SIZE: f32 = 16.0;
pub const CHORD_INFO_FONT_SIZE_NARROW: f32 = 14.0;
/// OrthographicProjection.scale: smaller = zoom in, larger = zoom out (1.0 = full fretboard).
pub const ZOOM_MIN: f32 = 0.45;
pub const ZOOM_MAX: f32 = 2.5;
pub const PAN_START_THRESHOLD_PX: f32 = 12.0;
pub const PAN_LIMIT_X: f32 = WORLD_WIDTH * 0.45;
pub const PAN_LIMIT_Y: f32 = WORLD_HEIGHT * 0.4;

pub fn ui_font_size(window_width: f32) -> f32 {
    if window_width < NARROW_UI_WIDTH_PX {
        UI_FONT_SIZE_NARROW
    } else {
        FONT_SIZE
    }
}

pub fn chord_info_font_size(window_width: f32) -> f32 {
    if window_width < NARROW_UI_WIDTH_PX {
        CHORD_INFO_FONT_SIZE_NARROW
    } else {
        CHORD_INFO_FONT_SIZE
    }
}
