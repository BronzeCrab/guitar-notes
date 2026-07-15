use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::*;
use rand::Rng;
use wasm_bindgen::prelude::*;

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
    Color::srgba(0.5, 0.0, 1.0, 1.0),   // C - Purple
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

#[derive(Resource, Clone, Copy, Debug)]
enum LearningMode {
    None,
    GuessNote,
    EarTraining,
}

#[derive(Resource)]
struct GameData {
    current_mode: LearningMode,
    target_note: Option<(usize, u8)>,
    score: u32,
    attempts: u32,
    correct_count: u32,
    streak: u32,
    best_streak: u32,
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
        let standard = [82.41, 110.0, 146.83, 196.0, 246.94, 329.63];
        match self {
            Tuning::Standard => standard[string_idx],
            Tuning::DropD => match string_idx {
                0 => 73.42,
                _ => standard[string_idx],
            },
            Tuning::DropC => match string_idx {
                0 => 65.41,
                1 => 98.0,
                2 => 130.81,
                _ => standard[string_idx],
            },
            Tuning::HalfStepDown => standard[string_idx] / 2_f32.powf(1.0/12.0),
            Tuning::OpenD => match string_idx {
                0 => 73.42,
                1 => 110.0,
                2 => 73.42,
                3 => 92.50,
                4 => 110.0,
                5 => 73.42,
            },
            Tuning::OpenG => match string_idx {
                0 => 73.42,
                1 => 196.0,
                2 => 73.42,
                3 => 196.0,
                4 => 246.94,
                5 => 392.0,
            },
        }
    }
}

#[derive(Component)]
struct GuitarNeck;
#[derive(Component)]
struct LearnText;
#[derive(Component)]
struct ScoreText;
#[derive(Component)]
struct StreakText;
#[derive(Component)]
struct FeedbackText;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: WgpuSettings {
                    backends: Some(Backends::VULKAN | Backends::BROWSER_WEBGPU),
                    ..default()
                }.into(),
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .init_state::<Tuning>()
        .init_resource::<GameData>()
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_key_input, update_learning_ui, update_score_ui))
        .add_systems(OnEnter(Tuning), update_fretboard)
        .run();
}

fn get_note_name(half_tones_from_a4: f32, open_hz: f32) -> (&'static str, f32, i8) {
    let hz = 440.0 * 2_f32.powf(half_tones_from_a4 / 12.0);
    
    let semitone_names: [(i32, &'static str); 12] = [
        (0, "A"), (1, "Bb/A#"), (2, "B"), (3, "C"), (4, "Db/C#"), (5, "D"),
        (6, "Eb/D#"), (7, "E"), (8, "F"), (9, "F#/Gb"), (10, "G"), (11, "G#/Ab")
    ];
    
    let octave = ((hz / 440.0).log2().round() as i32 + 4) as i8;
    
    let ratio = (hz / open_hz).log2() * 12.0;
    let semitone = (ratio.round() % 12 + 12) % 12;
    
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
    
    commands.spawn((
        Text::new("Guitar Notes - Interactive Fretboard"),
        TextFont { font_size: 28.0, ..default() },
        Transform::from_xyz(0.0, window.height() / 2.0 - 50.0, 0.0),
    ));
    
    setup_neck(&mut commands, &mut meshes, &mut materials, &window);
    setup_notes(&mut commands, &mut meshes, &mut materials, &window, Tuning::Standard);
    setup_ui_controls(&mut commands, &window);
    setup_score_ui(&mut commands, &window);
    
    let mut game_data = GameData {
        current_mode: LearningMode::None,
        target_note: None,
        score: 0,
        attempts: 0,
        correct_count: 0,
        streak: 0,
        best_streak: 0,
    };
    
    spawn_target_note(&mut commands, &mut game_data);
    
    commands.insert_resource(game_data);
}

fn spawn_target_note(commands: &mut Commands, game_data: &mut GameData) {
    let mut rng = rand::thread_rng();
    let string = rng.gen_range(0..6);
    let fret = rng.gen_range(0..=AMOUNT_OF_FRETS);
    game_data.target_note = Some((string, fret));
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

fn on_note_click(
    click: On<Pointer<Click>>,
    query_notes: Query<&FretNote>,
    mut game_data: ResMut<GameData>,
    mut commands: Commands,
    mut score_query: Query<&mut Text, With<ScoreText>>,
    mut streak_query: Query<&mut Text, With<StreakText>>,
    mut feedback_query: Query<&mut Text, With<FeedbackText>>,
) {
    let event = On::event(&click);
    let entity = event.event_target();
    
    if let Ok(note) = query_notes.get(entity) {
        if game_data.current_mode == LearningMode::GuessNote {
            if let Some((target_string, target_fret)) = game_data.target_note {
                game_data.attempts += 1;
                
                if note.string == target_string && note.fret == target_fret {
                    game_data.score += 100 + (game_data.streak * 10) as u32;
                    game_data.correct_count += 1;
                    game_data.streak += 1;
                    game_data.best_streak = game_data.best_streak.max(game_data.streak);
                    
                    let feedback = format!("✓ Правильно! +{}", 100 + (game_data.streak * 10) as u32);
                    update_feedback(&mut commands, &mut feedback_query, &feedback);
                    
                    spawn_target_note(&mut commands, &mut game_data);
                } else {
                    game_data.streak = 0;
                    let feedback = "✗ Неверно, попробуй ещё!";
                    update_feedback(&mut commands, &mut feedback_query, feedback);
                }
                
                update_score_ui_text(&mut score_query, game_data.score);
                update_streak_ui_text(&mut streak_query, game_data.streak);
            }
        }
    }
}

fn update_feedback(
    commands: &mut Commands,
    feedback_query: &mut Query<&mut Text, With<FeedbackText>>,
    text: &str,
) {
    if let Ok(mut feedback) = feedback_query.get_single_mut() {
        *feedback = Text::new(text);
    } else {
        commands.spawn((
            Text::new(text),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::GREEN),
            FeedbackText,
            Transform::from_xyz(0.0, -100.0, 0.0),
        ));
    }
}

fn handle_key_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_tuning: ResMut<NextState<Tuning>>,
    mut game_data: ResMut<GameData>,
    mut commands: Commands,
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

    if keyboard_input.just_pressed(KeyCode::KeyL) {
        game_data.current_mode = LearningMode::GuessNote;
        spawn_target_note(&mut commands, &mut game_data);
    }
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        game_data.current_mode = LearningMode::EarTraining;
        spawn_target_note(&mut commands, &mut game_data);
    }
    if keyboard_input.just_pressed(KeyCode::Space) {
        game_data.current_mode = LearningMode::None;
    }
}

fn update_learning_ui(
    game_data: Res<GameData>,
    mut text_query: Query<&mut Text, With<LearnText>>,
) {
    let mode_text = match game_data.current_mode {
        LearningMode::None => "Режим: Свободное исследование",
        LearningMode::GuessNote => "Режим: Угадай ноту (L)",
        LearningMode::EarTraining => "Режим: Тренировка слуха (E)",
    };
    
    if let Ok(mut text) = text_query.get_single_mut() {
        *text = Text::new(mode_text);
    }
}

fn setup_ui_controls(
    commands: &mut Commands,
    window: &Single<&Window>,
) {
    let start_y = -window.height() / 2.0 + 20.0;
    
    commands.spawn((
        Text::new("Controls: 1-6 (tunings), L (guess), E (ear), Space (reset)"),
        TextFont { font_size: 16.0, ..default() },
        Transform::from_xyz(0.0, start_y, 0.0),
    ));
}

fn setup_score_ui(
    commands: &mut Commands,
    window: &Single<&Window>,
) {
    let start_y = window.height() / 2.0 - 100.0;
    
    commands.spawn((
        Text::new("Score: 0"),
        ScoreText,
        TextFont { font_size: 24.0, ..default() },
        Transform::from_xyz(-200.0, start_y, 0.0),
    ));
    
    commands.spawn((
        Text::new("Streak: 0"),
        StreakText,
        TextFont { font_size: 24.0, ..default() },
        Transform::from_xyz(100.0, start_y, 0.0),
    ));
}

fn update_score_ui_text(
    score_query: &mut Query<&mut Text, With<ScoreText>>,
    score: u32,
) {
    if let Ok(mut text) = score_query.get_single_mut() {
        *text = Text::new(format!("Score: {}", score));
    }
}

fn update_streak_ui_text(
    streak_query: &mut Query<&mut Text, With<StreakText>>,
    streak: u32,
) {
    if let Ok(mut text) = streak_query.get_single_mut() {
        *text = Text::new(format!("Streak: {}", streak));
    }
}

fn update_score_ui(
    game_data: Res<GameData>,
    mut score_query: Query<&mut Text, With<ScoreText>>,
    mut streak_query: Query<&mut Text, With<StreakText>>,
) {
    update_score_ui_text(&mut score_query, game_data.score);
    update_streak_ui_text(&mut streak_query, game_data.streak);
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
    for entity in existing_notes.iter() {
        commands.entity(entity).despawn();
    }
    
    setup_notes(&mut commands, &mut meshes, &mut materials, &window, tuning.clone());
}