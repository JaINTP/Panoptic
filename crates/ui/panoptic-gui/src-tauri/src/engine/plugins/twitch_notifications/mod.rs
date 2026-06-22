pub mod alerts;
pub mod bit_triggers;
pub mod chat;
pub mod event_manager;
pub mod hype_train;
pub mod models;
pub mod websocket;

pub use alerts::TwitchAlertsPlugin;
pub use bit_triggers::TwitchBitTriggersPlugin;
pub use chat::TwitchChatPlugin;
pub use event_manager::TwitchEventManager;
pub use hype_train::TwitchHypeTrainPlugin;

use crate::engine::settings::AppSettings;

pub(crate) fn load_plugin_settings(
    settings_path: Option<std::path::PathBuf>,
    plugin_id: &str,
) -> serde_json::Value {
    settings_path
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
        .and_then(|s| s.plugins.get(plugin_id).cloned())
        .unwrap_or_else(|| serde_json::json!({}))
}
