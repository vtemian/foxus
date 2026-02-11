use super::{ActiveWindow, PlatformTracker};
use x11rb::connection::Connection;
use x11rb::protocol::screensaver;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};

pub struct LinuxTracker {
    conn: Option<x11rb::rust_connection::RustConnection>,
    root: Window,
}

impl Default for LinuxTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LinuxTracker {
    pub fn new() -> Self {
        match x11rb::connect(None) {
            Ok((conn, screen_num)) => {
                // Validate screen_num is within bounds to avoid potential panic
                let setup = conn.setup();
                if screen_num >= setup.roots.len() {
                    eprintln!(
                        "Warning: Invalid screen number {} (only {} screens available). Window tracking disabled.",
                        screen_num,
                        setup.roots.len()
                    );
                    return Self { conn: None, root: 0 };
                }
                let screen = &setup.roots[screen_num];
                let root = screen.root;
                Self {
                    conn: Some(conn),
                    root,
                }
            }
            Err(e) => {
                // Log the error but don't panic - the tracker will return empty results
                // This allows the app to run on Wayland or headless systems
                eprintln!(
                    "Warning: Failed to connect to X server: {}. Window tracking disabled.",
                    e
                );
                Self { conn: None, root: 0 }
            }
        }
    }

    fn get_atom(&self, name: &str) -> Option<u32> {
        self.conn.as_ref()?
            .intern_atom(false, name.as_bytes())
            .ok()?
            .reply()
            .ok()
            .map(|r| r.atom)
    }

    fn get_window_property(&self, window: Window, atom: u32) -> Option<String> {
        let reply = self.conn.as_ref()?
            .get_property(false, window, atom, AtomEnum::ANY, 0, 1024)
            .ok()?
            .reply()
            .ok()?;

        if reply.value.is_empty() {
            return None;
        }

        String::from_utf8(reply.value).ok()
    }

    fn get_active_window_id(&self) -> Option<Window> {
        let conn = self.conn.as_ref()?;
        let atom = self.get_atom("_NET_ACTIVE_WINDOW")?;
        let reply = conn
            .get_property(false, self.root, atom, AtomEnum::WINDOW, 0, 1)
            .ok()?
            .reply()
            .ok()?;

        if reply.value.len() >= 4 {
            Some(u32::from_ne_bytes([
                reply.value[0],
                reply.value[1],
                reply.value[2],
                reply.value[3],
            ]))
        } else {
            None
        }
    }
}

impl PlatformTracker for LinuxTracker {
    fn get_active_window(&self) -> Option<ActiveWindow> {
        let window_id = self.get_active_window_id()?;

        let name_atom = self
            .get_atom("_NET_WM_NAME")
            .or_else(|| Some(AtomEnum::WM_NAME.into()))?;

        let window_title = self
            .get_window_property(window_id, name_atom)
            .unwrap_or_else(|| "Unknown".to_string());

        let class_atom = AtomEnum::WM_CLASS.into();
        let app_name = self
            .get_window_property(window_id, class_atom)
            .map(|s| s.split('\0').next().unwrap_or("Unknown").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        Some(ActiveWindow {
            app_name,
            window_title,
            bundle_id: None,
        })
    }

    fn get_idle_time_secs(&self) -> u64 {
        let Some(conn) = self.conn.as_ref() else {
            return 0;
        };

        let info = screensaver::query_info(conn, self.root)
            .ok()
            .and_then(|cookie| cookie.reply().ok());

        info.map(|i| u64::from(i.ms_since_user_input / 1000))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires X11 display
    fn test_get_active_window() {
        let tracker = LinuxTracker::new();
        if let Some(window) = tracker.get_active_window() {
            println!("Active: {} - {}", window.app_name, window.window_title);
        }
    }

    #[test]
    #[ignore] // Requires X11 display
    fn test_get_idle_time() {
        let tracker = LinuxTracker::new();
        let idle = tracker.get_idle_time_secs();
        // Should be a reasonable value (less than a day in seconds)
        assert!(idle < 86400);
        println!("Idle time: {} seconds", idle);
    }
}
