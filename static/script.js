// Fetch the leaderboard of songs
const rankedSongs = new Map();
const songKey = new Set();


// Initial page load
window.onload = () => {
    const path = window.location.pathname.split('/')[1];
    displayAddSongForm();
    fetchLeaderboardInitial();
};

async function fetchLeaderboardInitial() {
    try {
        const response = await fetch('/songs', {
            method: 'GET',
        });
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
            songKey.add(song.name+song.artist)
        });

        fetchLeaderboard();
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
        song_container.classList.add('ranked-song');

        const rank_wrapper = document.createElement('div');
        rank_wrapper.classList.add('ranked-songs-wrapper');

        const songName = document.createElement('div');
        songName.classList.add('song-name');
        songName.textContent = song.name;

        const songArtist = document.createElement('div');
        songArtist.classList.add('song-artist');
        songArtist.textContent = song.artist;

        const songRank = document.createElement('div');
        songRank.classList.add('song-rank');
        songRank.textContent = `Rank #${song.rank}`;

        const albumArt = document.createElement('img');
        albumArt.classList.add('song-album');
        albumArt.src = song.album_cover_url;
        albumArt.alt = `${song.artist} - Album Art`;
        albumArt.height = 100;
        albumArt.width = 100;

        song_container.appendChild(songName)
        song_container.appendChild(songArtist);
        song_container.appendChild(songRank);

        rank_wrapper.appendChild(albumArt)
        rank_wrapper.appendChild(song_container)

        leaderboard.appendChild(rank_wrapper);
    });

    const saveButtonArea = document.createElement('div');
    saveButtonArea.classList.add("save-button")

    const saveButton = document.createElement('button');
    saveButton.textContent = 'Save!';
    saveButton.onclick = () => {
        persistSongs(rankedSongs)
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
                const songs = await search_response.json();
                console.log('Songs:', songs);
                displaySongs(songs, songRank);
            }
        } catch (error) {
            console.error('Error adding song:', error);
            alert('Error adding song.');
        }
    });
}

function displaySongs(songs, songRank) {
    const songListContainer = document.getElementById('right');
    songListContainer.innerHTML = '';  // Clear the list first

    const goBackButtonArea = document.createElement('div');
    goBackButtonArea.classList.add("go-back-button");

    const goBackButton = document.createElement('button');
    goBackButton.textContent = 'Go Back!';
    goBackButton.onclick = () => {
        displayAddSongForm()
    }

    goBackButtonArea.appendChild(goBackButton)
    songListContainer.appendChild(goBackButtonArea);

    songs.forEach((song) => {
        song.rank = Number(songRank);
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

        const albumArt = document.createElement('img');
        albumArt.classList.add('album-art');
        albumArt.src = song.album_cover_url;
        albumArt.alt = `${song.artist} - Album Art`;
        albumArt.height = 200;
        albumArt.width = 200;

        const rankButton = document.createElement('button');
        rankButton.textContent = 'Add to List';
        rankButton.onclick = () => {
            if (songKey.has(song.name+song.artist)) {
                alert('Duplicate song try again!');
            } else {
                if(rankedSongs.has(song.rank)) {
                    let songRemove = rankedSongs.get(song.rank)
                    songKey.delete(songRemove.name+songRemove.artist)
                }
                rankedSongs.set(song.rank, song);
                songKey.add(song.name+song.artist);
            }
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

async function persistSongs(songs){

    if(songs.length < 10) {
        alert('Ensure you have 10 songs added!');
        return;
    }

    try {
        const response = await fetch(`/songs`, {
            method: 'POST',
            body: JSON.stringify(Array.from(songs.values())), // Convert songs to JSON string
            headers: {
                'Content-Type': 'application/json' // Specify content type as JSON
            }
        });

        if (response.ok) {
            alert('Wooohooo! Successfully saved songs!');
        } else {
            alert('Failed to persist songs please try again.');
        }
    } catch (error) {
        console.error('Error adding song:', error);
        alert('Error adding song.');
    }
}