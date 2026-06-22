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
            ],
        })
    }
}
