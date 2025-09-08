use super::{LintStats, OutputFormatter};
use crate::linter::Problem;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// JSON output formatter
#[derive(Debug, Default)]
pub struct JsonFormatter;

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new() -> Self {
        Self
    }
}

/// JSON representation of linting results
#[derive(Debug, Serialize, Deserialize)]
struct JsonOutput {
    /// Statistics about the linting run
    stats: JsonStats,
    /// Results for each file
    files: Vec<JsonFileResult>,
}

/// JSON representation of linting statistics
#[derive(Debug, Serialize, Deserialize)]
struct JsonStats {
    total_files: usize,
    files_with_problems: usize,
    total_problems: usize,
    errors: usize,
    warnings: usize,
    info: usize,
}

impl From<&LintStats> for JsonStats {
    fn from(stats: &LintStats) -> Self {
        Self {
            total_files: stats.total_files,
            files_with_problems: stats.files_with_problems,
            total_problems: stats.total_problems,
            errors: stats.errors,
            warnings: stats.warnings,
            info: stats.info,
        }
    }
}

/// JSON representation of results for a single file
#[derive(Debug, Serialize, Deserialize)]
struct JsonFileResult {
    /// Path to the file
    path: String,
    /// Problems found in the file
    problems: Vec<JsonProblem>,
}

/// JSON representation of a single problem
#[derive(Debug, Serialize, Deserialize)]
struct JsonProblem {
    /// Line number (1-based)
    line: usize,
    /// Column number (1-based)
    column: usize,
    /// Severity level
    level: String,
    /// Rule that detected the problem
    rule: String,
    /// Problem description
    message: String,
    /// Optional suggestion for fixing the problem
    #[serde(skip_serializing_if = "Option::is_none")]
    suggestion: Option<String>,
}

impl From<&Problem> for JsonProblem {
    fn from(problem: &Problem) -> Self {
        Self {
            line: problem.line,
            column: problem.column,
            level: problem.level.to_string(),
            rule: problem.rule.clone(),
            message: problem.message.clone(),
            suggestion: problem.suggestion.clone(),
        }
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_results(&self, results: &[(PathBuf, Vec<Problem>)]) -> String {
        let stats = LintStats::from_results(results);

        let json_output = JsonOutput {
            stats: JsonStats::from(&stats),
            files: results
                .iter()
                .map(|(path, problems)| JsonFileResult {
                    path: path.display().to_string(),
                    problems: problems.iter().map(JsonProblem::from).collect(),
                })
                .collect(),
        };

        serde_json::to_string_pretty(&json_output)
            .unwrap_or_else(|e| format!(r#"{{"error": "Failed to serialize JSON: {e}"}}"#))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Level, Problem};

    #[test]
    fn test_json_formatter_empty_results() {
        let formatter = JsonFormatter::new();
        let results = vec![];

        let output = formatter.format_results(&results);
        let parsed: JsonOutput = serde_json::from_str(&output).expect("Invalid JSON");

        assert_eq!(parsed.stats.total_files, 0);
        assert_eq!(parsed.stats.total_problems, 0);
        assert!(parsed.files.is_empty());
    }

    #[test]
    fn test_json_formatter_with_problems() {
        let formatter = JsonFormatter::new();
        let results = vec![
            (
                PathBuf::from("test.yaml"),
                vec![
                    Problem::new(10, 5, Level::Error, "line-length", "line too long"),
                    Problem::with_suggestion(
                        15,
                        1,
                        Level::Warning,
                        "trailing-spaces",
                        "trailing whitespace",
                        "Remove trailing spaces",
                    ),
                ],
            ),
            (PathBuf::from("clean.yaml"), vec![]),
        ];

        let output = formatter.format_results(&results);
        let parsed: JsonOutput = serde_json::from_str(&output).expect("Invalid JSON");

        assert_eq!(parsed.stats.total_files, 2);
        assert_eq!(parsed.stats.files_with_problems, 1);
        assert_eq!(parsed.stats.total_problems, 2);
        assert_eq!(parsed.stats.errors, 1);
        assert_eq!(parsed.stats.warnings, 1);
        assert_eq!(parsed.stats.info, 0);

        assert_eq!(parsed.files.len(), 2);

        // Check first file with problems
        let first_file = &parsed.files[0];
        assert_eq!(first_file.path, "test.yaml");
        assert_eq!(first_file.problems.len(), 2);

        let first_problem = &first_file.problems[0];
        assert_eq!(first_problem.line, 10);
        assert_eq!(first_problem.column, 5);
        assert_eq!(first_problem.level, "error");
        assert_eq!(first_problem.rule, "line-length");
        assert_eq!(first_problem.message, "line too long");
        assert_eq!(first_problem.suggestion, None);

        let second_problem = &first_file.problems[1];
        assert_eq!(second_problem.line, 15);
        assert_eq!(second_problem.column, 1);
        assert_eq!(second_problem.level, "warning");
        assert_eq!(second_problem.rule, "trailing-spaces");
        assert_eq!(second_problem.message, "trailing whitespace");
        assert_eq!(second_problem.suggestion, Some("Remove trailing spaces".to_string()));

        // Check second file without problems
        let second_file = &parsed.files[1];
        assert_eq!(second_file.path, "clean.yaml");
        assert!(second_file.problems.is_empty());
    }

    #[test]
    fn test_json_problem_conversion() {
        let problem = Problem::with_suggestion(42, 13, Level::Info, "test-rule", "test message", "test suggestion");

        let json_problem = JsonProblem::from(&problem);

        assert_eq!(json_problem.line, 42);
        assert_eq!(json_problem.column, 13);
        assert_eq!(json_problem.level, "info");
        assert_eq!(json_problem.rule, "test-rule");
        assert_eq!(json_problem.message, "test message");
        assert_eq!(json_problem.suggestion, Some("test suggestion".to_string()));
    }

    #[test]
    fn test_json_stats_conversion() {
        let stats = LintStats {
            total_files: 5,
            files_with_problems: 3,
            total_problems: 10,
            errors: 4,
            warnings: 5,
            info: 1,
        };

        let json_stats = JsonStats::from(&stats);

        assert_eq!(json_stats.total_files, 5);
        assert_eq!(json_stats.files_with_problems, 3);
        assert_eq!(json_stats.total_problems, 10);
        assert_eq!(json_stats.errors, 4);
        assert_eq!(json_stats.warnings, 5);
        assert_eq!(json_stats.info, 1);
    }

    #[test]
    fn test_json_serialization_roundtrip() {
        let original = JsonOutput {
            stats: JsonStats {
                total_files: 1,
                files_with_problems: 1,
                total_problems: 1,
                errors: 1,
                warnings: 0,
                info: 0,
            },
            files: vec![JsonFileResult {
                path: "test.yaml".to_string(),
                problems: vec![JsonProblem {
                    line: 1,
                    column: 1,
                    level: "error".to_string(),
                    rule: "test-rule".to_string(),
                    message: "test message".to_string(),
                    suggestion: None,
                }],
            }],
        };

        let serialized = serde_json::to_string(&original).expect("Serialization failed");
        let deserialized: JsonOutput = serde_json::from_str(&serialized).expect("Deserialization failed");

        assert_eq!(deserialized.stats.total_files, original.stats.total_files);
        assert_eq!(deserialized.files.len(), original.files.len());
        assert_eq!(deserialized.files[0].path, original.files[0].path);
        assert_eq!(deserialized.files[0].problems.len(), original.files[0].problems.len());
    }
}
