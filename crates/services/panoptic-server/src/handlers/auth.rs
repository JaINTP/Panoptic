use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use panoptic_core::{AppState, AuthState};
use std::collections::HashMap;

pub async fn auth_callback(
    Path(provider): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(error) = params.get("error") {
        let error_desc = params.get("error_description").cloned().unwrap_or_default();
        let message = if error_desc.is_empty() {
            format!("Authentication failed for {}: {}", provider, error)
        } else {
            format!(
                "Authentication failed for {}: {} ({})",
                provider, error, error_desc
            )
        };
        return axum::response::Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Authentication Failed</title></head>
<body style="background-color: #0f0f13; color: #f43f5e; font-family: sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
    <div style="background-color: #1e1e24; padding: 30px; border-radius: 12px; text-align: center; max-width: 400px; border: 1px solid #f43f5e;">
        <h2 style="margin-top:0;">Authentication Failed</h2>
        <p>{}</p>
    </div>
</body>
</html>"#,
            message
        )).into_response();
    }

    if let Some(code) = params.get("code") {
        let _ = state.auth_tx.send(AuthState::Authenticating {
            provider: provider.clone(),
            code: code.clone(),
        });
        return axum::response::Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Authentication Successful</title></head>
<body style="background-color: #0f0f13; color: #a78bfa; font-family: sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
    <div style="background-color: #1e1e24; padding: 30px; border-radius: 12px; text-align: center; max-width: 400px; border: 1px solid #a78bfa;">
        <h2 style="margin-top:0; color: #a78bfa;">Authentication Successful!</h2>
        <p>Panoptic is linking your {} account. You can safely close this window now.</p>
    </div>
</body>
</html>"#,
            provider
        )).into_response();
    }

    if let Some(access_token) = params.get("access_token") {
        let _ = state.auth_tx.send(AuthState::Authenticated {
            provider: provider.clone(),
            access_token: access_token.clone(),
            refresh_token: "".to_string(),
        });
        return axum::response::Html(format!(
            r#"<!DOCTYPE html>
<html>
<head><title>Authentication Successful</title></head>
<body style="background-color: #0f0f13; color: #a78bfa; font-family: sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0;">
    <div style="background-color: #1e1e24; padding: 30px; border-radius: 12px; text-align: center; max-width: 400px; border: 1px solid #a78bfa;">
        <h2 style="margin-top:0; color: #a78bfa;">Authentication Successful!</h2>
        <p>Panoptic is linking your {} account. You can safely close this window now.</p>
    </div>
</body>
</html>"#,
            provider
        )).into_response();
    }

    // Otherwise serve the JS hash parsing page
    axum::response::Html(format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Connecting to Panoptic...</title>
    <style>
        body {{
            background-color: #0f0f13;
            color: #e2e8f0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
        }}
        .card {{
            background-color: #1e1e24;
            padding: 30px;
            border-radius: 12px;
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.4);
            text-align: center;
            max-width: 400px;
        }}
        h2 {{ color: #a78bfa; margin-top: 0; }}
        .spinner {{
            border: 4px solid rgba(167, 139, 250, 0.1);
            width: 36px;
            height: 36px;
            border-radius: 50%;
            border-left-color: #a78bfa;
            animation: spin 1s linear infinite;
            margin: 20px auto;
        }}
        @keyframes spin {{ 0% {{ transform: rotate(0deg); }} 100% {{ transform: rotate(360deg); }} }}
    </style>
    <script>
        window.addEventListener('DOMContentLoaded', () => {{
            const hash = window.location.hash;
            if (hash) {{
                const params = new URLSearchParams(hash.substring(1));
                const accessToken = params.get("access_token");
                const error = params.get("error");
                const errorDesc = params.get("error_description");
                
                if (error) {{
                    window.location.href = window.location.pathname + "?error=" + encodeURIComponent(error) + 
                        (errorDesc ? "&error_description=" + encodeURIComponent(errorDesc) : "");
                }} else if (accessToken) {{
                    window.location.href = window.location.pathname + "?access_token=" + encodeURIComponent(accessToken);
                }} else {{
                    document.getElementById('status').innerText = "Authentication failed: No access token found in redirect.";
                    document.getElementById('spinner').style.display = 'none';
                }}
            }} else {{
                const urlParams = new URLSearchParams(window.location.search);
                if (!urlParams.has("access_token") && !urlParams.has("code") && !urlParams.has("error")) {{
                    document.getElementById('status').innerText = "Authentication failed: Callback parameters missing.";
                    document.getElementById('spinner').style.display = 'none';
                }}
            }}
        }});
    </script>
</head>
<body>
    <div class="card">
        <h2>Connecting {provider}...</h2>
        <div id="spinner" class="spinner"></div>
        <p id="status">Completing authentication, please do not close this window...</p>
    </div>
</body>
</html>"#,
        provider = provider
    )).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use panoptic_core::PlaybackState;
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_auth_callback_success() {
        let (auth_tx, mut auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let (_, css_version_rx) = watch::channel(1u32);
        let app_state = AppState {
            auth_tx,
            state_rx,
            css_version_rx,
            settings_path: None,
            app_handle: None,
        };

        let mut params = HashMap::new();
        params.insert("code".to_string(), "test_auth_code_123".to_string());

        let response = auth_callback(Path("spotify".to_string()), Query(params), State(app_state))
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("Authentication Successful!"));
        assert!(body_str.contains("Panoptic is linking your spotify account"));

        let current_auth = auth_rx.borrow_and_update().clone();
        assert!(matches!(
            current_auth,
            AuthState::Authenticating { provider, code } if provider == "spotify" && code == "test_auth_code_123"
        ));
    }

    #[tokio::test]
    async fn test_auth_callback_implicit_success() {
        let (auth_tx, mut auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let (_, css_version_rx) = watch::channel(1u32);
        let app_state = AppState {
            auth_tx,
            state_rx,
            css_version_rx,
            settings_path: None,
            app_handle: None,
        };

        let mut params = HashMap::new();
        params.insert(
            "access_token".to_string(),
            "test_access_token_123".to_string(),
        );

        let response = auth_callback(Path("twitch".to_string()), Query(params), State(app_state))
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("Authentication Successful!"));
        assert!(body_str.contains("Panoptic is linking your twitch account"));

        let current_auth = auth_rx.borrow_and_update().clone();
        assert!(matches!(
            current_auth,
            AuthState::Authenticated { provider, access_token, refresh_token }
            if provider == "twitch" && access_token == "test_access_token_123" && refresh_token.is_empty()
        ));
    }

    #[tokio::test]
    async fn test_auth_callback_error() {
        let (auth_tx, mut auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let (_, css_version_rx) = watch::channel(1u32);
        let app_state = AppState {
            auth_tx,
            state_rx,
            css_version_rx,
            settings_path: None,
            app_handle: None,
        };

        let mut params = HashMap::new();
        params.insert("error".to_string(), "access_denied".to_string());

        let response = auth_callback(Path("twitch".to_string()), Query(params), State(app_state))
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("Authentication failed for twitch: access_denied"));

        let current_auth = auth_rx.borrow_and_update().clone();
        assert!(matches!(current_auth, AuthState::Unauthenticated));
    }

    #[tokio::test]
    async fn test_auth_callback_missing_params() {
        let (auth_tx, mut auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let (_, css_version_rx) = watch::channel(1u32);
        let app_state = AppState {
            auth_tx,
            state_rx,
            css_version_rx,
            settings_path: None,
            app_handle: None,
        };

        let params = HashMap::new();

        let response = auth_callback(Path("spotify".to_string()), Query(params), State(app_state))
            .await
            .into_response();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert!(body_str.contains("Connecting spotify..."));
        assert!(body_str.contains("Completing authentication"));

        let current_auth = auth_rx.borrow_and_update().clone();
        assert!(matches!(current_auth, AuthState::Unauthenticated));
    }
}
