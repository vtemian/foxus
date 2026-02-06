use super::{ActiveWindow, PlatformTracker};
use core_graphics::base::CGFloat;
use objc2_app_kit::NSWorkspace;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Cache for window title to avoid spawning osascript subprocess on every call.
/// The subprocess takes ~50-100ms, so we cache results for 1 second.
struct WindowTitleCache {
    title: Option<String>,
    last_updated: Option<Instant>,
}

impl WindowTitleCache {
    const TTL: Duration = Duration::from_secs(1);

    fn new() -> Self {
        Self {
            title: None,
            last_updated: None,
        }
    }

    fn get(&self) -> Option<&Option<String>> {
        match self.last_updated {
            Some(t) if t.elapsed() < Self::TTL => Some(&self.title),
            _ => None,
        }
    }

    fn set(&mut self, title: Option<String>) {
        self.title = title;
        self.last_updated = Some(Instant::now());
    }
}

static WINDOW_TITLE_CACHE: Mutex<Option<WindowTitleCache>> = Mutex::new(None);

pub struct MacOSTracker;

impl Default for MacOSTracker {
    fn default() -> Self {
        Self
    }
}

impl MacOSTracker {
    pub fn new() -> Self {
        Self::default()
    }
}

impl PlatformTracker for MacOSTracker {
    fn get_active_window(&self) -> Option<ActiveWindow> {
        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let app = workspace.frontmostApplication()?;

            let app_name = app
                .localizedName()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            let bundle_id = app.bundleIdentifier().map(|s| s.to_string());

            // Window title requires accessibility permissions
            // For now, use app name as window title fallback
            let window_title = get_window_title().unwrap_or_else(|| app_name.clone());

            Some(ActiveWindow {
                app_name,
                window_title,
                bundle_id,
            })
        }
    }

    fn get_idle_time_secs(&self) -> u64 {
        get_idle_time_secs_internal()
    }
}

/// Get system idle time using CoreGraphics CGEventSource API
fn get_idle_time_secs_internal() -> u64 {
    // CGEventSourceSecondsSinceLastEventType is not exposed by the core-graphics crate,
    // so we use the raw FFI binding directly.
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(
            state_id: u32,
            event_type: u32,
        ) -> CGFloat;
    }

    // kCGEventSourceStateCombinedSessionState = 0
    const COMBINED_SESSION_STATE: u32 = 0;
    // kCGAnyInputEventType = 0xFFFFFFFF (u32::MAX)
    // This captures all input event types (mouse, keyboard, etc.)
    const ANY_INPUT_EVENT_TYPE: u32 = u32::MAX;

    let idle_secs =
        unsafe { CGEventSourceSecondsSinceLastEventType(COMBINED_SESSION_STATE, ANY_INPUT_EVENT_TYPE) };

    idle_secs.max(0.0) as u64
}

/// Get window title using AppleScript with caching.
///
/// This spawns a subprocess which takes ~50-100ms, so results are cached
/// for 1 second to avoid excessive overhead when polling frequently.
///
/// The AppleScript approach works without accessibility permissions but is slow.
/// A better long-term solution would use the Accessibility API (AXUIElement) which
/// requires adding the accessibility entitlement and requesting user permission.
fn get_window_title() -> Option<String> {
    // Check cache first
    {
        let guard = WINDOW_TITLE_CACHE.lock().unwrap();
        if let Some(cache) = guard.as_ref() {
            if let Some(cached_title) = cache.get() {
                return cached_title.clone();
            }
        }
    }

    // Cache miss - fetch fresh title
    let title = fetch_window_title_uncached();

    // Update cache
    {
        let mut guard = WINDOW_TITLE_CACHE.lock().unwrap();
        let cache = guard.get_or_insert_with(WindowTitleCache::new);
        cache.set(title.clone());
    }

    title
}

/// Fetch window title from osascript without caching.
fn fetch_window_title_uncached() -> Option<String> {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(r#"tell application "System Events" to get name of first window of (first application process whose frontmost is true)"#)
        .output()
        .ok()?;

    if output.status.success() {
        let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !title.is_empty() {
            return Some(title);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires GUI environment - run manually with: cargo test macos -- --ignored --nocapture
    fn test_get_active_window() {
        let tracker = MacOSTracker::new();
        // This test may fail in CI but should work locally
        if let Some(window) = tracker.get_active_window() {
            assert!(!window.app_name.is_empty());
            println!("Active: {} - {}", window.app_name, window.window_title);
        }
    }

    #[test]
    #[ignore] // Requires GUI environment - run manually with: cargo test macos -- --ignored --nocapture
    fn test_get_idle_time() {
        let tracker = MacOSTracker::new();
        let idle = tracker.get_idle_time_secs();
        // Should be a reasonable value (less than a day in seconds)
        assert!(idle < 86400);
        println!("Idle time: {} seconds", idle);
    }
}
