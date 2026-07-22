use crate::tracker::{ActiveWindow, WindowTracker};
use anyhow::Result;
use std::process::Command;

pub struct WindowsTracker;

impl WindowsTracker {
    pub fn new() -> Result<Box<dyn WindowTracker>> {
        Ok(Box::new(WindowsTracker))
    }
}

impl WindowTracker for WindowsTracker {
    fn get_active_window(&self) -> Result<ActiveWindow> {
        // Use PowerShell to get active window
        let script = r#"
            Add-Type @"
            using System;
            using System.Runtime.InteropServices;
            using System.Text;
            public class Win32 {
                [DllImport("user32.dll")]
                public static extern IntPtr GetForegroundWindow();
                [DllImport("user32.dll")]
                public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);
                [DllImport("user32.dll")]
                public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);
            }
"@
            $hwnd = [Win32]::GetForegroundWindow()
            $title = New-Object System.Text.StringBuilder 256
            [Win32]::GetWindowText($hwnd, $title, 256) | Out-Null
            $pid = 0
            [Win32]::GetWindowThreadProcessId($hwnd, [ref]$pid) | Out-Null
            $proc = Get-Process -Id $pid -ErrorAction SilentlyContinue
            $appName = if ($proc) { $proc.ProcessName } else { "unknown" }
            "$appName|$($title.ToString())"
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()?;

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
