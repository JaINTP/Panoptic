use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyPlaybackResponse {
    pub item: Option<SpotifyItem>,
    pub progress_ms: u32,
    pub is_playing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyItem {
    pub name: String,
    pub artists: Vec<SpotifyArtist>,
    pub album: SpotifyAlbum,
    pub duration_ms: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyArtist {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyAlbum {
    pub name: String,
    pub images: Vec<SpotifyImage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpotifyImage {
    pub url: String,
}
