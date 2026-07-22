use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: &'static str,
    pub description: &'static str,
    pub config: PresetConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    pub poll_interval_secs: u64,
    pub switching_cost_mins: u64,
    pub categories: PresetCategories,
    pub focus_block: Vec<String>,
    pub streak_goal_mins: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetCategories {
    pub code: Vec<String>,
    pub distraction: Vec<String>,
    pub communication: Vec<String>,
    pub research: Vec<String>,
    pub system: Vec<String>,
}

impl Preset {
    pub fn development() -> Self {
        Self {
            name: "development",
            description: "Software developer focused on deep work. Strict distraction blocking.",
            config: PresetConfig {
                poll_interval_secs: 2,
                switching_cost_mins: 23,
                streak_goal_mins: 90,
                categories: PresetCategories {
                    code: vec![
                        "code".into(),
                        "vim".into(),
                        "neovim".into(),
                        "nvim".into(),
                        "emacs".into(),
                        "idea".into(),
                        "pycharm".into(),
                        "rustrover".into(),
                        "goland".into(),
                        "webstorm".into(),
                        "sublime".into(),
                        "zed".into(),
                        "helix".into(),
                        "terminal".into(),
                        "alacritty".into(),
                        "kitty".into(),
                        "wezterm".into(),
                        "gnome-terminal".into(),
                        "konsole".into(),
                        "tmux".into(),
                        "cursor".into(),
                        "windsurf".into(),
                        "docker".into(),
                        "kubectl".into(),
                        "ssh".into(),
                    ],
                    distraction: vec![
                        "twitter".into(),
                        "x.com".into(),
                        "reddit".into(),
                        "youtube".into(),
                        "twitch".into(),
                        "netflix".into(),
                        "instagram".into(),
                        "tiktok".into(),
                        "facebook".into(),
                        "snapchat".into(),
                        "steam".into(),
                    ],
                    communication: vec![
                        "slack".into(),
                        "teams".into(),
                        "zoom".into(),
                        "meet".into(),
                        "outlook".into(),
                        "thunderbird".into(),
                        "mail".into(),
                        "signal".into(),
                        "skype".into(),
                        "discord".into(),
                        "telegram".into(),
                        "whatsapp".into(),
                    ],
                    research: vec![
                        "firefox".into(),
                        "chrome".into(),
                        "chromium".into(),
                        "brave".into(),
                        "safari".into(),
                        "edge".into(),
                        "arc".into(),
                        "zen".into(),
                    ],
                    system: vec![
                        "finder".into(),
                        "nautilus".into(),
                        "dolphin".into(),
                        "thunar".into(),
                        "ranger".into(),
                        "yazi".into(),
                        "explorer".into(),
                        "settings".into(),
                        "gnome-shell".into(),
                        "kwin".into(),
                    ],
                },
                focus_block: vec!["discord".into(), "telegram".into(), "whatsapp".into()],
            },
        }
    }

    pub fn writing() -> Self {
        Self {
            name: "writing",
            description: "Writer/researcher. Browsers are research, not distraction.",
            config: PresetConfig {
                poll_interval_secs: 3,
                switching_cost_mins: 15,
                streak_goal_mins: 60,
                categories: PresetCategories {
                    code: vec![
                        "vim".into(),
                        "neovim".into(),
                        "emacs".into(),
                        "obsidian".into(),
                        "typora".into(),
                        "ia writer".into(),
                        "ulysses".into(),
                        "scrivener".into(),
                        "marktext".into(),
                        "zettlr".into(),
                    ],
                    distraction: vec![
                        "twitter".into(),
                        "x.com".into(),
                        "reddit".into(),
                        "youtube".into(),
                        "twitch".into(),
                        "netflix".into(),
                        "instagram".into(),
                        "tiktok".into(),
                        "facebook".into(),
                        "steam".into(),
                    ],
                    communication: vec![
                        "slack".into(),
                        "teams".into(),
                        "zoom".into(),
                        "meet".into(),
                        "outlook".into(),
                        "mail".into(),
                        "discord".into(),
                        "telegram".into(),
                        "whatsapp".into(),
                    ],
                    research: vec![
                        "firefox".into(),
                        "chrome".into(),
                        "chromium".into(),
                        "brave".into(),
                        "safari".into(),
                        "edge".into(),
                        "arc".into(),
                        "zen".into(),
                        "spotlight".into(),
                        "alfred".into(),
                    ],
                    system: vec![
                        "finder".into(),
                        "nautilus".into(),
                        "explorer".into(),
                        "settings".into(),
                        "gnome-shell".into(),
                    ],
                },
                focus_block: vec!["discord".into(), "telegram".into()],
            },
        }
    }

    pub fn research() -> Self {
        Self {
            name: "research",
            description: "Researcher/academic. Reading papers and writing. Lenient on browser.",
            config: PresetConfig {
                poll_interval_secs: 5,
                switching_cost_mins: 10,
                streak_goal_mins: 45,
                categories: PresetCategories {
                    code: vec![
                        "vim".into(),
                        "neovim".into(),
                        "emacs".into(),
                        "obsidian".into(),
                        "zotero".into(),
                        "jabref".into(),
                        "latex".into(),
                        "tex".into(),
                        "overleaf".into(),
                    ],
                    distraction: vec![
                        "twitter".into(),
                        "x.com".into(),
                        "reddit".into(),
                        "youtube".into(),
                        "netflix".into(),
                        "instagram".into(),
                        "tiktok".into(),
                        "facebook".into(),
                        "steam".into(),
                    ],
                    communication: vec![
                        "slack".into(),
                        "teams".into(),
                        "zoom".into(),
                        "meet".into(),
                        "outlook".into(),
                        "mail".into(),
                        "discord".into(),
                        "telegram".into(),
                        "whatsapp".into(),
                    ],
                    research: vec![
                        "firefox".into(),
                        "chrome".into(),
                        "chromium".into(),
                        "brave".into(),
                        "safari".into(),
                        "edge".into(),
                        "arc".into(),
                        "zen".into(),
                        "preview".into(),
                        "evince".into(),
                        "okular".into(),
                    ],
                    system: vec![
                        "finder".into(),
                        "nautilus".into(),
                        "explorer".into(),
                        "settings".into(),
                    ],
                },
                focus_block: vec![],
            },
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::development(), Self::writing(), Self::research()]
    }

    pub fn find(name: &str) -> Option<Self> {
        Self::all().into_iter().find(|p| p.name == name)
    }

    pub fn apply(&self) -> Result<()> {
        let config = crate::config::Config::from_preset(self);
        config.save()?;
        println!("\n  drift, preset applied: {}\n", self.name);
        println!("  ─────────────────────────────────────\n");
        println!("  {}", self.description);
        println!("  Polling:      {}s", self.config.poll_interval_secs);
        println!("  Switch cost:  {}min", self.config.switching_cost_mins);
        println!("  Streak goal:  {}min", self.config.streak_goal_mins);
        println!("\n  Config saved to ~/.config/drift/config.toml\n");
        Ok(())
    }
}
