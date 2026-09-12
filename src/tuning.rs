use bevy::prelude::*;
use guitar_notes::music::NoteName;
use std::sync::OnceLock;

#[derive(Component, Clone)]
pub struct Note {
    pub name: NoteName,
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

#[derive(Clone, Copy)]
struct Octave4Base {
    note: NoteName,
    /// Semitones from A4 within the 4th octave (A4 = 0).
    offset: f32,
}

/// Pitch-class reference inside octave 4: how far each natural note sits from A4.
const OCTAVE4_BASE: [Octave4Base; 7] = [
    Octave4Base {
        note: NoteName::A,
        offset: 0.0,
    }, // A4  = 0
    Octave4Base {
        note: NoteName::B,
        offset: 2.0,
    }, // B4  = +2
    Octave4Base {
        note: NoteName::C,
        offset: -9.0,
    }, // C4  = -9
    Octave4Base {
        note: NoteName::D,
        offset: -7.0,
    }, // D4  = -7
    Octave4Base {
        note: NoteName::E,
        offset: -5.0,
    }, // E4  = -5
    Octave4Base {
        note: NoteName::F,
        offset: -4.0,
    }, // F4  = -4
    Octave4Base {
        note: NoteName::G,
        offset: -2.0,
    }, // G4  = -2
];

pub fn semitones_from_a_4(name: NoteName, octave: i8) -> f32 {
    let base_offset = OCTAVE4_BASE
        .iter()
        .find(|entry| entry.note == name)
        .expect("unknown note name")
        .offset;
    base_offset - 12.0 * (4 - octave) as f32
}

pub fn note_hz(semitones_from_a_4: f32) -> f32 {
    440.0 * 2_f32.powf(semitones_from_a_4 / 12.0)
}

pub fn open_note(name: NoteName, octave: i8) -> Note {
    let s = semitones_from_a_4(name, octave);
    Note {
        name,
        semitones_from_a_4: s,
        octave,
        hz: note_hz(s),
    }
}

pub fn tunings() -> &'static [Tuning] {
    static TUNINGS: OnceLock<Vec<Tuning>> = OnceLock::new();
    TUNINGS.get_or_init(|| {
        vec![
            Tuning {
                name: "Standard",
                notes: [
                    open_note(NoteName::E, 2), // E2
                    open_note(NoteName::A, 2), // A2
                    open_note(NoteName::D, 3), // D3
                    open_note(NoteName::G, 3), // G3
                    open_note(NoteName::B, 3), // B3
                    open_note(NoteName::E, 4), // E4
                ],
            },
            Tuning {
                name: "Drop D",
                notes: [
                    open_note(NoteName::D, 2), // D2
                    open_note(NoteName::A, 2), // A2
                    open_note(NoteName::D, 3), // D3
                    open_note(NoteName::G, 3), // G3
                    open_note(NoteName::B, 3), // B3
                    open_note(NoteName::E, 4), // E4
                ],
            },
            Tuning {
                name: "Drop C",
                notes: [
                    open_note(NoteName::C, 2), // C2  ~65.4 Hz
                    open_note(NoteName::G, 2), // G2  ~98 Hz
                    open_note(NoteName::C, 3), // C3  ~130.8 Hz
                    open_note(NoteName::F, 3), // F3  ~174.6 Hz
                    open_note(NoteName::A, 3), // A3  220 Hz
                    open_note(NoteName::D, 4), // D4  ~293.7 Hz
                ],
            },
        ]
    })
}

pub fn tuning(index: usize) -> &'static Tuning {
    &tunings()[index]
}

pub fn color_index_for_note(name: NoteName) -> usize {
    name.index()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_hz(name: NoteName, octave: i8, expected: f32) {
        let note = open_note(name, octave);
        assert!(
            (note.hz - expected).abs() < 0.05,
            "{}{}: {} != {}",
            name,
            octave,
            note.hz,
            expected
        );
    }

    #[test]
    fn open_note_hz_standard_open_strings() {
        assert_hz(NoteName::E, 2, 82.41);
        assert_hz(NoteName::A, 2, 110.0);
        assert_hz(NoteName::D, 3, 146.83);
        assert_hz(NoteName::G, 3, 196.0);
        assert_hz(NoteName::B, 3, 246.94);
        assert_hz(NoteName::E, 4, 329.63);
    }

    #[test]
    fn open_note_hz_drop_c_low_string() {
        assert_hz(NoteName::C, 2, 65.41);
    }

    #[test]
    fn semitones_a4_reference() {
        assert_eq!(semitones_from_a_4(NoteName::A, 4), 0.0);
        assert_eq!(semitones_from_a_4(NoteName::E, 2), -29.0);
        assert_eq!(semitones_from_a_4(NoteName::C, 2), -33.0);
    }
}
