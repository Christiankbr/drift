use anyhow::Result;
use chrono::Local;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::tracker::create_tracker;

pub fn run_watch(config: &Config) -> Result<()> {
    let tracker = create_tracker()?;
    let interval = Duration::from_secs(config.poll_interval_secs);

    let mut last_app = String::new();
    let mut last_category: Option<crate::config::Category> = None;
    let mut category_start = Local::now();

    println!("\n  drift, live watch\n");
    println!("  ─────────────────────────────────────\n");
    println!("  Tracking active window. Press Ctrl+C to stop.\n");

    loop {
        let window = match tracker.get_active_window() {
            Ok(w) => w,
            Err(e) => {
                eprintln!("  [!] Failed to get active window: {}", e);
                thread::sleep(interval);
                continue;
            }
        };

        let category = config.classify(&window.app_name);
        let now = Local::now();

        if last_category.is_none() || last_category.unwrap() != category {
            category_start = now;
        }

        let streak_secs = (now - category_start).num_seconds().max(0) as u64;
        let streak = format_duration(streak_secs);

        // Clear line and print
        if window.app_name != last_app || last_category.unwrap_or(category) != category {
            println!(
                "  {}  {:<20}  {:<14}  streak: {}",
                now.format("%H:%M:%S"),
                window.app_name,
                category,
                streak
            );
        }

        last_app = window.app_name;
        last_category = Some(category);

        thread::sleep(interval);
    }
}

fn format_duration(secs: u64) -> String {
    let m = secs / 60;
    let s = secs % 60;
    if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
