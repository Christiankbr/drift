use crate::config::Config;
use crate::store::{DailySummary, Store};
use crate::ui;
use anyhow::Result;
use chrono::{Local, NaiveDate};
use colored::Colorize;
use std::io::Write;

pub fn daily_report(store: &Store, _config: &Config, date: Option<&str>) -> Result<()> {
    let date = if let Some(d) = date {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")?
    } else {
        Local::now().date_naive()
    };

    let summary = DailySummary::for_date(store, date)?;
    let activities = store.activities_for_date(date)?;
    let switches = store.switches_for_date(date)?;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    writeln!(
        out,
        "\n  {}",
        format!("drift, report — {}", date.format("%A, %B %d, %Y"))
            .cyan()
            .bold()
    )?;
    writeln!(out, "  {}\n", "─".repeat(37).dimmed())?;

    writeln!(
        out,
        "  {:<18} {}",
        "Tracked".dimmed(),
        format_duration(summary.total_tracked)
    )?;
    writeln!(
        out,
        "  {:<18} {}",
        "Switches".dimmed(),
        summary.switch_count
    )?;
    writeln!(
        out,
        "  {:<18} {}",
        "Focus loss".dimmed(),
        format_duration(summary.focus_loss)
    )?;
    writeln!(
        out,
        "  {:<18} {}\n",
        "Score".dimmed(),
        ui::focus_score(summary.focus_score)
    )?;

    writeln!(out, "  {}\n", "Time by category".dimmed())?;
    for (cat, dur) in &summary.by_category {
        let pct = if summary.total_tracked > 0 {
            (*dur as f64 / summary.total_tracked as f64) * 100.0
        } else {
            0.0
        };
        let bar = ui::bar(pct, 20);
        writeln!(
            out,
            "    {:<14} {} {:>6}  {}",
            ui::category_color(cat),
            bar,
            format_duration(*dur),
            format!("({:.0}%)", pct).dimmed()
        )?;
    }

    if !activities.is_empty() {
        writeln!(out, "\n  {}\n", "Timeline".dimmed())?;
        for a in &activities {
            writeln!(
                out,
                "    {}  {:<20}  {}",
                a.timestamp.format("%H:%M").to_string().dimmed(),
                a.app_name,
                ui::category_color(&a.category)
            )?;
        }
    }

    if !switches.is_empty() {
        writeln!(out, "\n  {}\n", "Switches".dimmed())?;
        for s in &switches {
            let cost = if s.cost_mins >= 20 {
                format!("{}min", s.cost_mins).red().to_string()
            } else {
                format!("{}min", s.cost_mins).dimmed().to_string()
            };
            writeln!(
                out,
                "    {}  {} {} {}  ({})",
                s.timestamp.format("%H:%M").to_string().dimmed(),
                ui::category_color(&s.from_category),
                "→".dimmed(),
                ui::category_color(&s.to_category),
                cost
            )?;
        }
    }

    writeln!(out, "\n  {}\n", "Insights".dimmed())?;
    if summary.focus_score >= 70 {
        writeln!(out, "    {} Strong focus day.", "+".green().bold())?;
    } else if summary.focus_score >= 40 {
        writeln!(
            out,
            "    {} Mixed focus. More deep work blocks could help.",
            "~".yellow()
        )?;
    } else {
        writeln!(
            out,
            "    {} High distraction. Try: {}",
            "!".red().bold(),
            "drift focus 90".cyan()
        )?;
    }

    let distraction_time: u64 = summary
        .by_category
        .iter()
        .filter(|(c, _)| c == "distraction")
        .map(|(_, d)| *d)
        .sum();
    if distraction_time > 0 && summary.total_tracked > 0 {
        let pct = (distraction_time as f64 / summary.total_tracked as f64) * 100.0;
        if pct > 20.0 {
            writeln!(
                out,
                "    {} {:.0}% of your day was distraction.",
                "!".red(),
                pct
            )?;
        }
    }

    if summary.switch_count > 30 {
        writeln!(
            out,
            "    {} {} switches — try batching communication.",
            "!".yellow(),
            summary.switch_count
        )?;
    }

    writeln!(out)?;
    Ok(())
}

pub fn export(store: &Store, _config: &Config, format: &str, date: Option<&str>) -> Result<()> {
    let date = if let Some(d) = date {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")?
    } else {
        Local::now().date_naive()
    };

    let activities = store.activities_for_date(date)?;
    let switches = store.switches_for_date(date)?;

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    match format.to_lowercase().as_str() {
        "json" => {
            let json = serde_json::json!({
                "date": date.to_string(),
                "activities": activities.iter().map(|a| serde_json::json!({
                    "timestamp": a.timestamp.to_string(),
                    "app": a.app_name,
                    "title": a.window_title,
                    "category": a.category,
                    "duration_secs": a.duration_secs
                })).collect::<Vec<_>>(),
                "switches": switches.iter().map(|s| serde_json::json!({
                    "timestamp": s.timestamp.to_string(),
                    "from": s.from_category,
                    "to": s.to_category,
                    "cost_mins": s.cost_mins
                })).collect::<Vec<_>>()
            });
            writeln!(out, "{}", serde_json::to_string_pretty(&json)?)?;
        }
        "csv" => {
            writeln!(
                out,
                "timestamp,app_name,window_title,category,duration_secs"
            )?;
            for a in &activities {
                writeln!(
                    out,
                    "{},{},{},{},{}",
                    a.timestamp, a.app_name, a.window_title, a.category, a.duration_secs
                )?;
            }
            writeln!(out)?;
            writeln!(out, "timestamp,from_category,to_category,cost_mins")?;
            for s in &switches {
                writeln!(
                    out,
                    "{},{},{},{}",
                    s.timestamp, s.from_category, s.to_category, s.cost_mins
                )?;
            }
        }
        _ => {
            anyhow::bail!("Unknown format: {}. Use 'json' or 'csv'.", format);
        }
    }

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

pub fn weekly_report(store: &Store, _config: &Config) -> Result<()> {
    let today = Local::now().date_naive();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    writeln!(out, "\n  {}", "drift, weekly report".cyan().bold())?;
    writeln!(out, "  {}\n", "─".repeat(37).dimmed())?;

    let mut week_scores: Vec<u64> = Vec::new();
    let mut week_switches: u64 = 0;
    let mut week_focus_loss: u64 = 0;
    let mut week_tracked: u64 = 0;
    let mut distraction_total: u64 = 0;

    writeln!(out, "  {}\n", "Last 7 days".dimmed())?;
    writeln!(
        out,
        "  {:<12}  {:<6}  {:<8}  {:<10}  {:<10}",
        "Date", "Score", "Switch", "Loss", "Tracked"
    )?;
    writeln!(out, "  {}", "─".repeat(54).dimmed())?;

    for i in (0..7).rev() {
        let date = today - chrono::Duration::days(i as i64);
        let summary = DailySummary::for_date(store, date)?;
        week_scores.push(summary.focus_score);
        week_switches += summary.switch_count;
        week_focus_loss += summary.focus_loss;
        week_tracked += summary.total_tracked;

        for (cat, dur) in &summary.by_category {
            if cat == "distraction" {
                distraction_total += dur;
            }
        }

        writeln!(
            out,
            "  {:<12}  {:<6}  {:<8}  {:<10}  {:<10}",
            date.format("%a %b %d").to_string(),
            ui::focus_score(summary.focus_score),
            summary.switch_count,
            format_duration(summary.focus_loss),
            format_duration(summary.total_tracked)
        )?;
    }

    let avg_score = if !week_scores.is_empty() {
        week_scores.iter().sum::<u64>() / week_scores.len() as u64
    } else {
        0
    };

    let mut prev_scores: Vec<u64> = Vec::new();
    for i in (7..14).rev() {
        let date = today - chrono::Duration::days(i as i64);
        let summary = DailySummary::for_date(store, date)?;
        prev_scores.push(summary.focus_score);
    }
    let prev_avg = if !prev_scores.is_empty() {
        prev_scores.iter().sum::<u64>() / prev_scores.len() as u64
    } else {
        0
    };

    let trend = if avg_score > prev_avg {
        "↑ better".green().to_string()
    } else if avg_score < prev_avg {
        "↓ worse".red().to_string()
    } else {
        "→ same".dimmed().to_string()
    };

    writeln!(out, "\n  {}\n", "Summary".dimmed())?;
    writeln!(
        out,
        "    {:<18} {} ({})",
        "Avg focus score".dimmed(),
        ui::focus_score(avg_score),
        trend
    )?;
    writeln!(
        out,
        "    {:<18} {}",
        "Total switches".dimmed(),
        week_switches
    )?;
    writeln!(
        out,
        "    {:<18} {}",
        "Total focus loss".dimmed(),
        format_duration(week_focus_loss)
    )?;
    writeln!(
        out,
        "    {:<18} {}",
        "Total tracked".dimmed(),
        format_duration(week_tracked)
    )?;
    writeln!(
        out,
        "    {:<18} {}",
        "Distraction".dimmed(),
        format_duration(distraction_total).red()
    )?;

    writeln!(out, "\n  {}\n", "Top distractions".dimmed())?;
    let mut distraction_apps: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();
    for i in (0..7).rev() {
        let date = today - chrono::Duration::days(i as i64);
        let activities = store.activities_for_date(date)?;
        for a in &activities {
            if a.category == "distraction" {
                *distraction_apps.entry(a.app_name.clone()).or_insert(0) += a.duration_secs;
            }
        }
    }
    let mut top_distractions: Vec<(String, u64)> = distraction_apps.into_iter().collect();
    top_distractions.sort_by_key(|b| std::cmp::Reverse(b.1));
    top_distractions.truncate(5);
    for (app, dur) in &top_distractions {
        writeln!(
            out,
            "    {:<20} {}",
            app.red(),
            format_duration(*dur).dimmed()
        )?;
    }

    let streaks = store.streak_history(7)?;
    let best_streak = streaks.iter().map(|(_, s)| *s).max().unwrap_or(0);
    writeln!(
        out,
        "\n  {}:  {}\n",
        "Best streak".dimmed(),
        format_duration(best_streak).green().bold()
    )?;

    writeln!(out)?;
    Ok(())
}
