/// Semitone offsets from A within one octave (chromatic pitch class).
/// Gaps of 2 = whole step; gaps of 1 = half step (B–C, E–F).
pub const NATURAL_PITCH_CLASSES: [(&str, u8); 7] = [
    ("A", 0),
    ("B", 2),
    ("C", 3),
    ("D", 5),
    ("E", 7),
    ("F", 8),
    ("G", 10),
];
pub const SEMITONES_PER_OCTAVE: u8 = 12;
pub const PERFECT_FIFTH_SEMITONES: u8 = 7;
pub const PERFECT_FOURTH_SEMITONES: u8 = 5; // inverted fifth

#[derive(Debug, Clone, PartialEq)]
pub struct PowerChordInfo {
    pub title: String,
    pub blurb: String,
}

#[derive(Debug, Clone)]
pub struct NotePlacement {
    pub name: &'static str,
    pub hz: f32,
    pub octave: i8,
    pub fret: u8,
    pub string_index: u8,
}

pub fn pitch_class(name: &str) -> u8 {
    NATURAL_PITCH_CLASSES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, pc)| *pc)
        .unwrap_or_else(|| panic!("unknown note name: {name}"))
}

pub fn note_name_for_pitch_class(pc: u8) -> Option<&'static str> {
    NATURAL_PITCH_CLASSES
        .iter()
        .find(|(_, mapped)| *mapped == pc)
        .map(|(name, _)| *name)
}

pub fn detect_power_chord(note_names: &[&str]) -> Option<PowerChordInfo> {
    if note_names.is_empty() {
        return None;
    }

    let mut pcs: Vec<u8> = note_names.iter().map(|n| pitch_class(n)).collect();
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
    entries.sort_by(|a, b| {
        a.hz
            .partial_cmp(&b.hz)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
