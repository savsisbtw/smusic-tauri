use tauri::{AppHandle, Emitter, Manager, State};

use crate::state::{AppState, PlaybackState, Track};
use crate::{audio, config, discord, library};

#[tauri::command]
pub fn get_state(state: State<AppState>) -> PlaybackState {
    state.player.lock().unwrap().snapshot()
}

#[tauri::command]
pub fn get_library(state: State<AppState>) -> serde_json::Value {
    let player = state.player.lock().unwrap();
    serde_json::json!({
        "tracks": player.tracks,
        "folders": player.library_folders,
    })
}

#[tauri::command]
pub fn pick_library_folder(app: AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog()
        .file()
        .blocking_pick_folder()
        .map(|p| p.to_string())
}

#[tauri::command]
pub fn add_library_folder(app: AppHandle, state: State<AppState>, path: String) {
    {
        let mut player = state.player.lock().unwrap();
        if !player.library_folders.contains(&path) {
            player.library_folders.push(path);
        }
    }
    persist(&app, &state);
}

#[tauri::command]
pub fn remove_library_folder(app: AppHandle, state: State<AppState>, path: String) {
    {
        let mut player = state.player.lock().unwrap();
        player.library_folders.retain(|f| f != &path);
    }
    persist(&app, &state);
}

#[tauri::command]
pub fn rescan_library(app: AppHandle, state: State<AppState>) -> Vec<Track> {
    let covers_dir = app.path().app_data_dir().unwrap().join("covers");
    let _ = std::fs::create_dir_all(&covers_dir);

    let folders = state.player.lock().unwrap().library_folders.clone();
    let tracks = library::scan(&folders, &covers_dir);

    {
        let mut player = state.player.lock().unwrap();
        player.tracks = tracks.clone();
    }
    persist(&app, &state);
    tracks
}

#[tauri::command]
pub fn play_track(app: AppHandle, state: State<AppState>, track_id: String) {
    {
        let mut player = state.player.lock().unwrap();
        if !player.queue.iter().any(|t| t.id == track_id) {
            player.queue = player.tracks.clone();
        }
        if let Some(index) = player.queue.iter().position(|t| t.id == track_id) {
            audio::load_and_play(&mut player, index);
        }
    }
    broadcast(&app, &state);
    persist(&app, &state);
}

#[tauri::command]
pub fn play(app: AppHandle, state: State<AppState>) {
    audio::resume(&state.player.lock().unwrap());
    broadcast(&app, &state);
}

#[tauri::command]
pub fn pause(app: AppHandle, state: State<AppState>) {
    audio::pause(&state.player.lock().unwrap());
    broadcast(&app, &state);
}

#[tauri::command]
pub fn next_track(app: AppHandle, state: State<AppState>) {
    {
        let mut player = state.player.lock().unwrap();
        audio::advance(&mut player);
    }
    broadcast(&app, &state);
}

#[tauri::command]
pub fn previous_track(app: AppHandle, state: State<AppState>) {
    {
        let mut player = state.player.lock().unwrap();
        audio::previous(&mut player);
    }
    broadcast(&app, &state);
}

#[tauri::command]
pub fn seek(app: AppHandle, state: State<AppState>, position_ms: u64) {
    audio::seek(&state.player.lock().unwrap(), position_ms);
    broadcast(&app, &state);
}

#[tauri::command]
pub fn set_volume(app: AppHandle, state: State<AppState>, volume: f32) {
    {
        let mut player = state.player.lock().unwrap();
        audio::set_volume(&mut player, volume);
    }
    broadcast(&app, &state);
    persist(&app, &state);
}

#[tauri::command]
pub fn set_queue(state: State<AppState>, track_ids: Vec<String>) {
    let mut player = state.player.lock().unwrap();
    let tracks = player.tracks.clone();
    player.queue = track_ids
        .iter()
        .filter_map(|id| tracks.iter().find(|t| &t.id == id).cloned())
        .collect();
}

#[tauri::command]
pub fn set_discord_enabled(app: AppHandle, state: State<AppState>, enabled: bool) {
    {
        let mut player = state.player.lock().unwrap();
        player.discord_enabled = enabled;
    }
    if !enabled {
        discord::clear(&state.discord);
    }
    persist(&app, &state);
    broadcast(&app, &state);
}

#[tauri::command]
pub fn hide_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

pub fn broadcast(app: &AppHandle, state: &State<AppState>) {
    let (snapshot, discord_enabled) = {
        let player = state.player.lock().unwrap();
        (player.snapshot(), player.discord_enabled)
    };

    let _ = app.emit("playback-state-changed", &snapshot);

    if discord_enabled {
        discord::update(&state.discord, &snapshot);
    }
}

fn persist(app: &AppHandle, state: &State<AppState>) {
    let player = state.player.lock().unwrap();
    let cfg = config::Config {
        library_folders: player.library_folders.clone(),
        tracks: player.tracks.clone(),
        volume: player.volume,
        last_track_id: player.current_track.as_ref().map(|t| t.id.clone()),
        discord_enabled: player.discord_enabled,
    };
    if let Ok(app_data_dir) = app.path().app_data_dir() {
        config::save(&app_data_dir, &cfg);
    }
}
