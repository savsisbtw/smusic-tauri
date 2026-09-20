use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::state::PlaybackState;

const CLIENT_ID: &str = "1543000586209853490";
const FALLBACK_IMAGE: &str = "https://mail.savsis.xyz/discord-rpc/appicon.png";
const PLAY_ICON: &str = "https://mail.savsis.xyz/discord-rpc/play_bw.png";
const PAUSE_ICON: &str = "https://mail.savsis.xyz/discord-rpc/pause_bw.png";
const RECONNECT_INTERVAL: Duration = Duration::from_secs(15);

struct Connection {
    pipe: Option<File>,
    last_attempt: Instant,
}

pub struct DiscordHandle {
    conn: Mutex<Connection>,
}

impl DiscordHandle {
    pub fn new() -> Self {
        DiscordHandle {
            conn: Mutex::new(Connection {
                pipe: None,
                last_attempt: Instant::now() - RECONNECT_INTERVAL,
            }),
        }
    }
}

fn try_connect() -> Option<File> {
    for i in 0..10 {
        let path = format!(r"\\.\pipe\discord-ipc-{}", i);
        let mut file = match OpenOptions::new().read(true).write(true).open(&path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let handshake = json!({ "v": 1, "client_id": CLIENT_ID });
        if send_frame(&mut file, 0, &handshake).is_err() {
            continue;
        }

        match read_frame(&mut file) {
            Ok(response) if response.get("evt").and_then(|v| v.as_str()) == Some("READY") => {
                return Some(file);
            }
            _ => continue,
        }
    }
    None
}

fn send_frame(file: &mut File, opcode: i32, payload: &Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(payload)?;
    let mut header = Vec::with_capacity(8);
    header.extend_from_slice(&opcode.to_le_bytes());
    header.extend_from_slice(&(body.len() as i32).to_le_bytes());
    file.write_all(&header)?;
    file.write_all(&body)?;
    file.flush()?;
    Ok(())
}

fn read_frame(file: &mut File) -> std::io::Result<Value> {
    let mut header = [0u8; 8];
    file.read_exact(&mut header)?;
    let length = i32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
    let mut body = vec![0u8; length];
    file.read_exact(&mut body)?;
    Ok(serde_json::from_slice(&body).unwrap_or(Value::Null))
}

fn ensure_connected(conn: &mut Connection) -> bool {
    if conn.pipe.is_some() {
        return true;
    }
    if conn.last_attempt.elapsed() < RECONNECT_INTERVAL {
        return false;
    }
    conn.last_attempt = Instant::now();
    conn.pipe = try_connect();
    conn.pipe.is_some()
}

fn truncate(value: &str) -> String {
    if value.is_empty() {
        return " ".to_string();
    }
    if value.chars().count() <= 128 {
        value.to_string()
    } else {
        value.chars().take(128).collect()
    }
}

fn build_activity(state: &PlaybackState) -> Option<Value> {
    let track = state.current_track.as_ref()?;

    let large_text = if track.album.is_empty() {
        &track.title
    } else {
        &track.album
    };

    let mut activity = json!({
        "type": 2,
        "details": truncate(&track.title),
        "assets": {
            "large_image": FALLBACK_IMAGE,
            "large_text": truncate(large_text),
            "small_image": if state.is_playing { PLAY_ICON } else { PAUSE_ICON },
            "small_text": if state.is_playing { "Playing" } else { "Paused" },
        }
    });

    if state.is_playing && track.duration_ms > 0 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let elapsed_secs = (state.position_ms / 1000) as i64;
        let total_secs = (track.duration_ms / 1000) as i64;
        let start = now - elapsed_secs;
        let end = start + total_secs;
        activity["timestamps"] = json!({ "start": start, "end": end });
    }

    Some(activity)
}

fn make_nonce() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:x}", nanos)
}

fn send_activity(conn: &mut Connection, activity: Value) {
    let payload = json!({
        "cmd": "SET_ACTIVITY",
        "args": { "pid": std::process::id(), "activity": activity },
        "nonce": make_nonce(),
    });

    if let Some(file) = conn.pipe.as_mut() {
        if send_frame(file, 1, &payload).is_err() || read_frame(file).is_err() {
            conn.pipe = None;
        }
    }
}

pub fn update(handle: &DiscordHandle, state: &PlaybackState) {
    let mut conn = handle.conn.lock().unwrap();
    if !ensure_connected(&mut conn) {
        return;
    }
    let activity = match build_activity(state) {
        Some(a) => a,
        None => Value::Null,
    };
    send_activity(&mut conn, activity);
}

pub fn clear(handle: &DiscordHandle) {
    let mut conn = handle.conn.lock().unwrap();
    if conn.pipe.is_none() {
        return;
    }
    send_activity(&mut conn, Value::Null);
}
