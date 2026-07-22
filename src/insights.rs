use crate::config::Config;
use crate::store::Store;
use anyhow::Result;
use chrono::{Datelike, Local, Timelike, Weekday};
use std::collections::HashMap;

pub fn insights(store: &Store, _config: &Config) -> Result<()> {
    let today = Local::now().date_naive();

    // Collect data for the last 7 days
    let mut all_activities = Vec::new();
    let mut all_switches = Vec::new();

    for i in 0..7 {
        let date = today - chrono::Duration::days(i);
        let activities = store.activities_for_date(date)?;
        let switches = store.switches_for_date(date)?;
        all_activities.extend(activities);
        all_switches.extend(switches);
    }

    if all_activities.is_empty() {
        println!("\n  drift, insights (last 7 days)\n");
        println!("  ─────────────────────────────────────\n");
        println!("  No data yet. Start tracking with: drift track\n");
        return Ok(());
    }

    println!("\n  drift, insights (last 7 days)\n");
    println!("  ─────────────────────────────────────\n");

    // 1. Most frequent switch target
    let switch_to_counts: HashMap<&str, u32> =
        all_switches.iter().fold(HashMap::new(), |mut acc, s| {
            *acc.entry(&s.to_category).or_insert(0) += 1;
            acc
        });
    let most_switched_to = switch_to_counts
        .iter()
        .max_by_key(|(_, c)| *c)
        .map(|(cat, count)| (*cat, *count));

    if let Some((cat, count)) = most_switched_to {
        let per_day = count as f64 / 7.0;
        let app_label = match cat {
            "distraction" => "distractions",
            "communication" => "communication apps",
            "research" => "research/browser",
            "code" => "code editors",
            "system" => "system tools",
            _ => "other apps",
        };
        println!(
            "  You switch to {} most often ({:.1}x/day, {}x total)",
            app_label, per_day, count
        );
    }

    // 1b. Most frequent specific app switches (by app name)
    let mut app_switch_counts: HashMap<String, u32> = HashMap::new();
    for a in &all_activities {
        if a.category == "distraction" || a.category == "communication" {
            *app_switch_counts.entry(a.app_name.clone()).or_insert(0) += 1;
        }
    }
    let top_app = app_switch_counts
        .iter()
        .max_by_key(|(_, c)| *c)
        .map(|(app, count)| (app.clone(), *count));

    if let Some((ref app, count)) = top_app {
        let per_day = count as f64 / 7.0;
        println!(
            "  Your top distraction app is \"{}\" ({:.1}x/day)",
            app, per_day
        );
    }

    // 2. Best focus time (hour of day with most focus time)
    let mut hour_focus: HashMap<u32, u64> = HashMap::new();
    for a in &all_activities {
        if a.category == "code" || a.category == "research" {
            let hour = a.timestamp.hour();
            *hour_focus.entry(hour).or_insert(0) += a.duration_secs;
        }
    }
    let best_hour = hour_focus
        .iter()
        .max_by_key(|(_, d)| *d)
        .map(|(h, d)| (*h, *d));

    if let Some((hour, dur)) = best_hour {
        let next_hour = (hour + 1) % 24;
        println!(
            "  Your best focus time is {}-{} ({} of focused work)",
            format_hour(hour),
            format_hour(next_hour),
            format_duration(dur)
        );
    }

    // 3. Most productive weekday
    let mut weekday_focus: HashMap<Weekday, u64> = HashMap::new();
    for a in &all_activities {
        if a.category == "code" || a.category == "research" {
            let wd = a.timestamp.weekday();
            *weekday_focus.entry(wd).or_insert(0) += a.duration_secs;
        }
    }

    let best_weekday = weekday_focus
        .iter()
        .max_by_key(|(_, d)| *d)
        .map(|(wd, d)| (*wd, *d));

    if let Some((wd, dur)) = best_weekday {
        println!(
            "  {}s are your most productive day ({} focused)",
            weekday_name(wd),
            format_duration(dur)
        );
    }

    // 4. Estimate context switching time loss
    let total_switch_cost_secs: u64 = all_switches.iter().map(|s| s.cost_mins * 60).sum();
    let per_day_loss = total_switch_cost_secs / 7;
    println!(
        "  You lose ~{}/day to context switching ({} over 7 days)",
        format_duration(per_day_loss),
        format_duration(total_switch_cost_secs)
    );

    // 5. Distraction time total
    let distraction_total: u64 = all_activities
        .iter()
        .filter(|a| a.category == "distraction")
        .map(|a| a.duration_secs)
        .sum();
    let distraction_per_day = distraction_total / 7;
    if distraction_per_day > 0 {
        println!(
            "  You spend ~{}/day on distractions ({} total)",
            format_duration(distraction_per_day),
            format_duration(distraction_total)
        );
    }

    // 6. Actionable recommendations
    println!("\n  Recommendations:\n");

    if let Some((ref app, count)) = top_app {
        let per_day = count as f64 / 7.0;
        if per_day > 5.0 {
            println!("  → Try blocking \"{}\" during morning focus hours", app);
        }
    }

    if let Some((hour, _)) = best_hour {
        if hour < 12 {
            println!(
                "  → Protect your morning focus ({}-{}h) — avoid Slack/email",
                hour,
                hour + 1
            );
        } else {
            println!(
                "  → Your peak focus is at {}h — schedule deep work then",
                hour
            );
        }
    }

    if per_day_loss > 3600 {
        println!(
            "  → You're losing {}+/day to switching — try batching communication",
            format_duration(per_day_loss)
        );
    }

    if distraction_per_day > 1800 {
        println!(
            "  → {}+/day on distractions — consider a site blocker or focus mode",
            format_duration(distraction_per_day)
        );
    }

    let total_switches = all_switches.len();
    if total_switches > 100 {
        println!(
            "  → {} switches in 7 days — try the Pomodoro technique (drift focus 25)",
            total_switches
        );
    }

    println!();
    Ok(())
}

fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else if m > 0 {
        format!("{}m", m)
    } else {
        format!("{}s", secs)
    }
}

fn format_hour(hour: u32) -> String {
    if hour == 0 {
        "12am".to_string()
    } else if hour < 12 {
        format!("{}am", hour)
    } else if hour == 12 {
        "12pm".to_string()
    } else {
        format!("{}pm", hour - 12)
    }
}

fn weekday_name(wd: Weekday) -> &'static str {
    match wd {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}
