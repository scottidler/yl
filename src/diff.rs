//! Diff-aware linting for CI/CD optimization
//!
//! This module provides functionality to lint only the changed parts of files,
//! making it ideal for CI/CD pipelines where you only want to check modifications.

pub use crate::diff_types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn test_diff_linter_creation() {
        let config = Config::default();
        let linter = DiffLinter::new(config);
        assert_eq!(linter.context_lines, 3);
    }

    #[test]
    fn test_with_context_lines() {
        let config = Config::default();
        let linter = DiffLinter::new(config).with_context_lines(5);
        assert_eq!(linter.context_lines, 5);
    }

    #[test]
    fn test_calculate_diff_simple() {
        let config = Config::default();
        let linter = DiffLinter::new(config);

        let old_content = "line1\nline2\nline3";
        let new_content = "line1\nmodified line2\nline3";

        let ranges = linter.calculate_diff(old_content, new_content).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 2);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn test_calculate_diff_addition() {
        let config = Config::default();
        let linter = DiffLinter::new(config);

        let old_content = "line1\nline2";
        let new_content = "line1\nline2\nline3";

        let ranges = linter.calculate_diff(old_content, new_content).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 3);
        assert_eq!(ranges[0].end_line, 3);
    }

    #[test]
    fn test_changed_range() {
        let range = ChangedRange {
            start_line: 5,
            end_line: 10,
            change_type: ChangeType::Modified,
        };

        assert_eq!(range.start_line, 5);
        assert_eq!(range.end_line, 10);
        assert_eq!(range.change_type, ChangeType::Modified);
    }

    #[test]
    fn test_git_diff_struct() {
        use std::path::PathBuf;

        let git_diff = GitDiff {
            file_path: PathBuf::from("test.yaml"),
            is_new_file: true,
            is_deleted_file: false,
        };

        assert_eq!(git_diff.file_path, PathBuf::from("test.yaml"));
        assert!(git_diff.is_new_file);
        assert!(!git_diff.is_deleted_file);
    }
}
