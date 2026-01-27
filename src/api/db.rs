use crate::api::types::{MusicTasteOverview, Song};
use sqlx::{FromRow, Transaction};
use sqlx_postgres::{PgPool, Postgres};
use std::option::Option;

#[derive(FromRow)]
pub struct User {
    pub id: i32,
    pub name: String,
}

#[derive(FromRow)]
pub struct MusicTasteIndividual {
    pub other_user_name: String,
    pub overlapping_songs: Option<i64>,
    pub song_rank_diff: Option<f64>,
    pub song_relationship_strength: Option<f64>,
    pub overlapping_artists: Option<i64>,
    pub total_songs_shared_artists: Option<i64>,
    pub artist_rank_diff: Option<f64>,
    pub combined_score: Option<f64>,
    pub overlapping_song_details: Option<serde_json::Value>,
    pub overlapping_artist_details: Option<serde_json::Value>,
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
    let row = sqlx::query_as!(User, "SELECT id, name FROM users WHERE name = $1", name)
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

#[derive(sqlx::FromRow)]
struct SongRow {
    id: i32,
    name: String,
    uri: String,
    artist: String,
    album_cover_url: String,
    rank: Option<i32>,
}

pub async fn get_songs_for_user_name(
    pool: &PgPool,
    name: &String,
) -> Result<Vec<Song>, sqlx::Error> {
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
    )
    .fetch_all(pool)
    .await?;

    // Convert the results from SongRow to Song, setting `key` to None
    let songs: Vec<Song> = rows
        .into_iter()
        .map(|row| Song {
            key: Some(format!("{}{}", row.name, row.artist)), // Set key to None
            name: row.name,
            uri: row.uri,
            artist: row.artist,
            album_cover_url: row.album_cover_url,
            rank: row.rank,
        })
        .collect();

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
            COUNT(r.user_id) + COALESCE(0.15 * (11-AVG(r.rank)), 0) ASC, s.name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(|song| song.uri).collect())
}

pub async fn get_music_taste_overview(
    pool: &PgPool,
) -> Result<Vec<MusicTasteOverview>, sqlx::Error> {
    let rows = sqlx::query_as!(
        MusicTasteOverview,
        r#"
        WITH user_pairs AS (
    SELECT 
        r1.user_id AS user1_id,
        r2.user_id AS user2_id,
        r1.song_id,
        r1.rank AS user1_rank,
        r2.rank AS user2_rank,
        ABS(r1.rank - r2.rank) AS rank_difference,
        s.artist
    FROM rankings r1
    JOIN rankings r2 
        ON r1.song_id = r2.song_id 
        AND r1.user_id < r2.user_id
    JOIN songs s ON r1.song_id = s.id
),
song_overlap AS (
    SELECT 
        user1_id,
        user2_id,
        COUNT(*) AS overlapping_songs,
        COUNT(DISTINCT artist) AS artists_in_overlap,
        AVG(rank_difference) AS avg_rank_difference,
        COUNT(*) * 10.0 - AVG(rank_difference) AS song_relationship_strength
    FROM user_pairs
    GROUP BY user1_id, user2_id
),
artist_overlap AS (
    SELECT 
        r1.user_id AS user1_id,
        r2.user_id AS user2_id,
        COUNT(DISTINCT s1.artist) AS shared_artists,
        COUNT(*) AS total_artist_overlaps,
        AVG(ABS(r1.rank - r2.rank)) AS avg_artist_rank_diff
    FROM rankings r1
    JOIN rankings r2 ON r1.user_id < r2.user_id
    JOIN songs s1 ON r1.song_id = s1.id
    JOIN songs s2 ON r2.song_id = s2.id
    WHERE s1.artist = s2.artist
    GROUP BY r1.user_id, r2.user_id
),
overlapping_song_details AS (
    -- Get the song details for each pair with full information
    SELECT 
        up.user1_id,
        up.user2_id,
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'song_name', s.name,
                'artist', s.artist,
                'user1_rank', up.user1_rank,
                'user2_rank', up.user2_rank,
                'rank_difference', up.rank_difference
            ) ORDER BY up.rank_difference ASC, up.user1_rank ASC
        ) AS songs
    FROM user_pairs up
    JOIN songs s ON up.song_id = s.id
    GROUP BY up.user1_id, up.user2_id
),
artist_detail_pairs AS (
    -- Get all song pairs by the same artist for each user pair
    SELECT 
        r1.user_id AS user1_id,
        r2.user_id AS user2_id,
        s1.artist,
        s1.name AS user1_song,
        r1.rank AS user1_rank,
        s2.name AS user2_song,
        r2.rank AS user2_rank,
        ABS(r1.rank - r2.rank) AS rank_difference
    FROM rankings r1
    JOIN rankings r2 ON r1.user_id < r2.user_id
    JOIN songs s1 ON r1.song_id = s1.id
    JOIN songs s2 ON r2.song_id = s2.id
    WHERE s1.artist = s2.artist
),
artist_overlap_details AS (
    -- Aggregate artist details with all song combinations
    SELECT 
        user1_id,
        user2_id,
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'artist', artist,
                'user1_song', user1_song,
                'user1_rank', user1_rank,
                'user2_song', user2_song,
                'user2_rank', user2_rank,
                'rank_difference', rank_difference
            ) ORDER BY rank_difference ASC, user1_rank ASC
        ) AS artist_details
    FROM artist_detail_pairs
    GROUP BY user1_id, user2_id
),
combined_metrics AS (
    SELECT 
        COALESCE(so.user1_id, ao.user1_id) AS user1_id,
        COALESCE(so.user2_id, ao.user2_id) AS user2_id,
        -- Song metrics
        COALESCE(so.overlapping_songs, 0) AS overlapping_songs,
        COALESCE(so.avg_rank_difference, 0) AS avg_song_rank_diff,
        COALESCE(so.song_relationship_strength, 0) AS song_strength,
        -- Artist metrics
        COALESCE(ao.shared_artists, 0) AS shared_artists,
        COALESCE(ao.total_artist_overlaps, 0) AS artist_song_overlaps,
        COALESCE(ao.avg_artist_rank_diff, 0) AS avg_artist_rank_diff,
        -- Combined compatibility score
        COALESCE(so.song_relationship_strength, 0) + 
        (COALESCE(ao.shared_artists, 0) * 3.0) - 
        COALESCE(ao.avg_artist_rank_diff, 0) * 0.5 AS combined_compatibility_score
    FROM song_overlap so
    FULL OUTER JOIN artist_overlap ao
        ON so.user1_id = ao.user1_id 
        AND so.user2_id = ao.user2_id
    WHERE COALESCE(so.overlapping_songs, 0) > 0 
       OR COALESCE(ao.shared_artists, 0) > 0
)
SELECT 
    u1.display_name AS user_1,
    u2.display_name AS user_2,
    cm.overlapping_songs AS overlapping_songs,
    CAST(ROUND(cm.avg_song_rank_diff, 2) AS DOUBLE PRECISION) AS song_rank_diff,
    CAST(ROUND(cm.song_strength, 2) AS DOUBLE PRECISION) AS song_relationship_strength,
    cm.shared_artists AS overlapping_artists,
    cm.artist_song_overlaps AS total_songs_shared_artists,
    CAST(ROUND(cm.avg_artist_rank_diff, 2) AS DOUBLE PRECISION) AS artist_rank_diff,
    CAST(ROUND(cm.combined_compatibility_score, 2) AS DOUBLE PRECISION) AS combined_score,
    -- Detailed JSON for HTML input
    COALESCE(osd.songs, '[]'::json) AS overlapping_song_details,
    COALESCE(aod.artist_details, '[]'::json) AS overlapping_artist_details
FROM combined_metrics cm
JOIN users u1 ON cm.user1_id = u1.id
JOIN users u2 ON cm.user2_id = u2.id
LEFT JOIN overlapping_song_details osd
    ON cm.user1_id = osd.user1_id
    AND cm.user2_id = osd.user2_id
LEFT JOIN artist_overlap_details aod
    ON cm.user1_id = aod.user1_id
    AND cm.user2_id = aod.user2_id
ORDER BY 
    cm.combined_compatibility_score DESC,
    cm.overlapping_songs DESC,
    cm.shared_artists DESC
LIMIT 5;
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn get_music_taste_user(pool: &PgPool, active_user_id: &i32) -> Result<Vec<MusicTasteIndividual>, sqlx::Error> {

    let rows = sqlx::query_as!(
        MusicTasteIndividual,
        r#"
        WITH active_user_songs AS (
    SELECT song_id, rank
    FROM rankings
    WHERE user_id = $1
),
other_users_songs AS (
    SELECT user_id, song_id, rank
    FROM rankings
    WHERE user_id != $1
),
song_overlap AS (
    SELECT 
        ous.user_id AS other_user_id,
        COUNT(*) AS overlapping_songs,
        COUNT(DISTINCT s.artist) AS artists_in_overlap,
        AVG(ABS(aus.rank - ous.rank)) AS avg_rank_difference,
        COUNT(*) * 10.0 - AVG(ABS(aus.rank - ous.rank)) AS song_relationship_strength
    FROM active_user_songs aus
    JOIN other_users_songs ous ON aus.song_id = ous.song_id
    JOIN songs s ON aus.song_id = s.id
    GROUP BY ous.user_id
),
active_user_artists AS (
    SELECT DISTINCT s.artist, r.rank, s.name as song_name
    FROM rankings r
    JOIN songs s ON r.song_id = s.id
    WHERE r.user_id = $1
),
other_users_artists AS (
    SELECT r.user_id, s.artist, r.rank, s.name as song_name
    FROM rankings r
    JOIN songs s ON r.song_id = s.id
    WHERE r.user_id != $1
),
artist_overlap AS (
    SELECT 
        oua.user_id AS other_user_id,
        COUNT(DISTINCT aua.artist) AS shared_artists,
        COUNT(*) AS total_artist_overlaps,
        AVG(ABS(aua.rank - oua.rank)) AS avg_artist_rank_diff
    FROM active_user_artists aua
    JOIN other_users_artists oua ON aua.artist = oua.artist
    GROUP BY oua.user_id
),
overlapping_song_details AS (
    SELECT 
        ous.user_id AS other_user_id,
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'song_name', s.name,
                'artist', s.artist,
                'active_user_rank', aus.rank,
                'other_user_rank', ous.rank,
                'rank_difference', ABS(aus.rank - ous.rank)
            ) ORDER BY ABS(aus.rank - ous.rank) ASC, aus.rank ASC
        ) AS songs
    FROM active_user_songs aus
    JOIN other_users_songs ous ON aus.song_id = ous.song_id
    JOIN songs s ON aus.song_id = s.id
    GROUP BY ous.user_id
),
artist_overlap_details AS (
    SELECT 
        oua.user_id AS other_user_id,
        JSON_AGG(
            JSON_BUILD_OBJECT(
                'artist', aua.artist,
                'active_user_song', aua.song_name,
                'active_user_rank', aua.rank,
                'other_user_song', oua.song_name,
                'other_user_rank', oua.rank,
                'rank_difference', ABS(aua.rank - oua.rank)
            ) ORDER BY ABS(aua.rank - oua.rank) ASC, aua.rank ASC
        ) AS artist_details
    FROM active_user_artists aua
    JOIN other_users_artists oua ON aua.artist = oua.artist
    GROUP BY oua.user_id
),
combined_metrics AS (
    SELECT 
        COALESCE(so.other_user_id, ao.other_user_id) AS other_user_id,
        COALESCE(so.overlapping_songs, 0) AS overlapping_songs,
        COALESCE(so.avg_rank_difference, 0) AS avg_song_rank_diff,
        COALESCE(so.song_relationship_strength, 0) AS song_strength,
        COALESCE(ao.shared_artists, 0) AS shared_artists,
        COALESCE(ao.total_artist_overlaps, 0) AS artist_song_overlaps,
        COALESCE(ao.avg_artist_rank_diff, 0) AS avg_artist_rank_diff,
        COALESCE(so.song_relationship_strength, 0) + 
        (COALESCE(ao.shared_artists, 0) * 3.0) - 
        COALESCE(ao.avg_artist_rank_diff, 0) * 0.5 AS combined_compatibility_score
    FROM song_overlap so
    FULL OUTER JOIN artist_overlap ao ON so.other_user_id = ao.other_user_id
    WHERE COALESCE(so.overlapping_songs, 0) > 0 
       OR COALESCE(ao.shared_artists, 0) > 0
)
SELECT 
    u.name AS other_user_name,
    cm.overlapping_songs,
    CAST(ROUND(cm.avg_song_rank_diff, 2) AS DOUBLE PRECISION) AS song_rank_diff,
    CAST(ROUND(cm.song_strength, 2) AS DOUBLE PRECISION) AS song_relationship_strength,
    cm.shared_artists AS overlapping_artists,
    cm.artist_song_overlaps AS total_songs_shared_artists,
    CAST(ROUND(cm.avg_artist_rank_diff, 2) AS DOUBLE PRECISION) AS artist_rank_diff,
    CAST(ROUND(cm.combined_compatibility_score, 2) AS DOUBLE PRECISION) AS combined_score,
    COALESCE(osd.songs, '[]'::json) AS overlapping_song_details,
    COALESCE(aod.artist_details, '[]'::json) AS overlapping_artist_details
FROM combined_metrics cm
JOIN users u ON cm.other_user_id = u.id
LEFT JOIN overlapping_song_details osd ON cm.other_user_id = osd.other_user_id
LEFT JOIN artist_overlap_details aod ON cm.other_user_id = aod.other_user_id
ORDER BY 
    cm.combined_compatibility_score DESC,
    cm.overlapping_songs DESC,
    cm.shared_artists DESC
        "#,
active_user_id
    ).fetch_all(pool).await?;

    Ok(rows)
}
