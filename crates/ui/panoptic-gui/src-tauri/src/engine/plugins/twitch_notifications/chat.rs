use super::event_manager::TwitchEventManager;
use super::models::{ChatBadge, ChatFragment, ChatMessageData};
use axum::{extract::State as AxumState, routing::get, Router};
use panoptic_core::{
    AppState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use std::sync::Arc;

pub struct TwitchChatPlugin {
    pub(super) manager: Arc<TwitchEventManager>,
}

impl TwitchChatPlugin {
    pub fn new(manager: Arc<TwitchEventManager>) -> Self {
        Self { manager }
    }
}

impl PanopticPlugin for TwitchChatPlugin {
    fn id(&self) -> &'static str {
        "twitch_chat"
    }

    fn name(&self) -> &'static str {
        "Twitch Chat"
    }

    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let chat_state = self.manager.chat_state.clone();
        router
            .route(
                "/twitch/chat",
                get(move |AxumState(app_state): AxumState<AppState>| {
                    let state = chat_state.lock().unwrap().clone();
                    let settings =
                        super::load_plugin_settings(app_state.settings_path, "twitch_chat");
                    async move {
                        axum::Json(serde_json::json!({
                            "messages": state.messages,
                            "settings": settings
                        }))
                    }
                }),
            )
            .route(
                "/overlay/twitch/chat",
                get(panoptic_server::handlers::twitch::get_twitch_chat_overlay),
            )
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField {
                    key: "message_template".into(),
                    label: "Message Template".into(),
                    description: Some("Format: {badges} {pronouns} {user}: {message}".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{badges} {pronouns} {user}: {message}"),
                },
                SettingField {
                    key: "chat_animation".into(),
                    label: "Entrance Animation".into(),
                    description: Some("Choose how messages appear.".into()),
                    field_type: SettingFieldType::Select {
                        options: vec!["Slide".into(), "Fade".into(), "Pop".into(), "Bounce".into()],
                    },
                    default_value: serde_json::json!("Slide"),
                },
                SettingField {
                    key: "chat_frame_style".into(),
                    label: "Frame Style".into(),
                    description: Some("Add decorative elements to messages.".into()),
                    field_type: SettingFieldType::Select {
                        options: vec!["None".into(), "Glass".into(), "Neon".into(), "Retro".into()],
                    },
                    default_value: serde_json::json!("None"),
                },
                SettingField {
                    key: "chat_background_blur".into(),
                    label: "Background Blur (px)".into(),
                    description: Some("Glass-morphism effect intensity.".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(0),
                },
                SettingField {
                    key: "show_pronouns".into(),
                    label: "Show Pronouns".into(),
                    description: None,
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(true),
                },
                SettingField {
                    key: "show_badges".into(),
                    label: "Show Badges".into(),
                    description: None,
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(true),
                },
                SettingField {
                    key: "max_messages".into(),
                    label: "Max Messages".into(),
                    description: None,
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(50),
                },
                SettingField {
                    key: "test_chat".into(),
                    label: "Test Chat".into(),
                    description: None,
                    field_type: SettingFieldType::Action {
                        button_label: "Simulate Message".into(),
                        action_name: "test_msg".into(),
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
        use tauri::Emitter;
        if action == "test_msg" {
            let app_handle = app.clone();
            let manager = self.manager.clone();
            tauri::async_runtime::spawn(async move {
                let info = { manager.broadcaster_info.lock().unwrap().clone() };
                let msg = ChatMessageData {
                    id: format!("test_{}", rand::random::<u16>()),
                    user_id: if info.id.is_empty() { "123".into() } else { info.id },
                    user_login: if info.login.is_empty() { "streamer".into() } else { info.login },
                    user_name: if info.display_name.is_empty() {
                        "Streamer".into()
                    } else {
                        info.display_name
                    },
                    message: "This is a test message! 🚀".to_string(),
                    fragments: vec![
                        ChatFragment::Text("This is a test message! ".to_string()),
                        ChatFragment::Emote {
                            id: "1".into(),
                            text: "🚀".into(),
                            url: "https://static-cdn.jtvnw.net/emoticons/v2/1/default/dark/1.0"
                                .into(),
                        },
                    ],
                    color: "#8b5cf6".to_string(),
                    pronouns: Some("they/them".to_string()),
                    badges: vec![ChatBadge {
                        set_id: "broadcaster".to_string(),
                        id: "1".to_string(),
                        info: String::new(),
                        image_url: Some(
                            "https://static-cdn.jtvnw.net/chat/badges/5527358c-052c-4c76-8251-0f8d99bc732e/1"
                                .to_string(),
                        ),
                    }],
                    is_mod: false,
                    is_sub: false,
                    is_vip: false,
                    is_broadcaster: true,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                let _ = app_handle.emit("twitch_chat_message", msg.clone());
                let mut state = manager.chat_state.lock().unwrap();
                state.messages.push(msg);
            });
            Ok(serde_json::json!({ "status": "sent" }))
        } else {
            Err("Unknown action".to_string())
        }
    }
}
