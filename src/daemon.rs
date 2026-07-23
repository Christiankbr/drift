use crate::config::Config;
use crate::store::Store;
use crate::tracker;
use anyhow::Result;
use std::fs;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;
use std::process;

unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
    fn fork() -> i32;
    fn setsid() -> i32;
    fn dup2(oldfd: i32, newfd: i32) -> i32;
}

pub fn pid_file_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".local").join("run").join("drift");
    fs::create_dir_all(&dir).ok();
    dir.join("drift.pid")
}

pub fn is_running() -> bool {
    if let Ok(content) = fs::read_to_string(pid_file_path()) {
        if let Ok(pid) = content.trim().parse::<i32>() {
            unsafe { kill(pid, 0) == 0 }
        } else {
            false
        }
    } else {
        false
    }
}

pub fn read_pid() -> Option<i32> {
    fs::read_to_string(pid_file_path())
        .ok()
        .and_then(|s| s.trim().parse::<i32>().ok())
}

pub fn start(config: &Config, interval: u64, alert: bool) -> Result<()> {
    if is_running() {
        eprintln!(
            "  drift daemon is already running (pid: {})",
            read_pid().unwrap_or(-1)
        );
        eprintln!("  Stop it first: drift daemon stop");
        std::process::exit(1);
    }

    let pid = unsafe { fork() };
    match pid {
        -1 => {
            anyhow::bail!("fork failed");
        }
        0 => {
            // Child: become daemon
            unsafe {
                setsid();
            }

            let child_pid = process::id() as i32;
            fs::write(pid_file_path(), child_pid.to_string())?;

            let devnull = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/null")?;
            unsafe {
                dup2(devnull.as_raw_fd(), 0);
                dup2(devnull.as_raw_fd(), 1);
                dup2(devnull.as_raw_fd(), 2);
            }

            let store = Store::open(&config.db_path())?;
            tracker::run_daemon(&store, config, interval, alert)?;

            let _ = fs::remove_file(pid_file_path());
            Ok(())
        }
        p if p > 0 => {
            println!("  drift daemon started (pid: {})", p);
            println!(
                "  Polling every {}s, alerts: {}",
                interval,
                if alert { "on" } else { "off" }
            );
            println!("\n  Commands:");
            println!("    drift daemon status   — check if running");
            println!("    drift daemon stop     — stop the daemon");
            println!("    drift status          — see today's summary");
            Ok(())
        }
        _ => {
            anyhow::bail!("unexpected fork result");
        }
    }
}

pub fn stop() -> Result<()> {
    let pid_file = pid_file_path();
    match read_pid() {
        Some(pid) => {
            unsafe {
                if kill(pid, 15) == 0 {
                    println!("  drift daemon stopped (pid: {})", pid);
                    let _ = fs::remove_file(&pid_file);
                } else {
                    eprintln!(
                        "  failed to stop daemon (pid: {}) — process may have already exited",
                        pid
                    );
                    let _ = fs::remove_file(&pid_file);
                }
            }
            Ok(())
        }
        None => {
            eprintln!("  drift daemon is not running");
            eprintln!("  No PID file found at {}", pid_file.display());
            std::process::exit(1);
        }
    }
}

pub fn status() -> Result<()> {
    match read_pid() {
        Some(pid) => unsafe {
            if kill(pid, 0) == 0 {
                println!("  drift daemon is running (pid: {})", pid);
                println!("  Started tracking in the background.");
                println!("\n  drift status  — see today's summary");
                println!("  drift daemon stop  — stop the daemon");
            } else {
                println!("  drift daemon is not running");
                println!("  (stale PID file found, cleaning up)");
                let _ = fs::remove_file(pid_file_path());
            }
        },
        None => {
            println!("  drift daemon is not running");
            println!("\n  Start it: drift daemon --interval 2 --alert");
        }
    }
    Ok(())
}
