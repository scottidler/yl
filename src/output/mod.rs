pub mod human;
pub mod json;

use crate::linter::Problem;
use std::path::PathBuf;

/// Trait for formatting linting results
pub trait OutputFormatter {
    /// Format the linting results for output
    fn format_results(&self, results: &[(PathBuf, Vec<Problem>)]) -> String;
}

/// Get the appropriate formatter for the given format
pub fn get_formatter(format: &crate::cli::OutputFormat) -> Box<dyn OutputFormatter> {
    match format {
        crate::cli::OutputFormat::Human => Box::new(human::HumanFormatter::new()),
        crate::cli::OutputFormat::Json => Box::new(json::JsonFormatter::new()),
    }
}

/// Statistics about linting results
#[derive(Debug, Default)]
pub struct LintStats {
    pub total_files: usize,
    pub files_with_problems: usize,
    pub total_problems: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

impl LintStats {
    /// Calculate statistics from linting results
    pub fn from_results(results: &[(PathBuf, Vec<Problem>)]) -> Self {
        let mut stats = Self::default();

        stats.total_files = results.len();
        stats.files_with_problems = results.iter().filter(|(_, problems)| !problems.is_empty()).count();

        for (_, problems) in results {
            stats.total_problems += problems.len();

            for problem in problems {
                match problem.level {
                    crate::linter::Level::Error => stats.errors += 1,
                    crate::linter::Level::Warning => stats.warnings += 1,
                    crate::linter::Level::Info => stats.info += 1,
                }
            }
        }

        stats
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    /// Check if there are any problems
    pub fn has_problems(&self) -> bool {
        self.total_problems > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Level, Problem};

    #[test]
    fn test_lint_stats_empty() {
        let results = vec![];
        let stats = LintStats::from_results(&results);

        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.files_with_problems, 0);
        assert_eq!(stats.total_problems, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.warnings, 0);
        assert_eq!(stats.info, 0);
        assert!(!stats.has_errors());
        assert!(!stats.has_problems());
    }

    #[test]
    fn test_lint_stats_with_problems() {
        let results = vec![
            (PathBuf::from("file1.yaml"), vec![
                Problem::new(1, 1, Level::Error, "rule1", "error message"),
                Problem::new(2, 1, Level::Warning, "rule2", "warning message"),
            ]),
            (PathBuf::from("file2.yaml"), vec![]),
            (PathBuf::from("file3.yaml"), vec![
                Problem::new(1, 1, Level::Info, "rule3", "info message"),
            ]),
        ];

        let stats = LintStats::from_results(&results);

        assert_eq!(stats.total_files, 3);
        assert_eq!(stats.files_with_problems, 2);
        assert_eq!(stats.total_problems, 3);
        assert_eq!(stats.errors, 1);
        assert_eq!(stats.warnings, 1);
        assert_eq!(stats.info, 1);
        assert!(stats.has_errors());
        assert!(stats.has_problems());
    }
}
