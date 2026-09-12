use crate::audio::{NoteAudio, play_note_hz};
use crate::constants::COLORS;
use crate::fretboard::{FretPosition, NoteVisual, SelectedNote, set_note_label_color};
use crate::tuning::Note;
use crate::ui::{
    ChordInfoText, ClearButton, ExplainButton, PlayButton, PowerChordPopup, PowerChordPopupText,
    SelectionPopupTitle,
};
use bevy::prelude::*;
use guitar_notes::music::{NoteName, NotePlacement, detect_power_chord, format_note_lines};

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
    mut popups: Query<&mut Visibility, With<PowerChordPopup>>,
    mut popup_texts: ParamSet<(
        Query<&mut Text, With<SelectionPopupTitle>>,
        Query<&mut Text, With<PowerChordPopupText>>,
    )>,
) {
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
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let entries: Vec<(&Note, &FretPosition)> = selected.iter().collect();
        let message = if entries.is_empty() {
            "Select notes first, then press Explain.".to_string()
        } else {
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
    mut materials: ResMut<Assets<ColorMaterial>>,
    selected: Query<(Entity, &NoteVisual, &MeshMaterial2d<ColorMaterial>), With<SelectedNote>>,
    children_q: Query<&Children>,
    mut text_colors: Query<&mut TextColor>,
    mut info_texts: Query<&mut Text, With<ChordInfoText>>,
    mut popups: Query<&mut Visibility, With<PowerChordPopup>>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        for (entity, visual, material) in &selected {
            if let Some(mut mat) = materials.get_mut(material.id()) {
                mat.color = COLORS[visual.color_index];
            }
            set_note_label_color(entity, Color::WHITE, &children_q, &mut text_colors);
            commands.entity(entity).remove::<SelectedNote>();
        }

        for mut text in &mut info_texts {
            *text = Text::new("Select up to 6 notes, then Play or Explain.");
        }
        for mut visibility in &mut popups {
            *visibility = Visibility::Hidden;
        }
    }
}
