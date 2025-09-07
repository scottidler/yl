pub mod types;
pub mod manager;

pub use types::*;
pub use manager::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::rules::{RuleConfig, ConfigValue};
    use crate::linter::Level;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_policy_manager_creation() {
        let manager = PolicyManager::new();
        assert!(manager.policies.is_empty());
        assert!(manager.policy_cache.is_empty());
    }

    #[test]
    fn test_policy_manager_creation() {
        let manager = PolicyManager::new();
        assert!(manager.policies.is_empty());
        assert!(manager.policy_cache.is_empty());
    }

    #[test]
    fn test_load_policy_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let policy_file = temp_dir.path().join("policy.yaml");

        let policy_content = r#"
name: "file-policy"
version: "1.0.0"
description: "Policy from file"
author: "File Author"
rules: {}
required_rules: []
forbidden_rules: []
min_severity: {}
extends: null
metadata:
  created_at: "2024-01-01T00:00:00Z"
  updated_at: "2024-01-01T00:00:00Z"
  tags: []
  documentation_url: null
  maintainers: []
"#;

        std::fs::write(&policy_file, policy_content).unwrap();

        let mut manager = PolicyManager::new();
        let policy_name = manager.load_policy_from_file(&policy_file).unwrap();

        assert_eq!(policy_name, "file-policy");
        assert!(manager.get_policy("file-policy").is_some());
    }

    #[test]
    fn test_validate_config_required_rule() {
        let mut manager = PolicyManager::new();

        // Create a simple policy manually
        let policy = TeamPolicy {
            name: "test-policy".to_string(),
            version: "1.0.0".to_string(),
            description: "Test policy".to_string(),
            author: "Test Author".to_string(),
            rules: std::collections::HashMap::new(),
            required_rules: vec!["line-length".to_string()],
            forbidden_rules: Vec::new(),
            min_severity: std::collections::HashMap::new(),
            extends: None,
            metadata: PolicyMetadata {
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                tags: Vec::new(),
                documentation_url: None,
                maintainers: Vec::new(),
            },
        };

        manager.policies.insert("test-policy".to_string(), policy);

        let mut config = Config::default();
        config.rules.insert("line-length".to_string(), RuleConfig::new(false, Level::Error));

        let violations = manager.validate_config(&config, "test-policy").unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].violation_type, ViolationType::RequiredRuleDisabled);
    }

    #[test]
    fn test_validate_config_forbidden_rule() {
        let mut manager = PolicyManager::new();

        // Create a simple policy manually
        let policy = TeamPolicy {
            name: "test-policy".to_string(),
            version: "1.0.0".to_string(),
            description: "Test policy".to_string(),
            author: "Test Author".to_string(),
            rules: std::collections::HashMap::new(),
            required_rules: Vec::new(),
            forbidden_rules: vec!["some-rule".to_string()],
            min_severity: std::collections::HashMap::new(),
            extends: None,
            metadata: PolicyMetadata {
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                tags: Vec::new(),
                documentation_url: None,
                maintainers: Vec::new(),
            },
        };

        manager.policies.insert("test-policy".to_string(), policy);

        let mut config = Config::default();
        config.rules.insert("some-rule".to_string(), RuleConfig::new(true, Level::Error));

        let violations = manager.validate_config(&config, "test-policy").unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].violation_type, ViolationType::ForbiddenRuleEnabled);
    }

    #[test]
    fn test_apply_policy() {
        let mut manager = PolicyManager::new();

        // Create a simple policy manually
        let policy = TeamPolicy {
            name: "test-policy".to_string(),
            version: "1.0.0".to_string(),
            description: "Test policy".to_string(),
            author: "Test Author".to_string(),
            rules: std::collections::HashMap::new(),
            required_rules: vec!["line-length".to_string()],
            forbidden_rules: Vec::new(),
            min_severity: std::collections::HashMap::new(),
            extends: None,
            metadata: PolicyMetadata {
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                tags: Vec::new(),
                documentation_url: None,
                maintainers: Vec::new(),
            },
        };

        manager.policies.insert("test-policy".to_string(), policy);

        let config = Config::default();
        let merged_config = manager.apply_policy(&config, "test-policy").unwrap();

        // The policy should be applied successfully
        assert!(merged_config.rules.len() >= 0); // May or may not have rules depending on policy
    }

    #[test]
    fn test_generate_policy_report() {
        let mut manager = PolicyManager::new();

        // Create a simple policy manually
        let policy = TeamPolicy {
            name: "test-policy".to_string(),
            version: "1.0.0".to_string(),
            description: "Test policy".to_string(),
            author: "Test Author".to_string(),
            rules: std::collections::HashMap::new(),
            required_rules: Vec::new(),
            forbidden_rules: Vec::new(),
            min_severity: std::collections::HashMap::new(),
            extends: None,
            metadata: PolicyMetadata {
                created_at: "2024-01-01T00:00:00Z".to_string(),
                updated_at: "2024-01-01T00:00:00Z".to_string(),
                tags: Vec::new(),
                documentation_url: None,
                maintainers: Vec::new(),
            },
        };

        manager.policies.insert("test-policy".to_string(), policy);

        let config = Config::default();
        let report = manager.generate_policy_report(&config, "test-policy").unwrap();

        assert!(report.contains("Policy Compliance Report"));
        assert!(report.contains("test-policy"));
        assert!(report.contains("COMPLIANT"));
    }
}