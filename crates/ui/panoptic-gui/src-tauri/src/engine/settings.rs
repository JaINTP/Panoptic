use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub client_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub template: Option<String>,
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

    pub fn load(app_handle: &tauri::AppHandle) -> Self {
        let Some(path) = Self::get_path(app_handle) else {
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        let Ok(content) = fs::read_to_string(&path) else {
            return Self::default();
        };

        serde_json::from_str::<AppSettings>(&content).unwrap_or_default()
    }

    pub fn save(&self, app_handle: &tauri::AppHandle) -> Result<(), String> {
        let path = Self::get_path(app_handle)
            .ok_or_else(|| "Could not resolve app config directory".to_string())?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write settings file: {}", e))?;

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
        };

        let json = serde_json::to_string(&settings).unwrap();
        let decoded: AppSettings = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded.client_id, Some("my-client-id".to_string()));
        assert_eq!(decoded.access_token, Some("my-access-token".to_string()));
        assert_eq!(decoded.refresh_token, Some("my-refresh-token".to_string()));
        assert_eq!(decoded.template, Some("Now Playing: {title}".to_string()));
    }

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.client_id.is_none());
        assert!(settings.access_token.is_none());
        assert!(settings.refresh_token.is_none());
        assert!(settings.template.is_none());
    }
}
