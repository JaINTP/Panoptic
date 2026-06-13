use dirs;
use serde_yaml;
use std::fs;

pub struct SchemaBootstrapper;

const SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/sonallux/spotify-web-api/main/fixed-spotify-open-api.yml";

impl SchemaBootstrapper {
    pub async fn bootstrap() -> Option<serde_yaml::Value> {
        let cache_dir = dirs::cache_dir()?.join("panoptic");
        let schema_path = cache_dir.join("fixed-spotify-open-api.yml");

        if let Ok(response) = reqwest::get(SCHEMA_URL).await {
            if let Ok(content) = response.text().await {
                let _ = fs::create_dir_all(&cache_dir);
                let _ = fs::write(&schema_path, &content);
                return serde_yaml::from_str(&content).ok();
            }
        }

        let content = fs::read_to_string(&schema_path).ok()?;
        serde_yaml::from_str(&content).ok()
    }
}
