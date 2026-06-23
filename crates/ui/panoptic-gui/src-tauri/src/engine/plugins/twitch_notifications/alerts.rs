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
                    label: "Follow Text Template".into(),
                    description: Some("Variables: {user}".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} just followed!"),
                },
                SettingField {
                    key: "follow_icon".into(),
                    label: "Follow Icon/Emoji".into(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("✨"),
                },
                SettingField {
                    key: "sub_text".into(),
                    label: "Subscription Text Template".into(),
                    description: Some("Variables: {user}, {tier}, {months}".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!(
                        "{user} subscribed at Tier {tier} for {months} months!"
                    ),
                },
                SettingField {
                    key: "sub_icon".into(),
                    label: "Subscription Icon/Emoji".into(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("💖"),
                },
                SettingField {
                    key: "giftsub_text".into(),
                    label: "Gift Sub Text Template".into(),
                    description: Some("Variables: {user}, {total}, {tier}".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!(
                        "{user} gifted {total} Tier {tier} subscriptions!"
                    ),
                },
                SettingField {
                    key: "giftsub_icon".into(),
                    label: "Gift Sub Icon/Emoji".into(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("🎁"),
                },
                SettingField {
                    key: "raid_text".into(),
                    label: "Raid Text Template".into(),
                    description: Some("Variables: {user}, {viewers}".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} raided with {viewers} viewers!"),
                },
                SettingField {
                    key: "raid_icon".into(),
                    label: "Raid Icon/Emoji".into(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("⚔️"),
                },
                SettingField {
                    key: "cheer_text".into(),
                    label: "Cheer Text Template".into(),
                    description: Some("Variables: {user}, {bits}, {message}".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} cheered {bits} bits: {message}"),
                },
                SettingField {
                    key: "cheer_icon".into(),
                    label: "Cheer Icon/Emoji".into(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("💎"),
                },
                SettingField {
                    key: "alert_duration".into(),
                    label: "Alert Duration (seconds)".into(),
                    description: Some("How long each alert stays on screen".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(8),
                },
                SettingField {
                    key: "keep_last_alert".into(),
                    label: "Keep Last Alert On Screen".into(),
                    description: Some("Keep the final alert visible instead of fading out".into()),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(false),
                },
                SettingField {
                    key: "extend_downwards".into(),
                    label: "Extend Downwards".into(),
                    description: Some("Extend/stack alerts downwards instead of upwards".into()),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(false),
                },
                SettingField {
                    key: "alert_limit".into(),
                    label: "Alert Limit".into(),
                    description: Some(
                        "Maximum number of alerts displayed simultaneously (0 for unlimited)"
                            .into(),
                    ),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(4),
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
                let alerts = vec![
                    TwitchAlert::Follow {
                        user_name: "Entity_Alpha".into(),
                    },
                    TwitchAlert::Subscription {
                        user_name: "Tauri_Dev".into(),
                        tier: "1000".into(),
                        is_gift: false,
                        cumulative_months: 3,
                    },
                    TwitchAlert::GiftSubscription {
                        user_name: "Generous_Giver".into(),
                        total: 5,
                        tier: "1000".into(),
                        is_anonymous: false,
                    },
                    TwitchAlert::Raid {
                        from_broadcaster_name: "Streamer_Bot".into(),
                        viewers: 42,
                    },
                    TwitchAlert::Cheer {
                        user_name: "Generous_Cheerer".into(),
                        bits: 500,
                        message: "Keep up the great work! Hype!".into(),
                    },
                ];
                for alert in alerts {
                    update_alert(&app_handle, &manager.alert_state, alert);
                    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;
                }
            });
            Ok(serde_json::json!({ "status": "initiated" }))
        } else {
            Err("Unknown action".to_string())
        }
    }
}
