use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct AccessTokenResponse {
    pub(crate) access_token: String,
}

// Struct to parse the query parameters
#[derive(FromForm)]
pub struct SearchSongsQuery {
    pub(crate) track: Option<String>,
    pub(crate) rank: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Song {
    pub key: Option<String>,
    pub name: String,
    pub uri: String,
    pub artist: String,
    pub album_cover_url: String,
    pub rank: Option<i32>
}

#[derive(Serialize, Debug)]
pub struct ErrorResponse {
    pub(crate) error: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePlaylistId {
    pub(crate) id: String
}


#[derive(Serialize, Debug)]
pub struct CreatePlaylistBody {
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) public: bool
}

#[derive(Serialize, Debug)]
pub struct AddSongsToPlaylistBody {
    pub(crate) uris: Vec<String>,
    pub(crate) position: i32
}

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct MusicTasteOverview {
    pub user_1: Option<String>,
    pub user_2: Option<String>,
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
