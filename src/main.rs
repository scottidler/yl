use clap::Parser;
use eyre::{Context, Result};

mod cli;
mod config;
mod linter;
mod output;
mod rules;

use cli::Cli;
use config::Config;
use linter::Linter;
use output::{get_formatter, LintStats};
use rules::{ConfigValue, RuleRegistry};

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Load configuration
    let mut config = Config::load(cli.config.as_ref())
        .context("Failed to load configuration")?;

    // Apply CLI overrides to configuration
    apply_cli_overrides(&mut config, &cli)?;

    // Handle special commands
    if cli.list_rules {
        return list_rules();
    }

    if cli.show_config {
        return show_config(&config);
    }

    // Create linter
    let linter = Linter::new(config);

    // Get files to lint
    let files = cli.get_files();

    // Perform linting
    let results = linter
        .lint_paths(&files)
        .context("Linting failed")?;

    // Filter results based on CLI options
    let filtered_results = filter_results(results, &cli);

    // Format and output results
    let formatter = get_formatter(&cli.format);
    let output = formatter.format_results(&filtered_results);
    println!("{}", output);

    // Calculate statistics and determine exit code
    let stats = LintStats::from_results(&filtered_results);
    
    if cli.verbose {
        eprintln!("Processed {} files", stats.total_files);
        if stats.has_problems() {
            eprintln!("Found {} problems in {} files", stats.total_problems, stats.files_with_problems);
        }
    }

    // Exit with error code if there are errors
    if stats.has_errors() {
        std::process::exit(1);
    }

    Ok(())
}

/// Apply CLI overrides to the configuration
fn apply_cli_overrides(config: &mut Config, cli: &Cli) -> Result<()> {
    let registry = RuleRegistry::with_default_rules();

    // Disable rules specified via CLI
    for rule_id in cli.get_disabled_rules() {
        if let Some(rule_config) = config.rules.get_mut(&rule_id) {
            rule_config.enabled = false;
        } else {
            // Add disabled rule config if it doesn't exist
            let mut rule_config = registry
                .get(&rule_id)
                .map(|rule| rule.default_config())
                .unwrap_or_default();
            rule_config.enabled = false;
            config.rules.insert(rule_id, rule_config);
        }
    }

    // Enable rules specified via CLI
    for rule_id in cli.get_enabled_rules() {
        if let Some(rule_config) = config.rules.get_mut(&rule_id) {
            rule_config.enabled = true;
        } else {
            // Add enabled rule config if it doesn't exist
            let rule_config = registry
                .get(&rule_id)
                .map(|rule| rule.default_config())
                .unwrap_or_default();
            config.rules.insert(rule_id, rule_config);
        }
    }

    // Apply rule parameter settings
    for (rule_id, param, value) in cli.get_rule_settings() {
        let rule_config = config.rules.entry(rule_id.clone()).or_insert_with(|| {
            registry
                .get(&rule_id)
                .map(|rule| rule.default_config())
                .unwrap_or_default()
        });

        // Parse the value based on common types
        let config_value = parse_config_value(&value)?;
        rule_config.set_param(param, config_value);
    }

    Ok(())
}

/// Parse a string value into a ConfigValue
fn parse_config_value(value: &str) -> Result<ConfigValue> {
    // Try to parse as boolean
    if let Ok(bool_val) = value.parse::<bool>() {
        return Ok(ConfigValue::Bool(bool_val));
    }

    // Try to parse as integer
    if let Ok(int_val) = value.parse::<i64>() {
        return Ok(ConfigValue::Int(int_val));
    }

    // Default to string
    Ok(ConfigValue::String(value.to_string()))
}

/// List all available rules
fn list_rules() -> Result<()> {
    let registry = RuleRegistry::with_default_rules();
    
    println!("Available rules:");
    println!();
    
    for rule in registry.rules() {
        println!("  {}", rule.id());
        println!("    {}", rule.description());
        
        let config = rule.default_config();
        if !config.params.is_empty() {
            println!("    Parameters:");
            for (key, value) in &config.params {
                println!("      {}: {:?}", key, value);
            }
        }
        println!();
    }
    
    Ok(())
}

/// Show the effective configuration
fn show_config(config: &Config) -> Result<()> {
    let yaml = serde_yaml::to_string(config)
        .context("Failed to serialize configuration")?;
    
    println!("Effective configuration:");
    println!("{}", yaml);
    
    Ok(())
}

/// Filter results based on CLI options
fn filter_results(
    results: Vec<(std::path::PathBuf, Vec<linter::Problem>)>,
    cli: &Cli,
) -> Vec<(std::path::PathBuf, Vec<linter::Problem>)> {
    if !cli.errors_only {
        return results;
    }

    // Filter to only show errors
    results
        .into_iter()
        .map(|(path, problems)| {
            let error_problems = problems
                .into_iter()
                .filter(|p| matches!(p.level, linter::Level::Error))
                .collect();
            (path, error_problems)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Level, Problem};
    use std::path::PathBuf;

    #[test]
    fn test_parse_config_value() {
        assert_eq!(parse_config_value("true").unwrap(), ConfigValue::Bool(true));
        assert_eq!(parse_config_value("false").unwrap(), ConfigValue::Bool(false));
        assert_eq!(parse_config_value("42").unwrap(), ConfigValue::Int(42));
        assert_eq!(parse_config_value("hello").unwrap(), ConfigValue::String("hello".to_string()));
    }

    #[test]
    fn test_filter_results_all() {
        let cli = Cli { errors_only: false, ..Default::default() };
        let results = vec![
            (PathBuf::from("test.yaml"), vec![
                Problem::new(1, 1, Level::Error, "rule1", "error"),
                Problem::new(2, 1, Level::Warning, "rule2", "warning"),
            ]),
        ];
        
        let filtered = filter_results(results.clone(), &cli);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1.len(), 2);
    }

    #[test]
    fn test_filter_results_errors_only() {
        let cli = Cli { errors_only: true, ..Default::default() };
        let results = vec![
            (PathBuf::from("test.yaml"), vec![
                Problem::new(1, 1, Level::Error, "rule1", "error"),
                Problem::new(2, 1, Level::Warning, "rule2", "warning"),
            ]),
        ];
        
        let filtered = filter_results(results, &cli);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].1.len(), 1);
        assert_eq!(filtered[0].1[0].level, Level::Error);
    }

    #[test]
    fn test_apply_cli_overrides_disable() {
        let mut config = Config::default();
        let cli = Cli {
            disable: vec!["line-length".to_string()],
            ..Default::default()
        };
        
        apply_cli_overrides(&mut config, &cli).expect("Failed to apply overrides");
        
        let rule_config = config.rules.get("line-length").unwrap();
        assert!(!rule_config.enabled);
    }

    #[test]
    fn test_apply_cli_overrides_set_param() {
        let mut config = Config::default();
        let cli = Cli {
            set: vec!["line-length.max=120".to_string()],
            ..Default::default()
        };
        
        apply_cli_overrides(&mut config, &cli).expect("Failed to apply overrides");
        
        let rule_config = config.rules.get("line-length").unwrap();
        assert_eq!(rule_config.get_int("max"), Some(120));
    }
}