use crate::config::Config;
use crate::store::Store;
use anyhow::Result;
use chrono::{Datelike, Local, Timelike, Weekday};
use colored::Colorize;
use std::collections::HashMap;

pub fn insights(store: &Store, _config: &Config) -> Result<()> {
    let today = Local::now().date_naive();

    let mut all_activities = Vec::new();
    let mut all_switches = Vec::new();

    for i in 0..7 {
        let date = today - chrono::Duration::days(i);
        let activities = store.activities_for_date(date)?;
        let switches = store.switches_for_date(date)?;
        all_activities.extend(activities);
        all_switches.extend(switches);
    }

    println!("\n  {}", "drift, insights (last 7 days)".cyan().bold());
    println!("  {}\n", "─".repeat(37).dimmed());

    if all_activities.is_empty() {
        println!(
            "  No data yet. Start tracking with: {}\n",
            "drift track".cyan()
        );
        return Ok(());
    }

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
            "distraction" => "distractions".red(),
            "communication" => "communication apps".yellow(),
            "research" => "research/browser".blue(),
            "code" => "code editors".green(),
            "system" => "system tools".dimmed(),
            _ => "other apps".white(),
        };
        println!(
            "  You switch to {} most often ({:.1}x/day, {}x total)",
            app_label, per_day, count
        );
    }

    // 1b. Most frequent specific app switches
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
            "  Your top distraction app is {} ({:.1}x/day)",
            app.red().bold(),
            per_day
        );
    }

    // 2. Best focus time
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
            "  Best focus time: {}{}  ({} focused)",
            format_hour(hour).green().bold(),
            format!("-{}", format_hour(next_hour)).green(),
            format_duration(dur).dimmed()
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
            weekday_name(wd).green().bold(),
            format_duration(dur).dimmed()
        );
    }

    // 4. Context switching time loss
    let total_switch_cost_secs: u64 = all_switches.iter().map(|s| s.cost_mins * 60).sum();
    let per_day_loss = total_switch_cost_secs / 7;
    println!(
        "  You lose ~{}/day to context switching ({} over 7 days)",
        format_duration(per_day_loss).yellow(),
        format_duration(total_switch_cost_secs).dimmed()
    );

    // 5. Distraction time
    let distraction_total: u64 = all_activities
        .iter()
        .filter(|a| a.category == "distraction")
        .map(|a| a.duration_secs)
        .sum();
    let distraction_per_day = distraction_total / 7;
    if distraction_per_day > 0 {
        println!(
            "  You spend ~{}/day on distractions ({} total)",
            format_duration(distraction_per_day).red(),
            format_duration(distraction_total).dimmed()
        );
    }

    // 6. Recommendations
    println!("\n  {}\n", "Recommendations".dimmed());

    if let Some((ref app, count)) = top_app {
        let per_day = count as f64 / 7.0;
        if per_day > 5.0 {
            println!(
                "  {} Try blocking {} during morning focus hours",
                "→".cyan(),
                app.red()
            );
        }
    }

    if let Some((hour, _)) = best_hour {
        if hour < 12 {
            println!(
                "  {} Protect your morning focus ({}-{}h) — avoid Slack/email",
                "→".cyan(),
                hour,
                hour + 1
            );
        } else {
            println!(
                "  {} Your peak focus is at {}h — schedule deep work then",
                "→".cyan(),
                hour
            );
        }
    }

    if per_day_loss > 3600 {
        println!(
            "  {} You're losing {}+/day to switching — try batching communication",
            "→".cyan(),
            format_duration(per_day_loss).yellow()
        );
    }

    if distraction_per_day > 1800 {
        println!(
            "  {} {}+/day on distractions — consider a site blocker or focus mode",
            "→".cyan(),
            format_duration(distraction_per_day).red()
        );
    }

    let total_switches = all_switches.len();
    if total_switches > 100 {
        println!(
            "  {} {} switches in 7 days — try Pomodoro ({})",
            "→".cyan(),
            total_switches,
            "drift focus 25".cyan()
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
