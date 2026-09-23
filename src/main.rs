mod audio;
mod constants;
mod fretboard;
mod selection;
mod sequence;
mod tuning;
mod ui;

use audio::{AudioSinkKeepAlive, NoteAudio};
use bevy::prelude::*;
use bevy::ui::UiSystems;
use bevy::window::WindowPlugin;
use rodio::DeviceSinkBuilder;
use rodio::mixer::mixer;
use sequence::{
    ChordPlayback, ChordSelection, ChordSelectionToken, CurrentMode, SelectionCounter, Sequence,
    SequencePlayback, SequenceToken,
};
use tuning::CurrentTuning;

fn main() {
    console_error_panic_hook::set_once();
    let sink = match DeviceSinkBuilder::open_default_sink() {
        Ok(mut s) => {
            s.log_on_drop(false);
            Some(s)
        }
        Err(e) => {
            error!("open default audio stream failed: {e}; running without audio");
            None
        }
    };
    let note_audio = NoteAudio {
        mixer: sink
            .as_ref()
            .map(|s| s.mixer().clone())
            .unwrap_or_else(|| mixer(2.try_into().unwrap(), 44100.try_into().unwrap()).0),
    };

    App::new()
        .insert_non_send(AudioSinkKeepAlive(sink))
        .insert_resource(note_audio)
        .insert_resource(CurrentTuning { index: 0 })
        .insert_resource(CurrentMode::default())
        .insert_resource(Sequence::default())
        .insert_resource(SequenceToken::default())
        .insert_resource(SequencePlayback::default())
        .insert_resource(ChordSelection::default())
        .insert_resource(ChordSelectionToken::default())
        .insert_resource(SelectionCounter::default())
        .insert_resource(ChordPlayback::default())
        .insert_resource(fretboard::FretboardLayoutState::default())
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Guitar Notes".into(),
                canvas: Some("#guitar-notes-canvas".into()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, fretboard::setup)
        .add_systems(
            Update,
            (
                ui::toggle_tuning_menu,
                ui::scroll_page,
                ui::drag_page_scrollbar,
                fretboard::apply_tuning_selection,
                fretboard::handle_note_click,
                selection::play_selected_notes,
                selection::explain_selection,
                selection::clear_selection,
                sequence::toggle_mode,
                sequence::track_selected_notes,
                sequence::refresh_chord_visuals.after(sequence::track_selected_notes),
                sequence::update_selected_notes_panel.after(sequence::refresh_chord_visuals),
                sequence::handle_selected_entry_click,
                sequence::chord_playback_tick,
                sequence::refresh_sequence_visuals.after(sequence::sequence_playback),
                sequence::sequence_playback,
                ui::dismiss_power_chord_popup,
            ),
        )
        .add_systems(
            PostUpdate,
            (
                ui::update_page_scrollbar.after(UiSystems::Layout),
                fretboard::layout_fretboard.before(UiSystems::Layout),
            ),
        )
        .run();
}
