function boot() {
  wirePlayerControls();
  wireLibraryControls();
  wireViewControls();

  Bridge.onStateChanged(applyState);

  Bridge.getState().then(applyState);
  refreshLibrary();
}

document.addEventListener("DOMContentLoaded", boot);
