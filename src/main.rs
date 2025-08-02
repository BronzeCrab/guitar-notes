use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::settings::*;

const NOTES: [&'static str; 7] = ["A", "B", "C", "D", "E", "F", "G"];
const GAP: f32 = 100.0;

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

    // Рисуем горизонтальные линии "струны":
    for i in 0..tunning.notes.len() {
        let line_start: f32 = -window_width / 2.0 + GAP;
        let line_end: f32 = window_width / 2.0 - GAP;

        // Координаты разделительной линии
        let y_of_line: f32 = i as f32 * 40.0;
        let vertices: Vec<[f32; 3]> =
            vec![[line_start, y_of_line, 0.0], [line_end, y_of_line, 0.0]];
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
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Draw the NOTE
        commands.spawn((
            Text2d::new(tunning.notes[i]),
            TextFont {
                font_size: 32.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(line_start - GAP / 2.0, y_of_line, 0.0),
        ));
    }
}
