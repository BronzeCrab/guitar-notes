use bevy::prelude::*;
use guitar_notes::music::note_index;
use std::sync::OnceLock;

#[derive(Component, Clone)]
pub struct Note {
    pub name: &'static str,
    pub hz: f32,
    pub octave: i8,
    pub half_tones_from_a_4: f32,
}

#[derive(Clone)]
pub struct Tuning {
    pub name: &'static str,
    pub notes: [Note; 6],
}

#[derive(Resource)]
pub struct CurrentTuning {
    pub index: usize,
}

pub fn get_note_hz_in_4_octave(half_tones_from_a_4: f32) -> f32 {
    440.0 * 2_f32.powf(half_tones_from_a_4 / 12.0)
}

pub fn open_note(name: &'static str, half_tones_from_a_4: f32, octave: i8) -> Note {
    let divisor: f32 = 2_f32.powi((4 - octave) as i32);
    Note {
        name,
        half_tones_from_a_4,
        octave,
        hz: get_note_hz_in_4_octave(half_tones_from_a_4) / divisor,
    }
}

pub fn tunings() -> &'static [Tuning] {
    static TUNINGS: OnceLock<Vec<Tuning>> = OnceLock::new();
    TUNINGS.get_or_init(|| {
        vec![
            Tuning {
                name: "Standard",
                notes: [
                    open_note("E", -5.0, 2),
                    open_note("A", 0.0, 2),
                    open_note("D", -7.0, 3),
                    open_note("G", -2.0, 3),
                    open_note("B", 2.0, 3),
                    open_note("E", -5.0, 4),
                ],
            },
            Tuning {
                name: "Drop D",
                notes: [
                    open_note("D", -7.0, 2),
                    open_note("A", 0.0, 2),
                    open_note("D", -7.0, 3),
                    open_note("G", -2.0, 3),
                    open_note("B", 2.0, 3),
                    open_note("E", -5.0, 4),
                ],
            },
        ]
    })
}

pub fn tuning(index: usize) -> &'static Tuning {
    &tunings()[index]
}

pub fn color_index_for_note(name: &str) -> usize {
    note_index(name)
}
