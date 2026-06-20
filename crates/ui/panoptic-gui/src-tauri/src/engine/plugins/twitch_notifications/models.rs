use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatBadge {
    pub set_id: String,
    pub id: String,
    pub info: String,
    pub image_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ChatFragment {
    Text(String),
    Emote {
        id: String,
        text: String,
        url: String,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessageData {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub message: String,
    pub fragments: Vec<ChatFragment>,
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

#[derive(Debug, Deserialize)]
pub(crate) struct TwitchEventSubMessage {
    pub(crate) metadata: Option<EventSubMetadata>,
    pub(crate) payload: Option<EventSubPayload>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EventSubMetadata {
    pub(crate) message_type: String,
    pub(crate) subscription_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EventSubPayload {
    pub(crate) session: Option<EventSubSession>,
    pub(crate) event: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct EventSubSession {
    pub(crate) id: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PronounEntry {
    pub(crate) name: String,
    pub(crate) display: String,
}

/// Session-scoped counters updated by EventSub events.
/// These reset on each stream session (or manually) and are used by the
/// Stream Goals plugin and the variable template engine.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SessionStats {
    pub followers: u64,
    pub subscribers: u64,
    pub bits: u64,
    pub raids: u64,
    /// Hosts are deprecated on Twitch (removed 2022) — always 0.
    pub hosts: u64,
    /// Total gifted subs (sum of gift counts, not event count).
    pub gift_subs: u64,
    pub chat_messages: u64,
    pub unique_chatters: u64,
    /// Chatters whose first-ever message in the channel was this session
    /// (`is_first_msg == true` in the EventSub payload).
    pub new_chatters: u64,
    pub hype_train_level: u64,
    /// Number of distinct Cheer events (not total bits).
    pub cheers_count: u64,
    pub redemptions: u64,
    /// Current viewer count — polled periodically from Helix /streams.
    pub viewer_count: u64,
    pub stream_title: String,
    pub category: String,

    /// Internal: tracks user IDs that have chatted this session (not serialised).
    #[serde(skip)]
    pub seen_chatter_ids: std::collections::HashSet<String>,
}
