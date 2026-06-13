#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthState {
    Unauthenticated,
    Authenticating {
        code: String,
    },
    Authenticated {
        access_token: String,
        refresh_token: String,
    },
}
