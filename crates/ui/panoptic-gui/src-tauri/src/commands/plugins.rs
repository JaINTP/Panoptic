use crate::engine::plugins::obs_websocket::ObsStatus;
use crate::engine::settings::AppSettings;
use panoptic_core::{PanopticPlugin, PluginMetadata};
use std::sync::Arc;

#[tauri::command]
pub fn get_plugins_metadata(
    plugins: tauri::State<'_, Arc<Vec<Box<dyn PanopticPlugin>>>>,
) -> Result<Vec<PluginMetadata>, String> {
    let mut meta = Vec::new();
    for plugin in plugins.iter() {
        let def = plugin.settings_definition();
        meta.push(PluginMetadata {
            id: plugin.id().to_string(),
            name: plugin.name().to_string(),
            category: def.as_ref().map(|d| d.category.clone()),
            fields: def.map(|d| d.fields).unwrap_or_default(),
        });
    }
    Ok(meta)
}

#[tauri::command]
pub fn get_plugin_settings(
    app: tauri::AppHandle,
    plugin_id: String,
) -> Result<serde_json::Value, String> {
    let settings = AppSettings::load(&app);
    let val = settings
        .plugins
        .get(&plugin_id)
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    Ok(val)
}

#[tauri::command]
pub fn set_plugin_settings(
    app: tauri::AppHandle,
    plugin_id: String,
    new_settings: serde_json::Value,
) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.plugins.insert(plugin_id.clone(), new_settings);
    settings.save(&app)?;
    use tauri::Emitter;
    let _ = app.emit("plugin_settings_updated", plugin_id);
    Ok(())
}

#[tauri::command]
pub fn get_obs_status(state: tauri::State<Arc<std::sync::Mutex<ObsStatus>>>) -> ObsStatus {
    state.lock().unwrap().clone()
}

#[tauri::command]
pub async fn trigger_plugin_action(
    app: tauri::AppHandle,
    plugins: tauri::State<'_, Arc<Vec<Box<dyn PanopticPlugin>>>>,
    plugin_id: String,
    action_name: String,
) -> Result<serde_json::Value, String> {
    for plugin in plugins.iter() {
        if plugin.id() == plugin_id {
            return plugin.handle_action(&action_name, &app);
        }
    }
    Err(format!("Plugin '{}' not found", plugin_id))
}
