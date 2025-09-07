//! Machine Learning integration for YAML linting
//!
//! This module provides pattern learning capabilities to automatically
//! suggest rule configurations based on existing codebases.

pub use crate::ml_types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_pattern_learner_creation() {
        let learner = PatternLearner::new();
        assert!(learner.learned_patterns.is_empty());
    }

    #[test]
    fn test_learn_from_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let mut learner = PatternLearner::new();

        // Should not fail on empty directory
        let result = learner.learn_from_codebase(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_config() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_file = temp_dir.path().join("test.yaml");

        // Create a sample YAML file
        fs::write(&yaml_file, "key: value\n  nested:\n    - item1\n    - item2\n").unwrap();

        let mut learner = PatternLearner::new();
        let config = learner.generate_config(temp_dir.path()).unwrap();

        // Should have some default rules
        assert!(!config.rules.is_empty());
        assert!(config.rules.contains_key("trailing-spaces"));
    }

    #[test]
    fn test_suggest_rules() {
        let learner = PatternLearner::new();
        let suggestions = learner.suggest_rules();

        // Empty learner should return no suggestions
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_pattern_info() {
        use crate::rules::RuleConfig;
        use crate::linter::Level;

        let pattern_info = PatternInfo {
            confidence: 0.8,
            suggested_config: RuleConfig::new(true, Level::Error),
        };

        assert_eq!(pattern_info.confidence, 0.8);
        assert!(pattern_info.suggested_config.enabled);
    }
}
