use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod classifier;
mod compare;
mod completions;
mod config;
mod daemon;
mod focus;
mod init;
mod insights;
mod presets;
mod report;
mod store;
mod switch;
mod tracker;
mod tui;
mod ui;
mod watch;

#[derive(Parser)]
#[command(
    name = "drift-tracker",
    about = "Developer-focused context switch tracker. Quantify your focus loss.",
    version,
    long_about = "Drift tracks your active window/app, detects context switches,\nquantifies focus loss, and shows you exactly how much your\nattention is... drifting."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the foreground tracker (blocks terminal)
    Track {
        /// Polling interval in seconds (default: 2)
        #[arg(short, long, default_value = "2")]
        interval: u64,
        /// Enable desktop alerts on distraction (rate limited: 1 per 5 min)
        #[arg(short, long)]
        alert: bool,
    },
    /// Run drift as a background daemon
    Daemon {
        /// Subcommand: start, stop, or status
        action: Option<String>,
        /// Polling interval in seconds (default: 2)
        #[arg(short, long, default_value = "2")]
        interval: u64,
        /// Enable desktop alerts on distraction
        #[arg(short, long)]
        alert: bool,
    },
    /// Open the TUI dashboard
    Show,
    /// Generate a daily report
    Report {
        /// Date in YYYY-MM-DD format (default: today)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Generate a weekly report (last 7 days)
    Week,
    /// Start focus mode for N minutes
    Focus {
        /// Duration in minutes
        minutes: u32,
    },
    /// Export data as JSON or CSV
    Export {
        /// Format: json or csv
        #[arg(short, long, default_value = "json")]
        format: String,
        /// Date in YYYY-MM-DD format (default: today)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Show current status
    Status {
        /// Output as JSON (for scripting/piping)
        #[arg(long)]
        json: bool,
    },
    /// Initialize drift config
    Init,
    /// Live watch mode: show active window in real-time
    Watch,
    /// Apply a config preset (development, writing, research)
    Preset {
        /// Preset name: development, writing, or research
        name: String,
    },
    /// List available config presets
    Presets,
    /// Show streak history and goals
    Streaks {
        /// Number of days to show (default: 7)
        #[arg(short, long, default_value = "7")]
        days: u32,
    },
    /// Generate shell completion scripts
    Completions {
        /// Shell: bash, zsh, fish, or powershell
        shell: String,
    },
    /// Show insights from your tracking data (last 7 days)
    Insights,
    /// Show or edit config
    Config {
        /// Subcommand: show, edit, or path
        action: Option<String>,
        /// Key to edit (e.g. poll_interval_secs)
        key: Option<String>,
        /// New value for the key
        value: Option<String>,
    },
    /// Show raw activity logs
    Log {
        /// Date in YYYY-MM-DD format (default: today)
        #[arg(short, long)]
        date: Option<String>,
        /// Filter by category
        #[arg(short, long)]
        category: Option<String>,
        /// Number of entries to show (default: 50)
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },
    /// Reset tracking data (with confirmation)
    Reset {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Compare two days or two weeks
    Compare {
        /// First date for comparison (YYYY-MM-DD)
        #[arg(long = "date1")]
        date1: Option<String>,
        /// Second date for comparison (YYYY-MM-DD)
        #[arg(long = "date2")]
        date2: Option<String>,
        /// Compare this week vs last week
        #[arg(long)]
        week: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            init::init_wizard()?;
        }
        Some(Commands::Daemon {
            action,
            interval,
            alert,
        }) => {
            let config = config::Config::load()?;
            match action.as_deref().unwrap_or("start") {
                "start" => daemon::start(&config, interval, alert)?,
                "stop" => daemon::stop()?,
                "status" => daemon::status()?,
                other => {
                    eprintln!("  [!] Unknown daemon action: {}", other);
                    eprintln!("      Use: start, stop, or status");
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Presets) => {
            list_presets();
        }
        Some(Commands::Preset { name }) => match presets::Preset::find(&name) {
            Some(p) => p.apply()?,
            None => {
                eprintln!("  [!] Unknown preset: {}", name);
                eprintln!("      Available: development, writing, research");
                std::process::exit(1);
            }
        },
        Some(Commands::Watch) => {
            let config = config::Config::load()?;
            watch::run_watch(&config)?;
        }
        Some(Commands::Streaks { days }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            show_streaks(&store, &config, days)?;
        }
        Some(Commands::Week) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            report::weekly_report(&store, &config)?;
        }
        Some(Commands::Completions { shell }) => {
            let shell = match shell.to_lowercase().as_str() {
                "bash" => clap_complete::Shell::Bash,
                "zsh" => clap_complete::Shell::Zsh,
                "fish" => clap_complete::Shell::Fish,
                "powershell" | "pwsh" => clap_complete::Shell::PowerShell,
                _ => {
                    eprintln!("  [!] Unknown shell: {}", shell);
                    eprintln!("      Supported: bash, zsh, fish, powershell");
                    std::process::exit(1);
                }
            };
            completions::generate(shell)?;
        }
        Some(Commands::Insights) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            insights::insights(&store, &config)?;
        }
        Some(Commands::Config { action, key, value }) => {
            handle_config(action.as_deref(), key.as_deref(), value.as_deref())?;
        }
        Some(Commands::Log {
            date,
            category,
            limit,
        }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            show_log(&store, date.as_deref(), category.as_deref(), limit)?;
        }
        Some(Commands::Reset { yes }) => {
            handle_reset(yes)?;
        }
        Some(Commands::Compare { date1, date2, week }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            compare::compare(&store, &config, date1.as_deref(), date2.as_deref(), week)?;
        }
        _ => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;

            match cli.command {
                Some(Commands::Track { interval, alert }) => {
                    tracker::run_daemon(&store, &config, interval, alert)?;
                }
                Some(Commands::Show) => {
                    tui::run_dashboard(&store, &config)?;
                }
                Some(Commands::Report { date }) => {
                    report::daily_report(&store, &config, date.as_deref())?;
                }
                Some(Commands::Focus { minutes }) => {
                    focus::start_focus_mode(&store, &config, minutes)?;
                }
                Some(Commands::Export { format, date }) => {
                    report::export(&store, &config, &format, date.as_deref())?;
                }
                Some(Commands::Status { json }) => {
                    if json {
                        print_status_json(&store, &config)?;
                    } else {
                        print_status(&store, &config)?;
                    }
                }
                None => {
                    tui::run_dashboard(&store, &config)?;
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}

fn list_presets() {
    println!("\n  drift, available presets\n");
    println!("  ─────────────────────────────────────\n");
    for p in presets::Preset::all() {
        println!("  {} — {}", p.name, p.description);
        println!(
            "    poll: {}s, switch cost: {}min, streak goal: {}min\n",
            p.config.poll_interval_secs, p.config.switching_cost_mins, p.config.streak_goal_mins
        );
    }
    println!("  Usage: drift preset development\n");
}

fn show_streaks(store: &store::Store, config: &config::Config, days: u32) -> Result<()> {
    let streaks = store.streak_history(days)?;
    let goal = config.streak_goal_mins;

    println!(
        "\n  {}\n",
        format!("drift, streaks (last {} days)", days).cyan().bold()
    );
    println!("  {}", "─".repeat(37).dimmed());
    println!("  Goal: {} minutes of uninterrupted focus\n", goal);
    println!("  {:<12}  {:<10}  {:<6}  Bar", "Date", "Streak", "Goal");
    println!("  {}", "─".repeat(45).dimmed());

    for (date, streak) in &streaks {
        let pct = if goal > 0 {
            (*streak as f64 / goal as f64 * 100.0).min(100.0) as u64
        } else {
            0
        };
        let bar = ui::bar(pct as f64, 20);
        let achieved = if *streak >= goal * 60 {
            "✓".green().bold().to_string()
        } else {
            " ".to_string()
        };
        let goal_met = if *streak >= goal * 60 {
            "met".green().to_string()
        } else {
            "—".dimmed().to_string()
        };

        println!(
            "  {:<12}  {:<10}  {:<6}  {} {}",
            date.format("%a %b %d").to_string(),
            format_duration(*streak),
            goal_met,
            bar,
            achieved
        );
    }

    let best = streaks.iter().map(|(_, s)| *s).max().unwrap_or(0);
    let avg = if !streaks.is_empty() {
        streaks.iter().map(|(_, s)| *s).sum::<u64>() / streaks.len() as u64
    } else {
        0
    };

    println!("\n  Best:  {}", format_duration(best).green().bold());
    println!("  Avg:   {}", format_duration(avg).dimmed());
    println!();

    Ok(())
}

fn print_status(store: &store::Store, config: &config::Config) -> Result<()> {
    let today = chrono::Local::now().date_naive();
    let summary = store::DailySummary::for_date(store, today)?;
    let streak = store.longest_streak_for_date(today)?;

    println!("\n  {}\n", "drift, status".cyan().bold());
    println!("  {} {}", "Date".dimmed(), today.format("%Y-%m-%d"));
    println!(
        "  {}     {}",
        "Tracked".dimmed(),
        format_duration(summary.total_tracked)
    );
    println!("  {} {}", "Switches".dimmed(), summary.switch_count);
    println!(
        "  {}     {}",
        "Loss".dimmed(),
        format_duration(summary.focus_loss)
    );
    println!(
        "  {}     {}",
        "Score".dimmed(),
        ui::focus_score(summary.focus_score)
    );
    println!(
        "  {}    {} / {}min",
        "Streak".dimmed(),
        format_duration(streak),
        config.streak_goal_mins
    );

    println!("\n  {}", "By category".dimmed());
    for (cat, dur) in &summary.by_category {
        println!(
            "    {:<14} {}",
            ui::category_color(cat),
            format_duration(*dur)
        );
    }
    println!();

    Ok(())
}

fn handle_config(action: Option<&str>, key: Option<&str>, value: Option<&str>) -> Result<()> {
    let action = action.unwrap_or("show");
    match action {
        "show" => {
            let config = config::Config::load()?;
            let path = config::Config::config_path();
            println!("\n  {}\n", "drift config".cyan().bold());
            println!("  {} {}", "Path".dimmed(), path.display());
            println!("  {}", "─".repeat(37).dimmed());
            println!(
                "  {:<20} {}",
                "poll_interval_secs", config.poll_interval_secs
            );
            println!(
                "  {:<20} {}",
                "switching_cost_mins", config.switching_cost_mins
            );
            println!("  {:<20} {}", "streak_goal_mins", config.streak_goal_mins);
            println!(
                "  {:<20} {}",
                "ignored_apps",
                config.ignored_apps.join(", ")
            );
            println!("  {:<20} {} items", "focus_block", config.focus_block.len());
            for (cat, apps) in [
                ("code", &config.categories.code),
                ("distraction", &config.categories.distraction),
                ("communication", &config.categories.communication),
                ("research", &config.categories.research),
                ("system", &config.categories.system),
            ] {
                println!("  {:<20} {}", cat, apps.join(", "));
            }
            println!();
        }
        "edit" => {
            let (key, value) = match (key, value) {
                (Some(k), Some(v)) => (k, v),
                _ => {
                    eprintln!("  Usage: drift config edit <key> <value>");
                    eprintln!("  Keys: poll_interval_secs, switching_cost_mins, streak_goal_mins");
                    std::process::exit(1);
                }
            };
            let mut config = config::Config::load()?;
            match key {
                "poll_interval_secs" => config.poll_interval_secs = value.parse()?,
                "switching_cost_mins" => config.switching_cost_mins = value.parse()?,
                "streak_goal_mins" => config.streak_goal_mins = value.parse()?,
                other => {
                    eprintln!("  [!] Unknown key: {}", other);
                    eprintln!(
                        "      Keys: poll_interval_secs, switching_cost_mins, streak_goal_mins"
                    );
                    std::process::exit(1);
                }
            }
            config.save()?;
            println!("  {} {} = {}", "Updated".green().bold(), key, value);
        }
        "path" => {
            let path = config::Config::config_path();
            println!("{}", path.display());
        }
        other => {
            eprintln!("  [!] Unknown config action: {}", other);
            eprintln!("      Use: show, edit, or path");
            std::process::exit(1);
        }
    }
    Ok(())
}

fn show_log(
    store: &store::Store,
    date: Option<&str>,
    category: Option<&str>,
    limit: usize,
) -> Result<()> {
    let date = if let Some(d) = date {
        chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")?
    } else {
        chrono::Local::now().date_naive()
    };

    let activities = store.activities_for_date(date)?;

    println!(
        "\n  {}\n",
        format!("drift log, {}", date.format("%Y-%m-%d"))
            .cyan()
            .bold()
    );
    println!("  {}", "─".repeat(37).dimmed());

    let filtered: Vec<_> = if let Some(cat) = category {
        activities.iter().filter(|a| a.category == cat).collect()
    } else {
        activities.iter().collect()
    };

    if filtered.is_empty() {
        println!("  {}", "No entries found.".dimmed());
        println!();
        return Ok(());
    }

    let count = filtered.len().min(limit);
    println!(
        "  {} entries (showing {}){}\n",
        filtered.len(),
        count,
        "".dimmed()
    );

    for a in filtered.iter().take(limit) {
        println!(
            "  {}  {:<20}  {}  {}",
            a.timestamp.format("%H:%M:%S").to_string().dimmed(),
            a.app_name,
            ui::category_color(&a.category),
            format_duration(a.duration_secs).dimmed()
        );
    }
    println!();

    Ok(())
}

fn handle_reset(yes: bool) -> Result<()> {
    if !yes {
        use std::io::{self, Write};
        print!("\n  This will delete ALL tracking data. Type 'yes' to confirm: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.trim() != "yes" {
            println!("  Cancelled.");
            return Ok(());
        }
    }

    let config = config::Config::load()?;
    let db_path = config.db_path();
    std::fs::remove_file(&db_path)?;
    let store = store::Store::open(&db_path)?;
    drop(store);
    println!("  {} All tracking data reset.", "Done.".green().bold());
    println!("  DB: {}", db_path.display());
    println!();
    Ok(())
}

fn print_status_json(store: &store::Store, config: &config::Config) -> Result<()> {
    let today = chrono::Local::now().date_naive();
    let summary = store::DailySummary::for_date(store, today)?;
    let streak = store.longest_streak_for_date(today)?;

    let json = serde_json::json!({
        "date": today.format("%Y-%m-%d").to_string(),
        "tracked_secs": summary.total_tracked,
        "switches": summary.switch_count,
        "focus_loss_secs": summary.focus_loss,
        "focus_score": summary.focus_score,
        "best_streak_secs": streak,
        "streak_goal_mins": config.streak_goal_mins,
        "by_category": summary.by_category.iter().map(|(k, v)| (k.clone(), v)).collect::<std::collections::HashMap<_, _>>()
    });
    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}

fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}
