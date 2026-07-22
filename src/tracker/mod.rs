use crate::config::{Category, Config};
use crate::store::Store;
use anyhow::Result;
use chrono::Local;
use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub struct ActiveWindow {
    pub app_name: String,
    pub window_title: String,
}

pub trait WindowTracker {
    fn get_active_window(&self) -> Result<ActiveWindow>;
}

#[cfg(target_os = "linux")]
pub fn create_tracker() -> Result<Box<dyn WindowTracker>> {
    linux::LinuxTracker::new()
}

#[cfg(target_os = "macos")]
pub fn create_tracker() -> Result<Box<dyn WindowTracker>> {
    macos::MacTracker::new()
}

#[cfg(target_os = "windows")]
pub fn create_tracker() -> Result<Box<dyn WindowTracker>> {
    windows::WindowsTracker::new()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn create_tracker() -> Result<Box<dyn WindowTracker>> {
    anyhow::bail!("Unsupported platform")
}

pub fn run_daemon(store: &Store, config: &Config, interval_secs: u64, alert: bool) -> Result<()> {
    let tracker = create_tracker()?;
    let interval = Duration::from_secs(interval_secs);

    let mut _last_app: Option<String> = None;
    let mut last_category: Option<Category> = None;
    let mut current_start = Local::now().naive_local();
    let mut current_app = String::new();
    let mut current_title = String::new();
    let mut current_category = Category::Other;
    let mut first_iteration = true;
    let mut last_alert_time: Option<chrono::NaiveDateTime> = None;

    tracing::info!("drift tracker started, polling every {}s", interval_secs);
    if alert {
        println!("  drift tracker running with alerts enabled\n");
    }

    loop {
        thread::sleep(interval);

        let window = match tracker.get_active_window() {
            Ok(w) => w,
            Err(e) => {
                tracing::warn!("Failed to get active window: {}", e);
                continue;
            }
        };

        let category = config.classify(&window.app_name);
        let now = Local::now().naive_local();

        if first_iteration {
            first_iteration = false;
            current_start = now;
            current_app = window.app_name.clone();
            current_title = window.window_title.clone();
            current_category = category;
            last_category = Some(category);
            continue;
        }

        // Category changed = context switch
        if let Some(prev_cat) = last_category
            && prev_cat != category
        {
            // Record the previous activity segment
            let duration = (now - current_start).num_seconds().max(0) as u64;
            store.insert_activity(
                current_start,
                &current_app,
                &current_title,
                current_category,
                duration,
            )?;

            // Record the context switch
            let cost_mins = if prev_cat.is_focus_breaking() || category.is_focus_breaking() {
                config.switching_cost_mins
            } else {
                // Switches within same focus area are cheaper
                config.switching_cost_mins / 3
            };
            store.insert_switch(now, prev_cat, category, cost_mins)?;

            tracing::debug!(
                "Switch: {} -> {} (cost: {}min)",
                prev_cat,
                category,
                cost_mins
            );

            // Start new segment
            current_start = now;
            current_app = window.app_name.clone();
            current_title = window.window_title.clone();
            current_category = category;

            // Alert on distraction (rate limited: 1 per 5 min)
            if alert && category == Category::Distraction {
                let should_alert = last_alert_time.is_none_or(|t| (now - t).num_seconds() >= 300);
                if should_alert {
                    last_alert_time = Some(now);
                    send_alert(&window.app_name);
                }
            }
        }

        last_category = Some(category);
        _last_app = Some(window.app_name);
    }
}

fn send_alert(app_name: &str) {
    let msg = format!("Distraction detected: {}", app_name);
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("notify-send")
            .args(["drift", &msg])
            .spawn()
            .ok();
    }
    #[cfg(target_os = "macos")]
    {
        let script = format!(
            "display notification \"{}\" with title \"drift\"",
            msg.replace('\"', "\\\"")
        );
        std::process::Command::new("osascript")
            .args(["-e", &script])
            .spawn()
            .ok();
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("msg")
            .args([std::env::var("USERNAME").unwrap_or_default(), &msg])
            .spawn()
            .ok();
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = msg;
    }
}
