use std::collections::HashSet;
use std::env;
use reqwest::Client;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::Redirect;
use rocket::serde::json::Json;
use rocket::State;
use rocket::time::Duration;
use crate::DB_POOL;
use crate::api::db;
use crate::api::types::{AccessTokenResponse, CreatePlaylistBody, CreatePlaylistId, ErrorResponse, SearchSongsQuery, Song};

static SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
static SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";

#[get("/")]
pub async fn index() -> Redirect {
    let db_pool = DB_POOL.get().unwrap();
    let client_id = db::get_client_id(db_pool).await.expect("Failed to get client id");
    if client_id.is_none() {
        Redirect::to("/fail");
    }

    let redirect_uri = env::var("SPOTIFY_REDIRECT_URI").expect("SPOTIFY_REDIRECT_URI must be set");

    let auth_url = format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&scope=playlist-modify-public%20user-read-private&state=random_state_string",
        SPOTIFY_AUTH_URL,
        client_id.unwrap(),
        urlencoding::encode(&redirect_uri)
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

    let client_secret = db::get_client_secret(db_pool).await.expect("publicled to get client id");
    if client_secret.is_none() {
        Redirect::to("/fail");
    }

    let redirect_uri = std::env::var("SPOTIFY_REDIRECT_URI").expect("SPOTIFY_REDIRECT_URI must be set");

    let client = Client::new();
    let response = client
        .post(SPOTIFY_TOKEN_URL)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", &redirect_uri),
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

    let profile_url = "https://api.spotify.com/v1/me";

    let response = client
        .get(profile_url)
        .header("Authorization", format!("Bearer {}", data.access_token.clone()))
        .send()
        .await.expect("Failed to parse token response");

    match response {
        res if res.status().is_success() => {
            let data = &res.json::<serde_json::Value>().await.expect("Failed to get access token");
            rocket::info!("Data {:#?}", data);
            cookies.add_private(
                Cookie::build(("user", data["uri"].to_string()))
                    .http_only(true)
                    .secure(true)
                    .max_age(Duration::minutes(60)))
        }
        res => {
            // Handle other HTTP statuses
            rocket::error!("Response was not successful: {:?}", res.status());
        }
    }


    cookies.add_private(
        Cookie::build(("api_token", data.access_token))
            .http_only(true)
            .secure(true)
            .max_age(Duration::minutes(60)));

    Redirect::to("/main")
}

pub async fn create_playlist(cookies: &CookieJar<'_>,
                             client: &State<Client>) -> Result<CreatePlaylistId, (Status, Json<ErrorResponse>)> {
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

    let split_name = user_name.split(':').nth(2).ok_or_else(|| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "Failed to parse Username to parse into Spotify API".to_string(),
            }),
        )
    })?;

    let access_token_opt = cookies.get_private("api_token").map(|cookie| cookie.value().to_string());
    if access_token_opt.is_none() {
        return Err((
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "No Token".to_string(),
            }),
        ));
    }

    let access_token = access_token_opt.unwrap();

    let json_body = serde_json::to_string(&CreatePlaylistBody {
        name: "Hottest100".to_string(),
        description: "Hottest100".to_string(),
        public: true,
    }).map_err(|err| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: format!("Failed to parse Username to parse into Spotify API: {}", err),
            }),
        )
    })?;

    let create_spotify_playlist = format!(
        "https://api.spotify.com/v1/users/{}/playlists",
        split_name
    );

    let response = client
        .post(&create_spotify_playlist)
        .body(json_body)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await.map_err(|err| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: format!("Failed to create Playlist via Spotify API: {}", err),
            }),
        )
    })?;

    if response.status().is_success() {
        if let Ok(data) = response.json::<serde_json::Value>().await {
            let id = &data["id"];
            // Process the `id` here
            Ok(CreatePlaylistId { id: id.to_string() })
        } else {
            Err((
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: "Failed to parse Spotify API response".to_string(),
                }),
            ))
        }
    } else {
        Err((
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "Failed to parse Spotify API response".to_string(),
            }),
        ))
    }
}

#[get("/search-songs?<query..>")]
pub async fn search_songs(
    cookies: &CookieJar<'_>,
    query: Option<SearchSongsQuery>,
    client: &State<Client>,
) -> Result<Json<Vec<Song>>, (Status, Json<ErrorResponse>)> {
    let query = query.unwrap();

    let token = cookies.get_private("api_token").map(|cookie| cookie.value().to_string());

    if token.is_none() {
        index().await;
        let token2 = cookies.get_private("api_token").map(|cookie| cookie.value().to_string());
        if token2.is_none() {
            return Err((
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: "No Token".to_string(),
                }),
            ));
        }
    }

    let access_token = token.unwrap();

    // Extract the track name from the query
    let track_name = query.track.unwrap();
    let rank = query.rank.unwrap();

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
                        key: Some(key),
                        name,
                        artist,
                        uri: item["uri"].as_str().unwrap_or_default().to_string(),
                        album_cover_url: item["album"]["images"][1]["url"]
                            .as_str()
                            .unwrap_or_default()
                            .to_string(),
                        rank: Some(rank),
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