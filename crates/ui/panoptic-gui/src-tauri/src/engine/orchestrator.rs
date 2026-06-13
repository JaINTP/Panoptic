use crate::engine::native::create_native_provider;
use crate::engine::settings::AppSettings;
use panoptic_core::{AuthState, PlaybackState};
use panoptic_provider_web::{WebFallbackEngine, WebPollError};
use std::time::Duration;
use tokio::sync::{mpsc, watch};

pub const DEFAULT_CLIENT_ID: &str = "66bb8b83521e4034b4b7c035f09d7844";

fn get_active_client_id(app_handle: &tauri::AppHandle) -> String {
    let settings = AppSettings::load(app_handle);
    settings
        .client_id
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_CLIENT_ID.to_string())
}

pub enum AppCommand {
    OpenSettings,
    InitiateAuth,
    Quit,
}

pub struct EngineOrchestrator {
    cmd_rx: mpsc::Receiver<AppCommand>,
    state_tx: watch::Sender<PlaybackState>,
    auth_tx: watch::Sender<AuthState>,
    auth_rx: watch::Receiver<AuthState>,
    state_rx: watch::Receiver<PlaybackState>,
}

impl EngineOrchestrator {
    pub fn new(
        cmd_rx: mpsc::Receiver<AppCommand>,
        state_tx: watch::Sender<PlaybackState>,
        state_rx: watch::Receiver<PlaybackState>,
        auth_tx: watch::Sender<AuthState>,
        auth_rx: watch::Receiver<AuthState>,
    ) -> Self {
        Self {
            cmd_rx,
            state_tx,
            auth_tx,
            auth_rx,
            state_rx,
        }
    }

    pub async fn run(mut self, app_handle: tauri::AppHandle) {
        // Load initial settings on startup
        let settings = AppSettings::load(&app_handle);
        if let (Some(at), Some(rt)) = (
            settings.access_token.clone(),
            settings.refresh_token.clone(),
        ) {
            let _ = self.auth_tx.send(AuthState::Authenticated {
                access_token: at,
                refresh_token: rt,
            });
        }

        let native_provider = create_native_provider();
        let mut web_fallback = WebFallbackEngine::new(self.auth_rx.clone());

        let auth_tx = self.auth_tx.clone();
        tokio::spawn(panoptic_server::start_server(
            self.state_rx.clone(),
            auth_tx,
        ));

        let code_verifier_store = std::sync::Arc::new(std::sync::Mutex::new(None::<String>));

        let mut ui_auth_rx = self.auth_rx.clone();
        let app_handle_clone = app_handle.clone();
        let auth_tx_clone = self.auth_tx.clone();
        let verifier_store = code_verifier_store.clone();
        tokio::spawn(async move {
            use tauri::Emitter;
            while ui_auth_rx.changed().await.is_ok() {
                let state = ui_auth_rx.borrow().clone();
                match state {
                    AuthState::Authenticating { code } => {
                        println!("Authentication callback code received, starting exchange...");
                        let verifier = {
                            let mut store = verifier_store.lock().unwrap();
                            store.take()
                        };

                        if let Some(code_verifier) = verifier {
                            let client_id = get_active_client_id(&app_handle_clone);
                            match exchange_tokens(&client_id, &code, &code_verifier).await {
                                Ok((access_token, refresh_token)) => {
                                    println!("Token exchange successful!");

                                    // Save settings
                                    let mut settings = AppSettings::load(&app_handle_clone);
                                    settings.access_token = Some(access_token.clone());
                                    settings.refresh_token = Some(refresh_token.clone());
                                    if let Err(e) = settings.save(&app_handle_clone) {
                                        println!("Failed to save settings: {}", e);
                                    }

                                    // Update state
                                    let _ = auth_tx_clone.send(AuthState::Authenticated {
                                        access_token,
                                        refresh_token,
                                    });
                                }
                                Err(e) => {
                                    println!("Failed to exchange tokens: {}", e);
                                    let _ = auth_tx_clone.send(AuthState::Unauthenticated);
                                }
                            }
                        } else {
                            println!("Error: PKCE code verifier not found in memory!");
                            let _ = auth_tx_clone.send(AuthState::Unauthenticated);
                        }
                    }
                    AuthState::Authenticated { .. } => {
                        let _ = app_handle_clone.emit("auth_success", true);
                    }
                    _ => {}
                }
            }
        });

        let app_handle_cmd = app_handle.clone();
        let verifier_store_cmd = code_verifier_store.clone();
        tokio::spawn(async move {
            while let Some(cmd) = self.cmd_rx.recv().await {
                match cmd {
                    AppCommand::OpenSettings => {
                        if let Some(window) =
                            tauri::Manager::get_webview_window(&app_handle_cmd, "main")
                        {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    AppCommand::InitiateAuth => {
                        use tauri_plugin_opener::OpenerExt;
                        let client_id = get_active_client_id(&app_handle_cmd);

                        let verifier = crate::engine::pkce::generate_verifier();
                        let challenge = crate::engine::pkce::generate_challenge(&verifier);

                        if let Ok(mut store) = verifier_store_cmd.lock() {
                            *store = Some(verifier);
                        }

                        let redirect_uri = "http://127.0.0.1:3000/callback";
                        let url = format!(
                            "https://accounts.spotify.com/authorize?client_id={}&response_type=code&redirect_uri={}&code_challenge_method=S256&code_challenge={}&scope=user-read-currently-playing",
                            client_id, redirect_uri, challenge
                        );

                        println!("Launching browser for Spotify Authentication...");
                        if let Err(e) = app_handle_cmd.opener().open_url(url, None::<&str>) {
                            println!("Failed to open system browser: {}", e);
                        }
                    }
                    AppCommand::Quit => {
                        std::process::exit(0);
                    }
                }
            }
        });

        loop {
            let mut state = match native_provider.fetch_now_playing().await {
                Ok(s) => s,
                Err(_) => {
                    match web_fallback.try_web_poll().await {
                        Ok(Some(s)) => s,
                        Ok(None) => PlaybackState::default(),
                        Err(WebPollError::Unauthorized) => {
                            println!("Spotify access token unauthorized. Attempting to refresh...");
                            let current_auth = self.auth_rx.borrow().clone();
                            if let AuthState::Authenticated { refresh_token, .. } = current_auth {
                                let client_id = get_active_client_id(&app_handle);
                                match refresh_spotify_token(&client_id, &refresh_token).await {
                                    Ok((new_at, new_rt)) => {
                                        println!("Spotify token refreshed successfully.");

                                        // Save settings
                                        let mut settings = AppSettings::load(&app_handle);
                                        settings.access_token = Some(new_at.clone());
                                        settings.refresh_token = Some(new_rt.clone());
                                        let _ = settings.save(&app_handle);

                                        // Update state
                                        let _ = self.auth_tx.send(AuthState::Authenticated {
                                            access_token: new_at,
                                            refresh_token: new_rt,
                                        });
                                    }
                                    Err(e) => {
                                        println!(
                                            "Failed to refresh Spotify token: {}. Unlinking.",
                                            e
                                        );
                                        // Clear settings
                                        let mut settings = AppSettings::load(&app_handle);
                                        settings.access_token = None;
                                        settings.refresh_token = None;
                                        let _ = settings.save(&app_handle);

                                        let _ = self.auth_tx.send(AuthState::Unauthenticated);
                                    }
                                }
                            }
                            PlaybackState::default()
                        }
                        Err(e) => {
                            println!("Web fallback error: {:?}", e);
                            PlaybackState::default()
                        }
                    }
                }
            };

            let template = AppSettings::load(&app_handle)
                .template
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| "Now Playing: {title} by {artist}".to_string());
            let formatted_string = state.format(&template);

            state.formatted_output = formatted_string.clone();

            // Write formatted track info to ~/.config/panoptic/current_track.txt
            if let Ok(home) = std::env::var("HOME") {
                let config_path = std::path::PathBuf::from(home).join(".config/panoptic");
                if !config_path.exists() {
                    let _ = std::fs::create_dir_all(&config_path);
                }
                let file_path = config_path.join("current_track.txt");
                let _ = std::fs::write(file_path, &formatted_string);
            }

            let _ = self.state_tx.send(state.clone());
            use tauri::Emitter;
            let _ = app_handle.emit("playback_update", state);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

async fn exchange_tokens(
    client_id: &str,
    code: &str,
    code_verifier: &str,
) -> Result<(String, String), String> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", "http://127.0.0.1:3000/callback"),
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
