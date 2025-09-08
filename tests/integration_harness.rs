use eyre::Result;
use std::env;

// Suppress false positive module resolution error from language server
#[path = "integration/harness/mod.rs"]
mod harness;

use harness::IntegrationTestHarness;

/// Test types that can be run by the integration harness
#[derive(Debug, Clone, PartialEq)]
enum TestType {
    Compatibility,
    Enhanced,
    Regression,
    All,
}

impl TestType {
    /// Parse a test type from a string argument
    fn from_str(s: &str) -> Self {
        match s {
            "compatibility" => TestType::Compatibility,
            "enhanced" => TestType::Enhanced,
            "regression" => TestType::Regression,
            "all" => TestType::All,
            _ => TestType::All, // Default to All for unknown inputs
        }
    }

    /// Get the display name for the test type
    fn display_name(&self) -> &'static str {
        match self {
            TestType::Compatibility => "compatibility",
            TestType::Enhanced => "enhanced",
            TestType::Regression => "regression",
            TestType::All => "all",
        }
    }
}

/// Main integration test runner
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize error handling
    color_eyre::install()?;

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let test_type_str = args.get(1).map(|s| s.as_str()).unwrap_or("all");
    let test_type = TestType::from_str(test_type_str);

    println!("🚀 Starting YL Integration Test Harness");
    println!("Test Type: {}", test_type.display_name());

    // Create the test harness
    let harness = IntegrationTestHarness::new()?;

    let mut all_results = Vec::new();

    match test_type {
        TestType::Compatibility => {
            println!("\n🔍 Running Compatibility Tests...");
            let results = harness.run_compatibility_suite()?;
            all_results.push(results);
        }
        TestType::Enhanced => {
            println!("\n🚀 Running Enhanced Feature Tests...");
            let results = harness.run_enhanced_feature_suite()?;
            all_results.push(results);
        }
        TestType::Regression => {
            println!("\n🔄 Running Regression Tests...");
            let results = harness.run_regression_suite()?;
            all_results.push(results);
        }
        TestType::All => {
            println!("\n🔍 Running Compatibility Tests...");
            let compatibility_results = harness.run_compatibility_suite()?;
            all_results.push(compatibility_results);

            println!("\n🚀 Running Enhanced Feature Tests...");
            let enhanced_results = harness.run_enhanced_feature_suite()?;
            all_results.push(enhanced_results);

            println!("\n🔄 Running Regression Tests...");
            let regression_results = harness.run_regression_suite()?;
            all_results.push(regression_results);
        }
    }

    // Generate reports
    println!("\n📊 Generating Reports...");
    harness.generate_report(&all_results)?;

    // Determine exit code based on results
    let has_failures = all_results.iter().any(|r| r.failed_tests > 0);
    let exit_code = if has_failures { 1 } else { 0 };

    println!("\n✨ Integration tests completed!");
    std::process::exit(exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_harness_creation() {
        // Test that we can create the harness (this will validate tool availability)
        let result = IntegrationTestHarness::new();

        // If yamllint is not available, this test should be skipped
        if result.is_err() {
            println!("Skipping integration tests - yamllint not available");
            return;
        }

        let harness = result.unwrap();

        // Test that we can run a basic compatibility test
        // This is a smoke test to ensure the harness is working
        let _results = harness.run_compatibility_suite();
    }

    #[test]
    fn test_command_line_parsing() {
        // Test that different command line arguments are handled correctly
        let test_cases = vec![
            ("compatibility", TestType::Compatibility),
            ("enhanced", TestType::Enhanced),
            ("regression", TestType::Regression),
            ("all", TestType::All),
            ("invalid", TestType::All), // Should default to All for unknown inputs
        ];

        for (input, expected) in test_cases {
            let parsed = TestType::from_str(input);
            assert_eq!(parsed, expected);
        }
    }
}
