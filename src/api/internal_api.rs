use std::collections::HashSet;
use std::path::{Path, PathBuf};
use reqwest::Client;
use rocket::fs::NamedFile;
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use serde::Serialize;
use crate::api::db;
use crate::api::external_api::create_playlist;
use crate::api::types::{ErrorResponse, Song};
use crate::DB_POOL;
#[get("/main")]
pub async fn main_page(cookies: &CookieJar<'_>) -> Result<NamedFile, Redirect> {
    if let Some(token_cookie) = cookies.get_private("api_token") {
        let token = token_cookie.value();
        let mut file_path = PathBuf::from("static");
        file_path.push("index.html");
        return NamedFile::open(file_path).await.map_err(|_| Redirect::to("/"));
    }

    Err(Redirect::to("/"))
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
    let user_name_opt = cookies.get_private("user").map(|cookie| cookie.value().to_string());

    if user_name_opt.is_none() {
        return Err((
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "No username".to_string(),
            }),
        ));
    }

    let user_name = user_name_opt.unwrap();
    let user = db::get_or_insert_user(db_pool, &user_name).await.map_err(|err| {
        (
            Default::default(),
            Json(ErrorResponse {
                error: format!("Database error: {}", err),
            }),
        )
    })?;

    db::insert_or_update_songs(db_pool, &user.id, &songs).await.map_err(|err| {
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
    let user_name_opt = cookies.get_private("user").map(|cookie| cookie.value().to_string());

    if user_name_opt.is_none() {
        return Err((
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "No Token".to_string(),
            }),
        ));
    }

    let user_name = user_name_opt.unwrap();
    let songs = db::get_songs_for_user_name(db_pool, &user_name).await.map_err(|err| {
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

#[get("/generate_playlist")]
pub async fn generate_playlist(cookies: &CookieJar<'_>,
                               client: &State<Client>
) -> Result<(), (Status, Json<ErrorResponse>)> {
    let db_pool = DB_POOL.get().unwrap();

    let songs = db::get_song_rankings(db_pool).await.map_err(|err| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: format!("Failed to parse Spotify API response: {}", err),
            }),
        )
    })?;

    create_playlist(cookies, client).await;
    todo!()
}
