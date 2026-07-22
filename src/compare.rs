use crate::config::Config;
use crate::store::{DailySummary, Store};
use anyhow::Result;
use chrono::{Local, NaiveDate};

pub fn compare(
    store: &Store,
    _config: &Config,
    date1: Option<&str>,
    date2: Option<&str>,
    week: bool,
) -> Result<()> {
    if week {
        compare_weeks(store)
    } else {
        match (date1, date2) {
            (Some(d1), Some(d2)) => {
                let dt1 = NaiveDate::parse_from_str(d1, "%Y-%m-%d")?;
                let dt2 = NaiveDate::parse_from_str(d2, "%Y-%m-%d")?;
                compare_days(store, dt1, dt2)
            }
            _ => {
                anyhow::bail!(
                    "Usage: drift compare --date1=YYYY-MM-DD --date2=YYYY-MM-DD\n       drift compare --week"
                )
            }
        }
    }
}

fn compare_days(store: &Store, date1: NaiveDate, date2: NaiveDate) -> Result<()> {
    let s1 = DailySummary::for_date(store, date1)?;
    let s2 = DailySummary::for_date(store, date2)?;

    println!("\n  drift, day comparison\n");
    println!("  ─────────────────────────────────────\n");

    print_day_header(&date1, &date2);
    print_row(
        "Focus score",
        &format!("{}", s1.focus_score),
        &format!("{}", s2.focus_score),
        "higher is better",
    );
    print_row(
        "Tracked time",
        &format_duration(s1.total_tracked),
        &format_duration(s2.total_tracked),
        "more is better",
    );
    print_row(
        "Context switches",
        &format!("{}", s1.switch_count),
        &format!("{}", s2.switch_count),
        "lower is better",
    );
    print_row(
        "Focus loss",
        &format_duration(s1.focus_loss),
        &format_duration(s2.focus_loss),
        "lower is better",
    );

    // Category breakdown
    println!("\n  By category:\n");
    let mut all_cats: Vec<String> = s1
        .by_category
        .iter()
        .map(|(c, _)| c.clone())
        .chain(s2.by_category.iter().map(|(c, _)| c.clone()))
        .collect();
    all_cats.sort();
    all_cats.dedup();

    for cat in &all_cats {
        let d1 = s1
            .by_category
            .iter()
            .find(|(c, _)| c == cat)
            .map(|(_, d)| *d)
            .unwrap_or(0);
        let d2 = s2
            .by_category
            .iter()
            .find(|(c, _)| c == cat)
            .map(|(_, d)| *d)
            .unwrap_or(0);
        print_row(cat, &format_duration(d1), &format_duration(d2), "");
    }

    // Delta summary
    println!("\n  Deltas:\n");
    let score_delta = s2.focus_score as i64 - s1.focus_score as i64;
    let switch_delta = s2.switch_count as i64 - s1.switch_count as i64;
    let loss_delta = s2.focus_loss as i64 - s1.focus_loss as i64;
    let tracked_delta = s2.total_tracked as i64 - s1.total_tracked as i64;

    print_delta("Focus score", score_delta, true);
    print_delta("Context switches", switch_delta, false);
    print_delta("Focus loss (secs)", loss_delta, false);
    print_delta("Tracked time (secs)", tracked_delta, true);

    println!();
    Ok(())
}

fn compare_weeks(store: &Store) -> Result<()> {
    let today = Local::now().date_naive();

    // This week: last 7 days
    // Last week: days 7-14
    let mut this_week = WeekData::default();
    let mut last_week = WeekData::default();

    for i in 0..7 {
        let date = today - chrono::Duration::days(i as i64);
        let s = DailySummary::for_date(store, date)?;
        this_week.add(s);
    }
    for i in 7..14 {
        let date = today - chrono::Duration::days(i as i64);
        let s = DailySummary::for_date(store, date)?;
        last_week.add(s);
    }

    let this_week_start = today - chrono::Duration::days(6);
    let last_week_start = today - chrono::Duration::days(13);

    println!("\n  drift, week comparison\n");
    println!("  ─────────────────────────────────────\n");

    print_day_header(&last_week_start, &this_week_start);

    let avg_score_last = last_week
        .focus_score_sum
        .checked_div(last_week.days)
        .unwrap_or(0);
    let avg_score_this = this_week
        .focus_score_sum
        .checked_div(this_week.days)
        .unwrap_or(0);

    print_row(
        "Avg focus score",
        &format!("{}", avg_score_last),
        &format!("{}", avg_score_this),
        "higher is better",
    );
    print_row(
        "Total switches",
        &format!("{}", last_week.switch_count),
        &format!("{}", this_week.switch_count),
        "lower is better",
    );
    print_row(
        "Total focus loss",
        &format_duration(last_week.focus_loss),
        &format_duration(this_week.focus_loss),
        "lower is better",
    );
    print_row(
        "Total tracked",
        &format_duration(last_week.total_tracked),
        &format_duration(this_week.total_tracked),
        "more is better",
    );
    print_row(
        "Total distraction",
        &format_duration(last_week.distraction_time),
        &format_duration(this_week.distraction_time),
        "lower is better",
    );

    // Deltas
    println!("\n  Deltas:\n");
    let score_delta = avg_score_this as i64 - avg_score_last as i64;
    let switch_delta = this_week.switch_count as i64 - last_week.switch_count as i64;
    let loss_delta = this_week.focus_loss as i64 - last_week.focus_loss as i64;
    let tracked_delta = this_week.total_tracked as i64 - last_week.total_tracked as i64;

    print_delta("Avg focus score", score_delta, true);
    print_delta("Total switches", switch_delta, false);
    print_delta("Total focus loss (secs)", loss_delta, false);
    print_delta("Total tracked (secs)", tracked_delta, true);

    println!();
    Ok(())
}

#[derive(Default)]
struct WeekData {
    days: u64,
    focus_score_sum: u64,
    switch_count: u64,
    focus_loss: u64,
    total_tracked: u64,
    distraction_time: u64,
}

impl WeekData {
    fn add(&mut self, s: DailySummary) {
        self.days += 1;
        self.focus_score_sum += s.focus_score;
        self.switch_count += s.switch_count;
        self.focus_loss += s.focus_loss;
        self.total_tracked += s.total_tracked;
        for (cat, dur) in &s.by_category {
            if cat == "distraction" {
                self.distraction_time += dur;
            }
        }
    }
}

fn print_day_header(date1: &NaiveDate, date2: &NaiveDate) {
    println!(
        "  {:<20}  {:<14}  {:<14}",
        "Metric",
        date1.format("%a %b %d"),
        date2.format("%a %b %d")
    );
    println!("  {}", "─".repeat(52));
}

fn print_row(label: &str, val1: &str, val2: &str, _hint: &str) {
    println!("  {:<20}  {:<14}  {:<14}", label, val1, val2);
}

fn print_delta(label: &str, delta: i64, positive_is_good: bool) {
    let sign = if delta > 0 { "+" } else { "" };
    let arrow = if delta > 0 {
        "↑"
    } else if delta < 0 {
        "↓"
    } else {
        "→"
    };
    let assessment = if delta == 0 {
        "no change"
    } else if (delta > 0) == positive_is_good {
        "good"
    } else {
        "bad"
    };

    println!(
        "  {:<20}  {}{} {}  ({})",
        label, sign, delta, arrow, assessment
    );
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
