# Plugin Development Guide

Panoptic is built around a powerful, modular plugin system. This guide explains how to create, register, and manage plugins to extend the toolkit's capabilities.

## Architecture Overview

Plugins in Panoptic are Rust structs that implement the `PanopticPlugin` trait. They can provide media metadata, register custom HTTP routes for overlays, and define their own settings UI.

### The `PanopticPlugin` Trait

The core of the plugin system is defined in `crates/panoptic-core/src/traits/plugin.rs`.

```rust
pub trait PanopticPlugin: Send + Sync {
    /// Unique identifier (e.g., "spotify", "twitch_alerts").
    fn id(&self) -> &'static str;

    /// Human-readable name.
    fn name(&self) -> &'static str;

    /// Lifecycle hook: Called when the Tauri application starts.
    /// Use this to spawn background tasks or initialize state.
    fn setup(&self, app: &tauri::AppHandle) -> Result<(), String>;

    /// Optional: Provide a media provider to feed metadata into the core loop.
    fn media_provider(&self) -> Option<Box<dyn MediaProvider>>;

    /// Optional: Register custom Axum routes to the local HTTP server.
    /// This is where you serve overlay HTML and data endpoints.
    fn register_routes(&self, router: Router<AppState>) -> Router<AppState>;

    /// Define UI fields that should appear in the Settings panel.
    fn settings_definition(&self) -> Option<PluginSettingsDefinition>;

    /// Handle button clicks (Actions) from the Settings UI.
    fn handle_action(&self, action: &str, app: &tauri::AppHandle) -> Result<Value, String>;
}
```

---

## Step-by-Step Development

### 1. Define your Plugin Struct
Create a new file in `crates/ui/panoptic-gui/src-tauri/src/engine/plugins/`.

```rust
pub struct MyNewPlugin;

impl PanopticPlugin for MyNewPlugin {
    fn id(&self) -> &'static str { "my_plugin" }
    fn name(&self) -> &'static str { "My Custom Plugin" }

    fn setup(&self, _app: &tauri::AppHandle) -> Result<(), String> {
        println!("Plugin initialized!");
        Ok(())
    }
}
```

### 2. Adding Settings UI
Panoptic automatically generates UI forms based on your `settings_definition`.

```rust
fn settings_definition(&self) -> Option<PluginSettingsDefinition> {
    Some(PluginSettingsDefinition {
        category: PluginCategory::General,
        fields: vec![
            SettingField {
                key: "username".into(),
                label: "Display Name".into(),
                description: Some("Your public name.".into()),
                field_type: SettingFieldType::Text,
                default_value: serde_json::json!("Streamer"),
            },
            SettingField {
                key: "test_btn".into(),
                label: "Test Action".into(),
                description: None,
                field_type: SettingFieldType::Action {
                    button_label: "Ping".into(),
                    action_name: "ping_action",
                },
                default_value: serde_json::Value::Null,
            },
        ],
    })
}
```

### 3. Handling Actions
When a user clicks a button defined in your `SettingFieldType::Action`, Panoptic calls `handle_action`.

```rust
fn handle_action(&self, action: &str, app: &tauri::AppHandle) -> Result<Value, String> {
    match action {
        "ping_action" => {
            println!("Pong!");
            Ok(serde_json::json!({ "status": "success" }))
        },
        _ => Err("Unknown action".into()),
    }
}
```

### 4. Custom HTTP Endpoints (Overlays)
Plugins can expose data to OBS Browser Sources by registering Axum routes.

```rust
fn register_routes(&self, router: Router<AppState>) -> Router<AppState> {
    router.route("/my-plugin/data", get(|| async {
        axum::Json(serde_json::json!({ "status": "online" }))
    }))
}
```

---

## Media Providers

If your plugin provides music or playback data, implement the `MediaProvider` trait and return it from the `media_provider()` method. The core orchestrator will query all registered providers every second.

```rust
#[async_trait]
pub trait MediaProvider: Send + Sync {
    async fn fetch_now_playing(&self) -> Result<PlaybackState, String>;
}
```

---

## State Management

Plugins often need to communicate with the React frontend. Use **Tauri Events** for real-time updates.

### Emitting Events (Rust)
```rust
use tauri::Emitter;

fn update_frontend(app: &tauri::AppHandle, data: MyData) {
    let _ = app.emit("my_plugin_update", data);
}
```

### Listening (React)
```typescript
import { listen } from '@tauri-apps/api/event';

useEffect(() => {
  const unlisten = listen('my_plugin_update', (event) => {
    console.log('Received data:', event.payload);
  });
  return () => { unlisten.then(f => f()); };
}, []);
```

---

## Registration

To activate your plugin, add it to the `PluginRegistry` in `crates/ui/panoptic-gui/src-tauri/src/lib.rs`.

```rust
let registry = PluginRegistry::new()
    .register(Box::new(SpotifyPlugin::new()))
    .register(Box::new(NativeMediaPlugin::new()))
    .register(Box::new(MyNewPlugin::new())); // Add your plugin here
```

## Best Practices

1.  **Non-Blocking Setup:** Use `tauri::async_runtime::spawn` in `setup()` if you need to perform long-running background tasks (like WebSocket connections).
2.  **Thread Safety:** Use `Arc<Mutex<T>>` for any state that needs to be shared between `setup`, `handle_action`, and `register_routes`.
3.  **Unique IDs:** Ensure your plugin `id()` is unique and uses snake_case, as it determines the storage key in `settings.json`.
4.  **Error Handling:** Always return descriptive `Result` types to help users debug configuration issues in the UI.
