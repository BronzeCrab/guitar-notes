use std::fmt;

/// Natural note names in chromatic order (index 0 = A).
/// `index()` matches the order of `COLORS` in constants.rs (A … G).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum NoteName {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
}

impl NoteName {
    pub const ALL: [NoteName; 7] = [
        NoteName::A,
        NoteName::B,
        NoteName::C,
        NoteName::D,
        NoteName::E,
        NoteName::F,
        NoteName::G,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            NoteName::A => "A",
            NoteName::B => "B",
            NoteName::C => "C",
            NoteName::D => "D",
            NoteName::E => "E",
            NoteName::F => "F",
            NoteName::G => "G",
        }
    }

    /// Semitone offset from A within one octave (chromatic pitch class).
    /// Gaps of 2 = whole step; gaps of 1 = half step (B–C, E–F).
    pub fn pitch_class(self) -> u8 {
        match self {
            NoteName::A => 0,
            NoteName::B => 2,
            NoteName::C => 3,
            NoteName::D => 5,
            NoteName::E => 7,
            NoteName::F => 8,
            NoteName::G => 10,
        }
    }

    /// Index in natural-note order (A=0 … G=6).
    pub fn index(self) -> usize {
        NoteName::ALL
            .iter()
            .position(|&n| n == self)
            .expect("NoteName is always in ALL")
    }
}

impl fmt::Display for NoteName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub const SEMITONES_PER_OCTAVE: u8 = 12;
pub const PERFECT_FIFTH_SEMITONES: u8 = 7;
pub const PERFECT_FOURTH_SEMITONES: u8 = 5; // inverted fifth

#[derive(Debug, Clone, PartialEq)]
pub struct PowerChordInfo {
    pub title: String,
    pub blurb: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NotePlacement {
    pub name: NoteName,
    pub hz: f32,
    pub octave: i8,
    pub fret: u8,
    pub string_index: u8,
}

pub fn note_name_for_pitch_class(pc: u8) -> Option<NoteName> {
    NoteName::ALL
        .iter()
        .copied()
        .find(|n| n.pitch_class() == pc)
}

pub fn detect_power_chord(note_names: &[NoteName]) -> Option<PowerChordInfo> {
    if note_names.is_empty() {
        return None;
    }

    let mut pcs: Vec<u8> = note_names.iter().map(|n| n.pitch_class()).collect();
    pcs.sort_unstable();
    pcs.dedup();

    if pcs.len() != 2 {
        return None;
    }

    let a = pcs[0];
    let b = pcs[1];
    let diff = (b + SEMITONES_PER_OCTAVE - a) % SEMITONES_PER_OCTAVE;

    let root_pc = if diff == PERFECT_FIFTH_SEMITONES {
        a
    } else if diff == PERFECT_FOURTH_SEMITONES {
        b
    } else {
        return None;
    };

    let root_name = note_name_for_pitch_class(root_pc)?;
    Some(PowerChordInfo {
        title: format!("{root_name}5"),
        blurb: format!(
            "{root_name}5 (power chord) - root and fifth, no third. Common in rock/metal. Usually 2-3 strings on guitar."
        ),
    })
}

pub fn guitar_string_number(string_index: u8) -> u8 {
    6 - string_index
}

pub fn format_note_lines(entries: &mut [NotePlacement]) -> String {
    entries.sort_by(|a, b| a.hz.partial_cmp(&b.hz).unwrap_or(std::cmp::Ordering::Equal));

    let mut lines = String::new();
    for note in entries.iter() {
        lines.push_str(&format!(
            "{} (oct {}) - fret {}, string {}\n",
            note.name,
            note.octave,
            note.fret,
            guitar_string_number(note.string_index),
        ));
    }
    lines
}
