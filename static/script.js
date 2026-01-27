// State
const rankedSongs = new Map();
const songKeys = new Set();

// Drag and drop state
let draggedItem = null;
let draggedRank = null;

// DOM Elements
const searchForm = document.getElementById('search-form');
const searchInput = document.getElementById('search-input');
const rankInput = document.getElementById('rank-input');
const searchResults = document.getElementById('search-results');
const rankingsList = document.getElementById('rankings-list');
const saveBtn = document.getElementById('save-btn');

// Initialize
window.onload = () => {
  loadSavedSongs();
  setupEventListeners();
};

function setupEventListeners() {
  searchForm.addEventListener('submit', handleSearch);
  saveBtn.addEventListener('click', handleSave);
}

// Load saved songs from server
async function loadSavedSongs() {
  try {
    const response = await fetch('/songs');
    const songs = await response.json();

    songs.forEach(song => {
      rankedSongs.set(song.rank, song);
      songKeys.add(song.name + song.artist);
    });

    renderRankings();
  } catch (error) {
    console.error('Error loading songs:', error);
  }
}

// Search for songs
async function handleSearch(event) {
  event.preventDefault();

  const query = searchInput.value.trim();
  const rank = parseInt(rankInput.value);

  if (!query || !rank) return;

  // Show loading state
  searchResults.innerHTML = `
    <div class="loading">
      <div class="spinner"></div>
    </div>
  `;

  try {
    const params = new URLSearchParams({ track: query, rank: rank });
    const response = await fetch(`/search-songs?${params}`);
    const songs = await response.json();

    renderSearchResults(songs, rank);
  } catch (error) {
    console.error('Error searching:', error);
    searchResults.innerHTML = `
      <div class="empty-state">
        <p class="empty-state-text">Error searching. Please try again.</p>
      </div>
    `;
  }
}

// Render search results
function renderSearchResults(songs, rank) {
  if (songs.length === 0) {
    searchResults.innerHTML = `
      <div class="empty-state">
        <p class="empty-state-text">No songs found</p>
      </div>
    `;
    return;
  }

  searchResults.innerHTML = songs.map(song => `
    <div class="song-card animate-fade-in" data-song='${JSON.stringify(song).replace(/'/g, "&#39;")}' data-rank="${rank}">
      <img src="${song.album_cover_url}" alt="${song.name}" class="song-card-artwork">
      <div class="song-card-info">
        <div class="song-card-title">${song.name}</div>
        <div class="song-card-artist">${song.artist}</div>
      </div>
      <div class="song-card-action">
        <button class="btn btn-secondary btn-icon add-song-btn">+</button>
      </div>
    </div>
  `).join('');

  // Add click handlers
  searchResults.querySelectorAll('.song-card').forEach(card => {
    card.addEventListener('click', () => handleAddSong(card));
  });
}

// Add song to rankings
function handleAddSong(card) {
  const song = JSON.parse(card.dataset.song.replace(/&#39;/g, "'"));
  const rank = parseInt(card.dataset.rank);
  const key = song.name + song.artist;

  // Check for duplicate
  if (songKeys.has(key)) {
    alert('This song is already in your list!');
    return;
  }

  // Remove existing song at this rank
  if (rankedSongs.has(rank)) {
    const existingSong = rankedSongs.get(rank);
    songKeys.delete(existingSong.name + existingSong.artist);
  }

  // Add new song
  song.rank = rank;
  rankedSongs.set(rank, song);
  songKeys.add(key);

  // Animate card
  card.classList.add('animate-pulse');

  // Update UI
  renderRankings();

  // Clear search
  searchInput.value = '';
  rankInput.value = '';
  searchResults.innerHTML = '';
}

// Render rankings list
function renderRankings() {
  if (rankedSongs.size === 0) {
    rankingsList.innerHTML = `
      <div class="empty-state">
        <div class="empty-state-icon">🎵</div>
        <p class="empty-state-text">Search for songs to add to your list</p>
      </div>
    `;
    return;
  }

  // Sort by rank
  const sortedSongs = [...rankedSongs.entries()].sort((a, b) => a[0] - b[0]);

  rankingsList.innerHTML = sortedSongs.map(([rank, song]) => `
    <div class="ranked-item animate-fade-in" data-rank="${rank}" draggable="true">
      <div class="rank-badge">${rank}</div>
      <img src="${song.album_cover_url}" alt="${song.name}" class="ranked-item-artwork">
      <div class="ranked-item-info">
        <div class="ranked-item-title">${song.name}</div>
        <div class="ranked-item-artist">${song.artist}</div>
      </div>
      <button class="btn btn-icon ranked-item-remove" data-rank="${rank}">×</button>
    </div>
  `).join('');

  // Add remove handlers
  rankingsList.querySelectorAll('.ranked-item-remove').forEach(btn => {
    btn.addEventListener('click', (e) => {
      e.stopPropagation();
      handleRemoveSong(parseInt(btn.dataset.rank));
    });
  });

  // Add drag event listeners
  rankingsList.querySelectorAll('.ranked-item').forEach(item => {
    item.addEventListener('dragstart', handleDragStart);
    item.addEventListener('dragend', handleDragEnd);
    item.addEventListener('dragover', handleDragOver);
    item.addEventListener('dragenter', handleDragEnter);
    item.addEventListener('dragleave', handleDragLeave);
    item.addEventListener('drop', handleDrop);
  });
}

// Remove song from rankings
function handleRemoveSong(rank) {
  const song = rankedSongs.get(rank);
  if (song) {
    songKeys.delete(song.name + song.artist);
    rankedSongs.delete(rank);
    renderRankings();
  }
}

// Save songs to server
async function handleSave() {
  if (rankedSongs.size < 10) {
    alert('Please add 10 songs before saving!');
    return;
  }

  saveBtn.disabled = true;
  saveBtn.textContent = 'Saving...';

  try {
    const response = await fetch('/songs', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(Array.from(rankedSongs.values()))
    });

    if (response.ok) {
      alert('Songs saved successfully!');
    } else {
      throw new Error('Failed to save');
    }
  } catch (error) {
    console.error('Error saving:', error);
    alert('Error saving songs. Please try again.');
  } finally {
    saveBtn.disabled = false;
    saveBtn.textContent = 'Save';
  }
}

// Drag and drop handlers
function handleDragStart(e) {
  draggedItem = this;
  draggedRank = parseInt(this.dataset.rank);
  this.classList.add('dragging');
  e.dataTransfer.effectAllowed = 'move';
  e.dataTransfer.setData('text/plain', draggedRank.toString());
}

function handleDragEnd() {
  this.classList.remove('dragging');
  document.querySelectorAll('.ranked-item').forEach(item => {
    item.classList.remove('drag-over-top', 'drag-over-bottom');
  });
  draggedItem = null;
  draggedRank = null;
}

function handleDragOver(e) {
  e.preventDefault();
  e.dataTransfer.dropEffect = 'move';

  if (this === draggedItem) return;

  const rect = this.getBoundingClientRect();
  const midpoint = rect.top + rect.height / 2;

  this.classList.remove('drag-over-top', 'drag-over-bottom');
  if (e.clientY < midpoint) {
    this.classList.add('drag-over-top');
  } else {
    this.classList.add('drag-over-bottom');
  }
}

function handleDragEnter(e) {
  e.preventDefault();
}

function handleDragLeave() {
  this.classList.remove('drag-over-top', 'drag-over-bottom');
}

function handleDrop(e) {
  e.preventDefault();

  if (draggedItem === this) return;

  const targetRank = parseInt(this.dataset.rank);
  const rect = this.getBoundingClientRect();
  const midpoint = rect.top + rect.height / 2;
  const insertBefore = e.clientY < midpoint;

  reorderSongs(draggedRank, targetRank, insertBefore);

  this.classList.remove('drag-over-top', 'drag-over-bottom');
}

function reorderSongs(fromRank, toRank, insertBefore) {
  // Get current order as array
  const songs = Array.from(rankedSongs.entries())
    .sort((a, b) => a[0] - b[0]);

  // Remove song from old position
  const [movedSong] = songs.splice(fromRank - 1, 1);

  // Calculate new index
  let newIndex = toRank - 1;
  if (!insertBefore) newIndex++;
  if (fromRank < toRank) newIndex--;

  // Insert at new position
  songs.splice(newIndex, 0, movedSong);

  // Reassign ranks and rebuild the map
  rankedSongs.clear();
  songs.forEach(([, song], index) => {
    song.rank = index + 1;
    rankedSongs.set(song.rank, song);
  });

  renderRankings();
}
