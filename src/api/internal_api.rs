use crate::api::db;
use crate::api::external_api::{
    add_songs_to_playlist, authenticate, create_playlist, search_spotify_songs,
};
use crate::api::types::{ErrorResponse, MusicTasteOverview, SearchSongsQuery, Song};
use crate::DB_POOL;
use reqwest::Client;
use rocket::fs::NamedFile;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use std::path::{Path, PathBuf};

#[get("/")]
pub async fn index(cookies: &CookieJar<'_>) -> Result<Redirect, Redirect> {
    if cookies.get_private("api_token").is_some() {
        return Ok(Redirect::to("/main"));
    }
    Ok(Redirect::to("/login"))
}


#[get("/login")]
pub async fn login_page() -> Redirect {
    authenticate().await
}

#[get("/main")]
pub async fn main_page(cookies: &CookieJar<'_>) -> Result<NamedFile, Redirect> {
    let token = cookies.get_private("api_token");
    rocket::info!("Cookie check - api_token present: {}", token.is_some());
    
    let mut file_path = PathBuf::from("static");
    file_path.push("index.html");
    NamedFile::open(file_path)
        .await
        .map_err(|_| Redirect::to("/fail"))
}

#[get("/<file..>")]
pub async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static").join(file)).await.ok()
}

#[post("/songs", format = "json", data = "<songs>")]
pub async fn save_songs(
    cookies: &CookieJar<'_>,
    songs: Json<Vec<Song>>,
) -> Result<(), (Status, Json<ErrorResponse>)> {
    let db_pool = DB_POOL.get().unwrap();
    let user_name_opt = cookies
        .get_private("user")
        .map(|cookie| cookie.value().to_string());

    if user_name_opt.is_none() {
        return Err((
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "No username".to_string(),
            }),
        ));
    }

    let user_name = user_name_opt.unwrap();
    let user = db::get_or_insert_user(db_pool, &user_name)
        .await
        .map_err(|err| {
            (
                Default::default(),
                Json(ErrorResponse {
                    error: format!("Database error: {}", err),
                }),
            )
        })?;

    db::insert_or_update_songs(db_pool, &user.id, &songs)
        .await
        .map_err(|err| {
            (
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: format!("Failed to insert or update the list of songs: {}", err),
                }),
            )
        })?;

    // add songs
    Ok(())
}

#[get("/songs")]
pub async fn get_songs(
    cookies: &CookieJar<'_>,
) -> Result<Json<Vec<Song>>, (Status, Json<ErrorResponse>)> {
    let db_pool = DB_POOL.get().unwrap();
    let user_name_opt = cookies
        .get_private("user")
        .map(|cookie| cookie.value().to_string());

    if user_name_opt.is_none() {
        return Err((
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "No Username Token".to_string(),
            }),
        ));
    }

    let user_name = user_name_opt.unwrap();
    let songs = db::get_songs_for_user_name(db_pool, &user_name)
        .await
        .map_err(|err| {
            (
                Default::default(),
                Json(ErrorResponse {
                    error: format!("Database error: {}", err),
                }),
            )
        })?;

    rocket::info!("Tracks {:#?}", songs);

    Ok(Json(songs))
}

#[get("/search-songs?<query..>")]
pub async fn search_songs(
    cookies: &CookieJar<'_>,
    query: Option<SearchSongsQuery>,
    client: &State<Client>,
) -> Result<Json<Vec<Song>>, (Status, Json<ErrorResponse>)> {
    search_spotify_songs(cookies, query, client).await
}

#[get("/generate_playlist")]
pub async fn generate_playlist(
    cookies: &CookieJar<'_>,
    client: &State<Client>,
) -> Result<(), (Status, Json<ErrorResponse>)> {
    let db_pool = DB_POOL.get().unwrap();

    let ranked_songs = db::get_song_rankings(db_pool).await.map_err(|err| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: format!("Failed to get the top songs: {}", err),
            }),
        )
    })?;

    let playlist_id = create_playlist(cookies, client).await.map_err(|err| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: format!("Failed to create Playlist via Spotify API: {:?}", err.1),
            }),
        )
    })?;

    // pass the playlist id into the external function with the songs to make the playlist
    add_songs_to_playlist(playlist_id, ranked_songs, &cookies, &client).await
}

#[get("/music-taste")]
pub async fn get_music_taste() -> Result<Json<Vec<MusicTasteOverview>>, (Status, Json<ErrorResponse>)> {
    let db_pool = DB_POOL.get().unwrap();

    let overview = db::get_music_taste_overview(db_pool)
        .await
        .map_err(|err| {
            (
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: format!("Failed to get music taste overview: {}", err),
                }),
            )
        })?;

    Ok(Json(overview))
}
