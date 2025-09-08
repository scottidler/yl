use super::{LintContext, Problem};
use crate::config::{Config, InlineConfigManager};
use crate::rules::RuleRegistry;
use eyre::Result;
use rayon::prelude::*;
use std::path::Path;
use std::sync::Arc;
use walkdir::WalkDir;

/// Main linting engine that coordinates rule execution
pub struct Linter {
    registry: RuleRegistry,
    config: Config,
}

impl Linter {
    /// Create a new linter with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            registry: RuleRegistry::with_default_rules(),
            config,
        }
    }

    /// Lint a single file
    pub fn lint_file<P: AsRef<Path>>(&self, file_path: P) -> Result<Vec<Problem>> {
        let file_path = file_path.as_ref();

        // Check if file should be ignored
        if self.config.is_file_ignored(file_path) {
            return Ok(Vec::new());
        }

        // Check if file is a YAML file
        if !self.config.is_yaml_file(file_path) {
            return Ok(Vec::new());
        }

        // Read file content
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| eyre::eyre!("Failed to read file {}: {}", file_path.display(), e))?;

        self.lint_content(file_path, &content)
    }

    /// Lint content with a given file path context
    pub fn lint_content<P: AsRef<Path>>(&self, file_path: P, content: &str) -> Result<Vec<Problem>> {
        let file_path = file_path.as_ref();
        let context = LintContext::new(file_path, content);
        let mut all_problems = Vec::new();

        // Process inline directives
        let mut inline_config = InlineConfigManager::new();
        inline_config.process_file(content)?;

        // Check if entire file should be ignored
        if inline_config.is_file_ignored() {
            return Ok(Vec::new());
        }

        // Run all enabled rules
        for rule in self.registry.rules() {
            let mut rule_config = self.config.get_rule_config(rule.id(), &self.registry);

            // Apply inline configuration overrides
            if let Some(inline_rule_config) = inline_config.get_rule_config(rule.id(), 0) {
                // Merge inline config with base config
                for (key, value) in &inline_rule_config.params {
                    rule_config.set_param(key.clone(), value.clone());
                }
            }

            if !rule_config.enabled {
                continue;
            }

            // Validate rule configuration
            if let Err(e) = rule.validate_config(&rule_config) {
                return Err(eyre::eyre!("Invalid configuration for rule '{}': {}", rule.id(), e));
            }

            // Run the rule
            match rule.check(&context, &rule_config) {
                Ok(problems) => {
                    // Filter problems based on inline configuration
                    let filtered_problems: Vec<Problem> = problems
                        .into_iter()
                        .filter(|p| !inline_config.is_rule_disabled(&p.rule, p.line))
                        .collect();
                    all_problems.extend(filtered_problems);
                }
                Err(e) => {
                    return Err(eyre::eyre!(
                        "Rule '{}' failed on file {}: {}",
                        rule.id(),
                        file_path.display(),
                        e
                    ));
                }
            }
        }

        // Sort problems by line and column
        all_problems.sort();
        Ok(all_problems)
    }

    /// Lint multiple files or directories
    pub fn lint_paths<P: AsRef<Path>>(&self, paths: &[P]) -> Result<Vec<(std::path::PathBuf, Vec<Problem>)>> {
        let mut file_paths = Vec::new();

        // Collect all file paths first
        for path in paths {
            let path = path.as_ref();

            if path.is_file() {
                file_paths.push(path.to_path_buf());
            } else if path.is_dir() {
                // Recursively find YAML files in directory
                for entry in WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                {
                    let file_path = entry.path();

                    // Skip if ignored or not a YAML file
                    if self.config.is_file_ignored(file_path) || !self.config.is_yaml_file(file_path) {
                        continue;
                    }

                    file_paths.push(file_path.to_path_buf());
                }
            } else {
                return Err(eyre::eyre!("Path does not exist: {}", path.display()));
            }
        }

        // Process files in parallel
        self.lint_files_parallel(&file_paths)
    }

    /// Lint multiple files in parallel
    pub fn lint_files_parallel(
        &self,
        file_paths: &[std::path::PathBuf],
    ) -> Result<Vec<(std::path::PathBuf, Vec<Problem>)>> {
        let config = Arc::new(&self.config);

        let results: Result<Vec<_>, _> = file_paths
            .par_iter()
            .map(|file_path| {
                // Create a temporary linter for this thread
                let thread_linter = Linter {
                    registry: RuleRegistry::with_default_rules(), // Each thread gets its own registry
                    config: (*config).clone(),
                };

                let problems = thread_linter.lint_file(file_path)?;
                Ok((file_path.clone(), problems))
            })
            .collect();

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        fs::write(&file_path, content).expect("Failed to write test file");
        file_path
    }

    #[test]
    fn test_linter_creation() {
        let config = Config::default();
        let linter = Linter::new(config);

        assert!(!linter.registry.rule_ids().is_empty());
    }

    #[test]
    fn test_lint_content_valid_yaml() {
        let config = Config::default();
        let linter = Linter::new(config);

        let content = "key: value\nother: data";
        let problems = linter.lint_content("test.yaml", content).expect("Linting failed");

        // Should have no problems for valid, short content
        assert!(problems.is_empty());
    }

    #[test]
    fn test_lint_content_long_lines() {
        let config = Config::default();
        let linter = Linter::new(config);

        // Use a breakable long line (with spaces)
        let long_line = "this is a very long line with many words that definitely exceeds the eighty character limit";
        let problems = linter.lint_content("test.yaml", long_line).expect("Linting failed");

        // Should have line-length problem
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].rule, "line-length");
        assert_eq!(problems[0].line, 1);
    }

    #[test]
    fn test_lint_content_trailing_spaces() {
        let config = Config::default();
        let linter = Linter::new(config);

        let content = "key: value   \nother: data";
        let problems = linter.lint_content("test.yaml", content).expect("Linting failed");

        // Should have trailing-spaces problem
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].rule, "trailing-spaces");
        assert_eq!(problems[0].line, 1);
    }

    #[test]
    fn test_lint_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = create_test_file(&temp_dir, "test.yaml", "key: value");

        let config = Config::default();
        let linter = Linter::new(config);

        let problems = linter.lint_file(&file_path).expect("Linting failed");
        assert!(problems.is_empty());
    }

    #[test]
    fn test_lint_file_ignored() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = create_test_file(&temp_dir, "test.generated.yaml", "key: value");

        let config = Config::default(); // Default config ignores *.generated.yaml
        let linter = Linter::new(config);

        let problems = linter.lint_file(&file_path).expect("Linting failed");
        assert!(problems.is_empty()); // Should be ignored
    }

    #[test]
    fn test_lint_file_not_yaml() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = create_test_file(&temp_dir, "test.txt", "not yaml content");

        let config = Config::default();
        let linter = Linter::new(config);

        let problems = linter.lint_file(&file_path).expect("Linting failed");
        assert!(problems.is_empty()); // Should be ignored as not YAML
    }

    #[test]
    fn test_lint_paths_single_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = create_test_file(&temp_dir, "test.yaml", "key: value");

        let config = Config::default();
        let linter = Linter::new(config);

        let results = linter.lint_paths(&[&file_path]).expect("Linting failed");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, file_path);
        assert!(results[0].1.is_empty());
    }

    #[test]
    fn test_lint_paths_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        create_test_file(&temp_dir, "test1.yaml", "key: value");
        create_test_file(&temp_dir, "test2.yml", "other: data");
        create_test_file(&temp_dir, "ignored.txt", "not yaml");

        let config = Config::default();
        let linter = Linter::new(config);

        let results = linter.lint_paths(&[temp_dir.path()]).expect("Linting failed");
        assert_eq!(results.len(), 2); // Only YAML files should be processed

        let file_names: Vec<String> = results
            .iter()
            .map(|(path, _)| path.file_name().unwrap().to_string_lossy().to_string())
            .collect();
        assert!(file_names.contains(&"test1.yaml".to_string()));
        assert!(file_names.contains(&"test2.yml".to_string()));
    }

    #[test]
    fn test_problem_sorting() {
        let config = Config::default();
        let linter = Linter::new(config);

        let content = format!(
            "{}\n{}\n{}",
            "this is a very long line with many words that definitely exceeds the eighty character limit", // Line 1: long line
            "short",                                                                                       // Line 2: ok
            "trailing   " // Line 3: trailing spaces
        );

        let problems = linter.lint_content("test.yaml", &content).expect("Linting failed");

        // Problems should be sorted by line number
        assert_eq!(problems.len(), 2);
        assert_eq!(problems[0].line, 1); // line-length problem
        assert_eq!(problems[1].line, 3); // trailing-spaces problem
    }
}
