use crate::engine::native::create_native_provider;
use panoptic_core::{MediaProvider, PanopticPlugin, PluginSettingsDefinition};
#[cfg(target_os = "windows")]
use tauri::Manager;

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

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        if let Ok(mut cache_dir) = app.path().app_cache_dir() {
            cache_dir.push("artworks");
            panoptic_provider_windows::set_art_cache_dir(cache_dir);
        }
        #[cfg(not(target_os = "windows"))]
        let _ = app;
        Ok(())
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        // No custom settings fields needed for native media right now,
        // but we could add controls to toggle specific player integrations later.
        None
    }
}
