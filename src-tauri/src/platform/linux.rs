use super::{ActiveWindow, PlatformTracker};

pub struct LinuxTracker;

impl LinuxTracker {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformTracker for LinuxTracker {
    fn get_active_window(&self) -> Option<ActiveWindow> {
        // TODO: Implement in Task 15
        Some(ActiveWindow {
            app_name: "Stub".to_string(),
            window_title: "Stub Window".to_string(),
            bundle_id: None,
        })
    }

    fn get_idle_time_secs(&self) -> u64 {
        // TODO: Implement in Task 15
        0
    }
}
