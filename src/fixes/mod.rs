use crate::linter::Problem;
use eyre::Result;
use std::collections::HashMap;

/// Trait for implementing automatic fixes for linting problems
pub trait AutoFix: Send + Sync {
    /// Check if this fix can handle the given problem
    fn can_fix(&self, problem: &Problem) -> bool;

    /// Apply the fix to the content and return the fixed content
    fn apply_fix(&self, content: &str, problem: &Problem) -> Result<String>;
}

/// Engine for applying automatic fixes to YAML content
pub struct FixEngine {
    fixes: HashMap<String, Box<dyn AutoFix>>,
}

impl FixEngine {
    /// Create a new fix engine with default fixes
    pub fn new() -> Self {
        let mut engine = Self { fixes: HashMap::new() };

        // Register default fixes
        engine.register_fix("trailing-spaces", Box::new(TrailingSpacesFix));
        engine.register_fix("new-line-at-end-of-file", Box::new(NewLineAtEndOfFileFix));
        engine.register_fix("empty-lines", Box::new(EmptyLinesFix));

        engine
    }

    /// Register a fix for a specific rule
    pub fn register_fix(&mut self, rule_id: &str, fix: Box<dyn AutoFix>) {
        self.fixes.insert(rule_id.to_string(), fix);
    }

    /// Apply fixes to content for the given problems
    pub fn fix_problems(&self, content: &str, problems: &[Problem]) -> Result<String> {
        let mut fixed_content = content.to_string();

        // Group problems by rule and sort by line number (reverse order to maintain positions)
        let mut rule_problems: HashMap<String, Vec<&Problem>> = HashMap::new();
        for problem in problems {
            rule_problems.entry(problem.rule.clone()).or_default().push(problem);
        }

        // Apply fixes for each rule in a consistent order
        let mut rule_ids: Vec<_> = rule_problems.keys().collect();
        rule_ids.sort(); // Ensure consistent ordering

        for rule_id in rule_ids {
            if let Some(fix) = self.fixes.get(rule_id) {
                let rule_problems = rule_problems.get(rule_id).unwrap();
                // Sort problems in reverse line order to maintain positions when fixing
                let mut sorted_problems = rule_problems.clone();
                sorted_problems.sort_by(|a, b| b.line.cmp(&a.line));

                for problem in sorted_problems {
                    if fix.can_fix(problem) {
                        fixed_content = fix.apply_fix(&fixed_content, problem)?;
                    }
                }
            }
        }

        Ok(fixed_content)
    }
}

impl Default for FixEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Fix for trailing spaces
pub struct TrailingSpacesFix;

impl AutoFix for TrailingSpacesFix {
    fn can_fix(&self, problem: &Problem) -> bool {
        problem.rule == "trailing-spaces"
    }

    fn apply_fix(&self, content: &str, problem: &Problem) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut fixed_lines = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            let line_number = i + 1;
            if line_number == problem.line {
                // Remove trailing whitespace from this line
                fixed_lines.push(line.trim_end());
            } else {
                fixed_lines.push(*line);
            }
        }

        let mut result = fixed_lines.join("\n");

        // Preserve the original ending - if the original content ended with a newline, keep it
        if content.ends_with('\n') {
            result.push('\n');
        }

        Ok(result)
    }
}

/// Fix for missing newline at end of file
pub struct NewLineAtEndOfFileFix;

impl AutoFix for NewLineAtEndOfFileFix {
    fn can_fix(&self, problem: &Problem) -> bool {
        problem.rule == "new-line-at-end-of-file"
    }

    fn apply_fix(&self, content: &str, _problem: &Problem) -> Result<String> {
        if content.is_empty() {
            return Ok(content.to_string());
        }

        if content.ends_with('\n') {
            Ok(content.to_string())
        } else {
            Ok(format!("{content}\n"))
        }
    }
}

/// Fix for empty lines issues
pub struct EmptyLinesFix;

impl AutoFix for EmptyLinesFix {
    fn can_fix(&self, problem: &Problem) -> bool {
        problem.rule == "empty-lines"
            && (problem.message.contains("too many blank lines")
                || problem.message.contains("at beginning")
                || problem.message.contains("at end"))
    }

    fn apply_fix(&self, content: &str, problem: &Problem) -> Result<String> {
        let lines: Vec<&str> = content.lines().collect();

        if problem.message.contains("at beginning") {
            // Remove empty lines at the beginning
            let mut start_index = 0;
            for (i, line) in lines.iter().enumerate() {
                if !line.trim().is_empty() {
                    start_index = i;
                    break;
                }
            }
            return Ok(lines[start_index..].join("\n"));
        }

        if problem.message.contains("at end") {
            // Remove excessive empty lines at the end
            let mut end_index = lines.len();
            let mut empty_count = 0;

            for (i, line) in lines.iter().enumerate().rev() {
                if line.trim().is_empty() {
                    empty_count += 1;
                } else {
                    end_index = i + 1;
                    break;
                }
            }

            // Keep at most one empty line at the end
            if empty_count > 1 {
                let mut result = lines[..end_index].to_vec();
                if end_index < lines.len() {
                    result.push(""); // Add one empty line
                }
                return Ok(result.join("\n"));
            }
        }

        if problem.message.contains("too many blank lines") {
            // Reduce consecutive empty lines to maximum of 2
            let mut fixed_lines = Vec::new();
            let mut consecutive_empty = 0;

            for line in lines {
                if line.trim().is_empty() {
                    consecutive_empty += 1;
                    if consecutive_empty <= 2 {
                        fixed_lines.push(line);
                    }
                } else {
                    consecutive_empty = 0;
                    fixed_lines.push(line);
                }
            }

            return Ok(fixed_lines.join("\n"));
        }

        Ok(content.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Level;

    #[test]
    fn test_fix_engine_creation() {
        let engine = FixEngine::new();
        assert!(!engine.fixes.is_empty());
    }

    #[test]
    fn test_trailing_spaces_fix() {
        let fix = TrailingSpacesFix;
        let problem = Problem::new(2, 10, Level::Error, "trailing-spaces", "trailing whitespace");
        let content = "line1\nline2   \nline3";

        assert!(fix.can_fix(&problem));

        let fixed = fix.apply_fix(content, &problem).unwrap();
        assert_eq!(fixed, "line1\nline2\nline3");
    }

    #[test]
    fn test_newline_at_end_fix() {
        let fix = NewLineAtEndOfFileFix;
        let problem = Problem::new(1, 5, Level::Error, "new-line-at-end-of-file", "missing newline");
        let content = "line1\nline2";

        assert!(fix.can_fix(&problem));

        let fixed = fix.apply_fix(content, &problem).unwrap();
        assert_eq!(fixed, "line1\nline2\n");
    }

    #[test]
    fn test_empty_lines_fix_consecutive() {
        let fix = EmptyLinesFix;
        let problem = Problem::new(3, 1, Level::Error, "empty-lines", "too many blank lines (3 > 2)");
        let content = "line1\n\n\n\nline2";

        assert!(fix.can_fix(&problem));

        let fixed = fix.apply_fix(content, &problem).unwrap();
        assert_eq!(fixed, "line1\n\n\nline2");
    }

    #[test]
    fn test_empty_lines_fix_at_beginning() {
        let fix = EmptyLinesFix;
        let problem = Problem::new(1, 1, Level::Error, "empty-lines", "too many blank lines at beginning");
        let content = "\n\nline1\nline2";

        assert!(fix.can_fix(&problem));

        let fixed = fix.apply_fix(content, &problem).unwrap();
        assert_eq!(fixed, "line1\nline2");
    }

    #[test]
    fn test_fix_engine_apply_multiple() {
        let engine = FixEngine::new();
        let problems = vec![
            Problem::new(1, 8, Level::Error, "trailing-spaces", "trailing whitespace"),
            Problem::new(3, 1, Level::Error, "new-line-at-end-of-file", "missing newline"),
        ];
        let content = "line1   \nline2\nline3";

        let fixed = engine.fix_problems(content, &problems).unwrap();
        assert_eq!(fixed, "line1\nline2\nline3\n");
    }

    #[test]
    fn test_fix_engine_no_applicable_fixes() {
        let engine = FixEngine::new();
        let problems = vec![Problem::new(1, 5, Level::Error, "unknown-rule", "some error")];
        let content = "line1\nline2";

        let fixed = engine.fix_problems(content, &problems).unwrap();
        assert_eq!(fixed, content); // Should be unchanged
    }
}
