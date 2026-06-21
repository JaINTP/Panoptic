use crate::engine::settings::AppSettings;
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use futures_util::{SinkExt, StreamExt};
use panoptic_core::{
    PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField, SettingFieldType,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

// ── Public status types emitted to frontend ───────────────────────────────────

#[derive(Clone, Serialize, Default)]
pub struct ObsAudioSource {
    pub name: String,
    pub muted: bool,
}

#[derive(Clone, Serialize, Default)]
pub struct ObsSceneItem {
    pub id: i64,
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Default)]
pub struct ObsStatus {
    pub connected: bool,
    pub current_scene: String,
    pub scenes: Vec<String>,
    pub audio_sources: Vec<ObsAudioSource>,
    pub scene_items: Vec<ObsSceneItem>,
    pub error: Option<String>,
}

// ── Internal request type for commanding the WS task ─────────────────────────

pub enum ObsRequest {
    SwitchScene(String),
    ToggleMute(String),
    SetSceneItemEnabled { scene: String, item_id: i64, enabled: bool },
}

// ── Plugin struct ─────────────────────────────────────────────────────────────

pub struct ObsWebsocketPlugin {
    pub status: Arc<std::sync::Mutex<ObsStatus>>,
    pub req_tx: Arc<std::sync::Mutex<Option<mpsc::Sender<ObsRequest>>>>,
    stop_tx: Arc<std::sync::Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl Default for ObsWebsocketPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl ObsWebsocketPlugin {
    pub fn new() -> Self {
        Self {
            status: Arc::new(std::sync::Mutex::new(ObsStatus::default())),
            req_tx: Arc::new(std::sync::Mutex::new(None)),
            stop_tx: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn stop_existing(&self) {
        if let Ok(mut tx) = self.stop_tx.lock() {
            if let Some(s) = tx.take() {
                let _ = s.send(());
            }
        }
        if let Ok(mut tx) = self.req_tx.lock() {
            *tx = None;
        }
    }

    fn start_if_enabled(&self, app: &tauri::AppHandle) {
        let settings = AppSettings::load(app);
        let obs_cfg = settings.plugins.get("obs-websocket");

        let enabled = obs_cfg
            .and_then(|v| v.get("is_enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !enabled {
            return;
        }

        let host = obs_cfg
            .and_then(|v| v.get("host"))
            .and_then(|v| v.as_str())
            .unwrap_or("127.0.0.1")
            .to_string();

        let port = obs_cfg
            .and_then(|v| v.get("port"))
            .and_then(|v| v.as_u64())
            .unwrap_or(4455) as u16;

        let password = obs_cfg
            .and_then(|v| v.get("password"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let (stop_tx, stop_rx) = tokio::sync::oneshot::channel::<()>();
        let (req_tx, req_rx) = mpsc::channel::<ObsRequest>(32);

        if let Ok(mut lock) = self.stop_tx.lock() {
            *lock = Some(stop_tx);
        }
        if let Ok(mut lock) = self.req_tx.lock() {
            *lock = Some(req_tx);
        }

        let status = self.status.clone();
        let app_clone = app.clone();

        tokio::spawn(async move {
            run_connection_loop(host, port, password, status, app_clone, req_rx, stop_rx).await;
        });
    }

    fn send_req(&self, req: ObsRequest) {
        if let Ok(lock) = self.req_tx.lock() {
            if let Some(tx) = lock.as_ref() {
                let _ = tx.try_send(req);
            }
        }
    }
}

impl PanopticPlugin for ObsWebsocketPlugin {
    fn id(&self) -> &'static str {
        "obs-websocket"
    }

    fn name(&self) -> &'static str {
        "OBS WebSocket"
    }

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        self.start_if_enabled(app);
        Ok(())
    }

    fn handle_action(
        &self,
        action: &str,
        app: &tauri::AppHandle,
    ) -> Result<serde_json::Value, String> {
        match action {
            "connect" => {
                self.stop_existing();
                reset_status(&self.status, app, None);
                self.start_if_enabled(app);
                Ok(serde_json::json!({ "status": "connecting" }))
            }
            "disconnect" => {
                self.stop_existing();
                reset_status(&self.status, app, None);
                Ok(serde_json::json!({ "status": "disconnected" }))
            }
            other => {
                if let Some(scene) = other.strip_prefix("switch_scene:") {
                    self.send_req(ObsRequest::SwitchScene(scene.to_string()));
                    return Ok(serde_json::json!({ "status": "ok" }));
                }

                if let Some(name) = other.strip_prefix("toggle_mute:") {
                    self.send_req(ObsRequest::ToggleMute(name.to_string()));
                    return Ok(serde_json::json!({ "status": "ok" }));
                }

                // "toggle_scene_item:{item_id}:{new_enabled}"
                // item_id and new_enabled are at the tail; current scene is read from status.
                if let Some(rest) = other.strip_prefix("toggle_scene_item:") {
                    let mut parts = rest.rsplitn(2, ':');
                    if let (Some(enabled_str), Some(id_str)) = (parts.next(), parts.next()) {
                        if let (Ok(item_id), Ok(enabled)) =
                            (id_str.parse::<i64>(), enabled_str.parse::<bool>())
                        {
                            let scene = self.status.lock().unwrap().current_scene.clone();
                            self.send_req(ObsRequest::SetSceneItemEnabled {
                                scene,
                                item_id,
                                enabled,
                            });
                            return Ok(serde_json::json!({ "status": "ok" }));
                        }
                    }
                }

                Err(format!("Unknown OBS action: {}", other))
            }
        }
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Output,
            fields: vec![
                SettingField {
                    key: "is_enabled".to_string(),
                    label: "Enable OBS WebSocket".to_string(),
                    description: Some(
                        "Connect to OBS WebSocket for scene and source control.".to_string(),
                    ),
                    field_type: SettingFieldType::Boolean,
                    default_value: serde_json::json!(false),
                },
                SettingField {
                    key: "host".to_string(),
                    label: "Host".to_string(),
                    description: Some("OBS WebSocket host address.".to_string()),
                    field_type: SettingFieldType::Text,
                    default_value: serde_json::json!("127.0.0.1"),
                },
                SettingField {
                    key: "port".to_string(),
                    label: "Port".to_string(),
                    description: Some("OBS WebSocket port (default 4455).".to_string()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(4455),
                },
                SettingField {
                    key: "password".to_string(),
                    label: "Password".to_string(),
                    description: Some(
                        "OBS WebSocket server password (leave blank if not set).".to_string(),
                    ),
                    field_type: SettingFieldType::Password,
                    default_value: serde_json::json!(""),
                },
                SettingField {
                    key: "connect_action".to_string(),
                    label: "Connection".to_string(),
                    description: Some("Connect or disconnect from OBS.".to_string()),
                    field_type: SettingFieldType::Action {
                        button_label: "Connect".to_string(),
                        action_name: "connect".to_string(),
                    },
                    default_value: serde_json::Value::Null,
                },
            ],
        })
    }
}

// ── OBS WebSocket v5 connection loop ─────────────────────────────────────────

async fn run_connection_loop(
    host: String,
    port: u16,
    password: String,
    status: Arc<std::sync::Mutex<ObsStatus>>,
    app: tauri::AppHandle,
    mut req_rx: mpsc::Receiver<ObsRequest>,
    mut stop_rx: tokio::sync::oneshot::Receiver<()>,
) {
    info!("OBS WebSocket: connection loop started for {}:{}", host, port);

    loop {
        let url = format!("ws://{}:{}", host, port);

        if stop_rx.try_recv().is_ok() {
            break;
        }

        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            connect_async(&url),
        )
        .await
        {
            Ok(Ok((ws_stream, _))) => {
                info!("OBS WebSocket: connected to {}", url);
                let (mut write, mut read) = ws_stream.split();
                let mut authenticated = false;

                loop {
                    tokio::select! {
                        biased;

                        _ = &mut stop_rx => {
                            let _ = write.send(Message::Close(None)).await;
                            return;
                        }

                        req = req_rx.recv() => {
                            if !authenticated {
                                continue;
                            }
                            match req {
                                Some(ObsRequest::SwitchScene(name)) => {
                                    let msg = build_request(
                                        "SetCurrentProgramScene",
                                        "switch_scene",
                                        serde_json::json!({ "sceneName": name }),
                                    );
                                    let _ = write.send(Message::Text(msg.into())).await;
                                }
                                Some(ObsRequest::ToggleMute(name)) => {
                                    let msg = build_request(
                                        "ToggleInputMute",
                                        "toggle_mute",
                                        serde_json::json!({ "inputName": name }),
                                    );
                                    let _ = write.send(Message::Text(msg.into())).await;
                                }
                                Some(ObsRequest::SetSceneItemEnabled { scene, item_id, enabled }) => {
                                    let msg = build_request(
                                        "SetSceneItemEnabled",
                                        "set_scene_item_enabled",
                                        serde_json::json!({
                                            "sceneName": scene,
                                            "sceneItemId": item_id,
                                            "sceneItemEnabled": enabled,
                                        }),
                                    );
                                    let _ = write.send(Message::Text(msg.into())).await;
                                }
                                None => break,
                            }
                        }

                        incoming = read.next() => {
                            match incoming {
                                Some(Ok(Message::Text(text))) => {
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text.as_str()) {
                                        let op = v["op"].as_u64().unwrap_or(99);
                                        match op {
                                            0 => { // Hello
                                                let identify = build_identify(&v["d"], &password);
                                                let _ = write.send(Message::Text(identify.into())).await;
                                            }
                                            2 => { // Identified
                                                authenticated = true;
                                                update_status(&status, &app, |s| {
                                                    s.connected = true;
                                                    s.error = None;
                                                });
                                                // Kick off initial data fetch
                                                let _ = write.send(Message::Text(build_request(
                                                    "GetSceneList", "get_scenes", serde_json::json!({}),
                                                ).into())).await;
                                                let _ = write.send(Message::Text(build_request(
                                                    "GetInputList", "get_inputs", serde_json::json!({}),
                                                ).into())).await;
                                            }
                                            5 if authenticated => { // Event
                                                let to_send = handle_event(&v["d"], &status, &app);
                                                for msg in to_send {
                                                    let _ = write.send(Message::Text(msg.into())).await;
                                                }
                                            }
                                            7 if authenticated => { // RequestResponse
                                                let to_send = handle_response(&v["d"], &status, &app);
                                                for msg in to_send {
                                                    let _ = write.send(Message::Text(msg.into())).await;
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                Some(Ok(Message::Ping(data))) => {
                                    let _ = write.send(Message::Pong(data)).await;
                                }
                                Some(Ok(Message::Close(frame))) => {
                                    warn!("OBS WebSocket: server closed: {:?}", frame);
                                    break;
                                }
                                None => break,
                                Some(Err(e)) => {
                                    error!("OBS WebSocket: read error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                reset_status(&status, &app, None);
                info!("OBS WebSocket: disconnected, will retry in 5 s");
            }
            Ok(Err(e)) => {
                warn!("OBS WebSocket: connection failed: {}", e);
                reset_status(&status, &app, Some(format!("Connection failed: {}", e)));
            }
            Err(_) => {
                warn!("OBS WebSocket: connection timed out");
                reset_status(&status, &app, Some("Connection timed out".to_string()));
            }
        }

        tokio::select! {
            _ = &mut stop_rx => break,
            _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
        }
    }

    info!("OBS WebSocket: connection loop stopped");
}

// ── Event handlers (return follow-up requests to send) ───────────────────────

fn handle_event(
    d: &serde_json::Value,
    status: &Arc<std::sync::Mutex<ObsStatus>>,
    app: &tauri::AppHandle,
) -> Vec<String> {
    let mut to_send = vec![];
    let event_type = d["eventType"].as_str().unwrap_or("");
    let data = &d["eventData"];

    match event_type {
        "CurrentProgramSceneChanged" => {
            let scene = data["sceneName"].as_str().unwrap_or("").to_string();
            update_status(status, app, |s| {
                s.current_scene = scene.clone();
                s.scene_items = vec![]; // will be repopulated below
            });
            if !scene.is_empty() {
                to_send.push(build_request(
                    "GetSceneItemList",
                    "get_scene_items",
                    serde_json::json!({ "sceneName": scene }),
                ));
            }
        }
        "SceneListChanged" => {
            let scenes = extract_scene_names(data.get("scenes"));
            update_status(status, app, |s| {
                s.scenes = scenes;
            });
        }
        "InputMuteStateChanged" => {
            let name = data["inputName"].as_str().unwrap_or("").to_string();
            let muted = data["inputMuted"].as_bool().unwrap_or(false);
            update_status(status, app, |s| {
                if let Some(src) = s.audio_sources.iter_mut().find(|a| a.name == name) {
                    src.muted = muted;
                }
            });
        }
        "InputCreated" => {
            // New input added to OBS - refresh the full input list
            to_send.push(build_request("GetInputList", "get_inputs", serde_json::json!({})));
        }
        "InputRemoved" => {
            let name = data["inputName"].as_str().unwrap_or("").to_string();
            update_status(status, app, |s| {
                s.audio_sources.retain(|a| a.name != name);
            });
        }
        "SceneItemEnableStateChanged" => {
            let item_id = data["sceneItemId"].as_i64().unwrap_or(-1);
            let enabled = data["sceneItemEnabled"].as_bool().unwrap_or(true);
            update_status(status, app, |s| {
                if let Some(item) = s.scene_items.iter_mut().find(|i| i.id == item_id) {
                    item.enabled = enabled;
                }
            });
        }
        "SceneItemCreated" | "SceneItemRemoved" => {
            // Refresh the scene item list for the current scene
            let scene = status.lock().unwrap().current_scene.clone();
            if !scene.is_empty() {
                to_send.push(build_request(
                    "GetSceneItemList",
                    "get_scene_items",
                    serde_json::json!({ "sceneName": scene }),
                ));
            }
        }
        _ => {}
    }

    to_send
}

// ── Response handlers (return follow-up requests to send) ─────────────────────

fn handle_response(
    d: &serde_json::Value,
    status: &Arc<std::sync::Mutex<ObsStatus>>,
    app: &tauri::AppHandle,
) -> Vec<String> {
    let mut to_send = vec![];
    let req_id = d["requestId"].as_str().unwrap_or("");
    let ok = d["requestStatus"]["result"].as_bool().unwrap_or(false);
    let resp = &d["responseData"];

    if req_id.starts_with("get_mute:") {
        // GetInputMute response — only add to audio_sources if the input actually has mute (ok)
        if ok {
            let name = req_id.strip_prefix("get_mute:").unwrap_or("").to_string();
            let muted = resp["inputMuted"].as_bool().unwrap_or(false);
            update_status(status, app, |s| {
                if !s.audio_sources.iter().any(|a| a.name == name) {
                    s.audio_sources.push(ObsAudioSource { name, muted });
                }
            });
        }
        return to_send;
    }

    match req_id {
        "get_scenes" if ok => {
            let scenes = extract_scene_names(resp.get("scenes"));
            let current = resp["currentProgramSceneName"]
                .as_str()
                .unwrap_or("")
                .to_string();
            update_status(status, app, |s| {
                s.scenes = scenes;
                s.current_scene = current.clone();
            });
            // Fetch scene items for the active scene
            if !current.is_empty() {
                to_send.push(build_request(
                    "GetSceneItemList",
                    "get_scene_items",
                    serde_json::json!({ "sceneName": current }),
                ));
            }
        }
        "get_inputs" if ok => {
            // Probe each input for mute capability; only audio inputs will succeed
            let input_names: Vec<String> = resp["inputs"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|i| i["inputName"].as_str().map(|n| n.to_string()))
                .collect();

            // Reset audio sources; they are rebuilt from successful GetInputMute responses
            update_status(status, app, |s| {
                s.audio_sources = vec![];
            });

            for name in &input_names {
                to_send.push(build_request(
                    "GetInputMute",
                    &format!("get_mute:{}", name),
                    serde_json::json!({ "inputName": name }),
                ));
            }
        }
        "get_scene_items" if ok => {
            let items = extract_scene_items(resp.get("sceneItems"));
            update_status(status, app, |s| {
                s.scene_items = items;
            });
        }
        "toggle_mute" if ok => {
            // ToggleInputMute succeeded - the InputMuteStateChanged event will update state
        }
        id if !ok && matches!(id, "switch_scene" | "toggle_mute" | "set_scene_item_enabled") => {
            let msg = d["requestStatus"]["comment"]
                .as_str()
                .unwrap_or("unknown error");
            error!("OBS WebSocket: request '{}' failed: {}", req_id, msg);
        }
        _ => {}
    }

    to_send
}

// ── Protocol helpers ──────────────────────────────────────────────────────────

/// OBS WebSocket v5 authentication:
/// auth = base64(sha256(base64(sha256(password + salt)) + challenge))
fn compute_auth(password: &str, salt: &str, challenge: &str) -> String {
    let mut h1 = Sha256::new();
    h1.update(password.as_bytes());
    h1.update(salt.as_bytes());
    let secret = B64.encode(h1.finalize());

    let mut h2 = Sha256::new();
    h2.update(secret.as_bytes());
    h2.update(challenge.as_bytes());
    B64.encode(h2.finalize())
}

fn build_identify(hello_data: &serde_json::Value, password: &str) -> String {
    let mut d = serde_json::json!({
        "rpcVersion": 1,
        // Scenes(4) | Inputs(8) | SceneItems(128) = 140
        "eventSubscriptions": 140
    });

    if !password.is_empty() {
        if let (Some(challenge), Some(salt)) = (
            hello_data["authentication"]["challenge"].as_str(),
            hello_data["authentication"]["salt"].as_str(),
        ) {
            d["authentication"] =
                serde_json::Value::String(compute_auth(password, salt, challenge));
        }
    }

    serde_json::json!({ "op": 1, "d": d }).to_string()
}

fn build_request(request_type: &str, request_id: &str, data: serde_json::Value) -> String {
    serde_json::json!({
        "op": 6,
        "d": {
            "requestType": request_type,
            "requestId": request_id,
            "requestData": data
        }
    })
    .to_string()
}

// ── Status helpers ────────────────────────────────────────────────────────────

fn update_status<F>(status: &Arc<std::sync::Mutex<ObsStatus>>, app: &tauri::AppHandle, f: F)
where
    F: FnOnce(&mut ObsStatus),
{
    use tauri::Emitter;
    if let Ok(mut lock) = status.lock() {
        f(&mut *lock);
        let snapshot = lock.clone();
        drop(lock);
        let _ = app.emit("obs_status", snapshot);
    }
}

fn reset_status(
    status: &Arc<std::sync::Mutex<ObsStatus>>,
    app: &tauri::AppHandle,
    error: Option<String>,
) {
    update_status(status, app, |s| {
        s.connected = false;
        s.current_scene = String::new();
        s.scenes = vec![];
        s.audio_sources = vec![];
        s.scene_items = vec![];
        s.error = error;
    });
}

// ── Data extraction helpers ───────────────────────────────────────────────────

fn extract_scene_names(scenes: Option<&serde_json::Value>) -> Vec<String> {
    scenes
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|s| s["sceneName"].as_str().map(|n| n.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn extract_scene_items(items: Option<&serde_json::Value>) -> Vec<ObsSceneItem> {
    items
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|i| {
                    let id = i["sceneItemId"].as_i64()?;
                    let name = i["sourceName"].as_str()?.to_string();
                    let enabled = i["sceneItemEnabled"].as_bool().unwrap_or(true);
                    Some(ObsSceneItem { id, name, enabled })
                })
                .collect()
        })
        .unwrap_or_default()
}
