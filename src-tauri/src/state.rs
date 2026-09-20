use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::discord::DiscordHandle;

#[derive(Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub path: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub cover_path: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct PlaybackState {
    pub current_track: Option<Track>,
    pub is_playing: bool,
    pub position_ms: u64,
    pub volume: f32,
    pub queue: Vec<Track>,
    pub queue_index: Option<usize>,
}

pub struct PlayerState {
    pub device: Option<rodio::MixerDeviceSink>,
    pub player: Option<rodio::Player>,
    pub current_track: Option<Track>,
    pub queue: Vec<Track>,
    pub queue_index: Option<usize>,
    pub volume: f32,
    pub discord_enabled: bool,
    pub library_folders: Vec<String>,
    pub tracks: Vec<Track>,
}

impl PlayerState {
    pub fn snapshot(&self) -> PlaybackState {
        let is_playing = self.player.as_ref().is_some_and(|p| !p.is_paused());
        let position_ms = self
            .player
            .as_ref()
            .map_or(0, |p| p.get_pos().as_millis() as u64);

        PlaybackState {
            current_track: self.current_track.clone(),
            is_playing,
            position_ms,
            volume: self.volume,
            queue: self.queue.clone(),
            queue_index: self.queue_index,
        }
    }
}

pub struct AppState {
    pub player: Mutex<PlayerState>,
    pub discord: DiscordHandle,
}
