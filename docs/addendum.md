# Architectural Refactoring Plan: Module Organization

## Problem Statement

The current codebase suffers from poor module organization where large functionality is crammed into single `mod.rs` files within directories. This is analogous to having a single `__init__.py` file in Python directories and is equally problematic for maintainability, readability, and code organization.

### Current Issues

1. **Monolithic `mod.rs` files**: Large files (300-500+ lines) containing multiple related but distinct concepts
2. **Poor separation of concerns**: Types, implementations, tests, and utilities all mixed together
3. **Difficult navigation**: Finding specific functionality requires scrolling through large files
4. **Merge conflicts**: Large files increase likelihood of git conflicts
5. **Cognitive overhead**: Understanding module structure requires reading entire files

### Current State Analysis

```
src/
├── analytics/mod.rs     (445 lines) - Everything analytics-related
├── diff/mod.rs          (449 lines) - Everything diff-related
├── ml/mod.rs            (429 lines) - Everything ML-related
├── policy/mod.rs        (537 lines) - Everything policy-related
├── migration/mod.rs     (386 lines) - Everything migration-related
├── rules/mod.rs         (313 lines) - All rule definitions
├── lsp/mod.rs           (310 lines) - LSP server implementation
├── fixes/mod.rs         (295 lines) - All fix implementations
├── plugins/mod.rs       (229 lines) - Plugin system
└── output/mod.rs        (110 lines) - Output formatting
```

## Proposed Architecture

### Design Principles

1. **Single Responsibility**: Each file should have one clear purpose
2. **Logical Grouping**: Related functionality should be grouped in modules
3. **Flat Structure**: Prefer `module_name.rs` over `module_name/mod.rs` when possible
4. **Clear Boundaries**: Separate types, implementations, tests, and utilities
5. **Discoverability**: File names should clearly indicate their purpose

### Refactoring Strategy

#### 1. Large Modules → Multiple Files

**Before:**
```
src/policy/mod.rs (537 lines)
```

**After:**
```
src/
├── policy.rs              # Main module with re-exports
├── policy_types.rs        # TeamPolicy, PolicyViolation, etc.
├── policy_manager.rs      # PolicyManager implementation
├── policy_validation.rs   # Validation logic
└── policy_reports.rs      # Report generation
```

#### 2. Rule System Reorganization

**Before:**
```
src/rules/mod.rs (313 lines)
```

**After:**
```
src/
├── rules.rs              # Main module with registry
├── rule_types.rs         # Rule trait, RuleConfig, etc.
├── style_rules.rs        # Line length, trailing spaces, etc.
├── syntax_rules.rs       # Document structure, anchors, etc.
├── semantic_rules.rs     # Truthy, octal values, etc.
└── formatting_rules.rs   # Colons, commas, brackets, etc.
```

#### 3. Analytics Module Split

**Before:**
```
src/analytics/mod.rs (445 lines)
```

**After:**
```
src/
├── analytics.rs          # Main module
├── analytics_types.rs   # LintAnalytics, AnalyticsReport, etc.
├── performance_tracker.rs # Performance monitoring
├── optimization_engine.rs # Suggestion generation
└── analytics_reports.rs  # Report formatting
```

#### 4. ML Module Organization

**Before:**
```
src/ml/mod.rs (429 lines)
```

**After:**
```
src/
├── ml.rs                # Main module
├── pattern_learner.rs   # PatternLearner implementation
├── ml_types.rs          # PatternInfo, etc.
└── config_generator.rs  # Configuration generation logic
```

#### 5. Diff Module Split

**Before:**
```
src/diff/mod.rs (449 lines)
```

**After:**
```
src/
├── diff.rs              # Main module
├── diff_types.rs        # GitDiff, ChangedRange, etc.
├── diff_linter.rs       # DiffLinter implementation
└── git_integration.rs   # Git operations
```

### Implementation Plan

#### Phase 1: Establish New Structure
1. Create new individual `.rs` files for each logical component
2. Move appropriate code sections to new files
3. Update module declarations and imports
4. Ensure all tests still pass

#### Phase 2: Refine Boundaries
1. Review code organization for logical consistency
2. Merge overly granular files if needed
3. Split files that are still too large (>200 lines as guideline)
4. Optimize import paths and re-exports

#### Phase 3: Clean Up
1. Remove old `mod.rs` files
2. Update documentation and examples
3. Run full test suite and linting
4. Verify build performance impact

### File Size Guidelines

- **Individual files**: Target 50-200 lines
- **Maximum file size**: 300 lines (exceptions for complex algorithms)
- **Minimum file size**: 20 lines (avoid over-fragmentation)

### Module Re-export Strategy

Each main module file (e.g., `policy.rs`) should:
1. Import from sub-modules
2. Re-export public APIs
3. Provide module-level documentation
4. Keep implementation details private

**Example:**
```rust
// src/policy.rs
//! Team policy management system

mod policy_types;
mod policy_manager;
mod policy_validation;
mod policy_reports;

pub use policy_types::*;
pub use policy_manager::PolicyManager;
pub use policy_validation::validate_policy;
pub use policy_reports::generate_report;

// Private implementation details stay private
use policy_validation::internal_validation_helper;
```

### Benefits of This Approach

1. **Improved Maintainability**: Smaller, focused files are easier to understand and modify
2. **Better Git History**: Changes are more isolated, reducing merge conflicts
3. **Enhanced Discoverability**: Clear file names make finding functionality intuitive
4. **Reduced Cognitive Load**: Developers can focus on one aspect at a time
5. **Easier Testing**: Focused modules are easier to test in isolation
6. **Better IDE Support**: Smaller files improve editor performance and navigation

### Migration Checklist

- [ ] Create new file structure
- [ ] Move code to appropriate files
- [ ] Update module declarations
- [ ] Fix import paths
- [ ] Update tests
- [ ] Verify documentation
- [ ] Run full test suite
- [ ] Check build performance
- [ ] Update CI/CD if needed
- [ ] Remove old `mod.rs` files

### Potential Risks & Mitigation

**Risk**: Breaking existing imports
**Mitigation**: Use re-exports to maintain backward compatibility during transition

**Risk**: Over-fragmentation
**Mitigation**: Follow file size guidelines and logical grouping principles

**Risk**: Increased compile time
**Mitigation**: Monitor build performance and adjust granularity if needed

**Risk**: Complex interdependencies
**Mitigation**: Carefully design module boundaries to minimize circular dependencies

## Conclusion

This refactoring will transform the codebase from a collection of monolithic `mod.rs` files into a well-organized, maintainable module structure. The new organization will improve developer productivity, code quality, and long-term maintainability while following Rust best practices for module organization.
