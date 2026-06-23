use axum::{
    extract::{Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use panoptic_core::AppState;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ArtQuery {
    v: Option<String>,
    t: Option<String>,
}

pub async fn get_art(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ArtQuery>,
) -> Response {
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let referer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let is_obs = user_agent.contains("OBS") || referer.contains("/overlay/");
    let client_type = if is_obs {
        "OBS Browser Source"
    } else {
        "Generic Client"
    };

    tracing::info!(
        "[Art Request] Incoming request from {} | User-Agent: {:?} | Referer: {:?}",
        client_type,
        user_agent,
        referer
    );

    // Use query param if provided, otherwise fall back to current state.
    let source = match params.v {
        Some(ref v) if !v.is_empty() => v.clone(),
        _ => {
            let s = state.state_rx.borrow().art_source.clone();
            s
        }
    };

    tracing::debug!(
        "[Art Request] Requested art source: {:?} | Cache buster (t): {:?}",
        source,
        params.t
    );

    if source.is_empty() {
        tracing::warn!("[Art Request] Art source is empty, returning 404");
        return StatusCode::NOT_FOUND.into_response();
    }

    if let Ok(url) = reqwest::Url::parse(&source) {
        if url.scheme() == "file" {
            if let Ok(fs_path) = url.to_file_path() {
                tracing::info!("[Art Request] Loading local artwork file: {:?}", fs_path);
                match tokio::fs::read(&fs_path).await {
                    Ok(bytes) => {
                        let mime = mime_guess::from_path(&fs_path)
                            .first_or_octet_stream()
                            .to_string();
                        tracing::info!(
                            "[Art Request] Successfully read local artwork file ({:?}), size: {} bytes, mime: {}",
                            fs_path,
                            bytes.len(),
                            mime
                        );
                        return (
                            StatusCode::OK,
                            [
                                (header::CONTENT_TYPE, mime),
                                (header::CACHE_CONTROL, "no-store".to_string()),
                            ],
                            bytes,
                        )
                            .into_response();
                    }
                    Err(e) => {
                        tracing::error!(
                            "[Art Request] Failed to read local artwork file at {:?}: {}",
                            fs_path,
                            e
                        );
                        return StatusCode::NOT_FOUND.into_response();
                    }
                }
            } else {
                tracing::error!(
                    "[Art Request] Failed to convert file URL to file path: {:?}",
                    url
                );
            }
        }
    }

    if source.starts_with("https://") || source.starts_with("http://") {
        tracing::info!("[Art Request] Proxying remote artwork URL: {}", source);
        match reqwest::get(&source).await {
            Ok(resp) if resp.status().is_success() => {
                let content_type = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("application/octet-stream")
                    .to_string();
                match resp.bytes().await {
                    Ok(bytes) => {
                        tracing::info!(
                            "[Art Request] Successfully proxied remote artwork, size: {} bytes, content-type: {}",
                            bytes.len(),
                            content_type
                        );
                        (
                            StatusCode::OK,
                            [
                                (header::CONTENT_TYPE, content_type),
                                (header::CACHE_CONTROL, "no-store".to_string()),
                            ],
                            bytes.to_vec(),
                        )
                            .into_response()
                    }
                    Err(e) => {
                        tracing::error!("[Art Request] Failed to read proxied bytes: {}", e);
                        StatusCode::BAD_GATEWAY.into_response()
                    }
                }
            }
            Ok(resp) => {
                tracing::error!(
                    "[Art Request] Remote server returned status error: {} for URL: {}",
                    resp.status(),
                    source
                );
                StatusCode::BAD_GATEWAY.into_response()
            }
            Err(e) => {
                tracing::error!("[Art Request] Network error proxying URL {}: {}", source, e);
                StatusCode::BAD_GATEWAY.into_response()
            }
        }
    } else {
        tracing::warn!(
            "[Art Request] Invalid or unsupported artwork URL scheme: {}",
            source
        );
        StatusCode::BAD_REQUEST.into_response()
    }
}
