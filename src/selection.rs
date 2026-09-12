use crate::audio::{NoteAudio, play_note_hz};
use crate::fretboard::{FretPosition, SelectedNote};
use crate::sequence::{
    AppMode, ChordPlayback, CurrentMode, SelectedOrder, Sequence, SequencePlayback,
};
use crate::tuning::Note;
use crate::ui::{
    ChordInfoText, ClearButton, ExplainButton, PlayButton, PowerChordPopup, PowerChordPopupText,
    SelectionPopupTitle,
};
use bevy::prelude::*;
use guitar_notes::music::{NoteName, NotePlacement, detect_power_chord, format_note_lines};

fn sequence_lines(sequence: &Sequence) -> String {
    sequence
        .0
        .iter()
        .enumerate()
        .map(|(i, placement)| {
            format!(
                "{}. {} (oct {}) - fret {}, string {}",
                i + 1,
                placement.name,
                placement.octave,
                placement.fret,
                guitar_notes::music::guitar_string_number(placement.string_index),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn note_placements(entries: &[(&Note, &FretPosition)]) -> Vec<NotePlacement> {
    entries
        .iter()
        .map(|(note, pos)| NotePlacement {
            name: note.name,
            hz: note.hz,
            octave: note.octave,
            fret: pos.fret,
            string_index: pos.string_index,
        })
        .collect()
}

#[allow(clippy::type_complexity)]
pub fn play_selected_notes(
    interactions: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    selected: Query<(&Note, &FretPosition), With<SelectedNote>>,
    audio: Res<NoteAudio>,
    mut chord_playback: ResMut<ChordPlayback>,
    mut popups: Query<&mut Visibility, With<PowerChordPopup>>,
    mut popup_texts: ParamSet<(
        Query<&mut Text, With<SelectionPopupTitle>>,
        Query<&mut Text, With<PowerChordPopupText>>,
    )>,
    mode: Res<CurrentMode>,
) {
    if mode.0 == AppMode::Sequence {
        return;
    }
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let entries: Vec<(&Note, &FretPosition)> = selected.iter().collect();
        for (note, _) in &entries {
            play_note_hz(&audio, note.hz);
        }

        if entries.is_empty() {
            continue;
        }

        chord_playback.playing = true;
        chord_playback.timer.reset();

        let names: Vec<NoteName> = entries.iter().map(|(note, _)| note.name).collect();
        let mut placements = note_placements(&entries);
        let note_lines = format_note_lines(&mut placements);
        let (title, body) = if let Some(info) = detect_power_chord(&names) {
            (
                info.title.clone(),
                format!("Why: root + fifth (no third).\n\nNotes:\n{note_lines}"),
            )
        } else {
            ("Played notes".to_string(), note_lines)
        };

        for mut text in &mut popup_texts.p0() {
            *text = Text::new(title.clone());
        }
        for mut text in &mut popup_texts.p1() {
            *text = Text::new(body.clone());
        }
        for mut visibility in &mut popups {
            *visibility = Visibility::Visible;
        }
    }
}

pub fn explain_selection(
    interactions: Query<&Interaction, (Changed<Interaction>, With<ExplainButton>)>,
    selected: Query<(&Note, &FretPosition), With<SelectedNote>>,
    mut info_texts: Query<&mut Text, With<ChordInfoText>>,
    mode: Res<CurrentMode>,
    sequence: Res<Sequence>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let message = if mode.0 == AppMode::Sequence {
            if sequence.0.is_empty() {
                "Sequence is empty. Click fretboard notes to add them in order.".to_string()
            } else {
                format!(
                    "Sequence ({} notes):\n{}",
                    sequence.0.len(),
                    sequence_lines(&sequence)
                )
            }
        } else if selected.is_empty() {
            "Select notes first, then press Explain.".to_string()
        } else {
            let entries: Vec<(&Note, &FretPosition)> = selected.iter().collect();
            let names: Vec<NoteName> = entries.iter().map(|(note, _)| note.name).collect();
            let mut placements = note_placements(&entries);
            let note_lines = format_note_lines(&mut placements);
            if let Some(info) = detect_power_chord(&names) {
                format!("{}\n{}\n\nNotes:\n{}", info.title, info.blurb, note_lines)
            } else {
                format!("Not a recognized chord.\n\nSelected notes:\n{note_lines}")
            }
        };

        for mut text in &mut info_texts {
            *text = Text::new(message.clone());
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn clear_selection(
    interactions: Query<&Interaction, (Changed<Interaction>, With<ClearButton>)>,
    mut commands: Commands,
    selected: Query<Entity, With<SelectedNote>>,
    mut info_texts: Query<&mut Text, With<ChordInfoText>>,
    mut popups: Query<&mut Visibility, With<PowerChordPopup>>,
    mode: Res<CurrentMode>,
    mut sequence: ResMut<Sequence>,
    mut playback: ResMut<SequencePlayback>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if mode.0 == AppMode::Sequence {
            sequence.0.clear();
            playback.playing = false;
            playback.cursor = 0;
        } else {
            for entity in &selected {
                commands
                    .entity(entity)
                    .remove::<(SelectedNote, SelectedOrder)>();
            }
        }

        for mut text in &mut info_texts {
            *text = Text::new("Select up to 6 notes, then Play or Explain.");
        }
        for mut visibility in &mut popups {
            *visibility = Visibility::Hidden;
        }
    }
}
