pub mod commands;
pub mod engine {
    pub mod native;
    pub mod orchestrator;
    pub mod pkce;
    pub mod plugin_registry;
    pub mod settings;
    pub mod plugins {
        pub mod avatar;
        pub mod discord_rpc;
        pub mod mpris_smtc;
        pub mod obs_websocket;
        pub mod pomodoro;
        pub mod spotify;
        pub mod stream_goals;
        pub mod twitch;
        pub mod twitch_notifications;
    }
}
pub mod logging;
pub mod tray;
pub mod update;

use crate::commands::overlay::{
    apply_aesthetic_pack, get_overlay_css, get_overlay_version, set_overlay_css,
};
use crate::commands::plugins::{
    get_obs_status, get_plugin_settings, get_plugins_metadata, set_plugin_settings,
    trigger_plugin_action,
};
use crate::commands::settings::{
    get_app_version, get_not_playing_settings, get_output_template, get_storage_paths,
    open_directory, set_not_playing_settings, set_output_template,
};
use crate::commands::stream_goals::{
    get_session_stats, get_stream_goals_config, reset_stream_goals_session, save_custom_vars,
    save_goals_config, update_custom_var,
};
use crate::engine::orchestrator::{AppCommand, EngineOrchestrator};
use crate::update::{get_update_status, UpdateStatus};
use panoptic_core::{AuthState, PlaybackState};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::{mpsc, watch};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let (cmd_tx, cmd_rx) = mpsc::channel::<AppCommand>(32);
    let (state_tx, state_rx) = watch::channel(PlaybackState::default());
    let (auth_tx, auth_rx) = watch::channel(AuthState::Unauthenticated);
    let (css_version_tx, css_version_rx) = watch::channel(1u32);
    let update_status = UpdateStatus(Arc::new(std::sync::Mutex::new(None)));

    let twitch_manager =
        Arc::new(crate::engine::plugins::twitch_notifications::TwitchEventManager::new());

    // Build OBS plugin separately so we can expose its status to Tauri commands.
    let obs_plugin = crate::engine::plugins::obs_websocket::ObsWebsocketPlugin::new();
    let obs_status_arc = obs_plugin.status.clone();

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
            crate::engine::plugins::twitch_notifications::TwitchAlertsPlugin::new(
                twitch_manager.clone(),
            ),
        ))
        .register(Box::new(
            crate::engine::plugins::twitch_notifications::TwitchChatPlugin::new(
                twitch_manager.clone(),
            ),
        ))
        .register(Box::new(
            crate::engine::plugins::stream_goals::StreamGoalsPlugin::new(twitch_manager.clone()),
        ))
        .register(Box::new(obs_plugin))
        .register(Box::new(
            crate::engine::plugins::pomodoro::PomodoroPlugin::new(),
        ))
        .register(Box::new(
            crate::engine::plugins::twitch_notifications::TwitchBitTriggersPlugin::new(),
        ))
        .register(Box::new(
            crate::engine::plugins::discord_rpc::DiscordRpcPlugin::new(),
        ))
        .register(Box::new(crate::engine::plugins::avatar::AvatarPlugin::new()));

    let plugins = registry.plugins;
    let auth_tx_for_app = auth_tx.clone();
    let auth_rx_for_app = auth_rx.clone();
    let cmd_tx_for_app = cmd_tx.clone();
    let update_status_for_app = update_status.clone();
    let twitch_manager_for_app = twitch_manager.clone();

    let orchestrator = EngineOrchestrator::new(
        cmd_rx,
        state_tx,
        state_rx,
        auth_tx,
        css_version_rx,
        plugins.clone(),
    );

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
            apply_aesthetic_pack,
            get_not_playing_settings,
            set_not_playing_settings,
            get_update_status,
            get_app_version,
            get_storage_paths,
            open_directory,
            get_obs_status,
            // Stream Goals commands
            get_session_stats,
            reset_stream_goals_session,
            get_stream_goals_config,
            save_goals_config,
            save_custom_vars,
            update_custom_var
        ])
        .setup(move |app| {
            if let Err(e) = crate::logging::init_logging(app.handle()) {
                eprintln!("Failed to initialize logging: {}", e);
            }

            app.manage(cmd_tx_for_app);
            app.manage(auth_tx_for_app);
            app.manage(auth_rx_for_app);
            app.manage(update_status_for_app.clone());
            app.manage(css_version_tx);
            app.manage(plugins);
            app.manage(panoptic_core::ThematicEffects::default());
            // Expose TwitchEventManager for stream-goals Tauri commands
            twitch_manager_for_app.load_session_stats(app.handle());
            app.manage(twitch_manager_for_app);
            // Expose OBS status for get_obs_status command
            app.manage(obs_status_arc);

            let orchestrator_handle = app.handle().clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(orchestrator.run(orchestrator_handle));
            });

            crate::tray::build_tray(app, cmd_tx, update_status_for_app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
