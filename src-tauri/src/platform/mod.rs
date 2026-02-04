pub mod types;

pub use types::{ActiveWindow, PlatformTracker};

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub use macos::MacOSTracker as NativeTracker;

#[cfg(target_os = "linux")]
pub use linux::LinuxTracker as NativeTracker;

// Stub for development on other platforms
#[cfg(not(any(target_os = "macos", target_os = "linux")))]
pub struct NativeTracker;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl PlatformTracker for NativeTracker {
    fn get_active_window(&self) -> Option<ActiveWindow> {
        Some(ActiveWindow {
            app_name: "TestApp".to_string(),
            window_title: "Test Window".to_string(),
            bundle_id: None,
        })
    }

    fn get_idle_time_secs(&self) -> u64 {
        0
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
impl NativeTracker {
    pub fn new() -> Self { Self }
}
