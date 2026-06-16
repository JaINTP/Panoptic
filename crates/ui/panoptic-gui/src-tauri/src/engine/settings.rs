use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub client_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub template: Option<String>,
    pub not_playing_title: Option<String>,
    pub not_playing_artist: Option<String>,
    pub not_playing_album: Option<String>,

    #[serde(default)]
    pub plugins: std::collections::HashMap<String, serde_json::Value>,
}

impl AppSettings {
    pub fn get_path(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
        let config_dir = app_handle.path().app_config_dir().ok()?;
        // Ensure config directory exists
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        Some(config_dir.join("settings.json"))
    }

    pub fn get_overlay_css_path(app_handle: &tauri::AppHandle, id: &str) -> Option<PathBuf> {
        let config_dir = app_handle.path().app_config_dir().ok()?;
        let overlays_dir = config_dir.join("overlays");
        if !overlays_dir.exists() {
            let _ = fs::create_dir_all(&overlays_dir);
        }
        Some(overlays_dir.join(format!("{}.css", id)))
    }

    pub fn load(app_handle: &tauri::AppHandle) -> Self {
        let Some(path) = Self::get_path(app_handle) else {
            error!("Could not resolve app config directory for loading settings");
            return Self::default();
        };

        if !path.exists() {
            info!("Settings file not found at {:?}, using defaults", path);
            return Self::default();
        }

        let Ok(content) = fs::read_to_string(&path) else {
            error!("Failed to read settings file at {:?}", path);
            return Self::default();
        };

        match serde_json::from_str::<AppSettings>(&content) {
            Ok(s) => {
                info!("Settings loaded successfully from {:?}", path);
                s
            }
            Err(e) => {
                error!("Failed to parse settings file at {:?}: {}", path, e);
                Self::default()
            }
        }
    }

    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        let path = Self::get_path(app_handle).ok_or_else(|| {
            error!("Could not resolve app config directory for saving settings");
            "Could not resolve app config directory".to_string()
        })?;

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            let err = format!("Failed to serialize settings: {}", e);
            error!("{}", err);
            err
        })?;

        fs::write(&path, content).map_err(|e| {
            let err = format!("Failed to write settings file to {:?}: {}", path, e);
            error!("{}", err);
            err
        })?;

        info!("Settings saved successfully to {:?}", path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_settings_serialization() {
        let settings = AppSettings {
            client_id: Some("my-client-id".to_string()),
            access_token: Some("my-access-token".to_string()),
            refresh_token: Some("my-refresh-token".to_string()),
            template: Some("Now Playing: {title}".to_string()),
            not_playing_title: Some("Nothing Playing".to_string()),
            not_playing_artist: Some("No One".to_string()),
            not_playing_album: Some("Void Album".to_string()),
            plugins: Default::default(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let decoded: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.client_id, Some("my-client-id".to_string()));
        assert_eq!(decoded.access_token, Some("my-access-token".to_string()));
        assert_eq!(decoded.refresh_token, Some("my-refresh-token".to_string()));
        assert_eq!(decoded.template, Some("Now Playing: {title}".to_string()));
        assert_eq!(
            decoded.not_playing_title,
            Some("Nothing Playing".to_string())
        );
        assert_eq!(decoded.not_playing_artist, Some("No One".to_string()));
        assert_eq!(decoded.not_playing_album, Some("Void Album".to_string()));
    }

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.client_id.is_none());
        assert!(settings.access_token.is_none());
        assert!(settings.refresh_token.is_none());
        assert!(settings.template.is_none());
        assert!(settings.not_playing_title.is_none());
        assert!(settings.not_playing_artist.is_none());
        assert!(settings.not_playing_album.is_none());
    }
}
