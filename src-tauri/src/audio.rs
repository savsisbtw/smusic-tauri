use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

use crate::state::PlayerState;

pub fn create_device() -> MixerDeviceSink {
    DeviceSinkBuilder::open_default_sink().expect("open default audio device")
}

pub fn load_and_play(state: &mut PlayerState, index: usize) {
    let track = match state.queue.get(index) {
        Some(t) => t.clone(),
        None => return,
    };

    let file = match File::open(&track.path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let decoder = match Decoder::new(BufReader::new(file)) {
        Ok(d) => d,
        Err(_) => return,
    };

    let device = match state.device.as_ref() {
        Some(d) => d,
        None => return,
    };

    let player = Player::connect_new(device.mixer());
    player.set_volume(state.volume);
    player.append(decoder);

    state.player = Some(player);
    state.current_track = Some(track);
    state.queue_index = Some(index);
}

pub fn pause(state: &PlayerState) {
    if let Some(player) = &state.player {
        player.pause();
    }
}

pub fn resume(state: &PlayerState) {
    if let Some(player) = &state.player {
        player.play();
    }
}

pub fn seek(state: &PlayerState, position_ms: u64) {
    if let Some(player) = &state.player {
        let _ = player.try_seek(Duration::from_millis(position_ms));
    }
}

pub fn set_volume(state: &mut PlayerState, volume: f32) {
    state.volume = volume;
    if let Some(player) = &state.player {
        player.set_volume(volume);
    }
}

pub fn is_finished(state: &PlayerState) -> bool {
    state.current_track.is_some() && state.player.as_ref().is_some_and(|p| p.empty())
}

pub fn advance(state: &mut PlayerState) {
    match state.queue_index {
        Some(i) if i + 1 < state.queue.len() => {
            load_and_play(state, i + 1);
        }
        _ => {
            state.player = None;
            state.current_track = None;
            state.queue_index = None;
        }
    }
}

pub fn previous(state: &mut PlayerState) {
    let position_ms = state
        .player
        .as_ref()
        .map_or(0, |p| p.get_pos().as_millis() as u64);

    if position_ms > 3000 {
        if let Some(i) = state.queue_index {
            load_and_play(state, i);
        }
        return;
    }

    match state.queue_index {
        Some(i) if i > 0 => load_and_play(state, i - 1),
        Some(i) => load_and_play(state, i),
        None => {}
    }
}
