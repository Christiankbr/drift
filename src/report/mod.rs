use crate::config::Config;
use crate::store::{DailySummary, Store};
use anyhow::Result;
use chrono::{Local, NaiveDate};
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
        "\n  drift, daily report for {}\n",
        date.format("%A, %B %d, %Y")
    )?;
    writeln!(out, "  ─────────────────────────────────────\n")?;

    // Overview
    writeln!(
        out,
        "  Tracked time:     {}",
        format_duration(summary.total_tracked)
    )?;
    writeln!(out, "  Context switches: {}", summary.switch_count)?;
    writeln!(
        out,
        "  Focus loss:       {}",
        format_duration(summary.focus_loss)
    )?;
    writeln!(out, "  Focus score:      {}/100\n", summary.focus_score)?;

    // Category breakdown
    writeln!(out, "  Time by category:\n")?;
    for (cat, dur) in &summary.by_category {
        let pct = if summary.total_tracked > 0 {
            (*dur as f64 / summary.total_tracked as f64) * 100.0
        } else {
            0.0
        };
        let bar_len = (pct / 5.0) as usize;
        let bar: String = "█".repeat(bar_len);
        writeln!(
            out,
            "    {:<14} {} {:>5}  ({:.0}%)",
            cat,
            bar,
            format_duration(*dur),
            pct
        )?;
    }

    // Timeline
    if !activities.is_empty() {
        writeln!(out, "\n  Activity timeline:\n")?;
        for a in &activities {
            writeln!(
                out,
                "    {}  {:<20}  {}",
                a.timestamp.format("%H:%M"),
                a.app_name,
                a.category
            )?;
        }
    }

    // Switches
    if !switches.is_empty() {
        writeln!(out, "\n  Context switches:\n")?;
        for s in &switches {
            writeln!(
                out,
                "    {}  {} -> {}  ({}min loss)",
                s.timestamp.format("%H:%M"),
                s.from_category,
                s.to_category,
                s.cost_mins
            )?;
        }
    }

    // Insights
    writeln!(out, "\n  Insights:\n")?;
    if summary.focus_score >= 70 {
        writeln!(out, "    [+] Strong focus day. Keep it up.")?;
    } else if summary.focus_score >= 40 {
        writeln!(
            out,
            "    [~] Mixed focus. More deep work blocks could help."
        )?;
    } else {
        writeln!(
            out,
            "    [!] High distraction. Consider focus mode: drift focus 90"
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
            writeln!(out, "    [!] {:.0}% of your day was distraction.", pct)?;
        }
    }

    if summary.switch_count > 30 {
        writeln!(
            out,
            "    [!] {} switches is above average. Try batching communication.",
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
            writeln!(out, "{{")?;
            writeln!(out, "  \"date\": \"{}\",", date)?;
            writeln!(out, "  \"activities\": [")?;
            for (i, a) in activities.iter().enumerate() {
                write!(
                    out,
                    "    {{\"timestamp\": \"{}\", \"app\": \"{}\", \"title\": \"{}\", \"category\": \"{}\", \"duration_secs\": {}}}",
                    a.timestamp, a.app_name, a.window_title, a.category, a.duration_secs
                )?;
                if i < activities.len() - 1 {
                    writeln!(out, ",")?;
                } else {
                    writeln!(out)?;
                }
            }
            writeln!(out, "  ],")?;
            writeln!(out, "  \"switches\": [")?;
            for (i, s) in switches.iter().enumerate() {
                write!(
                    out,
                    "    {{\"timestamp\": \"{}\", \"from\": \"{}\", \"to\": \"{}\", \"cost_mins\": {}}}",
                    s.timestamp, s.from_category, s.to_category, s.cost_mins
                )?;
                if i < switches.len() - 1 {
                    writeln!(out, ",")?;
                } else {
                    writeln!(out)?;
                }
            }
            writeln!(out, "  ]")?;
            writeln!(out, "}}")?;
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

    writeln!(out, "\n  drift, weekly report\n")?;
    writeln!(out, "  ─────────────────────────────────────\n")?;

    let mut week_scores: Vec<u64> = Vec::new();
    let mut week_switches: u64 = 0;
    let mut week_focus_loss: u64 = 0;
    let mut week_tracked: u64 = 0;
    let mut distraction_total: u64 = 0;

    writeln!(out, "  Last 7 days:\n")?;
    writeln!(
        out,
        "  {:<12}  {:<6}  {:<8}  {:<10}  {:<10}",
        "Date", "Score", "Switches", "Focus Loss", "Tracked"
    )?;
    writeln!(out, "  {}", "─".repeat(54))?;

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
            summary.focus_score,
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
        "↑ better"
    } else if avg_score < prev_avg {
        "↓ worse"
    } else {
        "→ same"
    };

    writeln!(out, "\n  Weekly summary:\n")?;
    writeln!(out, "    Avg focus score:  {}/100  ({})", avg_score, trend)?;
    writeln!(out, "    Total switches:   {}", week_switches)?;
    writeln!(
        out,
        "    Total focus loss:  {}",
        format_duration(week_focus_loss)
    )?;
    writeln!(
        out,
        "    Total tracked:     {}",
        format_duration(week_tracked)
    )?;
    writeln!(
        out,
        "    Distraction time:  {}",
        format_duration(distraction_total)
    )?;

    writeln!(out, "\n  Top distractions this week:\n")?;
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
        writeln!(out, "    {:<20} {}", app, format_duration(*dur))?;
    }

    let streaks = store.streak_history(7)?;
    let best_streak = streaks.iter().map(|(_, s)| *s).max().unwrap_or(0);
    writeln!(
        out,
        "\n  Best focus streak: {}\n",
        format_duration(best_streak)
    )?;

    writeln!(out)?;
    Ok(())
}
