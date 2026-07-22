use crate::tracker::{ActiveWindow, WindowTracker};
use anyhow::Result;
use std::process::Command;

pub struct LinuxTracker {
    // Try x11rb first, fall back to xdotool
    use_xdotool: bool,
}

impl LinuxTracker {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> Result<Box<dyn WindowTracker>> {
        // Check if xdotool is available as fallback
        let has_xdotool = Command::new("which")
            .arg("xdotool")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        Ok(Box::new(LinuxTracker {
            use_xdotool: has_xdotool,
        }))
    }
}

impl WindowTracker for LinuxTracker {
    fn get_active_window(&self) -> Result<ActiveWindow> {
        if self.use_xdotool {
            self.get_via_xdotool()
        } else {
            self.get_via_x11rb()
        }
    }
}

impl LinuxTracker {
    fn get_via_xdotool(&self) -> Result<ActiveWindow> {
        let app_name = Command::new("xdotool")
            .args(["getactivewindow", "getwindowname"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        // Get the process name for better classification
        let pid = Command::new("xdotool")
            .args(["getactivewindow", "getwindowpid"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "0".to_string());

        let proc_name = if pid != "0" {
            Command::new("ps")
                .args(["-p", &pid, "-o", "comm="])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|_| app_name.clone())
        } else {
            app_name.clone()
        };

        Ok(ActiveWindow {
            app_name: proc_name,
            window_title: app_name,
        })
    }

    fn get_via_x11rb(&self) -> Result<ActiveWindow> {
        // Fallback: try reading _NET_ACTIVE_WINDOW from root window via xprop
        let output = Command::new("xprop")
            .args(["-root", "_NET_ACTIVE_WINDOW"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse: _NET_ACTIVE_WINDOW(WINDOW): window id # 0x1234567
        let window_id = stdout
            .split('#')
            .nth(1)
            .and_then(|s| s.split_whitespace().next())
            .ok_or_else(|| anyhow::anyhow!("Could not parse active window from xprop"))?;

        // Get window name
        let name_output = Command::new("xprop")
            .args(["-id", window_id, "_NET_WM_NAME"])
            .output()?;

        let name_stdout = String::from_utf8_lossy(&name_output.stdout);
        let window_title = name_stdout
            .split('=')
            .nth(1)
            .map(|s| s.trim().trim_matches('"').to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Get WM_CLASS for app name
        let class_output = Command::new("xprop")
            .args(["-id", window_id, "WM_CLASS"])
            .output()?;

        let class_stdout = String::from_utf8_lossy(&class_output.stdout);
        let app_name = class_stdout
            .split('=')
            .nth(1)
            .and_then(|s| {
                s.split(',')
                    .next()
                    .map(|p| p.trim().trim_matches('"').to_string())
            })
            .unwrap_or_else(|| window_title.clone());

        Ok(ActiveWindow {
            app_name,
            window_title,
        })
    }
}
