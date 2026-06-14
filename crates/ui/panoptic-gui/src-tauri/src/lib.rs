pub mod engine {
    pub mod native;
    pub mod orchestrator;
    pub mod pkce;
    pub mod settings;
}

use crate::engine::orchestrator::{AppCommand, EngineOrchestrator};
use crate::engine::settings::AppSettings;
use panoptic_core::{AuthState, PlaybackState};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tokio::sync::{mpsc, watch};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
}

#[derive(Clone)]
pub struct UpdateStatus(pub std::sync::Arc<std::sync::Mutex<Option<GitHubRelease>>>);

fn is_newer(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect()
    };
    let c = parse(current);
    let l = parse(latest);
    l > c
}

async fn check_latest_release() -> Result<GitHubRelease, String> {
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

    let release: GitHubRelease = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse release JSON: {}", e))?;

    Ok(release)
}

#[tauri::command]
fn get_update_status(
    status: tauri::State<'_, UpdateStatus>,
) -> Result<Option<GitHubRelease>, String> {
    let lock = status.0.lock().map_err(|e| e.to_string())?;
    Ok(lock.clone())
}

#[tauri::command]
async fn link_spotify(state: tauri::State<'_, mpsc::Sender<AppCommand>>) -> Result<(), String> {
    state
        .send(AppCommand::InitiateAuth)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_spotify_client_id(app: tauri::AppHandle) -> Result<String, String> {
    let settings = AppSettings::load(&app);
    Ok(settings.client_id.unwrap_or_default())
}

#[tauri::command]
fn set_spotify_client_id(app: tauri::AppHandle, client_id: String) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.client_id = Some(client_id);
    settings.save(&app)
}

#[tauri::command]
async fn unlink_spotify(
    app: tauri::AppHandle,
    auth_state_tx: tauri::State<'_, watch::Sender<AuthState>>,
) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.access_token = None;
    settings.refresh_token = None;
    settings.save(&app)?;

    let _ = auth_state_tx.send(AuthState::Unauthenticated);
    Ok(())
}

#[tauri::command]
fn get_output_template(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let settings = AppSettings::load(&app);
    Ok(settings.template)
}

#[tauri::command]
fn set_output_template(app: tauri::AppHandle, template: String) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.template = Some(template);
    settings.save(&app)
}

#[tauri::command]
fn get_overlay_css(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let settings = AppSettings::load(&app);
    Ok(settings.css)
}

#[tauri::command]
fn set_overlay_css(app: tauri::AppHandle, css: String) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.css = Some(css);
    settings.save(&app)
}

#[tauri::command]
fn get_spotify_status(
    auth_state_rx: tauri::State<'_, watch::Receiver<AuthState>>,
) -> Result<bool, String> {
    let current = auth_state_rx.borrow().clone();
    Ok(matches!(current, AuthState::Authenticated { .. }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (cmd_tx, cmd_rx) = mpsc::channel::<AppCommand>(32);
    let (state_tx, state_rx) = watch::channel(PlaybackState::default());
    let (auth_tx, auth_rx) = watch::channel(AuthState::Unauthenticated);
    let update_status = UpdateStatus(std::sync::Arc::new(std::sync::Mutex::new(None)));

    let auth_tx_state = auth_tx.clone();
    let auth_rx_state = auth_rx.clone();
    let update_status_state = update_status.clone();
    let update_status_tray = update_status.clone();

    let orchestrator = EngineOrchestrator::new(cmd_rx, state_tx, state_rx, auth_tx, auth_rx);
    let cmd_tx_state = cmd_tx.clone();

    tauri::Builder::default()
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            link_spotify,
            unlink_spotify,
            get_spotify_client_id,
            set_spotify_client_id,
            get_spotify_status,
            get_output_template,
            set_output_template,
            get_overlay_css,
            set_overlay_css,
            get_update_status
        ])
        .setup(move |app| {
            app.manage(cmd_tx_state);
            app.manage(auth_tx_state);
            app.manage(auth_rx_state);
            app.manage(update_status_state);
            let app_handle = app.handle().clone();

            // Spawn the Engine Orchestrator
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(orchestrator.run(app_handle));
            });

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;

            let menu_clone = menu.clone();
            let update_status_task = update_status_tray.clone();
            let app_handle_update = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                // Wait a short bit after startup before checking
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                match check_latest_release().await {
                    Ok(release) => {
                        let current_ver = env!("CARGO_PKG_VERSION");
                        if is_newer(current_ver, &release.tag_name) {
                            println!("Update available: {} -> {}", current_ver, release.tag_name);
                            if let Ok(mut lock) = update_status_task.0.lock() {
                                *lock = Some(release.clone());
                            }
                            if let Ok(update_i) = MenuItem::with_id(&app_handle_update, "update", format!("Update Available ({})", release.tag_name), true, None::<&str>) {
                                let _ = menu_clone.prepend(&update_i);
                            }

                            use tauri::Emitter;
                            let _ = app_handle_update.emit("update_available", release);
                        }
                    }
                    Err(e) => {
                        println!("Failed to check for updates: {}", e);
                    }
                }
            });

            let update_status_tray_menu = update_status_tray.clone();
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false);

            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }

            let _tray = tray_builder
                .on_menu_event(move |app, event| {
                    if event.id == quit_i.id() {
                        let _ = cmd_tx.blocking_send(AppCommand::Quit);
                    } else if event.id == settings_i.id() {
                        let _ = cmd_tx.blocking_send(AppCommand::OpenSettings);
                    } else if event.id == "update" {
                        if let Ok(lock) = update_status_tray_menu.0.lock() {
                            if let Some(ref release) = *lock {
                                use tauri_plugin_opener::OpenerExt;
                                let _ = app.opener().open_url(&release.html_url, None::<&str>);
                            }
                        }
                    }
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Enter { .. } = event {
                        println!("Tray icon hovered!");
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
