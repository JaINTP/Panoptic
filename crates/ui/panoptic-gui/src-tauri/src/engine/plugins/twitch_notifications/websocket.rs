use super::event_manager::TwitchEventManager;
use super::models::{
    AlertState, ChatBadge, ChatFragment, ChatMessageData, EventSubSession, QueuedAlert,
    TwitchAlert, TwitchBroadcasterInfo, TwitchContribution, TwitchEventSubMessage,
};
use futures_util::StreamExt;
use std::sync::{Arc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

pub async fn fetch_broadcaster_info(
    client_id: &str,
    access_token: &str,
) -> Result<TwitchBroadcasterInfo, String> {
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
    let user = &data["data"][0];
    Ok(TwitchBroadcasterInfo {
        id: user["id"].as_str().unwrap_or_default().to_string(),
        login: user["login"].as_str().unwrap_or_default().to_string(),
        display_name: user["display_name"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
    })
}

pub async fn subscribe_all_events(
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
        ("channel.chat.message", "1"),
        ("channel.channel_points_custom_reward_redemption.add", "1"),
    ];

    for (sub_type, version) in subs {
        let mut condition = serde_json::json!({ "broadcaster_user_id": broadcaster_id });
        if sub_type == "channel.follow" {
            condition = serde_json::json!({
                "broadcaster_user_id": broadcaster_id,
                "moderator_user_id": broadcaster_id
            });
        }
        if sub_type == "channel.chat.message" {
            condition = serde_json::json!({
                "broadcaster_user_id": broadcaster_id,
                "user_id": broadcaster_id
            });
        }
        if sub_type == "channel.raid" {
            condition = serde_json::json!({ "to_broadcaster_user_id": broadcaster_id });
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

pub async fn run_websocket_loop(
    app: tauri::AppHandle,
    manager: Arc<TwitchEventManager>,
    client_id: String,
    access_token: String,
) {
    match fetch_broadcaster_info(&client_id, &access_token).await {
        Ok(info) => {
            info!(
                "Twitch EventSub: Starting WebSocket loop for broadcaster: {} ({})",
                info.display_name, info.id
            );
            {
                let mut lock = manager.broadcaster_info.lock().unwrap();
                *lock = info.clone();
            }
            manager
                .fetch_metadata(&client_id, &access_token, &info.id)
                .await;
            loop {
                if let Ok((mut ws, _)) = connect_async("wss://eventsub.wss.twitch.tv/ws").await {
                    info!("Twitch EventSub: WebSocket connected.");
                    while let Some(msg_result) = ws.next().await {
                        match msg_result {
                            Ok(Message::Text(text)) => {
                                if let Ok(msg) =
                                    serde_json::from_str::<TwitchEventSubMessage>(&text)
                                {
                                    handle_ws_message(
                                        &app,
                                        &manager,
                                        &client_id,
                                        &access_token,
                                        &info.id,
                                        msg,
                                    )
                                    .await;
                                }
                            }
                            Ok(Message::Close(frame)) => {
                                warn!("Twitch EventSub: WebSocket closed: {:?}", frame);
                                break;
                            }
                            Err(e) => {
                                error!("Twitch EventSub: WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
        Err(e) => error!(
            "Twitch EventSub: Failed to fetch broadcaster info: {}. WebSocket loop will not start.",
            e
        ),
    }
}

async fn handle_ws_message(
    app: &tauri::AppHandle,
    manager: &TwitchEventManager,
    client_id: &str,
    access_token: &str,
    broadcaster_id: &str,
    msg: TwitchEventSubMessage,
) {
    let Some(meta) = msg.metadata else { return };
    match meta.message_type.as_str() {
        "session_welcome" => {
            if let Some(EventSubSession { id: session_id }) = msg.payload.and_then(|p| p.session) {
                let _ = subscribe_all_events(client_id, access_token, broadcaster_id, &session_id)
                    .await;
            }
        }
        "notification" => {
            if let (Some(payload), Some(sub_type)) = (msg.payload, meta.subscription_type) {
                if let Some(event) = payload.event {
                    handle_event(app, manager, &sub_type, event).await;
                }
            }
        }
        _ => {}
    }
}

pub async fn handle_event(
    app: &tauri::AppHandle,
    manager: &TwitchEventManager,
    sub_type: &str,
    event: serde_json::Value,
) {
    use tauri::Emitter;
    match sub_type {
        "channel.hype_train.begin" | "channel.hype_train.progress" => {
            let level = event["level"].as_u64().unwrap_or(1) as u32;
            {
                let mut state = manager.hype_state.lock().unwrap();
                state.active = true;
                state.level = level;
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
            // Track hype train level in session stats
            {
                let mut stats = manager.session_stats.lock().unwrap();
                stats.hype_train_level = level as u64;
            }
            emit_session_stats(app, manager);
        }
        "channel.hype_train.end" => {
            let mut state = manager.hype_state.lock().unwrap();
            state.active = false;
            let _ = app.emit("twitch_hype_train", state.clone());
        }
        "channel.follow" => {
            {
                let mut stats = manager.session_stats.lock().unwrap();
                stats.followers += 1;
            }
            emit_session_stats(app, manager);
            update_alert(
                app,
                &manager.alert_state,
                TwitchAlert::Follow {
                    user_name: event["user_name"].as_str().unwrap_or("Someone").to_string(),
                },
            );
        }
        "channel.subscribe" => {
            {
                let mut stats = manager.session_stats.lock().unwrap();
                stats.subscribers += 1;
            }
            emit_session_stats(app, manager);
            update_alert(
                app,
                &manager.alert_state,
                TwitchAlert::Subscription {
                    user_name: event["user_name"].as_str().unwrap_or("Someone").to_string(),
                    tier: event["tier"].as_str().unwrap_or("1000").to_string(),
                    is_gift: event["is_gift"].as_bool().unwrap_or(false),
                    cumulative_months: event["cumulative_months"].as_u64().unwrap_or(1) as u32,
                },
            );
        }
        "channel.subscription.gift" => {
            let gift_count = event["total"].as_u64().unwrap_or(1);
            {
                let mut stats = manager.session_stats.lock().unwrap();
                stats.gift_subs += gift_count;
                // Each gift sub also counts toward subscribers
                stats.subscribers += gift_count;
            }
            emit_session_stats(app, manager);
            update_alert(
                app,
                &manager.alert_state,
                TwitchAlert::GiftSubscription {
                    user_name: event["user_name"]
                        .as_str()
                        .unwrap_or("Anonymous")
                        .to_string(),
                    total: gift_count as u32,
                    tier: event["tier"].as_str().unwrap_or("1000").to_string(),
                    is_anonymous: event["is_anonymous"].as_bool().unwrap_or(false),
                },
            );
        }
        "channel.raid" => {
            {
                let mut stats = manager.session_stats.lock().unwrap();
                stats.raids += 1;
            }
            emit_session_stats(app, manager);
            update_alert(
                app,
                &manager.alert_state,
                TwitchAlert::Raid {
                    from_broadcaster_name: event["from_broadcaster_user_name"]
                        .as_str()
                        .unwrap_or("Someone")
                        .to_string(),
                    viewers: event["viewers"].as_u64().unwrap_or(0) as u32,
                },
            );
        }
        "channel.cheer" => {
            let bits = event["bits"].as_u64().unwrap_or(0);
            {
                let mut stats = manager.session_stats.lock().unwrap();
                stats.bits += bits;
                stats.cheers_count += 1;
            }
            emit_session_stats(app, manager);
            update_alert(
                app,
                &manager.alert_state,
                TwitchAlert::Cheer {
                    user_name: event["user_name"].as_str().unwrap_or("Anon").to_string(),
                    bits: bits as u32,
                    message: event["message"].as_str().unwrap_or_default().to_string(),
                },
            );
        }
        "channel.channel_points_custom_reward_redemption.add" => {
            {
                let mut stats = manager.session_stats.lock().unwrap();
                stats.redemptions += 1;
            }
            emit_session_stats(app, manager);
        }
        "channel.chat.message" => handle_chat_message(app, manager, event).await,
        _ => {}
    }
}

/// Emit a snapshot of session stats so the frontend and stream-goals overlay
/// can react in real time without polling.
fn emit_session_stats(app: &tauri::AppHandle, manager: &TwitchEventManager) {
    use tauri::Emitter;
    let stats = manager.session_stats.lock().unwrap().clone();
    let _ = app.emit("session_stats_update", stats);
    manager.save_session_stats(app);
}

async fn handle_chat_message(
    app: &tauri::AppHandle,
    manager: &TwitchEventManager,
    event: serde_json::Value,
) {
    use tauri::Emitter;
    let user_login = event["chatter_user_login"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let user_id = event["chatter_user_id"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let is_first_msg = event["is_first_msg"].as_bool().unwrap_or(false);

    let pronouns = manager.get_user_pronouns(&user_login).await;

    let badges = resolve_badges(manager, &event);
    let fragments = resolve_fragments(manager, &event);

    let is_broadcaster = badges.iter().any(|b| b.set_id == "broadcaster");
    let is_mod = badges.iter().any(|b| b.set_id == "moderator");
    let is_vip = badges.iter().any(|b| b.set_id == "vip");
    let is_sub = badges.iter().any(|b| b.set_id == "subscriber");

    // Update session chat stats
    {
        let mut stats = manager.session_stats.lock().unwrap();
        stats.chat_messages += 1;
        let is_new_to_session = stats.seen_chatter_ids.insert(user_id.clone());
        if is_new_to_session {
            stats.unique_chatters += 1;
        }
        if is_first_msg {
            stats.new_chatters += 1;
        }
    }
    emit_session_stats(app, manager);

    let mut state = manager.chat_state.lock().unwrap();
    let msg = ChatMessageData {
        id: event["message_id"].as_str().unwrap_or_default().to_string(),
        user_id,
        user_login,
        user_name: event["chatter_user_name"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        message: event["message"]["text"]
            .as_str()
            .unwrap_or_default()
            .to_string(),
        fragments,
        color: event["color"].as_str().unwrap_or("#ffffff").to_string(),
        pronouns,
        badges,
        is_mod,
        is_sub,
        is_vip,
        is_broadcaster,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    state.messages.push(msg.clone());
    if state.messages.len() > 100 {
        state.messages.remove(0);
    }
    let _ = app.emit("twitch_chat_message", msg);
}

fn resolve_badges(manager: &TwitchEventManager, event: &serde_json::Value) -> Vec<ChatBadge> {
    let b_cache = manager.badge_cache.lock().unwrap();
    event["badges"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|b| {
            let set_id = b["set_id"].as_str().unwrap_or_default().to_string();
            let id = b["id"].as_str().unwrap_or_default().to_string();
            let info = b["info"].as_str().unwrap_or_default().to_string();
            let image_url = b_cache.get(&set_id).and_then(|v| v.get(&id)).cloned();
            ChatBadge {
                set_id,
                id,
                info,
                image_url,
            }
        })
        .collect()
}

fn resolve_fragments(manager: &TwitchEventManager, event: &serde_json::Value) -> Vec<ChatFragment> {
    let e_cache = manager.emote_cache.lock().unwrap();
    if let Some(arr) = event["message"]["fragments"].as_array() {
        arr.iter()
            .map(|f| {
                if f["type"].as_str().unwrap_or("text") == "emote" {
                    let id = f["emote"]["id"].as_str().unwrap_or_default().to_string();
                    let text = f["text"].as_str().unwrap_or_default().to_string();
                    let url = e_cache.get(&id).cloned().unwrap_or_else(|| {
                        format!(
                            "https://static-cdn.jtvnw.net/emoticons/v2/{}/default/dark/1.0",
                            id
                        )
                    });
                    ChatFragment::Emote { id, text, url }
                } else {
                    ChatFragment::Text(f["text"].as_str().unwrap_or_default().to_string())
                }
            })
            .collect()
    } else {
        vec![ChatFragment::Text(
            event["message"]["text"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        )]
    }
}

pub fn update_alert(
    app: &tauri::AppHandle,
    state_lock: &Arc<Mutex<AlertState>>,
    alert: TwitchAlert,
) {
    use tauri::Emitter;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let id = format!("alert_{}_{}", now, rand::random::<u16>());
    let mut state = state_lock.lock().unwrap();
    state.active_alerts.push(QueuedAlert {
        id,
        alert,
        timestamp: now,
    });
    if state.active_alerts.len() > 10 {
        state.active_alerts.remove(0);
    }
    let _ = app.emit("twitch_alert", state.clone());
}
