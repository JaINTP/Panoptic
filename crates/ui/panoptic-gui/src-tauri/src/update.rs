use tracing::{info, warn};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub html_url: String,
}

#[derive(Clone)]
pub struct UpdateStatus(pub std::sync::Arc<std::sync::Mutex<Option<GitHubRelease>>>);

#[derive(Clone, Debug)]
pub struct RateLimit {
    pub remaining: u32,
    pub reset: u64,
}

pub fn is_newer(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split('.')
            .map(|s| s.parse().unwrap_or(0))
            .collect()
    };
    parse(latest) > parse(current)
}

fn parse_rate_limit(headers: &reqwest::header::HeaderMap) -> RateLimit {
    let remaining = headers
        .get("x-ratelimit-remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(60);

    let reset = headers
        .get("x-ratelimit-reset")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_else(|| {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            now + 3600
        });

    RateLimit { remaining, reset }
}

fn calculate_delay(rate_limit: RateLimit) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let seconds_to_reset = rate_limit.reset.saturating_sub(now);

    // Add a 5-second buffer to the reset time to prevent race conditions at the reset boundary
    let seconds_with_buffer = seconds_to_reset + 5;

    // Use a safety reserve of 1 request so we don't accidentally consume the very last request
    // and trigger a rate limit block before the window resets.
    let safe_remaining = rate_limit.remaining.saturating_sub(1);

    if safe_remaining == 0 {
        return seconds_with_buffer;
    }

    // Distribute remaining requests evenly over the reset window
    let interval = seconds_with_buffer / safe_remaining as u64;
    // Clamp to a minimum of 60 seconds to comply with GitHub's guidelines and avoid abuse
    interval.max(60)
}

pub async fn check_latest_release(
) -> Result<(GitHubRelease, RateLimit), (String, Option<RateLimit>)> {
    let client = reqwest::Client::new();
    let res = client
        .get("https://api.github.com/repos/JaINTP/Panoptic/releases/latest")
        .header("User-Agent", "Panoptic")
        .send()
        .await
        .map_err(|e| (format!("Failed to send update request: {}", e), None))?;

    let rate_limit = parse_rate_limit(res.headers());

    if !res.status().is_success() {
        return Err((
            format!("GitHub API returned error: {}", res.status()),
            Some(rate_limit),
        ));
    }

    let release = res.json::<GitHubRelease>().await.map_err(|e| {
        (
            format!("Failed to parse JSON: {}", e),
            Some(rate_limit.clone()),
        )
    })?;

    Ok((release, rate_limit))
}

pub async fn spawn_update_check(
    app_handle: tauri::AppHandle,
    update_status: UpdateStatus,
    menu: tauri::menu::Menu<tauri::Wry>,
) {
    // Initial delay on startup
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    loop {
        let delay_secs = match check_latest_release().await {
            Ok((release, rate_limit)) => {
                let current_ver = env!("CARGO_PKG_VERSION");
                if is_newer(current_ver, &release.tag_name) {
                    info!("Update available: {} -> {}", current_ver, release.tag_name);

                    let is_new = {
                        if let Ok(mut lock) = update_status.0.lock() {
                            let old = lock.clone();
                            *lock = Some(release.clone());
                            old.map(|r| r.tag_name != release.tag_name).unwrap_or(true)
                        } else {
                            false
                        }
                    };

                    if is_new {
                        if let Ok(update_i) = tauri::menu::MenuItem::with_id(
                            &app_handle,
                            "update",
                            format!("Update Available ({})", release.tag_name),
                            true,
                            None::<&str>,
                        ) {
                            let _ = menu.prepend(&update_i);
                        }
                        use tauri::Emitter;
                        let _ = app_handle.emit("update_available", release);
                    }
                }
                calculate_delay(rate_limit)
            }
            Err((e, opt_rate_limit)) => {
                warn!("Failed to check for updates: {}", e);
                if let Some(rate_limit) = opt_rate_limit {
                    calculate_delay(rate_limit)
                } else {
                    300
                }
            }
        };

        info!("Next update check scheduled in {} seconds.", delay_secs);
        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
    }
}

#[tauri::command]
pub fn get_update_status(
    status: tauri::State<'_, UpdateStatus>,
) -> Result<Option<GitHubRelease>, String> {
    let lock = status.0.lock().map_err(|e| e.to_string())?;
    Ok(lock.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_delay() {
        // Case 1: normal limit reset with plenty of requests remaining (60 requests remaining, 3600 seconds to reset)
        let rate_limit = RateLimit {
            remaining: 60,
            reset: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 3600,
        };
        let delay = calculate_delay(rate_limit);
        assert!(
            (60..=62).contains(&delay),
            "Delay should be around 60-62 seconds, got {}",
            delay
        );

        // Case 2: rate limit exhausted (remaining = 0, 1800 seconds to reset)
        let rate_limit_exhausted = RateLimit {
            remaining: 0,
            reset: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 1800,
        };
        let delay_exhausted = calculate_delay(rate_limit_exhausted);
        assert!(
            (1800..=1810).contains(&delay_exhausted),
            "Delay should be reset time + buffer, got {}",
            delay_exhausted
        );

        // Case 3: only 1 request remaining (which is our safety buffer)
        let rate_limit_one = RateLimit {
            remaining: 1,
            reset: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                + 1800,
        };
        let delay_one = calculate_delay(rate_limit_one);
        assert!(
            (1800..=1810).contains(&delay_one),
            "With 1 request left, delay should wait until reset, got {}",
            delay_one
        );
    }
}
