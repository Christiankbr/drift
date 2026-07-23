# Changelog

## v0.9.0 — The Big One (2026-07-23)

### New Commands
- `drift timeline [-d YYYY-MM-DD]` — Hourly ASCII bar chart of the day with per-category breakdowns
- `drift summary [-d N]` — N-day summary with daily averages and trend indicators
- `drift avg` — Rolling averages across 7-day, 14-day, and 30-day windows
- `drift goals [show|set KEY VALUE]` — Daily goals tracker for focus time, switch count, and distraction minutes
- `drift doctor` — Full diagnostics: config validity, database health, daemon status, tracker platform, display output
- `drift export-config [-o FILE]` — Export config as JSON (to stdout or file)
- `drift import <PATH>` — Import config from a JSON file

### Major Upgrades
- Config import/export pipeline for sharing and backing up configurations
- Goals system with three trackable metrics: focus time, context switches, distraction minutes
- Diagnostic subsystem (`doctor`) for troubleshooting installs and runtime issues
- README fully rewritten with landing-page quality documentation

### Quality
- Comprehensive command coverage in README command table
- Config and DB path resolution centralized and testable
- All new commands follow existing color output conventions

## v0.8.0 (2026-07-23)

### New Commands
- `drift timeline` — Hourly ASCII bar chart showing category breakdowns per hour
- `drift summary` — Multi-day summary with averages and trend arrows
- `drift avg` — Rolling 7d / 14d / 30d averages
- `drift goals` — Daily goals tracker (focus time, switches, distraction)
- `drift doctor` — Diagnostics for config, DB, daemon, tracker, and display

### Major Upgrades
- Config import/export (`drift export-config`, `drift import`)
- README rewritten as a proper landing page with full command reference

### Quality
- Improved color output across report, insights, and compare
- Documentation overhaul

## v0.7.0 (2026-07-22)

### New Commands
- `drift ignore add/remove/list` — Manage `.driftignore` from the CLI
- `drift config edit KEY VALUE` — Edit any config key from the command line

### Major Upgrades
- `drift status` now shows daemon status (running/stopped with PID)
- `drift log -S` switch to show context switches instead of activities
- `drift watch` now uses color to highlight category changes

### Quality
- 21 tests added covering config, store, classifier, and switch logic
- Clippy clean

## v0.6.0 (2026-07-22)

### New Commands
- `drift config show|edit|path` — View and modify config without editing TOML
- `drift log [-d DATE] [-c CAT] [-S] [-n N]` — Raw activity and switch logs
- `drift reset [--yes]` — Wipe tracking data with confirmation

### Major Upgrades
- Color output in `report`, `insights`, and `compare`
- `--json` flag on `drift status` for scripting and piping

### Quality
- Cleaner output formatting across all commands

## v0.5.0 (2026-07-22)

### New Commands
- `drift daemon start|stop|status` — Background daemon with PID file management
- `drift init` — Interactive setup wizard

### Major Upgrades
- `.driftignore` file support (one app per line, `#` comments)
- Color output via `colored` crate

### Quality
- Daemon mode with proper PID tracking and signal handling

## v0.4.0 (2026-07-22)

### New Commands
- `drift insights` — 7-day pattern analysis (best focus hours, top distractions)
- `drift compare --date1 X --date2 Y` — Side-by-side day comparison
- `drift compare --week` — Week-over-week comparison
- `drift completions bash|zsh|fish|powershell` — Shell completion scripts

### Major Upgrades
- Landing page at `docs/index.html`
- Shell completions for all four major shells

## v0.3.0 (2026-07-22)

### New Commands
- `drift streaks [-d N]` — Streak history with progress bars
- `drift preset development|writing|research` — Apply config presets

### Major Upgrades
- Streak goals (`streak_goal_mins` in config)
- Streak info shown in `drift status`

### Quality
- Config preset system with three built-in profiles

## v0.2.0 (2026-07-21)

### New Commands
- `drift init` — Initial setup wizard (early version)
- `drift week` — Weekly report (last 7 days)
- `drift watch` — Live active window monitor
- `drift focus N` — Start N-minute focus session

### Major Upgrades
- Streak tracking
- Desktop alerts on distraction (rate-limited: 1 per 5 min)

### Quality
- Clippy warnings fixed
- CI fmt check passing

## v0.1.0 (2026-07-21)

### Initial Release

### Core Features
- Active window tracking (Linux x11rb/xdotool, macOS Cocoa, Windows Win32)
- Category classification (code, distraction, communication, research, system, other)
- Context switch detection with focus cost (default 23 min switching cost)
- Focus score calculation (0–100)
- `drift track` — Foreground tracker
- `drift status` — Today's focus summary
- `drift report [-d YYYY-MM-DD]` — Daily report
- `drift export -f json|csv` — Data export
- SQLite storage
- Config file at `~/.config/drift/config.toml`
- CI workflow on GitHub Actions

### Notes
- Originally published as `drift-tracker` on crates.io (name `drift` was taken)