# drift

> Developer-focused context switch tracker. Quantify your focus loss.

[![CI](https://github.com/christiankbr/drift/actions/workflows/ci.yml/badge.svg)](https://github.com/christiankbr/drift/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/drift.svg)](https://crates.io/crates/drift)

**drift** is a terminal-native productivity tool that tracks your active window, detects context switches, and tells you exactly how much your attention is drifting.

## Why?

Research shows it takes **~23 minutes** to refocus after a context switch. Yet developers switch between code, Slack, Twitter, and email hundreds of times per day. drift makes that cost visible.

## Features

- **Background tracking**: Monitors your active window/app every 2 seconds
- **Smart classification**: Auto-categorizes apps as code, distraction, communication, research, or system
- **Context switch detection**: Identifies category switches and calculates focus loss
- **TUI dashboard**: Beautiful terminal UI with focus score, timeline, and stats
- **Focus mode**: Block distractions for a set time period
- **Daily reports**: Text summaries of your day
- **Data export**: JSON or CSV for custom analysis
- **100% local**: No cloud, no telemetry, no accounts. Your data stays on your machine.

## Install

```bash
cargo install drift
```

Or build from source:

```bash
git clone https://github.com/christiankbr/drift.git
cd drift
cargo install --path .
```

## Usage

```bash
# Start the background tracker
drift track

# Open the TUI dashboard
drift show

# Generate a daily report
drift report

# Start focus mode for 90 minutes
drift focus 90

# Export today's data as JSON
drift export --format json

# Check current status
drift status
```

## Configuration

drift creates a config file at `~/.config/drift/config.toml` on first run:

```toml
poll_interval_secs = 2
switching_cost_mins = 23

[categories]
code = ["vscode", "neovim", "vim", "cargo", "terminal"]
distraction = ["twitter", "reddit", "youtube", "discord"]
communication = ["slack", "teams", "zoom"]
research = ["firefox", "chrome", "safari"]
system = ["finder", "nautilus", "settings"]

focus_block = ["twitter", "reddit"]
```

## How it works

1. **Tracking**: drift polls your active window every 2 seconds using platform-native APIs (X11/Wayland on Linux, NSWorkspace on macOS, Win32 on Windows)
2. **Classification**: Each app is categorized based on your config rules
3. **Switch detection**: When the category changes, drift records a context switch and calculates the focus cost
4. **Scoring**: Your daily focus score (0-100) is calculated from distraction ratio, focus loss, and switch frequency

## Privacy

drift is **100% local**. No data ever leaves your machine. No accounts, no telemetry, no cloud sync. Your activity data is stored in a local SQLite database at `~/.local/share/drift/drift.db`.

## Platform Support

| Platform | Status | Method |
|----------|--------|--------|
| Linux (X11) | ✅ | xdotool / xprop |
| Linux (Wayland) | 🔄 | wlr-foreign-toplevel |
| macOS | ✅ | AppleScript / NSWorkspace |
| Windows | ✅ | PowerShell / Win32 |

## Contributing

Contributions welcome! See [issues](https://github.com/christiankbr/drift/issues) for ideas.

```bash
git clone https://github.com/christiankbr/drift.git
cd drift
cargo build
cargo test
```

## License

MIT © Christian Kbr