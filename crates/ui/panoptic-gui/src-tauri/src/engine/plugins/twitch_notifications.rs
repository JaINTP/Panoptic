use crate::engine::settings::AppSettings;
use axum::{routing::get, Router};
use futures_util::StreamExt;
use panoptic_core::{
    AppState, AuthState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

/* ── Hype Train State ─────────────────────────────────────────── */

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HypeTrainState {
    pub active: bool,
    pub level: u32,
    pub total: u32,
    pub progress: u32,
    pub goal: u32,
    pub top_contributions: Vec<TwitchContribution>,
    pub last_contribution: Option<TwitchContribution>,
    pub started_at: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TwitchContribution {
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub type_field: String,
    pub total: u32,
}

/* ── Alert State ──────────────────────────────────────────────── */

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum TwitchAlert {
    Follow {
        user_name: String,
    },
    Subscription {
        user_name: String,
        tier: String,
        is_gift: bool,
        cumulative_months: u32,
    },
    GiftSubscription {
        user_name: String,
        total: u32,
        tier: String,
        is_anonymous: bool,
    },
    Raid {
        from_broadcaster_name: String,
        viewers: u32,
    },
    Cheer {
        user_name: String,
        bits: u32,
        message: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QueuedAlert {
    pub id: String,
    pub alert: TwitchAlert,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AlertState {
    pub active_alerts: Vec<QueuedAlert>,
}

/* ── WebSocket Models ─────────────────────────────────────────── */

#[derive(Debug, Deserialize)]
struct TwitchEventSubMessage {
    metadata: Option<EventSubMetadata>,
    payload: Option<EventSubPayload>,
}

#[derive(Debug, Deserialize)]
struct EventSubMetadata {
    message_type: String,
    subscription_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EventSubPayload {
    session: Option<EventSubSession>,
    event: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct EventSubSession {
    id: String,
}

/* ── Shared Event Manager ─────────────────────────────────────── */

pub struct TwitchEventManager {
    pub hype_state: Arc<Mutex<HypeTrainState>>,
    pub alert_state: Arc<Mutex<AlertState>>,
}

impl TwitchEventManager {
    pub fn new() -> Self {
        Self {
            hype_state: Arc::new(Mutex::new(HypeTrainState {
                active: false,
                level: 1,
                total: 0,
                progress: 0,
                goal: 100,
                top_contributions: Vec::new(),
                last_contribution: None,
                started_at: "".to_string(),
                expires_at: "".to_string(),
            })),
            alert_state: Arc::new(Mutex::new(AlertState {
                active_alerts: Vec::new(),
            })),
        }
    }
}

impl Default for TwitchEventManager {
    fn default() -> Self {
        Self::new()
    }
}

/* ── Plugins ─────────────────────────────────────────────────── */

pub struct TwitchHypeTrainPlugin {
    manager: Arc<TwitchEventManager>,
}

pub struct TwitchAlertsPlugin {
    manager: Arc<TwitchEventManager>,
}

impl TwitchHypeTrainPlugin {
    pub fn new(manager: Arc<TwitchEventManager>) -> Self {
        Self { manager }
    }
}

impl TwitchAlertsPlugin {
    pub fn new(manager: Arc<TwitchEventManager>) -> Self {
        Self { manager }
    }
}

/* ── Common logic for both ─────────────────────────────────────── */

async fn get_broadcaster_id(client_id: &str, access_token: &str) -> Result<String, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.twitch.tv/helix/users")
        .header("Client-ID", client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Helix API request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("Helix API returned error: {}", res.status()));
    }

    let data: serde_json::Value = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse Helix response: {}", e))?;
    data["data"][0]["id"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "No user data found in Helix response".to_string())
}

async fn subscribe_all_events(
    client_id: &str,
    access_token: &str,
    broadcaster_id: &str,
    session_id: &str,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let subs = vec![
        ("channel.hype_train.begin", "2"),
        ("channel.hype_train.progress", "2"),
        ("channel.hype_train.end", "2"),
        ("channel.follow", "2"),
        ("channel.subscribe", "1"),
        ("channel.subscription.gift", "1"),
        ("channel.raid", "1"),
        ("channel.cheer", "1"),
    ];

    for (sub_type, version) in subs {
        let mut condition = serde_json::json!({ "broadcaster_user_id": broadcaster_id });
        if sub_type == "channel.follow" {
            condition = serde_json::json!({ "broadcaster_user_id": broadcaster_id, "moderator_user_id": broadcaster_id });
        }

        let _ = client
            .post("https://api.twitch.tv/helix/eventsub/subscriptions")
            .header("Client-ID", client_id)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&serde_json::json!({
                "type": sub_type,
                "version": version,
                "condition": condition,
                "transport": { "method": "websocket", "session_id": session_id }
            }))
            .send()
            .await;
    }
    Ok(())
}

fn handle_event(
    app: &tauri::AppHandle,
    manager: &TwitchEventManager,
    sub_type: &str,
    event: serde_json::Value,
) {
    use tauri::Emitter;
    match sub_type {
        "channel.hype_train.begin" | "channel.hype_train.progress" => {
            let mut state = manager.hype_state.lock().unwrap();
            state.active = true;
            state.level = event["level"].as_u64().unwrap_or(1) as u32;
            state.total = event["total"].as_u64().unwrap_or(0) as u32;
            state.progress = event["progress"].as_u64().unwrap_or(0) as u32;
            state.goal = event["goal"].as_u64().unwrap_or(100) as u32;

            if let Some(top) = event["top_contributions"].as_array() {
                state.top_contributions = top
                    .iter()
                    .map(|c| TwitchContribution {
                        user_id: c["user_id"].as_str().unwrap_or_default().to_string(),
                        user_login: c["user_login"].as_str().unwrap_or_default().to_string(),
                        user_name: c["user_name"].as_str().unwrap_or_default().to_string(),
                        type_field: c["type"].as_str().unwrap_or_default().to_string(),
                        total: c["total"].as_u64().unwrap_or(0) as u32,
                    })
                    .collect();
            }
            let _ = app.emit("twitch_hype_train", state.clone());
        }
        "channel.hype_train.end" => {
            let mut state = manager.hype_state.lock().unwrap();
            state.active = false;
            let _ = app.emit("twitch_hype_train", state.clone());
        }
        "channel.follow" => {
            let alert = TwitchAlert::Follow {
                user_name: event["user_name"].as_str().unwrap_or("Someone").to_string(),
            };
            update_alert(app, &manager.alert_state, alert);
        }
        "channel.subscribe" => {
            let alert = TwitchAlert::Subscription {
                user_name: event["user_name"].as_str().unwrap_or("Someone").to_string(),
                tier: event["tier"].as_str().unwrap_or("1000").to_string(),
                is_gift: event["is_gift"].as_bool().unwrap_or(false),
                cumulative_months: event["cumulative_months"].as_u64().unwrap_or(1) as u32,
            };
            update_alert(app, &manager.alert_state, alert);
        }
        "channel.subscription.gift" => {
            let alert = TwitchAlert::GiftSubscription {
                user_name: event["user_name"]
                    .as_str()
                    .unwrap_or("Anonymous")
                    .to_string(),
                total: event["total"].as_u64().unwrap_or(1) as u32,
                tier: event["tier"].as_str().unwrap_or("1000").to_string(),
                is_anonymous: event["is_anonymous"].as_bool().unwrap_or(false),
            };
            update_alert(app, &manager.alert_state, alert);
        }
        "channel.raid" => {
            let alert = TwitchAlert::Raid {
                from_broadcaster_name: event["from_broadcaster_user_name"]
                    .as_str()
                    .unwrap_or("Someone")
                    .to_string(),
                viewers: event["viewers"].as_u64().unwrap_or(0) as u32,
            };
            update_alert(app, &manager.alert_state, alert);
        }
        "channel.cheer" => {
            let alert = TwitchAlert::Cheer {
                user_name: event["user_name"].as_str().unwrap_or("Anon").to_string(),
                bits: event["bits"].as_u64().unwrap_or(0) as u32,
                message: event["message"].as_str().unwrap_or_default().to_string(),
            };
            update_alert(app, &manager.alert_state, alert);
        }
        _ => {}
    }
}

fn update_alert(app: &tauri::AppHandle, state_lock: &Arc<Mutex<AlertState>>, alert: TwitchAlert) {
    use tauri::Emitter;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let id = format!("alert_{}_{}", now, rand::random::<u16>());

    let mut state = state_lock.lock().unwrap();
    state.active_alerts.push(QueuedAlert {
        id: id.clone(),
        alert: alert.clone(),
        timestamp: now,
    });

    // Keep only last 10 alerts in state to prevent bloat
    if state.active_alerts.len() > 10 {
        state.active_alerts.remove(0);
    }

    let _ = app.emit("twitch_alert", state.clone());
}

/* ── Hype Train Plugin Impl ──────────────────────────────────── */

impl PanopticPlugin for TwitchHypeTrainPlugin {
    fn id(&self) -> &'static str {
        "twitch_hype_train"
    }
    fn name(&self) -> &'static str {
        "Twitch Hype Train"
    }

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let auth_rx = app
            .try_state::<tokio::sync::watch::Receiver<AuthState>>()
            .ok_or("No auth state")?
            .inner()
            .clone();
        let app_handle = app.clone();
        let manager = self.manager.clone();

        tauri::async_runtime::spawn(async move {
            let mut rx = auth_rx;
            let mut current_task: Option<tokio::task::JoinHandle<()>> = None;

            while rx.changed().await.is_ok() {
                let state = rx.borrow().clone();
                if let AuthState::Authenticated {
                    provider,
                    access_token,
                    ..
                } = state
                {
                    if provider != "twitch" {
                        continue;
                    }
                    if let Some(t) = current_task.take() {
                        t.abort();
                    }

                    let app_handle_inner = app_handle.clone();
                    let manager_inner = manager.clone();
                    current_task = Some(tokio::spawn(async move {
                        let settings = AppSettings::load(&app_handle_inner);
                        let client_id = settings
                            .plugins
                            .get("twitch")
                            .and_then(|v| v.get("client_id"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        if client_id.is_empty() {
                            return;
                        }

                        if let Ok(broadcaster_id) =
                            get_broadcaster_id(&client_id, &access_token).await
                        {
                            loop {
                                if let Ok((mut ws, _)) =
                                    connect_async("wss://eventsub.wss.twitch.tv/ws").await
                                {
                                    while let Some(Ok(Message::Text(text))) = ws.next().await {
                                        if let Ok(msg) =
                                            serde_json::from_str::<TwitchEventSubMessage>(&text)
                                        {
                                            if let Some(meta) = msg.metadata {
                                                match meta.message_type.as_str() {
                                                    "session_welcome" => {
                                                        if let Some(s) =
                                                            msg.payload.and_then(|p| p.session)
                                                        {
                                                            let _ = subscribe_all_events(
                                                                &client_id,
                                                                &access_token,
                                                                &broadcaster_id,
                                                                &s.id,
                                                            )
                                                            .await;
                                                        }
                                                    }
                                                    "notification" => {
                                                        if let (Some(payload), Some(sub_type)) =
                                                            (msg.payload, meta.subscription_type)
                                                        {
                                                            if let Some(event) = payload.event {
                                                                handle_event(
                                                                    &app_handle_inner,
                                                                    &manager_inner,
                                                                    &sub_type,
                                                                    event,
                                                                );
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }
                                    }
                                }
                                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            }
                        }
                    }));
                } else if matches!(state, AuthState::Unauthenticated) {
                    if let Some(t) = current_task.take() {
                        t.abort();
                    }
                }
            }
        });
        Ok(())
    }

    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let hype_state = self.manager.hype_state.clone();
        router
            .route(
                "/twitch/hype-train",
                get(move || {
                    let state = hype_state.lock().unwrap().clone();
                    async move { axum::Json(state) }
                }),
            )
            .route(
                "/overlay/twitch/hype-train",
                get(panoptic_server::handlers::twitch::get_twitch_hype_train_overlay),
            )
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField {
                    key: "inactive_title".to_string(),
                    label: "Overlay Title".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("Hype Train"),
                },
                SettingField {
                    key: "inactive_subtitle".to_string(),
                    label: "Overlay Subtitle".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("awaiting event…"),
                },
                SettingField {
                    key: "active_title".to_string(),
                    label: "Active Title".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("Hype Train Active!"),
                },
                SettingField {
                    key: "level_prefix".to_string(),
                    label: "Level Prefix".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("Level"),
                },
                SettingField {
                    key: "progress_prefix".to_string(),
                    label: "Progress Prefix".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("Progress to"),
                },
                SettingField {
                    key: "leaderboard_title".to_string(),
                    label: "Leaderboard Title".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("⁕ Top Contributors ⁕"),
                },
                SettingField {
                    key: "empty_leaderboard_text".to_string(),
                    label: "Empty List Text".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("No contributors yet…"),
                },
                SettingField {
                    key: "test_action".to_string(),
                    label: "Test Overlay".to_string(),
                    description: None,
                    field_type: SettingFieldType::Action {
                        button_label: "Test Hype Train".to_string(),
                        action_name: "test_hype_train".to_string(),
                    },
                    default_value: serde_json::Value::Null,
                },
            ],
        })
    }

    fn handle_action(
        &self,
        action: &str,
        app: &tauri::AppHandle,
    ) -> Result<serde_json::Value, String> {
        if action == "test_hype_train" {
            let app_handle = app.clone();
            let state_lock = self.manager.hype_state.clone();
            tauri::async_runtime::spawn(async move {
                simulate_mock_hype_train(&app_handle, &state_lock).await;
            });
            Ok(serde_json::json!({ "status": "initiated" }))
        } else {
            Err("Unknown action".to_string())
        }
    }
}

/* ── Alerts Plugin Impl ───────────────────────────────────────── */

impl PanopticPlugin for TwitchAlertsPlugin {
    fn id(&self) -> &'static str {
        "twitch_alerts"
    }
    fn name(&self) -> &'static str {
        "Twitch Alerts"
    }

    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let alert_state = self.manager.alert_state.clone();
        router
            .route(
                "/twitch/alerts",
                get(move || {
                    let state = alert_state.lock().unwrap().clone();
                    async move { axum::Json(state) }
                }),
            )
            .route(
                "/overlay/twitch/alerts",
                get(panoptic_server::handlers::twitch::get_twitch_alerts_overlay),
            )
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField {
                    key: "follow_text".to_string(),
                    label: "Follow Text".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} just followed!"),
                },
                SettingField {
                    key: "sub_text".to_string(),
                    label: "Subscription Text".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} just subscribed!"),
                },
                SettingField {
                    key: "gift_sub_text".to_string(),
                    label: "Gift Sub Text".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} gifted {total} subs!"),
                },
                SettingField {
                    key: "raid_text".to_string(),
                    label: "Raid Text".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} is raiding with {viewers} viewers!"),
                },
                SettingField {
                    key: "cheer_text".to_string(),
                    label: "Cheer Text".to_string(),
                    description: None,
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("{user} cheered {bits} bits!"),
                },
                SettingField {
                    key: "alert_duration".to_string(),
                    label: "Alert Duration (s)".to_string(),
                    description: None,
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(8),
                },
                SettingField {
                    key: "keep_last_alert".to_string(),
                    label: "Keep Last Alert".to_string(),
                    description: Some("Keep the final alert visible until replaced.".to_string()),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(false),
                },
                SettingField {
                    key: "test_alerts".to_string(),
                    label: "Test Simulation".to_string(),
                    description: None,
                    field_type: SettingFieldType::Action {
                        button_label: "Simulate All Alerts".to_string(),
                        action_name: "test_all".to_string(),
                    },
                    default_value: serde_json::Value::Null,
                },
            ],
        })
    }

    fn handle_action(
        &self,
        action: &str,
        app: &tauri::AppHandle,
    ) -> Result<serde_json::Value, String> {
        match action {
            "test_all" => {
                let app_handle = app.clone();
                let manager = self.manager.clone();
                tauri::async_runtime::spawn(async move {
                    let alerts = vec![
                        TwitchAlert::Follow {
                            user_name: "Entity_Alpha".to_string(),
                        },
                        TwitchAlert::Subscription {
                            user_name: "Entity_Beta".to_string(),
                            tier: "1000".into(),
                            is_gift: false,
                            cumulative_months: 5,
                        },
                        TwitchAlert::GiftSubscription {
                            user_name: "Entity_Gamma".to_string(),
                            total: 5,
                            tier: "1000".into(),
                            is_anonymous: false,
                        },
                        TwitchAlert::Raid {
                            from_broadcaster_name: "MegaStreamer".to_string(),
                            viewers: 1337,
                        },
                        TwitchAlert::Cheer {
                            user_name: "Entity_Delta".to_string(),
                            bits: 500,
                            message: "LUL Love the overlay!".to_string(),
                        },
                    ];

                    for alert in alerts {
                        update_alert(&app_handle, &manager.alert_state, alert);
                        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                    }
                });
                Ok(serde_json::json!({ "status": "initiated" }))
            }
            _ => Err("Unknown action".to_string()),
        }
    }
}

async fn simulate_mock_hype_train(app: &tauri::AppHandle, state_lock: &Arc<Mutex<HypeTrainState>>) {
    use tauri::Emitter;
    let contributors = [
        "Entity_Alpha",
        "Entity_Beta",
        "Entity_Gamma",
        "Entity_Delta",
        "Entity_Epsilon",
    ];
    let mut level = 1;
    let mut progress = 0;
    let goal = 100;

    {
        let mut state = state_lock.lock().unwrap();
        state.active = true;
        state.level = level;
        state.progress = 0;
        state.goal = goal;
        state.top_contributions = Vec::new();
        let _ = app.emit("twitch_hype_train", state.clone());
    }

    for i in 0..10 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        let mut state = state_lock.lock().unwrap();
        progress += 15;
        if progress >= goal {
            level += 1;
            progress %= goal;
        }
        state.level = level;
        state.progress = progress;

        let user = contributors[i as usize % contributors.len()];
        let contrib = TwitchContribution {
            user_id: format!("user_{}", i),
            user_login: user.to_lowercase(),
            user_name: user.to_string(),
            type_field: "BITS".to_string(),
            total: (i as u32 + 1) * 100,
        };
        state.top_contributions.push(contrib);
        state
            .top_contributions
            .sort_by(|a, b| b.total.cmp(&a.total));
        state.top_contributions.truncate(5);

        let _ = app.emit("twitch_hype_train", state.clone());
        if level > 3 {
            break;
        }
    }

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    {
        let mut state = state_lock.lock().unwrap();
        state.active = false;
        let _ = app.emit("twitch_hype_train", state.clone());
    }

    tokio::time::sleep(std::time::Duration::from_secs(12)).await;
    let _ = app.emit("twitch_hype_train_clear", ());
}
