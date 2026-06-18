use tracing::{info, warn};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
}

#[derive(Clone)]
pub struct UpdateStatus(pub std::sync::Arc<std::sync::Mutex<Option<GitHubRelease>>>);

pub fn is_newer(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect()
    };
    parse(latest) > parse(current)
}

pub async fn check_latest_release() -> Result<GitHubRelease, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/repos/JaINTP/Panoptic/releases/latest")
        .header("User-Agent", "Panoptic")
        .send()
        .await
        .map_err(|e| format!("Failed to send update request: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("GitHub API returned error: {}", res.status()));
    }

    res.json::<GitHubRelease>()
        .await
        .map_err(|e| format!("Failed to parse release JSON: {}", e))
}

pub async fn spawn_update_check(
    app_handle: tauri::AppHandle,
    update_status: UpdateStatus,
    menu: tauri::menu::Menu<tauri::Wry>,
) {
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    match check_latest_release().await {
        Ok(release) => {
            let current_ver = env!("CARGO_PKG_VERSION");
            if is_newer(current_ver, &release.tag_name) {
                info!("Update available: {} -> {}", current_ver, release.tag_name);
                if let Ok(mut lock) = update_status.0.lock() {
                    *lock = Some(release.clone());
                }
                if let Ok(update_i) = tauri::menu::MenuItem::with_id(
                    &app_handle,
                    "update",
                    format!("Update Available ({})", release.tag_name),
                    true,
                    None::<&str>,
                ) {
                    let _ = menu.prepend(&update_i);
                }
                use tauri::Emitter;
                let _ = app_handle.emit("update_available", release);
            }
        }
        Err(e) => warn!("Failed to check for updates: {}", e),
    }
}

#[tauri::command]
pub fn get_update_status(
    status: tauri::State<'_, UpdateStatus>,
) -> Result<Option<GitHubRelease>, String> {
    let lock = status.0.lock().map_err(|e| e.to_string())?;
    Ok(lock.clone())
}
