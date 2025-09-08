pub mod inline;

use crate::rules::{RuleConfig, RuleRegistry};
use eyre::{Context, ContextCompat, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub use inline::InlineConfigManager;

/// Main configuration for the YAML linter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Base configuration to extend from
    pub extends: Option<String>,
    /// Rule-specific configurations
    pub rules: HashMap<String, RuleConfig>,
    /// File patterns to ignore
    pub ignore: Vec<String>,
    /// File patterns that should be treated as YAML files
    #[serde(rename = "yaml-files")]
    pub yaml_files: Vec<String>,
}

impl Config {
    /// Load configuration from a file path
    pub fn load(config_path: Option<&PathBuf>) -> Result<Self> {
        let config_file = match config_path {
            Some(path) => path.clone(),
            None => Self::default_config_path()?,
        };

        if config_file.exists() {
            let content = fs::read_to_string(&config_file).with_context(|| {
                format!("Failed to read config file: {}", config_file.display())
            })?;

            let mut config: Config = serde_yaml::from_str(&content).with_context(|| {
                format!("Failed to parse config file: {}", config_file.display())
            })?;

            // Handle extends
            if let Some(base_name) = &config.extends {
                let base_config = Self::load_base_config(base_name, &config_file)?;
                config = config.merge_with_base(base_config)?;
            }

            Ok(config)
        } else {
            // Return default config if file doesn't exist
            Ok(Self::default())
        }
    }

    /// Load a base configuration by name
    fn load_base_config(base_name: &str, current_config_path: &Path) -> Result<Self> {
        // First try built-in configurations
        match base_name {
            "default" => Ok(Self::default()),
            "strict" => Ok(Self::strict()),
            "relaxed" => Ok(Self::relaxed()),
            _ => {
                // Try to load as a file path relative to current config
                let base_path = if base_name.starts_with('/') {
                    PathBuf::from(base_name)
                } else {
                    current_config_path
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(base_name)
                };

                if base_path.exists() {
                    Self::load(Some(&base_path))
                } else {
                    Err(eyre::eyre!("Base configuration '{}' not found", base_name))
                }
            }
        }
    }

    /// Merge this configuration with a base configuration
    fn merge_with_base(mut self, base: Self) -> Result<Self> {
        // Start with base rules
        let mut merged_rules = base.rules;

        // Override with current rules
        for (rule_id, rule_config) in self.rules {
            merged_rules.insert(rule_id, rule_config);
        }

        self.rules = merged_rules;

        // Use current ignore patterns if specified, otherwise use base
        if self.ignore.is_empty() {
            self.ignore = base.ignore;
        }

        // Use current yaml-files patterns if specified, otherwise use base
        if self.yaml_files.is_empty() {
            self.yaml_files = base.yaml_files;
        }

        Ok(self)
    }

    /// Get the default configuration file path
    fn default_config_path() -> Result<PathBuf> {
        // Look for config files in order of preference
        let candidates = vec![
            PathBuf::from(".yl.yaml"),
            PathBuf::from(".yl.yml"),
            PathBuf::from("yl.yaml"),
            PathBuf::from("yl.yml"),
        ];

        for candidate in candidates {
            if candidate.exists() {
                return Ok(candidate);
            }
        }

        // If no config file found, return default location
        let config_dir = dirs::config_local_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .context("Could not determine config directory")?;

        Ok(config_dir.join("yl").join("config.yaml"))
    }

    /// Get the effective configuration for a rule
    pub fn get_rule_config(&self, rule_id: &str, registry: &RuleRegistry) -> RuleConfig {
        // First try to get from explicit configuration
        if let Some(config) = self.rules.get(rule_id) {
            return config.clone();
        }

        // Fall back to rule's default configuration
        if let Some(rule) = registry.get(rule_id) {
            return rule.default_config();
        }

        // Last resort: generic default
        RuleConfig::default()
    }

    /// Check if a file should be ignored based on ignore patterns
    pub fn is_file_ignored(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        for pattern in &self.ignore {
            // Simple glob-like matching (could be enhanced with proper glob library)
            if pattern.contains('*') {
                let pattern_regex = pattern.replace('*', ".*");
                if regex::Regex::new(&pattern_regex)
                    .map(|re| re.is_match(&path_str))
                    .unwrap_or(false)
                {
                    return true;
                }
            } else if path_str.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Check if a file should be treated as a YAML file
    pub fn is_yaml_file(&self, file_path: &Path) -> bool {
        let path_str = file_path.to_string_lossy();

        for pattern in &self.yaml_files {
            if pattern.contains('*') {
                let pattern_regex = pattern.replace('*', ".*");
                if regex::Regex::new(&pattern_regex)
                    .map(|re| re.is_match(&path_str))
                    .unwrap_or(false)
                {
                    return true;
                }
            } else if path_str.ends_with(pattern) {
                return true;
            }
        }

        false
    }

    /// Create a strict configuration preset
    pub fn strict() -> Self {
        let mut config = Self::default();

        // Make all rules errors
        for (_, rule_config) in config.rules.iter_mut() {
            rule_config.level = crate::linter::Level::Error;
        }

        config
    }

    /// Create a relaxed configuration preset
    pub fn relaxed() -> Self {
        let mut config = Self::default();

        // Make most rules warnings
        for (_, rule_config) in config.rules.iter_mut() {
            rule_config.level = crate::linter::Level::Warning;
        }

        config
    }
}

impl Default for Config {
    fn default() -> Self {
        let registry = RuleRegistry::with_default_rules();
        let mut rules = HashMap::new();

        // Add default configurations for all built-in rules
        for rule in registry.rules() {
            rules.insert(rule.id().to_string(), rule.default_config());
        }

        Self {
            extends: None,
            rules,
            ignore: vec![
                "*.generated.yaml".to_string(),
                "*.generated.yml".to_string(),
                ".git/**".to_string(),
                "node_modules/**".to_string(),
            ],
            yaml_files: vec![
                "*.yaml".to_string(),
                "*.yml".to_string(),
                ".yamllint".to_string(),
            ],
        }
    }
}
