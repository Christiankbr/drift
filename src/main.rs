use anyhow::Result;
use clap::{Parser, Subcommand};

mod classifier;
mod config;
mod focus;
mod report;
mod store;
mod switch;
mod tracker;
mod tui;

#[derive(Parser)]
#[command(
    name = "drift",
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
    },
    /// Open the TUI dashboard
    Show,
    /// Generate a daily report
    Report {
        /// Date in YYYY-MM-DD format (default: today)
        #[arg(short, long)]
        date: Option<String>,
    },
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
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    let config = config::Config::load()?;
    let store = store::Store::open(&config.db_path())?;

    match cli.command {
        Some(Commands::Track { interval }) => {
            tracker::run_daemon(&store, &config, interval)?;
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
            print_status(&store)?;
        }
        None => {
            tui::run_dashboard(&store, &config)?;
        }
    }

    Ok(())
}

fn print_status(store: &store::Store) -> Result<()> {
    let today = chrono::Local::now().date_naive();
    let summary = store::DailySummary::for_date(store, today)?;

    println!("drift, status for {}\n", today.format("%Y-%m-%d"));
    println!("  Tracked time:   {}", format_duration(summary.total_tracked));
    println!("  Context switches: {}", summary.switch_count);
    println!("  Focus loss:     {}", format_duration(summary.focus_loss));
    println!("  Focus score:    {}/100", summary.focus_score);

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