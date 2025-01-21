// Fetch the leaderboard of songs
const rankedSongs = new Map();

async function fetchLeaderboardInitial() {
    try {
        const response = await fetch('/user-ranked-songs');
        const songs = await response.json();

        const leaderboardContainer = document.getElementById('left');
        leaderboardContainer.innerHTML = ''; // Clear previous content

        if (songs.length === 0) {
            leaderboardContainer.innerHTML = '<p>No songs ranked yet.</p>';
            return;
        }

        // Create and display the leaderboard
        const leaderboard = document.createElement('div');
        songs.forEach((song, index) => {
            rankedSongs.set(song.rank, song);
        });

        fetchLeaderboard();

        // rankedSongs.forEach((song) => {
        //     const songItem = document.createElement('div');
        //     songItem.classList.add('song-item');
        //
        //     const songName = document.createElement('div');
        //     songName.classList.add('song-name');
        //     songName.textContent = song.name;
        //
        //     const songArtist = document.createElement('div');
        //     songArtist.classList.add('song-artist');
        //     songArtist.textContent = song.artist;
        //
        //     const songRank = document.createElement('div');
        //     songRank.classList.add('song-rank');
        //     songRank.textContent = `Rank #${song.rank}`;
        //
        //     songItem.appendChild(songName);
        //     songItem.appendChild(songArtist);
        //     songItem.appendChild(songRank);
        //
        //     leaderboard.appendChild(songItem);
        // });
        //
        // leaderboardContainer.appendChild(leaderboard);
    } catch (error) {
        document.getElementById('left').innerHTML = '<p>No songs ranked yet.</p>';
        console.error('Error fetching leaderboard:', error);
    }
}

async function fetchLeaderboard() {
    const sortedSongs = new Map([...rankedSongs.entries()].sort((a, b) => Number(a[0]) - Number(b[0])));

    const leaderboardContainer = document.getElementById('left');
    leaderboardContainer.innerHTML = ''; // Clear previous content

    if (sortedSongs.length === 0) {
        leaderboardContainer.innerHTML = '<p>No songs ranked yet.</p>';
        return;
    }

    // Create and display the leaderboard
    const leaderboard = document.createElement('div');
    sortedSongs.forEach((song, index) => {

        const song_container = document.createElement('div');
        song_container.classList.add('song');

        const songName = document.createElement('div');
        songName.classList.add('song-name');
        songName.textContent = song.name;

        const songArtist = document.createElement('div');
        songArtist.classList.add('song-artist');
        songArtist.textContent = song.artist;

        const songRank = document.createElement('div');
        songRank.classList.add('song-rank');
        songRank.textContent = `Rank #${song.rank}`;

        // Create an image element for the album art
        const albumArt = document.createElement('img');
        albumArt.classList.add('song-album');
        albumArt.src = song.image_url; // Replace with the actual image URL from your song object
        albumArt.alt = `${song.artist} - Album Art`;
        albumArt.height = 150;
        albumArt.width = 150;

        song_container.appendChild(songName)
        song_container.appendChild(songArtist);
        song_container.appendChild(albumArt)
        song_container.appendChild(songRank);

        leaderboard.appendChild(song_container);
    });

    const saveButtonArea = document.createElement('div');
    saveButtonArea.classList.add("save-button")

    const saveButton = document.createElement('button');
    saveButton.textContent = 'Save!';
    saveButton.onclick = () => {

    }

    saveButtonArea.appendChild(saveButton)
    leaderboardContainer.appendChild(saveButtonArea)
    leaderboardContainer.appendChild(leaderboard);

}

// Function to display the "Add Song" form
function displayAddSongForm() {
    const container = document.getElementById('right');
    container.innerHTML = `
      <form id="add-song-form">
          <h3>Search for a song!</h3>
          <input type="text" id="song-name" placeholder="Song Name" required>
          <input type="number" id="song-rank" placeholder="Rank (1-10)" min="1" max="10" required>
          <button type="submit">Find Song</button>
      </form>
  `;

    // Handle form submission
    const form = document.getElementById('add-song-form');
    form.addEventListener('submit', async (event) => {
        event.preventDefault();

        const songName = document.getElementById('song-name').value;
        const songRank = document.getElementById('song-rank').value;


        const queryParams = new URLSearchParams({
            track: songName,
            rank: songRank
        });


        try {
            const search_response = await fetch(`/search-songs?${queryParams}`, {
                method: 'GET',
            });

            if (search_response.ok) {
                const songs = await search_response.json(); // Parse JSON from the response
                console.log('Songs:', songs);
                displaySongs(songs, songRank);
            }
        } catch (error) {
            console.error('Error adding song:', error);
            alert('Error adding song.');
        }
    });
}

// Handle back and forward browser navigation
window.addEventListener('popstate', (event) => {
    if (event.state && event.state.page === 'leaderboard') {
        fetchLeaderboard();
    } else if (event.state && event.state.page === 'add-song') {
        displayAddSongForm();
    }
});

// Initial page load
window.onload = () => {
    const path = window.location.pathname.split('/')[1];
    displayAddSongForm();
    fetchLeaderboardInitial();
};

async function handleAddToRankedList(song, songRank){
    const queryParams = new URLSearchParams({
        track: song,
        rank: songRank
    });

    try {
        const response = await fetch(`/add-to-ranked-list?${queryParams}`, {
            method: 'POST',
        });

        if (response.ok) {
            displayAddSongForm();
        } else {
            alert('Failed to add song.');
        }
    } catch (error) {
        console.error('Error adding song:', error);
        alert('Error adding song.');
    }
    ;
}

// Function to display songs in the UI with rank input
function displaySongs(songs, songRank) {
    const songListContainer = document.getElementById('right');
    songListContainer.innerHTML = '';  // Clear the list first

    songs.forEach((song) => {
        song.rank = songRank;
        const songDiv = document.createElement('div');
        songDiv.classList.add('song');

        const songName = document.createElement('div');
        songName.classList.add('song-name');
        songName.textContent = song.name;

        const artistName = document.createElement('div');
        artistName.classList.add('artist-name');
        artistName.textContent = `Artist: ${song.artist}`;

        const rankElement = document.createElement('div');
        rankElement.classList.add('rank-element');
        rankElement.textContent = `Rank: ${song.rank}`;

        // Create an image element for the album art
        const albumArt = document.createElement('img');
        albumArt.classList.add('album-art');
        albumArt.src = song.image_url; // Replace with the actual image URL from your song object
        albumArt.alt = `${song.artist} - Album Art`;
        albumArt.height = 200;
        albumArt.width = 200;

        // Add song rank to list when selected
        const rankButton = document.createElement('button');
        rankButton.textContent = 'Add to List';
        rankButton.onclick = () => {
            rankedSongs.set(song.rank, song);
            fetchLeaderboard();
            displayAddSongForm();
        }

        songDiv.appendChild(songName);
        songDiv.appendChild(artistName);
        songDiv.appendChild(rankElement);
        songDiv.appendChild(albumArt);
        songDiv.appendChild(rankButton);

        songListContainer.appendChild(songDiv);
    });
}