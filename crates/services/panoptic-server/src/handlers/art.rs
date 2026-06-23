use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use panoptic_core::AppState;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ArtQuery {
    v: Option<String>,
}

pub async fn get_art(State(state): State<AppState>, Query(params): Query<ArtQuery>) -> Response {
    // Use query param if provided, otherwise fall back to current state.
    let source = match params.v {
        Some(ref v) if !v.is_empty() => v.clone(),
        _ => {
            let s = state.state_rx.borrow().art_source.clone();
            s
        }
    };

    if source.is_empty() {
        return StatusCode::NOT_FOUND.into_response();
    }

    if let Some(path) = source.strip_prefix("file:///") {
        // On Windows the path after file:/// is like C:/... - use as-is.
        // On Linux it would be /absolute/path - prepend the slash back.
        #[cfg(target_os = "windows")]
        let fs_path = std::path::PathBuf::from(path);
        #[cfg(not(target_os = "windows"))]
        let fs_path = std::path::PathBuf::from(format!("/{}", path));

        match tokio::fs::read(&fs_path).await {
            Ok(bytes) => {
                let mime = mime_guess::from_path(&fs_path)
                    .first_or_octet_stream()
                    .to_string();
                (
                    StatusCode::OK,
                    [
                        (header::CONTENT_TYPE, mime),
                        (header::CACHE_CONTROL, "no-store".to_string()),
                    ],
                    bytes,
                )
                    .into_response()
            }
            Err(_) => StatusCode::NOT_FOUND.into_response(),
        }
    } else if source.starts_with("https://") || source.starts_with("http://") {
        match reqwest::get(&source).await {
            Ok(resp) if resp.status().is_success() => {
                let content_type = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("application/octet-stream")
                    .to_string();
                match resp.bytes().await {
                    Ok(bytes) => (
                        StatusCode::OK,
                        [
                            (header::CONTENT_TYPE, content_type),
                            (header::CACHE_CONTROL, "no-store".to_string()),
                        ],
                        bytes.to_vec(),
                    )
                        .into_response(),
                    Err(_) => StatusCode::BAD_GATEWAY.into_response(),
                }
            }
            _ => StatusCode::BAD_GATEWAY.into_response(),
        }
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}
