# drift

**Developer-focused context switch tracker. Quantify your focus loss.**

[![Crates.io](https://img.shields.io/crates/v/drift-tracker)](https://crates.io/crates/drift-tracker)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build](https://github.com/Christiankbr/drift/actions/workflows/ci.yml/badge.svg)](https://github.com/Christiankbr/drift/actions)

Drift tracks your active window/app, detects context switches, quantifies focus loss, and shows you exactly how much your attention is... drifting.

## Install

```sh
cargo install drift-tracker
```

Or from source:

```sh
git clone https://github.com/Christiankbr/drift
cd drift
cargo install --path .
```

## Quick Start

```sh
drift init              # Interactive setup wizard
drift daemon start      # Start background tracker
drift status            # Check today's focus
drift report            # Daily report with timeline
drift insights          # 7-day pattern analysis
```

## Commands

| Command | Description |
|---|---|
| `drift init` | Interactive setup wizard |
| `drift track` | Start foreground tracker (blocks terminal) |
| `drift daemon start/stop/status` | Background daemon with PID management |
| `drift status [--json]` | Today's focus summary |
| `drift report [-d YYYY-MM-DD]` | Daily report with categories, switches, insights |
| `drift week` | Weekly report with trend comparison |
| `drift summary [-d N]` | N-day summary with averages and trend |
| `drift timeline [-d YYYY-MM-DD]` | Hourly ASCII bar chart of the day |
| `drift avg` | Rolling averages (7d, 14d, 30d) |
| `drift insights` | 7-day pattern analysis (best hours, top distractions) |
| `drift compare --date1 X --date2 Y` | Compare two days |
| `drift compare --week` | Compare this week vs last week |
| `drift streaks [-d N]` | Streak history with progress bars |
| `drift goals` | Daily goals tracker (focus time, switches, distraction) |
| `drift log [-d DATE] [-c CAT] [-S] [-n N]` | Raw activity/switch logs |
| `drift focus N` | Start N-minute focus session |
| `drift watch` | Live active window monitor |
| `drift show` | TUI dashboard |
| `drift export -f json/csv [-d DATE]` | Export data |
| `drift config show/edit/path` | View and modify config |
| `drift ignore add/remove/list` | Manage ignored apps |
| `drift preset development/writing/research` | Apply config preset |
| `drift doctor` | Diagnostics: config, DB, daemon, tracker, display |
| `drift reset [--yes]` | Wipe tracking data |
| `drift completions bash/zsh/fish/powershell` | Shell completions |

## Config

Config lives at `~/.config/drift/config.toml`:

```toml
poll_interval_secs = 2
switching_cost_mins = 23
streak_goal_mins = 90

[categories]
code = ["code", "vim", "neovim", "terminal", "cursor", "zed"]
distraction = ["twitter", "youtube", "reddit", "discord"]
communication = ["slack", "teams", "zoom", "telegram"]
research = ["firefox", "chrome", "brave", "safari"]
system = ["finder", "nautilus", "explorer", "settings"]

ignored_apps = ["screensaver"]
focus_block = ["discord", "telegram"]
```

`.driftignore` file (one app per line, `#` for comments):

```
# Apps that shouldn't be tracked
screensaver
lockscreen
```

## Config Import/Export

```sh
drift export-config           # Print config as JSON
drift export-config -o cfg.json   # Save to file
drift import cfg.json          # Load config from JSON
```

## How It Works

Drift polls your active window every N seconds (default 2), classifies it into a category (code, distraction, communication, research, system, other), and records:

- **Activities**: What app you used, for how long, in which category
- **Switches**: When you switched between categories, with a focus cost
- **Focus sessions**: Timed deep-work blocks with switch tracking

The focus score (0-100) is calculated from distraction ratio, focus loss, and switch count. Context switch cost defaults to 23 minutes (based on research), adjusted by switch type.

## Platforms

- **Linux**: x11rb + xdotool fallback
- **macOS**: Cocoa/objc
- **Windows**: Win32 API

## License

MIT