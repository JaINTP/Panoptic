use crate::engine::settings::AppSettings;
use panoptic_core::{AuthState, PlaybackState};
use std::time::Duration;
use tokio::sync::{mpsc, watch};
use tracing::error;

pub enum AppCommand {
    OpenSettings,
    InitiateAuth,
    Quit,
}

pub struct EngineOrchestrator {
    cmd_rx: mpsc::Receiver<AppCommand>,
    state_tx: watch::Sender<PlaybackState>,
    auth_tx: watch::Sender<AuthState>,
    state_rx: watch::Receiver<PlaybackState>,
    css_version_rx: watch::Receiver<u32>,
    plugins: std::sync::Arc<Vec<Box<dyn panoptic_core::PanopticPlugin>>>,
}

impl EngineOrchestrator {
    pub fn new(
        cmd_rx: mpsc::Receiver<AppCommand>,
        state_tx: watch::Sender<PlaybackState>,
        state_rx: watch::Receiver<PlaybackState>,
        auth_tx: watch::Sender<AuthState>,
        css_version_rx: watch::Receiver<u32>,
        plugins: std::sync::Arc<Vec<Box<dyn panoptic_core::PanopticPlugin>>>,
    ) -> Self {
        Self {
            cmd_rx,
            state_tx,
            auth_tx,
            state_rx,
            css_version_rx,
            plugins,
        }
    }

    pub async fn run(mut self, app_handle: tauri::AppHandle) {
        // Initialize all plugins
        for plugin in self.plugins.iter() {
            if let Err(e) = plugin.setup(&app_handle) {
                error!("Failed to setup plugin '{}': {}", plugin.id(), e);
            }
        }

        let auth_tx = self.auth_tx.clone();
        let settings_path = AppSettings::get_path(&app_handle);
        let plugins_server = self.plugins.clone();
        tokio::spawn(panoptic_server::start_server(
            self.state_rx.clone(),
            auth_tx,
            self.css_version_rx.clone(),
            settings_path,
            plugins_server,
        ));

        let app_handle_cmd = app_handle.clone();
        let plugins_cmd = self.plugins.clone();
        tokio::spawn(async move {
            while let Some(cmd) = self.cmd_rx.recv().await {
                match cmd {
                    AppCommand::OpenSettings => {
                        if let Some(window) =
                            tauri::Manager::get_webview_window(&app_handle_cmd, "main")
                        {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    AppCommand::InitiateAuth => {
                        // Forward to spotify plugin for backward compatibility if it exists
                        for plugin in plugins_cmd.iter() {
                            if plugin.id() == "spotify" {
                                let _ = plugin.handle_action("link", &app_handle_cmd);
                                break;
                            }
                        }
                    }
                    AppCommand::Quit => {
                        std::process::exit(0);
                    }
                }
            }
        });

        loop {
            let mut state = PlaybackState::default();

            // Query plugins for media providers
            for plugin in self.plugins.iter() {
                if let Some(provider) = plugin.media_provider() {
                    match provider.fetch_now_playing().await {
                        Ok(s) => {
                            state = s;
                            break;
                        }
                        Err(_) => continue,
                    }
                }
            }

            let template = AppSettings::load(&app_handle)
                .template
                .filter(|t| !t.is_empty())
                .unwrap_or_else(|| "Now Playing: {title} by {artist}".to_string());
            let formatted_string = state.format(&template);

            state.formatted_output = formatted_string.clone();

            // Write formatted track info to ~/.config/panoptic/current_track.txt
            if let Ok(home) = std::env::var("HOME") {
                let config_path = std::path::PathBuf::from(home).join(".config/panoptic");
                if !config_path.exists() {
                    let _ = std::fs::create_dir_all(&config_path);
                }
                let file_path = config_path.join("current_track.txt");
                let _ = std::fs::write(file_path, &formatted_string);
            }

            let _ = self.state_tx.send(state.clone());
            use tauri::Emitter;
            let _ = app_handle.emit("playback_update", state);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}
