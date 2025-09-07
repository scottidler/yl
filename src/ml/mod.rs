use crate::config::Config;
use crate::rules::{RuleConfig, ConfigValue};
use crate::linter::Level;
use eyre::Result;
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;




/// Pattern learner that analyzes codebases to suggest rule configurations
pub struct PatternLearner {
    learned_patterns: HashMap<String, PatternInfo>,
}

/// Information about learned patterns
#[derive(Debug, Clone)]
pub struct PatternInfo {
    pub confidence: f64,
    pub suggested_config: RuleConfig,
}

impl PatternLearner {
    /// Create a new pattern learner
    pub fn new() -> Self {
        Self {
            learned_patterns: HashMap::new(),
        }
    }

    /// Learn patterns from an existing codebase
    pub fn learn_from_codebase<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        let mut file_patterns = HashMap::new();
        let mut content_analysis = ContentAnalyzer::new();

        // Analyze all YAML files in the codebase
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();

            if self.is_yaml_file(file_path) {
                let content = std::fs::read_to_string(file_path)?;

                // Analyze patterns in this file
                let patterns = self.analyze_file_patterns(&content)?;
                for pattern in patterns {
                    *file_patterns.entry(pattern).or_insert(0) += 1;
                }

                // Add to content analysis
                content_analysis.add_sample(&content);
            }
        }

        // Learn from the collected patterns
        self.learn_patterns(file_patterns, content_analysis)?;

        Ok(())
    }

    /// Suggest rule configurations based on learned patterns
    pub fn suggest_rules(&self) -> Vec<RuleConfig> {
        let mut suggestions = Vec::new();

        for (_rule_name, pattern_info) in &self.learned_patterns {
            if pattern_info.confidence > 0.7 {
                let mut config = pattern_info.suggested_config.clone();
                config.enabled = true;
                suggestions.push(config);
            }
        }

        // Sort by confidence
        suggestions.sort_by(|a, b| {
            let a_confidence = self.get_rule_confidence(&a);
            let b_confidence = self.get_rule_confidence(&b);
            b_confidence.partial_cmp(&a_confidence).unwrap_or(std::cmp::Ordering::Equal)
        });

        suggestions
    }

    /// Generate a configuration based on project analysis
    pub fn generate_config<P: AsRef<Path>>(&mut self, project_path: P) -> Result<Config> {
        self.learn_from_codebase(project_path)?;

        let mut config = Config::default();
        let suggested_rules = self.suggest_rules();

        for rule_config in suggested_rules {
            // Extract rule name from the config (this would need to be added to RuleConfig)
            // For now, we'll use a placeholder approach
            config.rules.insert("suggested-rule".to_string(), rule_config);
        }

        Ok(config)
    }

    /// Analyze patterns in a single file
    fn analyze_file_patterns(&self, content: &str) -> Result<Vec<String>> {
        let mut patterns = Vec::new();

        // Analyze indentation patterns
        let indent_pattern = self.analyze_indentation(content);
        patterns.push(format!("indent:{}", indent_pattern));

        // Analyze line length patterns
        let line_length_pattern = self.analyze_line_lengths(content);
        patterns.push(format!("line-length:{}", line_length_pattern));

        // Analyze quote usage patterns
        let quote_pattern = self.analyze_quote_usage(content);
        patterns.push(format!("quotes:{}", quote_pattern));

        // Analyze spacing patterns
        let spacing_pattern = self.analyze_spacing(content);
        patterns.push(format!("spacing:{}", spacing_pattern));

        Ok(patterns)
    }

    /// Analyze indentation patterns in content
    fn analyze_indentation(&self, content: &str) -> String {
        let mut space_count = 0;
        let mut tab_count = 0;
        let mut indent_sizes = HashMap::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let leading_spaces = line.len() - line.trim_start().len();
            let leading_chars = &line[..leading_spaces];

            if leading_chars.contains('\t') {
                tab_count += 1;
            } else if leading_spaces > 0 {
                space_count += 1;
                *indent_sizes.entry(leading_spaces).or_insert(0) += 1;
            }
        }

        if tab_count > space_count {
            "tabs".to_string()
        } else if let Some((most_common_size, _)) = indent_sizes.iter().max_by_key(|(_, count)| *count) {
            format!("spaces:{}", *most_common_size)
        } else {
            "spaces:2".to_string()
        }
    }

    /// Analyze line length patterns
    fn analyze_line_lengths(&self, content: &str) -> String {
        let lengths: Vec<usize> = content.lines().map(|line| line.len()).collect();

        if lengths.is_empty() {
            return "80".to_string();
        }

        let max_length = *lengths.iter().max().unwrap();
        let avg_length = lengths.iter().sum::<usize>() / lengths.len();

        // Suggest based on common conventions
        if max_length <= 80 && avg_length <= 60 {
            "80".to_string()
        } else if max_length <= 100 && avg_length <= 80 {
            "100".to_string()
        } else if max_length <= 120 && avg_length <= 100 {
            "120".to_string()
        } else {
            "120".to_string()
        }
    }

    /// Analyze quote usage patterns
    fn analyze_quote_usage(&self, content: &str) -> String {
        let mut single_quotes = 0;
        let mut double_quotes = 0;
        let mut unquoted = 0;

        // Simple pattern matching for quote analysis
        for line in content.lines() {
            single_quotes += line.matches('\'').count();
            double_quotes += line.matches('"').count();

            // Count unquoted values (simplified)
            if line.contains(':') && !line.contains('"') && !line.contains('\'') {
                unquoted += 1;
            }
        }

        if double_quotes > single_quotes && double_quotes > unquoted {
            "double".to_string()
        } else if single_quotes > double_quotes && single_quotes > unquoted {
            "single".to_string()
        } else {
            "minimal".to_string()
        }
    }

    /// Analyze spacing patterns
    fn analyze_spacing(&self, content: &str) -> String {
        let mut colon_space_after = 0;
        let mut colon_no_space_after = 0;

        for line in content.lines() {
            if let Some(colon_pos) = line.find(':') {
                if colon_pos + 1 < line.len() {
                    let after_colon = &line[colon_pos + 1..colon_pos + 2];
                    if after_colon == " " {
                        colon_space_after += 1;
                    } else {
                        colon_no_space_after += 1;
                    }
                }
            }
        }

        if colon_space_after > colon_no_space_after {
            "space-after-colon".to_string()
        } else {
            "no-space-after-colon".to_string()
        }
    }

    /// Learn patterns from collected data
    fn learn_patterns(&mut self, patterns: HashMap<String, usize>, _content_analysis: ContentAnalyzer) -> Result<()> {
        let total_files = patterns.values().sum::<usize>() as f64;

        for (pattern, frequency) in patterns {
            let confidence = frequency as f64 / total_files;

            // Convert pattern to rule configuration
            if let Some(rule_config) = self.pattern_to_rule_config(&pattern, confidence) {
                let pattern_info = PatternInfo {
                    confidence,
                    suggested_config: rule_config,
                };

                self.learned_patterns.insert(pattern, pattern_info);
            }
        }

        Ok(())
    }

    /// Convert a pattern string to a rule configuration
    fn pattern_to_rule_config(&self, pattern: &str, confidence: f64) -> Option<RuleConfig> {
        let parts: Vec<&str> = pattern.split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let rule_type = parts[0];
        let value = parts[1];

        let mut config = RuleConfig::new(true, Level::Warning);

        match rule_type {
            "indent" => {
                if value == "tabs" {
                    config.params.insert("indent-sequences".to_string(), ConfigValue::Bool(true));
                } else if let Some(size_str) = value.strip_prefix("spaces:") {
                    if let Ok(size) = size_str.parse::<i64>() {
                        config.params.insert("spaces".to_string(), ConfigValue::Int(size));
                    }
                }
            }
            "line-length" => {
                if let Ok(max_length) = value.parse::<i64>() {
                    config.params.insert("max".to_string(), ConfigValue::Int(max_length));
                }
            }
            "quotes" => {
                config.params.insert("prefer".to_string(), ConfigValue::String(value.to_string()));
            }
            "spacing" => {
                if value == "space-after-colon" {
                    config.params.insert("min-spaces-after".to_string(), ConfigValue::Int(1));
                    config.params.insert("max-spaces-after".to_string(), ConfigValue::Int(1));
                }
            }
            _ => return None,
        }

        // Adjust confidence-based settings
        if confidence < 0.5 {
            config.level = Level::Info;
        } else if confidence > 0.8 {
            config.level = Level::Error;
        }

        Some(config)
    }

    /// Check if a file is a YAML file
    fn is_yaml_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            matches!(extension.to_str(), Some("yaml") | Some("yml"))
        } else {
            false
        }
    }

    /// Get confidence for a rule configuration
    fn get_rule_confidence(&self, _config: &RuleConfig) -> f64 {
        // Placeholder implementation
        0.5
    }
}

impl Default for PatternLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// Content analyzer for gathering statistics about YAML content
#[derive(Debug)]
struct ContentAnalyzer {
    samples: Vec<String>,
}

impl ContentAnalyzer {
    fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    fn add_sample(&mut self, content: &str) {
        self.samples.push(content.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_pattern_learner_creation() {
        let learner = PatternLearner::new();
        assert!(learner.learned_patterns.is_empty());
        assert!(learner.training_data.is_empty());
    }

    #[test]
    fn test_analyze_indentation_spaces() {
        let learner = PatternLearner::new();
        let content = "key1:\n  subkey1: value1\n  subkey2: value2\n";

        let pattern = learner.analyze_indentation(content);
        assert!(pattern.starts_with("spaces:"));
    }

    #[test]
    fn test_analyze_indentation_tabs() {
        let learner = PatternLearner::new();
        let content = "key1:\n\tsubkey1: value1\n\tsubkey2: value2\n";

        let pattern = learner.analyze_indentation(content);
        assert_eq!(pattern, "tabs");
    }

    #[test]
    fn test_analyze_line_lengths() {
        let learner = PatternLearner::new();
        let content = "short: line\nvery_long_line_that_exceeds_normal_length: value\n";

        let pattern = learner.analyze_line_lengths(content);
        assert!(["80", "100", "120"].contains(&pattern.as_str()));
    }

    #[test]
    fn test_analyze_quote_usage() {
        let learner = PatternLearner::new();
        let content = "key1: \"double quoted\"\nkey2: 'single quoted'\nkey3: unquoted\n";

        let pattern = learner.analyze_quote_usage(content);
        assert!(["double", "single", "minimal"].contains(&pattern.as_str()));
    }

    #[test]
    fn test_learn_from_codebase() {
        let temp_dir = TempDir::new().unwrap();
        let yaml_file = temp_dir.path().join("test.yaml");

        fs::write(&yaml_file, "key1:\n  subkey: value\nkey2: \"quoted value\"\n").unwrap();

        let mut learner = PatternLearner::new();
        let result = learner.learn_from_codebase(temp_dir.path());

        assert!(result.is_ok());
        assert!(!learner.learned_patterns.is_empty());
    }

    #[test]
    fn test_suggest_rules() {
        let mut learner = PatternLearner::new();

        // Add some mock learned patterns
        let pattern_info = PatternInfo {
            frequency: 10,
            confidence: 0.8,
            suggested_config: RuleConfig::new(true, Level::Warning),
            context: "indent:spaces:2".to_string(),
        };

        learner.learned_patterns.insert("indent:spaces:2".to_string(), pattern_info);

        let suggestions = learner.suggest_rules();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_pattern_to_rule_config() {
        let learner = PatternLearner::new();

        let config = learner.pattern_to_rule_config("line-length:100", 0.8);
        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(config.get_int("max"), Some(100));
    }
}
