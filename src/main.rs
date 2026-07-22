use anyhow::Result;
use clap::{Parser, Subcommand};

mod classifier;
mod config;
mod focus;
mod init;
mod presets;
mod report;
mod store;
mod switch;
mod tracker;
mod tui;
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
    /// Start the background tracker daemon
    Track {
        /// Polling interval in seconds (default: 2)
        #[arg(short, long, default_value = "2")]
        interval: u64,
        /// Enable desktop alerts on distraction (rate limited: 1 per 5 min)
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
    Status,
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
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => {
            init::init_config()?;
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
                Some(Commands::Status) => {
                    print_status(&store, &config)?;
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

    println!("\n  drift, streak history (last {} days)\n", days);
    println!("  ─────────────────────────────────────\n");
    println!("  Goal: {} minutes of uninterrupted focus\n", goal);
    println!("  Date             Streak      Goal   Bar");
    println!("  {}", "─".repeat(45));

    for (date, streak) in &streaks {
        let pct = if goal > 0 {
            (*streak as f64 / goal as f64 * 100.0).min(100.0) as u64
        } else {
            0
        };
        let bar_len = (pct / 5) as usize;
        let bar: String = "█".repeat(bar_len) + &"░".repeat(20 - bar_len);
        let achieved = if *streak >= goal * 60 { "✓" } else { " " };

        println!(
            "  {:<12}  {:<10}  {:<6}  {} {}",
            date.format("%a %b %d").to_string(),
            format_duration(*streak),
            if *streak >= goal * 60 { "met" } else { "—" },
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

    println!("\n  Best streak:  {}", format_duration(best));
    println!("  Avg streak:   {}", format_duration(avg));
    println!();

    Ok(())
}

fn print_status(store: &store::Store, config: &config::Config) -> Result<()> {
    let today = chrono::Local::now().date_naive();
    let summary = store::DailySummary::for_date(store, today)?;
    let streak = store.longest_streak_for_date(today)?;

    println!("drift, status for {}\n", today.format("%Y-%m-%d"));
    println!(
        "  Tracked time:     {}",
        format_duration(summary.total_tracked)
    );
    println!("  Context switches: {}", summary.switch_count);
    println!(
        "  Focus loss:       {}",
        format_duration(summary.focus_loss)
    );
    println!("  Focus score:      {}/100", summary.focus_score);
    println!(
        "  Best streak:      {} / {}min goal",
        format_duration(streak),
        config.streak_goal_mins
    );

    println!("\n  By category:");
    for (cat, dur) in &summary.by_category {
        println!("    {:<14} {}", cat, format_duration(*dur));
    }

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
