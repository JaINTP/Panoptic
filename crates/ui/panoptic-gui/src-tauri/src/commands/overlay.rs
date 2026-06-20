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

#[tauri::command]
pub fn apply_aesthetic_pack(
    app: tauri::AppHandle,
    css_version_tx: tauri::State<'_, watch::Sender<u32>>,
    pack_id: String,
) -> Result<(), String> {
    let css_content = match pack_id.as_str() {
        "cyber" => include_str!("../../../../../../examples/themes/cyber_complete.css"),
        "eldritch" => include_str!("../../../../../../examples/themes/eldritch_complete.css"),
        "rpg90s" => include_str!("../../../../../../examples/themes/rpg90s_complete.css"),
        "salem" => include_str!("../../../../../../examples/themes/salem_cauldron_complete.css"),
        _ => return Err(format!("Unknown aesthetic pack: {}", pack_id)),
    };

    let overlays = [
        "now_playing",
        "twitch_hype_train",
        "twitch_alerts",
        "twitch_chat",
        "pomodoro",
        "stream_goals",
    ];

    for overlay_id in overlays {
        let path = AppSettings::get_overlay_css_path(&app, overlay_id)
            .ok_or_else(|| format!("Could not resolve CSS path for {}", overlay_id))?;
        std::fs::write(path, css_content)
            .map_err(|e| format!("Failed to write CSS for {}: {}", overlay_id, e))?;
    }

    let new_version = CSS_VERSION.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = css_version_tx.send(new_version);

    use tauri::Emitter;
    let _ = app.emit(
        "css_updated",
        serde_json::json!({ "id": "all", "version": new_version }),
    );

    Ok(())
}

pub fn process_thematic_filtering(app: &tauri::AppHandle, message: &str, is_sub: bool) {
    use tauri::Manager;
    if let Some(effects) = app.try_state::<panoptic_core::ThematicEffects>() {
        let mut active = effects.active.lock().unwrap();
        let now = std::time::Instant::now();
        let duration = std::time::Duration::from_secs(5);

        let msg_lower = message.to_lowercase();
        // 1. Keyword "flash" or "hype" triggers overlay flash effect
        if msg_lower.contains("flash") || msg_lower.contains("hype") {
            active.insert("flash".to_string(), now + duration);
            use tauri::Emitter;
            let _ = app.emit(
                "visual_effect_trigger",
                serde_json::json!({ "effect": "flash", "duration": 5 }),
            );
        }

        // 2. A subscription message or keyword "shift" / "color" triggers a color shift effect
        if is_sub
            || msg_lower.contains("shift")
            || msg_lower.contains("color")
            || msg_lower.contains("colour")
        {
            active.insert("color-shift".to_string(), now + duration);
            use tauri::Emitter;
            let _ = app.emit(
                "visual_effect_trigger",
                serde_json::json!({ "effect": "color-shift", "duration": 5 }),
            );
        }
    }
}
