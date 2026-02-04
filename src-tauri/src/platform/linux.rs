use super::{ActiveWindow, PlatformTracker};
use x11rb::connection::Connection;
use x11rb::protocol::screensaver;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, Window};

pub struct LinuxTracker {
    conn: x11rb::rust_connection::RustConnection,
    root: Window,
}

impl LinuxTracker {
    pub fn new() -> Self {
        let (conn, screen_num) = x11rb::connect(None).expect("Failed to connect to X server");
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        Self { conn, root }
    }

    fn get_atom(&self, name: &str) -> Option<u32> {
        self.conn
            .intern_atom(false, name.as_bytes())
            .ok()?
            .reply()
            .ok()
            .map(|r| r.atom)
    }

    fn get_window_property(&self, window: Window, atom: u32) -> Option<String> {
        let reply = self
            .conn
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
        let atom = self.get_atom("_NET_ACTIVE_WINDOW")?;
        let reply = self
            .conn
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
        let info = screensaver::query_info(&self.conn, self.root)
            .ok()
            .and_then(|cookie| cookie.reply().ok());

        info.map(|i| (i.ms_since_user_input / 1000) as u64)
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
}
