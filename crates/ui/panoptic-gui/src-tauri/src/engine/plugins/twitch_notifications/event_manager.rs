use super::models::{
    AlertState, ChatState, HypeTrainState, PronounEntry, TwitchBroadcasterInfo,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

pub struct TwitchEventManager {
    pub hype_state: Arc<Mutex<HypeTrainState>>,
    pub alert_state: Arc<Mutex<AlertState>>,
    pub chat_state: Arc<Mutex<ChatState>>,
    pub broadcaster_info: Arc<Mutex<TwitchBroadcasterInfo>>,
    pub pronoun_map: Arc<Mutex<HashMap<String, String>>>,
    pub user_pronoun_cache: Arc<Mutex<HashMap<String, String>>>,
    pub badge_cache: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
    pub emote_cache: Arc<Mutex<HashMap<String, String>>>,
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
                started_at: String::new(),
                expires_at: String::new(),
            })),
            alert_state: Arc::new(Mutex::new(AlertState { active_alerts: Vec::new() })),
            chat_state: Arc::new(Mutex::new(ChatState { messages: Vec::new() })),
            broadcaster_info: Arc::new(Mutex::new(TwitchBroadcasterInfo::default())),
            pronoun_map: Arc::new(Mutex::new(HashMap::new())),
            user_pronoun_cache: Arc::new(Mutex::new(HashMap::new())),
            badge_cache: Arc::new(Mutex::new(HashMap::new())),
            emote_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn fetch_metadata(&self, client_id: &str, token: &str, broadcaster_id: &str) {
        info!("Twitch Chat: Refreshing metadata cache (badges & emotes)...");
        self.fetch_all_badges(client_id, token, broadcaster_id).await;
        self.fetch_all_emotes(client_id, token, broadcaster_id).await;
    }

    async fn fetch_all_badges(&self, client_id: &str, token: &str, broadcaster_id: &str) {
        let client = reqwest::Client::new();
        let mut new_cache: HashMap<String, HashMap<String, String>> = HashMap::new();

        if let Ok(res) = client
            .get("https://api.twitch.tv/helix/chat/badges/global")
            .header("Client-ID", client_id)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                self.parse_badge_response(&mut new_cache, data);
            }
        }

        let url = format!(
            "https://api.twitch.tv/helix/chat/badges?broadcaster_id={}",
            broadcaster_id
        );
        if let Ok(res) = client
            .get(url)
            .header("Client-ID", client_id)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                self.parse_badge_response(&mut new_cache, data);
            }
        }

        let mut cache = self.badge_cache.lock().unwrap();
        *cache = new_cache;
        info!("Twitch Chat: Badge cache updated ({} sets)", cache.len());
    }

    fn parse_badge_response(
        &self,
        cache: &mut HashMap<String, HashMap<String, String>>,
        data: serde_json::Value,
    ) {
        if let Some(sets) = data["data"].as_array() {
            for set in sets {
                let set_id = set["set_id"].as_str().unwrap_or_default().to_string();
                let versions = cache.entry(set_id).or_default();
                if let Some(v_arr) = set["versions"].as_array() {
                    for v in v_arr {
                        let id = v["id"].as_str().unwrap_or_default().to_string();
                        let url = v["image_url_1x"].as_str().unwrap_or_default().to_string();
                        versions.insert(id, url);
                    }
                }
            }
        }
    }

    async fn fetch_all_emotes(&self, client_id: &str, token: &str, broadcaster_id: &str) {
        let client = reqwest::Client::new();
        let mut new_cache: HashMap<String, String> = HashMap::new();

        if let Ok(res) = client
            .get("https://api.twitch.tv/helix/chat/emotes/global")
            .header("Client-ID", client_id)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                self.parse_emote_response(&mut new_cache, data);
            }
        }

        let url = format!(
            "https://api.twitch.tv/helix/chat/emotes?broadcaster_id={}",
            broadcaster_id
        );
        if let Ok(res) = client
            .get(url)
            .header("Client-ID", client_id)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
        {
            if let Ok(data) = res.json::<serde_json::Value>().await {
                self.parse_emote_response(&mut new_cache, data);
            }
        }

        let mut cache = self.emote_cache.lock().unwrap();
        *cache = new_cache;
        info!("Twitch Chat: Emote cache updated ({} emotes)", cache.len());
    }

    fn parse_emote_response(&self, cache: &mut HashMap<String, String>, data: serde_json::Value) {
        if let Some(emotes) = data["data"].as_array() {
            for e in emotes {
                let id = e["id"].as_str().unwrap_or_default().to_string();
                let url = e["images"]["url_1x"].as_str().unwrap_or_default().to_string();
                cache.insert(id, url);
            }
        }
    }

    pub async fn init_pronouns(&self) {
        let client = reqwest::Client::new();
        match client
            .get("https://api.pronouns.alejo.io/api/pronouns")
            .send()
            .await
        {
            Ok(res) => {
                if let Ok(entries) = res.json::<Vec<PronounEntry>>().await {
                    let map: HashMap<String, String> =
                        entries.into_iter().map(|e| (e.name, e.display)).collect();
                    let mut p_map = self.pronoun_map.lock().unwrap();
                    *p_map = map;
                    info!("Twitch Chat: Initialized pronouns map ({} entries)", p_map.len());
                }
            }
            Err(e) => error!("Twitch Chat: Failed to fetch pronouns map: {}", e),
        }
    }

    pub async fn get_user_pronouns(&self, login: &str) -> Option<String> {
        {
            let cache = self.user_pronoun_cache.lock().unwrap();
            if let Some(p) = cache.get(login) {
                return Some(p.clone());
            }
        }
        {
            let is_empty = self.pronoun_map.lock().unwrap().is_empty();
            if is_empty {
                self.init_pronouns().await;
            }
        }
        let client = reqwest::Client::new();
        let url = format!("https://api.pronouns.alejo.io/api/users/{}", login);
        match client.get(&url).send().await {
            Ok(res) => {
                if let Ok(user_data) = res.json::<serde_json::Value>().await {
                    if let Some(first) = user_data.as_array().and_then(|a| a.first()) {
                        if let Some(p_id) = first["pronoun_id"].as_str() {
                            let p_map = self.pronoun_map.lock().unwrap();
                            if let Some(p_str) = p_map.get(p_id) {
                                let mut cache = self.user_pronoun_cache.lock().unwrap();
                                cache.insert(login.to_string(), p_str.clone());
                                info!(
                                    "Twitch Chat: Resolved pronouns for {}: {}",
                                    login, p_str
                                );
                                return Some(p_str.clone());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                warn!("Twitch Chat: Failed to fetch user pronouns for {}: {}", login, e)
            }
        }
        None
    }
}

impl Default for TwitchEventManager {
    fn default() -> Self {
        Self::new()
    }
}
