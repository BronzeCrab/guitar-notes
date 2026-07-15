use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::*;
use rodio::source::{Dither, DitherAlgorithm, FadeIn, FadeOut, SineWave, Source, TakeDuration};
use rodio::{BitDepth, DeviceSinkBuilder, MixerDeviceSink};
use std::thread;
use std::time::Duration;

const NOTES: [&'static str; 7] = ["A", "B", "C", "D", "E", "F", "G"];
const GAP: f32 = 50.0;
const AMOUNT_OF_FRETS: u8 = 22;
const FONT_SIZE: f32 = 22.0;
const RECT_SIZE: f32 = 30.0;

const COLORS: [Color; 7] = [
    Color::srgba(1.0, 0.0, 0.0, 1.0),   // E - Red
    Color::srgba(1.0, 0.5, 0.0, 1.0),   // B - Orange
    Color::srgba(0.8, 0.7, 0.0, 1.0),   // G - Yellow
    Color::srgba(0.0, 1.0, 0.0, 1.0),   // D - Green
    Color::srgba(0.0, 0.5, 1.0, 1.0),   // A - Cyan
    Color::srgba(0.0, 0.0, 1.0, 1.0),   // E - Blue
    Color::srgba(0.5, 0.0, 1.0, 1.0),   // C - Purple (for open notes)
];

#[derive(Component, Clone, Copy)]
struct FretNote {
    string: usize,
    fret: u8,
    note_name: &'static str,
    hz: f32,
    octave: i8,
}

#[derive(Resource, Clone, Debug, PartialEq)]
enum Tuning {
    Standard,
    DropD,
    DropC,
    HalfStepDown,
    OpenD,
    OpenG,
}

impl Tuning {
    fn name(&self) -> &'static str {
        match self {
            Tuning::Standard => "Standard (E A D G B E)",
            Tuning::DropD => "Drop D (D A D G B E)",
            Tuning::DropC => "Drop C (C G C F A D)",
            Tuning::HalfStepDown => "Half Step Down (Eb Ab Db Gb Bb Eb)",
            Tuning::OpenD => "Open D (D A D F# A D)",
            Tuning::OpenG => "Open G (D G D G B D)",
        }
    }

    fn open_string_hz(&self, string_idx: usize) -> f32 {
        let standard = [82.41, 110.0, 146.83, 196.0, 246.94, 329.63]; // E2, A2, D3, G3, B3, E4
        match self {
            Tuning::Standard => standard[string_idx],
            Tuning::DropD => match string_idx {
                0 => 73.42, // D2
                _ => standard[string_idx],
            },
            Tuning::DropC => match string_idx {
                0 => 65.41, // C2
                1 => 98.0,  // G2
                2 => 130.81, // C3
                _ => standard[string_idx],
            },
            Tuning::HalfStepDown => standard[string_idx] / 2_f32.powf(1.0/12.0),
            Tuning::OpenD => match string_idx {
                0 => 73.42,   // D2
                1 => 110.0,   // A2
                2 => 73.42,   // D2
                3 => 92.50,   // F#2
                4 => 110.0,   // A2
                5 => 73.42,   // D3
            },
            Tuning::OpenG => match string_idx {
                0 => 73.42,   // D2
                1 => 196.0,   // G3
                2 => 73.42,   // D3
                3 => 196.0,   // G3
                4 => 246.94,  // B3
                5 => 392.0,   // E4
            },
        }
    }
}

#[derive(Component)]
struct GuitarNeck;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: Some(Backends::VULKAN),
                    ..default()
                }.into(),
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .init_state::<Tuning>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_key_input)
        .add_systems(OnEnter(Tuning), update_fretboard)
        .run()
}

fn get_note_name(half_tones_from_a4: f32, open_hz: f32) -> (&'static str, f32, i8) {
    let hz = 440.0 * 2_f32.powf(half_tones_from_a4 / 12.0);
    
    // Find note name from semitone offset
    let semitone_names: [(i32, &'static str); 12] = [
        (0, "A"), (1, "Bb/A#"), (2, "B"), (3, "C"), (4, "Db/C#"), (5, "D"),
        (6, "Eb/D#"), (7, "E"), (8, "F"), (9, "F#/Gb"), (10, "G"), (11, "G#/Ab")
    ];
    
    let octave = ((hz / 440.0).log2().round() as i32 + 4) as i8;
    
    // Simple mapping based on frequency ratio to open string
    let ratio = (hz / open_hz).log2() * 12.0;
    let semitone = (ratio.round() % 12 + 12) % 12;
    
    // Map to note index
    let note_idx = match semitone {
        0 => 0, 1 => 0, 2 => 1, 3 => 3, 4 => 3, 5 => 4,
        6 => 5, 7 => 6, 8 => 6, 9 => 6, 10 => 6, 11 => 6,
        _ => 0,
    };
    (NOTES[note_idx.min(6)], hz, octave)
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    commands.spawn(Camera2d);
    
    // Title
    commands.spawn((
        Text::new("Guitar Notes - Interactive Fretboard"),
    ));
    
    setup_neck(&mut commands, &mut meshes, &mut materials, &window);
    setup_notes(&mut commands, &mut meshes, &mut materials, &window, Tuning::Standard);
}

fn setup_neck(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    window: &Single<&Window>,
) {
    let window_width = window.width();
    let neck_width = window_width - GAP * 2.0;
    
    commands.spawn((
        GuitarNeck,
        Mesh2d(meshes.add(Rectangle::new(neck_width, GAP * 5.0 + 10.0))),
        MeshMaterial2d(materials.add(Color::srgba(0.1, 0.1, 0.1, 1.0))),
    ));
}

fn setup_notes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    window: &Single<&Window>,
    tuning: Tuning,
) {
    let window_width = window.width();
    let line_start_x = -window_width / 2.0 + GAP;
    let line_end_x = window_width / 2.0 - GAP;
    
    // Draw frets (22 total)
    for fret in 0..=AMOUNT_OF_FRETS {
        let x = line_start_x + fret as f32 * GAP * 2.0;
        let vertices: Vec<[f32; 3]> = (0..6).map(|i| {
            [x, i as f32 * GAP - GAP * 2.5, 0.0]
        }).collect();
        
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        commands.spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(Color::srgba(0.5, 0.5, 0.5, 0.7))),
        ));
    }
    
    // Draw strings (6)
    for string_idx in 0..6 {
        let y = string_idx as f32 * GAP - GAP * 2.5;
        let vertices: Vec<[f32; 3]> = vec![[line_start_x, y, 0.0], [line_end_x + GAP * 42.0, y, 0.0]];
        
        let mut mesh = Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::RENDER_WORLD);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        commands.spawn((
            Mesh2d(meshes.add(mesh)),
            MeshMaterial2d(materials.add(Color::WHITE)),
        ));
    }
    
    // Draw all notes for each string/fret
    for string_idx in 0..6 {
        let open_hz = tuning.open_string_hz(string_idx);
        for fret in 0..=AMOUNT_OF_FRETS {
            let (name, hz, octave) = get_note_at_fret(open_hz, fret);
            
            let x = line_start_x + fret as f32 * GAP * 2.0;
            let y = string_idx as f32 * GAP - GAP * 2.5;
            let note_idx = NOTES.iter().position(|&n| n == name).unwrap_or(0);
            
            commands.spawn((
                FretNote { string: string_idx, fret, note_name: name, hz, octave },
                Mesh2d(meshes.add(Rectangle::new(RECT_SIZE * 0.8, RECT_SIZE * 0.8))),
                MeshMaterial2d(materials.add(COLORS[note_idx])),
                Transform::from_xyz(x, y, 1.0),
                Visibility::Visible,
                Pickable::default(),
            )).observe(on_note_click);
        }
    }
}

fn get_note_at_fret(open_hz: f32, fret: u8) -> (&'static str, f32, i8) {
    let hz = open_hz * 2_f32.powf(fret as f32 / 12.0);
    
    // Find closest note name
    let note_names: [(f32, &'static str); 7] = [
        (261.63, "C"), (293.66, "D"), (329.63, "E"),
        (349.23, "F"), (392.00, "G"), (440.00, "A"), (493.88, "B"),
    ];
    
    let mut closest = ("E", hz);
    let mut min_dist = f32::MAX;
    
    for (ref_hz, name) in note_names {
        for octave_shift in [-3, -2, -1, 0, 1, 2, 3] {
            let target_hz = ref_hz * 2_f32.powf(octave_shift as f32);
            let dist = (hz - target_hz).abs();
            if dist < min_dist {
                min_dist = dist;
                closest = (*name, hz);
            }
        }
    }
    
    let octave = (hz / 440.0).log2().round() as i8;
    (closest.0, hz, octave)
}

fn on_note_click(click: On<Pointer<Click>>, query_notes: Query<&FretNote>) {
    let event = On::event(&click);
    let entity = event.event_target();
    if let Ok(note) = query_notes.get(entity) {
        println!("Note: {} ({}Hz, fret {})", note.note_name, note.hz, note.fret);
        
        let _ = thread::spawn(move || {
            if let Some(handle) = DeviceSinkBuilder::open_default_sink().ok() {
                let wave = SineWave::new(note.hz)
                    .take_duration(Duration::from_secs(2))
                    .fade_in(Duration::from_millis(200))
                    .fade_out(Duration::from_secs(1));
                
                let dithered = wave.dither(BitDepth::new(16).unwrap(), DitherAlgorithm::TPDF);
                handle.mixer().add(dithered);
                thread::sleep(Duration::from_secs(3));
            }
        });
    }
}

fn update_fretboard(
    trigger: Trigger<OnEnter<Tuning>>,
    tuning: Res<Tuning>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
    existing_notes: Query<Entity, With<FretNote>>,
) {
    // Despawn old notes
    for entity in existing_notes.iter() {
        commands.entity(entity).despawn();
    }
    
    // Spawn new notes with current tuning
    setup_notes(&mut commands, &mut meshes, &mut materials, &window, tuning.clone());
}

fn handle_key_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_tuning: ResMut<NextState<Tuning>>,
) {
    if keyboard_input.just_pressed(KeyCode::Digit1) {
        next_tuning.set(Tuning::Standard);
    }
    if keyboard_input.just_pressed(KeyCode::Digit2) {
        next_tuning.set(Tuning::DropD);
    }
    if keyboard_input.just_pressed(KeyCode::Digit3) {
        next_tuning.set(Tuning::DropC);
    }
    if keyboard_input.just_pressed(KeyCode::Digit4) {
        next_tuning.set(Tuning::HalfStepDown);
    }
    if keyboard_input.just_pressed(KeyCode::Digit5) {
        next_tuning.set(Tuning::OpenD);
    }
    if keyboard_input.just_pressed(KeyCode::Digit6) {
        next_tuning.set(Tuning::OpenG);
    }
}