use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const STORE_MIN_INTERVAL_SECS: u64 = 2;
const STORE_MAX_PER_MINUTE: u32 = 6;
const STORE_COOLDOWN_SECS: u64 = 60 * 60;
const WEB_API_MIN_INTERVAL_SECS: u64 = 1;
const WEB_API_MAX_PER_MINUTE: u32 = 20;

#[derive(Debug, Clone, Copy)]
pub enum SteamEndpoint {
    Store,
    WebApi,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct PersistedCooldowns {
    store_blocked_until: u64,
    web_api_blocked_until: u64,
}

#[derive(Debug)]
struct EndpointState {
    last_request: Option<Instant>,
    minute_started: Instant,
    requests_this_minute: u32,
}

impl EndpointState {
    fn new() -> Self {
        Self {
            last_request: None,
            minute_started: Instant::now(),
            requests_this_minute: 0,
        }
    }
}

struct RateLimiter {
    store: EndpointState,
    web_api: EndpointState,
    persisted: PersistedCooldowns,
}

lazy_static::lazy_static! {
    static ref LIMITER: Mutex<RateLimiter> = Mutex::new(RateLimiter {
        store: EndpointState::new(),
        web_api: EndpointState::new(),
        persisted: load_persisted(),
    });
}

fn state_path() -> PathBuf {
    crate::modules::settings::data_dir().join("steam_rate_limit.json")
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn load_persisted() -> PersistedCooldowns {
    let path = state_path();
    if !path.exists() {
        return PersistedCooldowns::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_persisted(cooldowns: &PersistedCooldowns) {
    let path = state_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cooldowns) {
        let _ = std::fs::write(path, json);
    }
}

fn blocked_until(endpoint: SteamEndpoint, persisted: &PersistedCooldowns) -> u64 {
    match endpoint {
        SteamEndpoint::Store => persisted.store_blocked_until,
        SteamEndpoint::WebApi => persisted.web_api_blocked_until,
    }
}

fn set_blocked_until(endpoint: SteamEndpoint, persisted: &mut PersistedCooldowns, until: u64) {
    match endpoint {
        SteamEndpoint::Store => persisted.store_blocked_until = until,
        SteamEndpoint::WebApi => persisted.web_api_blocked_until = until,
    }
}

fn min_interval(endpoint: SteamEndpoint) -> Duration {
    match endpoint {
        SteamEndpoint::Store => Duration::from_secs(STORE_MIN_INTERVAL_SECS),
        SteamEndpoint::WebApi => Duration::from_secs(WEB_API_MIN_INTERVAL_SECS),
    }
}

fn max_per_minute(endpoint: SteamEndpoint) -> u32 {
    match endpoint {
        SteamEndpoint::Store => STORE_MAX_PER_MINUTE,
        SteamEndpoint::WebApi => WEB_API_MAX_PER_MINUTE,
    }
}

fn cooldown_secs(endpoint: SteamEndpoint) -> u64 {
    match endpoint {
        SteamEndpoint::Store => STORE_COOLDOWN_SECS,
        SteamEndpoint::WebApi => STORE_COOLDOWN_SECS / 2,
    }
}

fn endpoint_label(endpoint: SteamEndpoint) -> &'static str {
    match endpoint {
        SteamEndpoint::Store => "Steam Store",
        SteamEndpoint::WebApi => "Steam Web API",
    }
}

pub fn cooldown_message(endpoint: SteamEndpoint) -> Option<String> {
    let limiter = LIMITER.lock().ok()?;
    let now = now_unix();
    let until = blocked_until(endpoint, &limiter.persisted);
    if until > now {
        let mins = (until - now).div_ceil(60);
        Some(format!(
            "{} is temporarily paused to protect your IP ({} min remaining). Card detection uses cached data only.",
            endpoint_label(endpoint),
            mins
        ))
    } else {
        None
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SteamRateLimitStatus {
    pub store_paused: bool,
    pub store_minutes_remaining: u32,
    pub web_api_paused: bool,
    pub web_api_minutes_remaining: u32,
}

pub fn rate_limit_status() -> SteamRateLimitStatus {
    let limiter = LIMITER.lock().ok();
    let now = now_unix();
    let (store_until, web_until) = limiter
        .as_ref()
        .map(|l| {
            (
                l.persisted.store_blocked_until,
                l.persisted.web_api_blocked_until,
            )
        })
        .unwrap_or((0, 0));

    let store_remaining = store_until.saturating_sub(now);
    let web_remaining = web_until.saturating_sub(now);

    SteamRateLimitStatus {
        store_paused: store_remaining > 0,
        store_minutes_remaining: store_remaining.div_ceil(60) as u32,
        web_api_paused: web_remaining > 0,
        web_api_minutes_remaining: web_remaining.div_ceil(60) as u32,
    }
}

pub fn reset_cooldowns() -> Result<(), String> {
    let mut limiter = LIMITER.lock().map_err(|e| e.to_string())?;
    limiter.persisted = PersistedCooldowns::default();
    save_persisted(&limiter.persisted);
    Ok(())
}

pub async fn acquire(endpoint: SteamEndpoint) -> Result<(), String> {
    loop {
        let wait_for = {
            let mut limiter = LIMITER.lock().map_err(|e| e.to_string())?;
            let now = now_unix();
            let blocked = blocked_until(endpoint, &limiter.persisted);
            if blocked > now {
                return Err(format!(
                    "{} requests paused for {} more minutes to avoid IP blocks.",
                    endpoint_label(endpoint),
                    (blocked - now).div_ceil(60)
                ));
            }

            let state = match endpoint {
                SteamEndpoint::Store => &mut limiter.store,
                SteamEndpoint::WebApi => &mut limiter.web_api,
            };

            if state.minute_started.elapsed() >= Duration::from_secs(60) {
                state.minute_started = Instant::now();
                state.requests_this_minute = 0;
            }

            if state.requests_this_minute >= max_per_minute(endpoint) {
                Duration::from_secs(60).saturating_sub(state.minute_started.elapsed())
            } else if let Some(last) = state.last_request {
                min_interval(endpoint).saturating_sub(last.elapsed())
            } else {
                Duration::ZERO
            }
        };

        if wait_for.is_zero() {
            break;
        }
        tokio::time::sleep(wait_for).await;
    }

    let mut limiter = LIMITER.lock().map_err(|e| e.to_string())?;
    let state = match endpoint {
        SteamEndpoint::Store => &mut limiter.store,
        SteamEndpoint::WebApi => &mut limiter.web_api,
    };
    state.last_request = Some(Instant::now());
    state.requests_this_minute += 1;
    Ok(())
}

pub fn report_rate_limited(endpoint: SteamEndpoint, status: u16) {
    if status != 403 && status != 429 && status != 503 {
        return;
    }

    let Ok(mut limiter) = LIMITER.lock() else {
        return;
    };

    let until = now_unix() + cooldown_secs(endpoint);
    set_blocked_until(endpoint, &mut limiter.persisted, until);
    save_persisted(&limiter.persisted);
}
