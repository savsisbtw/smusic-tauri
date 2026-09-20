mod audio;
mod commands;
mod config;
mod discord;
mod library;
mod state;
mod tags;
mod tray;

use std::sync::Mutex;
use std::time::Duration;

use tauri::{Manager, State};

use state::{AppState, PlayerState};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            let app_data_dir = app.path().app_data_dir().expect("app data dir");
            std::fs::create_dir_all(&app_data_dir).ok();
            std::fs::create_dir_all(app_data_dir.join("covers")).ok();

            let cfg = config::load(&app_data_dir);
            let device = audio::create_device();

            let player_state = PlayerState {
                device: Some(device),
                player: None,
                current_track: None,
                queue: Vec::new(),
                queue_index: None,
                volume: cfg.volume,
                discord_enabled: cfg.discord_enabled,
                library_folders: cfg.library_folders,
                tracks: cfg.tracks,
            };

            app.manage(AppState {
                player: Mutex::new(player_state),
                discord: discord::DiscordHandle::new(),
            });

            tray::build(app)?;

            let poll_handle = app_handle.clone();
            std::thread::spawn(move || loop {
                std::thread::sleep(Duration::from_millis(250));
                let state: State<AppState> = poll_handle.state();

                let finished = {
                    let player = state.player.lock().unwrap();
                    audio::is_finished(&player)
                };

                if finished {
                    {
                        let mut player = state.player.lock().unwrap();
                        audio::advance(&mut player);
                    }
                    commands::broadcast(&poll_handle, &state);
                }
            });

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::get_library,
            commands::pick_library_folder,
            commands::add_library_folder,
            commands::remove_library_folder,
            commands::rescan_library,
            commands::play_track,
            commands::play,
            commands::pause,
            commands::next_track,
            commands::previous_track,
            commands::seek,
            commands::set_volume,
            commands::set_queue,
            commands::set_discord_enabled,
            commands::hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod smoke_tests {
    use crate::{audio, library, state, tags};

    fn write_test_wav(path: &std::path::Path) {
        let sample_rate: u32 = 8000;
        let num_samples = sample_rate;
        let mut data = Vec::with_capacity(num_samples as usize * 2);
        for i in 0..num_samples {
            let t = i as f32 / sample_rate as f32;
            let sample = (t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 8000.0;
            data.extend_from_slice(&(sample as i16).to_le_bytes());
        }

        let byte_rate = sample_rate * 2;
        let data_len = data.len() as u32;
        let mut wav = Vec::new();
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&byte_rate.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_len.to_le_bytes());
        wav.extend_from_slice(&data);

        std::fs::write(path, wav).unwrap();
    }

    #[test]
    fn scan_reads_tags_and_plays() {
        let dir = std::env::temp_dir().join("smusic_smoke_test");
        std::fs::create_dir_all(&dir).unwrap();
        let wav_path = dir.join("tone.wav");
        write_test_wav(&wav_path);

        let track = tags::read_track(&wav_path).expect("read_track should succeed on a valid wav");
        assert_eq!(track.title, "tone");
        assert!(track.duration_ms >= 900 && track.duration_ms <= 1100);

        let covers_dir = dir.join("covers");
        std::fs::create_dir_all(&covers_dir).unwrap();
        let tracks = library::scan(&[dir.to_string_lossy().to_string()], &covers_dir);
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "tone");

        let device = audio::create_device();
        let mut player_state = state::PlayerState {
            device: Some(device),
            player: None,
            current_track: None,
            queue: tracks.clone(),
            queue_index: None,
            volume: 0.0,
            discord_enabled: false,
            library_folders: Vec::new(),
            tracks: tracks.clone(),
        };

        audio::load_and_play(&mut player_state, 0);
        assert!(
            player_state.player.is_some(),
            "player should be created after load_and_play"
        );

        std::thread::sleep(std::time::Duration::from_millis(200));
        let pos = player_state.player.as_ref().unwrap().get_pos();
        assert!(pos.as_millis() > 0, "playback position should advance");

        std::fs::remove_dir_all(&dir).ok();
    }
}
