use axum::response::IntoResponse;

pub struct HealthHandler;

impl HealthHandler {
    pub async fn check() -> impl IntoResponse {
        "OK"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;

    #[tokio::test]
    async fn test_health_check() {
        let response = HealthHandler::check().await.into_response();
        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
        assert_eq!(body_str, "OK");
    }
}
