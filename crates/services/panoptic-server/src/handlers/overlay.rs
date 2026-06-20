use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
};
use panoptic_core::AppState;

fn default_css(overlay_id: &str) -> String {
    match overlay_id {
        "now_playing" => {
            include_str!("../../../../../examples/now-playing/now-playing-default.css").to_string()
        }
        "twitch_hype_train" => {
            include_str!("../../../../../examples/twitch-hype-train/hype-train-default.css")
                .to_string()
        }
        "twitch_alerts" => {
            include_str!("../../../../../examples/twitch-alerts/twitch-alerts-default.css")
                .to_string()
        }
        "twitch_chat" => {
            include_str!("../../../../../examples/themes/cyber_complete.css").to_string()
        }
        "pomodoro" => {
            include_str!("../../../../../examples/pomodoro/pomodoro-default.css").to_string()
        }
        _ => String::new(),
    }
}

pub async fn get_overlay(State(state): State<AppState>) -> impl IntoResponse {
    let mut html = include_str!("../overlay.html").to_string();

    let mut config = serde_json::json!({
        "not_playing_title": "Not Playing",
        "not_playing_artist": "Unknown Artist",
        "not_playing_album": "Unknown Album"
    });

    if let Some(ref path) = state.settings_path {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    for key in [
                        "not_playing_title",
                        "not_playing_artist",
                        "not_playing_album",
                    ] {
                        if let Some(v) = val.get(key) {
                            if v.is_string() && !v.as_str().unwrap().trim().is_empty() {
                                config[key] = v.clone();
                            }
                        }
                    }
                }
            }
        }
    }

    let inject_js = format!("<script>window.PanopticSettings = {};</script>", config);
    html = html.replace("<head>", &format!("<head>{}", inject_js));

    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

pub async fn get_overlay_css(
    Path(overlay_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let css = if let Some(ref path) = state.settings_path {
        let file_path = path
            .parent()
            .unwrap()
            .join("overlays")
            .join(format!("{}.css", overlay_id));

        if file_path.exists() {
            std::fs::read_to_string(file_path).unwrap_or_else(|_| default_css(&overlay_id))
        } else {
            default_css(&overlay_id)
        }
    } else {
        default_css(&overlay_id)
    };

    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], css)
}

pub async fn get_overlay_version(State(state): State<AppState>) -> impl IntoResponse {
    let version = *state.css_version_rx.borrow();
    axum::Json(serde_json::json!({ "version": version }))
}
