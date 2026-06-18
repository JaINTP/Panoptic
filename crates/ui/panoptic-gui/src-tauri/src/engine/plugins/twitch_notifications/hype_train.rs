use super::event_manager::TwitchEventManager;
use super::models::{HypeTrainState, TwitchContribution};
use super::websocket::run_websocket_loop;
use crate::engine::settings::AppSettings;
use axum::{extract::State as AxumState, routing::get, Router};
use panoptic_core::{
    AppState, AuthState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct TwitchHypeTrainPlugin {
    pub(super) manager: Arc<TwitchEventManager>,
}

impl TwitchHypeTrainPlugin {
    pub fn new(manager: Arc<TwitchEventManager>) -> Self {
        Self { manager }
    }
}

impl PanopticPlugin for TwitchHypeTrainPlugin {
    fn id(&self) -> &'static str {
        "twitch_hype_train"
    }

    fn name(&self) -> &'static str {
        "Twitch Hype Train"
    }

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let auth_rx = app
            .try_state::<tokio::sync::watch::Receiver<AuthState>>()
            .ok_or("No auth state")?
            .inner()
            .clone();
        let app_handle = app.clone();
        let manager = self.manager.clone();

        tauri::async_runtime::spawn(async move {
            let mut rx = auth_rx;
            let mut current_task: Option<tokio::task::JoinHandle<()>> = None;
            manager.init_pronouns().await;
            while rx.changed().await.is_ok() {
                let state = rx.borrow().clone();
                if let AuthState::Authenticated { provider, access_token, .. } = state {
                    if provider != "twitch" {
                        continue;
                    }
                    if let Some(t) = current_task.take() {
                        t.abort();
                    }
                    let app_inner = app_handle.clone();
                    let manager_inner = manager.clone();
                    current_task = Some(tokio::spawn(async move {
                        let settings = AppSettings::load(&app_inner);
                        let client_id = settings
                            .plugins
                            .get("twitch")
                            .and_then(|v| v.get("client_id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if client_id.is_empty() {
                            return;
                        }
                        run_websocket_loop(app_inner, manager_inner, client_id, access_token)
                            .await;
                    }));
                } else if matches!(state, AuthState::Unauthenticated) {
                    if let Some(t) = current_task.take() {
                        t.abort();
                    }
                }
            }
        });
        Ok(())
    }

    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let hype_state = self.manager.hype_state.clone();
        router
            .route(
                "/twitch/hype-train",
                get(move |AxumState(app_state): AxumState<AppState>| {
                    let state = hype_state.lock().unwrap().clone();
                    let settings = super::load_plugin_settings(
                        app_state.settings_path,
                        "twitch_hype_train",
                    );
                    async move {
                        axum::Json(serde_json::json!({ "state": state, "settings": settings }))
                    }
                }),
            )
            .route(
                "/overlay/twitch/hype-train",
                get(panoptic_server::handlers::twitch::get_twitch_hype_train_overlay),
            )
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField {
                    key: "inactive_title".into(),
                    label: "Overlay Title".into(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("Hype Train"),
                },
                SettingField {
                    key: "active_title".into(),
                    label: "Active Title".into(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("Hype Train Active!"),
                },
                SettingField {
                    key: "test_action".into(),
                    label: "Test Overlay".into(),
                    description: None,
                    field_type: SettingFieldType::Action {
                        button_label: "Test Hype Train".into(),
                        action_name: "test_hype_train".into(),
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
        if action == "test_hype_train" {
            let app_handle = app.clone();
            let state_lock = self.manager.hype_state.clone();
            tauri::async_runtime::spawn(async move {
                simulate_mock_hype_train(&app_handle, &state_lock).await;
            });
            Ok(serde_json::json!({ "status": "initiated" }))
        } else {
            Err("Unknown action".to_string())
        }
    }
}

async fn simulate_mock_hype_train(
    app: &tauri::AppHandle,
    state_lock: &Arc<Mutex<HypeTrainState>>,
) {
    use tauri::Emitter;
    let mut state = state_lock.lock().unwrap();
    state.active = true;
    state.level = 1;
    state.progress = 50;
    state.goal = 100;
    state.top_contributions = vec![TwitchContribution {
        user_id: "1".into(),
        user_login: "alpha".into(),
        user_name: "Alpha".into(),
        type_field: "BITS".into(),
        total: 100,
    }];
    let _ = app.emit("twitch_hype_train", state.clone());
}
