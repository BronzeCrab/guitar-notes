use crate::constants::{
    AMOUNT_OF_FRETS, COLORS, GAP, GREY, GUITAR_OUTLINE_COLOR, MAX_FRETBOARD_CELL_PX,
    MAX_SELECTED_NOTES, PLAYING_TINT_COLOR, SELECTION_OUTLINE_COLOR, SELECTION_OUTLINE_THICKNESS,
    ui_font_size,
};
use crate::sequence::{
    AppMode, CurrentMode, SelectedOrder, SelectionCounter, Sequence, SequenceToken,
    add_sequence_note, spawn_mode_buttons,
};
use crate::tuning::{CurrentTuning, Note, Tuning, color_index_for_note, note_hz, tuning, tunings};
use crate::ui::{
    ChordInfoText, PowerChordPopup, TuningMenuLabel, TuningMenuPanel, TuningOption,
    spawn_chord_controls, spawn_power_chord_popup, spawn_tuning_dropdown,
};
use bevy::prelude::*;
use guitar_notes::music::NoteName;

#[derive(Component)]
pub struct OpenStringLabel;

#[derive(Component)]
pub struct FretNote;

#[derive(Component, Clone, Copy)]
pub struct FretPosition {
    /// 0 = lowest pitch string in tuning array (low E in Standard)
    pub string_index: u8,
    pub fret: u8,
}

#[derive(Component)]
pub struct SelectedNote;

/// White outline border around a selected note (child of the note node).
#[derive(Component)]
pub struct SelectionRing;

/// Semi-transparent cream overlay shown when a note is currently sounding.
#[derive(Component)]
pub struct PlayingTint;

#[derive(Component)]
pub struct NoteVisual {
    pub color_index: usize,
}

/// Flex zone that holds the fretboard. Grows/shrinks with the bottom panel.
#[derive(Component)]
pub struct FretboardZone;

/// Absolute container inside [`FretboardZone`]; children (strings, frets, notes)
/// are laid out relative to the zone's current `ComputedNode` size.
#[derive(Component)]
pub struct FretboardContent;

/// Horizontal string line; `index` is the string index in the tuning.
#[derive(Component)]
pub struct StringLine {
    pub index: u8,
}

/// Vertical fret line; `index * GAP` from the nut boundary (0 = nut).
#[derive(Component)]
pub struct FretLine {
    pub index: u8,
}

/// Remembers the zone pixel size so positions are only recomputed when it changes.
#[derive(Resource)]
pub struct FretboardLayoutState {
    pub zone_size: Option<Vec2>,
    pub dirty: bool,
}

impl Default for FretboardLayoutState {
    fn default() -> Self {
        Self {
            zone_size: None,
            dirty: true,
        }
    }
}

pub fn setup(mut commands: Commands, window: Single<&Window>, current_tuning: Res<CurrentTuning>) {
    commands.spawn(Camera2d);

    let active_tuning: &Tuning = tuning(current_tuning.index);
    let ui_font = ui_font_size(window.width());

    spawn_tuning_dropdown(&mut commands, current_tuning.index, ui_font);
    spawn_mode_buttons(&mut commands, ui_font);
    spawn_power_chord_popup(&mut commands, ui_font);

    let ui_root = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .id();

    let zone_entity = commands
        .spawn((
            FretboardZone,
            Node {
                flex_grow: 1.0,
                flex_shrink: 1.0,
                overflow: Overflow::clip(),
                ..default()
            },
        ))
        .id();
    commands.entity(ui_root).add_child(zone_entity);

    commands.entity(zone_entity).with_children(|zone| {
        zone.spawn((
            FretboardContent,
            Node {
                position_type: PositionType::Absolute,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BorderColor::all(GUITAR_OUTLINE_COLOR),
        ))
        .with_children(|content| {
            spawn_fretboard_lines(content, active_tuning.notes.len());
            spawn_tuning_labels_and_notes(content, active_tuning, ui_font);
        });
    });

    commands
        .entity(ui_root)
        .with_children(|root| spawn_chord_controls(root, ui_font, ui_font - 2.0));
}

/// Thin horizontal string lines and vertical fret lines inside the fretboard.
fn spawn_fretboard_lines(parent: &mut ChildSpawnerCommands, num_strings: usize) {
    for i in 0..num_strings {
        parent.spawn((
            StringLine { index: i as u8 },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Px(1.0),
                ..default()
            },
            BackgroundColor(GREY),
            Pickable::IGNORE,
        ));
    }
    for f in 0..=AMOUNT_OF_FRETS + 1 {
        parent.spawn((
            FretLine { index: f },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Px(1.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(GREY),
            Pickable::IGNORE,
        ));
    }
}

pub fn spawn_tuning_labels_and_notes(
    parent: &mut ChildSpawnerCommands,
    active_tuning: &Tuning,
    ui_font: f32,
) {
    for (string_i, open) in active_tuning.notes.iter().enumerate() {
        let color_index = color_index_for_note(open.name);
        spawn_clickable_note(
            parent,
            open.clone(),
            color_index,
            FretPosition {
                string_index: string_i as u8,
                fret: 0,
            },
            true,
            ui_font,
        );

        let mut semitones_from_a_4 = open.semitones_from_a_4;
        let mut octave = open.octave;
        let mut note_ind = (color_index + 1) % NoteName::ALL.len();
        let mut x: f32 = 0.0;

        loop {
            let name = NoteName::ALL[note_ind];
            if name == NoteName::C || name == NoteName::F {
                x += GAP;
                semitones_from_a_4 += 1.0;
                if name == NoteName::C {
                    octave += 1;
                }
            } else {
                x += 2.0 * GAP;
                semitones_from_a_4 += 2.0;
            }

            let fret = (x / GAP).round() as u8;
            if fret > AMOUNT_OF_FRETS {
                break;
            }

            spawn_clickable_note(
                parent,
                Note {
                    name,
                    hz: note_hz(semitones_from_a_4),
                    octave,
                    semitones_from_a_4,
                },
                note_ind,
                FretPosition {
                    string_index: string_i as u8,
                    fret,
                },
                false,
                ui_font,
            );

            note_ind += 1;
            if note_ind == NoteName::ALL.len() {
                note_ind = 0;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_clickable_note(
    parent: &mut ChildSpawnerCommands,
    note: Note,
    color_index: usize,
    position: FretPosition,
    open_string: bool,
    ui_font: f32,
) {
    let mut entity = parent.spawn((
        Button,
        FretNote,
        NoteVisual { color_index },
        note.clone(),
        position,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(COLORS[color_index]),
    ));
    if open_string {
        entity.insert(OpenStringLabel);
    }
    entity.with_children(|note_children| {
        note_children.spawn((
            SelectionRing,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                border: UiRect::all(Val::Px(SELECTION_OUTLINE_THICKNESS)),
                ..default()
            },
            BorderColor::all(SELECTION_OUTLINE_COLOR),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));

        note_children.spawn((
            PlayingTint,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                right: Val::Px(0.0),
                bottom: Val::Px(0.0),
                ..default()
            },
            BackgroundColor(PLAYING_TINT_COLOR),
            Visibility::Hidden,
            Pickable::IGNORE,
        ));

        note_children.spawn((
            Text::new(note.name.as_str()),
            TextFont {
                font_size: FontSize::Px(ui_font),
                ..default()
            },
            TextColor(Color::WHITE),
            Pickable::IGNORE,
        ));
    });
}

type StringLineNodeQuery<'w, 's> =
    Query<'w, 's, (&'static StringLine, &'static mut Node), (Without<FretLine>, Without<FretNote>)>;
type FretLineNodeQuery<'w, 's> =
    Query<'w, 's, (&'static FretLine, &'static mut Node), (Without<StringLine>, Without<FretNote>)>;
type NoteNodeQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static FretPosition,
        Option<&'static OpenStringLabel>,
        &'static mut Node,
    ),
    (With<FretNote>, Without<StringLine>, Without<FretLine>),
>;
type ContentNodeQuery<'w, 's> = Query<
    'w,
    's,
    &'static mut Node,
    (
        With<FretboardContent>,
        Without<StringLine>,
        Without<FretLine>,
        Without<FretNote>,
    ),
>;

/// Recompute string/fret/note pixel positions from the current zone size.
/// Runs after UI layout; the zone's size only depends on flex (root + panel),
/// so this never feeds back into the layout.
pub fn layout_fretboard(
    zones: Query<(&ComputedNode, &ComputedUiRenderTargetInfo), With<FretboardZone>>,
    mut contents: ContentNodeQuery,
    mut string_lines: StringLineNodeQuery,
    mut fret_lines: FretLineNodeQuery,
    mut notes: NoteNodeQuery,
    mut state: ResMut<FretboardLayoutState>,
) {
    let Ok((zone, target)) = zones.single() else {
        return;
    };
    let size = zone.size() / target.scale_factor();
    if size.x <= 0.0 || size.y <= 0.0 {
        return;
    }
    if !state.dirty && state.zone_size == Some(size) {
        return;
    }
    state.zone_size = Some(size);
    state.dirty = false;

    let num_strings = notes
        .iter()
        .map(|(pos, _, _)| pos.string_index as usize + 1)
        .max()
        .unwrap_or(1);

    let inset = (size.min_element() * 0.06).clamp(16.0, 64.0);
    let inner = size - Vec2::splat(inset * 2.0);

    // Square cells so the fretboard keeps its proportions and never stretches
    // to fill the whole zone; the grid is centered inside `inner`.
    let cell = (inner.x / (AMOUNT_OF_FRETS as f32 + 3.0))
        .min(inner.y / (num_strings as f32 + 1.0))
        .clamp(8.0, MAX_FRETBOARD_CELL_PX);
    let grid_w = cell * (AMOUNT_OF_FRETS as f32 + 3.0);
    let grid_h = cell * (num_strings as f32 + 1.0);
    let offset_x = inset + (inner.x - grid_w) * 0.5;
    let offset_y = inset + (inner.y - grid_h) * 0.5;
    let note_size = (cell * 0.5).max(10.0);

    if let Ok(mut content) = contents.single_mut() {
        content.left = Val::Px(offset_x);
        content.top = Val::Px(offset_y);
        content.width = Val::Px(grid_w);
        content.height = Val::Px(grid_h);
    }

    for (line, mut node) in &mut string_lines {
        let y = cell * (num_strings as f32 - line.index as f32);
        node.top = Val::Px(y - 1.0);
    }

    for (fret, mut node) in &mut fret_lines {
        let x = cell * (fret.index as f32 + 1.0);
        node.left = Val::Px(x - 1.0);
    }

    for (position, open, mut node) in &mut notes {
        let x = cell
            * if open.is_some() {
                1.0
            } else {
                position.fret as f32 + 1.5
            };
        let y = cell * (num_strings as f32 - position.string_index as f32);
        node.left = Val::Px(x - note_size * 0.5);
        node.top = Val::Px(y - note_size * 0.5);
        node.width = Val::Px(note_size);
        node.height = Val::Px(note_size);
    }
}

/// Set the text label, selection outline and playing tint on a note's child entities.
#[allow(clippy::too_many_arguments)]
pub fn set_note_visual(
    entity: Entity,
    label: &str,
    label_color: Color,
    ring_visible: bool,
    playing: bool,
    children_q: &Query<&Children>,
    labels: &mut Query<(&mut Text, &mut TextColor)>,
    rings: &mut Query<&mut Visibility, (With<SelectionRing>, Without<PlayingTint>)>,
    tints: &mut Query<&mut Visibility, (With<PlayingTint>, Without<SelectionRing>)>,
) {
    let Ok(children) = children_q.get(entity) else {
        return;
    };
    for child in children.iter() {
        if let Ok((mut text, mut text_color)) = labels.get_mut(child) {
            *text = Text::new(label);
            *text_color = TextColor(label_color);
        }
        if let Ok(mut visibility) = rings.get_mut(child) {
            *visibility = if ring_visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
        if let Ok(mut visibility) = tints.get_mut(child) {
            *visibility = if ring_visible && playing {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn handle_note_click(
    interactions: Query<
        (
            &Interaction,
            Entity,
            &Note,
            &FretPosition,
            Option<&SelectedNote>,
        ),
        (Changed<Interaction>, With<FretNote>, With<Button>),
    >,
    mut commands: Commands,
    selected_count_q: Query<&SelectedNote>,
    mode: Res<CurrentMode>,
    mut sequence: ResMut<Sequence>,
    mut counter: ResMut<SelectionCounter>,
) {
    for (interaction, entity, note, position, selected_marker) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }

        if mode.0 == AppMode::Sequence {
            add_sequence_note(&mut sequence, note, position);
            continue;
        }

        if selected_marker.is_some() {
            commands
                .entity(entity)
                .remove::<(SelectedNote, SelectedOrder)>();
            continue;
        }

        if selected_count_q.iter().count() >= MAX_SELECTED_NOTES {
            continue;
        }

        commands
            .entity(entity)
            .insert((SelectedNote, SelectedOrder { order: counter.0 }));
        counter.0 += 1;
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn apply_tuning_selection(
    mut commands: Commands,
    mut current_tuning: ResMut<CurrentTuning>,
    option_clicks: Query<(&Interaction, &TuningOption), Changed<Interaction>>,
    mut visibilities: ParamSet<(
        Query<&mut Visibility, With<TuningMenuPanel>>,
        Query<&mut Visibility, With<PowerChordPopup>>,
    )>,
    mut texts: ParamSet<(
        Query<&mut Text, With<TuningMenuLabel>>,
        Query<&mut Text, With<ChordInfoText>>,
    )>,
    fret_notes: Query<Entity, With<FretNote>>,
    string_lines: Query<Entity, With<StringLine>>,
    fret_lines: Query<Entity, With<FretLine>>,
    contents: Query<Entity, With<FretboardContent>>,
    windows: Single<&Window>,
    mut sequence: ResMut<Sequence>,
    mut sequence_token: ResMut<SequenceToken>,
    mut layout_state: ResMut<FretboardLayoutState>,
) {
    for (interaction, option) in &option_clicks {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if option.index == current_tuning.index {
            for mut visibility in &mut visibilities.p0() {
                *visibility = Visibility::Hidden;
            }
            continue;
        }
        if option.index >= tunings().len() {
            continue;
        }

        current_tuning.index = option.index;
        let active_tuning: &Tuning = tuning(option.index);

        for mut text in &mut texts.p0() {
            *text = Text::new(format!("{} v", active_tuning.name));
        }
        for mut visibility in &mut visibilities.p0() {
            *visibility = Visibility::Hidden;
        }
        for mut visibility in &mut visibilities.p1() {
            *visibility = Visibility::Hidden;
        }

        for entity in &fret_notes {
            commands.entity(entity).despawn();
        }
        for entity in &string_lines {
            commands.entity(entity).despawn();
        }
        for entity in &fret_lines {
            commands.entity(entity).despawn();
        }

        let font = ui_font_size(windows.width());
        let Ok(content) = contents.single() else {
            continue;
        };
        commands.entity(content).with_children(|content| {
            spawn_fretboard_lines(content, active_tuning.notes.len());
            spawn_tuning_labels_and_notes(content, active_tuning, font);
        });
        layout_state.dirty = true;

        for mut text in &mut texts.p1() {
            *text = Text::new("Select up to 6 notes, then Play or Explain.");
        }

        sequence.0.clear();
        sequence_token.0 += 1;
    }
}
