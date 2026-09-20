let allTracks = [];
let libraryFolders = [];
let currentSearch = "";

function matchesSearch(track, query) {
  if (!query) return true;
  const haystack = (track.title + " " + track.artist + " " + track.album).toLowerCase();
  return haystack.includes(query.toLowerCase());
}

function renderLibraryList() {
  const listEl = document.getElementById("library-list");
  const emptyEl = document.getElementById("library-empty");
  const filtered = allTracks.filter((t) => matchesSearch(t, currentSearch));

  listEl.innerHTML = "";

  if (filtered.length === 0) {
    emptyEl.classList.add("active");
    if (allTracks.length === 0) startIdleTypewriter();
    return;
  }

  emptyEl.classList.remove("active");
  stopIdleTypewriter();

  const rows = filtered.map((track) => {
    const row = document.createElement("div");
    row.className = "library-row";
    if (latestState && latestState.current_track && latestState.current_track.id === track.id) {
      row.classList.add("is-current");
    }
    row.innerHTML =
      '<span class="library-row-title"></span>' +
      '<span class="library-row-artist"></span>' +
      '<span class="library-row-duration"></span>';
    row.querySelector(".library-row-title").textContent = track.title;
    row.querySelector(".library-row-artist").textContent = track.artist || "";
    row.querySelector(".library-row-duration").textContent = formatTime(track.duration_ms);
    row.addEventListener("click", () => {
      Bridge.setQueue(filtered.map((t) => t.id)).then(() => Bridge.playTrack(track.id));
    });
    listEl.appendChild(row);
    return row;
  });

  staggerIn(rows, 0, 25, 30);
}

function renderFolderList() {
  const el = document.getElementById("folder-list");
  el.innerHTML = "";
  libraryFolders.forEach((folder) => {
    const row = document.createElement("div");
    row.className = "folder-row";
    row.innerHTML = "<span></span><button>убрать</button>";
    row.querySelector("span").textContent = folder;
    row.querySelector("button").addEventListener("click", () => {
      Bridge.removeLibraryFolder(folder).then(refreshLibrary);
    });
    el.appendChild(row);
  });
}

function refreshLibrary() {
  return Bridge.getLibrary().then((result) => {
    allTracks = result.tracks;
    libraryFolders = result.folders;
    renderLibraryList();
    renderFolderList();
  });
}

function wireLibraryControls() {
  document.getElementById("btn-add-folder").addEventListener("click", () => {
    Bridge.pickLibraryFolder().then((path) => {
      if (!path) return;
      Bridge.addLibraryFolder(path)
        .then(() => Bridge.rescanLibrary())
        .then(refreshLibrary);
    });
  });

  document.getElementById("btn-rescan").addEventListener("click", () => {
    Bridge.rescanLibrary().then(refreshLibrary);
  });

  document.getElementById("library-search").addEventListener("input", (e) => {
    currentSearch = e.target.value;
    renderLibraryList();
  });
}
