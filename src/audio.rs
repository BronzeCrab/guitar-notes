use crate::constants::NOTE_PLAY_DURATION_MS;
use bevy::prelude::*;
use rodio::MixerDeviceSink;
use rodio::mixer::Mixer;
use rodio::source::{SineWave, Source};
use std::time::Duration;

/// Keeps the OS audio stream alive for the app lifetime (`cpal::Stream` is `!Send`).
#[allow(dead_code)]
pub struct AudioSinkKeepAlive(pub MixerDeviceSink);

#[derive(Resource, Clone)]
pub struct NoteAudio {
    pub mixer: Mixer,
}

pub fn play_note_hz(audio: &NoteAudio, hz: f32) {
    let note_duration: Duration = Duration::from_millis(NOTE_PLAY_DURATION_MS);
    let wave = SineWave::new(hz)
        .amplify(0.2)
        .take_duration(note_duration)
        .fade_in(Duration::from_millis(5))
        .fade_out(note_duration);
    audio.mixer.add(wave);
}
