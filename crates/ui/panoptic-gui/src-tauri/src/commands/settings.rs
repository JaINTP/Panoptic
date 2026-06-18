use crate::engine::settings::AppSettings;

#[tauri::command]
pub fn get_output_template(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let settings = AppSettings::load(&app);
    Ok(settings.template)
}

#[tauri::command]
pub fn set_output_template(app: tauri::AppHandle, template: String) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.template = Some(template);
    settings.save(&app)
}

#[tauri::command]
pub fn get_not_playing_settings(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let settings = AppSettings::load(&app);
    Ok(serde_json::json!({
        "not_playing_title": settings.not_playing_title.unwrap_or_else(|| "Not Playing".to_string()),
        "not_playing_artist": settings.not_playing_artist.unwrap_or_else(|| "Unknown Artist".to_string()),
        "not_playing_album": settings.not_playing_album.unwrap_or_else(|| "Unknown Album".to_string()),
    }))
}

#[tauri::command]
pub fn set_not_playing_settings(
    app: tauri::AppHandle,
    not_playing_title: String,
    not_playing_artist: String,
    not_playing_album: String,
) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.not_playing_title = Some(not_playing_title);
    settings.not_playing_artist = Some(not_playing_artist);
    settings.not_playing_album = Some(not_playing_album);
    settings.save(&app)
}

#[tauri::command]
pub fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
