use super::{LintProblem, LintResult};
use serde::{Deserialize, Serialize};

/// Compares results between yamllint and yl for compatibility validation
pub struct ResultComparator {
    tolerance: ComparisonTolerance,
}

/// Configuration for comparison tolerance
#[derive(Debug, Clone)]
pub struct ComparisonTolerance {
    /// Allow minor differences in message formatting
    pub message_formatting: bool,
    /// Maximum acceptable difference in problem count
    pub max_problem_count_diff: usize,
}

/// Result of comparing two lint results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub is_compatible: bool,
    pub differences: Vec<Difference>,
    pub severity: CompatibilitySeverity,
    pub summary: String,
}

/// Severity level of compatibility differences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompatibilitySeverity {
    /// Perfect match - identical results
    Identical,
    /// Minor differences that are acceptable (formatting, etc.)
    Acceptable,
    /// Significant differences that may indicate issues
    Concerning,
    /// Major differences that break compatibility
    Incompatible,
}

/// Specific difference found between results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    pub diff_type: DifferenceType,
    pub description: String,
    pub yamllint_value: Option<String>,
    pub yl_value: Option<String>,
}

/// Type of difference between results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifferenceType {
    ExitCode,
    ProblemCount,
    MissingProblem,
    ExtraProblem,
    ProblemLocation,
    ProblemLevel,
    ProblemMessage,
    RuleId,
    ExecutionTime,
}

/// Expected behavior for enhanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedExpectation {
    pub should_respect_inline_comments: bool,
    pub should_preserve_formatting: bool,
    pub should_apply_project_ignores: bool,
    pub expected_problem_count: Option<usize>,
    pub expected_rules_triggered: Vec<String>,
    pub expected_rules_suppressed: Vec<String>,
}

/// Result of validating enhanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub failures: Vec<String>,
    pub summary: String,
}

impl ResultComparator {
    /// Create a new result comparator with default tolerance
    pub fn new() -> Self {
        Self {
            tolerance: ComparisonTolerance {
                message_formatting: true,
                max_problem_count_diff: 0,
            },
        }
    }

    /// Compare yamllint and yl results for compatibility
    pub fn compare_compatibility(&self, yamllint: &LintResult, yl: &LintResult) -> ComparisonResult {
        let mut differences = Vec::new();

        // Compare exit codes
        if yamllint.exit_code != yl.exit_code {
            differences.push(Difference {
                diff_type: DifferenceType::ExitCode,
                description: "Exit codes differ".to_string(),
                yamllint_value: Some(yamllint.exit_code.to_string()),
                yl_value: Some(yl.exit_code.to_string()),
            });
        }

        // Compare problem counts
        let problem_count_diff = if yamllint.problems.len() > yl.problems.len() {
            yamllint.problems.len() - yl.problems.len()
        } else {
            yl.problems.len() - yamllint.problems.len()
        };

        if problem_count_diff > self.tolerance.max_problem_count_diff {
            differences.push(Difference {
                diff_type: DifferenceType::ProblemCount,
                description: format!("Problem count differs by {}", problem_count_diff),
                yamllint_value: Some(yamllint.problems.len().to_string()),
                yl_value: Some(yl.problems.len().to_string()),
            });
        }

        // Compare individual problems
        self.compare_problems(&yamllint.problems, &yl.problems, &mut differences);

        // Determine severity and compatibility
        let severity = self.determine_severity(&differences);
        let is_compatible = matches!(
            severity,
            CompatibilitySeverity::Identical | CompatibilitySeverity::Acceptable
        );

        let summary = self.generate_summary(&differences, &severity);

        ComparisonResult {
            is_compatible,
            differences,
            severity,
            summary,
        }
    }

    /// Validate yl-specific enhanced features
    pub fn validate_enhanced_features(&self, result: &LintResult, expected: &EnhancedExpectation) -> ValidationResult {
        let mut failures = Vec::new();

        // Check expected problem count
        if let Some(expected_count) = expected.expected_problem_count {
            if result.problems.len() != expected_count {
                failures.push(format!(
                    "Expected {} problems, found {}",
                    expected_count,
                    result.problems.len()
                ));
            }
        }

        // Check that expected rules were triggered
        let triggered_rules: Vec<String> = result.problems.iter().filter_map(|p| p.rule_id.clone()).collect();

        for expected_rule in &expected.expected_rules_triggered {
            if !triggered_rules.contains(expected_rule) {
                failures.push(format!("Expected rule '{}' was not triggered", expected_rule));
            }
        }

        // Check that expected rules were suppressed
        for suppressed_rule in &expected.expected_rules_suppressed {
            if triggered_rules.contains(suppressed_rule) {
                failures.push(format!("Rule '{}' should have been suppressed", suppressed_rule));
            }
        }

        let is_valid = failures.is_empty();
        let summary = if is_valid {
            "All enhanced features working as expected".to_string()
        } else {
            format!("Enhanced feature validation failed: {} issues", failures.len())
        };

        ValidationResult {
            is_valid,
            failures,
            summary,
        }
    }

    /// Compare individual problems between yamllint and yl
    fn compare_problems(
        &self,
        yamllint_problems: &[LintProblem],
        yl_problems: &[LintProblem],
        differences: &mut Vec<Difference>,
    ) {
        // Find problems that exist in yamllint but not in yl
        for yamllint_problem in yamllint_problems {
            if !self.find_equivalent_problem(yamllint_problem, yl_problems) {
                differences.push(Difference {
                    diff_type: DifferenceType::MissingProblem,
                    description: format!(
                        "Problem missing in yl: {}:{} {}",
                        yamllint_problem.line,
                        yamllint_problem.column,
                        yamllint_problem.rule_id.as_deref().unwrap_or("unknown")
                    ),
                    yamllint_value: Some(format!("{:?}", yamllint_problem)),
                    yl_value: None,
                });
            }
        }

        // Find problems that exist in yl but not in yamllint
        for yl_problem in yl_problems {
            if !self.find_equivalent_problem(yl_problem, yamllint_problems) {
                differences.push(Difference {
                    diff_type: DifferenceType::ExtraProblem,
                    description: format!(
                        "Extra problem in yl: {}:{} {}",
                        yl_problem.line,
                        yl_problem.column,
                        yl_problem.rule_id.as_deref().unwrap_or("unknown")
                    ),
                    yamllint_value: None,
                    yl_value: Some(format!("{:?}", yl_problem)),
                });
            }
        }
    }

    /// Find an equivalent problem in the given list
    fn find_equivalent_problem(&self, target: &LintProblem, problems: &[LintProblem]) -> bool {
        problems.iter().any(|p| self.are_problems_equivalent(target, p))
    }

    /// Check if two problems are equivalent (considering tolerance settings)
    fn are_problems_equivalent(&self, p1: &LintProblem, p2: &LintProblem) -> bool {
        // Must match on location, level, and rule
        p1.line == p2.line &&
        p1.column == p2.column &&
        p1.level == p2.level &&
        p1.rule_id == p2.rule_id &&
        // Message can differ if tolerance allows it
        (self.tolerance.message_formatting || p1.message == p2.message)
    }

    /// Determine the severity of differences
    fn determine_severity(&self, differences: &[Difference]) -> CompatibilitySeverity {
        if differences.is_empty() {
            return CompatibilitySeverity::Identical;
        }

        let has_critical = differences.iter().any(|d| {
            matches!(
                d.diff_type,
                DifferenceType::ExitCode
                    | DifferenceType::ProblemCount
                    | DifferenceType::MissingProblem
                    | DifferenceType::ExtraProblem
            )
        });

        let has_concerning = differences
            .iter()
            .any(|d| matches!(d.diff_type, DifferenceType::ProblemLevel | DifferenceType::RuleId));

        if has_critical {
            CompatibilitySeverity::Incompatible
        } else if has_concerning {
            CompatibilitySeverity::Concerning
        } else {
            CompatibilitySeverity::Acceptable
        }
    }

    /// Generate a human-readable summary of the comparison
    fn generate_summary(&self, differences: &[Difference], severity: &CompatibilitySeverity) -> String {
        match severity {
            CompatibilitySeverity::Identical => "Results are identical - perfect compatibility".to_string(),
            CompatibilitySeverity::Acceptable => {
                format!("Results are compatible with {} minor differences", differences.len())
            }
            CompatibilitySeverity::Concerning => {
                format!(
                    "Results have {} concerning differences that should be investigated",
                    differences.len()
                )
            }
            CompatibilitySeverity::Incompatible => {
                format!(
                    "Results are incompatible with {} critical differences",
                    differences.len()
                )
            }
        }
    }
}

impl Default for ResultComparator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ComparisonTolerance {
    fn default() -> Self {
        Self {
            message_formatting: true,
            max_problem_count_diff: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_results() {
        let comparator = ResultComparator::new();

        let problem = LintProblem {
            file_path: "test.yaml".to_string(),
            line: 5,
            column: 10,
            level: "error".to_string(),
            message: "line too long".to_string(),
            rule_id: Some("line-length".to_string()),
        };

        let result1 = LintResult {
            exit_code: 1,
            stdout: "output".to_string(),
            stderr: "".to_string(),
            problems: vec![problem.clone()],
            execution_time: std::time::Duration::from_millis(100),
        };

        let result2 = result1.clone();

        let comparison = comparator.compare_compatibility(&result1, &result2);
        assert!(comparison.is_compatible);
        assert_eq!(comparison.severity, CompatibilitySeverity::Identical);
    }

    #[test]
    fn test_message_formatting_tolerance() {
        let comparator = ResultComparator::new();

        let problem1 = LintProblem {
            file_path: "test.yaml".to_string(),
            line: 5,
            column: 10,
            level: "error".to_string(),
            message: "line too long (80 chars)".to_string(),
            rule_id: Some("line-length".to_string()),
        };

        let problem2 = LintProblem {
            file_path: "test.yaml".to_string(),
            line: 5,
            column: 10,
            level: "error".to_string(),
            message: "line too long (80 characters)".to_string(), // Different message format
            rule_id: Some("line-length".to_string()),
        };

        assert!(comparator.are_problems_equivalent(&problem1, &problem2));
    }

    #[test]
    fn test_enhanced_feature_validation() {
        let comparator = ResultComparator::new();

        let result = LintResult {
            exit_code: 0,
            stdout: "".to_string(),
            stderr: "".to_string(),
            problems: vec![],
            execution_time: std::time::Duration::from_millis(50),
        };

        let expectation = EnhancedExpectation {
            should_respect_inline_comments: true,
            should_preserve_formatting: true,
            should_apply_project_ignores: true,
            expected_problem_count: Some(0),
            expected_rules_triggered: vec![],
            expected_rules_suppressed: vec!["line-length".to_string()],
        };

        let validation = comparator.validate_enhanced_features(&result, &expectation);
        assert!(validation.is_valid);
    }
}
