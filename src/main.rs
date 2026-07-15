use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use rodio::mixer::Mixer;
use rodio::source::{SineWave, Source};
use rodio::{DeviceSinkBuilder, MixerDeviceSink};
use std::time::Duration;

/// Keeps the OS audio stream alive for the app lifetime (`cpal::Stream` is `!Send`).
#[allow(dead_code)]
struct AudioSinkKeepAlive(MixerDeviceSink);

#[derive(Resource, Clone)]
struct NoteAudio {
    mixer: Mixer,
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

#[derive(Component)]
struct Note {
    name: &'static str,
    hz: f32,
    octave: i8,
    half_tones_from_a_4: f32,
}

struct Tunning {
    name: &'static str,
    notes: [Note; 6],
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
        .add_plugins((DefaultPlugins, MeshPickingPlugin))
        .add_systems(Startup, setup)
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    commands.spawn(Camera2d);

    let tunning: Tunning = Tunning {
        name: "standard",
        notes: [
            Note {
                name: "E",
                half_tones_from_a_4: -5.0,
                octave: 2,
                hz: get_note_hz_in_4_octave(-5.0) / 4.0,
            },
            Note {
                name: "A",
                half_tones_from_a_4: 0.0,
                octave: 2,
                hz: get_note_hz_in_4_octave(0.0) / 4.0,
            },
            Note {
                name: "D",
                half_tones_from_a_4: -7.0,
                octave: 3,
                hz: get_note_hz_in_4_octave(-7.0) / 2.0,
            },
            Note {
                name: "G",
                half_tones_from_a_4: -2.0,
                octave: 3,
                hz: get_note_hz_in_4_octave(-2.0) / 2.0,
            },
            Note {
                name: "B",
                half_tones_from_a_4: 2.0,
                octave: 3,
                hz: get_note_hz_in_4_octave(2.0) / 2.0,
            },
            Note {
                name: "E",
                half_tones_from_a_4: -5.0,
                octave: 4,
                hz: get_note_hz_in_4_octave(-5.0) / 1.0,
            },
        ],
    };

    // Text with one section
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Guitar notes"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: FontSize::Px(FONT_SIZE),
            ..default()
        },
        // Set the justification of the Text
        TextLayout::justify(Justify::Right),
        // Set the style of the Node itself.
        Node {
            justify_self: JustifySelf::Center,
            ..default()
        },
        Label,
    ));

    let window_width: f32 = window.width();
    let _window_height: f32 = window.height();

    let line_start_x: f32 = -window_width / 2.0 + GAP;
    let line_end_x: f32 = window_width / 2.0 - GAP;

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

        // Draw the NOTE name of the open string
        commands.spawn((
            Text2d::new(tunning.notes[i].name),
            TextFont {
                font_size: FontSize::Px(FONT_SIZE),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(line_start_x - GAP / 2.0, y_of_line, 0.0),
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

    // Натуральные ноты по кругу: F, G, A, B, C, D, E, F, ...
    // идём по ладам; конец ладов → следующая струна
    let mut string_i: usize = 0;
    let mut open: &Note = &tunning.notes[string_i];
    let mut y_of_note_name: f32 = string_i as f32 * GAP;
    let mut divisor: f32 = 2_f32.powi((4 - open.octave) as i32);
    let mut half_tones_from_a_4: f32 = open.half_tones_from_a_4;
    let mut octave: i8 = open.octave;
    // первая нота на 6-й струне — F (следующая натуральная после открытой E)
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
            // конец ладов на этой струне → следующая струна
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

    // Рисуем контуры гитары
    for i in 0..2 {
        let mut x: Option<f32> = None;
        let mut y: Option<f32> = None;
        if i == 0 {
            x = Some(0.0);
            y = Some(-0.5 * GAP);
        } else if i == 1 {
            x = Some(0.0);
            y = Some(last_str_y.unwrap() + 0.5 * GAP);
        }
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(window_width - 2.0 * GAP, 3.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from(GUITAR_OUTLINE_COLOR))),
            Transform::from_xyz(x.unwrap(), y.unwrap(), 0.0),
        ));
    }
}

fn on_note_click(
    click: On<Pointer<Click>>,
    note_name_rect_entity_q: Query<&Note>,
    audio: Res<NoteAudio>,
) {
    let event: &Pointer<Click> = On::event(&click);
    let entity: Entity = event.event_target();
    let anote: &Note = note_name_rect_entity_q.get(entity).unwrap();
    println!(
        "Click on note, name: {:?}, hz: {:?}, octave: {:?}",
        anote.name, anote.hz, anote.octave
    );

    let note_duration: Duration = Duration::from_millis(900);
    let wave = SineWave::new(anote.hz)
        .amplify(0.25)
        .take_duration(note_duration)
        .fade_in(Duration::from_millis(5))
        .fade_out(note_duration);

    audio.mixer.add(wave);
}
