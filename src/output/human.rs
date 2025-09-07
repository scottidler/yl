use super::{LintStats, OutputFormatter};
use crate::linter::{Level, Problem};
use std::path::PathBuf;

/// Human-readable output formatter
#[derive(Debug, Default)]
pub struct HumanFormatter {
    use_colors: bool,
}

#[allow(dead_code)] // Some methods are part of API for future phases
impl HumanFormatter {
    /// Create a new human formatter
    pub fn new() -> Self {
        Self {
            use_colors: Self::should_use_colors(),
        }
    }

    /// Create a new human formatter with explicit color setting
    pub fn with_colors(use_colors: bool) -> Self {
        Self { use_colors }
    }

    /// Determine if colors should be used based on environment
    fn should_use_colors() -> bool {
        // Check if we're in a terminal and colors are supported
        atty::is(atty::Stream::Stdout) && std::env::var("NO_COLOR").is_err()
    }

    /// Format a problem level with appropriate color
    fn format_level(&self, level: &Level) -> String {
        if self.use_colors {
            match level {
                Level::Error => "\x1b[31merror\x1b[0m".to_string(),   // Red
                Level::Warning => "\x1b[33mwarning\x1b[0m".to_string(), // Yellow
                Level::Info => "\x1b[36minfo\x1b[0m".to_string(),    // Cyan
            }
        } else {
            level.to_string()
        }
    }

    /// Format a file path with appropriate color
    fn format_path(&self, path: &PathBuf) -> String {
        if self.use_colors {
            format!("\x1b[1m{}\x1b[0m", path.display()) // Bold
        } else {
            path.display().to_string()
        }
    }

    /// Format line and column numbers
    fn format_position(&self, line: usize, column: usize) -> String {
        if self.use_colors {
            format!("\x1b[36m{}:{}\x1b[0m", line, column) // Cyan
        } else {
            format!("{}:{}", line, column)
        }
    }

    /// Format a rule ID
    fn format_rule(&self, rule: &str) -> String {
        if self.use_colors {
            format!("\x1b[90m({})\x1b[0m", rule) // Gray
        } else {
            format!("({})", rule)
        }
    }

    /// Format statistics summary
    fn format_stats(&self, stats: &LintStats) -> String {
        let mut parts = Vec::new();

        if stats.errors > 0 {
            let text = if self.use_colors {
                format!("\x1b[31m{} error{}\x1b[0m", stats.errors, if stats.errors == 1 { "" } else { "s" })
            } else {
                format!("{} error{}", stats.errors, if stats.errors == 1 { "" } else { "s" })
            };
            parts.push(text);
        }

        if stats.warnings > 0 {
            let text = if self.use_colors {
                format!("\x1b[33m{} warning{}\x1b[0m", stats.warnings, if stats.warnings == 1 { "" } else { "s" })
            } else {
                format!("{} warning{}", stats.warnings, if stats.warnings == 1 { "" } else { "s" })
            };
            parts.push(text);
        }

        if stats.info > 0 {
            let text = if self.use_colors {
                format!("\x1b[36m{} info\x1b[0m", stats.info)
            } else {
                format!("{} info", stats.info)
            };
            parts.push(text);
        }

        if parts.is_empty() {
            if self.use_colors {
                "\x1b[32mNo problems found\x1b[0m".to_string() // Green
            } else {
                "No problems found".to_string()
            }
        } else {
            format!("Found {}", parts.join(", "))
        }
    }
}

impl OutputFormatter for HumanFormatter {
    fn format_results(&self, results: &[(PathBuf, Vec<Problem>)]) -> String {
        let mut output = Vec::new();
        let stats = LintStats::from_results(results);

        // Format problems for each file
        for (file_path, problems) in results {
            if problems.is_empty() {
                continue;
            }

            output.push(self.format_path(file_path));

            for problem in problems {
                let level = self.format_level(&problem.level);
                let position = self.format_position(problem.line, problem.column);
                let rule = self.format_rule(&problem.rule);

                output.push(format!(
                    "  {}: {} {} {}",
                    position,
                    level,
                    problem.message,
                    rule
                ));

                // Add suggestion if available
                if let Some(suggestion) = &problem.suggestion {
                    let suggestion_text = if self.use_colors {
                        format!("    \x1b[36mSuggestion:\x1b[0m {}", suggestion)
                    } else {
                        format!("    Suggestion: {}", suggestion)
                    };
                    output.push(suggestion_text);
                }
            }

            output.push(String::new()); // Empty line between files
        }

        // Add summary
        output.push(self.format_stats(&stats));

        output.join("\n")
    }
}

// Add atty dependency for color detection
#[cfg(not(test))]
mod atty {
    pub enum Stream {
        Stdout,
    }

    pub fn is(_stream: Stream) -> bool {
        // Simple implementation - in a real implementation, you'd use the atty crate
        std::env::var("TERM").is_ok() && std::env::var("NO_COLOR").is_err()
    }
}

// Mock atty for tests
#[cfg(test)]
mod atty {
    pub enum Stream {
        Stdout,
    }

    pub fn is(_stream: Stream) -> bool {
        false // Disable colors in tests for predictable output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::{Level, Problem};

    #[test]
    fn test_human_formatter_no_problems() {
        let formatter = HumanFormatter::with_colors(false);
        let results = vec![
            (PathBuf::from("file1.yaml"), vec![]),
            (PathBuf::from("file2.yaml"), vec![]),
        ];

        let output = formatter.format_results(&results);
        assert_eq!(output, "No problems found");
    }

    #[test]
    fn test_human_formatter_with_problems() {
        let formatter = HumanFormatter::with_colors(false);
        let results = vec![
            (PathBuf::from("test.yaml"), vec![
                Problem::new(10, 5, Level::Error, "line-length", "line too long"),
                Problem::with_suggestion(
                    15,
                    1,
                    Level::Warning,
                    "trailing-spaces",
                    "trailing whitespace",
                    "Remove trailing spaces"
                ),
            ]),
        ];

        let output = formatter.format_results(&results);
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines[0], "test.yaml");
        assert_eq!(lines[1], "  10:5: error line too long (line-length)");
        assert_eq!(lines[2], "  15:1: warning trailing whitespace (trailing-spaces)");
        assert_eq!(lines[3], "    Suggestion: Remove trailing spaces");
        assert_eq!(lines[5], "Found 1 error, 1 warning");
    }

    #[test]
    fn test_format_level_no_colors() {
        let formatter = HumanFormatter::with_colors(false);

        assert_eq!(formatter.format_level(&Level::Error), "error");
        assert_eq!(formatter.format_level(&Level::Warning), "warning");
        assert_eq!(formatter.format_level(&Level::Info), "info");
    }

    #[test]
    fn test_format_level_with_colors() {
        let formatter = HumanFormatter::with_colors(true);

        assert_eq!(formatter.format_level(&Level::Error), "\x1b[31merror\x1b[0m");
        assert_eq!(formatter.format_level(&Level::Warning), "\x1b[33mwarning\x1b[0m");
        assert_eq!(formatter.format_level(&Level::Info), "\x1b[36minfo\x1b[0m");
    }

    #[test]
    fn test_format_position() {
        let formatter = HumanFormatter::with_colors(false);
        assert_eq!(formatter.format_position(10, 5), "10:5");
    }

    #[test]
    fn test_format_rule() {
        let formatter = HumanFormatter::with_colors(false);
        assert_eq!(formatter.format_rule("test-rule"), "(test-rule)");
    }

    #[test]
    fn test_format_stats_no_problems() {
        let formatter = HumanFormatter::with_colors(false);
        let stats = LintStats::default();

        assert_eq!(formatter.format_stats(&stats), "No problems found");
    }

    #[test]
    fn test_format_stats_with_problems() {
        let formatter = HumanFormatter::with_colors(false);
        let stats = LintStats {
            total_files: 3,
            files_with_problems: 2,
            total_problems: 5,
            errors: 2,
            warnings: 2,
            info: 1,
        };

        assert_eq!(formatter.format_stats(&stats), "Found 2 errors, 2 warnings, 1 info");
    }

    #[test]
    fn test_format_stats_single_items() {
        let formatter = HumanFormatter::with_colors(false);
        let stats = LintStats {
            total_files: 1,
            files_with_problems: 1,
            total_problems: 1,
            errors: 1,
            warnings: 0,
            info: 0,
        };

        assert_eq!(formatter.format_stats(&stats), "Found 1 error");
    }
}
