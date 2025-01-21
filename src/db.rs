use sqlx_postgres::PgPool;
use crate::routes;

async fn get_config_value(pool: &PgPool, key: &str) -> Result<Option<String>, String> {
    let row = sqlx::query!(
        "SELECT value FROM config WHERE key = $1",
        key
    )
        .fetch_optional(pool)
        .await.unwrap();

    Ok(row.map(|r| r.value))
}

pub async fn insert_user(pool: &PgPool, name: &str) -> Result<i32, sqlx::Error> {
    // Insert the user and return the generated ID
    let row = sqlx::query!(
        "INSERT INTO users (name) VALUES ($1) RETURNING id",
        name
    )
        .fetch_one(pool)
        .await?;

    Ok(row.id)
}

pub async fn user_exists(pool: &PgPool, name: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "SELECT EXISTS (SELECT 1 FROM users WHERE name = $1) AS exists",
        name
    )
        .fetch_one(pool)
        .await?;

    Ok(result.exists.unwrap_or(false))
}

pub async fn insert_or_update_ranking(
    pool: &PgPool,
    user_id: i32,
    rank: i32,
    song: &routes::Song,
) -> Result<(), sqlx::Error> {
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
        song.image_url
    )
        .fetch_one(pool)
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
        rank
    )
        .execute(pool)
        .await?;

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