use axum::{http::header, response::IntoResponse};

pub async fn get_twitch_hype_train_overlay() -> impl IntoResponse {
    let html = include_str!("../twitch_hype_train.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

pub async fn get_twitch_alerts_overlay() -> impl IntoResponse {
    let html = include_str!("../twitch_alerts.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

pub async fn get_twitch_chat_overlay() -> impl IntoResponse {
    let html = include_str!("../twitch_chat.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}
