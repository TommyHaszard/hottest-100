use sqlx::{FromRow, Transaction};
use sqlx_postgres::{PgPool, Postgres};
use crate::api::types::Song;

#[derive(FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
}

async fn get_config_value(pool: &PgPool, key: &str) -> Result<Option<String>, String> {
    let row = sqlx::query!(
        "SELECT value FROM config WHERE key = $1",
        key
    )
        .fetch_optional(pool)
        .await.unwrap();

    Ok(row.map(|r| r.value))
}

pub async fn get_or_insert_user(pool: &PgPool, name: &str) -> Result<User, sqlx::Error> {
    // Try to find the user first
    if let Some(user) = get_user(pool, name).await? {
        return Ok(user); // Return the existing user
    }

    // If the user doesn't exist, insert the user and return the new user
    let row = sqlx::query!(
        "INSERT INTO users (name) VALUES ($1) RETURNING id, name",
        name
    )
        .fetch_one(pool)
        .await?;

    // Return the newly inserted user
    Ok(User {
        id: row.id,
        name: row.name,
    })
}

pub async fn get_user(pool: &PgPool, name: &str) -> Result<Option<User>, sqlx::Error> {
    // Check if the user already exists
    let row = sqlx::query_as!(
        User,
        "SELECT id, name FROM users WHERE name = $1",
        name
    )
        .fetch_optional(pool)
        .await?;

    Ok(row) // Return Option<User>: Some(user) if found, None if not
}

pub async fn insert_or_update_songs(
    pool: &PgPool,
    user_id: &i32,
    songs: &Vec<Song>,
) -> Result<(), sqlx::Error> {
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    for (song) in songs {
        // Ensure the song exists in the database, insert it if not
        let song_id = sqlx::query!(
            r#"
            INSERT INTO songs (name, artist, uri, album_cover_url)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (name, artist) DO UPDATE SET
                uri = EXCLUDED.uri,
                album_cover_url = EXCLUDED.album_cover_url
            RETURNING id
            "#,
            song.name,
            song.artist,
            song.uri,
            song.album_cover_url
        )
            .fetch_one(&mut *tx) // Use the transaction instead of the pool
            .await?
            .id;

        // Insert or update the user's ranking for the song
        sqlx::query!(
            r#"
            INSERT INTO rankings (user_id, song_id, rank)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, rank) DO UPDATE SET
                song_id = EXCLUDED.song_id
            "#,
            user_id,
            song_id,
            song.rank.unwrap()
        )
            .execute(&mut *tx) // Use the transaction instead of the pool
            .await?;
    }

    // Commit the transaction
    tx.commit().await?;

    Ok(())
}


pub async fn get_client_id(pool: &PgPool) -> Result<Option<String>, String> {
    get_config_value(pool, "CLIENT_ID").await
}

pub async fn get_client_secret(pool: &PgPool) -> Result<Option<String>, String> {
    get_config_value(pool, "CLIENT_SECRET").await
}

pub async fn get_redirect_uri(pool: &PgPool) -> Result<Option<String>, String> {
    get_config_value(pool, "REDIRECT_URI").await
}

#[derive(sqlx::FromRow)]
struct SongRow {
    id: i32,
    name: String,
    uri: String,
    artist: String,
    album_cover_url: String,
    rank: Option<i32>,
}


pub async fn get_songs_for_user_name(pool: &PgPool, name: &String) -> Result<Vec<Song>, sqlx::Error>{
    // Check if the user already exists
    let rows = sqlx::query_as!(
        SongRow,
        r#"
            SELECT songs.*, rankings.rank FROM songs
            JOIN rankings ON songs.id = rankings.song_id
            JOIN users ON rankings.user_id = users.id
            WHERE users."name" = $1
        "#,
        name
    ).fetch_all(pool).await?;

    // Convert the results from SongRow to Song, setting `key` to None
    let songs: Vec<Song> = rows.into_iter().map(|row| Song {
        key: Some(format!("{}{}", row.name, row.artist)), // Set key to None
        name: row.name,
        uri: row.uri,
        artist: row.artist,
        album_cover_url: row.album_cover_url,
        rank: row.rank,
    }).collect();

    Ok(songs)
}

#[derive(sqlx::FromRow, Debug)]
struct Uri {
    uri: String,
}

pub async fn get_song_rankings(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Uri,
        r#"
        SELECT
            s.uri as URI
        FROM
            songs s
        LEFT JOIN
            rankings r ON s.id = r.song_id
        GROUP BY
            s.id, s.name
        ORDER BY
            COUNT(r.user_id) + COALESCE(0.15 * (11-AVG(r.rank)), 0) DESC, s.name
        "#,
    ).fetch_all(pool).await?;

    Ok(rows.into_iter().map(|song| song.uri).collect())

}