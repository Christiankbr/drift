use crate::config::Config;
use crate::store::{DailySummary, Store};
use anyhow::Result;
use chrono::Local;
use ratatui::{
    Terminal,
    crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{BarChart, Block, Borders, Gauge, List, ListItem, Paragraph},
};
use std::io::stdout;

pub fn run_dashboard(store: &Store, config: &Config) -> Result<()> {
    let _config = config;
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut refresh = true;

    loop {
        if refresh {
            terminal.draw(|f| draw_dashboard(f, store, config))?;
            refresh = false;
        }

        if event::poll(std::time::Duration::from_millis(2000))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Char('r') => refresh = true,
                _ => {}
            }
        } else {
            // Auto-refresh every 2s
            refresh = true;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn cat_color(cat: &str) -> Color {
    match cat {
        "code" => Color::Green,
        "distraction" => Color::Red,
        "communication" => Color::Yellow,
        "research" => Color::Blue,
        "system" => Color::DarkGray,
        _ => Color::Gray,
    }
}

fn draw_dashboard(f: &mut ratatui::Frame, store: &Store, _config: &Config) {
    let today = Local::now().date_naive();
    let summary = DailySummary::for_date(store, today).unwrap_or_else(|_| DailySummary {
        date: today,
        total_tracked: 0,
        switch_count: 0,
        focus_loss: 0,
        focus_score: 0,
        by_category: vec![],
        top_switches: vec![],
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(5), // Focus score gauge
            Constraint::Length(7), // Stats row
            Constraint::Min(10),   // Category breakdown + switches
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header with daemon status
    let daemon_status = if crate::daemon::is_running() {
        Span::styled(" ● running", Style::default().fg(Color::Green))
    } else {
        Span::styled(" ○ idle", Style::default().fg(Color::DarkGray))
    };
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " drift",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        daemon_status,
        Span::raw("  "),
        Span::styled(
            summary.date.format("%A, %B %d").to_string(),
            Style::default().fg(Color::DarkGray),
        ),
    ])])
    .block(Block::default().borders(Borders::BOTTOM));
    f.render_widget(header, chunks[0]);

    // Focus score gauge
    let score = summary.focus_score.min(100);
    let score_color = if score >= 70 {
        Color::Green
    } else if score >= 40 {
        Color::Yellow
    } else {
        Color::Red
    };
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title("Focus Score"))
        .gauge_style(Style::default().fg(score_color))
        .percent(score as u16)
        .label(format!("{} / 100", score));
    f.render_widget(gauge, chunks[1]);

    // Stats row — 4 stat cards
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(chunks[2]);

    let stat_items = [
        (
            "Tracked",
            format_duration(summary.total_tracked),
            Color::Cyan,
        ),
        ("Switches", summary.switch_count.to_string(), Color::Yellow),
        (
            "Focus Loss",
            format_duration(summary.focus_loss),
            Color::Red,
        ),
        (
            "Deep Work",
            format_duration(longest_deep_work(&summary)),
            Color::Green,
        ),
    ];

    for (i, (label, value, color)) in stat_items.iter().enumerate() {
        let stat = Paragraph::new(vec![
            Line::from(Span::styled(*label, Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled(
                value.clone(),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
        ])
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(stat, stats_chunks[i]);
    }

    // Bottom: category bars + top switches
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[3]);

    // Category bars with colors
    let mut bar_data: Vec<(&str, u64)> = summary
        .by_category
        .iter()
        .map(|(c, d)| (c.as_str(), d / 60))
        .collect();
    bar_data.sort_by_key(|b| std::cmp::Reverse(b.1));

    let bar_chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Time by Category (min)"),
        )
        .data(&bar_data)
        .bar_width(10)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(bar_chart, bottom_chunks[0]);

    // Top switches with category colors
    let switch_items: Vec<ListItem> = summary
        .top_switches
        .iter()
        .map(|s| {
            ListItem::new(vec![Line::from(vec![
                Span::styled(
                    format!(" {} → {} ", s.from_category, s.to_category),
                    Style::default().fg(cat_color(&s.to_category)),
                ),
                Span::styled(
                    format!("{}min", s.cost_mins),
                    Style::default().fg(if s.cost_mins >= 20 {
                        Color::Red
                    } else {
                        Color::DarkGray
                    }),
                ),
                Span::styled(
                    format!("  {}", s.timestamp.format("%H:%M")),
                    Style::default().fg(Color::DarkGray),
                ),
            ])])
        })
        .collect();

    let switch_list = List::new(switch_items)
        .block(Block::default().borders(Borders::ALL).title("Top Switches"))
        .style(Style::default().fg(Color::White));
    f.render_widget(switch_list, bottom_chunks[1]);

    // Footer / help bar
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " [q]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" quit  "),
        Span::styled(
            " [r]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" refresh  "),
        Span::styled(
            " [Esc]",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" quit  "),
        Span::styled(" drift v0.9.0 ", Style::default().fg(Color::DarkGray)),
    ])])
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[4]);
}

fn longest_deep_work(summary: &DailySummary) -> u64 {
    summary
        .by_category
        .iter()
        .filter(|(c, _)| c == "code" || c == "research")
        .map(|(_, d)| *d)
        .sum()
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
