use std::path::{Path, PathBuf};

use lofty::prelude::*;
use lofty::probe::Probe;

pub struct ScannedTrack {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration_ms: u64,
    pub cover_bytes: Option<Vec<u8>>,
}

pub fn read_track(path: &Path) -> Option<ScannedTrack> {
    let tagged_file = Probe::open(path).ok()?.read().ok()?;
    let tag = tagged_file.primary_tag().or_else(|| tagged_file.first_tag());

    let fallback_title = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let title = tag
        .and_then(|t| t.title())
        .map(|c| c.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback_title);
    let artist = tag
        .and_then(|t| t.artist())
        .map(|c| c.to_string())
        .unwrap_or_default();
    let album = tag
        .and_then(|t| t.album())
        .map(|c| c.to_string())
        .unwrap_or_default();
    let duration_ms = tagged_file.properties().duration().as_millis() as u64;

    let cover_bytes = tag
        .and_then(|t| t.pictures().first())
        .map(|pic| pic.data().to_vec());

    Some(ScannedTrack {
        title,
        artist,
        album,
        duration_ms,
        cover_bytes,
    })
}

pub fn save_cover(cover_bytes: &[u8], out_path: &PathBuf) -> bool {
    let img = match image::load_from_memory(cover_bytes) {
        Ok(i) => i,
        Err(_) => return false,
    };
    let resized = img.thumbnail(600, 600);
    resized
        .save_with_format(out_path, image::ImageFormat::Jpeg)
        .is_ok()
}
