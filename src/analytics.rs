//! Performance analytics and optimization for YAML linting
//!
//! This module provides comprehensive analytics for linting performance,
//! including rule execution times, problem statistics, and optimization suggestions.

pub use crate::analytics_types::{LintAnalytics};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Level;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn test_analytics_creation() {
        let analytics = LintAnalytics::new();
        assert_eq!(analytics.total_files_processed, 0);
        assert_eq!(analytics.total_problems_found, 0);
        assert!(analytics.rule_performance.is_empty());
    }

    #[test]
    fn test_suggest_optimizations() {
        let mut analytics = LintAnalytics::new();

        // Simulate some processing time to trigger suggestions
        std::thread::sleep(Duration::from_millis(50));

        // Add some performance data
        let file_path = PathBuf::from("test.yaml");
        analytics.file_processing_times.insert(file_path.clone(), Duration::from_millis(150));
        analytics.total_files_processed = 15;

        // Add a slow rule
        use crate::analytics_types::RulePerformanceMetrics;
        let slow_rule = RulePerformanceMetrics {
            rule_id: "slow-rule".to_string(),
            total_execution_time: Duration::from_millis(200),
            execution_count: 1,
            average_execution_time: Duration::from_millis(200),
            max_execution_time: Duration::from_millis(200),
            min_execution_time: Duration::from_millis(200),
            slowest_files: vec![(file_path, Duration::from_millis(200))],
        };
        analytics.rule_performance.insert("slow-rule".to_string(), slow_rule);

        let suggestions = analytics.suggest_optimizations();
        assert!(!suggestions.is_empty());
    }

    #[test]
    fn test_generate_report() {
        let analytics = LintAnalytics::new();
        let report = analytics.generate_report();

        assert_eq!(report.session_info.files_processed, 0);
        assert_eq!(report.session_info.total_problems, 0);
        assert!(report.rule_metrics.is_empty());
    }

    #[test]
    fn test_export_json() {
        let analytics = LintAnalytics::new();
        let json = analytics.export_json().unwrap();

        assert!(json.contains("session_info"));
        assert!(json.contains("rule_metrics"));
        assert!(json.contains("problem_statistics"));
    }
}
