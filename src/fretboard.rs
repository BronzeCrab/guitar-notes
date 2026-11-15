use crate::constants::{
    AMOUNT_OF_FRETS, COLORS, GAP, GREY, GUITAR_OUTLINE_COLOR, MAX_FRETBOARD_CELL_PX,
    MAX_PORTRAIT_FRET_STEP_PX, MAX_PORTRAIT_STRING_STEP_PX, MAX_SELECTED_NOTES,
    MIN_PORTRAIT_FRET_STEP_PX, MIN_PORTRAIT_STRING_STEP_PX, PLAYING_TINT_COLOR,
    SELECTION_OUTLINE_COLOR, SELECTION_OUTLINE_THICKNESS,
};
use crate::sequence::{
    AppMode, CurrentMode, SelectedOrder, SelectionCounter, Sequence, SequenceToken,
    add_sequence_note, spawn_mode_buttons,
};
use crate::tuning::{CurrentTuning, Note, Tuning, color_index_for_note, note_hz, tuning, tunings};
use crate::ui::{
    ChordInfoText, PageScroll, PageScrollThumb, PageScrollTrack, PowerChordPopup, TuningMenuLabel,
    TuningMenuPanel, TuningOption, spawn_chord_controls, spawn_power_chord_popup,
    spawn_tuning_dropdown,
};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{FocusPolicy, Pressed};
use guitar_notes::music::NoteName;

#[derive(Component)]
pub struct OpenStringLabel;

#[derive(Component)]
pub struct FretNote;

/// Letter label inside a note dot; its font size follows the dot size.
#[derive(Component)]
pub struct NoteLabel;

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
    pub portrait: Option<bool>,
    pub dirty: bool,
}

impl Default for FretboardLayoutState {
    fn default() -> Self {
        Self {
            zone_size: None,
            portrait: None,
            dirty: true,
        }
    }
}

pub fn setup(mut commands: Commands, current_tuning: Res<CurrentTuning>) {
    commands.spawn(Camera2d);

    let active_tuning: &Tuning = tuning(current_tuning.index);

    spawn_power_chord_popup(&mut commands);

    // The whole app is a flex column: an auto-sized control bar on top and a
    // growing scroll area below it. Nothing is pinned to hard-coded offsets.
    let shell = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();

    let header = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                row_gap: Val::Px(4.0),
                padding: UiRect::axes(Val::Vw(1.0), Val::Px(4.0)),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(shell).add_child(header);
    commands.entity(header).with_children(|bar| {
        spawn_mode_buttons(bar);
        spawn_tuning_dropdown(bar, current_tuning.index);
    });

    let main = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                min_height: Val::Px(0.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .id();
    commands.entity(shell).add_child(main);

    let ui_root = commands
        .spawn((
            PageScroll,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                scrollbar_width: 0.0,
                ..default()
            },
            ScrollPosition::default(),
            Hovered::default(),
            FocusPolicy::Block,
        ))
        .id();
    commands.entity(main).add_child(ui_root);

    let zone_entity = commands
        .spawn((
            FretboardZone,
            Node {
                flex_grow: 1.0,
                flex_shrink: 0.0,
                // A soft floor keeps the neck visible when the panel grows,
                // without pushing the panel below the fold on short screens.
                min_height: Val::Vh(40.0),
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
            spawn_tuning_labels_and_notes(content, active_tuning);
        });
    });

    commands.entity(ui_root).with_children(spawn_chord_controls);

    // Overlay belongs to the scroll area, so it never covers the header.
    commands.entity(main).with_children(|overlay| {
        overlay
            .spawn((
                PageScrollTrack,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Px(12.0),
                    ..default()
                },
                FocusPolicy::Pass,
                Visibility::Hidden,
            ))
            .with_children(|track| {
                track.spawn((
                    PageScrollThumb,
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(2.0),
                        top: Val::Px(0.0),
                        width: Val::Px(8.0),
                        height: Val::Px(24.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.4)),
                    Visibility::Hidden,
                ));
            });
    });
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
    for f in 1..=AMOUNT_OF_FRETS + 1 {
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

pub fn spawn_tuning_labels_and_notes(parent: &mut ChildSpawnerCommands, active_tuning: &Tuning) {
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
) {
    let mut entity = parent.spawn((
        FretNote,
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
            NoteLabel,
            Text::new(note.name.as_str()),
            TextFont {
                font_size: FontSize::Rem(0.8),
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
type ZoneNodeQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static ComputedNode,
        &'static ComputedUiRenderTargetInfo,
        &'static mut Node,
    ),
    (
        With<FretboardZone>,
        Without<FretboardContent>,
        Without<StringLine>,
        Without<FretLine>,
        Without<FretNote>,
    ),
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

/// Number of grid steps along the neck: open-string area (2), one per fret,
/// and a trailing margin.
const FRET_UNITS: f32 = AMOUNT_OF_FRETS as f32 + 3.0;

struct FretboardGeometry {
    portrait: bool,
    offset_x: f32,
    offset_y: f32,
    width: f32,
    height: f32,
    /// Distance between adjacent frets (x in landscape, y in portrait).
    fret_step: f32,
    /// Distance between adjacent strings (y in landscape, x in portrait).
    string_step: f32,
    note_size: f32,
}

impl FretboardGeometry {
    fn compute(size: Vec2, num_strings: usize) -> Self {
        let strings = num_strings as f32;
        // On tall screens the neck is rotated: frets stack downwards and the
        // strings spread across the width, so the fretboard fills a phone
        // screen instead of shrinking to a thin strip.
        let portrait = size.y > size.x * 1.05;

        if portrait {
            let margin_x = 4.0;
            let margin_y = 6.0;

            let string_step = ((size.x - margin_x * 2.0) / (strings + 1.0))
                .clamp(MIN_PORTRAIT_STRING_STEP_PX, MAX_PORTRAIT_STRING_STEP_PX);
            let grid_w = string_step * (strings + 1.0);

            let available_h = (size.y - margin_y * 2.0).max(1.0);
            let fret_step = (available_h / FRET_UNITS)
                .clamp(MIN_PORTRAIT_FRET_STEP_PX, MAX_PORTRAIT_FRET_STEP_PX);
            let grid_h = fret_step * FRET_UNITS;
            let extra_y = (available_h - grid_h).max(0.0);

            Self {
                portrait,
                offset_x: ((size.x - grid_w) * 0.5).max(margin_x),
                offset_y: margin_y + extra_y * 0.5,
                width: grid_w,
                height: grid_h,
                fret_step,
                string_step,
                note_size: (fret_step.min(string_step) * 0.62).clamp(16.0, 34.0),
            }
        } else {
            let inset = (size.min_element() * 0.06).clamp(16.0, 64.0);
            let inner = (size - Vec2::splat(inset * 2.0)).max(Vec2::splat(1.0));

            // Square cells so the fretboard keeps its proportions and never
            // stretches to fill the whole zone; the grid is centered in `inner`.
            let cell = (inner.x / FRET_UNITS)
                .min(inner.y / (strings + 1.0))
                .clamp(8.0, MAX_FRETBOARD_CELL_PX);
            let grid_w = cell * FRET_UNITS;
            let grid_h = cell * (strings + 1.0);

            Self {
                portrait,
                offset_x: inset + (inner.x - grid_w) * 0.5,
                offset_y: inset + (inner.y - grid_h) * 0.5,
                width: grid_w,
                height: grid_h,
                fret_step: cell,
                string_step: cell,
                note_size: (cell * 0.5).max(10.0),
            }
        }
    }
}

/// Recompute string/fret/note pixel positions from the current zone size.
/// Runs after UI layout; the zone's size only depends on flex (root + panel)
/// plus the explicit height set here when the portrait neck does not fit,
/// so the computed size converges after one extra layout pass.
pub fn layout_fretboard(
    mut zones: ZoneNodeQuery,
    mut contents: ContentNodeQuery,
    mut string_lines: StringLineNodeQuery,
    mut fret_lines: FretLineNodeQuery,
    mut notes: NoteNodeQuery,
    mut labels: Query<&mut TextFont, With<NoteLabel>>,
    mut state: ResMut<FretboardLayoutState>,
) {
    let Ok((zone, target, mut zone_node)) = zones.single_mut() else {
        return;
    };
    let size = zone.size() / target.scale_factor();
    if size.x <= 0.0 || size.y <= 0.0 {
        return;
    }

    let num_strings = notes
        .iter()
        .map(|(pos, _, _)| pos.string_index as usize + 1)
        .max()
        .unwrap_or(1);
    let strings = num_strings as f32;
    let geometry = FretboardGeometry::compute(size, num_strings);
    let portrait = geometry.portrait;

    if !state.dirty && state.zone_size == Some(size) && state.portrait == Some(portrait) {
        return;
    }
    state.zone_size = Some(size);
    state.portrait = Some(portrait);
    state.dirty = false;

    // A portrait neck can be taller than the visible zone; let the zone grow
    // and the page scroll instead of shrinking the frets further.
    let needed_h = geometry.offset_y + geometry.height + 6.0;
    if portrait && needed_h > size.y + 0.5 {
        let needed = needed_h;
        if zone_node.height != Val::Px(needed) {
            zone_node.height = Val::Px(needed);
        }
        if zone_node.flex_grow != 0.0 {
            zone_node.flex_grow = 0.0;
        }
    } else if zone_node.height != Val::Auto || zone_node.flex_grow != 1.0 {
        zone_node.height = Val::Auto;
        zone_node.flex_grow = 1.0;
    }

    if let Ok(mut content) = contents.single_mut() {
        content.left = Val::Px(geometry.offset_x);
        content.top = Val::Px(geometry.offset_y);
        content.width = Val::Px(geometry.width);
        content.height = Val::Px(geometry.height);
    }

    for (line, mut node) in &mut string_lines {
        if portrait {
            let x = geometry.string_step * (line.index as f32 + 1.0) - 0.5;
            node.left = Val::Px(x);
            node.top = Val::Px(0.0);
            node.width = Val::Px(1.0);
            node.height = Val::Percent(100.0);
        } else {
            let y = geometry.string_step * (strings - line.index as f32) - 1.0;
            node.left = Val::Px(0.0);
            node.top = Val::Px(y);
            node.width = Val::Percent(100.0);
            node.height = Val::Px(1.0);
        }
    }

    for (fret, mut node) in &mut fret_lines {
        if portrait {
            let y = geometry.fret_step * (fret.index as f32 + 1.0) - 0.5;
            node.left = Val::Px(0.0);
            node.top = Val::Px(y);
            node.width = Val::Percent(100.0);
            node.height = Val::Px(1.0);
        } else {
            let x = geometry.fret_step * (fret.index as f32 + 1.0) - 1.0;
            node.left = Val::Px(x);
            node.top = Val::Px(0.0);
            node.width = Val::Px(1.0);
            node.height = Val::Percent(100.0);
        }
    }

    for (position, open, mut node) in &mut notes {
        let along_fret = if open.is_some() {
            1.0
        } else {
            position.fret as f32 + 1.5
        };
        let along_string = position.string_index as f32 + 1.0;
        let (x, y) = if portrait {
            (
                geometry.string_step * along_string,
                geometry.fret_step * along_fret,
            )
        } else {
            (
                geometry.fret_step * along_fret,
                geometry.string_step * (strings + 1.0 - along_string),
            )
        };
        let note_size = geometry.note_size;
        node.left = Val::Px(x - note_size * 0.5);
        node.top = Val::Px(y - note_size * 0.5);
        node.width = Val::Px(note_size);
        node.height = Val::Px(note_size);
    }

    // Keep the letters inside the dots at any screen size.
    let label_size = (geometry.note_size * 0.72).clamp(9.0, 20.0);
    for mut font in &mut labels {
        font.font_size = FontSize::Px(label_size);
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
            &Pressed,
            Entity,
            &Note,
            &FretPosition,
            Option<&SelectedNote>,
        ),
        (Changed<Pressed>, With<FretNote>),
    >,
    mut commands: Commands,
    selected_count_q: Query<&SelectedNote>,
    mode: Res<CurrentMode>,
    mut sequence: ResMut<Sequence>,
    mut counter: ResMut<SelectionCounter>,
) {
    for (_pressed, entity, note, position, selected_marker) in &interactions {
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
    option_clicks: Query<(&Pressed, &TuningOption), Changed<Pressed>>,
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
    mut sequence: ResMut<Sequence>,
    mut sequence_token: ResMut<SequenceToken>,
    mut layout_state: ResMut<FretboardLayoutState>,
) {
    for (_pressed, option) in &option_clicks {
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

        let Ok(content) = contents.single() else {
            continue;
        };
        commands.entity(content).with_children(|content| {
            spawn_fretboard_lines(content, active_tuning.notes.len());
            spawn_tuning_labels_and_notes(content, active_tuning);
        });
        layout_state.dirty = true;

        for mut text in &mut texts.p1() {
            *text = Text::new("Select up to 6 notes, then Play or Explain.");
        }

        sequence.0.clear();
        sequence_token.0 += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phone_portrait_fills_the_width() {
        let geometry = FretboardGeometry::compute(Vec2::new(390.0, 660.0), 6);
        assert!(geometry.portrait);
        assert!(
            geometry.width > 350.0,
            "neck width {} leaves the phone half empty",
            geometry.width
        );
        assert!(
            geometry.note_size >= 16.0,
            "note dot {} is too small to read",
            geometry.note_size
        );
        assert!(
            geometry.offset_y + geometry.height <= 660.5,
            "all frets should fit a typical phone"
        );
    }

    #[test]
    fn desktop_stays_horizontal_and_capped() {
        let geometry = FretboardGeometry::compute(Vec2::new(1440.0, 800.0), 6);
        assert!(!geometry.portrait);
        assert_eq!(geometry.fret_step, MAX_FRETBOARD_CELL_PX);
        assert!(geometry.note_size >= 24.0);
    }

    #[test]
    fn phone_landscape_stays_horizontal() {
        let geometry = FretboardGeometry::compute(Vec2::new(844.0, 390.0), 6);
        assert!(!geometry.portrait);
    }
}
