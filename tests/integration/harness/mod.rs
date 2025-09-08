use eyre::Result;
use std::path::Path;

mod comparator;
mod reporter;
mod yamllint_runner;
mod yl_runner;

pub use comparator::{ComparisonResult, CompatibilitySeverity, ResultComparator};
pub use reporter::{TestReporter, TestSuiteResults};
pub use yamllint_runner::{LintProblem, LintResult, YamllintRunner};
pub use yl_runner::{EnhancedMode, YlRunner};

/// Main integration test harness that orchestrates compatibility and feature testing
pub struct IntegrationTestHarness {
    yamllint_runner: YamllintRunner,
    yl_runner: YlRunner,
    comparator: ResultComparator,
    reporter: TestReporter,
}

impl IntegrationTestHarness {
    /// Create a new integration test harness
    pub fn new() -> Result<Self> {
        let yamllint_runner = YamllintRunner::new()?;
        let yl_runner = YlRunner::new()?;
        let comparator = ResultComparator::new();
        let reporter = TestReporter::new();

        // Validate that both tools are available
        yamllint_runner.validate_installation()?;
        yl_runner.validate_installation()?;

        Ok(Self {
            yamllint_runner,
            yl_runner,
            comparator,
            reporter,
        })
    }

    /// Run the complete compatibility test suite
    pub fn run_compatibility_suite(&self) -> Result<TestSuiteResults> {
        let mut results = TestSuiteResults::new("Compatibility Tests");

        // Load test matrix configuration
        let test_matrix = self.load_test_matrix()?;

        for test_case in test_matrix.compatibility_tests {
            let yamllint_result = self
                .yamllint_runner
                .run_test(&test_case.yamllint_config, &test_case.fixture)?;

            let yl_result = self.yl_runner.run_test(&test_case.yl_config, &test_case.fixture)?;

            let comparison = self.comparator.compare_compatibility(&yamllint_result, &yl_result);
            results.add_test_result(test_case.name, comparison);
        }

        Ok(results)
    }

    /// Run yl-specific enhanced feature tests
    pub fn run_enhanced_feature_suite(&self) -> Result<TestSuiteResults> {
        let mut results = TestSuiteResults::new("Enhanced Features");

        // Test inline comment directives
        self.test_inline_comments(&mut results)?;

        // Test formatting preservation
        self.test_format_preservation(&mut results)?;

        // Test project-specific ignores
        self.test_project_ignores(&mut results)?;

        Ok(results)
    }

    /// Run regression tests
    pub fn run_regression_suite(&self) -> Result<TestSuiteResults> {
        let mut results = TestSuiteResults::new("Regression Tests");

        // Load and run regression test cases
        let regression_fixtures = self.load_regression_fixtures()?;

        for fixture in regression_fixtures {
            let yl_result = self.yl_runner.run_test(&fixture.config, &fixture.file)?;
            let is_valid = self.validate_regression_result(&yl_result, &fixture.expected)?;

            results.add_regression_result(fixture.name, is_valid);
        }

        Ok(results)
    }

    /// Generate comprehensive test report
    pub fn generate_report(&self, results: &[TestSuiteResults]) -> Result<()> {
        self.reporter.generate_html_report(results)?;
        self.reporter.generate_console_summary(results)?;
        Ok(())
    }

    // Private helper methods

    fn load_test_matrix(&self) -> Result<TestMatrix> {
        // Load test_matrix.yaml configuration
        let config_path = Path::new("tests/integration/configs/test_matrix.yaml");
        let content = std::fs::read_to_string(config_path)?;
        Ok(serde_yaml::from_str(&content)?)
    }

    fn test_inline_comments(&self, results: &mut TestSuiteResults) -> Result<()> {
        let fixtures_dir = Path::new("tests/integration/fixtures/enhanced/inline_comments");

        for entry in std::fs::read_dir(fixtures_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                let result = self.yl_runner.run_enhanced_test(
                    &Path::new("tests/integration/configs/yl/enhanced.yaml"),
                    &entry.path(),
                    EnhancedMode::Enhanced,
                )?;

                let is_valid = self.validate_inline_comment_behavior(&result)?;
                results.add_enhanced_result(entry.file_name().to_string_lossy().to_string(), is_valid);
            }
        }

        Ok(())
    }

    fn test_format_preservation(&self, results: &mut TestSuiteResults) -> Result<()> {
        let fixtures_dir = Path::new("tests/integration/fixtures/enhanced/formatting_hints");

        for entry in std::fs::read_dir(fixtures_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                let result = self.yl_runner.run_enhanced_test(
                    &Path::new("tests/integration/configs/yl/enhanced.yaml"),
                    &entry.path(),
                    EnhancedMode::Enhanced,
                )?;

                let is_valid = self.validate_format_preservation(&result)?;
                results.add_enhanced_result(entry.file_name().to_string_lossy().to_string(), is_valid);
            }
        }

        Ok(())
    }

    fn test_project_ignores(&self, results: &mut TestSuiteResults) -> Result<()> {
        let fixtures_dir = Path::new("tests/integration/fixtures/enhanced/project_ignores");

        for entry in std::fs::read_dir(fixtures_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                let result = self.yl_runner.run_enhanced_test(
                    &Path::new("tests/integration/configs/yl/enhanced.yaml"),
                    &entry.path(),
                    EnhancedMode::Enhanced,
                )?;

                let is_valid = self.validate_project_ignores(&result)?;
                results.add_enhanced_result(entry.file_name().to_string_lossy().to_string(), is_valid);
            }
        }

        Ok(())
    }

    fn load_regression_fixtures(&self) -> Result<Vec<RegressionFixture>> {
        let fixtures_dir = Path::new("tests/integration/fixtures/regression");
        let mut fixtures = Vec::new();

        for entry in std::fs::read_dir(fixtures_dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                // Load corresponding expected result
                let expected_path = entry.path().with_extension("expected.json");
                if expected_path.exists() {
                    let expected_content = std::fs::read_to_string(&expected_path)?;
                    let expected: ExpectedResult = serde_json::from_str(&expected_content)?;

                    fixtures.push(RegressionFixture {
                        name: entry.file_name().to_string_lossy().to_string(),
                        file: entry.path(),
                        config: Path::new("tests/integration/configs/yl/default.yaml").to_path_buf(),
                        expected,
                    });
                }
            }
        }

        Ok(fixtures)
    }

    fn validate_regression_result(&self, result: &LintResult, expected: &ExpectedResult) -> Result<bool> {
        // Validate that the result matches expected behavior
        Ok(result.problems.len() == expected.problem_count && result.exit_code == expected.exit_code)
    }

    fn validate_inline_comment_behavior(&self, _result: &LintResult) -> Result<bool> {
        // Validate that inline comments properly disable/enable rules
        // This is a placeholder - actual implementation would parse the fixture
        // and verify that comment directives were respected
        Ok(true)
    }

    fn validate_format_preservation(&self, _result: &LintResult) -> Result<bool> {
        // Validate that formatting hints were respected
        // This is a placeholder - actual implementation would check that
        // format preservation directives were honored
        Ok(true)
    }

    fn validate_project_ignores(&self, _result: &LintResult) -> Result<bool> {
        // Validate that project-specific ignores worked correctly
        // This is a placeholder - actual implementation would verify
        // that ignore directives were properly applied
        Ok(true)
    }
}

impl Default for IntegrationTestHarness {
    fn default() -> Self {
        Self::new().expect("Failed to create integration test harness")
    }
}

// Supporting types and structures

#[derive(Debug, serde::Deserialize)]
struct TestMatrix {
    compatibility_tests: Vec<CompatibilityTest>,
}

#[derive(Debug, serde::Deserialize)]
struct CompatibilityTest {
    name: String,
    yamllint_config: std::path::PathBuf,
    yl_config: std::path::PathBuf,
    fixture: std::path::PathBuf,
}

#[derive(Debug)]
struct RegressionFixture {
    name: String,
    file: std::path::PathBuf,
    config: std::path::PathBuf,
    expected: ExpectedResult,
}

#[derive(Debug, serde::Deserialize)]
struct ExpectedResult {
    exit_code: i32,
    problem_count: usize,
}
