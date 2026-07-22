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

    loop {
        let today = Local::now().date_naive();
        let summary = DailySummary::for_date(store, today)?;

        terminal.draw(|f| draw_dashboard(f, &summary))?;

        if event::poll(std::time::Duration::from_millis(1000))?
            && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    break;
                }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn draw_dashboard(f: &mut ratatui::Frame, summary: &DailySummary) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(5), // Focus score gauge
            Constraint::Length(8), // Stats row
            Constraint::Min(10),   // Category breakdown + switches
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            " drift ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
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

    // Stats row
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
            format_duration(longest_deep_work(summary)),
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
        ])
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(stat, stats_chunks[i]);
    }

    // Bottom section: category bars + top switches
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[3]);

    // Category breakdown as bar chart
    let mut bar_data: Vec<(&str, u64)> = summary
        .by_category
        .iter()
        .map(|(c, d)| (c.as_str(), d / 60)) // convert to minutes
        .collect();
    bar_data.sort_by_key(|b| std::cmp::Reverse(b.1));

    let bar_chart = BarChart::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Time by Category (minutes)"),
        )
        .data(&bar_data)
        .bar_width(8)
        .bar_style(Style::default().fg(Color::Cyan))
        .value_style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(bar_chart, bottom_chunks[0]);

    // Top switches list
    let switch_items: Vec<ListItem> = summary
        .top_switches
        .iter()
        .map(|s| {
            ListItem::new(vec![Line::from(vec![
                Span::styled(
                    format!(" {} -> {} ", s.from_category, s.to_category),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(format!(" {}min", s.cost_mins)),
                Span::styled(
                    format!("  {}", s.timestamp.format("%H:%M")),
                    Style::default().fg(Color::DarkGray),
                ),
            ])])
        })
        .collect();

    let switch_list = List::new(switch_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Top Context Switches"),
        )
        .style(Style::default().fg(Color::White));
    f.render_widget(switch_list, bottom_chunks[1]);

    // Footer
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
        Span::styled(" drift v0.1.0 ", Style::default().fg(Color::DarkGray)),
    ])])
    .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[4]);
}

fn longest_deep_work(summary: &DailySummary) -> u64 {
    // Approximate: code + research time
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
        format!("{}h {}m", h, m)
    } else if m > 0 {
        format!("{}m", m)
    } else {
        format!("{}s", secs)
    }
}
