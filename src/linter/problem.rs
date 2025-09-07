use serde::{Deserialize, Serialize};

/// Represents the severity level of a linting problem
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Level {
    /// Informational message
    Info,
    /// Warning that doesn't prevent success
    Warning,
    /// Error that should cause failure
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Level::Info => write!(f, "info"),
            Level::Warning => write!(f, "warning"),
            Level::Error => write!(f, "error"),
        }
    }
}

/// Represents a linting problem found in a YAML file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Problem {
    /// Line number where the problem was found (1-based)
    pub line: usize,
    /// Column number where the problem was found (1-based)
    pub column: usize,
    /// Severity level of the problem
    pub level: Level,
    /// ID of the rule that detected the problem
    pub rule: String,
    /// Human-readable description of the problem
    pub message: String,
    /// Optional suggestion for fixing the problem
    pub suggestion: Option<String>,
}

impl Problem {
    /// Create a new problem
    pub fn new(
        line: usize,
        column: usize,
        level: Level,
        rule: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            line,
            column,
            level,
            rule: rule.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    /// Create a new problem with a suggestion
    pub fn with_suggestion(
        line: usize,
        column: usize,
        level: Level,
        rule: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) -> Self {
        Self {
            line,
            column,
            level,
            rule: rule.into(),
            message: message.into(),
            suggestion: Some(suggestion.into()),
        }
    }

    /// Get a formatted message including the rule ID
    pub fn formatted_message(&self) -> String {
        format!("{} ({})", self.message, self.rule)
    }
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.column, self.formatted_message())
    }
}

impl PartialOrd for Problem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Problem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.line
            .cmp(&other.line)
            .then_with(|| self.column.cmp(&other.column))
            .then_with(|| self.level.cmp(&other.level))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_display() {
        assert_eq!(Level::Info.to_string(), "info");
        assert_eq!(Level::Warning.to_string(), "warning");
        assert_eq!(Level::Error.to_string(), "error");
    }

    #[test]
    fn test_level_ordering() {
        assert!(Level::Info < Level::Warning);
        assert!(Level::Warning < Level::Error);
        assert!(Level::Info < Level::Error);
    }

    #[test]
    fn test_problem_creation() {
        let problem = Problem::new(10, 5, Level::Error, "test-rule", "Test message");
        
        assert_eq!(problem.line, 10);
        assert_eq!(problem.column, 5);
        assert_eq!(problem.level, Level::Error);
        assert_eq!(problem.rule, "test-rule");
        assert_eq!(problem.message, "Test message");
        assert_eq!(problem.suggestion, None);
    }

    #[test]
    fn test_problem_with_suggestion() {
        let problem = Problem::with_suggestion(
            15, 
            8, 
            Level::Warning, 
            "style-rule", 
            "Style issue", 
            "Fix suggestion"
        );
        
        assert_eq!(problem.line, 15);
        assert_eq!(problem.column, 8);
        assert_eq!(problem.level, Level::Warning);
        assert_eq!(problem.rule, "style-rule");
        assert_eq!(problem.message, "Style issue");
        assert_eq!(problem.suggestion, Some("Fix suggestion".to_string()));
    }

    #[test]
    fn test_problem_formatted_message() {
        let problem = Problem::new(1, 1, Level::Error, "test", "message");
        assert_eq!(problem.formatted_message(), "message (test)");
    }

    #[test]
    fn test_problem_display() {
        let problem = Problem::new(10, 5, Level::Error, "test-rule", "Test message");
        assert_eq!(problem.to_string(), "10:5: Test message (test-rule)");
    }

    #[test]
    fn test_problem_ordering() {
        let p1 = Problem::new(1, 1, Level::Error, "rule", "msg");
        let p2 = Problem::new(1, 2, Level::Error, "rule", "msg");
        let p3 = Problem::new(2, 1, Level::Error, "rule", "msg");
        let p4 = Problem::new(1, 1, Level::Warning, "rule", "msg");
        
        assert!(p1 < p2); // Same line, different column
        assert!(p1 < p3); // Different line
        assert!(p4 < p1); // Same position, different level
    }

    #[test]
    fn test_problem_equality() {
        let p1 = Problem::new(1, 1, Level::Error, "rule", "msg");
        let p2 = Problem::new(1, 1, Level::Error, "rule", "msg");
        let p3 = Problem::new(1, 1, Level::Error, "rule", "different msg");
        
        assert_eq!(p1, p2);
        assert_ne!(p1, p3);
    }

    #[test]
    fn test_serde_serialization() {
        let problem = Problem::with_suggestion(
            10, 
            5, 
            Level::Warning, 
            "test-rule", 
            "Test message",
            "Fix it"
        );
        
        let serialized = serde_yaml::to_string(&problem).expect("Failed to serialize");
        let deserialized: Problem = serde_yaml::from_str(&serialized).expect("Failed to deserialize");
        
        assert_eq!(problem, deserialized);
    }
}
