# drift-tracker

> Developer-focused context switch tracker. Quantify your focus loss.

[![CI](https://github.com/christiankbr/drift/actions/workflows/ci.yml/badge.svg)](https://github.com/christiankbr/drift/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/drift-tracker.svg)](https://crates.io/crates/drift-tracker)

**drift** is a terminal-native productivity tool that tracks your active window, detects context switches, and tells you exactly how much your attention is drifting.

## Why?

Research shows it takes **~23 minutes** to refocus after a context switch. Yet developers switch between code, Slack, Twitter, and email hundreds of times per day. drift makes that cost visible.

## Features

- **Background tracking**: Monitors your active window/app every 2 seconds
- **Context switch detection**: Classifies apps into categories (code, distraction, communication, research, system)
- **Focus score**: 0-100 score based on switch count, distraction time, and focus loss
- **TUI dashboard**: Real-time terminal UI with category breakdowns and switch history
- **Daily reports**: Detailed breakdown of your day with insights
- **Weekly reports**: 7-day summary with trend analysis and top distractions
- **Focus mode**: Start a timed focus session and track interruptions
- **Live watch mode**: See your active window and focus streak in real-time
- **Streak tracking**: Longest consecutive focus streak per day with goals
- **Config presets**: development, writing, research presets out of the box
- **Alerts**: Desktop notifications when you get distracted (rate-limited)
- **Export**: JSON or CSV export of your data
- **Cross-platform**: Linux (X11), macOS, and Windows

## Installation

### From crates.io

```bash
cargo install drift-tracker
```

### From source

```bash
git clone https://github.com/christiankbr/drift.git
cd drift
cargo build --release
# Binary is at target/release/drift-tracker
```

## Usage

```bash
# Initialize config
drift init

# Apply a preset (development, writing, research)
drift preset development

# List available presets
drift presets

# Start background tracker
drift track

# Start tracker with desktop alerts on distraction
drift track --alert

# Live watch mode
drift watch

# Open TUI dashboard
drift show

# Show current status (with streak goal)
drift status

# Show streak history with goals
drift streaks
drift streaks --days=14

# Daily report
drift report

# Weekly report with trend
drift week

# Start focus mode for 90 minutes
drift focus 90

# Export today's data as JSON
drift export --format=json

# Export specific date as CSV
drift export --format=csv --date=2026-07-22
```

## Config Presets

**development** — Software developer focused on deep work. Strict distraction blocking. 90min streak goal.

**writing** — Writer/researcher. Browsers are research, not distraction. 60min streak goal.

**research** — Researcher/academic. Lenient on browser switching. 45min streak goal.

## Configuration

drift creates a config at `~/.config/drift/config.toml`:

```toml
poll_interval_secs = 2
switching_cost_mins = 23
streak_goal_mins = 90

[categories]
code = ["code", "vim", "neovim", "idea", "terminal"]
distraction = ["twitter", "reddit", "youtube", "discord"]
communication = ["slack", "teams", "zoom"]
research = ["firefox", "chrome"]
system = ["finder", "settings"]

focus_block = ["discord", "telegram"]
```

Data is stored in SQLite at `~/.local/share/drift/drift.db`.

## License

MIT