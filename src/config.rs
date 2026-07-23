use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub poll_interval_secs: u64,
    #[serde(default)]
    pub switching_cost_mins: u64,
    #[serde(default)]
    pub categories: CategoryRules,
    #[serde(default)]
    pub focus_block: Vec<String>,
    #[serde(default = "default_streak_goal")]
    pub streak_goal_mins: u64,
    #[serde(default)]
    pub ignored_apps: Vec<String>,
}

fn default_streak_goal() -> u64 {
    90
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoryRules {
    #[serde(default)]
    pub code: Vec<String>,
    #[serde(default)]
    pub distraction: Vec<String>,
    #[serde(default)]
    pub communication: Vec<String>,
    #[serde(default)]
    pub research: Vec<String>,
    #[serde(default)]
    pub system: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            poll_interval_secs: 2,
            switching_cost_mins: 23,
            categories: CategoryRules::default(),
            focus_block: vec![],
            streak_goal_mins: 90,
            ignored_apps: vec![],
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config.with_defaults())
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    pub fn config_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".config").join("drift").join("config.toml")
    }

    pub fn db_path(&self) -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let dir = home.join(".local").join("share").join("drift");
        std::fs::create_dir_all(&dir).ok();
        dir.join("drift.db")
    }

    pub fn classify(&self, app_name: &str) -> Category {
        let name = app_name.to_lowercase();
        let rules = &self.categories;

        for pattern in &rules.code {
            if name.contains(&pattern.to_lowercase()) {
                return Category::Code;
            }
        }
        for pattern in &rules.distraction {
            if name.contains(&pattern.to_lowercase()) {
                return Category::Distraction;
            }
        }
        for pattern in &rules.communication {
            if name.contains(&pattern.to_lowercase()) {
                return Category::Communication;
            }
        }
        for pattern in &rules.research {
            if name.contains(&pattern.to_lowercase()) {
                return Category::Research;
            }
        }
        for pattern in &rules.system {
            if name.contains(&pattern.to_lowercase()) {
                return Category::System;
            }
        }

        // Built-in defaults
        Self::classify_default(&name)
    }

    fn classify_default(name: &str) -> Category {
        let code_apps = [
            "code",
            "vim",
            "neovim",
            "nvim",
            "emacs",
            "idea",
            "pycharm",
            "rustrover",
            "goland",
            "webstorm",
            "sublime",
            "atom",
            "zed",
            "helix",
            "kak",
            "nano",
            "vi",
            "terminal",
            "alacritty",
            "kitty",
            "wezterm",
            "gnome-terminal",
            "konsole",
            "xterm",
            "tilix",
            "tmux",
            "screen",
            "tabby",
            "iterm",
            "windsurf",
            "cursor",
            "fleet",
            "neovide",
            "cargo",
            "rust-analyzer",
            "tsserver",
            "eslint",
            "docker",
            "kubectl",
            "ssh",
            "mosh",
        ];
        let distraction_apps = [
            "twitter",
            "x.com",
            "reddit",
            "youtube",
            "twitch",
            "netflix",
            "spotify",
            "discord",
            "telegram",
            "whatsapp",
            "instagram",
            "tiktok",
            "facebook",
            "snapchat",
            "steam",
            "games",
            "minecraft",
            "spotify",
        ];
        let comm_apps = [
            "slack",
            "teams",
            "zoom",
            "meet",
            "outlook",
            "thunderbird",
            "mail",
            "signal",
            "skype",
            "webex",
            "gmail",
            "protonmail",
        ];
        let research_apps = [
            "firefox",
            "chrome",
            "chromium",
            "brave",
            "safari",
            "edge",
            "arc",
            "zen",
            "qutebrowser",
            "epiphany",
            "sphinx",
            "devdocs",
            "zed",
        ];
        let system_apps = [
            "finder",
            "nautilus",
            "dolphin",
            "thunar",
            "ranger",
            "yazi",
            "nemo",
            "explorer",
            "settings",
            "system",
            "activity monitor",
            "task manager",
            "gnome-shell",
            "kwin",
            "i3",
            "sway",
            "rofi",
            "dmenu",
            "fuzzel",
        ];

        if code_apps.iter().any(|a| name.contains(a)) {
            Category::Code
        } else if distraction_apps.iter().any(|a| name.contains(a)) {
            Category::Distraction
        } else if comm_apps.iter().any(|a| name.contains(a)) {
            Category::Communication
        } else if research_apps.iter().any(|a| name.contains(a)) {
            Category::Research
        } else if system_apps.iter().any(|a| name.contains(a)) {
            Category::System
        } else {
            Category::Other
        }
    }

    fn with_defaults(mut self) -> Self {
        if self.poll_interval_secs == 0 {
            self.poll_interval_secs = 2;
        }
        if self.switching_cost_mins == 0 {
            self.switching_cost_mins = 23;
        }
        if self.streak_goal_mins == 0 {
            self.streak_goal_mins = 90;
        }
        self
    }

    pub fn load_driftignore(&self) -> Vec<String> {
        let path = Self::config_path()
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .join(".driftignore");
        if let Ok(content) = std::fs::read_to_string(&path) {
            content
                .lines()
                .map(|l| l.trim().to_lowercase())
                .filter(|l| !l.is_empty() && !l.starts_with('#'))
                .collect()
        } else {
            vec![]
        }
    }

    pub fn is_ignored(&self, app_name: &str) -> bool {
        let name = app_name.to_lowercase();
        self.ignored_apps
            .iter()
            .any(|a| name.contains(&a.to_lowercase()))
            || {
                let ignored = self.load_driftignore();
                ignored.iter().any(|a| name.contains(a))
            }
    }

    pub fn from_preset(preset: &crate::presets::Preset) -> Self {
        let pc = &preset.config;
        Self {
            poll_interval_secs: pc.poll_interval_secs,
            switching_cost_mins: pc.switching_cost_mins,
            categories: CategoryRules {
                code: pc.categories.code.clone(),
                distraction: pc.categories.distraction.clone(),
                communication: pc.categories.communication.clone(),
                research: pc.categories.research.clone(),
                system: pc.categories.system.clone(),
            },
            focus_block: pc.focus_block.clone(),
            streak_goal_mins: pc.streak_goal_mins,
            ignored_apps: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    Code,
    Distraction,
    Communication,
    Research,
    System,
    Other,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Code => "code",
            Category::Distraction => "distraction",
            Category::Communication => "communication",
            Category::Research => "research",
            Category::System => "system",
            Category::Other => "other",
        }
    }

    #[allow(dead_code)]
    pub fn color(&self) -> ratatui::style::Color {
        match self {
            Category::Code => ratatui::style::Color::Green,
            Category::Distraction => ratatui::style::Color::Red,
            Category::Communication => ratatui::style::Color::Yellow,
            Category::Research => ratatui::style::Color::Blue,
            Category::System => ratatui::style::Color::DarkGray,
            Category::Other => ratatui::style::Color::Gray,
        }
    }

    pub fn is_focus_breaking(&self) -> bool {
        matches!(self, Category::Distraction | Category::Communication)
    }

    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Self {
        match s {
            "code" => Category::Code,
            "distraction" => Category::Distraction,
            "communication" => Category::Communication,
            "research" => Category::Research,
            "system" => Category::System,
            _ => Category::Other,
        }
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_code_apps() {
        let config = Config::default();
        assert_eq!(config.classify("code"), Category::Code);
        assert_eq!(config.classify("neovim"), Category::Code);
        assert_eq!(config.classify("terminal"), Category::Code);
        assert_eq!(config.classify("cursor"), Category::Code);
    }

    #[test]
    fn test_classify_distraction_apps() {
        let config = Config::default();
        assert_eq!(config.classify("twitter"), Category::Distraction);
        assert_eq!(config.classify("youtube"), Category::Distraction);
        assert_eq!(config.classify("discord"), Category::Distraction);
    }

    #[test]
    fn test_classify_communication_apps() {
        let config = Config::default();
        assert_eq!(config.classify("slack"), Category::Communication);
        assert_eq!(config.classify("zoom"), Category::Communication);
    }

    #[test]
    fn test_classify_research_apps() {
        let config = Config::default();
        assert_eq!(config.classify("firefox"), Category::Research);
        assert_eq!(config.classify("chrome"), Category::Research);
    }

    #[test]
    fn test_classify_unknown_app_is_other() {
        let config = Config::default();
        assert_eq!(config.classify("someunknownapp"), Category::Other);
    }

    #[test]
    fn test_is_ignored() {
        let mut config = Config::default();
        assert!(!config.is_ignored("code"));
        config.ignored_apps.push("screensaver".to_string());
        assert!(config.is_ignored("screensaver"));
        assert!(config.is_ignored("Screensaver"));
        assert!(!config.is_ignored("code"));
    }

    #[test]
    fn test_category_is_focus_breaking() {
        assert!(Category::Distraction.is_focus_breaking());
        assert!(Category::Communication.is_focus_breaking());
        assert!(!Category::Code.is_focus_breaking());
        assert!(!Category::Research.is_focus_breaking());
        assert!(!Category::System.is_focus_breaking());
    }

    #[test]
    fn test_config_with_defaults() {
        let config = Config {
            poll_interval_secs: 0,
            switching_cost_mins: 0,
            categories: CategoryRules::default(),
            focus_block: vec![],
            streak_goal_mins: 0,
            ignored_apps: vec![],
        };
        let with_defaults = config.with_defaults();
        assert_eq!(with_defaults.poll_interval_secs, 2);
        assert_eq!(with_defaults.switching_cost_mins, 23);
        assert_eq!(with_defaults.streak_goal_mins, 90);
    }
}
