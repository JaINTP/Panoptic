use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use panoptic_core::AuthState;
use std::collections::HashMap;

pub async fn spotify_callback(
    Query(params): Query<HashMap<String, String>>,
    State(state): State<crate::AppState>,
) -> impl IntoResponse {
    if let Some(error) = params.get("error") {
        return format!("Authentication failed: {}", error);
    }

    if let Some(code) = params.get("code") {
        let _ = state
            .auth_tx
            .send(AuthState::Authenticating { code: code.clone() });
        "Authentication successful! Panoptic is linking your account. You can close this window."
            .to_string()
    } else {
        "Authentication failed. No authorization code provided.".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use panoptic_core::PlaybackState;
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_spotify_callback_success() {
        let (auth_tx, mut auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let app_state = crate::AppState { auth_tx, state_rx };

        let mut params = HashMap::new();
        params.insert("code".to_string(), "test_auth_code_123".to_string());

        let response = spotify_callback(Query(params), State(app_state))
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("Authentication successful"));

        let current_auth = auth_rx.borrow_and_update().clone();
        assert!(matches!(
            current_auth,
            AuthState::Authenticating { code } if code == "test_auth_code_123"
        ));
    }

    #[tokio::test]
    async fn test_spotify_callback_error() {
        let (auth_tx, mut auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let app_state = crate::AppState { auth_tx, state_rx };

        let mut params = HashMap::new();
        params.insert("error".to_string(), "access_denied".to_string());

        let response = spotify_callback(Query(params), State(app_state))
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("Authentication failed: access_denied"));

        let current_auth = auth_rx.borrow_and_update().clone();
        assert!(matches!(current_auth, AuthState::Unauthenticated));
    }

    #[tokio::test]
    async fn test_spotify_callback_missing_params() {
        let (auth_tx, mut auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let app_state = crate::AppState { auth_tx, state_rx };

        let params = HashMap::new();

        let response = spotify_callback(Query(params), State(app_state))
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("No authorization code provided"));

        let current_auth = auth_rx.borrow_and_update().clone();
        assert!(matches!(current_auth, AuthState::Unauthenticated));
    }
}
