use super::{ComparisonResult, CompatibilitySeverity};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::fs;

/// Generates reports for integration test results
pub struct TestReporter {
    output_dir: std::path::PathBuf,
}

/// Results from a complete test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResults {
    pub suite_name: String,
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub test_results: Vec<TestResult>,
    pub summary: TestSummary,
}

/// Individual test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub test_name: String,
    pub test_type: TestType,
    pub status: TestStatus,
    pub comparison_result: Option<ComparisonResult>,
    pub execution_time: std::time::Duration,
    pub details: String,
}

/// Type of test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestType {
    Compatibility,
    EnhancedFeature,
    Regression,
    Performance,
}

/// Status of a test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Error,
}

/// Summary statistics for a test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub compatibility_score: f64,
    pub enhanced_features_working: usize,
    pub enhanced_features_total: usize,
    pub regression_tests_passed: usize,
    pub regression_tests_total: usize,
    pub overall_status: OverallStatus,
}

/// Overall status of the test suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverallStatus {
    AllPassed,
    SomeFailures,
    CriticalFailures,
    SystemError,
}

impl TestReporter {
    /// Create a new test reporter
    pub fn new() -> Self {
        Self {
            output_dir: std::path::PathBuf::from("target/integration-reports"),
        }
    }

    /// Generate an HTML report for test results
    pub fn generate_html_report(&self, results: &[TestSuiteResults]) -> Result<()> {
        // Ensure output directory exists
        fs::create_dir_all(&self.output_dir)?;

        let html_content = self.generate_html_content(results)?;
        let report_path = self.output_dir.join("integration-report.html");
        fs::write(report_path, html_content)?;

        // Generate JSON data for the report
        let json_content = serde_json::to_string_pretty(results)?;
        let json_path = self.output_dir.join("integration-results.json");
        fs::write(json_path, json_content)?;

        println!(
            "Integration test report generated in: {}",
            self.output_dir.display()
        );
        Ok(())
    }

    /// Generate a console summary of test results
    pub fn generate_console_summary(&self, results: &[TestSuiteResults]) -> Result<()> {
        println!("\n🧪 Integration Test Results Summary");
        println!("=====================================");

        let mut total_tests = 0;
        let mut total_passed = 0;
        let mut total_failed = 0;

        for suite in results {
            total_tests += suite.total_tests;
            total_passed += suite.passed_tests;
            total_failed += suite.failed_tests;

            println!("\n📋 {} Suite:", suite.suite_name);
            println!("   ✅ Passed: {}/{}", suite.passed_tests, suite.total_tests);

            if suite.failed_tests > 0 {
                println!("   ❌ Failed: {}", suite.failed_tests);
            }

            // Show compatibility score for compatibility tests
            if suite.suite_name.contains("Compatibility") {
                println!(
                    "   🎯 Compatibility Score: {:.1}%",
                    suite.summary.compatibility_score
                );
            }

            // Show enhanced feature status
            if suite.suite_name.contains("Enhanced") {
                println!(
                    "   🚀 Enhanced Features: {}/{} working",
                    suite.summary.enhanced_features_working, suite.summary.enhanced_features_total
                );
            }
        }

        println!("\n📊 Overall Results:");
        println!("   Total Tests: {}", total_tests);
        println!(
            "   Passed: {} ({:.1}%)",
            total_passed,
            (total_passed as f64 / total_tests as f64) * 100.0
        );

        if total_failed > 0 {
            println!(
                "   Failed: {} ({:.1}%)",
                total_failed,
                (total_failed as f64 / total_tests as f64) * 100.0
            );
        }

        // Determine overall status
        let overall_status = self.determine_overall_status(results);
        match overall_status {
            OverallStatus::AllPassed => println!("   🎉 Status: All tests passed!"),
            OverallStatus::SomeFailures => println!("   ⚠️  Status: Some tests failed"),
            OverallStatus::CriticalFailures => println!("   🚨 Status: Critical failures detected"),
            OverallStatus::SystemError => println!("   💥 Status: System errors encountered"),
        }

        Ok(())
    }

    /// Generate HTML content for the report
    fn generate_html_content(&self, results: &[TestSuiteResults]) -> Result<String> {
        let mut html = String::new();

        // HTML header
        let timestamp = chrono::Utc::now()
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string();
        html.push_str(&format!(
            r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>YL Integration Test Report</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 20px; }}
        .header {{ background: #f8f9fa; padding: 20px; border-radius: 8px; margin-bottom: 20px; }}
        .suite {{ background: white; border: 1px solid #dee2e6; border-radius: 8px; margin-bottom: 20px; }}
        .suite-header {{ background: #e9ecef; padding: 15px; border-radius: 8px 8px 0 0; }}
        .test-result {{ padding: 10px 15px; border-bottom: 1px solid #f1f3f4; }}
        .test-result:last-child {{ border-bottom: none; }}
        .status-passed {{ color: #28a745; }}
        .status-failed {{ color: #dc3545; }}
        .status-skipped {{ color: #6c757d; }}
        .compatibility-score {{ font-size: 1.2em; font-weight: bold; }}
        .details {{ font-size: 0.9em; color: #6c757d; margin-top: 5px; }}
        .summary {{ background: #f8f9fa; padding: 15px; border-radius: 8px; margin-top: 20px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🧪 YL Integration Test Report</h1>
        <p>Generated on: {}</p>
    </div>
"#,
            timestamp
        ));

        // Generate content for each test suite
        for suite in results {
            html.push_str(&format!(r#"
    <div class="suite">
        <div class="suite-header">
            <h2>📋 {}</h2>
            <p>Tests: {} | Passed: <span class="status-passed">{}</span> | Failed: <span class="status-failed">{}</span></p>
"#, suite.suite_name, suite.total_tests, suite.passed_tests, suite.failed_tests));

            // Add compatibility score if applicable
            if suite.suite_name.contains("Compatibility") {
                html.push_str(&format!(
                    r#"
            <p class="compatibility-score">🎯 Compatibility Score: {:.1}%</p>
"#,
                    suite.summary.compatibility_score
                ));
            }

            html.push_str("        </div>");

            // Add individual test results
            for test in &suite.test_results {
                let _status_class = match test.status {
                    TestStatus::Passed => "status-passed",
                    TestStatus::Failed => "status-failed",
                    TestStatus::Skipped => "status-skipped",
                    TestStatus::Error => "status-failed",
                };

                let status_icon = match test.status {
                    TestStatus::Passed => "✅",
                    TestStatus::Failed => "❌",
                    TestStatus::Skipped => "⏭️",
                    TestStatus::Error => "💥",
                };

                html.push_str(&format!(
                    r#"
        <div class="test-result">
            <strong>{} {} {}</strong>
            <div class="details">{}</div>
        </div>
"#,
                    status_icon, test.test_name, "", test.details
                ));
            }

            html.push_str("    </div>");
        }

        // Add summary
        let overall_status = self.determine_overall_status(results);
        let status_text = match overall_status {
            OverallStatus::AllPassed => "🎉 All tests passed!",
            OverallStatus::SomeFailures => "⚠️ Some tests failed",
            OverallStatus::CriticalFailures => "🚨 Critical failures detected",
            OverallStatus::SystemError => "💥 System errors encountered",
        };

        html.push_str(&format!(
            r#"
    <div class="summary">
        <h2>📊 Summary</h2>
        <p><strong>Overall Status:</strong> {}</p>
    </div>
</body>
</html>
"#,
            status_text
        ));

        Ok(html)
    }

    /// Determine the overall status across all test suites
    fn determine_overall_status(&self, results: &[TestSuiteResults]) -> OverallStatus {
        let has_failures = results.iter().any(|r| r.failed_tests > 0);
        let has_critical = results.iter().any(|r| {
            r.test_results.iter().any(|t| {
                matches!(t.status, TestStatus::Error)
                    || (matches!(t.status, TestStatus::Failed)
                        && t.comparison_result.as_ref().map_or(false, |c| {
                            matches!(c.severity, CompatibilitySeverity::Incompatible)
                        }))
            })
        });

        if has_critical {
            OverallStatus::CriticalFailures
        } else if has_failures {
            OverallStatus::SomeFailures
        } else {
            OverallStatus::AllPassed
        }
    }
}

impl TestSuiteResults {
    /// Create a new test suite results container
    pub fn new(suite_name: &str) -> Self {
        Self {
            suite_name: suite_name.to_string(),
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            test_results: Vec::new(),
            summary: TestSummary {
                compatibility_score: 0.0,
                enhanced_features_working: 0,
                enhanced_features_total: 0,
                regression_tests_passed: 0,
                regression_tests_total: 0,
                overall_status: OverallStatus::AllPassed,
            },
        }
    }

    /// Add a compatibility test result
    pub fn add_test_result(&mut self, test_name: String, comparison: ComparisonResult) {
        let status = if comparison.is_compatible {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        let test_result = TestResult {
            test_name,
            test_type: TestType::Compatibility,
            status: status.clone(),
            comparison_result: Some(comparison),
            execution_time: std::time::Duration::from_millis(0), // Would be filled in real implementation
            details: "Compatibility test".to_string(),
        };

        self.test_results.push(test_result);
        self.total_tests += 1;

        match status {
            TestStatus::Passed => self.passed_tests += 1,
            TestStatus::Failed => self.failed_tests += 1,
            _ => {}
        }

        self.update_summary();
    }

    /// Add an enhanced feature test result
    pub fn add_enhanced_result(&mut self, test_name: String, is_valid: bool) {
        let status = if is_valid {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        let test_result = TestResult {
            test_name,
            test_type: TestType::EnhancedFeature,
            status: status.clone(),
            comparison_result: None,
            execution_time: std::time::Duration::from_millis(0),
            details: "Enhanced feature test".to_string(),
        };

        self.test_results.push(test_result);
        self.total_tests += 1;
        self.summary.enhanced_features_total += 1;

        match status {
            TestStatus::Passed => {
                self.passed_tests += 1;
                self.summary.enhanced_features_working += 1;
            }
            TestStatus::Failed => self.failed_tests += 1,
            _ => {}
        }

        self.update_summary();
    }

    /// Add a regression test result
    pub fn add_regression_result(&mut self, test_name: String, is_valid: bool) {
        let status = if is_valid {
            TestStatus::Passed
        } else {
            TestStatus::Failed
        };

        let test_result = TestResult {
            test_name,
            test_type: TestType::Regression,
            status: status.clone(),
            comparison_result: None,
            execution_time: std::time::Duration::from_millis(0),
            details: "Regression test".to_string(),
        };

        self.test_results.push(test_result);
        self.total_tests += 1;
        self.summary.regression_tests_total += 1;

        match status {
            TestStatus::Passed => {
                self.passed_tests += 1;
                self.summary.regression_tests_passed += 1;
            }
            TestStatus::Failed => self.failed_tests += 1,
            _ => {}
        }

        self.update_summary();
    }

    /// Update the summary statistics
    fn update_summary(&mut self) {
        // Calculate compatibility score
        let compatibility_tests = self
            .test_results
            .iter()
            .filter(|t| matches!(t.test_type, TestType::Compatibility))
            .count();

        if compatibility_tests > 0 {
            let compatible_tests = self
                .test_results
                .iter()
                .filter(|t| {
                    matches!(t.test_type, TestType::Compatibility)
                        && matches!(t.status, TestStatus::Passed)
                })
                .count();

            self.summary.compatibility_score =
                (compatible_tests as f64 / compatibility_tests as f64) * 100.0;
        }

        // Update overall status
        self.summary.overall_status = if self.failed_tests == 0 {
            OverallStatus::AllPassed
        } else {
            OverallStatus::SomeFailures
        };
    }
}

impl Default for TestReporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_suite_results_creation() {
        let results = TestSuiteResults::new("Test Suite");
        assert_eq!(results.suite_name, "Test Suite");
        assert_eq!(results.total_tests, 0);
        assert_eq!(results.passed_tests, 0);
        assert_eq!(results.failed_tests, 0);
    }

    #[test]
    fn test_add_test_result() {
        let mut results = TestSuiteResults::new("Compatibility Tests");

        let comparison = ComparisonResult {
            is_compatible: true,
            differences: vec![],
            severity: CompatibilitySeverity::Identical,
            summary: "Perfect match".to_string(),
        };

        results.add_test_result("test1".to_string(), comparison);

        assert_eq!(results.total_tests, 1);
        assert_eq!(results.passed_tests, 1);
        assert_eq!(results.failed_tests, 0);
        assert_eq!(results.summary.compatibility_score, 100.0);
    }
}
