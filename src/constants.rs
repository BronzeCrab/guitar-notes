use bevy::prelude::*;

pub const GAP: f32 = 50.0;
pub const GREY: Color = Color::srgba(0.6, 0.6, 0.6, 1.0);
pub const AMOUNT_OF_FRETS: u8 = 22;
pub const FONT_SIZE: f32 = 22.0;
pub const GUITAR_OUTLINE_COLOR: Color = Color::srgba(0.15, 0.15, 0.15, 1.0);

/// One color per natural note, matching `NoteName::ALL` order (A … G).
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
pub const SELECTION_OUTLINE_THICKNESS: f32 = 3.0;
pub const SELECTION_OUTLINE_COLOR: Color = Color::WHITE;
pub const PLAYING_TINT_COLOR: Color = Color::srgba(1.0, 0.95, 0.7, 0.5);

pub const MAX_SELECTED_NOTES: usize = 6;
pub const NOTE_PLAY_DURATION_MS: u64 = 900;
pub const NARROW_UI_WIDTH_PX: f32 = 700.0;
pub const MAX_FRETBOARD_CELL_PX: f32 = 52.0;

pub fn ui_font_size(window_width: f32) -> f32 {
    if window_width < NARROW_UI_WIDTH_PX {
        16.0
    } else {
        FONT_SIZE
    }
}
