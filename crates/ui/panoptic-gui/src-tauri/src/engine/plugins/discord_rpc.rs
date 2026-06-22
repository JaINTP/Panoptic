use crate::engine::plugins::twitch_notifications::models::{AlertState, TwitchAlert};
use crate::engine::settings::AppSettings;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use panoptic_core::{
    PanopticPlugin, PlaybackState, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use std::sync::{mpsc, Mutex};
use std::time::{Duration, Instant};
use tauri::Listener;
use tracing::{error, info};

pub enum DiscordRpcCommand {
    UpdatePlayback(PlaybackState),
    UpdateAlert(String),
    ReloadSettings,
}

struct DiscordWorker {
    app_handle: tauri::AppHandle,
    client: Option<DiscordIpcClient>,
    client_id: String,
    enabled: bool,
    show_now_playing: bool,
    show_alerts: bool,
    last_playback: Option<PlaybackState>,
    alert_msg: Option<String>,
    alert_expires_at: Option<Instant>,
    last_connect_attempt: Option<Instant>,
}

impl DiscordWorker {
    fn new(app_handle: tauri::AppHandle, client_id: String) -> Self {
        Self {
            app_handle,
            client: None,
            client_id,
            enabled: true,
            show_now_playing: true,
            show_alerts: true,
            last_playback: None,
            alert_msg: None,
            alert_expires_at: None,
            last_connect_attempt: None,
        }
    }

    fn load_config(&mut self) {
        let settings = AppSettings::load(&self.app_handle);
        if let Some(cfg) = settings.plugins.get("discord_rpc") {
            self.enabled = cfg.get("enable").and_then(|v| v.as_bool()).unwrap_or(true);
            self.show_now_playing = cfg
                .get("show_now_playing")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);
            self.show_alerts = cfg
                .get("show_alerts")
                .and_then(|v| v.as_bool())
                .unwrap_or(true);

            let new_client_id = cfg
                .get("client_id")
                .and_then(|v| v.as_str())
                .unwrap_or("1246193796850454558")
                .to_string();

            if new_client_id != self.client_id {
                self.client_id = new_client_id;
                if let Some(mut c) = self.client.take() {
                    let _ = c.close();
                }
            }
        }
    }

    fn connect_discord(&mut self) -> bool {
        if !self.enabled {
            if let Some(mut c) = self.client.take() {
                let _ = c.close();
            }
            return false;
        }

        if self.client.is_some() {
            return true;
        }

        if let Some(last_attempt) = self.last_connect_attempt {
            if last_attempt.elapsed() < Duration::from_secs(15) {
                return false;
            }
        }
        self.last_connect_attempt = Some(Instant::now());

        info!("Discord RPC: Connecting with client ID: {}", self.client_id);
        match DiscordIpcClient::new(&self.client_id) {
            Ok(mut c) => match c.connect() {
                Ok(_) => {
                    info!("Discord RPC: Connected successfully.");
                    self.client = Some(c);
                    true
                }
                Err(e) => {
                    error!("Discord RPC: Failed to connect IPC: {}", e);
                    false
                }
            },
            Err(e) => {
                error!("Discord RPC: Failed to initialize IPC client: {}", e);
                false
            }
        }
    }

    fn update_activity(&mut self) {
        if !self.connect_discord() {
            return;
        }

        if let Some(expiry) = self.alert_expires_at {
            if Instant::now() >= expiry {
                self.alert_msg = None;
                self.alert_expires_at = None;
            }
        }

        let mut act = activity::Activity::new();

        if let (true, Some(msg)) = (self.show_alerts, &self.alert_msg) {
            act = act.state(msg).details("Twitch Alert 🔔").assets(
                activity::Assets::new()
                    .large_image("twitch_alert")
                    .large_text("Alert!"),
            );

            if let Some(ref mut client) = self.client {
                if let Err(e) = client.set_activity(act) {
                    error!("Discord RPC: Failed to set activity: {}. Disconnecting.", e);
                    let _ = client.close();
                    self.client = None;
                }
            }
        } else if let (true, Some(pb)) = (self.show_now_playing, &self.last_playback) {
            if pb.is_playing {
                let details = format!("🎵 {}", pb.title);
                let state = format!("by {}", pb.artist);

                let mut timestamps = activity::Timestamps::new();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;

                let elapsed = pb.progress_ms / 1000;
                timestamps = timestamps.start(now - elapsed as i64);
                if pb.duration_ms > 0 {
                    let remaining = (pb.duration_ms - pb.progress_ms) / 1000;
                    timestamps = timestamps.end(now + remaining as i64);
                }

                let album_name = if pb.album.is_empty() {
                    "Now Playing"
                } else {
                    &pb.album
                };

                act = act
                    .details(&details)
                    .state(&state)
                    .timestamps(timestamps)
                    .assets(
                        activity::Assets::new()
                            .large_image("now_playing")
                            .large_text(album_name),
                    );

                if let Some(ref mut client) = self.client {
                    if let Err(e) = client.set_activity(act) {
                        error!("Discord RPC: Failed to set activity: {}. Disconnecting.", e);
                        let _ = client.close();
                        self.client = None;
                    }
                }
            } else {
                act = act
                    .details("Listening to Music")
                    .state("Idle (Paused)")
                    .assets(
                        activity::Assets::new()
                            .large_image("now_playing")
                            .large_text("Paused"),
                    );

                if let Some(ref mut client) = self.client {
                    if let Err(e) = client.set_activity(act) {
                        error!("Discord RPC: Failed to set activity: {}. Disconnecting.", e);
                        let _ = client.close();
                        self.client = None;
                    }
                }
            }
        } else {
            act = act
                .details("Panoptic Overlay Controller")
                .state("Idle")
                .assets(
                    activity::Assets::new()
                        .large_image("now_playing")
                        .large_text("Idle"),
                );

            if let Some(ref mut client) = self.client {
                if let Err(e) = client.set_activity(act) {
                    error!("Discord RPC: Failed to set activity: {}. Disconnecting.", e);
                    let _ = client.close();
                    self.client = None;
                }
            }
        }
    }
}

pub fn start_discord_rpc_worker(
    app_handle: tauri::AppHandle,
    rx: mpsc::Receiver<DiscordRpcCommand>,
) {
    std::thread::spawn(move || {
        let mut worker = DiscordWorker::new(app_handle, "1246193796850454558".to_string());
        worker.load_config();

        loop {
            match rx.recv_timeout(Duration::from_millis(1000)) {
                Ok(cmd) => match cmd {
                    DiscordRpcCommand::UpdatePlayback(pb) => {
                        worker.last_playback = Some(pb);
                        worker.update_activity();
                    }
                    DiscordRpcCommand::UpdateAlert(msg) => {
                        worker.alert_msg = Some(msg);
                        worker.alert_expires_at = Some(Instant::now() + Duration::from_secs(5));
                        worker.update_activity();
                    }
                    DiscordRpcCommand::ReloadSettings => {
                        worker.load_config();
                        worker.update_activity();
                    }
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if (worker.client.is_none() && worker.enabled)
                        || worker.alert_expires_at.is_some()
                    {
                        worker.update_activity();
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break;
                }
            }
        }
    });
}

pub struct DiscordRpcPlugin {
    tx: Mutex<Option<mpsc::Sender<DiscordRpcCommand>>>,
}

impl DiscordRpcPlugin {
    pub fn new() -> Self {
        Self {
            tx: Mutex::new(None),
        }
    }
}

impl Default for DiscordRpcPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PanopticPlugin for DiscordRpcPlugin {
    fn id(&self) -> &'static str {
        "discord_rpc"
    }

    fn name(&self) -> &'static str {
        "Discord Rich Presence"
    }

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let (tx, rx) = mpsc::channel();

        start_discord_rpc_worker(app.clone(), rx);

        let tx_pb = tx.clone();
        app.listen("playback_update", move |event| {
            if let Ok(state) = serde_json::from_str::<PlaybackState>(event.payload()) {
                let _ = tx_pb.send(DiscordRpcCommand::UpdatePlayback(state));
            }
        });

        let tx_alert = tx.clone();
        app.listen("twitch_alert", move |event| {
            if let Ok(state) = serde_json::from_str::<AlertState>(event.payload()) {
                if let Some(queued) = state.active_alerts.last() {
                    let alert_msg = match &queued.alert {
                        TwitchAlert::Follow { user_name } => {
                            format!("{} just followed!", user_name)
                        }
                        TwitchAlert::Subscription { user_name, .. } => {
                            format!("{} subscribed!", user_name)
                        }
                        TwitchAlert::GiftSubscription {
                            user_name, total, ..
                        } => format!("{} gifted {} subs!", user_name, total),
                        TwitchAlert::Raid {
                            from_broadcaster_name,
                            viewers,
                        } => format!("Raid from {} ({} viewers)!", from_broadcaster_name, viewers),
                        TwitchAlert::Cheer {
                            user_name, bits, ..
                        } => format!("{} cheered {} bits!", user_name, bits),
                    };
                    let _ = tx_alert.send(DiscordRpcCommand::UpdateAlert(alert_msg));
                }
            }
        });

        let tx_settings = tx.clone();
        app.listen("plugin_settings_updated", move |event| {
            if let Ok(plugin_id) = serde_json::from_str::<String>(event.payload()) {
                if plugin_id == "discord_rpc" {
                    let _ = tx_settings.send(DiscordRpcCommand::ReloadSettings);
                }
            }
        });

        *self.tx.lock().unwrap() = Some(tx);
        Ok(())
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::General,
            fields: vec![
                SettingField {
                    key: "enable".into(),
                    label: "Enable Discord Rich Presence".into(),
                    description: Some("Show your track info and alerts on Discord".into()),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(true),
                },
                SettingField {
                    key: "client_id".into(),
                    label: "Discord Application Client ID".into(),
                    description: Some(
                        "Customize to use your own Discord application assets".into(),
                    ),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("1246193796850454558"),
                },
                SettingField {
                    key: "show_now_playing".into(),
                    label: "Show Now Playing Track".into(),
                    description: Some("Display active track details on Discord".into()),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(true),
                },
                SettingField {
                    key: "show_alerts".into(),
                    label: "Show Stream Alerts".into(),
                    description: Some(
                        "Briefly show alerts (follows, cheers, subs) on Discord".into(),
                    ),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(true),
                },
            ],
        })
    }
}
