use anyhow::Result;
use std::io::{self, BufRead, Write};

use crate::config::{CategoryRules, Config};

pub fn init_wizard() -> Result<()> {
    let path = Config::config_path();

    if path.exists() {
        eprintln!("  Config already exists at {}", path.display());
        eprintln!("  Remove it first: rm {}", path.display());
        std::process::exit(1);
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    println!("\n  drift, interactive setup\n");
    println!("  ─────────────────────────────────────\n");

    // Polling interval
    let poll_interval = prompt_u64(&mut stdout, &stdin, "Polling interval (seconds)", 2)?;

    // Switching cost
    let switch_cost = prompt_u64(
        &mut stdout,
        &stdin,
        "Context switch cost (minutes, research says ~23)",
        23,
    )?;

    // Streak goal
    let streak_goal = prompt_u64(&mut stdout, &stdin, "Daily focus streak goal (minutes)", 90)?;

    // Code apps
    println!("\n  Define your app categories. Press Enter to accept defaults.\n");
    let code_apps = prompt_list(
        &mut stdout,
        &stdin,
        "Code/editor apps (comma-separated)",
        "code, vim, neovim, terminal, cursor, zed, helix, docker, ssh",
    )?;

    let distraction_apps = prompt_list(
        &mut stdout,
        &stdin,
        "Distraction apps (comma-separated)",
        "twitter, reddit, youtube, netflix, instagram, tiktok, steam, discord",
    )?;

    let comm_apps = prompt_list(
        &mut stdout,
        &stdin,
        "Communication apps (comma-separated)",
        "slack, teams, zoom, meet, outlook, mail, telegram, whatsapp, signal",
    )?;

    let research_apps = prompt_list(
        &mut stdout,
        &stdin,
        "Research/browser apps (comma-separated)",
        "firefox, chrome, chromium, brave, safari, edge, arc, zen",
    )?;

    let system_apps = prompt_list(
        &mut stdout,
        &stdin,
        "System/file manager apps (comma-separated)",
        "finder, nautilus, dolphin, explorer, settings, gnome-shell",
    )?;

    let config = Config {
        poll_interval_secs: poll_interval,
        switching_cost_mins: switch_cost,
        categories: CategoryRules {
            code: code_apps,
            distraction: distraction_apps,
            communication: comm_apps,
            research: research_apps,
            system: system_apps,
        },
        focus_block: vec![],
        streak_goal_mins: streak_goal,
        ignored_apps: vec![],
    };

    config.save()?;

    // Also create .driftignore if it doesn't exist
    let ignore_path = Config::config_path().parent().unwrap().join(".driftignore");
    if !ignore_path.exists() {
        std::fs::write(
            &ignore_path,
            "# Apps listed here (one per line) will not be tracked.\n# Example:\n# lockscreen\n# screensaver\n\n",
        )?;
    }

    let db_path = config.db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("\n  ─────────────────────────────────────\n");
    println!("  Config saved:  {}", path.display());
    println!("  DB path:       {}", db_path.display());
    println!("  .driftignore:  {}", ignore_path.display());
    println!("\n  Next: drift track   (or: drift daemon --alert)\n");

    Ok(())
}

fn prompt_u64(
    stdout: &mut io::Stdout,
    stdin: &io::Stdin,
    label: &str,
    default: u64,
) -> Result<u64> {
    write!(stdout, "  {} [{}]: ", label, default)?;
    stdout.flush()?;
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed.parse().unwrap_or(default))
    }
}

fn prompt_list(
    stdout: &mut io::Stdout,
    stdin: &io::Stdin,
    label: &str,
    default: &str,
) -> Result<Vec<String>> {
    write!(stdout, "  {} [{}]:\n  > ", label, default)?;
    stdout.flush()?;
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        Ok(default
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    } else {
        Ok(trimmed
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }
}
