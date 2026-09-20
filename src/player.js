let latestState = null;
let latestStateAt = 0;
let shownTrackId = null;
let seeking = false;

function formatTime(ms) {
  const totalSeconds = Math.max(0, Math.floor(ms / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return minutes + ":" + String(seconds).padStart(2, "0");
}

function applyState(state) {
  latestState = state;
  latestStateAt = Date.now();

  const track = state.current_track;
  const titleEl = document.getElementById("track-title");
  const artistEl = document.getElementById("track-artist");

  if (track) {
    titleEl.textContent = track.title;
    artistEl.textContent = track.artist || "";
  } else {
    titleEl.textContent = "ничего не играет";
    artistEl.textContent = "";
  }

  const trackId = track ? track.id : null;
  if (trackId !== shownTrackId) {
    shownTrackId = trackId;
    const coverUrl = track && track.cover_path ? window.__TAURI__.core.convertFileSrc(track.cover_path) : "";
    setCoverArt(coverUrl);
  }

  setPlayIconState(state.is_playing);

  const volumeSlider = document.getElementById("volume-slider");
  if (document.activeElement !== volumeSlider) {
    volumeSlider.value = Math.round(state.volume * 100);
  }

  renderProgress(state.position_ms, track ? track.duration_ms : 0);
}

function renderProgress(positionMs, durationMs) {
  const fill = document.getElementById("progress-fill");
  const thumb = document.getElementById("progress-thumb");
  const elapsedEl = document.getElementById("time-elapsed");
  const remainingEl = document.getElementById("time-remaining");

  const pct = durationMs > 0 ? Math.min(100, (positionMs / durationMs) * 100) : 0;
  fill.style.width = pct + "%";
  thumb.style.left = pct + "%";
  elapsedEl.textContent = formatTime(positionMs);
  remainingEl.textContent = "-" + formatTime(Math.max(0, durationMs - positionMs));
}

function progressLoop() {
  requestAnimationFrame(progressLoop);
  if (!latestState || !latestState.is_playing || seeking) return;

  const track = latestState.current_track;
  if (!track) return;

  const elapsedSinceEvent = Date.now() - latestStateAt;
  const interpolatedPosition = latestState.position_ms + elapsedSinceEvent;
  renderProgress(Math.min(interpolatedPosition, track.duration_ms), track.duration_ms);
}

function seekFromClientX(clientX) {
  const track = latestState ? latestState.current_track : null;
  if (!track) return;

  const rect = document.getElementById("progress-track").getBoundingClientRect();
  const ratio = Math.min(1, Math.max(0, (clientX - rect.left) / rect.width));
  const targetMs = Math.round(ratio * track.duration_ms);
  renderProgress(targetMs, track.duration_ms);
  return targetMs;
}

function wirePlayerControls() {
  document.getElementById("btn-play").addEventListener("click", () => {
    if (latestState && latestState.is_playing) {
      Bridge.pause();
    } else {
      Bridge.play();
    }
  });

  document.getElementById("btn-prev").addEventListener("click", () => Bridge.previousTrack());
  document.getElementById("btn-next").addEventListener("click", () => Bridge.nextTrack());

  const progressTrack = document.getElementById("progress-track");
  progressTrack.addEventListener("mousedown", (e) => {
    seeking = true;
    seekFromClientX(e.clientX);
  });
  window.addEventListener("mousemove", (e) => {
    if (seeking) seekFromClientX(e.clientX);
  });
  window.addEventListener("mouseup", (e) => {
    if (!seeking) return;
    seeking = false;
    const targetMs = seekFromClientX(e.clientX);
    if (targetMs !== undefined) Bridge.seek(targetMs);
  });

  document.getElementById("volume-slider").addEventListener("input", (e) => {
    Bridge.setVolume(Number(e.target.value) / 100);
  });

  requestAnimationFrame(progressLoop);
}
