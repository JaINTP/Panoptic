pub mod engine {
    pub mod native;
    pub mod orchestrator;
    pub mod pkce;
    pub mod plugin_registry;
    pub mod settings;
    pub mod plugins {
        pub mod mpris_smtc;
        pub mod spotify;
        pub mod twitch;
        pub mod twitch_notifications;
    }
}

use crate::engine::orchestrator::{AppCommand, EngineOrchestrator};
use crate::engine::settings::AppSettings;
use panoptic_core::{AuthState, PlaybackState};
use std::sync::Arc;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tokio::sync::{mpsc, watch};
use tracing::{info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn init_logging(app_handle: &tauri::AppHandle) -> Result<(), String> {
    let log_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| format!("Failed to get config dir: {}", e))?;

    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir)
            .map_err(|e| format!("Failed to create log dir: {}", e))?;
    }

    let file_appender = tracing_appender::rolling::never(log_dir.clone(), "error.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Keep the guard alive for the duration of the program
    Box::leak(Box::new(_guard));

    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true);

    let console_layer = fmt::layer().with_target(true).with_ansi(true);

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(console_layer)
        .with(file_layer)
        .init();

    info!("Logging initialized. Logs are saved to: {:?}", log_dir);
    Ok(())
}

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
fn get_plugins_metadata(
    plugins: tauri::State<'_, std::sync::Arc<Vec<Box<dyn panoptic_core::PanopticPlugin>>>>,
) -> Result<Vec<panoptic_core::PluginMetadata>, String> {
    let mut meta = Vec::new();
    for plugin in plugins.iter() {
        let def = plugin.settings_definition();
        meta.push(panoptic_core::PluginMetadata {
            id: plugin.id().to_string(),
            name: plugin.name().to_string(),
            category: def.as_ref().map(|d| d.category.clone()),
            fields: def.map(|d| d.fields).unwrap_or_default(),
        });
    }
    Ok(meta)
}

#[tauri::command]
fn get_plugin_settings(
    app: tauri::AppHandle,
    plugin_id: String,
) -> Result<serde_json::Value, String> {
    let settings = AppSettings::load(&app);
    let val = settings
        .plugins
        .get(&plugin_id)
        .cloned()
        .unwrap_or(serde_json::Value::Object(Default::default()));
    Ok(val)
}

#[tauri::command]
fn set_plugin_settings(
    app: tauri::AppHandle,
    plugin_id: String,
    new_settings: serde_json::Value,
) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.plugins.insert(plugin_id, new_settings);
    settings.save(&app)
}

#[tauri::command]
async fn trigger_plugin_action(
    app: tauri::AppHandle,
    plugins: tauri::State<'_, std::sync::Arc<Vec<Box<dyn panoptic_core::PanopticPlugin>>>>,
    plugin_id: String,
    action_name: String,
) -> Result<serde_json::Value, String> {
    for plugin in plugins.iter() {
        if plugin.id() == plugin_id {
            return plugin.handle_action(&action_name, &app);
        }
    }
    Err(format!("Plugin '{}' not found", plugin_id))
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

use std::sync::atomic::{AtomicU32, Ordering};
static CSS_VERSION: AtomicU32 = AtomicU32::new(1);

#[tauri::command]
fn get_overlay_css(app: tauri::AppHandle, id: String) -> Result<String, String> {
    let path = AppSettings::get_overlay_css_path(&app, &id)
        .ok_or_else(|| "Could not resolve overlay CSS path".to_string())?;

    if !path.exists() {
        // Return default theme based on ID
        if id == "now_playing" {
            return Ok(
                include_str!("../../../../../examples/now-playing/now-playing-default.css")
                    .to_string(),
            );
        } else if id == "twitch_hype_train" {
            return Ok(include_str!(
                "../../../../../examples/twitch-hype-train/hype-train-default.css"
            )
            .to_string());
        } else if id == "twitch_alerts" {
            return Ok(include_str!(
                "../../../../../examples/twitch-alerts/twitch-alerts-default.css"
            )
            .to_string());
        }
        return Ok("".to_string());
    }

    std::fs::read_to_string(path).map_err(|e| format!("Failed to read CSS file: {}", e))
}

#[tauri::command]
fn set_overlay_css(
    app: tauri::AppHandle,
    css_version_tx: tauri::State<'_, watch::Sender<u32>>,
    id: String,
    css: String,
) -> Result<(), String> {
    let path = AppSettings::get_overlay_css_path(&app, &id)
        .ok_or_else(|| "Could not resolve overlay CSS path".to_string())?;

    std::fs::write(path, css).map_err(|e| format!("Failed to write CSS file: {}", e))?;

    // Increment version and notify overlays
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
fn get_overlay_version() -> Result<u32, String> {
    Ok(CSS_VERSION.load(Ordering::SeqCst))
}

#[tauri::command]
fn get_not_playing_settings(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    let settings = AppSettings::load(&app);
    Ok(serde_json::json!({
        "not_playing_title": settings.not_playing_title.unwrap_or_else(|| "Not Playing".to_string()),
        "not_playing_artist": settings.not_playing_artist.unwrap_or_else(|| "Unknown Artist".to_string()),
        "not_playing_album": settings.not_playing_album.unwrap_or_else(|| "Unknown Album".to_string()),
    }))
}

#[tauri::command]
fn set_not_playing_settings(
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (cmd_tx, cmd_rx) = mpsc::channel::<AppCommand>(32);
    let (state_tx, state_rx) = watch::channel(PlaybackState::default());
    let (auth_tx, auth_rx) = watch::channel(AuthState::Unauthenticated);
    let (css_version_tx, css_version_rx) = watch::channel(1u32);
    let update_status = UpdateStatus(std::sync::Arc::new(std::sync::Mutex::new(None)));

    // Initialize Plugin Registry
    let twitch_manager =
        Arc::new(crate::engine::plugins::twitch_notifications::TwitchEventManager::new());
    let registry = crate::engine::plugin_registry::PluginRegistry::new()
        .register(Box::new(
            crate::engine::plugins::spotify::SpotifyPlugin::new(),
        ))
        .register(Box::new(
            crate::engine::plugins::mpris_smtc::NativeMediaPlugin::new(),
        ))
        .register(Box::new(crate::engine::plugins::twitch::TwitchPlugin::new()))
        .register(Box::new(
            crate::engine::plugins::twitch_notifications::TwitchHypeTrainPlugin::new(
                twitch_manager.clone(),
            ),
        ))
        .register(Box::new(
            crate::engine::plugins::twitch_notifications::TwitchAlertsPlugin::new(twitch_manager),
        ));

    let plugins = registry.plugins;

    let auth_tx_state = auth_tx.clone();
    let auth_rx_state = auth_rx.clone();
    let update_status_state = update_status.clone();
    let update_status_tray = update_status.clone();

    let orchestrator = EngineOrchestrator::new(
        cmd_rx,
        state_tx,
        state_rx,
        auth_tx,
        css_version_rx,
        plugins.clone(),
    );
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
            get_plugins_metadata,
            get_plugin_settings,
            set_plugin_settings,
            trigger_plugin_action,
            get_output_template,
            set_output_template,
            get_overlay_css,
            set_overlay_css,
            get_overlay_version,
            get_not_playing_settings,
            set_not_playing_settings,
            get_update_status
        ])
        .setup(move |app| {
            // Initialize logging
            if let Err(e) = init_logging(app.handle()) {
                eprintln!("Failed to initialize logging: {}", e);
            }

            app.manage(cmd_tx_state);
            app.manage(auth_tx_state);
            app.manage(auth_rx_state);
            app.manage(update_status_state);
            app.manage(css_version_tx);
            app.manage(plugins);

            let orchestrator_handle = app.handle().clone();

            // Spawn the Engine Orchestrator
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(orchestrator.run(orchestrator_handle));
            });

            let app_handle = app.handle().clone();
            let quit_i = MenuItem::with_id(&app_handle, "quit", "Quit", true, None::<&str>)?;
            let settings_i =
                MenuItem::with_id(&app_handle, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(&app_handle, &[&settings_i, &quit_i])?;

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
                            info!("Update available: {} -> {}", current_ver, release.tag_name);
                            if let Ok(mut lock) = update_status_task.0.lock() {
                                *lock = Some(release.clone());
                            }
                            if let Ok(update_i) = MenuItem::with_id(
                                &app_handle_update,
                                "update",
                                format!("Update Available ({})", release.tag_name),
                                true,
                                None::<&str>,
                            ) {
                                let _ = menu_clone.prepend(&update_i);
                            }

                            use tauri::Emitter;
                            let _ = app_handle_update.emit("update_available", release);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to check for updates: {}", e);
                    }
                }
            });

            let update_status_tray_menu = update_status_tray.clone();
            let mut tray_builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false);

            if let Some(icon) = app.handle().default_window_icon() {
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
                        // info!("Tray icon hovered!");
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
