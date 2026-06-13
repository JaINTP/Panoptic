pub mod engine {
    pub mod native;
    pub mod orchestrator;
    pub mod pkce;
    pub mod settings;
}

use crate::engine::orchestrator::{AppCommand, EngineOrchestrator};
use panoptic_core::{AuthState, PlaybackState};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::Manager;
use tokio::sync::{mpsc, watch};

use crate::engine::settings::AppSettings;

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
fn get_output_template(app: tauri::AppHandle) -> Result<String, String> {
    let settings = AppSettings::load(&app);
    Ok(settings.template.unwrap_or_default())
}

#[tauri::command]
fn set_output_template(app: tauri::AppHandle, template: String) -> Result<(), String> {
    let mut settings = AppSettings::load(&app);
    settings.template = Some(template);
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

    let auth_tx_state = auth_tx.clone();
    let auth_rx_state = auth_rx.clone();

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
            set_output_template
        ])
        .setup(move |app| {
            app.manage(cmd_tx_state);
            app.manage(auth_tx_state);
            app.manage(auth_rx_state);
            let app_handle = app.handle().clone();

            // Spawn the Engine Orchestrator
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(orchestrator.run(app_handle));
            });

            // Set up System Tray
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(move |_app, event| {
                    if event.id == quit_i.id() {
                        let _ = cmd_tx.blocking_send(AppCommand::Quit);
                    } else if event.id == settings_i.id() {
                        let _ = cmd_tx.blocking_send(AppCommand::OpenSettings);
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
