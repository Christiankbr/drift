use crate::config::Config;
use crate::store::{DailySummary, Store};
use anyhow::Result;
use chrono::{Local, NaiveDate, Timelike};
use colored::Colorize;
use std::collections::HashMap;

pub fn timeline(store: &Store, _config: &Config, date: Option<&str>) -> Result<()> {
    let date = if let Some(d) = date {
        NaiveDate::parse_from_str(d, "%Y-%m-%d")?
    } else {
        Local::now().date_naive()
    };

    let activities = store.activities_for_date(date)?;

    println!(
        "\n  {}",
        format!("drift timeline — {}", date.format("%Y-%m-%d"))
            .cyan()
            .bold()
    );
    println!("  {}\n", "─".repeat(37).dimmed());

    if activities.is_empty() {
        println!("  {}", "No data for this day.".dimmed());
        println!();
        return Ok(());
    }

    // Group by hour
    let mut hour_buckets: HashMap<u32, Vec<(String, u64)>> = HashMap::new();
    for a in &activities {
        let hour = a.timestamp.hour();
        hour_buckets
            .entry(hour)
            .or_default()
            .push((a.category.clone(), a.duration_secs));
    }

    println!("  Hour  Category breakdown (colored bars)\n");

    let total_secs: u64 = activities.iter().map(|a| a.duration_secs).sum();
    let max_hour_secs = hour_buckets
        .values()
        .map(|v| v.iter().map(|(_, d)| *d).sum::<u64>())
        .max()
        .unwrap_or(1);

    for hour in 0..24 {
        if let Some(entries) = hour_buckets.get(&hour) {
            let hour_total: u64 = entries.iter().map(|(_, d)| *d).sum();
            let pct = (hour_total as f64 / max_hour_secs as f64 * 100.0).min(100.0);

            // Per-category bars
            let mut cat_secs: HashMap<String, u64> = HashMap::new();
            for (cat, dur) in entries {
                *cat_secs.entry(cat.clone()).or_insert(0) += dur;
            }

            let bar = crate::ui::bar(pct, 30);
            let h = format!("{:02}:00", hour).dimmed();

            // Category summary for this hour
            let cats: Vec<String> = cat_secs
                .iter()
                .map(|(c, d)| format!("{} {}", crate::ui::category_color(c), format_duration(*d)))
                .collect();

            println!("  {}  {}  {}", h, bar, cats.join("  "));
        }
    }

    println!(
        "\n  {}  {}",
        "Total".dimmed(),
        format_duration(total_secs).white().bold()
    );
    println!();

    Ok(())
}

pub fn summary(store: &Store, _config: &Config, days: u32) -> Result<()> {
    let today = Local::now().date_naive();

    println!(
        "\n  {}",
        format!("drift summary (last {} days)", days).cyan().bold()
    );
    println!("  {}\n", "─".repeat(37).dimmed());

    let mut scores: Vec<(NaiveDate, u64, u64, u64, u64)> = Vec::new();
    let mut total_focus: u64 = 0;
    let mut total_distraction: u64 = 0;
    let mut total_switches: u64 = 0;
    let mut total_tracked: u64 = 0;
    let mut total_loss: u64 = 0;

    for i in (0..days).rev() {
        let date = today - chrono::Duration::days(i as i64);
        let s = DailySummary::for_date(store, date)?;
        total_focus += s
            .by_category
            .iter()
            .filter(|(c, _)| c == "code" || *c == "research")
            .map(|(_, d)| *d)
            .sum::<u64>();
        total_distraction += s
            .by_category
            .iter()
            .filter(|(c, _)| c == "distraction")
            .map(|(_, d)| *d)
            .sum::<u64>();
        total_switches += s.switch_count;
        total_tracked += s.total_tracked;
        total_loss += s.focus_loss;
        scores.push((
            date,
            s.focus_score,
            s.switch_count,
            s.total_tracked,
            s.focus_loss,
        ));
    }

    if scores.is_empty() || total_tracked == 0 {
        println!(
            "  {}",
            "No data found. Start tracking with: drift track".dimmed()
        );
        println!();
        return Ok(());
    }

    // Day-by-day table
    println!(
        "  {:<12}  {:<6}  {:<7}  {:<10}  {:<10}",
        "Date", "Score", "Switch", "Tracked", "Loss",
    );
    println!("  {}", "─".repeat(52).dimmed());
    for (date, score, switches, tracked, loss) in &scores {
        println!(
            "  {:<12}  {:<6}  {:<7}  {:<10}  {:<10}",
            date.format("%a %b %d").to_string(),
            crate::ui::focus_score(*score),
            switches,
            format_duration(*tracked),
            format_duration(*loss),
        );
    }

    // Averages
    let avg_score = scores.iter().map(|(_, s, _, _, _)| *s).sum::<u64>() / scores.len() as u64;
    let avg_switches = total_switches / days as u64;
    let avg_tracked = total_tracked / days as u64;
    let avg_loss = total_loss / days as u64;
    let focus_pct = if total_tracked > 0 {
        (total_focus as f64 / total_tracked as f64 * 100.0) as u64
    } else {
        0
    };
    let distraction_pct = if total_tracked > 0 {
        (total_distraction as f64 / total_tracked as f64 * 100.0) as u64
    } else {
        0
    };

    println!("\n  {}\n", "Averages".dimmed());
    println!(
        "  {:<18} {}",
        "Focus score",
        crate::ui::focus_score(avg_score)
    );
    println!("  {:<18} {}", "Switches/day", avg_switches);
    println!("  {:<18} {}", "Tracked/day", format_duration(avg_tracked));
    println!(
        "  {:<18} {}",
        "Loss/day",
        format_duration(avg_loss).yellow()
    );
    println!("  {:<18} {}%", "Focus time", focus_pct.to_string().green());
    println!(
        "  {:<18} {}%",
        "Distraction",
        distraction_pct.to_string().red()
    );

    // Trend: compare first half vs second half
    if scores.len() >= 4 {
        let mid = scores.len() / 2;
        let first_half: u64 =
            scores[..mid].iter().map(|(_, s, _, _, _)| *s).sum::<u64>() / mid as u64;
        let second_half: u64 = scores[mid..].iter().map(|(_, s, _, _, _)| *s).sum::<u64>()
            / (scores.len() - mid) as u64;
        let delta = second_half as i64 - first_half as i64;
        let trend = if delta > 0 {
            format!("{}{} improving", "↑".green(), format!("+{}", delta).green())
        } else if delta < 0 {
            format!("{}{} declining", "↓".red(), delta.to_string().red())
        } else {
            "→ stable".dimmed().to_string()
        };
        println!("\n  {:<18} {}", "Trend", trend);
    }

    println!();
    Ok(())
}

pub fn rolling_avg(store: &Store, _config: &Config) -> Result<()> {
    let today = Local::now().date_naive();

    println!("\n  {}", "drift avg — rolling averages".cyan().bold());
    println!("  {}\n", "─".repeat(37).dimmed());

    for &window in &[7, 14, 30] {
        let mut scores = Vec::new();
        let mut switches = 0u64;
        let mut tracked = 0u64;
        let mut loss = 0u64;
        let mut distraction = 0u64;

        for i in 0..window {
            let date = today - chrono::Duration::days(i as i64);
            let s = DailySummary::for_date(store, date)?;
            scores.push(s.focus_score);
            switches += s.switch_count;
            tracked += s.total_tracked;
            loss += s.focus_loss;
            distraction += s
                .by_category
                .iter()
                .filter(|(c, _)| c == "distraction")
                .map(|(_, d)| *d)
                .sum::<u64>();
        }

        let avg_score = if !scores.is_empty() {
            scores.iter().sum::<u64>() / scores.len() as u64
        } else {
            0
        };
        let avg_tracked = tracked / window as u64;
        let avg_switches = switches / window as u64;
        let avg_loss = loss / window as u64;
        let avg_distraction = distraction / window as u64;

        let label = format!("{}d", window).cyan().bold();
        println!(
            "  {}  score {}  switches {}  tracked {}  loss {}  distract {}",
            label,
            crate::ui::focus_score(avg_score),
            avg_switches,
            format_duration(avg_tracked).dimmed(),
            format_duration(avg_loss).yellow(),
            format_duration(avg_distraction).red(),
        );
    }
    println!();
    Ok(())
}

pub fn goals(
    store: &Store,
    config: &Config,
    action: Option<&str>,
    key: Option<&str>,
    value: Option<&str>,
) -> Result<()> {
    let action = action.unwrap_or("show");
    match action {
        "show" => {
            let today = Local::now().date_naive();
            let summary = DailySummary::for_date(store, today)?;
            let focus_secs: u64 = summary
                .by_category
                .iter()
                .filter(|(c, _)| c == "code" || c == "research")
                .map(|(_, d)| *d)
                .sum();
            let distraction_secs: u64 = summary
                .by_category
                .iter()
                .filter(|(c, _)| c == "distraction")
                .map(|(_, d)| *d)
                .sum();

            println!("\n  {}", "drift goals — today".cyan().bold());
            println!("  {}\n", "─".repeat(37).dimmed());

            // Focus time goal
            let focus_goal_secs = config.streak_goal_mins * 60;
            let focus_pct = if focus_goal_secs > 0 {
                (focus_secs as f64 / focus_goal_secs as f64 * 100.0).min(100.0)
            } else {
                0.0
            };
            println!(
                "  {:<20} {} / {}",
                "Focus time".dimmed(),
                format_duration(focus_secs).green(),
                format_duration(focus_goal_secs).dimmed()
            );
            println!("    {}", crate::ui::bar(focus_pct, 30));

            // Switches goal (suggested: <30)
            let switch_goal = 30u64;
            let switch_pct = if summary.switch_count >= switch_goal {
                100.0
            } else {
                summary.switch_count as f64 / switch_goal as f64 * 100.0
            };
            let switch_status = if summary.switch_count <= switch_goal {
                format!("{} / {}", summary.switch_count, switch_goal).green()
            } else {
                format!("{} / {}", summary.switch_count, switch_goal).red()
            };
            println!("  {:<20} {}", "Switches".dimmed(), switch_status);
            println!("    {}", crate::ui::bar(switch_pct, 30));

            // Distraction goal (suggested: <30min)
            let distract_goal_secs = 1800u64;
            let distract_pct = if distraction_secs >= distract_goal_secs {
                100.0
            } else {
                distraction_secs as f64 / distract_goal_secs as f64 * 100.0
            };
            let distract_status = if distraction_secs <= distract_goal_secs {
                format!(
                    "{} / {}",
                    format_duration(distraction_secs),
                    format_duration(distract_goal_secs)
                )
                .green()
            } else {
                format!(
                    "{} / {}",
                    format_duration(distraction_secs),
                    format_duration(distract_goal_secs)
                )
                .red()
            };
            println!("  {:<20} {}", "Distraction".dimmed(), distract_status);
            println!("    {}", crate::ui::bar(distract_pct, 30));

            let goals_met = focus_secs >= focus_goal_secs
                && summary.switch_count <= switch_goal
                && distraction_secs <= distract_goal_secs;
            if goals_met {
                println!("\n  {} All goals met today!", "✓".green().bold());
            }
            println!();
        }
        "set" => {
            let (key, value) = match (key, value) {
                (Some(k), Some(v)) => (k, v),
                _ => {
                    eprintln!("  Usage: drift goals set <key> <value>");
                    eprintln!("  Keys: streak_goal_mins");
                    std::process::exit(1);
                }
            };
            let mut cfg = Config::load()?;
            match key {
                "streak_goal_mins" => cfg.streak_goal_mins = value.parse()?,
                other => {
                    eprintln!("  [!] Unknown goal: {}", other);
                    std::process::exit(1);
                }
            }
            cfg.save()?;
            println!("  {} {} = {}", "Updated".green().bold(), key, value);
        }
        other => {
            eprintln!("  [!] Unknown action: {}", other);
            eprintln!("      Use: show or set");
            std::process::exit(1);
        }
    }
    Ok(())
}

pub fn doctor() -> Result<()> {
    println!("\n  {}", "drift doctor — diagnostics".cyan().bold());
    println!("  {}\n", "─".repeat(37).dimmed());

    // Check 1: config file
    let config_path = Config::config_path();
    if config_path.exists() {
        println!("  {} Config found: {}", "✓".green(), config_path.display());
    } else {
        println!(
            "  {} Config missing. Run: {}",
            "✗".red(),
            "drift init".cyan()
        );
    }

    // Check 2: config loads
    match Config::load() {
        Ok(cfg) => {
            println!(
                "  {} Config loads (poll={}s, cost={}min)",
                "✓".green(),
                cfg.poll_interval_secs,
                cfg.switching_cost_mins
            );
        }
        Err(e) => {
            println!("  {} Config load failed: {}", "✗".red(), e);
        }
    }

    // Check 3: DB
    let config = Config::load()?;
    let db_path = config.db_path();
    if db_path.exists() {
        match Store::open(&db_path) {
            Ok(store) => {
                let today = Local::now().date_naive();
                let acts = store.activities_for_date(today).unwrap_or_default();
                let sws = store.switches_for_date(today).unwrap_or_default();
                println!(
                    "  {} DB accessible: {} activities, {} switches today",
                    "✓".green(),
                    acts.len(),
                    sws.len()
                );
            }
            Err(e) => {
                println!("  {} DB open failed: {}", "✗".red(), e);
            }
        }
    } else {
        println!(
            "  {} DB not found (will be created on first track)",
            "~".yellow()
        );
    }

    // Check 4: daemon
    if crate::daemon::is_running() {
        let pid = crate::daemon::read_pid().unwrap_or(-1);
        println!("  {} Daemon running (pid: {})", "✓".green(), pid);
    } else {
        println!("  {} Daemon not running", "~".yellow());
    }

    // Check 5: tracker
    match crate::tracker::create_tracker() {
        Ok(_) => println!("  {} Window tracker available", "✓".green()),
        Err(e) => println!("  {} Window tracker failed: {}", "✗".red(), e),
    }

    // Check 6: .driftignore
    let ignore_path = Config::config_path().parent().unwrap().join(".driftignore");
    if ignore_path.exists() {
        let content = std::fs::read_to_string(&ignore_path)?;
        let count = content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .count();
        println!("  {} .driftignore: {} entries", "✓".green(), count);
    } else {
        println!("  {} .driftignore not found (optional)", "~".yellow());
    }

    // Check 7: display
    #[cfg(target_os = "linux")]
    {
        let has_display =
            std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
        if has_display {
            println!("  {} Display detected", "✓".green());
        } else {
            println!(
                "  {} No DISPLAY/WAYLAND_DISPLAY — tracker may fail",
                "✗".red()
            );
        }
    }

    println!();
    Ok(())
}

fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{}h{}m", h, m)
    } else if m > 0 {
        format!("{}m", m)
    } else {
        format!("{}s", secs)
    }
}
