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