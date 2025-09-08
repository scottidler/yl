use super::{LintProblem, LintResult};
use eyre::Result;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

/// Runner for executing yl and parsing its output
pub struct YlRunner {
    yl_binary: PathBuf,
}

/// Enhanced mode configuration for yl
#[derive(Debug, Clone)]
pub enum EnhancedMode {
    /// yamllint-compatible mode (no enhanced features)
    Compatible,
    /// Full yl feature set enabled
    Enhanced,
    /// Specific features enabled
    Selective(Vec<String>),
}

impl YlRunner {
    /// Create a new yl runner
    pub fn new() -> Result<Self> {
        let yl_binary = Self::find_yl_binary()?;
        Ok(Self { yl_binary })
    }

    /// Run yl on a fixture with the specified configuration
    pub fn run_test(&self, config: &Path, fixture: &Path) -> Result<LintResult> {
        self.run_with_mode(config, fixture, EnhancedMode::Compatible)
    }

    /// Run yl with enhanced features enabled
    pub fn run_enhanced_test(&self, config: &Path, fixture: &Path, mode: EnhancedMode) -> Result<LintResult> {
        self.run_with_mode(config, fixture, mode)
    }

    /// Validate that yl is installed and get its version
    pub fn validate_installation(&self) -> Result<String> {
        let output = Command::new(&self.yl_binary).arg("--version").output()?;

        if !output.status.success() {
            return Err(eyre::eyre!("yl binary is not properly built or accessible"));
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    /// Run yl with the specified mode and configuration
    fn run_with_mode(&self, config: &Path, fixture: &Path, mode: EnhancedMode) -> Result<LintResult> {
        let start_time = Instant::now();

        let mut cmd = Command::new(&self.yl_binary);
        cmd.arg("--config").arg(config);
        cmd.arg("--format").arg("parsable");

        // Configure enhanced mode
        match mode {
            EnhancedMode::Compatible => {
                cmd.arg("--yamllint-compatible");
            }
            EnhancedMode::Enhanced => {
                cmd.arg("--enhanced-features");
            }
            EnhancedMode::Selective(features) => {
                for feature in features {
                    cmd.arg("--enable-feature").arg(feature);
                }
            }
        }

        cmd.arg(fixture);

        let output = cmd.output()?;
        let execution_time = start_time.elapsed();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let problems = self.parse_yl_output(&stdout)?;

        Ok(LintResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout,
            stderr,
            problems,
            execution_time,
        })
    }

    /// Find the yl binary
    fn find_yl_binary() -> Result<PathBuf> {
        // Try to find the yl binary in target directory
        let candidates = vec!["target/release/yl", "target/debug/yl", "./yl", "yl"];

        for candidate in candidates {
            let path = PathBuf::from(candidate);
            if path.exists() {
                return Ok(path);
            }
        }

        Err(eyre::eyre!(
            "yl binary not found. Please build with: cargo build --release"
        ))
    }

    /// Parse yl's parsable output format into structured problems
    fn parse_yl_output(&self, output: &str) -> Result<Vec<LintProblem>> {
        let mut problems = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            // yl should use the same parsable format as yamllint for compatibility
            if let Some(problem) = self.parse_yl_line(line)? {
                problems.push(problem);
            }
        }

        Ok(problems)
    }

    /// Parse a single line of yl output (should match yamllint format)
    fn parse_yl_line(&self, line: &str) -> Result<Option<LintProblem>> {
        // yl should output in yamllint-compatible format:
        // "/path/to/file.yaml:5:10: [error] line too long (101 > 80 characters) (line-length)"
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

impl Default for YlRunner {
    fn default() -> Self {
        Self::new().expect("Failed to create yl runner")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yl_line() {
        let runner = YlRunner::new().unwrap();
        let line = "/path/to/file.yaml:5:10: [error] line too long (101 > 80 characters) (line-length)";

        let problem = runner.parse_yl_line(line).unwrap().unwrap();

        assert_eq!(problem.file_path, "/path/to/file.yaml");
        assert_eq!(problem.line, 5);
        assert_eq!(problem.column, 10);
        assert_eq!(problem.level, "error");
        assert_eq!(problem.message, "line too long (101 > 80 characters)");
        assert_eq!(problem.rule_id, Some("line-length".to_string()));
    }

    #[test]
    fn test_enhanced_mode_variants() {
        let compatible = EnhancedMode::Compatible;
        let enhanced = EnhancedMode::Enhanced;
        let selective = EnhancedMode::Selective(vec!["inline-comments".to_string()]);

        // Test that modes can be created and cloned
        let _compatible_clone = compatible.clone();
        let _enhanced_clone = enhanced.clone();
        let _selective_clone = selective.clone();
    }
}
