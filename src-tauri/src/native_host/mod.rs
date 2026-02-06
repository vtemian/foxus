use crate::categorizer::Categorizer;
use crate::db::Database;
use crate::focus::FocusManager;
use crate::models::Activity;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingMessage {
    #[serde(rename = "activity")]
    Activity { url: String, title: String, timestamp: i64 },
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
        Self { db, focus_manager, categorizer }
    }

    pub fn run(&self) -> io::Result<()> {
        loop {
            let message = self.read_message()?;
            let response = self.handle_message(message);

            if let Some(resp) = response {
                self.write_message(&resp)?;
            }
        }
    }

    fn read_message(&self) -> io::Result<IncomingMessage> {
        // Chrome Native Messaging protocol specifies little-endian byte order
        let mut len_bytes = [0u8; 4];
        io::stdin().read_exact(&mut len_bytes)?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        // Chrome limits native messaging to 1MB (1024 * 1024 bytes)
        const MAX_MESSAGE_SIZE: usize = 1024 * 1024;
        if len > MAX_MESSAGE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Message too large: {} bytes (max: {} bytes)", len, MAX_MESSAGE_SIZE),
            ));
        }

        let mut buffer = vec![0u8; len];
        io::stdin().read_exact(&mut buffer)?;

        serde_json::from_slice(&buffer)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn write_message(&self, message: &OutgoingMessage) -> io::Result<()> {
        let json = serde_json::to_vec(message)?;
        let len = json.len() as u32;

        // Chrome Native Messaging protocol specifies little-endian byte order
        io::stdout().write_all(&len.to_le_bytes())?;
        io::stdout().write_all(&json)?;
        io::stdout().flush()?;

        Ok(())
    }

    fn handle_message(&self, message: IncomingMessage) -> Option<OutgoingMessage> {
        match message {
            IncomingMessage::Activity { url, title, timestamp } => {
                self.record_activity(&url, &title, timestamp);
                None
            }
            IncomingMessage::RequestState => {
                Some(self.get_state())
            }
            IncomingMessage::UseDistractionTime => {
                self.use_distraction_time()
            }
        }
    }

    fn record_activity(&self, url: &str, title: &str, _timestamp: i64) {
        let domain = extract_domain(url);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let category_id = {
            let cat = self.categorizer.lock().unwrap();
            cat.categorize_url(&domain)
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

fn extract_domain(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_domain() {
        assert_eq!(extract_domain("https://www.reddit.com/r/rust"), "www.reddit.com");
        assert_eq!(extract_domain("http://github.com"), "github.com");
        assert_eq!(extract_domain("https://docs.rs/tauri/latest"), "docs.rs");
    }
}
