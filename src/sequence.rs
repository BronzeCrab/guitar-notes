use crate::audio::{NoteAudio, play_note_hz};
use crate::constants::{
    COLORS, NOTE_PLAY_DURATION_MS, SELECTED_NOTE_COLOR, SELECTED_NOTE_TEXT_COLOR,
};
use crate::fretboard::{
    FretNote, FretPosition, NoteVisual, PlayingTint, SelectedNote, SelectionRing, set_note_visual,
};
use crate::tuning::Note;
use crate::ui::PlayButton;
use bevy::prelude::*;
use guitar_notes::music::{NotePlacement, guitar_string_number};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum AppMode {
    #[default]
    Chord,
    Sequence,
}

#[derive(Resource, Default)]
pub struct CurrentMode(pub AppMode);

#[derive(Resource, Default)]
pub struct Sequence(pub Vec<NotePlacement>);

/// Ordered chord selection mirrored from the fretboard each frame.
#[derive(Resource, Default)]
pub struct ChordSelection {
    pub entries: Vec<NotePlacement>,
}

#[derive(Resource, Default)]
pub struct SelectionCounter(pub u64);

/// Click order (1-based rank is derived from the sorted list on the fly).
#[derive(Component, Clone, Copy)]
pub struct SelectedOrder {
    pub order: u64,
}

#[derive(Resource, Default)]
pub struct ChordSelectionToken(pub u64);

#[derive(Component)]
pub struct ChordMark {
    pub token: u64,
}

#[derive(Resource)]
pub struct ChordPlayback {
    pub playing: bool,
    pub timer: Timer,
}

impl Default for ChordPlayback {
    fn default() -> Self {
        Self {
            playing: false,
            timer: Timer::from_seconds(NOTE_PLAY_DURATION_MS as f32 / 1000.0, TimerMode::Once),
        }
    }
}

#[derive(Component)]
pub struct ChordModeButton;

#[derive(Component)]
pub struct SequenceModeButton;

#[derive(Component)]
pub struct SelectedNotesPanel {
    pub main: f32,
    pub sub: f32,
}

#[derive(Component)]
pub struct SelectedNotesEntry {
    pub index: usize,
}

#[derive(Component)]
pub struct SequenceMark {
    pub token: u64,
}

#[derive(Resource, Default)]
pub struct SequenceToken(pub u64);

#[derive(Resource)]
pub struct SequencePlayback {
    pub playing: bool,
    pub cursor: usize,
    pub timer: Timer,
}

impl Default for SequencePlayback {
    fn default() -> Self {
        Self {
            playing: false,
            cursor: 0,
            timer: Timer::from_seconds(NOTE_PLAY_DURATION_MS as f32 / 1000.0, TimerMode::Once),
        }
    }
}

fn mode_button_style() -> (BackgroundColor, BorderColor) {
    (
        BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
        BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
    )
}

fn active_button_style() -> (BackgroundColor, BorderColor) {
    (
        BackgroundColor(Color::srgb(0.35, 0.35, 0.45)),
        BorderColor::all(Color::srgb(0.85, 0.8, 0.5)),
    )
}

fn spawn_mode_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    marker: impl Bundle,
    font_size: f32,
    active: bool,
) {
    let (bg, border) = if active {
        active_button_style()
    } else {
        mode_button_style()
    };
    parent
        .spawn((
            Button,
            marker,
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                min_width: Val::Px(90.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            bg,
            border,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(font_size),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

pub fn spawn_mode_buttons(commands: &mut Commands, font_size: f32) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                ..default()
            },
            ZIndex(10),
            Pickable::IGNORE,
        ))
        .with_children(|bar| {
            spawn_mode_button(bar, "Chord", ChordModeButton, font_size, true);
            spawn_mode_button(bar, "Sequence", SequenceModeButton, font_size, false);
        });
}

/// Selected-notes display placed inside the bottom panel, above the Play/Explain row.
/// Shown as a distinct sub-block in both chord and sequence modes.
pub fn spawn_selected_notes_panel(
    parent: &mut ChildSpawnerCommands,
    main_font_size: f32,
    sub_font_size: f32,
) {
    parent
        .spawn((
            SelectedNotesPanel {
                main: main_font_size,
                sub: sub_font_size,
            },
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.14, 0.14, 0.18, 0.9)),
            BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Selected Notes:"),
                TextFont {
                    font_size: FontSize::Px(main_font_size),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.9)),
            ));
            panel.spawn((Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(6.0),
                align_items: AlignItems::Center,
                ..default()
            },));
        });
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn toggle_mode(
    chord_clicks: Query<&Interaction, (Changed<Interaction>, With<ChordModeButton>)>,
    sequence_clicks: Query<&Interaction, (Changed<Interaction>, With<SequenceModeButton>)>,
    mut current_mode: ResMut<CurrentMode>,
    mut sequence: ResMut<Sequence>,
    mut playback: ResMut<SequencePlayback>,
    mut commands: Commands,
    selected: Query<Entity, With<SelectedNote>>,
    mut mode_changers: ParamSet<(
        Query<(&ChordModeButton, &mut BackgroundColor, &mut BorderColor)>,
        Query<(&SequenceModeButton, &mut BackgroundColor, &mut BorderColor)>,
    )>,
) {
    let want_chord = chord_clicks.iter().any(|i| *i == Interaction::Pressed);
    let want_sequence = sequence_clicks.iter().any(|i| *i == Interaction::Pressed);
    if !want_chord && !want_sequence {
        return;
    }

    let target = if want_chord {
        AppMode::Chord
    } else {
        AppMode::Sequence
    };
    if target == current_mode.0 {
        return;
    }

    current_mode.0 = target;

    if target == AppMode::Sequence {
        for entity in &selected {
            commands
                .entity(entity)
                .remove::<(SelectedNote, SelectedOrder)>();
        }
    } else {
        sequence.0.clear();
        playback.playing = false;
        playback.cursor = 0;
    }

    set_mode_button_styles(&target, &mut mode_changers);
}

#[allow(clippy::type_complexity)]
fn set_mode_button_styles(
    target: &AppMode,
    changers: &mut ParamSet<(
        Query<(&ChordModeButton, &mut BackgroundColor, &mut BorderColor)>,
        Query<(&SequenceModeButton, &mut BackgroundColor, &mut BorderColor)>,
    )>,
) {
    for (_, mut bg, mut border) in &mut changers.p0() {
        let style = if *target == AppMode::Chord {
            active_button_style()
        } else {
            mode_button_style()
        };
        *bg = style.0;
        *border = style.1;
    }
    for (_, mut bg, mut border) in &mut changers.p1() {
        let style = if *target == AppMode::Sequence {
            active_button_style()
        } else {
            mode_button_style()
        };
        *bg = style.0;
        *border = style.1;
    }
}

/// Keep `ChordSelection` in sync with the actual `SelectedNote` markers on the fretboard.
pub fn track_selected_notes(
    selected: Query<(&SelectedOrder, &Note, &FretPosition), With<SelectedNote>>,
    mut chord_selection: ResMut<ChordSelection>,
) {
    let mut ranked: Vec<(u64, NotePlacement)> = selected
        .iter()
        .map(|(order, note, position)| {
            (
                order.order,
                NotePlacement {
                    name: note.name,
                    hz: note.hz,
                    octave: note.octave,
                    fret: position.fret,
                    string_index: position.string_index,
                },
            )
        })
        .collect();
    ranked.sort_by_key(|(order, _)| *order);
    let entries: Vec<NotePlacement> = ranked.into_iter().map(|(_, p)| p).collect();
    if entries != chord_selection.entries {
        chord_selection.entries = entries;
    }
}

/// Rebuild the full "Selected Notes" block whenever the active list changes.
/// The whole subtree (header + row + columns) is re-created from the panel entity,
/// so it does not depend on a pre-existing row marker.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_selected_notes_panel(
    sequence: Res<Sequence>,
    chord_selection: Res<ChordSelection>,
    mode: Res<CurrentMode>,
    playback: Res<SequencePlayback>,
    chord_playback: Res<ChordPlayback>,
    mut commands: Commands,
    panels: Query<(Entity, &SelectedNotesPanel)>,
    children_q: Query<&Children>,
) {
    if !(sequence.is_changed()
        || chord_selection.is_changed()
        || mode.is_changed()
        || playback.is_changed()
        || chord_playback.is_changed())
    {
        return;
    }

    let in_sequence = mode.0 == AppMode::Sequence;
    let entries: Vec<(usize, NotePlacement)> = if in_sequence {
        sequence
            .0
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.clone()))
            .collect()
    } else {
        chord_selection
            .entries
            .iter()
            .enumerate()
            .map(|(i, p)| (i, p.clone()))
            .collect()
    };

    for (panel, font) in &panels {
        if let Ok(children) = children_q.get(panel) {
            for child in children.iter() {
                commands.entity(child).despawn();
            }
        }

        commands.entity(panel).with_children(|parent| {
            parent.spawn((
                Text::new("Selected Notes:"),
                TextFont {
                    font_size: FontSize::Px(font.main),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.9)),
            ));

            if entries.is_empty() {
                let text = if in_sequence {
                    "Click fretboard notes to add them in order."
                } else {
                    "No notes selected yet. Click fretboard notes."
                };
                parent.spawn((
                    Text::new(text),
                    TextFont {
                        font_size: FontSize::Px(font.sub),
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.65)),
                ));
                return;
            }

            let count = entries.len();
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    for (index, placement) in &entries {
                        let highlighted = if in_sequence {
                            playback.playing && playback.cursor == *index
                        } else {
                            chord_playback.playing
                        };
                        let (bg, border) = if highlighted {
                            (
                                BackgroundColor(Color::srgb(0.55, 0.45, 0.2)),
                                BorderColor::all(Color::srgb(1.0, 0.78, 0.16)),
                            )
                        } else {
                            (
                                BackgroundColor(Color::srgb(0.22, 0.22, 0.28)),
                                BorderColor::all(Color::srgb(0.4, 0.4, 0.45)),
                            )
                        };
                        row.spawn((
                            Button,
                            SelectedNotesEntry { index: *index },
                            Node {
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            bg,
                            border,
                        ))
                        .with_children(|entry| {
                            entry.spawn((
                                Text::new(format!("{}. {}", index + 1, placement.name)),
                                TextFont {
                                    font_size: FontSize::Px(font.main),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                            entry.spawn((
                                Text::new(format!(
                                    "fret {}, str {}",
                                    placement.fret,
                                    guitar_string_number(placement.string_index)
                                )),
                                TextFont {
                                    font_size: FontSize::Px(font.sub),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.7, 0.7, 0.75)),
                            ));
                        });
                        if index + 1 < count {
                            row.spawn((
                                Text::new(","),
                                TextFont {
                                    font_size: FontSize::Px(font.main),
                                    ..default()
                                },
                                TextColor(Color::srgb(0.7, 0.7, 0.75)),
                            ));
                        }
                    }
                });
        });
    }
}

#[allow(clippy::type_complexity)]
pub fn handle_selected_entry_click(
    interactions: Query<(&Interaction, &SelectedNotesEntry), (Changed<Interaction>, With<Button>)>,
    mut commands: Commands,
    mode: Res<CurrentMode>,
    mut sequence: ResMut<Sequence>,
    mut playback: ResMut<SequencePlayback>,
    chord_selection: Res<ChordSelection>,
    chord_notes: Query<(Entity, &FretPosition), With<SelectedNote>>,
) {
    for (interaction, entry) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if mode.0 == AppMode::Sequence {
            if entry.index < sequence.0.len() {
                sequence.0.remove(entry.index);
            }
            playback.playing = false;
            continue;
        }
        if let Some(placement) = chord_selection.entries.get(entry.index) {
            for (entity, position) in &chord_notes {
                if position.string_index == placement.string_index
                    && position.fret == placement.fret
                {
                    commands
                        .entity(entity)
                        .remove::<(SelectedNote, SelectedOrder)>();
                    break;
                }
            }
        }
    }
}

/// Reset stale chord marks, then stamp the current chord selection onto the fretboard.
#[allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::collapsible_if
)]
pub fn refresh_chord_visuals(
    mut commands: Commands,
    mode: Res<CurrentMode>,
    chord_selection: Res<ChordSelection>,
    mut chord_token: ResMut<ChordSelectionToken>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    notes: Query<
        (
            Entity,
            &Note,
            &NoteVisual,
            &FretPosition,
            &MeshMaterial2d<ColorMaterial>,
            Option<&ChordMark>,
        ),
        With<FretNote>,
    >,
    children_q: Query<&Children>,
    mut labels: Query<(&mut Text, &mut TextColor)>,
    mut rings: Query<&mut Visibility, (With<SelectionRing>, Without<PlayingTint>)>,
    mut tints: Query<&mut Visibility, (With<PlayingTint>, Without<SelectionRing>)>,
) {
    if !(chord_selection.is_changed() || mode.is_changed()) {
        return;
    }

    chord_token.0 += 1;
    let token = chord_token.0;

    for (entity, note, visual, _, material, mark) in &notes {
        if let Some(mark) = mark {
            if mark.token == token {
                continue;
            }
            if let Some(mut mat) = materials.get_mut(material.id()) {
                mat.color = COLORS[visual.color_index];
            }
            set_note_visual(
                entity,
                note.name.as_str(),
                Color::WHITE,
                false,
                false,
                &children_q,
                &mut labels,
                &mut rings,
                &mut tints,
            );
            commands.entity(entity).remove::<ChordMark>();
        }
    }

    if mode.0 != AppMode::Chord || chord_selection.entries.is_empty() {
        return;
    }

    let mut by_position: HashMap<(u8, u8), Entity> = HashMap::new();
    for (entity, _, _, position, _, _) in &notes {
        by_position.insert((position.string_index, position.fret), entity);
    }

    for (index, placement) in chord_selection.entries.iter().enumerate() {
        let Some(entity) = by_position.get(&(placement.string_index, placement.fret)) else {
            continue;
        };
        if let Ok((_, _, _, _, material, _)) = notes.get(*entity) {
            if let Some(mut mat) = materials.get_mut(material.id()) {
                mat.color = SELECTED_NOTE_COLOR;
            }
        }
        set_note_visual(
            *entity,
            &(index + 1).to_string(),
            SELECTED_NOTE_TEXT_COLOR,
            true,
            false,
            &children_q,
            &mut labels,
            &mut rings,
            &mut tints,
        );
        commands.entity(*entity).insert(ChordMark { token });
    }
}

/// Tick the chord playback timer so the panel highlight turns off after the notes ring out.
pub fn chord_playback_tick(
    time: Res<Time>,
    mode: Res<CurrentMode>,
    mut playback: ResMut<ChordPlayback>,
) {
    if mode.0 != AppMode::Chord {
        if playback.playing {
            playback.playing = false;
            playback.timer.reset();
        }
        return;
    }
    if !playback.playing {
        return;
    }
    playback.timer.tick(time.delta());
    if playback.timer.is_finished() {
        playback.playing = false;
    }
}

/// Reset stale sequence marks, then stamp the current sequence order onto the fretboard.
#[allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::collapsible_if
)]
pub fn refresh_sequence_visuals(
    mut commands: Commands,
    mode: Res<CurrentMode>,
    sequence: Res<Sequence>,
    mut sequence_token: ResMut<SequenceToken>,
    playback: Res<SequencePlayback>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    notes: Query<
        (
            Entity,
            &Note,
            &NoteVisual,
            &FretPosition,
            &MeshMaterial2d<ColorMaterial>,
            Option<&SequenceMark>,
        ),
        With<FretNote>,
    >,
    children_q: Query<&Children>,
    mut labels: Query<(&mut Text, &mut TextColor)>,
    mut rings: Query<&mut Visibility, (With<SelectionRing>, Without<PlayingTint>)>,
    mut tints: Query<&mut Visibility, (With<PlayingTint>, Without<SelectionRing>)>,
) {
    if !(sequence.is_changed() || mode.is_changed() || playback.is_changed()) {
        return;
    }

    sequence_token.0 += 1;
    let token = sequence_token.0;

    for (entity, note, visual, _, material, mark) in &notes {
        if let Some(mark) = mark {
            if mark.token == token {
                continue;
            }
            if let Some(mut mat) = materials.get_mut(material.id()) {
                mat.color = COLORS[visual.color_index];
            }
            set_note_visual(
                entity,
                note.name.as_str(),
                Color::WHITE,
                false,
                false,
                &children_q,
                &mut labels,
                &mut rings,
                &mut tints,
            );
            commands.entity(entity).remove::<SequenceMark>();
        }
    }

    if mode.0 != AppMode::Sequence || sequence.0.is_empty() {
        return;
    }

    let mut by_position: HashMap<(u8, u8), Entity> = HashMap::new();
    for (entity, _, _, position, _, _) in &notes {
        by_position.insert((position.string_index, position.fret), entity);
    }

    for (index, placement) in sequence.0.iter().enumerate() {
        let Some(entity) = by_position.get(&(placement.string_index, placement.fret)) else {
            continue;
        };
        let playing = playback.playing && playback.cursor == index;
        if let Ok((_, _, _, _, material, _)) = notes.get(*entity) {
            if let Some(mut mat) = materials.get_mut(material.id()) {
                mat.color = SELECTED_NOTE_COLOR;
            }
        }
        set_note_visual(
            *entity,
            &(index + 1).to_string(),
            SELECTED_NOTE_TEXT_COLOR,
            true,
            playing,
            &children_q,
            &mut labels,
            &mut rings,
            &mut tints,
        );
        commands.entity(*entity).insert(SequenceMark { token });
    }
}

/// Plays the sequence in order, one note per `NOTE_PLAY_DURATION_MS`.
#[allow(clippy::type_complexity)]
pub fn sequence_playback(
    time: Res<Time>,
    mode: Res<CurrentMode>,
    mut playback: ResMut<SequencePlayback>,
    sequence: Res<Sequence>,
    audio: Res<NoteAudio>,
    play_clicks: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
) {
    if mode.0 != AppMode::Sequence {
        return;
    }

    for interaction in &play_clicks {
        if *interaction == Interaction::Pressed && !sequence.0.is_empty() {
            playback.playing = true;
            playback.cursor = 0;
            playback.timer.reset();
            play_note_hz(&audio, sequence.0[0].hz);
        }
    }

    if !playback.playing {
        return;
    }

    playback.timer.tick(time.delta());

    if playback.timer.is_finished() {
        playback.cursor += 1;
        if playback.cursor >= sequence.0.len() {
            playback.playing = false;
            playback.cursor = 0;
        } else {
            playback.timer.reset();
            play_note_hz(&audio, sequence.0[playback.cursor].hz);
        }
    }
}

pub fn add_sequence_note(sequence: &mut Sequence, note: &Note, position: &FretPosition) {
    sequence.0.push(NotePlacement {
        name: note.name,
        hz: note.hz,
        octave: note.octave,
        fret: position.fret,
        string_index: position.string_index,
    });
}
