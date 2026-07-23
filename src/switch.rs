use crate::config::Category;

/// Calculate the focus cost of a context switch in minutes.
/// Based on research: average switching cost is ~23 minutes for deep work.
/// We adjust based on the type of switch.
pub fn switch_cost(from: Category, to: Category, base_cost_mins: u64) -> u64 {
    // Switching TO a distraction or communication is most expensive
    // Switching BETWEEN code and research is cheaper
    // Switching FROM distraction back to code has a recovery cost

    match (from, to) {
        // Anything -> Distraction: full cost (you lost focus)
        (_, Category::Distraction) => base_cost_mins,
        (_, Category::Communication) => base_cost_mins,

        // Distraction -> Code: recovery cost (need to rebuild focus)
        (Category::Distraction, Category::Code) => base_cost_mins,
        (Category::Communication, Category::Code) => (base_cost_mins * 2) / 3,

        // Code -> Research or Research -> Code: low cost (same focus domain)
        (Category::Code, Category::Research) | (Category::Research, Category::Code) => {
            base_cost_mins / 4
        }

        // Code -> Code (different apps, same category): minimal
        (Category::Code, Category::Code) => 0,

        // System or Other transitions: low cost
        (Category::System, _) | (_, Category::System) => base_cost_mins / 5,
        (Category::Other, _) | (_, Category::Other) => base_cost_mins / 3,

        // Default
        _ => base_cost_mins / 2,
    }
}

/// Detect a context switch from a sequence of activities.
/// Returns the index where a switch occurred, if any.
#[allow(dead_code)]
pub fn detect_switch_point(
    activities: &[crate::store::ActivityEntry],
) -> Vec<(usize, Category, Category)> {
    let mut switches = Vec::new();
    for i in 1..activities.len() {
        let prev = Category::from_str(&activities[i - 1].category);
        let curr = Category::from_str(&activities[i].category);
        if prev != curr {
            switches.push((i, prev, curr));
        }
    }
    switches
}

/// Calculate a focus streak: longest consecutive time in "code" category
#[allow(dead_code)]
pub fn longest_focus_streak(activities: &[crate::store::ActivityEntry]) -> u64 {
    let mut max_streak = 0u64;
    let mut current_streak = 0u64;

    for a in activities {
        if a.category == "code" || a.category == "research" {
            current_streak += a.duration_secs;
            if current_streak > max_streak {
                max_streak = current_streak;
            }
        } else {
            current_streak = 0;
        }
    }

    max_streak
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_cost_focus_to_distraction() {
        assert_eq!(switch_cost(Category::Code, Category::Distraction, 23), 23);
        assert_eq!(
            switch_cost(Category::Research, Category::Distraction, 23),
            23
        );
    }

    #[test]
    fn test_switch_cost_to_communication() {
        assert_eq!(switch_cost(Category::Code, Category::Communication, 23), 23);
    }

    #[test]
    fn test_switch_cost_distraction_to_code() {
        assert_eq!(switch_cost(Category::Distraction, Category::Code, 23), 23);
    }

    #[test]
    fn test_switch_cost_comm_to_code() {
        assert_eq!(
            switch_cost(Category::Communication, Category::Code, 23),
            (23 * 2) / 3
        );
    }

    #[test]
    fn test_switch_cost_code_to_research() {
        assert_eq!(switch_cost(Category::Code, Category::Research, 23), 23 / 4);
        assert_eq!(switch_cost(Category::Research, Category::Code, 23), 23 / 4);
    }

    #[test]
    fn test_switch_cost_same_category() {
        assert_eq!(switch_cost(Category::Code, Category::Code, 23), 0);
    }

    #[test]
    fn test_switch_cost_system_low() {
        assert_eq!(switch_cost(Category::System, Category::Code, 23), 23 / 5);
        assert_eq!(switch_cost(Category::Code, Category::System, 23), 23 / 5);
    }

    #[test]
    fn test_switch_cost_other() {
        assert_eq!(switch_cost(Category::Other, Category::Code, 23), 23 / 3);
    }
}
