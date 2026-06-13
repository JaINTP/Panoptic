use crate::client::spotify::SpotifyApiClient;
use crate::schema::bootstrapper::SchemaBootstrapper;
use panoptic_core::{AuthState, PlaybackState};
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub enum WebPollError {
    Unauthorized,
    Other(String),
}

pub struct WebFallbackEngine {
    pub auth_state: watch::Receiver<AuthState>,
    pub web_client: Option<SpotifyApiClient>,
    pub current_token: Option<String>,
}

impl WebFallbackEngine {
    pub fn new(auth_state: watch::Receiver<AuthState>) -> Self {
        Self {
            auth_state,
            web_client: None,
            current_token: None,
        }
    }

    pub async fn try_web_poll(&mut self) -> Result<Option<PlaybackState>, WebPollError> {
        let current_auth = self.auth_state.borrow().clone();
        match current_auth {
            AuthState::Authenticated { access_token, .. } => {
                if self.web_client.is_none() || self.current_token.as_ref() != Some(&access_token) {
                    let schema = match SchemaBootstrapper::bootstrap().await {
                        Some(s) => s,
                        None => {
                            return Err(WebPollError::Other("Schema bootstrap failed".to_string()))
                        }
                    };
                    self.web_client = Some(SpotifyApiClient::new(&access_token, schema));
                    self.current_token = Some(access_token.clone());
                }

                let client = match &self.web_client {
                    Some(c) => c,
                    None => return Err(WebPollError::Other("Web client unavailable".to_string())),
                };

                match client.fetch_playback().await {
                    Ok(state) => Ok(state),
                    Err(err) => {
                        if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
                            Err(WebPollError::Unauthorized)
                        } else {
                            Err(WebPollError::Other(err.to_string()))
                        }
                    }
                }
            }
            _ => Ok(None),
        }
    }
}
