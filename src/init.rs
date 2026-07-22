use anyhow::Result;

use crate::config::Config;

pub fn init_config() -> Result<()> {
    let path = Config::config_path();

    if path.exists() {
        eprintln!("  [!] Config already exists at {}", path.display());
        eprintln!(
            "      Remove it first if you want to start fresh: rm {}",
            path.display()
        );
        std::process::exit(1);
    }

    let config = Config::default();
    config.save()?;

    let db_path = config.db_path();
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    println!("\n  drift, config initialized\n");
    println!("  ─────────────────────────────────────\n");
    println!("  Created: {}", path.display());
    println!("  DB path: {}", db_path.display());
    println!("\n  Edit the config to customize your app categories.\n");

    Ok(())
}
