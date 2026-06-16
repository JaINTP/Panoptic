use crate::traits::provider::MediaProvider;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "options")]
pub enum SettingFieldType {
    Text,
    Password,
    Number,
    Boolean,
    Select {
        options: Vec<String>,
    },
    Action {
        button_label: String,
        action_name: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SettingField {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
    pub field_type: SettingFieldType,
    pub default_value: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PluginCategory {
    Auth,
    Overlay,
    Output,
    Storage,
    General,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginSettingsDefinition {
    pub category: PluginCategory,
    pub fields: Vec<SettingField>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub category: Option<PluginCategory>,
    pub fields: Vec<SettingField>,
}

pub trait PanopticPlugin: Send + Sync {
    /// Unique identifier for the plugin (e.g. "spotify", "mpris", "twitch").
    fn id(&self) -> &'static str;

    /// Human-readable name of the plugin.
    fn name(&self) -> &'static str;

    /// Lifecycle hook: initialized when the Tauri app starts.
    #[cfg(feature = "plugin")]
    fn setup(&self, _app: &tauri::AppHandle) -> Result<(), String> {
        info!("Plugin '{}' setup complete", self.name());
        Ok(())
    }

    /// Optional: Provide a media ingestion provider to feed metadata into the core loop.
    fn media_provider(&self) -> Option<Box<dyn MediaProvider>> {
        None
    }

    /// Optional: Register custom HTTP endpoints to the local Axum server.
    #[cfg(feature = "plugin")]
    fn register_routes(
        &self,
        router: axum::Router<crate::AppState>,
    ) -> axum::Router<crate::AppState> {
        router
    }

    /// Define UI setting fields that should appear in the settings panel.
    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        None
    }

    /// Handle settings action clicks (e.g., triggering a PKCE login flow).
    #[cfg(feature = "plugin")]
    fn handle_action(
        &self,
        _action: &str,
        _app: &tauri::AppHandle,
    ) -> Result<serde_json::Value, String> {
        Err("Action handler not implemented".to_string())
    }
}
