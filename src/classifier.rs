// Classifier module, currently classification logic lives in config.rs
// This module is reserved for future ML-based classification

use crate::config::Category;

/// Future: ML-based app classification
/// For now, classification is rule-based via Config::classify()
pub fn classify(app_name: &str, _window_title: &str, config: &crate::config::Config) -> Category {
    config.classify(app_name)
}

/// Suggest a category based on window title heuristics
pub fn suggest_from_title(title: &str) -> Option<Category> {
    let t = title.to_lowercase();
    
    // Code-related titles
    if t.contains(".rs") || t.contains(".py") || t.contains(".ts") || t.contains(".js")
        || t.contains("cargo") || t.contains("npm") || t.contains("git")
        || t.contains("vscode") || t.contains("neovim") || t.contains("vim")
    {
        return Some(Category::Code);
    }
    
    // Social media
    if t.contains("twitter") || t.contains("x.com") || t.contains("reddit")
        || t.contains("youtube") || t.contains("instagram") || t.contains("tiktok")
    {
        return Some(Category::Distraction);
    }
    
    // Communication
    if t.contains("slack") || t.contains("discord") || t.contains("teams")
        || t.contains("zoom") || t.contains("meet") || t.contains("telegram")
    {
        return Some(Category::Communication);
    }
    
    None
}