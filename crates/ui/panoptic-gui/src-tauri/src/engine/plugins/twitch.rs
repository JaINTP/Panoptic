use crate::engine::settings::AppSettings;
use panoptic_core::{
    AuthState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use std::sync::Arc;
use tauri::Manager;
use tracing::{error, info};

const DEFAULT_CLIENT_ID: &str = "j58sbgcd48oeuzt7tmxgb0ktkdtxfe";

pub struct TwitchPlugin {
    app_handle: Arc<std::sync::Mutex<Option<tauri::AppHandle>>>,
}

impl Default for TwitchPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl TwitchPlugin {
    pub fn new() -> Self {
        Self {
            app_handle: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn get_active_client_id(&self, app_handle: &tauri::AppHandle) -> String {
        let settings = AppSettings::load(app_handle);
        settings
            .plugins
            .get("twitch")
            .and_then(|v| v.get("client_id"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(DEFAULT_CLIENT_ID)
            .to_string()
    }
}

impl PanopticPlugin for TwitchPlugin {
    fn id(&self) -> &'static str {
        "twitch"
    }

    fn name(&self) -> &'static str {
        "Twitch"
    }

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        // Store AppHandle
        if let Ok(mut lock) = self.app_handle.lock() {
            *lock = Some(app.clone());
        }

        // Load initial settings on setup
        let settings = AppSettings::load(app);
        if let Some(twitch_val) = settings.plugins.get("twitch") {
            let access_token = twitch_val.get("access_token").and_then(|v| v.as_str());
            let refresh_token = twitch_val.get("refresh_token").and_then(|v| v.as_str());
            if let (Some(at), Some(rt)) = (access_token, refresh_token) {
                if let Some(auth_tx) = app.try_state::<tokio::sync::watch::Sender<AuthState>>() {
                    let _ = auth_tx.send(AuthState::Authenticated {
                        provider: "twitch".to_string(),
                        access_token: at.to_string(),
                        refresh_token: rt.to_string(),
                    });
                }
            }
        }

        // Spawn observer to handle token saving
        let auth_rx = app
            .try_state::<tokio::sync::watch::Receiver<AuthState>>()
            .ok_or_else(|| "Could not resolve AuthState receiver".to_string())?
            .inner()
            .clone();

        let app_handle = app.clone();

        tauri::async_runtime::spawn(async move {
            let mut rx = auth_rx;
            while rx.changed().await.is_ok() {
                let state = rx.borrow().clone();
                if let AuthState::Authenticated {
                    provider,
                    access_token,
                    refresh_token,
                } = state
                {
                    if provider != "twitch" {
                        continue;
                    }

                    // Check if token has actually changed to prevent saving loop
                    let settings = AppSettings::load(&app_handle);
                    let current_token = settings
                        .plugins
                        .get("twitch")
                        .and_then(|v| v.get("access_token"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");

                    if current_token != access_token {
                        info!("TwitchPlugin: New authenticated token received, saving...");
                        let mut settings = AppSettings::load(&app_handle);
                        let mut twitch_settings = settings
                            .plugins
                            .get("twitch")
                            .cloned()
                            .unwrap_or_else(|| serde_json::json!({}));
                        twitch_settings["access_token"] = serde_json::json!(access_token.clone());
                        twitch_settings["refresh_token"] = serde_json::json!(refresh_token.clone());
                        settings
                            .plugins
                            .insert("twitch".to_string(), twitch_settings);
                        if let Err(e) = settings.save(&app_handle) {
                            error!("TwitchPlugin: Failed to save settings: {}", e);
                        }

                        // Notify UI
                        use tauri::Emitter;
                        let _ = app_handle.emit("auth_success", "twitch");
                    }
                }
            }
        });

        info!("Plugin 'Twitch' setup complete");
        Ok(())
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Auth,
            fields: vec![
                SettingField {
                    key: "client_id".to_string(),
                    label: "Twitch Client ID".to_string(),
                    description: Some(
                        "Required: Your Twitch Developer application Client ID.".to_string(),
                    ),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::Value::String(DEFAULT_CLIENT_ID.to_string()),
                },
                SettingField {
                    key: "link_action".to_string(),
                    label: "Twitch Integration".to_string(),
                    description: Some(
                        "Link your Twitch account to enable chat and stream features.".to_string(),
                    ),
                    field_type: SettingFieldType::Action {
                        button_label: "Link Twitch".to_string(),
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
                if client_id.is_empty() {
                    return Err(
                        "Twitch Client ID is required to initiate authentication".to_string()
                    );
                }

                let redirect_uri = "http://127.0.0.1:3000/callback/twitch";
                let encoded_redirect = urlencoding::encode(redirect_uri);
                let url = format!(
                    "https://id.twitch.tv/oauth2/authorize?client_id={}&response_type=token&redirect_uri={}&scope=user:read:email+chat:read+chat:edit+channel:read:hype_train+user:read:chat+channel:read:subscriptions+moderator:read:followers+bits:read",
                    client_id, encoded_redirect
                );

                info!("TwitchPlugin: Launching browser for Twitch Authentication...");
                if let Err(e) = app.opener().open_url(url, None::<&str>) {
                    return Err(format!("Failed to open system browser: {}", e));
                }

                Ok(serde_json::json!({ "status": "initiated" }))
            }
            "unlink" => {
                let mut settings = AppSettings::load(app);
                if let Some(twitch_settings) = settings.plugins.get_mut("twitch") {
                    twitch_settings["access_token"] = serde_json::Value::Null;
                    twitch_settings["refresh_token"] = serde_json::Value::Null;
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
