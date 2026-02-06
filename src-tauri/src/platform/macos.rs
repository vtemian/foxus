use super::{ActiveWindow, PlatformTracker};
use core_graphics::base::CGFloat;
use objc2_app_kit::NSWorkspace;

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

/// Get window title using AppleScript.
///
/// PERFORMANCE NOTE: This spawns a subprocess on every call, which takes ~50-100ms.
/// For high-frequency polling, consider caching results or using accessibility APIs
/// (requires entitlements and user permission).
///
/// The AppleScript approach works without accessibility permissions but is slow.
/// A better long-term solution would use the Accessibility API (AXUIElement) which
/// requires adding the accessibility entitlement and requesting user permission.
fn get_window_title() -> Option<String> {
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
