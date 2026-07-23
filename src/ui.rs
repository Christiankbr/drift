#![allow(dead_code)]
use colored::*;

pub fn header(text: &str) {
    println!("\n  {}", text.cyan().bold());
    println!("  {}\n", "─".repeat(37).dimmed());
}

pub fn label(text: &str) -> String {
    text.dimmed().to_string()
}

pub fn value(text: &str) -> String {
    text.white().to_string()
}

pub fn good(text: &str) -> String {
    text.green().to_string()
}

pub fn bad(text: &str) -> String {
    text.red().to_string()
}

pub fn warn(text: &str) -> String {
    text.yellow().to_string()
}

pub fn info(text: &str) -> String {
    text.blue().to_string()
}

pub fn dim(text: &str) -> String {
    text.dimmed().to_string()
}

pub fn bold(text: &str) -> String {
    text.bold().to_string()
}

pub fn category_color(cat: &str) -> String {
    match cat {
        "code" => cat.green().bold().to_string(),
        "distraction" => cat.red().bold().to_string(),
        "communication" => cat.yellow().bold().to_string(),
        "research" => cat.blue().bold().to_string(),
        "system" => cat.dimmed().to_string(),
        _ => cat.white().to_string(),
    }
}

pub fn focus_score(score: u64) -> String {
    if score >= 70 {
        format!("{}/100", score).green().bold().to_string()
    } else if score >= 40 {
        format!("{}/100", score).yellow().bold().to_string()
    } else {
        format!("{}/100", score).red().bold().to_string()
    }
}

pub fn delta(delta: i64, positive_is_good: bool) -> String {
    let arrow = if delta > 0 {
        "↑"
    } else if delta < 0 {
        "↓"
    } else {
        "→"
    };
    let sign = if delta > 0 { "+" } else { "" };
    let text = format!("{}{} {}", sign, delta, arrow);
    if delta == 0 {
        text.dimmed().to_string()
    } else if (delta > 0) == positive_is_good {
        text.green().to_string()
    } else {
        text.red().to_string()
    }
}

pub fn bar(pct: f64, width: usize) -> String {
    let filled = (pct / 100.0 * width as f64).round() as usize;
    let bar: String = "█".repeat(filled.min(width));
    let rest: String = "░".repeat(width.saturating_sub(filled.min(width)));
    format!("{}{}", bar.green(), rest.dimmed())
}
