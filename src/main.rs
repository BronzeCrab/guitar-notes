use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::*;
use bevy_asset::RenderAssetUsages;
use rodio::BitDepth;
use rodio::source::DitherAlgorithm;
use rodio::source::{SineWave, Source};
use std::thread;
use std::time::Duration;

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
}

struct Tunning {
    name: &'static str,
    notes: [Note; 6],
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: Some(Backends::VULKAN),
                    ..default()
                }
                .into(),
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

fn get_note_hz_in_4_octave(half_tones_from_a_4: f32) -> f32 {
    440.0 * 2_f32.powf(half_tones_from_a_4 / 12.0)
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
                hz: get_note_hz_in_4_octave(-5.0) / 4.0,
                octave: 2,
            },
            Note {
                name: "A",
                hz: get_note_hz_in_4_octave(0.0) / 4.0,
                octave: 2,
            },
            Note {
                name: "D",
                hz: get_note_hz_in_4_octave(-7.0) / 2.0,
                octave: 3,
            },
            Note {
                name: "G",
                hz: get_note_hz_in_4_octave(-2.0) / 2.0,
                octave: 3,
            },
            Note {
                name: "B",
                hz: get_note_hz_in_4_octave(2.0) / 2.0,
                octave: 3,
            },
            Note {
                name: "E",
                hz: get_note_hz_in_4_octave(-5.0) / 1.0,
                octave: 4,
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
            font_size: FONT_SIZE,
            ..default()
        },
        // Set the justification of the Text
        TextLayout::new_with_justify(Justify::Right),
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
                font_size: FONT_SIZE,
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
        for _i in 0..12 {
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

    // Рисуем названия нот
    let mut note_ind: usize = 5;
    let mut x_of_note_name: f32 = line_start_x;
    let y_of_note_name: f32 = 0.0;
    let mut octave: i8 = 2;
    // это для открытой E2 - басса (6я струна)
    let mut half_notes_from_a_4: f32 = -5.0;

    for i in 0..12 {
        if note_ind == NOTES.len() {
            note_ind = 0;
        }

        let note: &'static str = NOTES[note_ind];

        if i == 0 {
            x_of_note_name += GAP;
            half_notes_from_a_4 += 1.0;
        } else if note == "C" || note == "F" {
            x_of_note_name += GAP;
            half_notes_from_a_4 += 1.0;
            if note == "C" {
                octave += 1;
            }
        } else {
            x_of_note_name += 2.0 * GAP;
            half_notes_from_a_4 += 2.0;
        }

        let note_hz: f32 = get_note_hz_in_4_octave(half_notes_from_a_4) / 4.0;

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
                },
                Pickable::default(),
            ))
            .with_child((
                Text2d::new(note),
                TextFont {
                    font_size: FONT_SIZE,
                    ..default()
                },
                TextColor(Color::WHITE),
                Visibility::Visible,
            ))
            .observe(on_note_click);

        note_ind += 1;
    }

    // Рисуем контуры гитары
    for i in 0..1 {
        commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(window_width - 2.0 * GAP, 3.0))),
            MeshMaterial2d(materials.add(ColorMaterial::from(GUITAR_OUTLINE_COLOR))),
            Transform::from_xyz(0.0, -0.5 * GAP, 0.0),
        ));
    }
}

fn on_note_click(
    click: On<Pointer<Click>>,
    note_name_rect_entity_q: Query<&Note>,
    mut pitch_assets: ResMut<Assets<Pitch>>,
    mut commands: Commands,
) {
    let event: &Pointer<Click> = On::event(&click);
    let entity: Entity = event.event_target();
    let anote: &Note = note_name_rect_entity_q.get(entity).unwrap();
    println!(
        "Click on note, name: {:?}, hz: {:?}, octave: {:?}",
        anote.name, anote.hz, anote.octave
    );

    let hz_value: f32 = anote.hz; // переменная, которую нужно передать

    // Запускаем новый поток
    let _ = thread::spawn(move || {
        let handle: rodio::MixerDeviceSink =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        rodio::Player::connect_new(&handle.mixer());
        // Generate sine wave.
        let wave = SineWave::new(hz_value)
            // .amplify(0.2)
            .take_duration(Duration::from_secs(3))
            .fade_in(Duration::from_secs(1))
            .fade_out(Duration::from_secs(1));

        let dithered = wave.dither(BitDepth::new(16).unwrap(), DitherAlgorithm::TPDF);

        handle.mixer().add(dithered);

        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        std::thread::sleep(Duration::from_secs(3));
    });

    // let dur: Duration = Duration::new(3, 0);

    // commands.spawn((
    //     AudioPlayer(pitch_assets.add(Pitch::new(anote.hz, dur))),
    //     PlaybackSettings {
    //         mode: PlaybackMode::Once,
    //         volume: Volume::Linear(1.0).fade_towards(Volume::Linear(0.0), 0.1),
    //         speed: 1.0,
    //         paused: false,
    //         muted: false,
    //         spatial: false,
    //         spatial_scale: None,
    //         start_position: None,
    //         duration: Some(dur),
    //     },
    // ));

    // audio
    //     .play(pitch_assets.add(Pitch::new(anote.hz, dur)))
    //     // The first 0.5 seconds will not be looped and are the "intro"
    //     .loop_from(0.5)
    //     // Fade-in with a dynamic easing
    //     .fade_in(AudioTween::new(
    //         Duration::from_secs(2),
    //         AudioEasing::OutPowi(2),
    //     ))
    //     // Only play on our right ear
    //     .with_panning(1.0)
    //     // Increase playback rate by 50% (this also increases the pitch)
    //     .with_playback_rate(1.5)
    //     // Play at lower volume (-10dB)
    //     .with_volume(-10.)
    //     // play the track reversed
    //     .reverse();
}
