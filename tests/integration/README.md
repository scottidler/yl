# YL Integration Test Harness

This directory contains a comprehensive integration test harness for validating `yl`'s compatibility with `yamllint` and testing enhanced features.

## Overview

The integration test harness ensures that:
1. **Compatibility**: `yl` produces identical results to `yamllint` for standard linting scenarios
2. **Enhanced Features**: `yl`'s unique features work as designed (inline comments, format preservation, etc.)
3. **Regression Prevention**: Previously fixed issues don't reoccur

## Architecture

```
tests/integration/
├── harness/                    # Core test harness implementation
│   ├── mod.rs                 # Main orchestration
│   ├── yamllint_runner.rs     # yamllint execution wrapper
│   ├── yl_runner.rs           # yl execution wrapper
│   ├── comparator.rs          # Result comparison engine
│   └── reporter.rs            # Test result reporting
├── fixtures/                  # Test YAML files
│   ├── compatibility/         # yamllint compatibility tests
│   ├── enhanced/              # yl-specific enhanced features
│   └── regression/            # Regression test cases
├── configs/                   # Configuration files
│   ├── yamllint/             # yamllint configurations
│   ├── yl/                   # yl configurations
│   └── test_matrix.yaml      # Test configuration matrix
└── expected/                  # Expected test outputs
```

## Running Tests

### Prerequisites

1. Install `yamllint`:
   ```bash
   pip install yamllint
   ```

2. Build `yl`:
   ```bash
   cargo build --release
   ```

### Running the Integration Tests

Run all integration tests:
```bash
cargo test --test integration_harness
```

Run specific test suites:
```bash
# Run only compatibility tests
cargo run --bin integration_harness compatibility

# Run only enhanced feature tests
cargo run --bin integration_harness enhanced

# Run only regression tests
cargo run --bin integration_harness regression
```

## Test Categories

### Compatibility Tests

These tests ensure `yl` produces identical results to `yamllint`:

- **Basic YAML Structure**: Valid/invalid YAML syntax
- **Rule-Specific Tests**: Line length, trailing spaces, indentation, etc.
- **Configuration Compatibility**: yamllint configs work with yl
- **Edge Cases**: Boundary conditions and special scenarios

### Enhanced Feature Tests

These tests validate `yl`'s unique capabilities:

- **Inline Comment Directives**: `# yl:disable line-length`
- **Format Preservation**: `# yl:preserve-format`
- **Project-Specific Ignores**: `# yl:disable-file line-length`
- **Advanced Rule Control**: Context-aware rule management

### Regression Tests

These tests prevent previously fixed bugs from reoccurring:

- Historical bug fixes
- Performance regressions
- Configuration edge cases

## Test Fixtures

### Compatibility Fixtures

Located in `fixtures/compatibility/`, these files test standard yamllint functionality:

- `basic/valid.yaml` - Well-formed YAML
- `basic/invalid_indent.yaml` - Indentation errors
- `basic/trailing_spaces.yaml` - Trailing whitespace
- `rules/line_length_80.yaml` - Line length violations (80 chars)
- `rules/empty_lines.yaml` - Empty line violations

### Enhanced Feature Fixtures

Located in `fixtures/enhanced/`, these files test yl-specific features:

- `inline_comments/disable_line_length.yaml` - Inline rule disabling
- `formatting_hints/multiline_strings.yaml` - Format preservation
- `project_ignores/ignore_long_lines.yaml` - File-level ignores

## Configuration Files

### yamllint Configurations

Standard yamllint configuration files for different test scenarios:

- `default.yaml` - Default yamllint rules
- `line_length_80.yaml` - 80-character line limit
- `line_length_120.yaml` - 120-character line limit
- `empty_lines.yaml` - Empty line rules only

### yl Configurations

Corresponding yl configuration files:

- `yamllint_compatible.yaml` - Perfect yamllint compatibility mode
- `enhanced.yaml` - Full yl feature set enabled
- `line_length_*.yaml` - Specific rule configurations

## Test Results and Reporting

The harness generates comprehensive reports:

### Console Output
- Real-time test progress
- Summary statistics
- Compatibility scores
- Failed test details

### HTML Report
Generated in `target/integration-reports/integration-report.html`:
- Visual test results
- Compatibility matrix
- Enhanced feature status
- Performance metrics

### JSON Data
Raw test data in `target/integration-reports/integration-results.json`:
- Machine-readable results
- Detailed comparison data
- Performance metrics

## Adding New Tests

### Adding Compatibility Tests

1. Create test fixture in `fixtures/compatibility/`
2. Add corresponding yamllint and yl configurations
3. Update `configs/test_matrix.yaml` with new test case
4. Run tests to verify

### Adding Enhanced Feature Tests

1. Create test fixture in `fixtures/enhanced/`
2. Add expected behavior specification
3. Update test harness to validate new feature
4. Run tests to verify

### Adding Regression Tests

1. Create test fixture in `fixtures/regression/`
2. Create expected result file (`.expected.json`)
3. Test harness will automatically discover and run

## Troubleshooting

### yamllint Not Found
```
Error: yamllint not found. Please install yamllint: pip install yamllint
```
**Solution**: Install yamllint with `pip install yamllint`

### yl Binary Not Found
```
Error: yl binary not found. Please build with: cargo build --release
```
**Solution**: Build the project with `cargo build --release`

### Test Failures
Check the generated HTML report for detailed failure analysis:
- Compatibility differences
- Enhanced feature validation failures
- Performance regressions

## Integration with CI/CD

The test harness is designed for continuous integration:

```yaml
# .github/workflows/integration-tests.yml
- name: Install yamllint
  run: pip install yamllint

- name: Build yl
  run: cargo build --release

- name: Run Integration Tests
  run: cargo test --test integration_harness

- name: Upload Test Results
  uses: actions/upload-artifact@v3
  with:
    name: integration-test-results
    path: target/integration-reports/
```

## Contributing

When adding new features to `yl`:

1. Add compatibility tests if the feature affects yamllint compatibility
2. Add enhanced feature tests for new yl-specific functionality
3. Update configurations and test matrix as needed
4. Ensure all tests pass before submitting PR

The integration test harness ensures `yl` maintains perfect compatibility with `yamllint` while providing the enhanced features that solve real-world YAML linting challenges.
