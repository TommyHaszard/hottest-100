CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) unique NOT NULL
);

CREATE TABLE songs (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    artist VARCHAR(255) NOT NULL,
    uri VARCHAR(255) UNIQUE NOT NULL,
    album_cover_url TEXT NOT NULL
);

CREATE TABLE rankings (
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    song_id INT REFERENCES songs(id) ON DELETE CASCADE,
    rank INT CHECK (rank >= 1 AND rank <= 10),
    PRIMARY KEY (user_id, song_id)
);

ALTER TABLE songs ADD CONSTRAINT unique_name_artist UNIQUE (name, artist);
ALTER TABLE rankings ADD CONSTRAINT unique_user_rank UNIQUE (user_id, rank);

