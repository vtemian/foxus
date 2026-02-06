/// Information about the currently active window.
#[derive(Debug, Clone, Default)]
pub struct ActiveWindow {
    pub app_name: String,
    pub window_title: String,
    /// macOS bundle identifier (e.g., "com.apple.Safari").
    /// Populated by platform tracker; kept for future rule matching by bundle ID.
    #[allow(dead_code)]
    pub bundle_id: Option<String>,
}

pub trait PlatformTracker: Send + Sync {
    fn get_active_window(&self) -> Option<ActiveWindow>;
    fn get_idle_time_secs(&self) -> u64;
}
