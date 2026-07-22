use crate::tracker::{ActiveWindow, WindowTracker};
use anyhow::Result;
use std::process::Command;

pub struct MacTracker;

impl MacTracker {
    pub fn new() -> Result<Box<dyn WindowTracker>> {
        Ok(Box::new(MacTracker))
    }
}

impl WindowTracker for MacTracker {
    fn get_active_window(&self) -> Result<ActiveWindow> {
        // Use osascript to get frontmost app and window title
        let script = r#"
            tell application "System Events"
                set frontApp to first application process whose frontmost is true
                set appName to name of frontApp
                try
                    set winTitle to title of first window of frontApp
                on error
                    set winTitle to ""
                end try
                return appName & "|" & winTitle
            end tell
        "#;

        let output = Command::new("osascript").args(["-e", script]).output()?;

        let result = String::from_utf8_lossy(&output.stdout);
        let parts: Vec<&str> = result.trim().splitn(2, '|').collect();

        let app_name = parts
            .first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let window_title = parts
            .get(1)
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::new());

        Ok(ActiveWindow {
            app_name,
            window_title,
        })
    }
}
