# drift v0.4.0 — Marketing

## Show HN Post

**Title:** Show HN: drift — Terminal tool that quantifies your context switching cost

**Body:**

Hi HN, I built a CLI tool that runs in the background, tracks what window you're focused on, and tells you exactly how much your attention is drifting.

It's based on research from Gloria Mark (UC Irvine) showing it takes ~23 minutes to refocus after a context switch. Yet as developers, we switch between code, Slack, Twitter, and email hundreds of times per day — and we don't realize the cost.

drift tracks your active window, classifies it (code, distraction, communication, research), detects context switches, and gives you:

- A daily focus score (0-100)
- Context switch count and focus loss time
- Insights like "Your best focus time is 9-11am" or "You switch to Discord 12x/day"
- Day-to-day and week-to-week comparisons
- Focus mode for timed deep work sessions

It's terminal-native, cross-platform (Linux/macOS/Windows), stores data locally in SQLite, and has zero cloud dependency.

```bash
cargo install drift-tracker
drift init
drift preset development
drift track
```

Then check `drift insights` after a few days of tracking.

GitHub: https://github.com/christiankbr/drift
Landing page: https://christiankbr.github.io/drift

Happy to answer questions about the implementation (window tracking on each OS, the focus score algorithm, etc.).

---

## Reddit r/programming Post

**Title:** drift — Open source CLI tool that tracks your context switches and quantifies focus loss

**Body:**

I kept finding myself opening Discord or Twitter instead of writing code, and wondering where my day went. So I built a terminal tool that tracks this.

**What it does:** Runs in the background, monitors your active window every 2 seconds, classifies it (code/distraction/communication/research), and detects when you context-switch. Then it quantifies the cost.

Based on research showing it takes ~23 minutes to refocus after an interruption (Gloria Mark, UC Irvine).

**Key features:**
- Focus score (0-100) per day
- "drift insights" — pattern recognition: best focus hours, most distracting app, most productive weekday
- "drift compare" — compare two days or two weeks side-by-side
- Focus mode for timed deep work sessions
- TUI dashboard, daily/weekly reports
- Shell completions (bash/zsh/fish/powershell)
- Presets for development, writing, and research workflows
- 100% local, no cloud, no accounts

**Install:**
```bash
cargo install drift-tracker
```

GitHub: https://github.com/christiankbr/drift
crates.io: https://crates.io/crates/drift-tracker

It's written in Rust, uses SQLite for storage, and supports Linux (X11), macOS, and Windows.

Would love feedback on the focus score algorithm and the insights pattern recognition.

---

## Twitter/X Thread

**Tweet 1:**
I kept opening Discord instead of writing code.

So I built drift — a terminal tool that tracks your active window and tells you exactly how much your attention is drifting.

cargo install drift-tracker

🧵 Here's what it does ↓

**Tweet 2:**
drift runs in the background, monitors your active window every 2 seconds, and classifies it:
- 🟢 code
- 🔴 distraction  
- 🟡 communication
- 🔵 research

When you switch, it records the context switch and calculates the focus cost.

**Tweet 3:**
Based on research showing it takes ~23 minutes to refocus after an interruption.

Yet developers switch between code, Slack, Twitter hundreds of times per day.

drift makes that cost visible. ↓

**Tweet 4:**
My favorite command: `drift insights`

It analyzes 7 days of data and tells you:
→ "Your best focus time is 9-11am"
→ "You switch to Discord 12x/day"  
→ "Tuesdays are your most productive day"
→ "You lose ~2.5h/day to context switching"

**Tweet 5:**
`drift compare --week` gives you a side-by-side comparison of this week vs last week.

Focus score delta, switch count delta, distraction time delta. See if you're getting better or worse.

**Tweet 6:**
Also includes:
→ TUI dashboard (drift show)
→ Focus mode with interruption tracking (drift focus 90)
→ Daily & weekly reports
- Shell completions (bash/zsh/fish/powershell)
→ Presets for dev/writing/research
→ 100% local, no cloud

**Tweet 7:**
It's open source, written in Rust, cross-platform (Linux/macOS/Windows), and your data stays in a local SQLite database.

No accounts, no cloud, no tracking the tracker.

GitHub: https://github.com/christiankbr/drift
crates.io: https://crates.io/crates/drift-tracker

---

## Dev.to Article Outline

**Title:** "I built a CLI tool that tells me how much time I waste on Discord"

**Tags:** `#rust #productivity #cli #developers`

**Sections:**

### 1. The Problem
- Personal anecdote: opening Discord/Twitter reflexively
- The 23-minute refocus stat (Gloria Mark, UC Irvine)
- Why existing tools didn't work (browser extensions miss desktop apps, RescueTime is GUI-first)

### 2. Enter drift
- What it is: terminal-native, background tracking, local-first
- Install: `cargo install drift-tracker`
- 30-second setup: `drift init && drift preset development && drift track`

### 3. How it works
- Window tracking on each OS (X11/ActiveAccessibility/AppKit)
- Category classification system
- Switch detection and cost calculation
- The focus score algorithm explained

### 4. The insights that changed my habits
- Code example: `drift insights` output
- "You switch to Discord 12x/day" → I blocked Discord during mornings
- "Your best focus time is 9-11am" → I stopped scheduling meetings before 11
- "You lose ~2.5h/day to context switching" → I started batching Slack

### 5. Comparing weeks
- `drift compare --week` code example
- Before/after: how tracking changed my behavior over 4 weeks
- Week 1 vs Week 4 comparison

### 6. Technical deep dive (for Rust devs)
- Why Rust for a CLI tool (clap, ratatui, rusqlite)
- Cross-platform window tracking architecture
- The focus score formula: `100 * (1 - 0.4*distraction_ratio - 0.3*loss_ratio - 0.3*switch_penalty)`
- SQLite schema design

### 7. Roadmap
- What's coming: ML classification, team mode, webhook integrations
- Community contributions welcome

### 8. Conclusion
- You can't improve what you don't measure
- drift is the measurement tool, not the cure
- Links: GitHub, crates.io, landing page
- Call to action: try it for a week, share your insights output