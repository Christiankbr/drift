// Classifier module, currently classification logic lives in config.rs
// This module is reserved for future ML-based classification

use crate::config::Category;

/// Future: ML-based app classification
/// For now, classification is rule-based via Config::classify()
#[allow(dead_code)]
pub fn classify(app_name: &str, _window_title: &str, config: &crate::config::Config) -> Category {
    config.classify(app_name)
}

/// Suggest a category based on window title heuristics
#[allow(dead_code)]
pub fn suggest_from_title(title: &str) -> Option<Category> {
    let t = title.to_lowercase();

    if t.contains(".rs")
        || t.contains(".py")
        || t.contains(".ts")
        || t.contains(".js")
        || t.contains("cargo")
        || t.contains("npm")
        || t.contains("git")
        || t.contains("vscode")
        || t.contains("neovim")
        || t.contains("vim")
    {
        return Some(Category::Code);
    }

    if t.contains("twitter")
        || t.contains("x.com")
        || t.contains("reddit")
        || t.contains("youtube")
        || t.contains("instagram")
        || t.contains("tiktok")
    {
        return Some(Category::Distraction);
    }

    if t.contains("slack")
        || t.contains("discord")
        || t.contains("teams")
        || t.contains("zoom")
        || t.contains("meet")
        || t.contains("telegram")
    {
        return Some(Category::Communication);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_suggest_from_title_code() {
        assert_eq!(suggest_from_title("main.rs"), Some(Category::Code));
        assert_eq!(suggest_from_title("app.tsx"), Some(Category::Code));
        assert_eq!(suggest_from_title("cargo build"), Some(Category::Code));
        assert_eq!(suggest_from_title("npm install"), Some(Category::Code));
    }

    #[test]
    fn test_suggest_from_title_distraction() {
        assert_eq!(
            suggest_from_title("twitter - home"),
            Some(Category::Distraction)
        );
        assert_eq!(
            suggest_from_title("youtube - cats"),
            Some(Category::Distraction)
        );
    }

    #[test]
    fn test_suggest_from_title_communication() {
        assert_eq!(
            suggest_from_title("slack - general"),
            Some(Category::Communication)
        );
        assert_eq!(
            suggest_from_title("zoom meeting"),
            Some(Category::Communication)
        );
    }

    #[test]
    fn test_suggest_from_title_none() {
        assert_eq!(suggest_from_title("random document"), None);
    }

    #[test]
    fn test_classify_delegates_to_config() {
        let config = Config::default();
        assert_eq!(classify("code", "", &config), Category::Code);
        assert_eq!(classify("twitter", "", &config), Category::Distraction);
    }
}
