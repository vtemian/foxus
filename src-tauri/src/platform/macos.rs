use super::{ActiveWindow, PlatformTracker};

pub struct MacOSTracker;

impl MacOSTracker {
    pub fn new() -> Self {
        Self
    }
}

impl PlatformTracker for MacOSTracker {
    fn get_active_window(&self) -> Option<ActiveWindow> {
        // TODO: Implement in Task 6
        Some(ActiveWindow {
            app_name: "Stub".to_string(),
            window_title: "Stub Window".to_string(),
            bundle_id: None,
        })
    }

    fn get_idle_time_secs(&self) -> u64 {
        // TODO: Implement in Task 6
        0
    }
}
