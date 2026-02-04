#[derive(Debug, Clone, Default)]
pub struct ActiveWindow {
    pub app_name: String,
    pub window_title: String,
    pub bundle_id: Option<String>,
}

pub trait PlatformTracker: Send + Sync {
    fn get_active_window(&self) -> Option<ActiveWindow>;
    fn get_idle_time_secs(&self) -> u64;
}
