use crate::engine::plugins::stream_goals::{CustomVar, GoalConfig, StreamGoalsSettings};
use crate::engine::plugins::twitch_notifications::{models::SessionStats, TwitchEventManager};
use crate::engine::settings::AppSettings;
use std::sync::Arc;

/// Return the current live session statistics (followers gained, bits cheered, etc.).
/// These counters reset on stream start or when the user calls `reset_stream_goals_session`.
#[tauri::command]
pub fn get_session_stats(
    manager: tauri::State<'_, Arc<TwitchEventManager>>,
) -> Result<SessionStats, String> {
    let stats = manager.session_stats.lock().unwrap().clone();
    Ok(stats)
}

/// Reset all session counters to zero (followers, subs, bits, raids, chats…).
#[tauri::command]
pub fn reset_stream_goals_session(
    app: tauri::AppHandle,
    manager: tauri::State<'_, Arc<TwitchEventManager>>,
) -> Result<(), String> {
    use tauri::Emitter;
    let mut stats = manager.session_stats.lock().unwrap();
    *stats = SessionStats::default();
    let _ = app.emit("session_stats_update", stats.clone());
    Ok(())
}

/// Return the current goals config and custom variable definitions.
#[tauri::command]
pub fn get_stream_goals_config(app: tauri::AppHandle) -> Result<StreamGoalsSettings, String> {
    let settings = AppSettings::load(&app);
    let sg: StreamGoalsSettings = settings
        .plugins
        .get("stream_goals")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();
    Ok(sg)
}

/// Persist the full goals configuration (array of GoalConfig).
#[tauri::command]
pub fn save_goals_config(
    app: tauri::AppHandle,
    goals: Vec<GoalConfig>,
) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    let raw = settings
        .plugins
        .entry("stream_goals".into())
        .or_insert_with(|| serde_json::json!({}));
    let mut sg: StreamGoalsSettings =
        serde_json::from_value(raw.clone()).unwrap_or_default();
    sg.goals = goals;
    *raw = serde_json::to_value(&sg).map_err(|e| e.to_string())?;
    settings.save(&app)
}

/// Persist the custom variable definitions (and their current values).
#[tauri::command]
pub fn save_custom_vars(
    app: tauri::AppHandle,
    custom_vars: Vec<CustomVar>,
) -> Result<(), String> {
    use tauri::Emitter;
    let mut settings = AppSettings::load(&app);
    let raw = settings
        .plugins
        .entry("stream_goals".into())
        .or_insert_with(|| serde_json::json!({}));
    let mut sg: StreamGoalsSettings =
        serde_json::from_value(raw.clone()).unwrap_or_default();
    sg.custom_vars = custom_vars.clone();
    *raw = serde_json::to_value(&sg).map_err(|e| e.to_string())?;
    settings.save(&app)?;
    let _ = app.emit("stream_goals_custom_var_update", &custom_vars);
    Ok(())
}

/// Increment, decrement, or reset a named custom variable by its step value.
/// `op` must be one of `"increment"`, `"decrement"`, or `"reset"`.
#[tauri::command]
pub fn update_custom_var(
    app: tauri::AppHandle,
    name: String,
    op: String,
) -> Result<f64, String> {
    use tauri::Emitter;
    let mut settings = AppSettings::load(&app);
    let raw = settings
        .plugins
        .entry("stream_goals".into())
        .or_insert_with(|| serde_json::json!({}));
    let mut sg: StreamGoalsSettings =
        serde_json::from_value(raw.clone()).unwrap_or_default();

    let cv = sg
        .custom_vars
        .iter_mut()
        .find(|v| v.name == name)
        .ok_or_else(|| format!("Custom var '{}' not found", name))?;

    match op.as_str() {
        "increment" => cv.value += cv.step,
        "decrement" => cv.value -= cv.step,
        "reset" => cv.value = 0.0,
        other => return Err(format!("Unknown op: {}", other)),
    }
    let new_val = cv.value;
    let updated_vars = sg.custom_vars.clone();
    *raw = serde_json::to_value(&sg).map_err(|e| e.to_string())?;
    settings.save(&app)?;
    let _ = app.emit("stream_goals_custom_var_update", &updated_vars);
    Ok(new_val)
}
