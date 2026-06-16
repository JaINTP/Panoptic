use crate::engine::native::create_native_provider;
use panoptic_core::{MediaProvider, PanopticPlugin, PluginSettingsDefinition};

pub struct NativeMediaPlugin;

impl NativeMediaPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NativeMediaPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PanopticPlugin for NativeMediaPlugin {
    fn id(&self) -> &'static str {
        "native_media"
    }

    fn name(&self) -> &'static str {
        "Native Media"
    }

    fn media_provider(&self) -> Option<Box<dyn MediaProvider>> {
        Some(create_native_provider())
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        // No custom settings fields needed for native media right now,
        // but we could add controls to toggle specific player integrations later.
        None
    }
}
