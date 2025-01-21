use std::collections::HashSet;
use rocket::tokio::task;
use rocket::response::Redirect;
use rocket::serde::{json::Json, Deserialize};
use reqwest::{Client, Error, Response};
use rocket::State;
use std::env;
use std::path::{Path, PathBuf};
use rocket::fs::NamedFile;
use rocket::http::{Cookie, CookieJar, Status};
use serde::Serialize;
use crate::{db, DB_POOL};

static SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
static SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";

#[derive(Deserialize)]
struct AccessTokenResponse {
    access_token: String,
}

#[get("/")]
pub async fn index() -> Redirect {
    let db_pool = DB_POOL.get().unwrap();
    let client_id = db::get_client_id(db_pool).await.expect("Failed to get client id");
    if client_id.is_none() {
        Redirect::to("/fail");
    }

    let redirect_uri = db::get_redirect_uri(db_pool).await.expect("Failed to get client id");
    if redirect_uri.is_none() {
        Redirect::to("/fail");
    }

    let auth_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope=playlist-modify-public%20user-read-private&state=random_state_string",
        SPOTIFY_AUTH_URL,
        client_id.unwrap(),
        urlencoding::encode(&redirect_uri.unwrap())
    );
    Redirect::to(auth_url)
}

#[get("/callback?<code>")]
pub async fn callback(cookies: &CookieJar<'_>, code: String) -> Redirect {
    let db_pool = DB_POOL.get().unwrap();
    let client_id = db::get_client_id(db_pool).await.expect("Failed to get client id");
    if client_id.is_none() {
        Redirect::to("/fail");
    }

    let client_secret = db::get_client_secret(db_pool).await.expect("Failed to get client id");
    if client_secret.is_none() {
        Redirect::to("/fail");
    }

    let redirect_uri = db::get_redirect_uri(db_pool).await.expect("Failed to get client id");
    if redirect_uri.is_none() {
        Redirect::to("/fail");
    }


    let client = Client::new();
    let response = client
        .post(SPOTIFY_TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect_uri.unwrap()),
            ("client_id", &client_id.unwrap()),
            ("client_secret", &client_secret.unwrap()),
        ])
        .send()
        .await
        .expect("Failed to get access token");

    let data: AccessTokenResponse = response
        .json()
        .await
        .expect("Failed to parse token response");

    let profile_url =
        "https://api.spotify.com/v1/me";

    let response = client
        .get(profile_url)
        .header("Authorization", format!("Bearer {}", data.access_token.clone()))
        .send()
        .await.expect("Failed to parse token response");

    match response {
        res if res.status().is_success() => {
            let data = &res.json::<serde_json::Value>().await.expect("Failed to get access token");
            rocket::info!("Data {:#?}", data);
            cookies.add_private(Cookie::new("user", data["uri"].to_string()));
        }
        res => {
            // Handle other HTTP statuses
            println!("Response was not successful: {:?}:{:?}", res.status(), res.json::<serde_json::Value>().await.expect("Failed to get error msg"));
        }
    }


    cookies.add_private(Cookie::new("api_token", data.access_token));

    Redirect::to("/main")
}

#[get("/main")]
pub async fn main_page(cookies: &CookieJar<'_>) -> NamedFile {
    let token = cookies.get_private("api_token").map(|cookie| cookie.value().to_string());
    if token.is_none() {

        Redirect::to("/");
    }
    let mut file_path = PathBuf::from("static");
    file_path.push("index.html");

    NamedFile::open(Path::new("static/index.html")).await.unwrap()

}

#[get("/<file..>")]
pub async fn files(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static").join(file)).await.ok()
}

// Struct to parse the query parameters
#[derive(FromForm)]
struct SearchQuery {
    track: Option<String>,
    rank: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Song {
    pub key: String,
    pub name: String,
    pub uri: String,
    pub artist: String,
    pub image_url: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[get("/search-songs?<query..>")]
pub async fn search_songs(
    cookies: &CookieJar<'_>,
    query: Option<SearchQuery>,
    client: &State<Client>,
) -> Result<Json<Vec<Song>>, (Status, Json<ErrorResponse>)> {
    let query = query.unwrap();

    let token = cookies.get_private("api_token").map(|cookie| cookie.value().to_string());

    if token.is_none() {
        return Err((
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "No Token".to_string(),
            }),
        ));
    }

    let access_token = token.unwrap();

    // Extract the track name from the query
    let track_name = query.track.unwrap();
    let spotify_url = format!(
        "https://api.spotify.com/v1/search?q={}&type=track&limit=10",
        track_name
    );
    let response = client
        .get(&spotify_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;

    match response {
        Ok(res) if res.status().is_success() => {
            let data = res.json::<serde_json::Value>().await.map_err(|err| {
                (
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error: format!("Failed to parse Spotify API response: {}", err),
                    }),
                )
            })?;

            let items = data["tracks"]["items"].as_array();

            let mut seen_keys = HashSet::new();
            let songs: Vec<Song> = items.unwrap()
                .to_vec()
                .into_iter()
                .filter_map(|item| {
                    let name = item["name"].as_str()?.to_string();
                    let artist = item["artists"][0]["name"].as_str()?.to_string();
                    let key = format!("{}{}", name, artist);

                    // Skip if the key is a duplicate
                    if !seen_keys.insert(key.clone()) {
                        return None;
                    }

                    Some(Song {
                        key,
                        name,
                        artist,
                        uri: item["uri"].as_str().unwrap_or_default().to_string(),
                        image_url: item["album"]["images"][1]["url"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                    })
                })
                .collect();


            rocket::info!("Tracks {:#?}", songs);

            Ok(Json(songs))
        }
        Ok(res) => {
            let error_text = res.text().await.unwrap_or_default();
            Err((
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: format!("Spotify API error: {}", error_text),
                }),
            ))
        }
        Err(err) => Err((
            Status::InternalServerError,
            Json(ErrorResponse {
                error: format!("Failed to call Spotify API: {}", err),
            }),
        )),
    }
}