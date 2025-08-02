use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::settings::*;

const NOTES: [&'static str; 7] = ["A", "B", "aC", "D", "E", "F", "G"];
const GAP: f32 = 100.0;
const GREY: Color = Color::srgba(0.3, 0.3, 0.3, 1.0);
const AMOUNT_OF_FRETS: u8 = 10;

struct Tunning {
    name: &'static str,
    notes: [&'static str; 6],
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
            font_size: 32.0,
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
        let y_of_line: f32 = i as f32 * GAP / 2.0;
        if i == tunning.notes.len() - 1 {
            last_str_y = Some(y_of_line);
        }
        let vertices: Vec<[f32; 3]> =
            vec![[line_start_x, y_of_line, 0.0], [line_end_x, y_of_line, 0.0]];
        // Создаём mesh линий
        let mut line_mesh: Mesh = Mesh::new(
            bevy::render::mesh::PrimitiveTopology::LineStrip,
            RenderAssetUsages::RENDER_WORLD,
        );
        line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        // Создаём Entity с этим Mesh
        commands.spawn((
            Mesh2d(meshes.add(line_mesh)),
            MeshMaterial2d(materials.add(Color::WHITE)),
        ));

        // Draw the NOTE name
        commands.spawn((
            Text2d::new(tunning.notes[i]),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(line_start_x - GAP / 2.0, y_of_line, 0.0),
        ));
    }

    if let Some(value) = last_str_y {
        let line_end_y: f32 = value + GAP / 2.0;
        // Рисуем вертикальные линии - "разделители ладов":
        for i in 0..AMOUNT_OF_FRETS {
            let line_start_y: f32 = -GAP / 2.0;
            let x_of_line: f32 = line_start_x + i as f32 * GAP;
            let vertices: Vec<[f32; 3]> =
                vec![[x_of_line, line_start_y, 0.0], [x_of_line, line_end_y, 0.0]];
            // Создаём mesh линий
            let mut line_mesh: Mesh = Mesh::new(
                bevy::render::mesh::PrimitiveTopology::LineStrip,
                RenderAssetUsages::RENDER_WORLD,
            );
            line_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
            // Создаём Entity с этим Mesh
            commands.spawn((
                Mesh2d(meshes.add(line_mesh)),
                MeshMaterial2d(materials.add(GREY)),
            ));
        }
    }
}
