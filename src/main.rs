use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod classifier;
mod compare;
mod completions;
mod config;
mod daemon;
mod extra;
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
        /// Show switches instead of activities
        #[arg(short = 'S', long)]
        switches: bool,
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
    /// Manage ignored apps (.driftignore)
    Ignore {
        /// Action: add, remove, or list
        action: String,
        /// App name to add/remove
        app: Option<String>,
    },
    /// Show hourly timeline for a day
    Timeline {
        /// Date in YYYY-MM-DD format (default: today)
        #[arg(short, long)]
        date: Option<String>,
    },
    /// Show summary with trends (last N days)
    Summary {
        /// Number of days to show (default: 14)
        #[arg(short, long, default_value = "14")]
        days: u32,
    },
    /// Show rolling averages (7d, 14d, 30d)
    Avg,
    /// Show or set daily goals
    Goals {
        /// Action: show or set
        action: Option<String>,
        /// Goal key to set
        key: Option<String>,
        /// Goal value
        value: Option<String>,
    },
    /// Run diagnostics
    Doctor,
    /// Import config from JSON file
    Import {
        /// Path to JSON config file
        path: String,
    },
    /// Export config as JSON
    ExportConfig {
        /// Output path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// GitHub-style activity heatmap (last 12 weeks)
    Heatmap,
    /// All-time best streaks leaderboard
    StreakBest,
    /// Deep pattern analysis (weekday×hour matrix, distraction chains, recovery)
    Patterns,
    /// Generate SVG focus badge for README/website
    Badge {
        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Export tracking data as JSON bundle (for sync between machines)
    SyncExport {
        /// Output file path
        #[arg(short, long)]
        output: String,
    },
    /// Import tracking data from JSON bundle
    SyncImport {
        /// Input file path
        #[arg(short, long)]
        input: String,
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
            switches,
        }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            if switches {
                show_switch_log(&store, date.as_deref(), limit)?;
            } else {
                show_log(&store, date.as_deref(), category.as_deref(), limit)?;
            }
        }
        Some(Commands::Reset { yes }) => {
            handle_reset(yes)?;
        }
        Some(Commands::Compare { date1, date2, week }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            compare::compare(&store, &config, date1.as_deref(), date2.as_deref(), week)?;
        }
        Some(Commands::Ignore { action, app }) => {
            handle_ignore(action, app)?;
        }
        Some(Commands::Timeline { date }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::timeline(&store, &config, date.as_deref())?;
        }
        Some(Commands::Summary { days }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::summary(&store, &config, days)?;
        }
        Some(Commands::Avg) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::rolling_avg(&store, &config)?;
        }
        Some(Commands::Goals { action, key, value }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::goals(
                &store,
                &config,
                action.as_deref(),
                key.as_deref(),
                value.as_deref(),
            )?;
        }
        Some(Commands::Doctor) => {
            extra::doctor()?;
        }
        Some(Commands::Import { path }) => {
            let content = std::fs::read_to_string(&path)?;
            let config: config::Config = serde_json::from_str(&content)?;
            config.save()?;
            println!("  {} Config imported from {}", "✓".green().bold(), path);
            println!(
                "  poll: {}s, cost: {}min, goal: {}min",
                config.poll_interval_secs, config.switching_cost_mins, config.streak_goal_mins
            );
        }
        Some(Commands::ExportConfig { output }) => {
            let config = config::Config::load()?;
            let json = serde_json::to_string_pretty(&config)?;
            match output {
                Some(path) => {
                    std::fs::write(&path, &json)?;
                    println!("  {} Config exported to {}", "✓".green().bold(), path);
                }
                None => println!("{}", json),
            }
        }
        Some(Commands::Heatmap) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::heatmap(&store, &config)?;
        }
        Some(Commands::StreakBest) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::streak_best(&store, &config)?;
        }
        Some(Commands::Patterns) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::patterns(&store, &config)?;
        }
        Some(Commands::Badge { output }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::badge(&store, &config, output.as_deref())?;
        }
        Some(Commands::SyncExport { output }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::sync_export(&store, &output)?;
        }
        Some(Commands::SyncImport { input }) => {
            let config = config::Config::load()?;
            let store = store::Store::open(&config.db_path())?;
            extra::sync_import(&store, &input)?;
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

    // Daemon status
    if daemon::is_running() {
        let pid = daemon::read_pid().unwrap_or(-1);
        println!(
            "  {} {}",
            "Daemon".dimmed(),
            format!("running (pid: {})", pid).green().bold()
        );
    } else {
        println!("  {} {}", "Daemon".dimmed(), "not running".dimmed());
    }

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
                    eprintln!("  Keys: poll_interval_secs, switching_cost_mins, streak_goal_mins,");
                    eprintln!("        add_ignore <app>, remove_ignore <app>,");
                    eprintln!("        add_focus_block <app>, remove_focus_block <app>");
                    std::process::exit(1);
                }
            };
            let mut config = config::Config::load()?;
            match key {
                "poll_interval_secs" => config.poll_interval_secs = value.parse()?,
                "switching_cost_mins" => config.switching_cost_mins = value.parse()?,
                "streak_goal_mins" => config.streak_goal_mins = value.parse()?,
                "add_ignore" => {
                    if !config.ignored_apps.iter().any(|a| a == value) {
                        config.ignored_apps.push(value.to_string());
                    }
                }
                "remove_ignore" => {
                    config.ignored_apps.retain(|a| a != value);
                }
                "add_focus_block" => {
                    if !config.focus_block.iter().any(|a| a == value) {
                        config.focus_block.push(value.to_string());
                    }
                }
                "remove_focus_block" => {
                    config.focus_block.retain(|a| a != value);
                }
                other => {
                    eprintln!("  [!] Unknown key: {}", other);
                    eprintln!(
                        "      Keys: poll_interval_secs, switching_cost_mins, streak_goal_mins,"
                    );
                    eprintln!(
                        "            add_ignore, remove_ignore, add_focus_block, remove_focus_block"
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

fn show_switch_log(store: &store::Store, date: Option<&str>, limit: usize) -> Result<()> {
    let date = if let Some(d) = date {
        chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")?
    } else {
        chrono::Local::now().date_naive()
    };

    let switches = store.switches_for_date(date)?;

    println!(
        "\n  {}\n",
        format!("drift switches, {}", date.format("%Y-%m-%d"))
            .cyan()
            .bold()
    );
    println!("  {}", "─".repeat(37).dimmed());

    if switches.is_empty() {
        println!("  {}", "No switches found.".dimmed());
        println!();
        return Ok(());
    }

    let count = switches.len().min(limit);
    println!("  {} switches (showing {})\n", switches.len(), count);

    for s in switches.iter().take(limit) {
        let cost = if s.cost_mins >= 20 {
            format!("{}min", s.cost_mins).red().to_string()
        } else {
            format!("{}min", s.cost_mins).dimmed().to_string()
        };
        println!(
            "  {}  {} {} {}  ({})",
            s.timestamp.format("%H:%M:%S").to_string().dimmed(),
            ui::category_color(&s.from_category),
            "→".dimmed(),
            ui::category_color(&s.to_category),
            cost
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

fn handle_ignore(action: String, app: Option<String>) -> Result<()> {
    let mut config = config::Config::load()?;
    match action.as_str() {
        "add" => {
            let app =
                app.ok_or_else(|| anyhow::anyhow!("App name required: drift ignore add <app>"))?;
            if !config.ignored_apps.iter().any(|a| a == &app) {
                config.ignored_apps.push(app.clone());
                config.save()?;
                println!("  {} Added {} to ignore list", "+".green().bold(), app);
            } else {
                println!("  {} Already in ignore list: {}", "~".yellow(), app);
            }
        }
        "remove" => {
            let app =
                app.ok_or_else(|| anyhow::anyhow!("App name required: drift ignore remove <app>"))?;
            let before = config.ignored_apps.len();
            config.ignored_apps.retain(|a| a != &app);
            if config.ignored_apps.len() < before {
                config.save()?;
                println!("  {} Removed {} from ignore list", "-".red().bold(), app);
            } else {
                println!("  {} Not in ignore list: {}", "~".yellow(), app);
            }
        }
        "list" => {
            let ignored = config.load_driftignore();
            let combined: std::collections::HashSet<String> = config
                .ignored_apps
                .iter()
                .map(|s| s.to_lowercase())
                .chain(ignored.iter().cloned())
                .collect();
            println!("\n  {}\n", "drift, ignored apps".cyan().bold());
            if combined.is_empty() {
                println!("  {}", "No apps ignored.".dimmed());
            } else {
                for app in &combined {
                    println!("    {}", app);
                }
            }
            println!();
        }
        other => {
            eprintln!("  [!] Unknown action: {}", other);
            eprintln!("      Use: add, remove, or list");
            std::process::exit(1);
        }
    }
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
