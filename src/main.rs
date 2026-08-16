mod audio;
mod camera;
mod constants;
mod fretboard;
mod selection;
mod tuning;
mod ui;

use audio::{AudioSinkKeepAlive, NoteAudio};
use bevy::prelude::*;
use bevy::window::WindowPlugin;
use camera::{PinchZoom, TouchPan};
use rodio::DeviceSinkBuilder;
use tuning::CurrentTuning;

fn main() {
    let mut sink = DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
    sink.log_on_drop(false);
    let note_audio = NoteAudio {
        mixer: sink.mixer().clone(),
    };

    App::new()
        .insert_non_send(AudioSinkKeepAlive(sink))
        .insert_resource(note_audio)
        .insert_resource(CurrentTuning { index: 0 })
        .insert_resource(PinchZoom::default())
        .insert_resource(TouchPan::default())
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Guitar Notes".into(),
                    canvas: Some("#guitar-notes-canvas".into()),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            }),
            MeshPickingPlugin,
        ))
        .add_systems(Startup, fretboard::setup)
        .add_systems(
            Update,
            (
                ui::toggle_tuning_menu,
                fretboard::apply_tuning_selection,
                selection::play_selected_notes,
                selection::explain_selection,
                selection::clear_selection,
                ui::dismiss_power_chord_popup,
                camera::pinch_zoom_camera,
                camera::touch_pan_camera,
            ),
        )
        .run();
}
