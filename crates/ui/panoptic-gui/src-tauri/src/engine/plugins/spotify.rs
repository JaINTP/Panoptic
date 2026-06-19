use crate::engine::pkce;
use crate::engine::settings::AppSettings;
use panoptic_core::{
    AuthState, MediaProvider, PanopticPlugin, PlaybackState, PluginCategory,
    PluginSettingsDefinition, SettingField, SettingFieldType,
};
use panoptic_provider_web::{WebFallbackEngine, WebPollError};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

const DEFAULT_CLIENT_ID: &str = "66bb8b83521e4034b4b7c035f09d7844";

pub struct SpotifyPlugin {
    app_handle: Arc<std::sync::Mutex<Option<tauri::AppHandle>>>,
    web_fallback: Arc<Mutex<Option<WebFallbackEngine>>>,
    code_verifier: Arc<std::sync::Mutex<Option<String>>>,
}

impl Default for SpotifyPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotifyPlugin {
    pub fn new() -> Self {
        Self {
            app_handle: Arc::new(std::sync::Mutex::new(None)),
            web_fallback: Arc::new(Mutex::new(None)),
            code_verifier: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn get_active_client_id(&self, app_handle: &tauri::AppHandle) -> String {
        let settings = AppSettings::load(app_handle);
        settings
            .plugins
            .get("spotify")
            .and_then(|v| v.get("client_id"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(DEFAULT_CLIENT_ID)
            .to_string()
    }
}

impl PanopticPlugin for SpotifyPlugin {
    fn id(&self) -> &'static str {
        "spotify"
    }

    fn name(&self) -> &'static str {
        "Spotify"
    }

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        // Store AppHandle
        if let Ok(mut lock) = self.app_handle.lock() {
            *lock = Some(app.clone());
        }

        // Load initial settings on setup
        let settings = AppSettings::load(app);
        if let Some(spotify_val) = settings.plugins.get("spotify") {
            let access_token = spotify_val.get("access_token").and_then(|v| v.as_str());
            let refresh_token = spotify_val.get("refresh_token").and_then(|v| v.as_str());
            if let (Some(at), Some(rt)) = (access_token, refresh_token) {
                if let Some(auth_tx) = app.try_state::<tokio::sync::watch::Sender<AuthState>>() {
                    let _ = auth_tx.send(AuthState::Authenticated {
                        provider: "spotify".to_string(),
                        access_token: at.to_string(),
                        refresh_token: rt.to_string(),
                    });
                }
            }
        }

        // Spawn observer to handle token exchange
        let auth_rx = app
            .try_state::<tokio::sync::watch::Receiver<AuthState>>()
            .ok_or_else(|| "Could not resolve AuthState receiver".to_string())?
            .inner()
            .clone();

        let auth_tx = app
            .try_state::<tokio::sync::watch::Sender<AuthState>>()
            .ok_or_else(|| "Could not resolve AuthState sender".to_string())?
            .inner()
            .clone();

        let app_handle = app.clone();
        let plugin_arc = Arc::new(Self {
            app_handle: self.app_handle.clone(),
            web_fallback: self.web_fallback.clone(),
            code_verifier: self.code_verifier.clone(),
        });

        let verifier_store = self.code_verifier.clone();

        tauri::async_runtime::spawn(async move {
            let mut rx = auth_rx;
            while rx.changed().await.is_ok() {
                let state = rx.borrow().clone();
                if let AuthState::Authenticating { provider, code } = state {
                    if provider != "spotify" {
                        continue;
                    }
                    info!("SpotifyPlugin: Authentication callback code received, starting exchange...");
                    let verifier = {
                        let mut store = verifier_store.lock().unwrap();
                        store.take()
                    };

                    if let Some(code_verifier) = verifier {
                        let client_id = plugin_arc.get_active_client_id(&app_handle);
                        match exchange_tokens(&client_id, &code, &code_verifier).await {
                            Ok((access_token, refresh_token)) => {
                                info!("SpotifyPlugin: Token exchange successful!");

                                // Save settings
                                let mut settings = AppSettings::load(&app_handle);
                                let mut spotify_settings = settings
                                    .plugins
                                    .get("spotify")
                                    .cloned()
                                    .unwrap_or_else(|| serde_json::json!({}));
                                spotify_settings["access_token"] =
                                    serde_json::json!(access_token.clone());
                                spotify_settings["refresh_token"] =
                                    serde_json::json!(refresh_token.clone());
                                settings
                                    .plugins
                                    .insert("spotify".to_string(), spotify_settings);
                                if let Err(e) = settings.save(&app_handle) {
                                    error!("SpotifyPlugin: Failed to save settings: {}", e);
                                }

                                // Update state
                                let _ = auth_tx.send(AuthState::Authenticated {
                                    provider: "spotify".to_string(),
                                    access_token,
                                    refresh_token,
                                });

                                // Notify UI to refresh settings
                                use tauri::Emitter;
                                let _ = app_handle.emit("auth_success", "spotify");
                            }
                            Err(e) => {
                                error!("SpotifyPlugin: Failed to exchange tokens: {}", e);
                                let _ = auth_tx.send(AuthState::Unauthenticated);
                            }
                        }
                    } else {
                        error!("SpotifyPlugin: Error: PKCE code verifier not found in memory!");
                        let _ = auth_tx.send(AuthState::Unauthenticated);
                    }
                }
            }
        });

        info!("Plugin 'Spotify' setup complete");
        Ok(())
    }

    fn media_provider(&self) -> Option<Box<dyn MediaProvider>> {
        let app_handle = self.app_handle.lock().unwrap().clone()?;
        let auth_rx = app_handle
            .try_state::<tokio::sync::watch::Receiver<AuthState>>()?
            .inner()
            .clone();
        let auth_tx = app_handle
            .try_state::<tokio::sync::watch::Sender<AuthState>>()?
            .inner()
            .clone();

        let web_fallback_lock = self.web_fallback.clone();
        let plugin = Self {
            app_handle: self.app_handle.clone(),
            web_fallback: self.web_fallback.clone(),
            code_verifier: self.code_verifier.clone(),
        };

        Some(Box::new(SpotifyMediaProvider {
            app_handle,
            auth_rx,
            auth_tx,
            web_fallback: web_fallback_lock,
            plugin,
        }))
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Auth,
            fields: vec![
                SettingField {
                    key: "client_id".to_string(),
                    label: "Custom Client ID".to_string(),
                    description: Some("Optional: Use your own Spotify Developer application Client ID.".to_string()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::Value::String(DEFAULT_CLIENT_ID.to_string()),
                },
                SettingField {
                    key: "link_action".to_string(),
                    label: "Spotify Integration".to_string(),
                    description: Some("Link your Spotify account to enable web API fallback when local pipes are unavailable.".to_string()),
                    field_type: SettingFieldType::Action {
                        button_label: "Link Spotify".to_string(),
                        action_name: "link".to_string(),
                    },
                    default_value: serde_json::Value::Null,
                },
            ],
        })
    }

    fn handle_action(
        &self,
        action: &str,
        app: &tauri::AppHandle,
    ) -> Result<serde_json::Value, String> {
        match action {
            "link" => {
                use tauri_plugin_opener::OpenerExt;
                let client_id = self.get_active_client_id(app);

                let verifier = pkce::generate_verifier();
                let challenge = pkce::generate_challenge(&verifier);

                if let Ok(mut store) = self.code_verifier.lock() {
                    *store = Some(verifier);
                }

                let redirect_uri = "http://127.0.0.1:3000/callback/spotify";
                let encoded_redirect = urlencoding::encode(redirect_uri);
                let url = format!(
                    "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&code_challenge_method=S256&code_challenge={}&scope=user-read-currently-playing",
                    client_id, encoded_redirect, challenge
                );

                info!("SpotifyPlugin: Launching browser for Spotify Authentication...");
                if let Err(e) = app.opener().open_url(url, None::<&str>) {
                    return Err(format!("Failed to open system browser: {}", e));
                }

                Ok(serde_json::json!({ "status": "initiated" }))
            }
            "unlink" => {
                let mut settings = AppSettings::load(app);
                if let Some(spotify_settings) = settings.plugins.get_mut("spotify") {
                    spotify_settings["access_token"] = serde_json::Value::Null;
                    spotify_settings["refresh_token"] = serde_json::Value::Null;
                }
                let _ = settings.save(app);

                if let Some(auth_tx) = app.try_state::<tokio::sync::watch::Sender<AuthState>>() {
                    let _ = auth_tx.send(AuthState::Unauthenticated);
                }

                Ok(serde_json::json!({ "status": "unlinked" }))
            }
            _ => Err(format!("Unknown action '{}'", action)),
        }
    }
}

struct SpotifyMediaProvider {
    app_handle: tauri::AppHandle,
    auth_rx: tokio::sync::watch::Receiver<AuthState>,
    auth_tx: tokio::sync::watch::Sender<AuthState>,
    web_fallback: Arc<Mutex<Option<WebFallbackEngine>>>,
    plugin: SpotifyPlugin,
}

impl MediaProvider for SpotifyMediaProvider {
    fn fetch_now_playing(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<PlaybackState, String>> + Send>> {
        let auth_rx = self.auth_rx.clone();
        let auth_tx = self.auth_tx.clone();
        let app_handle = self.app_handle.clone();
        let web_fallback_lock = self.web_fallback.clone();
        let plugin = SpotifyPlugin {
            app_handle: self.plugin.app_handle.clone(),
            web_fallback: self.plugin.web_fallback.clone(),
            code_verifier: self.plugin.code_verifier.clone(),
        };

        Box::pin(async move {
            if web_fallback_lock.lock().await.is_none() {
                let mut lock = web_fallback_lock.lock().await;
                *lock = Some(WebFallbackEngine::new(auth_rx.clone()));
            }

            let mut web_fallback_lock = web_fallback_lock.lock().await;
            let fallback = web_fallback_lock.as_mut().unwrap();
            match fallback.try_web_poll().await {
                Ok(Some(state)) => Ok(state),
                Ok(None) => Err("No active playback".to_string()),
                Err(WebPollError::Unauthorized) => {
                    warn!("Spotify access token unauthorized. Attempting to refresh...");
                    let current_auth = auth_rx.borrow().clone();
                    if let AuthState::Authenticated {
                        provider,
                        refresh_token,
                        ..
                    } = current_auth
                    {
                        if provider != "spotify" {
                            return Err("Current auth provider is not spotify".to_string());
                        }
                        let client_id = plugin.get_active_client_id(&app_handle);
                        match refresh_spotify_token(&client_id, &refresh_token).await {
                            Ok((new_at, new_rt)) => {
                                info!("Spotify token refreshed successfully.");

                                // Save settings
                                let mut settings = AppSettings::load(&app_handle);
                                let mut spotify_settings = settings
                                    .plugins
                                    .get("spotify")
                                    .cloned()
                                    .unwrap_or_else(|| serde_json::json!({}));
                                spotify_settings["access_token"] =
                                    serde_json::json!(new_at.clone());
                                spotify_settings["refresh_token"] =
                                    serde_json::json!(new_rt.clone());
                                settings
                                    .plugins
                                    .insert("spotify".to_string(), spotify_settings);
                                let _ = settings.save(&app_handle);

                                // Update state
                                let _ = auth_tx.send(AuthState::Authenticated {
                                    provider: "spotify".to_string(),
                                    access_token: new_at,
                                    refresh_token: new_rt,
                                });

                                // Retry polling once after successful token refresh
                                match fallback.try_web_poll().await {
                                    Ok(Some(state)) => Ok(state),
                                    Ok(None) => Err("No active playback after refresh".to_string()),
                                    Err(e) => {
                                        Err(format!("Web fallback error after refresh: {:?}", e))
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to refresh Spotify token: {}. Unlinking.", e);
                                // Clear settings
                                let mut settings = AppSettings::load(&app_handle);
                                if let Some(spotify_settings) = settings.plugins.get_mut("spotify")
                                {
                                    spotify_settings["access_token"] = serde_json::Value::Null;
                                    spotify_settings["refresh_token"] = serde_json::Value::Null;
                                }
                                let _ = settings.save(&app_handle);

                                let _ = auth_tx.send(AuthState::Unauthenticated);
                                Err("Token refresh failed, unlinked".to_string())
                            }
                        }
                    } else {
                        Err("Token expired and no refresh token available".to_string())
                    }
                }
                Err(e) => Err(format!("Web fallback error: {:?}", e)),
            }
        })
    }
}

// Token Exchange & Refresh helpers
async fn exchange_tokens(
    client_id: &str,
    code: &str,
    code_verifier: &str,
) -> Result<(String, String), String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", "http://127.0.0.1:3000/callback/spotify"),
        ("client_id", client_id),
        ("code_verifier", code_verifier),
    ];

    let res = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !res.status().is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return Err(format!(
            "Spotify token exchange returned error: {}",
            err_body
        ));
    }

    #[derive(serde::Deserialize)]
    struct TokenResponse {
        access_token: String,
        refresh_token: String,
    }

    let tokens: TokenResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    Ok((tokens.access_token, tokens.refresh_token))
}

async fn refresh_spotify_token(
    client_id: &str,
    refresh_token: &str,
) -> Result<(String, String), String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
    ];

    let res = client
        .post("https://accounts.spotify.com/api/token")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !res.status().is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return Err(format!("Spotify token refresh failed: {}", err_body));
    }

    #[derive(serde::Deserialize)]
    struct RefreshResponse {
        access_token: String,
        refresh_token: Option<String>,
    }

    let resp: RefreshResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;

    let new_refresh = resp
        .refresh_token
        .unwrap_or_else(|| refresh_token.to_string());
    Ok((resp.access_token, new_refresh))
}
