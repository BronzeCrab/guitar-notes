use bevy::prelude::*;
use guitar_notes::music::note_index;
use std::sync::OnceLock;

#[derive(Component, Clone)]
pub struct Note {
    pub name: &'static str,
    pub hz: f32,
    pub octave: i8,
    pub semitones_from_a_4: f32,
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

pub fn note_hz(semitones_from_a_4: f32) -> f32 {
    440.0 * 2_f32.powf(semitones_from_a_4 / 12.0)
}

pub fn open_note(name: &'static str, semitones_from_a_4: f32, octave: i8) -> Note {
    Note {
        name,
        semitones_from_a_4,
        octave,
        hz: note_hz(semitones_from_a_4),
    }
}

pub fn tunings() -> &'static [Tuning] {
    static TUNINGS: OnceLock<Vec<Tuning>> = OnceLock::new();
    TUNINGS.get_or_init(|| {
        vec![
            Tuning {
                name: "Standard",
                notes: [
                    open_note("E", -29.0, 2),
                    open_note("A", -24.0, 2),
                    open_note("D", -19.0, 3),
                    open_note("G", -14.0, 3),
                    open_note("B", -10.0, 3),
                    open_note("E", -5.0, 4),
                ],
            },
            Tuning {
                name: "Drop D",
                notes: [
                    open_note("D", -31.0, 2),
                    open_note("A", -24.0, 2),
                    open_note("D", -19.0, 3),
                    open_note("G", -14.0, 3),
                    open_note("B", -10.0, 3),
                    open_note("E", -5.0, 4),
                ],
            },
            Tuning {
                name: "Drop C",
                notes: [
                    open_note("C", -33.0, 2), // C2  ~65.4 Hz
                    open_note("G", -26.0, 2), // G2  ~98 Hz
                    open_note("C", -21.0, 3), // C3  ~130.8 Hz
                    open_note("F", -16.0, 3), // F3  ~174.6 Hz
                    open_note("A", -12.0, 3), // A3  220 Hz
                    open_note("D", -7.0, 4),  // D4  ~293.7 Hz
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
