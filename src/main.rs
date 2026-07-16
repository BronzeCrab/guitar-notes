use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use guitar_notes::music::{
    detect_power_chord, format_note_lines, NotePlacement,
};
use rodio::mixer::Mixer;
use rodio::source::{SineWave, Source};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};
use std::sync::OnceLock;
use std::time::Duration;

/// Keeps the OS audio stream alive for the app lifetime (`cpal::Stream` is `!Send`).
#[allow(dead_code)]
struct AudioSinkKeepAlive(MixerDeviceSink);

#[derive(Resource, Clone)]
struct NoteAudio {
    mixer: Mixer,
}

#[derive(Resource)]
struct CurrentTuning {
    index: usize,
}

#[derive(Resource)]
struct FretboardLayout {
    line_start_x: f32,
}

const NOTES: [&'static str; 7] = ["A", "B", "C", "D", "E", "F", "G"];
const GAP: f32 = 50.0;
const CUSTOM_WHITE: Color = Color::srgba(1.0, 1.0, 1.0, 0.5);
const GREY: Color = Color::srgba(0.6, 0.6, 0.6, 1.0);
const AMOUNT_OF_FRETS: u8 = 22;
const FONT_SIZE: f32 = 22.0;
const RECT_SIZE: f32 = 30.0;
const GUITAR_OUTLINE_COLOR: Color = Color::srgba(0.15, 0.15, 0.15, 1.0);

const COLORS: [Color; 7] = [
    Color::srgba(1.0, 0.0, 0.0, 1.0), // Красный (Red)
    Color::srgba(1.0, 0.5, 0.0, 1.0), // Оранжевый (Orange)
    Color::srgba(0.8, 0.7, 0.0, 1.0), // Жёлтый (Yellow)
    Color::srgba(0.0, 1.0, 0.0, 1.0), // Зелёный (Green)
    Color::srgba(0.0, 0.5, 1.0, 1.0), // Голубой (Blue-green/Cyan)
    Color::srgba(0.0, 0.0, 1.0, 1.0), // Синий (Blue)
    Color::srgba(0.5, 0.0, 1.0, 1.0), // Фиолетовый (Purple/Violet)
];

const MAX_SELECTED_NOTES: usize = 6;
const NOTE_PLAY_DURATION_MS: u64 = 900;

#[derive(Component, Clone)]
struct Note {
    name: &'static str,
    hz: f32,
    octave: i8,
    half_tones_from_a_4: f32,
}

#[derive(Clone)]
struct Tunning {
    name: &'static str,
    notes: [Note; 6],
}

#[derive(Component)]
struct OpenStringLabel;

#[derive(Component)]
struct FretNote;

#[derive(Component, Clone, Copy)]
struct FretPosition {
    /// 0 = lowest pitch string in tuning array (low E in Standard)
    string_index: u8,
    fret: u8,
}

#[derive(Component)]
struct SelectedNote;

#[derive(Component)]
struct NoteVisual {
    color_index: usize,
}

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct ExplainButton;

#[derive(Component)]
struct ClearButton;

#[derive(Component)]
struct ChordInfoText;

#[derive(Component)]
struct PowerChordPopup;

#[derive(Component)]
struct PowerChordPopupText;

#[derive(Component)]
struct DismissPowerChordPopup;

#[derive(Component)]
struct TuningMenuButton;

#[derive(Component)]
struct TuningMenuPanel;

#[derive(Component)]
struct TuningMenuLabel;

#[derive(Component)]
struct TuningOption {
    index: usize,
}

fn main() {
    let mut sink: MixerDeviceSink =
        DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    sink.log_on_drop(false);
    let note_audio: NoteAudio = NoteAudio {
        mixer: sink.mixer().clone(),
    };

    App::new()
        .insert_non_send(AudioSinkKeepAlive(sink))
        .insert_resource(note_audio)
        .insert_resource(CurrentTuning { index: 0 })
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_tuning_menu,
                apply_tuning_selection,
                play_selected_notes,
                explain_selection,
                clear_selection,
                dismiss_power_chord_popup,
            ),
        )
        .run();
}

fn get_note_hz_in_4_octave(half_tones_from_a_4: f32) -> f32 {
    440.0 * 2_f32.powf(half_tones_from_a_4 / 12.0)
}

fn note_index(name: &str) -> usize {
    NOTES
        .iter()
        .position(|&n| n == name)
        .expect("note name must be in NOTES")
}

fn open_note(name: &'static str, half_tones_from_a_4: f32, octave: i8) -> Note {
    let divisor: f32 = 2_f32.powi((4 - octave) as i32);
    Note {
        name,
        half_tones_from_a_4,
        octave,
        hz: get_note_hz_in_4_octave(half_tones_from_a_4) / divisor,
    }
}

fn tunings() -> &'static [Tunning] {
    static TUNINGS: OnceLock<Vec<Tunning>> = OnceLock::new();
    TUNINGS.get_or_init(|| {
        vec![
            Tunning {
                name: "Standard",
                notes: [
                    open_note("E", -5.0, 2),
                    open_note("A", 0.0, 2),
                    open_note("D", -7.0, 3),
                    open_note("G", -2.0, 3),
                    open_note("B", 2.0, 3),
                    open_note("E", -5.0, 4),
                ],
            },
            Tunning {
                name: "Drop D",
                notes: [
                    open_note("D", -7.0, 2),
                    open_note("A", 0.0, 2),
                    open_note("D", -7.0, 3),
                    open_note("G", -2.0, 3),
                    open_note("B", 2.0, 3),
                    open_note("E", -5.0, 4),
                ],
            },
        ]
    })
}

fn tuning(index: usize) -> &'static Tunning {
    &tunings()[index]
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
    current_tuning: Res<CurrentTuning>,
) {
    commands.spawn(Camera2d);

    let tunning: &Tunning = tuning(current_tuning.index);

    commands.spawn((
        Text::new("Guitar notes"),
        TextFont {
            font_size: FontSize::Px(FONT_SIZE),
            ..default()
        },
        TextLayout::justify(Justify::Right),
        Node {
            justify_self: JustifySelf::Center,
            ..default()
        },
        Label,
    ));

    spawn_tuning_dropdown(&mut commands, current_tuning.index);
    spawn_chord_controls(&mut commands);
    spawn_power_chord_popup(&mut commands);

    let window_width: f32 = window.width();
    let line_start_x: f32 = -window_width / 2.0 + GAP;
    let line_end_x: f32 = window_width / 2.0 - GAP;
    commands.insert_resource(FretboardLayout { line_start_x });

    let mut last_str_y: Option<f32> = None;
    // Рисуем горизонтальные линии - "струны":
    for i in 0..tunning.notes.len() {
        let y_of_line: f32 = i as f32 * GAP;
        if i == tunning.notes.len() - 1 {
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
        ));
    }

    // Рисуем вертикальные линии - "разделители ладов":
    let mut vert_line_start_x: f32 = line_start_x + GAP / 2.0;
    if let Some(value) = last_str_y {
        let vert_line_start_y: f32 = -GAP / 2.0;
        let vert_line_end_y: f32 = value + GAP / 2.0;
        for _i in 0..AMOUNT_OF_FRETS {
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
            ));
            vert_line_start_x += GAP;
        }
    }

    spawn_tuning_labels_and_notes(
        &mut commands,
        &mut meshes,
        &mut materials,
        tunning,
        line_start_x,
    );

    // Рисуем контуры гитары (рамка: верх, низ, лево, право)
    let outline_left: f32 = line_start_x;
    let outline_right: f32 = line_end_x;
    let outline_bottom: f32 = -0.5 * GAP;
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
            Transform::from_xyz(0.0, y, 0.0),
        ));
    }
    for x in [outline_left, outline_right] {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(OUTLINE_THICKNESS, outline_height))),
            MeshMaterial2d(outline_material.clone()),
            Transform::from_xyz(x, outline_mid_y, 0.0),
        ));
    }
}

fn spawn_tuning_dropdown(commands: &mut Commands, current_index: usize) {
    let current_name: &'static str = tuning(current_index).name;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                top: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ZIndex(10),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Button,
                    TuningMenuButton,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        min_width: Val::Px(140.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
                    BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new(format!("{current_name} v")),
                        TextFont {
                            font_size: FontSize::Px(FONT_SIZE),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TuningMenuLabel,
                    ));
                });

            parent
                .spawn((
                    TuningMenuPanel,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        min_width: Val::Px(140.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                    Visibility::Hidden,
                ))
                .with_children(|panel| {
                    for (index, tunning) in tunings().iter().enumerate() {
                        panel
                            .spawn((
                                Button,
                                TuningOption { index },
                                Node {
                                    padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                                    justify_content: JustifyContent::FlexStart,
                                    align_items: AlignItems::Center,
                                    width: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.22, 0.22, 0.28)),
                            ))
                            .with_children(|opt| {
                                opt.spawn((
                                    Text::new(tunning.name),
                                    TextFont {
                                        font_size: FontSize::Px(FONT_SIZE),
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}

fn spawn_tuning_labels_and_notes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    tunning: &Tunning,
    line_start_x: f32,
) {
    for (i, open) in tunning.notes.iter().enumerate() {
        let y_of_line: f32 = i as f32 * GAP;
        commands.spawn((
            Text2d::new(open.name),
            TextFont {
                font_size: FontSize::Px(FONT_SIZE),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(line_start_x - GAP / 2.0, y_of_line, 0.0),
            OpenStringLabel,
        ));
    }

    let mut string_i: usize = 0;
    let mut open: &Note = &tunning.notes[string_i];
    let mut y_of_note_name: f32 = string_i as f32 * GAP;
    let mut divisor: f32 = 2_f32.powi((4 - open.octave) as i32);
    let mut half_tones_from_a_4: f32 = open.half_tones_from_a_4;
    let mut octave: i8 = open.octave;
    let mut note_ind: usize = note_index(open.name) + 1;
    if note_ind == NOTES.len() {
        note_ind = 0;
    }
    let mut x_of_note_name: f32 = line_start_x;

    loop {
        let note: &'static str = NOTES[note_ind];

        if note == "C" || note == "F" {
            x_of_note_name += GAP;
            half_tones_from_a_4 += 1.0;
            if note == "C" {
                octave += 1;
            }
        } else {
            x_of_note_name += 2.0 * GAP;
            half_tones_from_a_4 += 2.0;
        }

        let fret: f32 = (x_of_note_name - line_start_x) / GAP;
        if fret > AMOUNT_OF_FRETS as f32 {
            string_i += 1;
            if string_i == tunning.notes.len() {
                break;
            }
            open = &tunning.notes[string_i];
            y_of_note_name = string_i as f32 * GAP;
            divisor = 2_f32.powi((4 - open.octave) as i32);
            half_tones_from_a_4 = open.half_tones_from_a_4;
            octave = open.octave;
            note_ind = note_index(open.name) + 1;
            if note_ind == NOTES.len() {
                note_ind = 0;
            }
            x_of_note_name = line_start_x;
            continue;
        }

        let note_hz: f32 = get_note_hz_in_4_octave(half_tones_from_a_4) / divisor;

        commands
            .spawn((
                Mesh2d(meshes.add(Rectangle::new(RECT_SIZE, RECT_SIZE))),
                MeshMaterial2d(materials.add(ColorMaterial::from(COLORS[note_ind]))),
                Transform::from_xyz(x_of_note_name, y_of_note_name, 0.0),
                Visibility::Visible,
                Note {
                    name: note,
                    hz: note_hz,
                    octave: octave,
                    half_tones_from_a_4: half_tones_from_a_4,
                },
                NoteVisual {
                    color_index: note_ind,
                },
                FretPosition {
                    string_index: string_i as u8,
                    fret: fret.round() as u8,
                },
                FretNote,
                Pickable::default(),
            ))
            .with_child((
                Text2d::new(note),
                TextFont {
                    font_size: FontSize::Px(FONT_SIZE),
                    ..default()
                },
                TextColor(Color::WHITE),
                Visibility::Visible,
            ))
            .observe(on_note_click);

        note_ind += 1;
        if note_ind == NOTES.len() {
            note_ind = 0;
        }
    }
}

fn toggle_tuning_menu(
    interactions: Query<&Interaction, (Changed<Interaction>, With<TuningMenuButton>)>,
    mut panels: Query<&mut Visibility, With<TuningMenuPanel>>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for mut visibility in &mut panels {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }
}

fn apply_tuning_selection(
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
    open_labels: Query<Entity, With<OpenStringLabel>>,
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
        let tunning: &Tunning = tuning(option.index);

        for mut text in &mut texts.p0() {
            *text = Text::new(format!("{} v", tunning.name));
        }
        for mut visibility in &mut visibilities.p0() {
            *visibility = Visibility::Hidden;
        }
        for mut visibility in &mut visibilities.p1() {
            *visibility = Visibility::Hidden;
        }

        for entity in &open_labels {
            commands.entity(entity).despawn();
        }
        for entity in &fret_notes {
            commands.entity(entity).despawn();
        }

        spawn_tuning_labels_and_notes(
            &mut commands,
            &mut meshes,
            &mut materials,
            tunning,
            layout.line_start_x,
        );

        for mut text in &mut texts.p1() {
            *text = Text::new("Select up to 6 notes, then Play or Explain.");
        }
    }
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

fn spawn_power_chord_popup(commands: &mut Commands) {
    commands
        .spawn((
            PowerChordPopup,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ZIndex(100),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        max_width: Val::Px(420.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.16, 0.16, 0.2)),
                    BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
                ))
                .with_children(|panel| {
                    panel.spawn((
                        Text::new("Power chord"),
                        TextFont {
                            font_size: FontSize::Px(FONT_SIZE),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    panel.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.85, 0.85, 0.9)),
                        PowerChordPopupText,
                    ));

                    spawn_action_button(panel, "OK", DismissPowerChordPopup);
                });
        });
}

fn selected_note_color(color_index: usize) -> Color {
    let base = COLORS[color_index].to_srgba();
    Color::srgba(
        base.red * 0.45 + 0.55,
        base.green * 0.45 + 0.55,
        base.blue * 0.45 + 0.55,
        1.0,
    )
}

fn spawn_chord_controls(commands: &mut Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(16.0),
                bottom: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                max_width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.85)),
            ZIndex(10),
        ))
        .with_children(|parent| {
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                },))
                .with_children(|row| {
                    spawn_action_button(row, "Play", PlayButton);
                    spawn_action_button(row, "Explain", ExplainButton);
                    spawn_action_button(row, "Clear", ClearButton);
                });

            parent.spawn((
                Text::new("Select up to 6 notes, then Play or Explain."),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.85, 0.9)),
                Node {
                    max_width: Val::Px(296.0),
                    ..default()
                },
                ChordInfoText,
            ));
        });
}

fn spawn_action_button(parent: &mut ChildSpawnerCommands, label: &str, marker: impl Bundle) {
    parent
        .spawn((
            Button,
            marker,
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                min_width: Val::Px(72.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.25)),
            BorderColor::all(Color::srgb(0.45, 0.45, 0.5)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(FONT_SIZE),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn play_note_hz(audio: &NoteAudio, hz: f32) {
    let note_duration: Duration = Duration::from_millis(NOTE_PLAY_DURATION_MS);
    let wave = SineWave::new(hz)
        .amplify(0.2)
        .take_duration(note_duration)
        .fade_in(Duration::from_millis(5))
        .fade_out(note_duration);
    audio.mixer.add(wave);
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
) {
    let event: &Pointer<Click> = On::event(&click);
    let entity: Entity = event.event_target();
    let Ok((visual, material, selected_marker)) = notes_q.get(entity) else {
        return;
    };

    if selected_marker.is_some() {
        commands.entity(entity).remove::<SelectedNote>();
        if let Some(mut mat) = materials.get_mut(material.id()) {
            mat.color = COLORS[visual.color_index];
        }
        return;
    }

    let selected_count = notes_q.iter().filter(|(_, _, s)| s.is_some()).count();
    if selected_count >= MAX_SELECTED_NOTES {
        return;
    }

    let material_id = material.id();
    commands.entity(entity).insert(SelectedNote);
    if let Some(mut mat) = materials.get_mut(material_id) {
        mat.color = selected_note_color(visual.color_index);
    }
}

fn play_selected_notes(
    interactions: Query<&Interaction, (Changed<Interaction>, With<PlayButton>)>,
    selected: Query<(&Note, &FretPosition), With<SelectedNote>>,
    audio: Res<NoteAudio>,
    mut popups: Query<&mut Visibility, With<PowerChordPopup>>,
    mut popup_texts: Query<&mut Text, With<PowerChordPopupText>>,
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

        let names: Vec<&str> = entries.iter().map(|(note, _)| note.name).collect();
        let mut placements = note_placements(&entries);
        let note_lines = format_note_lines(&mut placements);
        let body = if let Some(info) = detect_power_chord(&names) {
            format!(
                "You played {} (power chord).\n\nWhy: root + fifth (no third).\n\nNotes:\n{}",
                info.title, note_lines
            )
        } else {
            format!("Played notes:\n{note_lines}")
        };

        for mut text in &mut popup_texts {
            *text = Text::new(body.clone());
        }
        for mut visibility in &mut popups {
            *visibility = Visibility::Visible;
        }
    }
}

fn dismiss_power_chord_popup(
    interactions: Query<&Interaction, (Changed<Interaction>, With<DismissPowerChordPopup>)>,
    mut popups: Query<&mut Visibility, With<PowerChordPopup>>,
) {
    for interaction in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        for mut visibility in &mut popups {
            *visibility = Visibility::Hidden;
        }
    }
}

fn explain_selection(
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
            "Select notes first (root + fifth for a power chord).".to_string()
        } else {
            let names: Vec<&str> = entries.iter().map(|(note, _)| note.name).collect();
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

fn clear_selection(
    interactions: Query<&Interaction, (Changed<Interaction>, With<ClearButton>)>,
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    selected: Query<(Entity, &NoteVisual, &MeshMaterial2d<ColorMaterial>), With<SelectedNote>>,
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
