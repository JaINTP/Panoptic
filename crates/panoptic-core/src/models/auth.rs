#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AuthState {
    Unauthenticated,
    Authenticating {
        provider: String,
        code: String,
    },
    Authenticated {
        provider: String,
        access_token: String,
        refresh_token: String,
    },
}

#[derive(Clone)]
pub struct AppState {
    pub auth_tx: tokio::sync::watch::Sender<AuthState>,
    pub state_rx: tokio::sync::watch::Receiver<crate::models::playback::PlaybackState>,
    pub css_version_rx: tokio::sync::watch::Receiver<u32>,
    pub settings_path: Option<std::path::PathBuf>,
    #[cfg(feature = "plugin")]
    pub app_handle: Option<tauri::AppHandle>,
}
