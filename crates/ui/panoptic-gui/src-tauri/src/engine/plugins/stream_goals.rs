//! StreamGoals plugin - universal progress-bar overlay driven by live Twitch EventSub data.
//!
//! # Data Flow
//! 1. `TwitchEventManager::session_stats` is updated by every incoming EventSub event
//!    (wired in `websocket.rs`).
//! 2. A background task polls the Helix `/streams` endpoint every 30 s to keep
//!    `viewer_count`, `stream_title`, and `category` current.
//! 3. The Axum route `/stream-goals/state` exposes all resolved variables plus the
//!    saved goal configs so the browser-source overlay can render progress bars.
//! 4. The Tauri command `get_session_stats` lets the React preview update in real time
//!    alongside the `session_stats_update` event.

use crate::engine::plugins::twitch_notifications::TwitchEventManager;
use crate::engine::settings::AppSettings;
use axum::{
    extract::{Path, Query, State as AxumState},
    http::header,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use panoptic_core::{
    AppState, AuthState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::Manager;
use tracing::info;

// ─────────────────────────────────────────────────────────────────────────────
// Data types
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for one progress-bar goal, persisted in plugin settings.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GoalConfig {
    /// Unique identifier (client-generated UUID or timestamp string).
    pub id: String,
    /// Display label shown above the bar. Supports `{{variable}}` templates.
    pub label: String,
    /// Variable name to track (e.g. `"followers"`, `"deaths"`).
    pub variable: String,
    /// Target value at which the bar fills to 100 %.
    pub target: f64,
    /// CSS colour string for the fill (e.g. `"#9147ff"`).
    pub color: String,
    /// Whether to display the percentage text.
    #[serde(default = "default_true")]
    pub show_percentage: bool,
    /// Whether to display `current / target` numbers.
    #[serde(default = "default_true")]
    pub show_numbers: bool,
    /// Flash/glow animation when the bar hits 100 %.
    #[serde(default = "default_true")]
    pub milestone_celebration: bool,
    /// Whether to include in the active overlay.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Optional steps/tiers (e.g. [10.0, 25.0, 50.0]). If provided, the goal is treated as a multistep goal.
    #[serde(default)]
    pub steps: Vec<f64>,
}

fn default_true() -> bool {
    true
}

/// A user-defined named counter with a configurable step.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomVar {
    /// Variable name (e.g. `"deaths"`).
    pub name: String,
    /// Current value - persisted in plugin settings.
    pub value: f64,
    /// Amount to add/subtract on each increment/decrement.
    #[serde(default = "default_step")]
    pub step: f64,
}

fn default_step() -> f64 {
    1.0
}

/// Full structure stored in `settings.plugins["stream_goals"]`.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct StreamGoalsSettings {
    #[serde(default)]
    pub goals: Vec<GoalConfig>,
    #[serde(default)]
    pub custom_vars: Vec<CustomVar>,
}

/// Payload served by `/stream-goals/state` and available to the overlay.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamGoalsState {
    /// All numeric variables: session stats + custom var values.
    pub variables: HashMap<String, f64>,
    /// String-typed stream-metadata variables.
    pub string_variables: HashMap<String, String>,
    pub goals: Vec<GoalConfig>,
    pub custom_vars: Vec<CustomVar>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Plugin struct
// ─────────────────────────────────────────────────────────────────────────────

pub struct StreamGoalsPlugin {
    manager: Arc<TwitchEventManager>,
}

impl StreamGoalsPlugin {
    pub fn new(manager: Arc<TwitchEventManager>) -> Self {
        Self { manager }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: build variable map from live session stats + custom vars
// ─────────────────────────────────────────────────────────────────────────────

pub fn build_variables(
    manager: &TwitchEventManager,
    custom_vars: &[CustomVar],
) -> (HashMap<String, f64>, HashMap<String, String>) {
    let stats = manager.session_stats.lock().unwrap();

    let mut numeric: HashMap<String, f64> = HashMap::new();
    numeric.insert("followers".into(), stats.followers as f64);
    numeric.insert("subscribers".into(), stats.subscribers as f64);
    numeric.insert("bits".into(), stats.bits as f64);
    numeric.insert("raids".into(), stats.raids as f64);
    numeric.insert("hosts".into(), stats.hosts as f64);
    numeric.insert("gift_subs".into(), stats.gift_subs as f64);
    numeric.insert("chat_messages".into(), stats.chat_messages as f64);
    numeric.insert("unique_chatters".into(), stats.unique_chatters as f64);
    numeric.insert("new_chatters".into(), stats.new_chatters as f64);
    numeric.insert("hype_train_level".into(), stats.hype_train_level as f64);
    numeric.insert("cheers_count".into(), stats.cheers_count as f64);
    numeric.insert("redemptions".into(), stats.redemptions as f64);
    numeric.insert("viewer_count".into(), stats.viewer_count as f64);

    let mut strings: HashMap<String, String> = HashMap::new();
    strings.insert("stream_title".into(), stats.stream_title.clone());
    strings.insert("category".into(), stats.category.clone());

    // Merge user-defined custom vars (they override nothing built-in since names
    // should differ, but users are responsible for name conflicts).
    for cv in custom_vars {
        numeric.insert(cv.name.clone(), cv.value);
    }

    (numeric, strings)
}

// ─────────────────────────────────────────────────────────────────────────────
// PanopticPlugin impl
// ─────────────────────────────────────────────────────────────────────────────

impl PanopticPlugin for StreamGoalsPlugin {
    fn id(&self) -> &'static str {
        "stream_goals"
    }

    fn name(&self) -> &'static str {
        "Stream Goals"
    }

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        // Wait for Twitch auth, then spin up a periodic poller for stream metadata.
        let auth_rx = app
            .try_state::<tokio::sync::watch::Receiver<AuthState>>()
            .ok_or("No auth state")?
            .inner()
            .clone();
        let app_handle = app.clone();
        let manager = self.manager.clone();

        tauri::async_runtime::spawn(async move {
            let mut rx = auth_rx;
            let mut poll_task: Option<tokio::task::JoinHandle<()>> = None;

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
                    if let Some(t) = poll_task.take() {
                        t.abort();
                    }
                    let app_inner = app_handle.clone();
                    let manager_inner = manager.clone();
                    let token = access_token.clone();
                    poll_task = Some(tokio::spawn(async move {
                        let settings = AppSettings::load(&app_inner);
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
                        // Poll stream metadata every 30 seconds
                        let mut interval =
                            tokio::time::interval(tokio::time::Duration::from_secs(30));
                        loop {
                            interval.tick().await;
                            fetch_stream_metadata(&app_inner, &manager_inner, &client_id, &token)
                                .await;
                        }
                    }));
                } else if matches!(state, AuthState::Unauthenticated) {
                    if let Some(t) = poll_task.take() {
                        t.abort();
                    }
                }
            }
        });

        info!("StreamGoalsPlugin setup complete");
        Ok(())
    }

    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let manager = self.manager.clone();
        router
            .route(
                "/stream-goals/state",
                get(move |AxumState(app_state): AxumState<AppState>| {
                    let manager = manager.clone();
                    let settings_path = app_state.settings_path.clone();
                    async move {
                        let sg_settings = load_stream_goals_settings(settings_path.as_ref());
                        let (variables, string_variables) =
                            build_variables(&manager, &sg_settings.custom_vars);
                        axum::Json(StreamGoalsState {
                            variables,
                            string_variables,
                            goals: sg_settings.goals,
                            custom_vars: sg_settings.custom_vars,
                        })
                    }
                }),
            )
            .route("/overlay/stream-goals", get(get_stream_goals_overlay))
            .route(
                "/stream-goals/variable/:name/increment",
                post(increment_custom_var),
            )
            .route(
                "/stream-goals/variable/:name/decrement",
                post(decrement_custom_var),
            )
            .route("/stream-goals/variable/:name/set", post(set_custom_var))
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![SettingField {
                key: "reset_session".into(),
                label: "Reset Session Stats".into(),
                description: Some(
                    "Reset all session counters (followers, subs, bits, raids…) to zero.".into(),
                ),
                field_type: SettingFieldType::Action {
                    button_label: "Reset Session".into(),
                    action_name: "reset_session".into(),
                },
                default_value: serde_json::Value::Null,
            }],
        })
    }

    fn handle_action(
        &self,
        action: &str,
        app: &tauri::AppHandle,
    ) -> Result<serde_json::Value, String> {
        // Actions have the form: "reset_session", "increment:varname",
        // "decrement:varname", "reset_var:varname"
        if action == "reset_session" {
            use tauri::Emitter;
            let mut stats = self.manager.session_stats.lock().unwrap();
            *stats = Default::default();
            let _ = app.emit("session_stats_update", stats.clone());
            self.manager.save_session_stats(app);
            return Ok(serde_json::json!({ "status": "reset" }));
        }

        let parts: Vec<&str> = action.splitn(2, ':').collect();
        if parts.len() == 2 {
            let op = parts[0];
            let var_name = parts[1];
            match op {
                "increment" | "decrement" | "reset_var" => {
                    return update_custom_var_in_settings(app, var_name, op, None);
                }
                _ => {}
            }
        }

        Err(format!("Unknown stream_goals action: {}", action))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP overlay handler
// ─────────────────────────────────────────────────────────────────────────────

pub async fn get_stream_goals_overlay() -> impl IntoResponse {
    let html =
        include_str!("../../../../../../../crates/services/panoptic-server/src/stream_goals.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}

// ─────────────────────────────────────────────────────────────────────────────
// Settings helpers
// ─────────────────────────────────────────────────────────────────────────────

fn load_stream_goals_settings(settings_path: Option<&std::path::PathBuf>) -> StreamGoalsSettings {
    settings_path
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
        .and_then(|s| s.plugins.get("stream_goals").cloned())
        .and_then(|v| serde_json::from_value::<StreamGoalsSettings>(v).ok())
        .unwrap_or_default()
}

fn update_custom_var_in_settings(
    app: &tauri::AppHandle,
    var_name: &str,
    op: &str,
    val: Option<f64>,
) -> Result<serde_json::Value, String> {
    use tauri::Emitter;
    let mut settings = AppSettings::load(app);
    let raw = settings
        .plugins
        .entry("stream_goals".into())
        .or_insert_with(|| serde_json::json!({}));
    let mut sg: StreamGoalsSettings = serde_json::from_value(raw.clone()).unwrap_or_default();

    if let Some(cv) = sg.custom_vars.iter_mut().find(|v| v.name == var_name) {
        match op {
            "increment" => cv.value += cv.step,
            "decrement" => cv.value -= cv.step,
            "reset_var" => cv.value = 0.0,
            "set" => {
                if let Some(v) = val {
                    cv.value = v;
                }
            }
            _ => {}
        }
        let new_val = cv.value;
        *raw = serde_json::to_value(&sg).unwrap_or(serde_json::json!({}));
        settings.save(app).map_err(|e| e.to_string())?;
        let _ = app.emit("stream_goals_custom_var_update", &sg.custom_vars);
        Ok(serde_json::json!({ "name": var_name, "value": new_val }))
    } else {
        Err(format!("Custom var '{}' not found", var_name))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helix stream metadata polling
// ─────────────────────────────────────────────────────────────────────────────

async fn fetch_stream_metadata(
    app: &tauri::AppHandle,
    manager: &TwitchEventManager,
    client_id: &str,
    access_token: &str,
) {
    use tauri::Emitter;
    let broadcaster_id = {
        let info = manager.broadcaster_info.lock().unwrap();
        info.id.clone()
    };
    if broadcaster_id.is_empty() {
        return;
    }

    let client = reqwest::Client::new();
    let url = format!(
        "https://api.twitch.tv/helix/streams?user_id={}",
        broadcaster_id
    );
    let Ok(res) = client
        .get(&url)
        .header("Client-ID", client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
    else {
        return;
    };
    let Ok(data) = res.json::<serde_json::Value>().await else {
        return;
    };

    if let Some(stream) = data["data"].as_array().and_then(|a| a.first()) {
        let viewer_count = stream["viewer_count"].as_u64().unwrap_or(0);
        let title = stream["title"].as_str().unwrap_or("").to_string();
        let category = stream["game_name"].as_str().unwrap_or("").to_string();
        {
            let mut stats = manager.session_stats.lock().unwrap();
            stats.viewer_count = viewer_count;
            stats.stream_title = title;
            stats.category = category;
        }
        let stats = manager.session_stats.lock().unwrap().clone();
        let _ = app.emit("session_stats_update", stats);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP API Route Handlers
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SetQuery {
    value: Option<f64>,
}

#[derive(Deserialize)]
struct SetBody {
    value: f64,
}

async fn increment_custom_var(
    AxumState(app_state): AxumState<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Some(ref app) = app_state.app_handle {
        match update_custom_var_in_settings(app, &name, "increment", None) {
            Ok(json) => (axum::http::StatusCode::OK, axum::Json(json)).into_response(),
            Err(err) => (axum::http::StatusCode::NOT_FOUND, err).into_response(),
        }
    } else {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Tauri AppHandle not initialized".to_string(),
        )
            .into_response()
    }
}

async fn decrement_custom_var(
    AxumState(app_state): AxumState<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Some(ref app) = app_state.app_handle {
        match update_custom_var_in_settings(app, &name, "decrement", None) {
            Ok(json) => (axum::http::StatusCode::OK, axum::Json(json)).into_response(),
            Err(err) => (axum::http::StatusCode::NOT_FOUND, err).into_response(),
        }
    } else {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Tauri AppHandle not initialized".to_string(),
        )
            .into_response()
    }
}

async fn set_custom_var(
    AxumState(app_state): AxumState<AppState>,
    Path(name): Path<String>,
    Query(query): Query<SetQuery>,
    body: Option<axum::Json<SetBody>>,
) -> impl IntoResponse {
    let value = if let Some(v) = query.value {
        Some(v)
    } else if let Some(axum::Json(b)) = body {
        Some(b.value)
    } else {
        None
    };

    let Some(v) = value else {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "Missing 'value' parameter in query or JSON body".to_string(),
        )
            .into_response();
    };

    if let Some(ref app) = app_state.app_handle {
        match update_custom_var_in_settings(app, &name, "set", Some(v)) {
            Ok(json) => (axum::http::StatusCode::OK, axum::Json(json)).into_response(),
            Err(err) => (axum::http::StatusCode::NOT_FOUND, err).into_response(),
        }
    } else {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Tauri AppHandle not initialized".to_string(),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::{Path, Query, State as AxumState};
    use axum::response::IntoResponse;
    use panoptic_core::{AppState, AuthState, PlaybackState};
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_handlers_without_app_handle() {
        let (auth_tx, _auth_rx) = watch::channel(AuthState::Unauthenticated);
        let (_state_tx, state_rx) = watch::channel(PlaybackState::default());
        let (_, css_version_rx) = watch::channel(1u32);

        let app_state = AppState {
            auth_tx,
            state_rx,
            css_version_rx,
            settings_path: None,
            app_handle: None,
        };

        // Test increment
        let res_inc =
            increment_custom_var(AxumState(app_state.clone()), Path("deaths".to_string()))
                .await
                .into_response();
        assert_eq!(
            res_inc.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );

        // Test decrement
        let res_dec =
            decrement_custom_var(AxumState(app_state.clone()), Path("deaths".to_string()))
                .await
                .into_response();
        assert_eq!(
            res_dec.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );

        // Test set
        let query = SetQuery { value: Some(10.0) };
        let res_set = set_custom_var(
            AxumState(app_state),
            Path("deaths".to_string()),
            Query(query),
            None,
        )
        .await
        .into_response();
        assert_eq!(
            res_set.status(),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}
