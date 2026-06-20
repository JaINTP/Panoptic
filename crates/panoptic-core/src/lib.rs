pub mod models {
    pub mod auth;
    pub mod playback;
}
pub mod traits {
    pub mod plugin;
    pub mod provider;
}

pub use models::auth::AppState;
pub use models::auth::AuthState;
pub use models::auth::ThematicEffects;
pub use models::playback::PlaybackState;
pub use traits::plugin::{
    PanopticPlugin, PluginCategory, PluginMetadata, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
pub use traits::provider::MediaProvider;
