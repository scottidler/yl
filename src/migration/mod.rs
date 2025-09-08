use crate::config::Config;
use crate::linter::Level;
use crate::rules::{ConfigValue, RuleConfig};
use eyre::Result;
use regex::Regex;
use serde_yaml::Value;
use std::fs;
use std::path::Path;

/// Migration utilities for converting from yamllint to yl
pub struct YamllintMigrator;

impl YamllintMigrator {
    /// Convert a yamllint configuration file to yl format
    pub fn convert_config<P: AsRef<Path>>(yamllint_config_path: P) -> Result<Config> {
        let content = fs::read_to_string(yamllint_config_path)?;
        let yamllint_config: Value = serde_yaml::from_str(&content)?;

        let mut yl_config = Config::default();
        yl_config.rules.clear(); // Start with empty rules

        // Handle extends
        if let Some(extends) = yamllint_config.get("extends") {
            if let Some(extends_str) = extends.as_str() {
                yl_config.extends = Some(Self::convert_extends(extends_str));
            }
        }

        // Convert rules
        if let Some(rules) = yamllint_config.get("rules") {
            if let Some(rules_map) = rules.as_mapping() {
                for (rule_name, rule_config) in rules_map {
                    if let Some(rule_name_str) = rule_name.as_str() {
                        let yl_rule_name = Self::convert_rule_name(rule_name_str);
                        let yl_rule_config = Self::convert_rule_config(rule_config)?;
                        yl_config.rules.insert(yl_rule_name, yl_rule_config);
                    }
                }
            }
        }

        // Convert ignore patterns
        if let Some(ignore) = yamllint_config.get("ignore") {
            if let Some(ignore_str) = ignore.as_str() {
                yl_config.ignore = vec![ignore_str.to_string()];
            } else if let Some(ignore_seq) = ignore.as_sequence() {
                yl_config.ignore = ignore_seq
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }

        Ok(yl_config)
    }

    /// Convert yamllint directives in YAML content to yl directives
    pub fn convert_directives(content: &str) -> String {
        let mut converted = content.to_string();

        // Convert yamllint disable directives
        let patterns = vec![
            (r"# yamllint disable-line rule:([a-zA-Z0-9_-]+)", "# yl:disable-line $1"),
            (r"# yamllint disable rule:([a-zA-Z0-9_-]+)", "# yl:disable $1"),
            (r"# yamllint enable rule:([a-zA-Z0-9_-]+)", "# yl:enable $1"),
            (r"# yamllint disable-line", "# yl:disable-line"),
            (r"# yamllint disable", "# yl:disable"),
            (r"# yamllint enable", "# yl:enable"),
        ];

        for (pattern, replacement) in patterns {
            let regex = Regex::new(pattern).unwrap();
            converted = regex.replace_all(&converted, replacement).to_string();
        }

        converted
    }

    /// Convert yamllint extends to yl format
    fn convert_extends(extends: &str) -> String {
        match extends {
            "default" => "default".to_string(),
            "relaxed" => "relaxed".to_string(),
            _ => extends.to_string(),
        }
    }

    /// Convert yamllint rule names to yl rule names
    fn convert_rule_name(yamllint_name: &str) -> String {
        match yamllint_name {
            "braces" => "braces".to_string(),
            "brackets" => "brackets".to_string(),
            "colons" => "colons".to_string(),
            "commas" => "commas".to_string(),
            "comments" => "comments".to_string(),
            "comments-indentation" => "comments".to_string(), // Map to our comments rule
            "document-end" => "document-structure".to_string(),
            "document-start" => "document-structure".to_string(),
            "empty-lines" => "empty-lines".to_string(),
            "empty-values" => "truthy".to_string(), // Similar concept
            "hyphens" => "hyphens".to_string(),
            "indentation" => "indentation".to_string(),
            "key-duplicates" => "key-duplicates".to_string(),
            "key-ordering" => "key-ordering".to_string(),
            "line-length" => "line-length".to_string(),
            "new-line-at-end-of-file" => "new-line-at-end-of-file".to_string(),
            "octal-values" => "octal-values".to_string(),
            "quoted-strings" => "quoted-strings".to_string(),
            "trailing-spaces" => "trailing-spaces".to_string(),
            "truthy" => "truthy".to_string(),
            _ => yamllint_name.to_string(), // Keep unknown rules as-is
        }
    }

    /// Convert yamllint rule configuration to yl format
    fn convert_rule_config(config: &Value) -> Result<RuleConfig> {
        match config {
            Value::String(s) => {
                // Handle simple enable/disable
                match s.as_str() {
                    "enable" => Ok(RuleConfig::new(true, Level::Error)),
                    "disable" => Ok(RuleConfig::new(false, Level::Error)),
                    _ => Ok(RuleConfig::new(true, Level::Error)),
                }
            }
            Value::Mapping(map) => {
                let mut rule_config = RuleConfig::new(true, Level::Error);

                // Handle level
                if let Some(level_val) = map.get(&Value::String("level".to_string())) {
                    if let Some(level_str) = level_val.as_str() {
                        rule_config.level = match level_str {
                            "error" => Level::Error,
                            "warning" => Level::Warning,
                            "info" => Level::Info,
                            _ => Level::Error,
                        };
                    }
                }

                // Convert other parameters
                for (key, value) in map {
                    if let Some(key_str) = key.as_str() {
                        if key_str != "level" {
                            let config_value = Self::convert_config_value(value)?;
                            rule_config.params.insert(key_str.to_string(), config_value);
                        }
                    }
                }

                Ok(rule_config)
            }
            _ => Ok(RuleConfig::new(true, Level::Error)),
        }
    }

    /// Convert yamllint config values to yl ConfigValue
    fn convert_config_value(value: &Value) -> Result<ConfigValue> {
        match value {
            Value::Bool(b) => Ok(ConfigValue::Bool(*b)),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(ConfigValue::Int(i))
                } else {
                    Ok(ConfigValue::String(n.to_string()))
                }
            }
            Value::String(s) => Ok(ConfigValue::String(s.clone())),
            Value::Sequence(seq) => {
                let converted: Result<Vec<ConfigValue>, _> =
                    seq.iter().map(|v| Self::convert_config_value(v)).collect();
                Ok(ConfigValue::Array(converted?))
            }
            _ => Ok(ConfigValue::String(format!("{:?}", value))),
        }
    }

    /// Generate a migration report showing what was converted
    pub fn generate_migration_report(_original_config: &str, converted_config: &Config) -> Result<String> {
        let mut report = String::new();

        report.push_str("# YL Migration Report\n\n");
        report.push_str("## Original yamllint configuration converted to yl format\n\n");

        // Show extends
        if let Some(extends) = &converted_config.extends {
            report.push_str(&format!("**Extends**: {}\n\n", extends));
        }

        // Show converted rules
        report.push_str("## Converted Rules\n\n");
        for (rule_name, rule_config) in &converted_config.rules {
            report.push_str(&format!("- **{}**: ", rule_name));
            if rule_config.enabled {
                report.push_str(&format!(
                    "enabled ({})",
                    match rule_config.level {
                        Level::Error => "error",
                        Level::Warning => "warning",
                        Level::Info => "info",
                    }
                ));
            } else {
                report.push_str("disabled");
            }

            if !rule_config.params.is_empty() {
                report.push_str(" with parameters:");
                for (key, value) in &rule_config.params {
                    report.push_str(&format!("\n  - {}: {:?}", key, value));
                }
            }
            report.push('\n');
        }

        // Show ignore patterns
        if !converted_config.ignore.is_empty() {
            report.push_str("\n## Ignore Patterns\n\n");
            for pattern in &converted_config.ignore {
                report.push_str(&format!("- {}\n", pattern));
            }
        }

        report.push_str("\n## Migration Notes\n\n");
        report.push_str("- All yamllint directives in YAML files should be converted using `yl migrate-directives`\n");
        report.push_str("- Some rule names may have been mapped to equivalent yl rules\n");
        report.push_str("- Review the converted configuration and adjust as needed\n");

        Ok(report)
    }

    /// Migrate a complete yamllint project to yl
    pub fn migrate_project<P: AsRef<Path>>(project_path: P) -> Result<()> {
        let project_path = project_path.as_ref();

        // Look for yamllint config files
        let yamllint_configs = vec![
            project_path.join(".yamllint"),
            project_path.join(".yamllint.yml"),
            project_path.join(".yamllint.yaml"),
        ];

        for config_path in yamllint_configs {
            if config_path.exists() {
                println!("Found yamllint config: {}", config_path.display());

                // Convert config
                let yl_config = Self::convert_config(&config_path)?;

                // Write yl config
                let yl_config_path = project_path.join(".yl.yaml");
                let yl_config_content = serde_yaml::to_string(&yl_config)?;
                fs::write(&yl_config_path, yl_config_content)?;

                println!("Created yl config: {}", yl_config_path.display());

                // Generate migration report
                let original_content = fs::read_to_string(&config_path)?;
                let report = Self::generate_migration_report(&original_content, &yl_config)?;
                let report_path = project_path.join("yl-migration-report.md");
                fs::write(&report_path, report)?;

                println!("Generated migration report: {}", report_path.display());
                break;
            }
        }

        // Convert directives in YAML files
        Self::migrate_directives_in_directory(project_path)?;

        Ok(())
    }

    /// Migrate yamllint directives in all YAML files in a directory
    fn migrate_directives_in_directory<P: AsRef<Path>>(dir: P) -> Result<()> {
        use walkdir::WalkDir;

        let dir = dir.as_ref();
        let mut converted_files = 0;

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            // Check if it's a YAML file
            if let Some(extension) = path.extension() {
                let is_yaml = match extension.to_str() {
                    Some("yaml") | Some("yml") => true,
                    _ => false,
                };

                if is_yaml {
                    let content = fs::read_to_string(path)?;
                    let converted_content = Self::convert_directives(&content);

                    if content != converted_content {
                        fs::write(path, converted_content)?;
                        converted_files += 1;
                        println!("Converted directives in: {}", path.display());
                    }
                }
            }
        }

        if converted_files > 0 {
            println!("Converted directives in {} files", converted_files);
        } else {
            println!("No yamllint directives found to convert");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_directives() {
        let content = r#"
key: value
# yamllint disable-line rule:line-length
very_long_line: "this line would normally be too long"
# yamllint disable rule:trailing-spaces
messy_content: "value"
# yamllint enable rule:trailing-spaces
clean_content: "value"
"#;

        let converted = YamllintMigrator::convert_directives(content);

        assert!(converted.contains("# yl:disable-line line-length"));
        assert!(converted.contains("# yl:disable trailing-spaces"));
        assert!(converted.contains("# yl:enable trailing-spaces"));
    }

    #[test]
    fn test_convert_rule_name() {
        assert_eq!(YamllintMigrator::convert_rule_name("line-length"), "line-length");
        assert_eq!(
            YamllintMigrator::convert_rule_name("document-start"),
            "document-structure"
        );
        assert_eq!(
            YamllintMigrator::convert_rule_name("document-end"),
            "document-structure"
        );
        assert_eq!(YamllintMigrator::convert_rule_name("comments-indentation"), "comments");
    }

    #[test]
    fn test_convert_config_value() {
        let bool_val = Value::Bool(true);
        let converted = YamllintMigrator::convert_config_value(&bool_val).unwrap();
        assert_eq!(converted, ConfigValue::Bool(true));

        let int_val = Value::Number(serde_yaml::Number::from(42));
        let converted = YamllintMigrator::convert_config_value(&int_val).unwrap();
        assert_eq!(converted, ConfigValue::Int(42));

        let str_val = Value::String("test".to_string());
        let converted = YamllintMigrator::convert_config_value(&str_val).unwrap();
        assert_eq!(converted, ConfigValue::String("test".to_string()));
    }

    #[test]
    fn test_convert_rule_config_simple() {
        let enable_val = Value::String("enable".to_string());
        let config = YamllintMigrator::convert_rule_config(&enable_val).unwrap();
        assert!(config.enabled);

        let disable_val = Value::String("disable".to_string());
        let config = YamllintMigrator::convert_rule_config(&disable_val).unwrap();
        assert!(!config.enabled);
    }

    #[test]
    fn test_generate_migration_report() {
        let mut config = Config::default();
        config.extends = Some("default".to_string());

        let report = YamllintMigrator::generate_migration_report("original", &config).unwrap();

        assert!(report.contains("# YL Migration Report"));
        assert!(report.contains("**Extends**: default"));
        assert!(report.contains("## Migration Notes"));
    }
}
