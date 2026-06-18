use crate::engine::settings::AppSettings;
use axum::{routing::get, Router};
use futures_util::StreamExt;
use panoptic_core::{
    AppState, AuthState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

/* ── State Models ─────────────────────────────────────────────── */

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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum TwitchAlert {
    Follow { user_name: String },
    Subscription { user_name: String, tier: String, is_gift: bool, cumulative_months: u32 },
    GiftSubscription { user_name: String, total: u32, tier: String, is_anonymous: bool },
    Raid { from_broadcaster_name: String, viewers: u32 },
    Cheer { user_name: String, bits: u32, message: String },
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatBadge {
    pub set_id: String,
    pub id: String,
    pub info: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessageData {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub message: String,
    pub color: String,
    pub pronouns: Option<String>,
    pub badges: Vec<ChatBadge>,
    pub is_mod: bool,
    pub is_sub: bool,
    pub is_vip: bool,
    pub is_broadcaster: bool,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatState {
    pub messages: Vec<ChatMessageData>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TwitchBroadcasterInfo {
    pub id: String,
    pub login: String,
    pub display_name: String,
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
    pub chat_state: Arc<Mutex<ChatState>>,
    pub broadcaster_info: Arc<Mutex<TwitchBroadcasterInfo>>,
    pub pronoun_map: Arc<Mutex<HashMap<String, String>>>,
    pub user_pronoun_cache: Arc<Mutex<HashMap<String, String>>>,
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
            alert_state: Arc::new(Mutex::new(AlertState { active_alerts: Vec::new() })),
            chat_state: Arc::new(Mutex::new(ChatState { messages: Vec::new() })),
            broadcaster_info: Arc::new(Mutex::new(TwitchBroadcasterInfo::default())),
            pronoun_map: Arc::new(Mutex::new(HashMap::new())),
            user_pronoun_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn init_pronouns(&self) {
        let client = reqwest::Client::new();
        if let Ok(res) = client.get("https://pronouns.alejo.io/api/pronouns").send().await {
            if let Ok(map) = res.json::<HashMap<String, String>>().await {
                let mut p_map = self.pronoun_map.lock().unwrap();
                *p_map = map;
                info!("Twitch Chat: Initialized pronouns map ({} entries)", p_map.len());
            }
        }
    }

    pub async fn get_user_pronouns(&self, login: &str) -> Option<String> {
        {
            let cache = self.user_pronoun_cache.lock().unwrap();
            if let Some(p) = cache.get(login) { return Some(p.clone()); }
        }

        let client = reqwest::Client::new();
        let url = format!("https://pronouns.alejo.io/api/users/{}", login);
        if let Ok(res) = client.get(&url).send().await {
            if let Ok(user_data) = res.json::<serde_json::Value>().await {
                if let Some(p_id) = user_data["pronoun_id"].as_str() {
                    let p_map = self.pronoun_map.lock().unwrap();
                    if let Some(p_str) = p_map.get(p_id) {
                        let mut cache = self.user_pronoun_cache.lock().unwrap();
                        cache.insert(login.to_string(), p_str.clone());
                        return Some(p_str.clone());
                    }
                }
            }
        }
        None
    }
}

impl Default for TwitchEventManager { fn default() -> Self { Self::new() } }

/* ── Plugins ─────────────────────────────────────────────────── */

pub struct TwitchHypeTrainPlugin { manager: Arc<TwitchEventManager> }
pub struct TwitchAlertsPlugin { manager: Arc<TwitchEventManager> }
pub struct TwitchChatPlugin { manager: Arc<TwitchEventManager> }

impl TwitchHypeTrainPlugin { pub fn new(manager: Arc<TwitchEventManager>) -> Self { Self { manager } } }
impl TwitchAlertsPlugin { pub fn new(manager: Arc<TwitchEventManager>) -> Self { Self { manager } } }
impl TwitchChatPlugin { pub fn new(manager: Arc<TwitchEventManager>) -> Self { Self { manager } } }

/* ── Common logic ───────────────────────────────────────────── */

async fn fetch_broadcaster_info(client_id: &str, access_token: &str) -> Result<TwitchBroadcasterInfo, String> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.twitch.tv/helix/users")
        .header("Client-ID", client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .map_err(|e| format!("Helix API request failed: {}", e))?;

    if !res.status().is_success() { return Err(format!("Helix API returned error: {}", res.status())); }

    let data: serde_json::Value = res.json().await.map_err(|e| format!("Failed to parse Helix response: {}", e))?;
    let user = &data["data"][0];
    Ok(TwitchBroadcasterInfo {
        id: user["id"].as_str().unwrap_or_default().to_string(),
        login: user["login"].as_str().unwrap_or_default().to_string(),
        display_name: user["display_name"].as_str().unwrap_or_default().to_string(),
    })
}

async fn subscribe_all_events(client_id: &str, access_token: &str, broadcaster_id: &str, session_id: &str) -> Result<(), String> {
    let client = reqwest::Client::new();
    let subs = vec![
        ("channel.hype_train.begin", "2"), ("channel.hype_train.progress", "2"), ("channel.hype_train.end", "2"),
        ("channel.follow", "2"), ("channel.subscribe", "1"), ("channel.subscription.gift", "1"),
        ("channel.raid", "1"), ("channel.cheer", "1"), ("channel.chat.message", "1"),
    ];

    for (sub_type, version) in subs {
        let mut condition = serde_json::json!({ "broadcaster_user_id": broadcaster_id });
        if sub_type == "channel.follow" { condition = serde_json::json!({ "broadcaster_user_id": broadcaster_id, "moderator_user_id": broadcaster_id }); }
        if sub_type == "channel.chat.message" { condition = serde_json::json!({ "broadcaster_user_id": broadcaster_id, "user_id": broadcaster_id }); }

        let res = client
            .post("https://api.twitch.tv/helix/eventsub/subscriptions")
            .header("Client-ID", client_id)
            .header("Authorization", format!("Bearer {}", access_token))
            .json(&serde_json::json!({
                "type": sub_type, "version": version, "condition": condition,
                "transport": { "method": "websocket", "session_id": session_id }
            }))
            .send().await;

        if let Ok(r) = res {
            if !r.status().is_success() {
                warn!("Twitch EventSub: Failed to subscribe to {} (v{}): {}", sub_type, version, r.text().await.unwrap_or_default());
            } else {
                info!("Twitch EventSub: Successfully subscribed to {}", sub_type);
            }
        }
    }
    Ok(())
}

async fn handle_event(app: &tauri::AppHandle, manager: &TwitchEventManager, sub_type: &str, event: serde_json::Value) {
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
                state.top_contributions = top.iter().map(|c| TwitchContribution {
                    user_id: c["user_id"].as_str().unwrap_or_default().to_string(),
                    user_login: c["user_login"].as_str().unwrap_or_default().to_string(),
                    user_name: c["user_name"].as_str().unwrap_or_default().to_string(),
                    type_field: c["type"].as_str().unwrap_or_default().to_string(),
                    total: c["total"].as_u64().unwrap_or(0) as u32,
                }).collect();
            }
            let _ = app.emit("twitch_hype_train", state.clone());
        }
        "channel.hype_train.end" => {
            let mut state = manager.hype_state.lock().unwrap();
            state.active = false;
            let _ = app.emit("twitch_hype_train", state.clone());
        }
        "channel.follow" => {
            update_alert(app, &manager.alert_state, TwitchAlert::Follow { user_name: event["user_name"].as_str().unwrap_or("Someone").to_string() });
        }
        "channel.subscribe" => {
            update_alert(app, &manager.alert_state, TwitchAlert::Subscription {
                user_name: event["user_name"].as_str().unwrap_or("Someone").to_string(),
                tier: event["tier"].as_str().unwrap_or("1000").to_string(),
                is_gift: event["is_gift"].as_bool().unwrap_or(false),
                cumulative_months: event["cumulative_months"].as_u64().unwrap_or(1) as u32,
            });
        }
        "channel.subscription.gift" => {
            update_alert(app, &manager.alert_state, TwitchAlert::GiftSubscription {
                user_name: event["user_name"].as_str().unwrap_or("Anonymous").to_string(),
                total: event["total"].as_u64().unwrap_or(1) as u32,
                tier: event["tier"].as_str().unwrap_or("1000").to_string(),
                is_anonymous: event["is_anonymous"].as_bool().unwrap_or(false),
            });
        }
        "channel.raid" => {
            update_alert(app, &manager.alert_state, TwitchAlert::Raid {
                from_broadcaster_name: event["from_broadcaster_user_name"].as_str().unwrap_or("Someone").to_string(),
                viewers: event["viewers"].as_u64().unwrap_or(0) as u32,
            });
        }
        "channel.cheer" => {
            update_alert(app, &manager.alert_state, TwitchAlert::Cheer {
                user_name: event["user_name"].as_str().unwrap_or("Anon").to_string(),
                bits: event["bits"].as_u64().unwrap_or(0) as u32,
                message: event["message"].as_str().unwrap_or_default().to_string(),
            });
        }
        "channel.chat.message" => {
            let user_login = event["chatter_user_login"].as_str().unwrap_or_default().to_string();
            let pronouns = manager.get_user_pronouns(&user_login).await;
            let mut badges = Vec::new();
            if let Some(arr) = event["badges"].as_array() {
                for b in arr {
                    badges.push(ChatBadge {
                        set_id: b["set_id"].as_str().unwrap_or_default().to_string(),
                        id: b["id"].as_str().unwrap_or_default().to_string(),
                        info: b["info"].as_str().unwrap_or_default().to_string(),
                    });
                }
            }
            let is_broadcaster = badges.iter().any(|b| b.set_id == "broadcaster");
            let is_mod = badges.iter().any(|b| b.set_id == "moderator");
            let is_vip = badges.iter().any(|b| b.set_id == "vip");
            let is_sub = badges.iter().any(|b| b.set_id == "subscriber");

            let mut state = manager.chat_state.lock().unwrap();
            let msg = ChatMessageData {
                id: event["message_id"].as_str().unwrap_or_default().to_string(),
                user_id: event["chatter_user_id"].as_str().unwrap_or_default().to_string(),
                user_login,
                user_name: event["chatter_user_name"].as_str().unwrap_or_default().to_string(),
                message: event["message"]["text"].as_str().unwrap_or_default().to_string(),
                color: event["color"].as_str().unwrap_or("#ffffff").to_string(),
                pronouns, badges, is_mod, is_sub, is_vip, is_broadcaster,
                timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            };
            state.messages.push(msg.clone());
            if state.messages.len() > 100 { state.messages.remove(0); }
            let _ = app.emit("twitch_chat_message", msg);
        }
        _ => {}
    }
}

fn update_alert(app: &tauri::AppHandle, state_lock: &Arc<Mutex<AlertState>>, alert: TwitchAlert) {
    use tauri::Emitter;
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let id = format!("alert_{}_{}", now, rand::random::<u16>());
    let mut state = state_lock.lock().unwrap();
    state.active_alerts.push(QueuedAlert { id, alert, timestamp: now });
    if state.active_alerts.len() > 10 { state.active_alerts.remove(0); }
    let _ = app.emit("twitch_alert", state.clone());
}

/* ── Plugin Implementations ──────────────────────────────────── */

impl PanopticPlugin for TwitchHypeTrainPlugin {
    fn id(&self) -> &'static str { "twitch_hype_train" }
    fn name(&self) -> &'static str { "Twitch Hype Train" }
    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let auth_rx = app.try_state::<tokio::sync::watch::Receiver<AuthState>>().ok_or("No auth state")?.inner().clone();
        let app_handle = app.clone();
        let manager = self.manager.clone();
        tauri::async_runtime::spawn(async move {
            let mut rx = auth_rx;
            let mut current_task: Option<tokio::task::JoinHandle<()>> = None;
            manager.init_pronouns().await;
            while rx.changed().await.is_ok() {
                let state = rx.borrow().clone();
                if let AuthState::Authenticated { provider, access_token, .. } = state {
                    if provider != "twitch" { continue; }
                    if let Some(t) = current_task.take() { t.abort(); }
                    let app_handle_inner = app_handle.clone();
                    let manager_inner = manager.clone();
                    current_task = Some(tokio::spawn(async move {
                        let settings = AppSettings::load(&app_handle_inner);
                        let client_id = settings.plugins.get("twitch").and_then(|v| v.get("client_id")).and_then(|v| v.as_str()).unwrap_or("").to_string();
                        if client_id.is_empty() { return; }
                        match fetch_broadcaster_info(&client_id, &access_token).await {
                            Ok(info) => {
                                info!("Twitch EventSub: Starting WebSocket loop for broadcaster: {} ({})", info.display_name, info.id);
                                { let mut lock = manager_inner.broadcaster_info.lock().unwrap(); *lock = info.clone(); }
                                loop {
                                    if let Ok((mut ws, _)) = connect_async("wss://eventsub.wss.twitch.tv/ws").await {
                                        info!("Twitch EventSub: WebSocket connected.");
                                        while let Some(msg_result) = ws.next().await {
                                            match msg_result {
                                                Ok(Message::Text(text)) => {
                                                    if let Ok(msg) = serde_json::from_str::<TwitchEventSubMessage>(&text) {
                                                        if let Some(meta) = msg.metadata {
                                                            match meta.message_type.as_str() {
                                                                "session_welcome" => {
                                                                    if let Some(s) = msg.payload.and_then(|p| p.session) {
                                                                        let _ = subscribe_all_events(&client_id, &access_token, &info.id, &s.id).await;
                                                                    }
                                                                }
                                                                "notification" => {
                                                                    if let (Some(payload), Some(sub_type)) = (msg.payload, meta.subscription_type) {
                                                                        if let Some(event) = payload.event {
                                                                            handle_event(&app_handle_inner, &manager_inner, &sub_type, event).await;
                                                                        }
                                                                    }
                                                                }
                                                                "session_keepalive" => {
                                                                    // Quietly ignore keepalives
                                                                }
                                                                _ => {
                                                                    info!("Twitch EventSub: Received message type: {}", meta.message_type);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                Ok(Message::Close(frame)) => {
                                                    warn!("Twitch EventSub: WebSocket closed by server: {:?}", frame);
                                                    break;
                                                }
                                                Err(e) => {
                                                    error!("Twitch EventSub: WebSocket error: {}", e);
                                                    break;
                                                }
                                                _ => {}
                                            }
                                        }
                                        warn!("Twitch EventSub: WebSocket disconnected, retrying in 5s...");
                                    }
                                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                                }
                            },
                            Err(e) => error!("Twitch EventSub: Failed to fetch broadcaster info: {}. WebSocket loop will not start.", e),
                        }
                    }));
                } else if matches!(state, AuthState::Unauthenticated) { if let Some(t) = current_task.take() { t.abort(); } }
            }
        });
        Ok(())
    }
    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let hype_state = self.manager.hype_state.clone();
        router.route("/twitch/hype-train", get(move || { let state = hype_state.lock().unwrap().clone(); async move { axum::Json(state) } }))
              .route("/overlay/twitch/hype-train", get(panoptic_server::handlers::twitch::get_twitch_hype_train_overlay))
    }
    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField { key: "inactive_title".into(), label: "Overlay Title".into(), description: None, field_type: SettingFieldType::Text, default_value: serde_json::json!("Hype Train") },
                SettingField { key: "active_title".into(), label: "Active Title".into(), description: None, field_type: SettingFieldType::Text, default_value: serde_json::json!("Hype Train Active!") },
                SettingField { key: "test_action".into(), label: "Test Overlay".into(), description: None, field_type: SettingFieldType::Action { button_label: "Test Hype Train".into(), action_name: "test_hype_train".into() }, default_value: serde_json::Value::Null },
            ],
        })
    }
    fn handle_action(&self, action: &str, app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
        if action == "test_hype_train" {
            let app_handle = app.clone(); let state_lock = self.manager.hype_state.clone();
            tauri::async_runtime::spawn(async move { simulate_mock_hype_train(&app_handle, &state_lock).await; });
            Ok(serde_json::json!({ "status": "initiated" }))
        } else { Err("Unknown action".to_string()) }
    }
}

impl PanopticPlugin for TwitchAlertsPlugin {
    fn id(&self) -> &'static str { "twitch_alerts" }
    fn name(&self) -> &'static str { "Twitch Alerts" }
    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let alert_state = self.manager.alert_state.clone();
        router.route("/twitch/alerts", get(move || { let state = alert_state.lock().unwrap().clone(); async move { axum::Json(state) } }))
              .route("/overlay/twitch/alerts", get(panoptic_server::handlers::twitch::get_twitch_alerts_overlay))
    }
    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField { key: "follow_text".into(), label: "Follow Text".into(), description: None, field_type: SettingFieldType::Text, default_value: serde_json::json!("{user} just followed!") },
                SettingField { key: "test_alerts".into(), label: "Test Simulation".into(), description: None, field_type: SettingFieldType::Action { button_label: "Simulate All Alerts".into(), action_name: "test_all".into() }, default_value: serde_json::Value::Null },
            ],
        })
    }
    fn handle_action(&self, action: &str, app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
        if action == "test_all" {
            let app_handle = app.clone(); let manager = self.manager.clone();
            tauri::async_runtime::spawn(async move {
                let alerts = vec![ TwitchAlert::Follow { user_name: "Entity_Alpha".into() } ];
                for alert in alerts { update_alert(&app_handle, &manager.alert_state, alert); tokio::time::sleep(std::time::Duration::from_millis(1500)).await; }
            });
            Ok(serde_json::json!({ "status": "initiated" }))
        } else { Err("Unknown action".to_string()) }
    }
}

impl PanopticPlugin for TwitchChatPlugin {
    fn id(&self) -> &'static str { "twitch_chat" }
    fn name(&self) -> &'static str { "Twitch Chat" }
    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let chat_state = self.manager.chat_state.clone();
        router.route("/twitch/chat", get(move || { let state = chat_state.lock().unwrap().clone(); async move { axum::Json(state) } }))
              .route("/overlay/twitch/chat", get(panoptic_server::handlers::twitch::get_twitch_chat_overlay))
    }
    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField { key: "message_template".into(), label: "Message Template".into(), description: Some("Format: {badges} {pronouns} {user}: {message}".into()), field_type: SettingFieldType::Text, default_value: serde_json::json!("{pronouns} {user}: {message}") },
                SettingField { key: "chat_animation".into(), label: "Entrance Animation".into(), description: Some("Choose how messages appear.".into()), field_type: SettingFieldType::Select { options: vec!["Slide".into(), "Fade".into(), "Pop".into(), "Bounce".into()] }, default_value: serde_json::json!("Slide") },
                SettingField { key: "chat_frame_style".into(), label: "Frame Style".into(), description: Some("Add decorative elements to messages.".into()), field_type: SettingFieldType::Select { options: vec!["None".into(), "Glass".into(), "Neon".into(), "Retro".into()] }, default_value: serde_json::json!("None") },
                SettingField { key: "chat_background_blur".into(), label: "Background Blur (px)".into(), description: Some("Glass-morphism effect intensity.".into()), field_type: SettingFieldType::Number, default_value: serde_json::json!(0) },
                SettingField { key: "show_pronouns".into(), label: "Show Pronouns".into(), description: None, field_type: SettingFieldType::Boolean, default_value: serde_json::json!(true) },
                SettingField { key: "show_badges".into(), label: "Show Badges".into(), description: None, field_type: SettingFieldType::Boolean, default_value: serde_json::json!(true) },
                SettingField { key: "max_messages".into(), label: "Max Messages".into(), description: None, field_type: SettingFieldType::Number, default_value: serde_json::json!(50) },
                SettingField { key: "test_chat".into(), label: "Test Chat".into(), description: None, field_type: SettingFieldType::Action { button_label: "Simulate Message".into(), action_name: "test_msg".into() }, default_value: serde_json::Value::Null },
            ],
        })
    }
    fn handle_action(&self, action: &str, app: &tauri::AppHandle) -> Result<serde_json::Value, String> {
        use tauri::Emitter;
        if action == "test_msg" {
            let app_handle = app.clone(); let manager = self.manager.clone();
            tauri::async_runtime::spawn(async move {
                let info = { let lock = manager.broadcaster_info.lock().unwrap(); lock.clone() };
                let msg = ChatMessageData {
                    id: format!("test_{}", rand::random::<u16>()),
                    user_id: if info.id.is_empty() { "123".into() } else { info.id },
                    user_login: if info.login.is_empty() { "streamer".into() } else { info.login },
                    user_name: if info.display_name.is_empty() { "Streamer".into() } else { info.display_name },
                    message: "This is a test message! 🚀".to_string(),
                    color: "#8b5cf6".to_string(),
                    pronouns: Some("they/them".to_string()),
                    badges: vec![ ChatBadge { set_id: "broadcaster".to_string(), id: "1".to_string(), info: "".to_string() } ],
                    is_mod: false, is_sub: false, is_vip: false, is_broadcaster: true,
                    timestamp: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                };
                let _ = app_handle.emit("twitch_chat_message", msg.clone());
                let mut state = manager.chat_state.lock().unwrap();
                state.messages.push(msg);
            });
            Ok(serde_json::json!({ "status": "sent" }))
        } else { Err("Unknown action".to_string()) }
    }
}

async fn simulate_mock_hype_train(app: &tauri::AppHandle, state_lock: &Arc<Mutex<HypeTrainState>>) {
    use tauri::Emitter;
    let mut state = state_lock.lock().unwrap();
    state.active = true; state.level = 1; state.progress = 50; state.goal = 100;
    state.top_contributions = vec![TwitchContribution { user_id: "1".into(), user_login: "alpha".into(), user_name: "Alpha".into(), type_field: "BITS".into(), total: 100 }];
    let _ = app.emit("twitch_hype_train", state.clone());
}
