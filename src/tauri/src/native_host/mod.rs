use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::focus::FocusManager;
use crate::models::Activity;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use url::Url;

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingMessage {
    #[serde(rename = "activity")]
    Activity {
        url: String,
        title: String,
        timestamp: i64,
    },
    #[serde(rename = "request_state")]
    RequestState,
    #[serde(rename = "use_distraction_time")]
    UseDistractionTime,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutgoingMessage {
    #[serde(rename = "state")]
    State {
        #[serde(rename = "focusActive")]
        focus_active: bool,
        #[serde(rename = "budgetRemaining")]
        budget_remaining: i32,
        #[serde(rename = "blockedDomains")]
        blocked_domains: Vec<String>,
    },
    #[serde(rename = "budget_updated")]
    BudgetUpdated { remaining: i32 },
    #[serde(rename = "hard_blocked")]
    HardBlocked,
}

/// Chrome Native Messaging protocol maximum message size (1 MB).
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

pub struct NativeHost {
    db: Arc<Mutex<Database>>,
    focus_manager: Arc<FocusManager>,
    categorizer: Arc<Mutex<Categorizer>>,
}

impl NativeHost {
    pub fn new(
        db: Arc<Mutex<Database>>,
        focus_manager: Arc<FocusManager>,
        categorizer: Arc<Mutex<Categorizer>>,
    ) -> Self {
        Self {
            db,
            focus_manager,
            categorizer,
        }
    }

    pub fn run(&self) -> io::Result<()> {
        loop {
            let message = Self::read_message()?;
            let response = self.handle_message(message);

            if let Some(resp) = response {
                Self::write_message(&resp)?;
            }
        }
    }

    #[expect(
        clippy::as_conversions,
        reason = "u32 -> usize widening cast is safe on all supported platforms (32-bit and 64-bit)"
    )]
    fn read_message() -> io::Result<IncomingMessage> {
        // Chrome Native Messaging protocol specifies little-endian byte order
        let mut len_bytes = [0u8; 4];
        io::stdin().read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {len} bytes (max: {MAX_MESSAGE_SIZE} bytes)"),
            ));
        }

        let mut buffer = vec![0u8; len];
        io::stdin().read_exact(&mut buffer)?;

        serde_json::from_slice(&buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "Message size is validated to be <= MAX_MESSAGE_SIZE (1MB), well within u32 range"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "usize -> u32 narrowing cast is safe because json.len() is validated <= MAX_MESSAGE_SIZE (1MB)"
    )]
    fn write_message(message: &OutgoingMessage) -> io::Result<()> {
        let json = serde_json::to_vec(message)?;

        if json.len() > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Outgoing message too large: {} bytes (max: {MAX_MESSAGE_SIZE} bytes)",
                    json.len()
                ),
            ));
        }

        let len = json.len() as u32;

        // Chrome Native Messaging protocol specifies little-endian byte order
        io::stdout().write_all(&len.to_le_bytes())?;
        io::stdout().write_all(&json)?;
        io::stdout().flush()?;

        Ok(())
    }

    fn handle_message(&self, message: IncomingMessage) -> Option<OutgoingMessage> {
        match message {
            IncomingMessage::Activity {
                url,
                title,
                timestamp,
            } => {
                self.record_activity(&url, &title, timestamp);
                None
            }
            IncomingMessage::RequestState => Some(self.get_state()),
            IncomingMessage::UseDistractionTime => self.use_distraction_time(),
        }
    }

    #[expect(
        clippy::cast_possible_wrap,
        reason = "Unix timestamps won't exceed i64::MAX until year 292 billion"
    )]
    #[expect(
        clippy::as_conversions,
        reason = "u64 -> i64 widening cast is safe for timestamps (won't overflow until year 292 billion)"
    )]
    fn record_activity(&self, url: &str, title: &str, _timestamp: i64) {
        // Input validation: limit URL and title length to prevent DoS
        const MAX_URL_LEN: usize = 2048;
        const MAX_TITLE_LEN: usize = 512;

        let url = url.get(..MAX_URL_LEN).unwrap_or(url);
        let title = title.get(..MAX_TITLE_LEN).unwrap_or(title);

        let domain = extract_domain(url);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs() as i64;

        let category_id = match self.categorizer.lock() {
            Ok(cat) => cat.categorize_url(&domain),
            Err(poisoned) => poisoned.into_inner().categorize_url(&domain),
        };

        let mut activity = Activity::new(timestamp, 5, "browser", None, Some(title));
        activity.url = Some(url.to_string());
        activity.domain = Some(domain);
        activity.category_id = Some(category_id);

        if let Ok(db) = self.db.lock() {
            let _ = activity.save(db.connection());
        }
    }

    fn get_state(&self) -> OutgoingMessage {
        match self.focus_manager.get_state() {
            Ok(state) => OutgoingMessage::State {
                focus_active: state.active,
                budget_remaining: state.budget_remaining,
                blocked_domains: state.blocked_domains,
            },
            Err(_) => OutgoingMessage::State {
                focus_active: false,
                budget_remaining: 0,
                blocked_domains: vec![],
            },
        }
    }

    fn use_distraction_time(&self) -> Option<OutgoingMessage> {
        match self.focus_manager.use_distraction_time(30) {
            Ok(Some(remaining)) => {
                if remaining <= 0 {
                    Some(OutgoingMessage::HardBlocked)
                } else {
                    Some(OutgoingMessage::BudgetUpdated { remaining })
                }
            }
            _ => None,
        }
    }
}

/// Extract domain from a URL using proper URL parsing.
/// This handles edge cases like URLs with userinfo (e.g., `<https://user@evil.com@legitimate.com>`).
fn extract_domain(url_str: &str) -> String {
    if let Ok(url) = Url::parse(url_str) {
        url.host_str().unwrap_or("").to_string()
    } else {
        // For malformed URLs that don't start with a scheme, return empty
        // This prevents treating random text as a domain
        if !url_str.starts_with("http://") && !url_str.starts_with("https://") {
            return String::new();
        }
        // Fallback for malformed URLs with scheme: try basic extraction
        url_str
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .and_then(|s| s.split('@').next_back()) // Handle userinfo attacks
            .unwrap_or("")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(
            extract_domain("https://www.reddit.com/r/rust"),
            "www.reddit.com"
        );
        assert_eq!(extract_domain("http://github.com"), "github.com");
        assert_eq!(extract_domain("https://docs.rs/tauri/latest"), "docs.rs");
    }

    #[test]
    fn test_extract_domain_with_userinfo() {
        // This is a potential security attack: URLs with @ can trick naive parsers
        assert_eq!(
            extract_domain("https://evil.com@legitimate.com/"),
            "legitimate.com"
        );
    }

    #[test]
    fn test_extract_domain_empty_and_malformed() {
        assert_eq!(extract_domain(""), "");
        assert_eq!(extract_domain("not-a-url"), "");
        assert_eq!(extract_domain("https://"), "");
    }

    #[test]
    fn test_extract_domain_with_port() {
        assert_eq!(extract_domain("https://localhost:3000/api"), "localhost");
        assert_eq!(
            extract_domain("http://example.com:8080/path"),
            "example.com"
        );
    }
}
