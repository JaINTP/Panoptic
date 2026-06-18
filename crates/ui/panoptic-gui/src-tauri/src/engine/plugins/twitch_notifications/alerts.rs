use super::event_manager::TwitchEventManager;
use super::models::TwitchAlert;
use super::websocket::update_alert;
use axum::{extract::State as AxumState, routing::get, Router};
use panoptic_core::{
    AppState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use std::sync::Arc;

pub struct TwitchAlertsPlugin {
    pub(super) manager: Arc<TwitchEventManager>,
}

impl TwitchAlertsPlugin {
    pub fn new(manager: Arc<TwitchEventManager>) -> Self {
        Self { manager }
    }
}

impl PanopticPlugin for TwitchAlertsPlugin {
    fn id(&self) -> &'static str {
        "twitch_alerts"
    }

    fn name(&self) -> &'static str {
        "Twitch Alerts"
    }

    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let alert_state = self.manager.alert_state.clone();
        router
            .route(
                "/twitch/alerts",
                get(move |AxumState(app_state): AxumState<AppState>| {
                    let state = alert_state.lock().unwrap().clone();
                    let settings =
                        super::load_plugin_settings(app_state.settings_path, "twitch_alerts");
                    async move {
                        axum::Json(serde_json::json!({
                            "active_alerts": state.active_alerts,
                            "settings": settings
                        }))
                    }
                }),
            )
            .route(
                "/overlay/twitch/alerts",
                get(panoptic_server::handlers::twitch::get_twitch_alerts_overlay),
            )
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField {
                    key: "follow_text".into(),
                    label: "Follow Text".into(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} just followed!"),
                },
                SettingField {
                    key: "test_alerts".into(),
                    label: "Test Simulation".into(),
                    description: None,
                    field_type: SettingFieldType::Action {
                        button_label: "Simulate All Alerts".into(),
                        action_name: "test_all".into(),
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
        if action == "test_all" {
            let app_handle = app.clone();
            let manager = self.manager.clone();
            tauri::async_runtime::spawn(async move {
                let alerts = vec![TwitchAlert::Follow { user_name: "Entity_Alpha".into() }];
                for alert in alerts {
                    update_alert(&app_handle, &manager.alert_state, alert);
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                }
            });
            Ok(serde_json::json!({ "status": "initiated" }))
        } else {
            Err("Unknown action".to_string())
        }
    }
}
