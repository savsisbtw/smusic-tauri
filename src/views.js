function switchView(viewName) {
  document.querySelectorAll(".tab").forEach((tab) => {
    tab.classList.toggle("active", tab.dataset.view === viewName);
  });
  document.querySelectorAll(".view").forEach((view) => {
    view.classList.toggle("active", view.id === "view-" + viewName);
  });
  if (viewName === "library") {
    renderLibraryList();
  }
}

function wireViewControls() {
  document.querySelectorAll(".tab").forEach((tab) => {
    tab.addEventListener("click", () => switchView(tab.dataset.view));
  });

  document.getElementById("btn-hide").addEventListener("click", () => {
    Bridge.hideWindow();
  });

  document.getElementById("toggle-discord").addEventListener("change", (e) => {
    Bridge.setDiscordEnabled(e.target.checked);
  });
}
