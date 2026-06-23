use axum::{extract::State, http::header, response::IntoResponse, routing::get, Router};
use panoptic_core::{
    AppState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};

pub struct AvatarPlugin;

impl AvatarPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AvatarPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PanopticPlugin for AvatarPlugin {
    fn id(&self) -> &'static str {
        "avatar"
    }

    fn name(&self) -> &'static str {
        "Talk-Back Avatar"
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField {
                    key: "enable".into(),
                    label: "Enable Talk-Back Avatar".into(),
                    description: Some(
                        "Turn the microphone-reactive avatar overlay on or off".into(),
                    ),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(true),
                },
                SettingField {
                    key: "sensitivity".into(),
                    label: "Microphone Sensitivity".into(),
                    description: Some("Amplification multiplier for your microphone level".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(1.5),
                },
                SettingField {
                    key: "speaking_threshold".into(),
                    label: "Speaking Threshold (Noise Gate)".into(),
                    description: Some("Volume level required to trigger active animations".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(10),
                },
                SettingField {
                    key: "bounce_height".into(),
                    label: "Bounce Height (Pixels)".into(),
                    description: Some("Maximum height the avatar will bounce up in pixels".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(20),
                },
                SettingField {
                    key: "avatar_style".into(),
                    label: "Avatar Animation Style".into(),
                    description: Some("Select the visual response style when speaking".into()),
                    field_type: SettingFieldType::Select {
                        options: vec!["Bounce".into(), "Scale".into(), "Mouth-Open".into()],
                    },
                    default_value: serde_json::json!("Bounce"),
                },
                SettingField {
                    key: "avatar_color".into(),
                    label: "Idle Avatar Color".into(),
                    description: Some("CSS hex color for the idle vector robot mascot".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("#8b5cf6"),
                },
                SettingField {
                    key: "speaking_color".into(),
                    label: "Speaking Glow Color".into(),
                    description: Some("CSS hex color for the active glow effect".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("#ff007f"),
                },
                SettingField {
                    key: "custom_image_url".into(),
                    label: "Custom PNG Avatar Image URL".into(),
                    description: Some("Optional URL to a custom PNG/GIF avatar image".into()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!(""),
                },
            ],
        })
    }

    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        router
            .route("/overlay/avatar", get(get_avatar_overlay))
            .route("/avatar/settings", get(get_avatar_settings))
    }
}

// ---------------------------------------------------------------------------
// HTTP handlers
// ---------------------------------------------------------------------------

pub async fn get_avatar_overlay() -> impl IntoResponse {
    let html = include_str!("../../../../../../../crates/services/panoptic-server/src/avatar.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

pub async fn get_avatar_settings(State(state): State<AppState>) -> impl IntoResponse {
    let settings = if let Some(ref path) = state.settings_path {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    val.get("plugins")
                        .and_then(|p| p.get("avatar"))
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}))
                } else {
                    serde_json::json!({})
                }
            } else {
                serde_json::json!({})
            }
        } else {
            serde_json::json!({})
        }
    } else {
        serde_json::json!({})
    };

    let final_settings = serde_json::json!({
        "enable": settings.get("enable").and_then(|v| v.as_bool()).unwrap_or(true),
        "sensitivity": settings.get("sensitivity").and_then(|v| v.as_f64()).unwrap_or(1.5),
        "speaking_threshold": settings.get("speaking_threshold").and_then(|v| v.as_f64()).unwrap_or(10.0),
        "bounce_height": settings.get("bounce_height").and_then(|v| v.as_f64()).unwrap_or(20.0),
        "avatar_style": settings.get("avatar_style").and_then(|v| v.as_str()).unwrap_or("Bounce"),
        "avatar_color": settings.get("avatar_color").and_then(|v| v.as_str()).unwrap_or("#8b5cf6"),
        "speaking_color": settings.get("speaking_color").and_then(|v| v.as_str()).unwrap_or("#ff007f"),
        "custom_image_url": settings.get("custom_image_url").and_then(|v| v.as_str()).unwrap_or(""),
    });

    axum::Json(final_settings)
}
