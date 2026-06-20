use crate::engine::settings::AppSettings;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::watch;

pub static CSS_VERSION: AtomicU32 = AtomicU32::new(1);

fn default_css(id: &str) -> String {
    match id {
        "now_playing" => {
            include_str!("../../../../../../examples/now-playing/now-playing-default.css")
                .to_string()
        }
        "twitch_hype_train" => {
            include_str!("../../../../../../examples/twitch-hype-train/hype-train-default.css")
                .to_string()
        }
        "twitch_alerts" => {
            include_str!("../../../../../../examples/twitch-alerts/twitch-alerts-default.css")
                .to_string()
        }
        _ => String::new(),
    }
}

#[tauri::command]
pub fn get_overlay_css(app: tauri::AppHandle, id: String) -> Result<String, String> {
    let path = AppSettings::get_overlay_css_path(&app, &id)
        .ok_or_else(|| "Could not resolve overlay CSS path".to_string())?;

    if !path.exists() {
        return Ok(default_css(&id));
    }

    std::fs::read_to_string(path).map_err(|e| format!("Failed to read CSS file: {}", e))
}

#[tauri::command]
pub fn set_overlay_css(
    app: tauri::AppHandle,
    css_version_tx: tauri::State<'_, watch::Sender<u32>>,
    id: String,
    css: String,
) -> Result<(), String> {
    let path = AppSettings::get_overlay_css_path(&app, &id)
        .ok_or_else(|| "Could not resolve overlay CSS path".to_string())?;

    std::fs::write(path, css).map_err(|e| format!("Failed to write CSS file: {}", e))?;

    let new_version = CSS_VERSION.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = css_version_tx.send(new_version);

    use tauri::Emitter;
    let _ = app.emit(
        "css_updated",
        serde_json::json!({ "id": id, "version": new_version }),
    );

    Ok(())
}

#[tauri::command]
pub fn get_overlay_version() -> Result<u32, String> {
    Ok(CSS_VERSION.load(Ordering::SeqCst))
}
