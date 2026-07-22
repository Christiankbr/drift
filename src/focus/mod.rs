use crate::config::Config;
use crate::store::Store;
use anyhow::Result;
use chrono::Local;
use std::thread;
use std::time::Duration;

pub fn start_focus_mode(store: &Store, config: &Config, minutes: u32) -> Result<()> {
    let session_id = store.start_focus_session(minutes)?;
    let start = Local::now();
    let end = start + chrono::Duration::minutes(minutes as i64);

    println!("\n  drift, focus mode\n");
    println!("  ─────────────────────────────────────\n");
    println!("  Session started at  {}", start.format("%H:%M"));
    println!("  Duration:           {} minutes", minutes);
    println!("  Ends at:            {}", end.format("%H:%M"));
    println!("\n  Tracking context switches during focus...\n");
    println!("  Press Ctrl+C to end early.\n");

    let tracker = crate::tracker::create_tracker()?;
    let interval = Duration::from_secs(config.poll_interval_secs);
    let mut last_category: Option<crate::config::Category> = None;
    let mut switch_count: u32 = 0;
    let mut interrupted = false;

    loop {
        let now = Local::now();
        if now >= end {
            break;
        }

        thread::sleep(interval);

        let window = match tracker.get_active_window() {
            Ok(w) => w,
            Err(_) => continue,
        };

        let category = config.classify(&window.app_name);

        if let Some(prev) = &last_category
            && *prev != category
        {
            switch_count += 1;
            let cost = crate::switch::switch_cost(*prev, category, config.switching_cost_mins);

            store.insert_switch(now.naive_local(), *prev, category, cost)?;

            let warning = if category.is_focus_breaking() {
                format!("  [!] DISTRACTION: {} ({})", window.app_name, category)
            } else {
                format!("  [~] switch: {} -> {}", prev, category)
            };

            println!("  {} {}", now.format("%H:%M:%S"), warning);

            // Check if focus is broken by a distraction
            if category == crate::config::Category::Distraction {
                println!("  [!] Focus broken by distraction!");
                interrupted = true;
            }
        }

        last_category = Some(category);
    }

    store.end_focus_session(session_id, interrupted, switch_count)?;

    println!("\n  ─────────────────────────────────────\n");
    println!("  Focus session complete.\n");
    println!("  Duration:      {} minutes", minutes);
    println!("  Switches:      {}", switch_count);
    println!(
        "  Interrupted:   {}",
        if interrupted { "yes" } else { "no" }
    );

    if switch_count == 0 {
        println!("\n  [+] Perfect focus. Zero context switches.");
    } else if switch_count <= 3 {
        println!("\n  [+] Good focus. Minimal switching.");
    } else {
        println!(
            "\n  [!] {} switches. Try removing distractions next time.",
            switch_count
        );
    }

    println!();
    Ok(())
}
