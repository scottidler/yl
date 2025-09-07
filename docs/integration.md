# Integration Test Harness Design

## Overview

This document outlines the architecture for a comprehensive test harness that validates `yl`'s compatibility with `yamllint` while testing the enhanced features that differentiate `yl` from existing YAML linters.

## Problem Statement

Based on the conversation in `convo.txt`, existing YAML formatters like `yamlfmt` have limitations:
- No inline comment-based rule configuration
- Aggressive reformatting that ignores intentional formatting choices
- Limited project-specific ignore capabilities
- No fine-grained control over formatting behavior

The `yl` linter aims to solve these issues while maintaining compatibility with `yamllint`'s core functionality.

## Test Harness Architecture

### Core Components

```
tests/
├── integration/
│   ├── harness/
│   │   ├── mod.rs                    # Main harness orchestration
│   │   ├── yamllint_runner.rs        # yamllint execution wrapper
│   │   ├── yl_runner.rs              # yl execution wrapper
│   │   ├── comparator.rs             # Result comparison engine
│   │   └── reporter.rs               # Test result reporting
│   ├── fixtures/
│   │   ├── compatibility/            # yamllint compatibility tests
│   │   │   ├── basic/               # Basic YAML linting scenarios
│   │   │   ├── rules/               # Per-rule validation
│   │   │   ├── configs/             # Configuration compatibility
│   │   │   └── edge_cases/          # Edge case scenarios
│   │   ├── enhanced/                # yl-specific enhanced features
│   │   │   ├── inline_comments/     # Comment-based rule control
│   │   │   ├── formatting_hints/    # Formatting preservation
│   │   │   ├── project_ignores/     # Advanced ignore patterns
│   │   │   └── custom_rules/        # yl-specific rules
│   │   └── regression/              # Regression test cases
│   ├── configs/
│   │   ├── yamllint/               # yamllint configuration files
│   │   ├── yl/                     # yl configuration files
│   │   └── test_matrix.yaml        # Test configuration matrix
│   └── expected/
│       ├── yamllint/               # Expected yamllint outputs
│       ├── yl_compatible/          # Expected yl outputs (compatible mode)
│       └── yl_enhanced/            # Expected yl outputs (enhanced mode)
```

### Test Categories

#### 1. Compatibility Tests (`tests/integration/fixtures/compatibility/`)

**Purpose**: Ensure `yl` produces identical results to `yamllint` for standard linting scenarios.

**Test Structure**:
```yaml
# test_matrix.yaml
compatibility_tests:
  - name: "basic_yaml_structure"
    description: "Basic YAML syntax validation"
    yamllint_config: "configs/yamllint/default.yaml"
    yl_config: "configs/yl/yamllint_compatible.yaml"
    fixtures:
      - "fixtures/compatibility/basic/valid.yaml"
      - "fixtures/compatibility/basic/invalid_indent.yaml"
      - "fixtures/compatibility/basic/trailing_spaces.yaml"

  - name: "line_length_rules"
    description: "Line length validation compatibility"
    yamllint_config: "configs/yamllint/line_length.yaml"
    yl_config: "configs/yl/line_length_compatible.yaml"
    fixtures:
      - "fixtures/compatibility/rules/line_length_80.yaml"
      - "fixtures/compatibility/rules/line_length_120.yaml"
```

#### 2. Enhanced Feature Tests (`tests/integration/fixtures/enhanced/`)

**Purpose**: Validate `yl`'s unique features that address the limitations mentioned in `convo.txt`.

**Key Features to Test**:

1. **Inline Comment Directives**:
   ```yaml
   # Example fixture: inline_comments/format_preservation.yaml
   external-dns.alpha.kubernetes.io/hostname: |  # yl:preserve-format
     airflow.tataridev.com.,
     argocd-cli.prod.tatari.dev.,
     api.tatari.tv.,
     auth.tatari.tv.
   ```

2. **Formatting Hints**:
   ```yaml
   # Example fixture: formatting_hints/multiline_strings.yaml
   description: >  # yl:no-reflow
     This string should not be
     reformatted or combined into
     a single line by formatters.
   ```

3. **Project-Specific Ignores**:
   ```yaml
   # Example fixture: project_ignores/selective_disable.yaml
   # yl:disable-file line-length
   very_long_line_that_exceeds_normal_limits_but_is_intentional_for_this_specific_file: "value"
   ```

### Test Harness Implementation

#### Core Harness (`tests/integration/harness/mod.rs`)

```rust
pub struct IntegrationTestHarness {
    yamllint_runner: YamllintRunner,
    yl_runner: YlRunner,
    comparator: ResultComparator,
    reporter: TestReporter,
}

impl IntegrationTestHarness {
    pub fn new() -> Result<Self> {
        // Initialize runners and validate tool availability
    }

    pub fn run_compatibility_suite(&self) -> Result<TestSuiteResults> {
        // Run all compatibility tests
    }

    pub fn run_enhanced_feature_suite(&self) -> Result<TestSuiteResults> {
        // Run yl-specific feature tests
    }

    pub fn run_regression_suite(&self) -> Result<TestSuiteResults> {
        // Run regression tests
    }

    pub fn generate_report(&self, results: &TestSuiteResults) -> Result<()> {
        // Generate comprehensive test report
    }
}
```

#### yamllint Runner (`tests/integration/harness/yamllint_runner.rs`)

```rust
pub struct YamllintRunner {
    yamllint_path: PathBuf,
    timeout: Duration,
}

impl YamllintRunner {
    pub fn run_test(&self, config: &Path, fixture: &Path) -> Result<LintResult> {
        // Execute yamllint with specified config on fixture
        // Parse output and return structured result
    }

    pub fn validate_installation(&self) -> Result<Version> {
        // Verify yamllint is installed and get version
    }
}

#[derive(Debug, Clone)]
pub struct LintResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub problems: Vec<LintProblem>,
    pub execution_time: Duration,
}
```

#### yl Runner (`tests/integration/harness/yl_runner.rs`)

```rust
pub struct YlRunner {
    yl_binary: PathBuf,
    timeout: Duration,
}

impl YlRunner {
    pub fn run_test(&self, config: &Path, fixture: &Path) -> Result<LintResult> {
        // Execute yl with specified config on fixture
    }

    pub fn run_enhanced_test(&self, config: &Path, fixture: &Path, mode: EnhancedMode) -> Result<LintResult> {
        // Run yl with enhanced features enabled
    }
}

#[derive(Debug, Clone)]
pub enum EnhancedMode {
    Compatible,      // yamllint-compatible mode
    Enhanced,        // Full yl feature set
    Selective(Vec<String>), // Specific features enabled
}
```

#### Result Comparator (`tests/integration/harness/comparator.rs`)

```rust
pub struct ResultComparator {
    tolerance: ComparisonTolerance,
}

impl ResultComparator {
    pub fn compare_compatibility(&self, yamllint: &LintResult, yl: &LintResult) -> ComparisonResult {
        // Compare results for compatibility testing
        // Allow for acceptable differences (e.g., message formatting)
    }

    pub fn validate_enhanced_features(&self, result: &LintResult, expected: &EnhancedExpectation) -> ValidationResult {
        // Validate yl-specific enhanced features
    }
}

#[derive(Debug)]
pub struct ComparisonResult {
    pub is_compatible: bool,
    pub differences: Vec<Difference>,
    pub severity: CompatibilitySeverity,
}

#[derive(Debug)]
pub enum CompatibilitySeverity {
    Identical,           // Perfect match
    Acceptable,          // Minor differences (formatting, etc.)
    Concerning,          // Significant differences that may indicate issues
    Incompatible,        // Major differences that break compatibility
}
```

### Test Configuration Matrix

#### Compatibility Test Matrix (`tests/integration/configs/test_matrix.yaml`)

```yaml
test_matrix:
  yamllint_versions:
    - "1.32.0"  # Current stable
    - "1.31.0"  # Previous stable

  rule_combinations:
    - name: "default"
      rules: ["line-length", "trailing-spaces", "empty-lines", "indentation"]
    - name: "strict"
      rules: ["all"]
    - name: "minimal"
      rules: ["syntax-only"]

  configuration_variants:
    - name: "default_80_char"
      line_length: 80
      indent_spaces: 2
    - name: "extended_120_char"
      line_length: 120
      indent_spaces: 4
    - name: "tabs_preferred"
      indent_type: "tabs"

  fixture_categories:
    - basic_syntax
    - complex_structures
    - edge_cases
    - real_world_examples
```

### Enhanced Feature Test Specifications

#### 1. Inline Comment Directives

```yaml
# Test: Comment-based rule control
test_cases:
  - name: "disable_line_length_inline"
    fixture: |
      short_line: "value"
      very_long_line_that_exceeds_configured_limit: "but should be ignored"  # yl:disable line-length
      another_short: "value"
    expected_yl:
      problems: []  # No line-length violations
    expected_yamllint:
      problems:
        - line: 2
          rule: "line too long"
```

#### 2. Format Preservation

```yaml
# Test: Preserve intentional formatting
test_cases:
  - name: "preserve_multiline_strings"
    fixture: |
      # yl:preserve-format
      external-dns.alpha.kubernetes.io/hostname: |
        airflow.tataridev.com.,
        argocd-cli.prod.tatari.dev.,
        api.tatari.tv.,
        auth.tatari.tv.
    expected_behavior:
      - yl_should_not_suggest_reformatting
      - preserve_line_breaks_and_spacing
      - respect_intentional_structure
```

### Test Execution Workflow

#### 1. Pre-Test Validation

```rust
pub fn validate_test_environment() -> Result<()> {
    // Check yamllint installation and version
    // Verify yl binary exists and is executable
    // Validate test fixture integrity
    // Ensure configuration files are valid
}
```

#### 2. Test Execution Pipeline

```rust
pub fn execute_test_pipeline() -> Result<TestResults> {
    let harness = IntegrationTestHarness::new()?;

    // Phase 1: Compatibility Tests
    let compatibility_results = harness.run_compatibility_suite()?;

    // Phase 2: Enhanced Feature Tests
    let enhanced_results = harness.run_enhanced_feature_suite()?;

    // Phase 3: Regression Tests
    let regression_results = harness.run_regression_suite()?;

    // Phase 4: Performance Comparison
    let performance_results = harness.run_performance_comparison()?;

    Ok(TestResults {
        compatibility: compatibility_results,
        enhanced: enhanced_results,
        regression: regression_results,
        performance: performance_results,
    })
}
```

#### 3. Result Analysis and Reporting

```rust
pub fn generate_comprehensive_report(results: &TestResults) -> Result<()> {
    // Generate HTML report with:
    // - Compatibility matrix
    // - Enhanced feature validation
    // - Performance comparison charts
    // - Regression test status
    // - Recommendations for improvements
}
```

### Continuous Integration Integration

#### GitHub Actions Workflow (`.github/workflows/integration-tests.yml`)

```yaml
name: Integration Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        yamllint-version: ["1.32.0", "1.31.0"]

    steps:
      - uses: actions/checkout@v4

      - name: Install yamllint
        run: pip install yamllint==${{ matrix.yamllint-version }}

      - name: Build yl
        run: cargo build --release

      - name: Run Integration Tests
        run: cargo test --test integration_harness

      - name: Generate Report
        run: cargo run --bin integration-report

      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: integration-test-results-${{ matrix.yamllint-version }}
          path: target/integration-reports/
```

### Test Fixture Management

#### Fixture Generation Strategy

1. **Real-World Examples**: Collect YAML files from popular projects
2. **Synthetic Edge Cases**: Generate specific test cases for boundary conditions
3. **Regression Cases**: Maintain cases for previously fixed bugs
4. **Performance Cases**: Large files for performance testing

#### Fixture Validation

```rust
pub fn validate_fixtures() -> Result<()> {
    // Ensure all fixtures are valid YAML
    // Verify expected results are consistent
    // Check that test cases cover all rule combinations
    // Validate that enhanced feature tests are comprehensive
}
```

### Success Criteria

#### Compatibility Requirements

1. **100% Rule Compatibility**: All yamllint rules must produce identical results
2. **Configuration Compatibility**: yamllint configs must work with yl
3. **Output Format Compatibility**: Error messages and formats should match
4. **Performance Parity**: yl should not be significantly slower than yamllint

#### Enhanced Feature Requirements

1. **Inline Comment Processing**: Must correctly parse and apply comment directives
2. **Format Preservation**: Must respect formatting hints and preserve intentional structure
3. **Advanced Ignores**: Must support project-specific and context-aware ignoring
4. **Backward Compatibility**: Enhanced features must not break yamllint compatibility

### Future Enhancements

#### Planned Improvements

1. **Fuzzing Integration**: Add property-based testing with arbitrary YAML generation
2. **Performance Benchmarking**: Automated performance regression detection
3. **Cross-Platform Testing**: Windows, macOS, and Linux compatibility validation
4. **Version Matrix Testing**: Support for multiple yamllint versions simultaneously

#### Extensibility

The test harness is designed to be extensible for:
- Additional YAML linters (yamlfmt, prettier, etc.)
- Custom rule implementations
- Plugin system validation
- Integration with other development tools

## Implementation Timeline

### Phase 1: Core Harness (Week 1-2)
- Implement basic test runners
- Create fixture management system
- Build result comparison engine

### Phase 2: Compatibility Testing (Week 3-4)
- Develop comprehensive compatibility test suite
- Implement yamllint result parsing
- Create compatibility validation logic

### Phase 3: Enhanced Features (Week 5-6)
- Build enhanced feature test framework
- Implement comment directive testing
- Create format preservation validation

### Phase 4: Integration & Reporting (Week 7-8)
- Integrate with CI/CD pipeline
- Implement comprehensive reporting
- Add performance benchmarking

This integration test harness will ensure that `yl` maintains perfect compatibility with `yamllint` while providing the enhanced features that address the real-world pain points mentioned in the conversation, particularly around format preservation and inline rule control.
