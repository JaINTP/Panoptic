use crate::engine::settings::AppSettings;
use axum::{http::header, response::IntoResponse, routing::get, Router};
use panoptic_core::{
    AppState, PanopticPlugin, PluginCategory, PluginSettingsDefinition, SettingField,
    SettingFieldType,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::Emitter;
use tracing::info;

// ---------------------------------------------------------------------------
// State types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PomodoroPhase {
    Work,
    ShortBreak,
    LongBreak,
}

impl PomodoroPhase {
    pub fn label(&self) -> &'static str {
        match self {
            PomodoroPhase::Work => "Work",
            PomodoroPhase::ShortBreak => "Short Break",
            PomodoroPhase::LongBreak => "Long Break",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PomodoroState {
    pub phase: PomodoroPhase,
    pub remaining_secs: u64,
    pub total_secs: u64,
    pub is_running: bool,
    pub completed_sessions: u32,
    pub sessions_before_long_break: u32,
    pub work_duration_mins: u64,
    pub short_break_mins: u64,
    pub long_break_mins: u64,
}

impl Default for PomodoroState {
    fn default() -> Self {
        Self {
            phase: PomodoroPhase::Work,
            remaining_secs: 25 * 60,
            total_secs: 25 * 60,
            is_running: false,
            completed_sessions: 0,
            sessions_before_long_break: 4,
            work_duration_mins: 25,
            short_break_mins: 5,
            long_break_mins: 15,
        }
    }
}

impl PomodoroState {
    fn duration_for_phase(&self, phase: &PomodoroPhase) -> u64 {
        match phase {
            PomodoroPhase::Work => self.work_duration_mins * 60,
            PomodoroPhase::ShortBreak => self.short_break_mins * 60,
            PomodoroPhase::LongBreak => self.long_break_mins * 60,
        }
    }

    fn next_phase(&self) -> PomodoroPhase {
        match self.phase {
            PomodoroPhase::Work => {
                // Count this session's completion
                let next_completed = self.completed_sessions + 1;
                if next_completed % self.sessions_before_long_break == 0 {
                    PomodoroPhase::LongBreak
                } else {
                    PomodoroPhase::ShortBreak
                }
            }
            PomodoroPhase::ShortBreak | PomodoroPhase::LongBreak => PomodoroPhase::Work,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin struct
// ---------------------------------------------------------------------------

pub struct PomodoroPlugin {
    state: Arc<Mutex<PomodoroState>>,
}

impl Default for PomodoroPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl PomodoroPlugin {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(PomodoroState::default())),
        }
    }

    fn apply_saved_settings(state: &mut PomodoroState, plugin_cfg: &serde_json::Value) {
        if let Some(v) = plugin_cfg.get("work_duration_mins").and_then(|v| v.as_u64()) {
            state.work_duration_mins = v.max(1);
        }
        if let Some(v) = plugin_cfg.get("short_break_mins").and_then(|v| v.as_u64()) {
            state.short_break_mins = v.max(1);
        }
        if let Some(v) = plugin_cfg.get("long_break_mins").and_then(|v| v.as_u64()) {
            state.long_break_mins = v.max(1);
        }
        if let Some(v) = plugin_cfg
            .get("sessions_before_long_break")
            .and_then(|v| v.as_u64())
        {
            state.sessions_before_long_break = (v.max(1)) as u32;
        }
    }
}

// ---------------------------------------------------------------------------
// PanopticPlugin impl
// ---------------------------------------------------------------------------

impl PanopticPlugin for PomodoroPlugin {
    fn id(&self) -> &'static str {
        "pomodoro"
    }

    fn name(&self) -> &'static str {
        "Pomodoro Timer"
    }

    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String> {
        // Load persisted settings and initialise state
        let settings = AppSettings::load(app);
        {
            let mut s = self.state.lock().unwrap();
            if let Some(cfg) = settings.plugins.get("pomodoro") {
                Self::apply_saved_settings(&mut s, cfg);
            }
            let dur = s.duration_for_phase(&PomodoroPhase::Work);
            s.remaining_secs = dur;
            s.total_secs = dur;
        }

        let shared = self.state.clone();
        let app_handle = app.clone();

        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(1));
            loop {
                interval.tick().await;

                let snapshot = {
                    let mut s = shared.lock().unwrap();
                    if !s.is_running {
                        continue;
                    }

                    if s.remaining_secs > 0 {
                        s.remaining_secs -= 1;
                        s.clone()
                    } else {
                        // Phase expired - advance to next phase, pause for manual resume
                        let next = s.next_phase();
                        if s.phase == PomodoroPhase::Work {
                            s.completed_sessions += 1;
                        }
                        let dur = s.duration_for_phase(&next);
                        s.phase = next;
                        s.remaining_secs = dur;
                        s.total_secs = dur;
                        s.is_running = false;
                        let snap = s.clone();
                        // Notify frontend that a phase completed (automation hook)
                        let _ = app_handle.emit("pomodoro_phase_complete", &snap);
                        snap
                    }
                };

                let _ = app_handle.emit("pomodoro_tick", &snapshot);
            }
        });

        info!("PomodoroPlugin setup complete");
        Ok(())
    }

    fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
        let state_for_get = self.state.clone();

        router
            .route(
                "/pomodoro/state",
                get(move || {
                    let snap = state_for_get.lock().unwrap().clone();
                    async move { axum::Json(snap) }
                }),
            )
            .route("/overlay/pomodoro", get(get_pomodoro_overlay))
    }

    fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
        Some(PluginSettingsDefinition {
            category: PluginCategory::Overlay,
            fields: vec![
                SettingField {
                    key: "work_duration_mins".into(),
                    label: "Work Duration (minutes)".into(),
                    description: Some("Length of each work session.".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(25),
                },
                SettingField {
                    key: "short_break_mins".into(),
                    label: "Short Break (minutes)".into(),
                    description: Some("Length of short breaks between sessions.".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(5),
                },
                SettingField {
                    key: "long_break_mins".into(),
                    label: "Long Break (minutes)".into(),
                    description: Some("Length of long breaks after N sessions.".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(15),
                },
                SettingField {
                    key: "sessions_before_long_break".into(),
                    label: "Sessions Before Long Break".into(),
                    description: Some("How many work sessions before a long break triggers.".into()),
                    field_type: SettingFieldType::Number,
                    default_value: serde_json::json!(4),
                },
                SettingField {
                    key: "action_toggle".into(),
                    label: "Start / Pause Timer".into(),
                    description: None,
                    field_type: SettingFieldType::Action {
                        button_label: "Start / Pause".into(),
                        action_name: "toggle".into(),
                    },
                    default_value: serde_json::json!(null),
                },
                SettingField {
                    key: "action_skip".into(),
                    label: "Skip Phase".into(),
                    description: Some("Advance to the next phase immediately.".into()),
                    field_type: SettingFieldType::Action {
                        button_label: "Skip".into(),
                        action_name: "skip".into(),
                    },
                    default_value: serde_json::json!(null),
                },
                SettingField {
                    key: "action_reset".into(),
                    label: "Reset Timer".into(),
                    description: Some("Stop and reset to the start of a work session.".into()),
                    field_type: SettingFieldType::Action {
                        button_label: "Reset".into(),
                        action_name: "reset".into(),
                    },
                    default_value: serde_json::json!(null),
                },
            ],
        })
    }

    fn handle_action(
        &self,
        action: &str,
        app: &tauri::AppHandle,
    ) -> Result<serde_json::Value, String> {
        let snapshot = match action {
            "toggle" => {
                let settings = AppSettings::load(app);
                let mut s = self.state.lock().unwrap();

                // Re-apply saved settings each time the user starts the timer so
                // number field changes take effect without needing a separate save step.
                if !s.is_running {
                    if let Some(cfg) = settings.plugins.get("pomodoro") {
                        Self::apply_saved_settings(&mut s, cfg);
                        // If paused at 0, jump to next phase so the user gets a fresh interval
                        if s.remaining_secs == 0 {
                            let next = s.next_phase();
                            let dur = s.duration_for_phase(&next);
                            s.phase = next;
                            s.remaining_secs = dur;
                            s.total_secs = dur;
                        }
                    }
                }

                s.is_running = !s.is_running;
                s.clone()
            }

            "skip" => {
                let mut s = self.state.lock().unwrap();
                if s.phase == PomodoroPhase::Work {
                    s.completed_sessions += 1;
                }
                let next = s.next_phase();
                let dur = s.duration_for_phase(&next);
                s.phase = next;
                s.remaining_secs = dur;
                s.total_secs = dur;
                s.is_running = false;
                s.clone()
            }

            "reset" => {
                let settings = AppSettings::load(app);
                let mut s = self.state.lock().unwrap();
                if let Some(cfg) = settings.plugins.get("pomodoro") {
                    Self::apply_saved_settings(&mut s, cfg);
                }
                let dur = s.duration_for_phase(&PomodoroPhase::Work);
                s.phase = PomodoroPhase::Work;
                s.remaining_secs = dur;
                s.total_secs = dur;
                s.is_running = false;
                s.completed_sessions = 0;
                s.clone()
            }

            other => return Err(format!("Unknown pomodoro action: {}", other)),
        };

        // Push the updated state to the frontend immediately so the preview
        // reflects the change without waiting for the next tick.
        let _ = app.emit("pomodoro_tick", &snapshot);

        Ok(serde_json::to_value(&snapshot).unwrap_or(serde_json::json!({})))
    }
}

// ---------------------------------------------------------------------------
// HTTP handler - serves the OBS overlay HTML
// ---------------------------------------------------------------------------

pub async fn get_pomodoro_overlay() -> impl IntoResponse {
    let html = include_str!("../../../../../../../crates/services/panoptic-server/src/pomodoro.html");
    ([(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
}
