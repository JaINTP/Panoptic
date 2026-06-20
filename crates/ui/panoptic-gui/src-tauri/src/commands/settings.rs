use crate::engine::settings::AppSettings;
use tauri::Manager;

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

#[tauri::command]
pub fn get_storage_paths(app: tauri::AppHandle) -> serde_json::Value {
    let config_dir = app
        .path()
        .app_config_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    let artwork_dir = app
        .path()
        .app_cache_dir()
        .map(|mut p| {
            p.push("artworks");
            p.to_string_lossy().to_string()
        })
        .unwrap_or_default();

    serde_json::json!({
        "config_dir": config_dir,
        "artwork_dir": artwork_dir,
    })
}

#[tauri::command]
pub fn open_directory(app: tauri::AppHandle, path: String) -> Result<(), String> {
    use std::path::Path;
    let path_buf = Path::new(&path);
    if !path_buf.exists() {
        if let Err(e) = std::fs::create_dir_all(path_buf) {
            return Err(format!("Failed to create directory: {}", e));
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try calling the freedesktop FileManager1 D-Bus interface using dbus-send.
        // This opens the directory in the registered graphical file manager directly,
        // avoiding incorrect MIME associations (like opening a console) and QDBusErrors
        // associated with direct process spawning of Dolphin.
        let uri = format!("file://{}", path_buf.to_string_lossy());
        if std::process::Command::new("dbus-send")
            .args([
                "--session",
                "--dest=org.freedesktop.FileManager1",
                "/org/freedesktop/FileManager1",
                "org.freedesktop.FileManager1.ShowFolders",
                &format!("array:string:{}", uri),
                "string:",
            ])
            .spawn()
            .is_ok()
        {
            return Ok(());
        }
    }

    // Fallback to tauri-plugin-opener (which calls ShellExecute on Windows, open on macOS, xdg-open on Linux)
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}
