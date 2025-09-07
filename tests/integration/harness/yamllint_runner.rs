use eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// Runner for executing yamllint and parsing its output
pub struct YamllintRunner {
    yamllint_path: PathBuf,
}

impl YamllintRunner {
    /// Create a new yamllint runner
    pub fn new() -> Result<Self> {
        let yamllint_path = Self::find_yamllint_binary()?;
        Ok(Self {
            yamllint_path,
        })
    }

    /// Run yamllint on a fixture with the specified configuration
    pub fn run_test(&self, config: &Path, fixture: &Path) -> Result<LintResult> {
        let start_time = Instant::now();

        let output = Command::new(&self.yamllint_path)
            .arg("-f")
            .arg("parsable")
            .arg("-c")
            .arg(config)
            .arg(fixture)
            .output()?;

        let execution_time = start_time.elapsed();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let problems = self.parse_yamllint_output(&stdout)?;

        Ok(LintResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
            problems,
            execution_time,
        })
    }

    /// Validate that yamllint is installed and get its version
    pub fn validate_installation(&self) -> Result<String> {
        let output = Command::new(&self.yamllint_path)
            .arg("--version")
            .output()?;

        if !output.status.success() {
            return Err(eyre::eyre!("yamllint is not properly installed"));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    /// Find the yamllint binary in the system PATH
    fn find_yamllint_binary() -> Result<PathBuf> {
        // Try common locations
        let candidates = vec!["yamllint", "/usr/local/bin/yamllint", "/usr/bin/yamllint"];

        for candidate in candidates {
            if let Ok(output) = Command::new(candidate).arg("--version").output() {
                if output.status.success() {
                    return Ok(PathBuf::from(candidate));
                }
            }
        }

        Err(eyre::eyre!(
            "yamllint not found. Please install yamllint: pip install yamllint"
        ))
    }

    /// Parse yamllint's parsable output format into structured problems
    fn parse_yamllint_output(&self, output: &str) -> Result<Vec<LintProblem>> {
        let mut problems = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // yamllint parsable format: file:line:column: [level] message (rule)
            if let Some(problem) = self.parse_yamllint_line(line)? {
                problems.push(problem);
            }
        }

        Ok(problems)
    }

    /// Parse a single line of yamllint output
    fn parse_yamllint_line(&self, line: &str) -> Result<Option<LintProblem>> {
        // Example: "/path/to/file.yaml:5:10: [error] line too long (101 > 80 characters) (line-length)"
        let parts: Vec<&str> = line.splitn(4, ':').collect();
        if parts.len() < 4 {
            return Ok(None);
        }

        let file_path = parts[0].to_string();
        let line_number: usize = parts[1].parse().unwrap_or(0);
        let column_number: usize = parts[2].parse().unwrap_or(0);

        let message_part = parts[3].trim();
        
        // Extract level, message, and rule from the message part
        if let Some(level_end) = message_part.find(']') {
            let level_start = message_part.find('[').unwrap_or(0) + 1;
            let level = message_part[level_start..level_end].to_string();
            
            let remaining = &message_part[level_end + 1..].trim();
            
            // Extract rule name from parentheses at the end
            let (message, rule) = if let Some(rule_start) = remaining.rfind('(') {
                let rule_end = remaining.rfind(')').unwrap_or(remaining.len());
                let rule = remaining[rule_start + 1..rule_end].to_string();
                let message = remaining[..rule_start].trim().to_string();
                (message, Some(rule))
            } else {
                (remaining.to_string(), None)
            };

            Ok(Some(LintProblem {
                file_path,
                line: line_number,
                column: column_number,
                level,
                message,
                rule_id: rule,
            }))
        } else {
            Ok(None)
        }
    }
}

/// Result of running a linter (yamllint or yl)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub problems: Vec<LintProblem>,
    pub execution_time: Duration,
}

/// A single linting problem found by a linter
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LintProblem {
    pub file_path: String,
    pub line: usize,
    pub column: usize,
    pub level: String,
    pub message: String,
    pub rule_id: Option<String>,
}

impl LintProblem {
    /// Create a new lint problem
    pub fn new(
        file_path: String,
        line: usize,
        column: usize,
        level: String,
        message: String,
        rule_id: Option<String>,
    ) -> Self {
        Self {
            file_path,
            line,
            column,
            level,
            message,
            rule_id,
        }
    }

    /// Check if this problem is equivalent to another (ignoring minor differences)
    pub fn is_equivalent(&self, other: &LintProblem) -> bool {
        self.line == other.line &&
        self.column == other.column &&
        self.level == other.level &&
        self.rule_id == other.rule_id
        // Note: We don't compare message text as it might have minor formatting differences
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yamllint_line() {
        let runner = YamllintRunner::new().unwrap();
        let line = "/path/to/file.yaml:5:10: [error] line too long (101 > 80 characters) (line-length)";
        
        let problem = runner.parse_yamllint_line(line).unwrap().unwrap();
        
        assert_eq!(problem.file_path, "/path/to/file.yaml");
        assert_eq!(problem.line, 5);
        assert_eq!(problem.column, 10);
        assert_eq!(problem.level, "error");
        assert_eq!(problem.message, "line too long (101 > 80 characters)");
        assert_eq!(problem.rule_id, Some("line-length".to_string()));
    }

    #[test]
    fn test_parse_yamllint_line_without_rule() {
        let runner = YamllintRunner::new().unwrap();
        let line = "/path/to/file.yaml:1:1: [error] syntax error";
        
        let problem = runner.parse_yamllint_line(line).unwrap().unwrap();
        
        assert_eq!(problem.file_path, "/path/to/file.yaml");
        assert_eq!(problem.line, 1);
        assert_eq!(problem.column, 1);
        assert_eq!(problem.level, "error");
        assert_eq!(problem.message, "syntax error");
        assert_eq!(problem.rule_id, None);
    }

    #[test]
    fn test_lint_problem_equivalence() {
        let problem1 = LintProblem::new(
            "file.yaml".to_string(),
            5,
            10,
            "error".to_string(),
            "line too long".to_string(),
            Some("line-length".to_string()),
        );

        let problem2 = LintProblem::new(
            "file.yaml".to_string(),
            5,
            10,
            "error".to_string(),
            "line too long (different message format)".to_string(),
            Some("line-length".to_string()),
        );

        assert!(problem1.is_equivalent(&problem2));
    }
}
