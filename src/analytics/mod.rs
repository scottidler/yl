use eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// Analytics collector for linting performance and usage patterns
pub struct LintAnalytics {
    /// Performance metrics for each rule
    pub rule_performance: HashMap<String, RulePerformanceMetrics>,

    /// File processing times
    pub file_processing_times: HashMap<PathBuf, Duration>,

    /// Problem statistics by rule
    pub problem_statistics: HashMap<String, ProblemStats>,

    /// Session start time
    session_start: Instant,

    /// Total files processed
    total_files_processed: usize,

    /// Total problems found
    total_problems_found: usize,
}

/// Performance metrics for a specific rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulePerformanceMetrics {
    /// Rule identifier
    pub rule_id: String,

    /// Total execution time across all files
    pub total_execution_time: Duration,

    /// Number of times the rule was executed
    pub execution_count: usize,

    /// Average execution time per file
    pub average_execution_time: Duration,

    /// Maximum execution time for a single file
    pub max_execution_time: Duration,

    /// Minimum execution time for a single file
    pub min_execution_time: Duration,

    /// Files where this rule took the longest
    pub slowest_files: Vec<(PathBuf, Duration)>,
}

/// Statistics about problems found by a rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemStats {
    /// Rule identifier
    pub rule_id: String,

    /// Total number of problems found
    pub total_problems: usize,

    /// Problems by severity level
    pub problems_by_level: HashMap<String, usize>,

    /// Files with problems
    pub files_with_problems: usize,

    /// Average problems per file (for files with problems)
    pub average_problems_per_file: f64,

    /// Most common problem messages
    pub common_messages: HashMap<String, usize>,
}

/// Complete analytics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    /// Session information
    pub session_info: SessionInfo,

    /// Performance summary
    pub performance_summary: PerformanceSummary,

    /// Rule performance details
    pub rule_performance: Vec<RulePerformanceMetrics>,

    /// Problem statistics
    pub problem_statistics: Vec<ProblemStats>,

    /// Optimization suggestions
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    /// Session duration
    pub duration: Duration,

    /// Total files processed
    pub total_files: usize,

    /// Total problems found
    pub total_problems: usize,

    /// Average processing time per file
    pub average_file_time: Duration,

    /// Timestamp when session started
    pub started_at: String,
}

/// Performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    /// Total processing time
    pub total_time: Duration,

    /// Time spent on rule execution
    pub rule_execution_time: Duration,

    /// Time spent on file I/O
    pub file_io_time: Duration,

    /// Slowest rules (top 5)
    pub slowest_rules: Vec<(String, Duration)>,

    /// Fastest rules (top 5)
    pub fastest_rules: Vec<(String, Duration)>,

    /// Files that took longest to process
    pub slowest_files: Vec<(PathBuf, Duration)>,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// Type of optimization
    pub suggestion_type: OptimizationType,

    /// Description of the suggestion
    pub description: String,

    /// Potential impact (estimated time savings)
    pub potential_impact: Duration,

    /// Priority level
    pub priority: Priority,

    /// Specific recommendations
    pub recommendations: Vec<String>,
}

/// Type of optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    RuleConfiguration,
    FileFiltering,
    ParallelProcessing,
    RuleOrdering,
    CachingStrategy,
}

/// Priority level for suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl LintAnalytics {
    /// Create a new analytics collector
    pub fn new() -> Self {
        Self {
            rule_performance: HashMap::new(),
            file_processing_times: HashMap::new(),
            problem_statistics: HashMap::new(),
            session_start: Instant::now(),
            total_files_processed: 0,
            total_problems_found: 0,
        }
    }

    /// Record rule execution time
    #[allow(dead_code)]
    pub fn record_rule_execution(&mut self, rule_id: &str, file_path: &PathBuf, execution_time: Duration) {
        let metrics = self.rule_performance.entry(rule_id.to_string()).or_insert_with(|| {
            RulePerformanceMetrics {
                rule_id: rule_id.to_string(),
                total_execution_time: Duration::new(0, 0),
                execution_count: 0,
                average_execution_time: Duration::new(0, 0),
                max_execution_time: Duration::new(0, 0),
                min_execution_time: Duration::from_secs(u64::MAX),
                slowest_files: Vec::new(),
            }
        });

        metrics.total_execution_time += execution_time;
        metrics.execution_count += 1;
        metrics.average_execution_time = metrics.total_execution_time / metrics.execution_count as u32;

        if execution_time > metrics.max_execution_time {
            metrics.max_execution_time = execution_time;
        }

        if execution_time < metrics.min_execution_time {
            metrics.min_execution_time = execution_time;
        }

        // Track slowest files (keep top 5)
        metrics.slowest_files.push((file_path.clone(), execution_time));
        metrics.slowest_files.sort_by(|a, b| b.1.cmp(&a.1));
        metrics.slowest_files.truncate(5);
    }


    /// Generate a comprehensive analytics report
    pub fn generate_report(&self) -> AnalyticsReport {
        let session_duration = self.session_start.elapsed();

        let session_info = SessionInfo {
            duration: session_duration,
            total_files: self.total_files_processed,
            total_problems: self.total_problems_found,
            average_file_time: if self.total_files_processed > 0 {
                session_duration / self.total_files_processed as u32
            } else {
                Duration::new(0, 0)
            },
            started_at: chrono::Utc::now().to_rfc3339(),
        };

        let performance_summary = self.generate_performance_summary();
        let optimization_suggestions = self.suggest_optimizations();

        AnalyticsReport {
            session_info,
            performance_summary,
            rule_performance: self.rule_performance.values().cloned().collect(),
            problem_statistics: self.problem_statistics.values().cloned().collect(),
            optimization_suggestions,
        }
    }

    /// Generate performance summary
    fn generate_performance_summary(&self) -> PerformanceSummary {
        let total_rule_time: Duration = self.rule_performance.values()
            .map(|m| m.total_execution_time)
            .sum();

        let total_file_time: Duration = self.file_processing_times.values().sum();

        // Get slowest and fastest rules
        let mut rule_times: Vec<(String, Duration)> = self.rule_performance.iter()
            .map(|(id, metrics)| (id.clone(), metrics.average_execution_time))
            .collect();
        rule_times.sort_by(|a, b| b.1.cmp(&a.1));

        let slowest_rules = rule_times.iter().take(5).cloned().collect();
        let fastest_rules = rule_times.iter().rev().take(5).cloned().collect();

        // Get slowest files
        let mut file_times: Vec<(PathBuf, Duration)> = self.file_processing_times.iter()
            .map(|(path, time)| (path.clone(), *time))
            .collect();
        file_times.sort_by(|a, b| b.1.cmp(&a.1));
        let slowest_files = file_times.into_iter().take(10).collect();

        PerformanceSummary {
            total_time: self.session_start.elapsed(),
            rule_execution_time: total_rule_time,
            file_io_time: total_file_time - total_rule_time,
            slowest_rules,
            fastest_rules,
            slowest_files,
        }
    }

    /// Generate optimization suggestions
    pub fn suggest_optimizations(&self) -> Vec<OptimizationSuggestion> {
        let mut suggestions = Vec::new();

        // Suggest disabling slow rules with few problems
        for (rule_id, metrics) in &self.rule_performance {
            if let Some(stats) = self.problem_statistics.get(rule_id) {
                let problems_per_second = stats.total_problems as f64 / metrics.total_execution_time.as_secs_f64();

                if problems_per_second < 0.1 && metrics.average_execution_time > Duration::from_millis(100) {
                    suggestions.push(OptimizationSuggestion {
                        suggestion_type: OptimizationType::RuleConfiguration,
                        description: format!("Rule '{}' is slow and finds few problems", rule_id),
                        potential_impact: metrics.total_execution_time / 2,
                        priority: Priority::Medium,
                        recommendations: vec![
                            format!("Consider disabling rule '{}' if problems are not critical", rule_id),
                            "Review rule configuration parameters".to_string(),
                            "Consider running this rule only on specific file types".to_string(),
                        ],
                    });
                }
            }
        }

        // Suggest parallel processing if beneficial
        if self.total_files_processed > 10 {
            let avg_file_time = if self.total_files_processed > 0 {
                self.session_start.elapsed() / self.total_files_processed as u32
            } else {
                Duration::from_secs(0)
            };
            if avg_file_time > Duration::from_millis(50) {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: OptimizationType::ParallelProcessing,
                    description: "Files take significant time to process individually".to_string(),
                    potential_impact: self.session_start.elapsed() / 2,
                    priority: Priority::High,
                    recommendations: vec![
                        "Enable parallel file processing".to_string(),
                        "Consider using multiple CPU cores".to_string(),
                        "Batch process files in chunks".to_string(),
                    ],
                });
            }
        }

        // Suggest file filtering optimizations
        let slow_file_count = self.file_processing_times.values()
            .filter(|&&time| time > Duration::from_millis(200))
            .count();

        if slow_file_count > 0 {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: OptimizationType::FileFiltering,
                description: format!("{} files are taking unusually long to process", slow_file_count),
                potential_impact: Duration::from_millis(100 * slow_file_count as u64),
                priority: Priority::Medium,
                recommendations: vec![
                    "Review file ignore patterns".to_string(),
                    "Consider excluding large or generated files".to_string(),
                    "Check for files with complex YAML structures".to_string(),
                ],
            });
        }

        // Sort suggestions by priority and potential impact
        suggestions.sort_by(|a, b| {
            match (a.priority.clone(), b.priority.clone()) {
                (Priority::High, Priority::High) => b.potential_impact.cmp(&a.potential_impact),
                (Priority::High, _) => std::cmp::Ordering::Less,
                (_, Priority::High) => std::cmp::Ordering::Greater,
                (Priority::Medium, Priority::Medium) => b.potential_impact.cmp(&a.potential_impact),
                (Priority::Medium, _) => std::cmp::Ordering::Less,
                (_, Priority::Medium) => std::cmp::Ordering::Greater,
                (Priority::Low, Priority::Low) => b.potential_impact.cmp(&a.potential_impact),
            }
        });

        suggestions
    }

    /// Export analytics data to JSON
    pub fn export_json(&self) -> Result<String> {
        let report = self.generate_report();
        Ok(serde_json::to_string_pretty(&report)?)
    }

}

impl Default for LintAnalytics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::linter::Level;

    #[test]
    fn test_analytics_creation() {
        let analytics = LintAnalytics::new();
        assert_eq!(analytics.total_files_processed, 0);
        assert_eq!(analytics.total_problems_found, 0);
        assert!(analytics.rule_performance.is_empty());
    }

    #[test]
    fn test_record_rule_execution() {
        let mut analytics = LintAnalytics::new();
        let file_path = PathBuf::from("test.yaml");
        let execution_time = Duration::from_millis(50);

        analytics.record_rule_execution("test-rule", &file_path, execution_time);

        assert!(analytics.rule_performance.contains_key("test-rule"));
        let metrics = analytics.rule_performance.get("test-rule").unwrap();
        assert_eq!(metrics.execution_count, 1);
        assert_eq!(metrics.total_execution_time, execution_time);
        assert_eq!(metrics.max_execution_time, execution_time);
    }


    #[test]
    fn test_generate_report() {
        let mut analytics = LintAnalytics::new();
        let file_path = PathBuf::from("test.yaml");

        analytics.record_file_processing(file_path.clone(), Duration::from_millis(100));
        analytics.record_rule_execution("test-rule", &file_path, Duration::from_millis(50));

        let problems = vec![Problem::new(1, 1, Level::Error, "test-rule", "Test error")];
        analytics.record_problems("test-rule", &problems, &file_path);

        let report = analytics.generate_report();

        assert_eq!(report.session_info.total_files, 1);
        assert_eq!(report.session_info.total_problems, 1);
        assert_eq!(report.rule_performance.len(), 1);
        assert_eq!(report.problem_statistics.len(), 1);
    }

    #[test]
    fn test_suggest_optimizations() {
        let analytics = LintAnalytics::new();
        let suggestions = analytics.suggest_optimizations();

        // Empty analytics should have no suggestions
        assert!(suggestions.is_empty());
    }


    #[test]
    fn test_export_json() {
        let analytics = LintAnalytics::new();
        let json = analytics.export_json().unwrap();

        // Should be valid JSON
        let _: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(json.contains("session_info"));
        assert!(json.contains("performance_summary"));
    }
}
