pub mod common;
pub mod formatting;
pub mod semantic;
pub mod style;
pub mod syntax;

use crate::linter::{LintContext, Problem};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration value that can be used in rule parameters
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Bool(bool),
    Int(i64),
    String(String),
    Array(Vec<ConfigValue>),
}

#[allow(dead_code)] // Some methods are part of API for future phases
impl ConfigValue {
    /// Try to get the value as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ConfigValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get the value as an integer
    pub fn as_int(&self) -> Option<i64> {
        match self {
            ConfigValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get the value as a string
    pub fn as_string(&self) -> Option<&str> {
        match self {
            ConfigValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get the value as an array
    pub fn as_array(&self) -> Option<&Vec<ConfigValue>> {
        match self {
            ConfigValue::Array(arr) => Some(arr),
            _ => None,
        }
    }
}

impl From<bool> for ConfigValue {
    fn from(value: bool) -> Self {
        ConfigValue::Bool(value)
    }
}

impl From<i64> for ConfigValue {
    fn from(value: i64) -> Self {
        ConfigValue::Int(value)
    }
}

impl From<String> for ConfigValue {
    fn from(value: String) -> Self {
        ConfigValue::String(value)
    }
}

impl From<&str> for ConfigValue {
    fn from(value: &str) -> Self {
        ConfigValue::String(value.to_string())
    }
}

impl From<Vec<ConfigValue>> for ConfigValue {
    fn from(value: Vec<ConfigValue>) -> Self {
        ConfigValue::Array(value)
    }
}

/// Configuration for a specific rule
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleConfig {
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Severity level for problems found by this rule
    pub level: crate::linter::Level,
    /// Rule-specific parameters
    pub params: HashMap<String, ConfigValue>,
}

#[allow(dead_code)] // Some methods are part of API for future phases
impl RuleConfig {
    /// Create a new rule configuration
    pub fn new(enabled: bool, level: crate::linter::Level) -> Self {
        Self {
            enabled,
            level,
            params: HashMap::new(),
        }
    }

    /// Get a parameter value as a boolean
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.params.get(key)?.as_bool()
    }

    /// Get a parameter value as an integer
    pub fn get_int(&self, key: &str) -> Option<i64> {
        self.params.get(key)?.as_int()
    }

    /// Get a parameter value as a string
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.params.get(key)?.as_string()
    }

    /// Set a parameter value
    pub fn set_param(&mut self, key: impl Into<String>, value: impl Into<ConfigValue>) {
        self.params.insert(key.into(), value.into());
    }
}

impl Default for RuleConfig {
    fn default() -> Self {
        Self::new(true, crate::linter::Level::Error)
    }
}

/// Trait that all linting rules must implement
pub trait Rule: Send + Sync {
    /// Get the unique identifier for this rule
    fn id(&self) -> &'static str;

    /// Check the given context and return any problems found
    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>>;

    /// Get the default configuration for this rule
    fn default_config(&self) -> RuleConfig;

    /// Validate that the given configuration is valid for this rule
    fn validate_config(&self, config: &RuleConfig) -> Result<()> {
        // Default implementation accepts any configuration
        let _ = config;
        Ok(())
    }

    /// Get a human-readable description of this rule
    fn description(&self) -> &'static str {
        "No description available"
    }
}

/// Registry for managing all available rules
#[derive(Default)]
pub struct RuleRegistry {
    rules: HashMap<String, Box<dyn Rule>>,
}

#[allow(dead_code)] // Some methods are part of API for future phases
impl RuleRegistry {
    /// Create a new rule registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a rule
    pub fn register(&mut self, rule: Box<dyn Rule>) {
        let id = rule.id().to_string();
        self.rules.insert(id, rule);
    }

    /// Get a rule by ID
    pub fn get(&self, id: &str) -> Option<&dyn Rule> {
        self.rules.get(id).map(|r| r.as_ref())
    }

    /// Get all registered rule IDs
    pub fn rule_ids(&self) -> Vec<&str> {
        self.rules.keys().map(|s| s.as_str()).collect()
    }

    /// Get all registered rules
    pub fn rules(&self) -> Vec<&dyn Rule> {
        self.rules.values().map(|r| r.as_ref()).collect()
    }

    /// Create a registry with default rules
    pub fn with_default_rules() -> Self {
        let mut registry = Self::new();

        // Register style rules
        registry.register(Box::new(style::LineLengthRule::new()));
        registry.register(Box::new(style::TrailingSpacesRule::new()));
        registry.register(Box::new(style::EmptyLinesRule::new()));
        registry.register(Box::new(style::IndentationRule::new()));
        registry.register(Box::new(style::NewLineAtEndOfFileRule::new()));

        // Register syntax rules
        registry.register(Box::new(syntax::KeyDuplicatesRule::new()));
        registry.register(Box::new(syntax::DocumentStructureRule::new()));
        registry.register(Box::new(syntax::AnchorsRule::new()));
        registry.register(Box::new(syntax::YamlSyntaxRule::new()));
        registry.register(Box::new(syntax::CommentsRule::new()));

        // Register formatting rules
        registry.register(Box::new(formatting::BracketsRule::new()));
        registry.register(Box::new(formatting::BracesRule::new()));
        registry.register(Box::new(formatting::ColonsRule::new()));
        registry.register(Box::new(formatting::CommasRule::new()));
        registry.register(Box::new(formatting::HyphensRule::new()));

        // Register semantic rules
        registry.register(Box::new(semantic::TruthyRule::new()));
        registry.register(Box::new(semantic::QuotedStringsRule::new()));
        registry.register(Box::new(semantic::KeyOrderingRule::new()));
        registry.register(Box::new(semantic::FloatValuesRule::new()));
        registry.register(Box::new(semantic::OctalValuesRule::new()));

        registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Level;

    #[test]
    fn test_config_value_conversions() {
        let bool_val = ConfigValue::from(true);
        assert_eq!(bool_val.as_bool(), Some(true));
        assert_eq!(bool_val.as_int(), None);

        let int_val = ConfigValue::from(42i64);
        assert_eq!(int_val.as_int(), Some(42));
        assert_eq!(int_val.as_bool(), None);

        let string_val = ConfigValue::from("test");
        assert_eq!(string_val.as_string(), Some("test"));
        assert_eq!(string_val.as_int(), None);

        let array_val = ConfigValue::from(vec![ConfigValue::from(1i64), ConfigValue::from(2i64)]);
        assert_eq!(array_val.as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_rule_config() {
        let mut config = RuleConfig::new(true, Level::Warning);

        assert!(config.enabled);
        assert_eq!(config.level, Level::Warning);
        assert!(config.params.is_empty());

        config.set_param("max", 100i64);
        config.set_param("enabled", true);

        assert_eq!(config.get_int("max"), Some(100));
        assert_eq!(config.get_bool("enabled"), Some(true));
        assert_eq!(config.get_string("nonexistent"), None);
    }

    #[test]
    fn test_rule_config_default() {
        let config = RuleConfig::default();
        assert!(config.enabled);
        assert_eq!(config.level, Level::Error);
        assert!(config.params.is_empty());
    }

    #[test]
    fn test_rule_registry() {
        let mut registry = RuleRegistry::new();
        assert!(registry.rule_ids().is_empty());

        let rule = Box::new(style::LineLengthRule::new());
        let rule_id = rule.id();
        registry.register(rule);

        assert_eq!(registry.rule_ids(), vec![rule_id]);
        assert!(registry.get(rule_id).is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_rule_registry_with_defaults() {
        let registry = RuleRegistry::with_default_rules();
        assert!(!registry.rule_ids().is_empty());
        assert!(registry.get("line-length").is_some());
    }

    #[test]
    fn test_config_value_serde() {
        let values = vec![
            ConfigValue::Bool(true),
            ConfigValue::Int(42),
            ConfigValue::String("test".to_string()),
            ConfigValue::Array(vec![ConfigValue::Int(1), ConfigValue::Int(2)]),
        ];

        for value in values {
            let serialized = serde_yaml::to_string(&value).expect("Failed to serialize");
            let deserialized: ConfigValue = serde_yaml::from_str(&serialized).expect("Failed to deserialize");
            assert_eq!(value, deserialized);
        }
    }
}
