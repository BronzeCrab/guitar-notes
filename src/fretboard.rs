use crate::camera::MainCamera;
use crate::constants::{
    AMOUNT_OF_FRETS, CHORD_PANEL_CLEARANCE_Y, COLORS, FONT_SIZE, GAP, GREY, GUITAR_OUTLINE_COLOR,
    MAX_SELECTED_NOTES, OPEN_STRING_OFFSET_X, RECT_SIZE, SELECTED_NOTE_COLOR,
    SELECTED_NOTE_TEXT_COLOR, WORLD_HEIGHT, WORLD_WIDTH, chord_info_font_size, ui_font_size,
};
use crate::tuning::{
    CurrentTuning, Note, Tuning, color_index_for_note, get_note_hz_in_4_octave, tuning, tunings,
};
use crate::ui::{
    ChordInfoText, PowerChordPopup, TuningMenuLabel, TuningMenuPanel, TuningOption,
    spawn_chord_controls, spawn_power_chord_popup, spawn_tuning_dropdown,
};
use bevy::asset::RenderAssetUsages;
use bevy::camera::ScalingMode;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use guitar_notes::music::NOTE_NAMES;

#[derive(Resource)]
pub struct FretboardLayout {
    pub line_start_x: f32,
    /// World Y of string index 0 (low E in Standard); higher indices go up by GAP.
    pub string_y0: f32,
}

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

#[derive(Component)]
pub struct NoteVisual {
    pub color_index: usize,
}

pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
    current_tuning: Res<CurrentTuning>,
) {
    commands.spawn((
        Camera2d,
        MainCamera,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: WORLD_WIDTH,
                min_height: WORLD_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));

    let active_tuning: &Tuning = tuning(current_tuning.index);
    let ui_font = ui_font_size(window.width());
    let info_font = chord_info_font_size(window.width());

    spawn_tuning_dropdown(&mut commands, current_tuning.index, ui_font);
    spawn_chord_controls(&mut commands, ui_font, info_font);
    spawn_power_chord_popup(&mut commands, ui_font);

    // World coords (not window pixels): leave 1.5*GAP on the left so open-string
    // labels sit beside the nut, inside the camera.
    let line_start_x: f32 = -WORLD_WIDTH / 2.0 + GAP * 1.5;
    let line_end_x: f32 = line_start_x + GAP / 2.0 + AMOUNT_OF_FRETS as f32 * GAP;
    // Center the fretboard, then lift it so the bottom chord panel does not cover notes.
    let string_y0: f32 =
        -((active_tuning.notes.len() - 1) as f32 * GAP) * 0.5 + CHORD_PANEL_CLEARANCE_Y * 0.5;
    commands.insert_resource(FretboardLayout {
        line_start_x,
        string_y0,
    });

    let mut last_str_y: Option<f32> = None;
    for i in 0..active_tuning.notes.len() {
        let y_of_line: f32 = string_y0 + i as f32 * GAP;
        if i == active_tuning.notes.len() - 1 {
            last_str_y = Some(y_of_line);
        }
        let vertices: Vec<[f32; 3]> =
            vec![[line_start_x, y_of_line, 0.0], [line_end_x, y_of_line, 0.0]];
        let mut line_mesh: Mesh = Mesh::new(
            PrimitiveTopology::LineStrip,
            RenderAssetUsages::RENDER_WORLD,
        );
        line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        commands.spawn((
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(Color::WHITE)),
            Pickable::IGNORE,
        ));
    }

    let mut vert_line_start_x: f32 = line_start_x + GAP / 2.0;
    if let Some(value) = last_str_y {
        let vert_line_start_y: f32 = string_y0 - GAP / 2.0;
        let vert_line_end_y: f32 = value + GAP / 2.0;
        // One extra line closes the last fret on the right.
        for _i in 0..=AMOUNT_OF_FRETS {
            let vertices: Vec<[f32; 3]> = vec![
                [vert_line_start_x, vert_line_start_y, 0.0],
                [vert_line_start_x, vert_line_end_y, 0.0],
            ];
            let mut line_mesh: Mesh = Mesh::new(
                PrimitiveTopology::LineStrip,
                RenderAssetUsages::RENDER_WORLD,
            );
            line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
            commands.spawn((
                Mesh2d(meshes.add(line_mesh)),
                MeshMaterial2d(materials.add(GREY)),
                Pickable::IGNORE,
            ));
            vert_line_start_x += GAP;
        }
    }

    spawn_tuning_labels_and_notes(
        &mut commands,
        &mut meshes,
        &mut materials,
        active_tuning,
        line_start_x,
        string_y0,
    );

    let outline_left: f32 = line_start_x;
    let outline_right: f32 = line_end_x;
    let outline_bottom: f32 = string_y0 - 0.5 * GAP;
    let outline_top: f32 = last_str_y.unwrap() + 0.5 * GAP;
    let outline_width: f32 = outline_right - outline_left;
    let outline_height: f32 = outline_top - outline_bottom;
    let outline_mid_y: f32 = (outline_top + outline_bottom) * 0.5;
    const OUTLINE_THICKNESS: f32 = 3.0;
    let outline_material = materials.add(ColorMaterial::from(GUITAR_OUTLINE_COLOR));

    for y in [outline_bottom, outline_top] {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(outline_width, OUTLINE_THICKNESS))),
            MeshMaterial2d(outline_material.clone()),
            Transform::from_xyz((outline_left + outline_right) * 0.5, y, 0.0),
            Pickable::IGNORE,
        ));
    }
    for x in [outline_left, outline_right] {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(OUTLINE_THICKNESS, outline_height))),
            MeshMaterial2d(outline_material.clone()),
            Transform::from_xyz(x, outline_mid_y, 0.0),
            Pickable::IGNORE,
        ));
    }
}

pub fn spawn_tuning_labels_and_notes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    active_tuning: &Tuning,
    line_start_x: f32,
    string_y0: f32,
) {
    for (string_i, open) in active_tuning.notes.iter().enumerate() {
        let y = string_y0 + string_i as f32 * GAP;
        let color_index = color_index_for_note(open.name);
        spawn_clickable_note(
            commands,
            meshes,
            materials,
            open.clone(),
            color_index,
            line_start_x - OPEN_STRING_OFFSET_X,
            y,
            FretPosition {
                string_index: string_i as u8,
                fret: 0,
            },
            true,
        );

        let divisor: f32 = 2_f32.powi((4 - open.octave) as i32);
        let mut half_tones_from_a_4 = open.half_tones_from_a_4;
        let mut octave = open.octave;
        let mut note_ind = (color_index + 1) % NOTE_NAMES.len();
        let mut x = line_start_x;

        loop {
            let name = NOTE_NAMES[note_ind];
            if name == "C" || name == "F" {
                x += GAP;
                half_tones_from_a_4 += 1.0;
                if name == "C" {
                    octave += 1;
                }
            } else {
                x += 2.0 * GAP;
                half_tones_from_a_4 += 2.0;
            }

            let fret = ((x - line_start_x) / GAP).round() as u8;
            if fret > AMOUNT_OF_FRETS {
                break;
            }

            spawn_clickable_note(
                commands,
                meshes,
                materials,
                Note {
                    name,
                    hz: get_note_hz_in_4_octave(half_tones_from_a_4) / divisor,
                    octave,
                    half_tones_from_a_4,
                },
                note_ind,
                x,
                y,
                FretPosition {
                    string_index: string_i as u8,
                    fret,
                },
                false,
            );

            note_ind += 1;
            if note_ind == NOTE_NAMES.len() {
                note_ind = 0;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_clickable_note(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    note: Note,
    color_index: usize,
    x: f32,
    y: f32,
    position: FretPosition,
    open_string: bool,
) {
    let mut entity = commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(RECT_SIZE, RECT_SIZE))),
        MeshMaterial2d(materials.add(ColorMaterial::from(COLORS[color_index]))),
        Transform::from_xyz(x, y, 1.0),
        Visibility::Visible,
        note.clone(),
        NoteVisual { color_index },
        position,
        FretNote,
        Pickable::default(),
    ));
    if open_string {
        entity.insert(OpenStringLabel);
    }
    entity
        .with_child((
            Text2d::new(note.name),
            TextFont {
                font_size: FontSize::Px(FONT_SIZE),
                ..default()
            },
            TextColor(Color::WHITE),
            Visibility::Visible,
            Pickable::IGNORE,
        ))
        .observe(on_note_click);
}

pub fn set_note_label_color(
    entity: Entity,
    color: Color,
    children_q: &Query<&Children>,
    text_colors: &mut Query<&mut TextColor>,
) {
    let Ok(children) = children_q.get(entity) else {
        return;
    };
    for child in children.iter() {
        if let Ok(mut text_color) = text_colors.get_mut(child) {
            *text_color = TextColor(color);
        }
    }
}

fn on_note_click(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    notes_q: Query<(
        &NoteVisual,
        &MeshMaterial2d<ColorMaterial>,
        Option<&SelectedNote>,
    )>,
    parents: Query<&ChildOf>,
    children_q: Query<&Children>,
    mut text_colors: Query<&mut TextColor>,
) {
    let event: &Pointer<Click> = On::event(&click);
    let mut entity: Entity = event.event_target();
    // Clicks may hit the Text2d child; walk up to the note mesh entity.
    while notes_q.get(entity).is_err() {
        let Ok(child_of) = parents.get(entity) else {
            return;
        };
        entity = child_of.parent();
    }
    let Ok((visual, material, selected_marker)) = notes_q.get(entity) else {
        return;
    };

    if selected_marker.is_some() {
        commands.entity(entity).remove::<SelectedNote>();
        if let Some(mut mat) = materials.get_mut(material.id()) {
            mat.color = COLORS[visual.color_index];
        }
        set_note_label_color(entity, Color::WHITE, &children_q, &mut text_colors);
        return;
    }

    let selected_count = notes_q.iter().filter(|(_, _, s)| s.is_some()).count();
    if selected_count >= MAX_SELECTED_NOTES {
        return;
    }

    let material_id = material.id();
    commands.entity(entity).insert(SelectedNote);
    if let Some(mut mat) = materials.get_mut(material_id) {
        mat.color = SELECTED_NOTE_COLOR;
    }
    set_note_label_color(
        entity,
        SELECTED_NOTE_TEXT_COLOR,
        &children_q,
        &mut text_colors,
    );
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn apply_tuning_selection(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut current_tuning: ResMut<CurrentTuning>,
    layout: Res<FretboardLayout>,
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

        spawn_tuning_labels_and_notes(
            &mut commands,
            &mut meshes,
            &mut materials,
            active_tuning,
            layout.line_start_x,
            layout.string_y0,
        );

        for mut text in &mut texts.p1() {
            *text = Text::new("Select up to 6 notes, then Play or Explain.");
        }
    }
}
