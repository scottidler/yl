use std::path::Path;
use serde_yaml::Value;

/// Context information available to rules during linting
#[derive(Debug)]
#[allow(dead_code)] // Fields are part of API for future phases
pub struct LintContext<'a> {
    /// Path to the file being linted
    pub file_path: &'a Path,
    /// Content of the file being linted
    pub content: &'a str,
    /// Current line number being processed (1-based)
    pub current_line: usize,
    /// Path within the YAML structure (e.g., ["spec", "containers", "0", "name"])
    pub yaml_path: Vec<String>,
    /// Parsed YAML value (if parsing succeeded)
    pub yaml_value: Option<Value>,
}

#[allow(dead_code)] // Methods are part of API for future phases
impl<'a> LintContext<'a> {
    /// Create a new lint context
    pub fn new(file_path: &'a Path, content: &'a str) -> Self {
        let yaml_value = serde_yaml::from_str(content).ok();
        Self {
            file_path,
            content,
            current_line: 0,
            yaml_path: Vec::new(),
            yaml_value,
        }
    }

    /// Get the file name as a string
    pub fn file_name(&self) -> &str {
        self.file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>")
    }

    /// Get the lines of the content as an iterator
    pub fn lines(&self) -> impl Iterator<Item = (usize, &str)> {
        self.content.lines().enumerate().map(|(i, line)| (i + 1, line))
    }

    /// Get a specific line by number (1-based)
    pub fn get_line(&self, line_number: usize) -> Option<&str> {
        if line_number == 0 {
            return None;
        }
        self.content.lines().nth(line_number - 1)
    }

    /// Get the total number of lines
    pub fn line_count(&self) -> usize {
        self.content.lines().count()
    }

    /// Check if the current YAML path matches a pattern
    /// Pattern examples: "spec.containers.*", "metadata.name"
    pub fn yaml_path_matches(&self, pattern: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('.').collect();

        if pattern_parts.len() != self.yaml_path.len() {
            return false;
        }

        pattern_parts
            .iter()
            .zip(self.yaml_path.iter())
            .all(|(pattern_part, path_part)| {
                pattern_part == &"*" || pattern_part == path_part
            })
    }

    /// Get the current YAML path as a dot-separated string
    pub fn yaml_path_string(&self) -> String {
        self.yaml_path.join(".")
    }

    /// Check if YAML parsing was successful
    pub fn has_valid_yaml(&self) -> bool {
        self.yaml_value.is_some()
    }

    /// Get the parsed YAML value
    pub fn yaml(&self) -> Option<&Value> {
        self.yaml_value.as_ref()
    }

    /// Navigate to a specific path in the YAML structure
    pub fn get_yaml_at_path(&self, path: &[&str]) -> Option<&Value> {
        let mut current = self.yaml()?;
        for segment in path {
            match current {
                Value::Mapping(map) => {
                    current = map.get(&Value::String(segment.to_string()))?;
                }
                Value::Sequence(seq) => {
                    if let Ok(index) = segment.parse::<usize>() {
                        current = seq.get(index)?;
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        Some(current)
    }

    /// Check if the current YAML path contains duplicate keys
    pub fn has_duplicate_keys(&self) -> Vec<String> {
        let mut duplicates = Vec::new();
        if let Some(Value::Mapping(map)) = self.yaml() {
            let mut seen_keys = std::collections::HashSet::new();
            for key in map.keys() {
                if let Value::String(key_str) = key {
                    if !seen_keys.insert(key_str.clone()) {
                        duplicates.push(key_str.clone());
                    }
                }
            }
        }
        duplicates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_context_creation() {
        let path = PathBuf::from("test.yaml");
        let content = "key: value\nother: data";
        let context = LintContext::new(&path, content);

        assert_eq!(context.file_path, &path);
        assert_eq!(context.content, content);
        assert_eq!(context.current_line, 0);
        assert!(context.yaml_path.is_empty());
    }

    #[test]
    fn test_file_name() {
        let path = PathBuf::from("/path/to/test.yaml");
        let content = "";
        let context = LintContext::new(&path, content);

        assert_eq!(context.file_name(), "test.yaml");
    }

    #[test]
    fn test_file_name_unknown() {
        let path = PathBuf::from("");
        let content = "";
        let context = LintContext::new(&path, content);

        assert_eq!(context.file_name(), "<unknown>");
    }

    #[test]
    fn test_lines_iterator() {
        let path = PathBuf::from("test.yaml");
        let content = "line1\nline2\nline3";
        let context = LintContext::new(&path, content);

        let lines: Vec<(usize, &str)> = context.lines().collect();
        assert_eq!(lines, vec![(1, "line1"), (2, "line2"), (3, "line3")]);
    }

    #[test]
    fn test_get_line() {
        let path = PathBuf::from("test.yaml");
        let content = "line1\nline2\nline3";
        let context = LintContext::new(&path, content);

        assert_eq!(context.get_line(1), Some("line1"));
        assert_eq!(context.get_line(2), Some("line2"));
        assert_eq!(context.get_line(3), Some("line3"));
        assert_eq!(context.get_line(0), None);
        assert_eq!(context.get_line(4), None);
    }

    #[test]
    fn test_line_count() {
        let path = PathBuf::from("test.yaml");
        let content = "line1\nline2\nline3";
        let context = LintContext::new(&path, content);

        assert_eq!(context.line_count(), 3);
    }

    #[test]
    fn test_line_count_empty() {
        let path = PathBuf::from("test.yaml");
        let content = "";
        let context = LintContext::new(&path, content);

        assert_eq!(context.line_count(), 0);
    }

    #[test]
    fn test_yaml_path_matches() {
        let path = PathBuf::from("test.yaml");
        let content = "";
        let mut context = LintContext::new(&path, content);

        context.yaml_path = vec!["spec".to_string(), "containers".to_string(), "0".to_string()];

        assert!(context.yaml_path_matches("spec.containers.0"));
        assert!(context.yaml_path_matches("spec.containers.*"));
        assert!(context.yaml_path_matches("spec.*.0"));
        assert!(context.yaml_path_matches("*.*.*"));

        assert!(!context.yaml_path_matches("spec.containers"));
        assert!(!context.yaml_path_matches("spec.containers.0.name"));
        assert!(!context.yaml_path_matches("metadata.name.test"));
    }

    #[test]
    fn test_yaml_path_string() {
        let path = PathBuf::from("test.yaml");
        let content = "";
        let mut context = LintContext::new(&path, content);

        context.yaml_path = vec!["spec".to_string(), "containers".to_string(), "0".to_string()];
        assert_eq!(context.yaml_path_string(), "spec.containers.0");

        context.yaml_path.clear();
        assert_eq!(context.yaml_path_string(), "");
    }
}
