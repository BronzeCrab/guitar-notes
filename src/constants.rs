use bevy::prelude::*;

pub const GAP: f32 = 50.0;
pub const GREY: Color = Color::srgba(0.6, 0.6, 0.6, 1.0);
pub const AMOUNT_OF_FRETS: u8 = 22;

/// Bounds of the root font size (Bevy's `RemSize`). All UI text and spacings
/// are expressed in `rem`, so the whole chrome scales with the window.
pub const MIN_REM_SIZE_PX: f32 = 13.0;
pub const MAX_REM_SIZE_PX: f32 = 22.0;
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
pub const SELECTION_OUTLINE_THICKNESS: f32 = 3.0;
pub const SELECTION_OUTLINE_COLOR: Color = Color::srgb(1.0, 0.78, 0.16);
pub const PLAYING_TINT_COLOR: Color = Color::srgba(1.0, 0.95, 0.7, 0.5);

pub const MAX_SELECTED_NOTES: usize = 6;
pub const NOTE_PLAY_DURATION_MS: u64 = 900;
pub const MAX_FRETBOARD_CELL_PX: f32 = 52.0;

/// Smallest / largest row height (fret to fret) in the portrait fretboard.
/// The minimum is low enough that a typical phone shows all 22 frets; only
/// very small screens fall back to scrolling.
pub const MIN_PORTRAIT_FRET_STEP_PX: f32 = 24.0;
pub const MAX_PORTRAIT_FRET_STEP_PX: f32 = 44.0;

/// Smallest / largest column width (string to string) in the portrait fretboard.
pub const MIN_PORTRAIT_STRING_STEP_PX: f32 = 34.0;
pub const MAX_PORTRAIT_STRING_STEP_PX: f32 = 72.0;

/// Root font size for the viewport's smaller side: ~13 px on a phone in any
/// orientation and 22 px on a desktop, growing continuously instead of
/// switching at breakpoints.
pub fn rem_size_for_viewport(viewport_min: f32) -> f32 {
    (viewport_min * 0.034).clamp(MIN_REM_SIZE_PX, MAX_REM_SIZE_PX)
}
