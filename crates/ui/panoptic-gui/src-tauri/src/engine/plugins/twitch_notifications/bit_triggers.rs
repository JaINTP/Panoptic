use panoptic_core::{
    PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField, SettingFieldType,
};

pub struct TwitchBitTriggersPlugin;

impl TwitchBitTriggersPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TwitchBitTriggersPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PanopticPlugin for TwitchBitTriggersPlugin {
    fn id(&self) -> &'static str {
        "twitch_bit_triggers"
    }

    fn name(&self) -> &'static str {
        "Interactive Bit Triggers"
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::General,
            fields: vec![
                SettingField {
                    key: "enable".into(),
                    label: "Enable Bit Triggers".into(),
                    description: Some(
                        "Trigger overlay visual effects when viewers cheer bits".into(),
                    ),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(true),
                },
                SettingField {
                    key: "glow_threshold".into(),
                    label: "Glow Effect Threshold (Bits)".into(),
                    description: Some("Minimum bits to trigger a glow effect".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(100),
                },
                SettingField {
                    key: "glitch_threshold".into(),
                    label: "Glitch Effect Threshold (Bits)".into(),
                    description: Some("Minimum bits to trigger a glitch effect".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(500),
                },
                SettingField {
                    key: "chaos_threshold".into(),
                    label: "Chaos Mode Threshold (Bits)".into(),
                    description: Some(
                        "Minimum bits to trigger both glow and glitch effects for 10s".into(),
                    ),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(1000),
                },
                SettingField {
                    key: "action_simulate_glow".into(),
                    label: "Simulate Glow Effect".into(),
                    description: Some("Test the Glow overlay animation".into()),
                    field_type: SettingFieldType::Action {
                        button_label: "Test Glow".into(),
                        action_name: "simulate_glow".into(),
                    },
                    default_value: serde_json::json!(null),
                },
                SettingField {
                    key: "action_simulate_glitch".into(),
                    label: "Simulate Glitch Effect".into(),
                    description: Some("Test the Glitch overlay animation".into()),
                    field_type: SettingFieldType::Action {
                        button_label: "Test Glitch".into(),
                        action_name: "simulate_glitch".into(),
                    },
                    default_value: serde_json::json!(null),
                },
                SettingField {
                    key: "action_simulate_chaos".into(),
                    label: "Simulate Chaos Mode".into(),
                    description: Some("Test both Glow and Glitch animations together".into()),
                    field_type: SettingFieldType::Action {
                        button_label: "Test Chaos".into(),
                        action_name: "simulate_chaos".into(),
                    },
                    default_value: serde_json::json!(null),
                },
            ],
        })
    }

    fn handle_action(
        &self,
        action: &str,
        app: &tauri::AppHandle,
    ) -> Result<serde_json::Value, String> {
        use crate::engine::settings::AppSettings;
        let settings = AppSettings::load(app);
        let plugin_cfg = settings.plugins.get("twitch_bit_triggers");

        let glow_threshold = plugin_cfg
            .and_then(|v| v.get("glow_threshold"))
            .and_then(|v| v.as_u64())
            .unwrap_or(100);

        let glitch_threshold = plugin_cfg
            .and_then(|v| v.get("glitch_threshold"))
            .and_then(|v| v.as_u64())
            .unwrap_or(500);

        let chaos_threshold = plugin_cfg
            .and_then(|v| v.get("chaos_threshold"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1000);

        match action {
            "simulate_glow" => {
                super::websocket::process_bit_triggers(app, glow_threshold);
                Ok(serde_json::json!({ "status": "success" }))
            }
            "simulate_glitch" => {
                super::websocket::process_bit_triggers(app, glitch_threshold);
                Ok(serde_json::json!({ "status": "success" }))
            }
            "simulate_chaos" => {
                super::websocket::process_bit_triggers(app, chaos_threshold);
                Ok(serde_json::json!({ "status": "success" }))
            }
            _ => Err(format!("Unknown action: {}", action)),
        }
    }
}
