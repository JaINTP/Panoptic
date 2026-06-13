use crate::schema::spotify::SpotifyPlaybackResponse;
use panoptic_core::PlaybackState;

pub struct SpotifyApiClient {
    access_token: String,
    http_client: reqwest::Client,
    #[allow(dead_code)]
    schema: serde_yaml::Value,
}

impl SpotifyApiClient {
    pub fn new(access_token: &str, schema: serde_yaml::Value) -> Self {
        Self {
            access_token: access_token.to_string(),
            http_client: reqwest::Client::new(),
            schema,
        }
    }

    pub async fn fetch_playback(&self) -> Result<Option<PlaybackState>, reqwest::Error> {
        let res = self
            .http_client
            .get("https://api.spotify.com/v1/me/player/currently-playing")
            .bearer_auth(&self.access_token)
            .send()
            .await?;

        if res.status() == reqwest::StatusCode::UNAUTHORIZED {
            // Trigger an error to be handled as unauthorized
            return Err(res.error_for_status().unwrap_err());
        }

        if res.status() == 204 {
            return Ok(None);
        }

        let res = res.error_for_status()?;
        let spotify_state: SpotifyPlaybackResponse = res.json().await?;
        let item = match spotify_state.item {
            Some(i) => i,
            None => return Ok(None),
        };

        Ok(Some(PlaybackState {
            title: item.name,
            artist: item
                .artists
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            album: item.album.name,
            art_source: item
                .album
                .images
                .first()
                .map(|img| img.url.clone())
                .unwrap_or_default(),
            progress_ms: spotify_state.progress_ms,
            duration_ms: item.duration_ms,
            is_playing: spotify_state.is_playing,
            formatted_output: String::new(),
        }))
    }
}
