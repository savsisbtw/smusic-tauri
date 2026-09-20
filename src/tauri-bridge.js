const invoke = window.__TAURI__.core.invoke;
const tauriListen = window.__TAURI__.event.listen;

const Bridge = {
  getState() {
    return invoke("get_state");
  },
  getLibrary() {
    return invoke("get_library");
  },
  pickLibraryFolder() {
    return invoke("pick_library_folder");
  },
  addLibraryFolder(path) {
    return invoke("add_library_folder", { path });
  },
  removeLibraryFolder(path) {
    return invoke("remove_library_folder", { path });
  },
  rescanLibrary() {
    return invoke("rescan_library");
  },
  playTrack(trackId) {
    return invoke("play_track", { trackId });
  },
  play() {
    return invoke("play");
  },
  pause() {
    return invoke("pause");
  },
  nextTrack() {
    return invoke("next_track");
  },
  previousTrack() {
    return invoke("previous_track");
  },
  seek(positionMs) {
    return invoke("seek", { positionMs });
  },
  setVolume(volume) {
    return invoke("set_volume", { volume });
  },
  setQueue(trackIds) {
    return invoke("set_queue", { trackIds });
  },
  setDiscordEnabled(enabled) {
    return invoke("set_discord_enabled", { enabled });
  },
  hideWindow() {
    return invoke("hide_window");
  },
  onStateChanged(handler) {
    return tauriListen("playback-state-changed", (event) => handler(event.payload));
  },
};
