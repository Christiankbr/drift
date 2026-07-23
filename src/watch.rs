use anyhow::Result;
use chrono::Local;
use colored::Colorize;
use std::thread;
use std::time::Duration;

use crate::config::Config;
use crate::tracker::create_tracker;
use crate::ui;

pub fn run_watch(config: &Config) -> Result<()> {
    let tracker = create_tracker()?;
    let interval = Duration::from_secs(config.poll_interval_secs);

    let mut last_app = String::new();
    let mut last_category: Option<crate::config::Category> = None;
    let mut category_start = Local::now();

    println!("\n  {}", "drift, live watch".cyan().bold());
    println!("  {}\n", "─".repeat(37).dimmed());
    println!("  Tracking active window. Press Ctrl+C to stop.\n");

    loop {
        let window = match tracker.get_active_window() {
            Ok(w) => w,
            Err(e) => {
                eprintln!("  {} Failed to get active window: {}", "!".red(), e);
                thread::sleep(interval);
                continue;
            }
        };

        // Skip ignored apps
        if config.is_ignored(&window.app_name) {
            thread::sleep(interval);
            continue;
        }

        let category = config.classify(&window.app_name);
        let now = Local::now();

        if last_category.is_none() || last_category.unwrap() != category {
            category_start = now;
        }

        let streak_secs = (now - category_start).num_seconds().max(0) as u64;
        let streak = format_duration(streak_secs);

        if window.app_name != last_app || last_category.unwrap_or(category) != category {
            let cat_str = ui::category_color(category.as_str());
            let streak_colored = if streak_secs >= 600 {
                streak.green().bold()
            } else if streak_secs >= 60 {
                streak.normal()
            } else {
                streak.dimmed()
            };
            println!(
                "  {}  {:<20}  {}  streak: {}",
                now.format("%H:%M:%S").to_string().dimmed(),
                window.app_name,
                cat_str,
                streak_colored
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
