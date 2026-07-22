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
