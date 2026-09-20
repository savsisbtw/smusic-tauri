use std::path::Path;

use walkdir::WalkDir;

use crate::state::Track;
use crate::tags;

fn is_audio_file(path: &Path) -> bool {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
    {
        Some(ext) => ext == "flac" || ext == "wav" || ext == "mp3",
        None => false,
    }
}

pub fn scan(folders: &[String], covers_dir: &Path) -> Vec<Track> {
    let mut tracks = Vec::new();

    for folder in folders {
        for entry in WalkDir::new(folder).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() || !is_audio_file(path) {
                continue;
            }

            let scanned = match tags::read_track(path) {
                Some(s) => s,
                None => continue,
            };

            let id = blake3::hash(path.to_string_lossy().as_bytes())
                .to_hex()
                .to_string();

            let cover_path = match &scanned.cover_bytes {
                Some(bytes) => {
                    let out_path = covers_dir.join(format!("{}.jpg", id));
                    if tags::save_cover(bytes, &out_path) {
                        Some(out_path.to_string_lossy().to_string())
                    } else {
                        None
                    }
                }
                None => None,
            };

            tracks.push(Track {
                id,
                path: path.to_string_lossy().to_string(),
                title: scanned.title,
                artist: scanned.artist,
                album: scanned.album,
                duration_ms: scanned.duration_ms,
                cover_path,
            });
        }
    }

    tracks
}
