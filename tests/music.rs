use guitar_notes::music::{
    detect_power_chord, format_note_lines, guitar_string_number, note_name_for_pitch_class,
    pitch_class, NotePlacement,
};

#[test]
fn pitch_class_natural_notes() {
    assert_eq!(pitch_class("A"), 0);
    assert_eq!(pitch_class("E"), 7);
    assert_eq!(pitch_class("G"), 10);
}

#[test]
fn pitch_class_roundtrip() {
    for &(name, pc) in guitar_notes::music::NATURAL_PITCH_CLASSES.iter() {
        assert_eq!(pitch_class(name), pc);
        assert_eq!(note_name_for_pitch_class(pc), Some(name));
    }
}

#[test]
fn detects_e_power_chord() {
    let info = detect_power_chord(&["E", "B"]).unwrap();
    assert_eq!(info.title, "E5");
}

#[test]
fn detects_a_power_chord() {
    let info = detect_power_chord(&["A", "E"]).unwrap();
    assert_eq!(info.title, "A5");
}

#[test]
fn detects_inverted_fifth_as_e_power_chord() {
    let info = detect_power_chord(&["B", "E"]).unwrap();
    assert_eq!(info.title, "E5");
}

#[test]
fn detects_power_chord_with_octave_doubling() {
    let info = detect_power_chord(&["E", "B", "E"]).unwrap();
    assert_eq!(info.title, "E5");
}

#[test]
fn rejects_empty_selection() {
    assert!(detect_power_chord(&[]).is_none());
}

#[test]
fn rejects_single_note() {
    assert!(detect_power_chord(&["E"]).is_none());
}

#[test]
fn rejects_unrelated_interval() {
    assert!(detect_power_chord(&["C", "E"]).is_none());
}

#[test]
fn guitar_string_number_convention() {
    assert_eq!(guitar_string_number(0), 6);
    assert_eq!(guitar_string_number(5), 1);
}

#[test]
fn format_note_lines_sorts_by_hz() {
    let mut entries = [
        NotePlacement {
            name: "B",
            hz: 246.94,
            octave: 3,
            fret: 2,
            string_index: 1,
        },
        NotePlacement {
            name: "E",
            hz: 82.41,
            octave: 2,
            fret: 2,
            string_index: 0,
        },
    ];

    let lines = format_note_lines(&mut entries);
    let e_pos = lines.find("E (oct 2)").unwrap();
    let b_pos = lines.find("B (oct 3)").unwrap();
    assert!(e_pos < b_pos);
    assert!(lines.contains("fret 2, string 6"));
    assert!(lines.contains("fret 2, string 5"));
}
