use crate::config::Config;
use crate::linter::{Linter, Problem};
use eyre::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Diff-aware linter that only lints changed lines and their context
pub struct DiffLinter {
    base_linter: Linter,
    context_lines: usize,
}

/// Represents a changed line range in a file
#[derive(Debug, Clone, PartialEq)]
pub struct ChangedRange {
    pub start_line: usize,
    pub end_line: usize,
    pub change_type: ChangeType,
}

/// Type of change in a diff
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Modified,
}

/// Git diff information
#[derive(Debug, Clone)]
pub struct GitDiff {
    pub file_path: PathBuf,
    pub is_new_file: bool,
    pub is_deleted_file: bool,
}

impl DiffLinter {
    /// Create a new diff-aware linter
    pub fn new(config: Config) -> Self {
        Self {
            base_linter: Linter::new(config),
            context_lines: 3, // Default context lines
        }
    }

    /// Set the number of context lines to include around changes
    pub fn with_context_lines(mut self, context_lines: usize) -> Self {
        self.context_lines = context_lines;
        self
    }

    /// Lint only the changed lines in the provided content
    pub fn lint_diff(&self, old_content: &str, new_content: &str, file_path: &Path) -> Result<Vec<Problem>> {
        // Calculate the diff between old and new content
        let changed_ranges = self.calculate_diff(old_content, new_content)?;

        // Get all problems from the new content
        let all_problems = self.base_linter.lint_content(file_path, new_content)?;

        // Filter problems to only include those in changed areas
        let filtered_problems = self.filter_problems_by_changes(&all_problems, &changed_ranges);

        Ok(filtered_problems)
    }

    /// Lint changes in a git working directory
    pub fn lint_git_diff<P: AsRef<Path>>(&self, repo_path: P) -> Result<Vec<(PathBuf, Vec<Problem>)>> {
        let repo_path = repo_path.as_ref();
        let git_diffs = self.get_git_diff(repo_path)?;

        let mut results = Vec::new();

        for git_diff in git_diffs {
            let file_path = repo_path.join(&git_diff.file_path);

            // Skip deleted files
            if git_diff.is_deleted_file {
                continue;
            }

            // For new files, lint the entire file
            if git_diff.is_new_file {
                if file_path.exists() {
                    let problems = self.base_linter.lint_file(&file_path)?;
                    results.push((git_diff.file_path, problems));
                }
                continue;
            }

            // For modified files, lint only changed areas
            if file_path.exists() {
                let current_content = std::fs::read_to_string(&file_path)?;
                let old_content = self.get_git_file_content(repo_path, &git_diff.file_path, "HEAD")?;

                let problems = self.lint_diff(&old_content, &current_content, &file_path)?;
                if !problems.is_empty() {
                    results.push((git_diff.file_path, problems));
                }
            }
        }

        Ok(results)
    }

    /// Lint changes in a git commit
    pub fn lint_git_commit<P: AsRef<Path>>(&self, repo_path: P, commit_hash: &str) -> Result<Vec<(PathBuf, Vec<Problem>)>> {
        let repo_path = repo_path.as_ref();
        let git_diffs = self.get_git_commit_diff(repo_path, commit_hash)?;

        let mut results = Vec::new();

        for git_diff in git_diffs {
            // Skip deleted files
            if git_diff.is_deleted_file {
                continue;
            }

            let new_content = self.get_git_file_content(repo_path, &git_diff.file_path, commit_hash)?;

            if git_diff.is_new_file {
                // For new files, lint the entire content
                let problems = self.base_linter.lint_content(&git_diff.file_path, &new_content)?;
                if !problems.is_empty() {
                    results.push((git_diff.file_path, problems));
                }
            } else {
                // For modified files, lint only changed areas
                let old_content = self.get_git_file_content(repo_path, &git_diff.file_path, &format!("{}^", commit_hash))?;
                let problems = self.lint_diff(&old_content, &new_content, &git_diff.file_path)?;
                if !problems.is_empty() {
                    results.push((git_diff.file_path, problems));
                }
            }
        }

        Ok(results)
    }

    /// Calculate diff between two content strings
    fn calculate_diff(&self, old_content: &str, new_content: &str) -> Result<Vec<ChangedRange>> {
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();

        let mut changed_ranges = Vec::new();
        let mut i = 0;
        let mut j = 0;

        while i < old_lines.len() || j < new_lines.len() {
            if i < old_lines.len() && j < new_lines.len() && old_lines[i] == new_lines[j] {
                // Lines are the same, move forward
                i += 1;
                j += 1;
            } else {
                // Found a difference, determine the range
                let start_line = j + 1; // 1-based line numbers
                let mut end_line = start_line;

                // Skip different lines in old content
                while i < old_lines.len() && (j >= new_lines.len() || old_lines[i] != new_lines[j]) {
                    i += 1;
                }

                // Skip different lines in new content
                while j < new_lines.len() && (i >= old_lines.len() || old_lines[i] != new_lines[j]) {
                    j += 1;
                    end_line = j; // 1-based line numbers
                }

                if end_line >= start_line {
                    changed_ranges.push(ChangedRange {
                        start_line,
                        end_line,
                        change_type: ChangeType::Modified,
                    });
                }
            }
        }

        Ok(changed_ranges)
    }

    /// Filter problems to only include those in changed areas
    fn filter_problems_by_changes(&self, problems: &[Problem], changed_ranges: &[ChangedRange]) -> Vec<Problem> {
        let mut relevant_lines = HashSet::new();

        // Collect all lines that should be checked (changed lines + context)
        for range in changed_ranges {
            let context_start = range.start_line.saturating_sub(self.context_lines);
            let context_end = range.end_line + self.context_lines;

            for line_num in context_start..=context_end {
                if line_num > 0 {
                    relevant_lines.insert(line_num);
                }
            }
        }

        // Filter problems to only include those on relevant lines
        problems
            .iter()
            .filter(|problem| relevant_lines.contains(&problem.line))
            .cloned()
            .collect()
    }

    /// Get git diff for working directory changes
    fn get_git_diff<P: AsRef<Path>>(&self, repo_path: P) -> Result<Vec<GitDiff>> {
        let output = Command::new("git")
            .args(&["diff", "--name-status"])
            .current_dir(repo_path.as_ref())
            .output()?;

        if !output.status.success() {
            return Err(eyre::eyre!("Git diff command failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let diff_output = String::from_utf8(output.stdout)?;
        self.parse_git_diff_output(&diff_output, repo_path.as_ref())
    }

    /// Get git diff for a specific commit
    fn get_git_commit_diff<P: AsRef<Path>>(&self, repo_path: P, commit_hash: &str) -> Result<Vec<GitDiff>> {
        let output = Command::new("git")
            .args(&["diff", "--name-status", &format!("{}^", commit_hash), commit_hash])
            .current_dir(repo_path.as_ref())
            .output()?;

        if !output.status.success() {
            return Err(eyre::eyre!("Git diff command failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let diff_output = String::from_utf8(output.stdout)?;
        self.parse_git_diff_output(&diff_output, repo_path.as_ref())
    }

    /// Parse git diff output
    fn parse_git_diff_output(&self, output: &str, repo_path: &Path) -> Result<Vec<GitDiff>> {
        let mut diffs = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let status = parts[0];
            let file_path = PathBuf::from(parts[1]);

            // Only process YAML files
            if !self.is_yaml_file(&file_path) {
                continue;
            }

            let is_new_file = status == "A";
            let is_deleted_file = status == "D";

            // Get detailed diff for the file if it's modified
            let _changed_ranges = if !is_new_file && !is_deleted_file {
                self.get_file_changed_ranges(repo_path, &file_path)?
            } else {
                Vec::new()
            };

            diffs.push(GitDiff {
                file_path,
                is_new_file,
                is_deleted_file,
            });
        }

        Ok(diffs)
    }

    /// Get changed line ranges for a specific file
    fn get_file_changed_ranges(&self, repo_path: &Path, file_path: &Path) -> Result<Vec<ChangedRange>> {
        let output = Command::new("git")
            .args(&["diff", "-U0", "--", file_path.to_string_lossy().as_ref()])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new()); // No changes or error, return empty
        }

        let diff_output = String::from_utf8(output.stdout)?;
        self.parse_unified_diff(&diff_output)
    }

    /// Parse unified diff format to extract changed ranges
    fn parse_unified_diff(&self, diff_output: &str) -> Result<Vec<ChangedRange>> {
        let mut ranges = Vec::new();

        for line in diff_output.lines() {
            if line.starts_with("@@") {
                // Parse hunk header: @@ -old_start,old_count +new_start,new_count @@
                if let Some(hunk_info) = line.split("@@").nth(1) {
                    let parts: Vec<&str> = hunk_info.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        let new_part = parts[1];
                        if let Some(new_info) = new_part.strip_prefix('+') {
                            let new_parts: Vec<&str> = new_info.split(',').collect();
                            if let Ok(start_line) = new_parts[0].parse::<usize>() {
                                let count = if new_parts.len() > 1 {
                                    new_parts[1].parse::<usize>().unwrap_or(1)
                                } else {
                                    1
                                };

                                if count > 0 {
                                    ranges.push(ChangedRange {
                                        start_line,
                                        end_line: start_line + count - 1,
                                        change_type: ChangeType::Modified,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(ranges)
    }

    /// Get file content from git at a specific revision
    fn get_git_file_content(&self, repo_path: &Path, file_path: &Path, revision: &str) -> Result<String> {
        let output = Command::new("git")
            .args(&["show", &format!("{}:{}", revision, file_path.to_string_lossy())])
            .current_dir(repo_path)
            .output()?;

        if !output.status.success() {
            return Err(eyre::eyre!("Failed to get git file content: {}", String::from_utf8_lossy(&output.stderr)));
        }

        Ok(String::from_utf8(output.stdout)?)
    }

    /// Check if a file is a YAML file
    fn is_yaml_file(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            matches!(extension.to_str(), Some("yaml") | Some("yml"))
        } else {
            false
        }
    }
}

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

        let old_content = "line1\nline2\nline3\n";
        let new_content = "line1\nmodified line2\nline3\n";

        let ranges = linter.calculate_diff(old_content, new_content).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 2);
        assert_eq!(ranges[0].end_line, 3);
    }

    #[test]
    fn test_calculate_diff_addition() {
        let config = Config::default();
        let linter = DiffLinter::new(config);

        let old_content = "line1\nline3\n";
        let new_content = "line1\nline2\nline3\n";

        let ranges = linter.calculate_diff(old_content, new_content).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 2);
        assert_eq!(ranges[0].end_line, 3);
    }

    #[test]
    fn test_filter_problems_by_changes() {
        let config = Config::default();
        let linter = DiffLinter::new(config);

        let problems = vec![
            Problem::new(1, 1, crate::linter::Level::Error, "rule1", "error on line 1"),
            Problem::new(5, 1, crate::linter::Level::Error, "rule2", "error on line 5"),
            Problem::new(10, 1, crate::linter::Level::Error, "rule3", "error on line 10"),
        ];

        let changed_ranges = vec![
            ChangedRange {
                start_line: 5,
                end_line: 5,
                change_type: ChangeType::Modified,
            }
        ];

        let filtered = linter.filter_problems_by_changes(&problems, &changed_ranges);

        // Should include line 5 and context lines (2-8 with context_lines=3)
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].line, 5);
    }

    #[test]
    fn test_parse_unified_diff() {
        let config = Config::default();
        let linter = DiffLinter::new(config);

        let diff_output = "@@ -1,3 +1,4 @@\n line1\n+added line\n line2\n line3\n";

        let ranges = linter.parse_unified_diff(diff_output).unwrap();
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 1);
        assert_eq!(ranges[0].end_line, 4);
    }

    #[test]
    fn test_is_yaml_file() {
        let config = Config::default();
        let linter = DiffLinter::new(config);

        assert!(linter.is_yaml_file(Path::new("test.yaml")));
        assert!(linter.is_yaml_file(Path::new("test.yml")));
        assert!(!linter.is_yaml_file(Path::new("test.txt")));
        assert!(!linter.is_yaml_file(Path::new("test")));
    }
}
