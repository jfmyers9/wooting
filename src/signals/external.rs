use serde_json::Value;
use std::env;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExternalStatus {
    Idle,
    Running,
    Positive,
    Negative,
    Alert,
    Stale,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExternalSnapshot {
    pub status: ExternalStatus,
    pub event_key: String,
    pub message: String,
}

impl ExternalSnapshot {
    pub fn idle(message: impl Into<String>) -> Self {
        Self {
            status: ExternalStatus::Idle,
            event_key: "idle".to_string(),
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            status: ExternalStatus::Error,
            event_key: format!("error:{message}"),
            message,
        }
    }

    pub fn stale(message: impl Into<String>) -> Self {
        Self {
            status: ExternalStatus::Stale,
            event_key: "stale".to_string(),
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ExternalPollState {
    next_poll: Instant,
    last_success: Option<Instant>,
    last_event_key: Option<String>,
    last_alert: Option<Instant>,
}

impl Default for ExternalPollState {
    fn default() -> Self {
        Self {
            next_poll: Instant::now(),
            last_success: None,
            last_event_key: None,
            last_alert: None,
        }
    }
}

impl ExternalPollState {
    pub fn should_poll(&self, now: Instant) -> bool {
        now >= self.next_poll
    }

    pub fn mark_success(&mut self, snapshot: &ExternalSnapshot, now: Instant, poll_seconds: u64) {
        self.record(snapshot, now);
        self.last_success = Some(now);
        self.next_poll = now + Duration::from_secs(poll_seconds.max(5));
    }

    pub fn mark_error(&mut self, snapshot: &ExternalSnapshot, now: Instant, poll_seconds: u64) {
        self.record(snapshot, now);
        self.next_poll = now + Duration::from_secs((poll_seconds * 2).clamp(10, 300));
    }

    pub fn stale_snapshot(&mut self, now: Instant, stale_seconds: u64) -> Option<ExternalSnapshot> {
        let last_success = self.last_success?;
        if last_success.elapsed() > Duration::from_secs(stale_seconds.max(1)) {
            let snapshot = ExternalSnapshot::stale("external source status is stale");
            self.record(&snapshot, now);
            Some(snapshot)
        } else {
            None
        }
    }

    pub fn record(&mut self, snapshot: &ExternalSnapshot, now: Instant) {
        if self.last_event_key.as_deref() != Some(snapshot.event_key.as_str()) {
            self.last_event_key = Some(snapshot.event_key.clone());
            self.last_alert = Some(now);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExternalFetchError {
    #[error("external API URL is not configured")]
    MissingUrl,
    #[error("external API request failed: {0}")]
    Request(String),
    #[error("external API response was not valid text: {0}")]
    Body(#[from] std::io::Error),
    #[error("external API response was not valid JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub fn fetch_json(api_url: &str, token_env: &str) -> Result<Value, ExternalFetchError> {
    if api_url.trim().is_empty() {
        return Err(ExternalFetchError::MissingUrl);
    }

    let token = env::var(token_env).ok();
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(10))
        .build();
    let mut request = agent
        .get(api_url)
        .set("Accept", "application/json")
        .set("User-Agent", "wooting-signals");
    if let Some(token) = token.filter(|token| !token.is_empty()) {
        request = request.set("Authorization", &format!("Bearer {token}"));
    }
    let body = request
        .call()
        .map_err(|error| ExternalFetchError::Request(error.to_string()))?
        .into_string()?;
    Ok(serde_json::from_str(&body)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poll_state_dedupes_repeated_event_keys() {
        let mut state = ExternalPollState::default();
        let now = Instant::now();
        let snapshot = ExternalSnapshot {
            status: ExternalStatus::Alert,
            event_key: "event-1".to_string(),
            message: "alert".to_string(),
        };

        state.mark_success(&snapshot, now, 60);
        state.mark_success(&snapshot, now + Duration::from_secs(1), 60);

        assert_eq!(state.last_event_key.as_deref(), Some("event-1"));
    }

    #[test]
    fn empty_url_is_missing_url_error() {
        assert!(matches!(
            fetch_json("", "TOKEN"),
            Err(ExternalFetchError::MissingUrl)
        ));
    }
}
