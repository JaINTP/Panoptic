use axum::{extract::State, http::header, response::IntoResponse};
use panoptic_core::AppState;

pub async fn get_current_track(State(state): State<AppState>) -> impl IntoResponse {
    let formatted = state.state_rx.borrow().formatted_output.clone();
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        formatted,
    )
}

pub async fn get_playback(State(state): State<AppState>) -> impl IntoResponse {
    let playback = state.state_rx.borrow().clone();
    (
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        serde_json::to_string(&playback).unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use panoptic_core::{AuthState, PlaybackState};
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_get_current_track_handler() {
        let (_state_tx, state_rx) = watch::channel(PlaybackState {
            formatted_output: "Now Playing: Song by Artist".to_string(),
            ..Default::default()
        });
        let (auth_tx, _auth_rx) = watch::channel(AuthState::Unauthenticated);

        let (_, css_version_rx) = watch::channel(1u32);
        let state = AppState {
            auth_tx,
            state_rx,
            css_version_rx,
            settings_path: None,
        };

        let response = get_current_track(State(state)).await.into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let headers = response.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "text/plain; charset=utf-8"
        );

        // Extract body bytes in Axum v0.7 using axum::body::to_bytes
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

        assert_eq!(body_str, "Now Playing: Song by Artist");
    }

    #[tokio::test]
    async fn test_get_playback_handler() {
        let (_state_tx, state_rx) = watch::channel(PlaybackState {
            title: "Song".to_string(),
            artist: "Artist".to_string(),
            album: "Album".to_string(),
            is_playing: true,
            ..Default::default()
        });
        let (auth_tx, _auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_, css_version_rx) = watch::channel(1u32);
        let state = AppState {
            auth_tx,
            state_rx,
            css_version_rx,
            settings_path: None,
        };

        let response = get_playback(State(state)).await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let headers = response.headers();
        assert_eq!(
            headers.get(header::CONTENT_TYPE).unwrap(),
            "application/json; charset=utf-8"
        );

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        let parsed: PlaybackState = serde_json::from_str(&body_str).unwrap();

        assert_eq!(parsed.title, "Song");
        assert_eq!(parsed.artist, "Artist");
        assert_eq!(parsed.album, "Album");
        assert!(parsed.is_playing);
    }
}
