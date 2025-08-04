use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::settings::*;

const NOTES: [&'static str; 7] = ["A", "B", "C", "D", "E", "F", "G"];
const GAP: f32 = 50.0;
const CUSTOM_WHITE: Color = Color::srgba(1.0, 1.0, 1.0, 0.5);
const GREY: Color = Color::srgba(0.6, 0.6, 0.6, 1.0);
const AMOUNT_OF_FRETS: u8 = 22;
const FONT_SIZE: f32 = 22.0;
const RECT_SIZE: f32 = 30.0;

const COLORS: [Color; 7] = [
    Color::srgba(1.0, 0.0, 0.0, 1.0),      // Красный (Red)
    Color::srgba(1.0, 0.5, 0.0, 1.0),      // Оранжевый (Orange)
    Color::srgba(0.8, 0.7, 0.0, 1.0),      // Жёлтый (Yellow)
    Color::srgba(0.0, 1.0, 0.0, 1.0),      // Зелёный (Green)
    Color::srgba(0.0, 0.5, 1.0, 1.0),      // Голубой (Blue-green/Cyan)
    Color::srgba(0.0, 0.0, 1.0, 1.0),      // Синий (Blue)
    Color::srgba(0.5, 0.0, 1.0, 1.0),      // Фиолетовый (Purple/Violet)
];

struct Tunning {
    name: &'static str,
    notes: [&'static str; 6],
}

#[derive(Component)]
pub struct NoteNameRectLabel {
    note_name: &'static str,
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

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Single<&Window>,
) {
    commands.spawn(Camera2d);

    let tunning: Tunning = Tunning {
        name: "standard",
        notes: ["E", "A", "D", "G", "B", "E"],
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
        TextLayout::new_with_justify(JustifyText::Right),
        // Set the style of the Node itself.
        Node {
            justify_self: JustifySelf::Center,
            ..default()
        },
        Label,
    ));

    let window_width: f32 = window.width();
    let window_height: f32 = window.height();

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
            bevy::render::mesh::PrimitiveTopology::LineStrip,
            RenderAssetUsages::RENDER_WORLD,
        );
        line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        commands.spawn((
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(Color::WHITE)),
        ));

        // Draw the NOTE name of the open string
        commands.spawn((
            Text2d::new(tunning.notes[i]),
            TextFont {
                font_size: FONT_SIZE,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(line_start_x - GAP / 2.0, y_of_line, 0.0),
        ));
    }

    // Рисуем вертикальные линии - "разделители ладов":
    if let Some(value) = last_str_y {
        let line_end_y: f32 = value + GAP / 2.0;
        for i in 0..AMOUNT_OF_FRETS {
            let line_start_y: f32 = -GAP / 2.0;
            let x_of_line: f32 = line_start_x + i as f32 * GAP;
            let vertices: Vec<[f32; 3]> =
                vec![[x_of_line, line_start_y, 0.0], [x_of_line, line_end_y, 0.0]];
            let mut line_mesh: Mesh = Mesh::new(
                bevy::render::mesh::PrimitiveTopology::LineStrip,
                RenderAssetUsages::RENDER_WORLD,
            );
            line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
            commands.spawn((
                Mesh2d(meshes.add(line_mesh)),
                MeshMaterial2d(materials.add(GREY)),
            ));
        }
    }

    let mut note_ind: usize = 5;
    let mut x_of_note_name: f32 = line_start_x;
    let y_of_note_name: f32 = -GAP;

    for i in 0..AMOUNT_OF_FRETS - 1 {
        if note_ind == 7 {
            note_ind = 0;
        }

        let note: &'static str = NOTES[note_ind];

        if i == 0 {
            x_of_note_name += 0.5 * GAP;
        } else if note == "C" || note == "F" {
            x_of_note_name += GAP;
        } else {
            x_of_note_name += 2.0 * GAP;
        }

        commands
            .spawn((
                Mesh2d(meshes.add(Rectangle::new(RECT_SIZE, RECT_SIZE))),
                MeshMaterial2d(materials.add(ColorMaterial::from(COLORS[note_ind]))),
                Transform::from_xyz(x_of_note_name, y_of_note_name, 0.0),
                Visibility::Visible,
                NoteNameRectLabel { note_name: note },
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
            .observe(on_cirlce_click);

        note_ind += 1;
    }
}

fn on_cirlce_click(
    click: Trigger<Pointer<Click>>,
    mut commands: Commands,
    mut note_name_rect_entity_q: Query<(Entity, &mut NoteNameRectLabel), With<NoteNameRectLabel>>,
    children_query: Query<&Children>,
) {
    let atuple = note_name_rect_entity_q.get_mut(click.target).unwrap();
    let note_name: &'static str = atuple.1.note_name;
    println!("Click on note, {:?}", note_name);
}
