use super::{ConfigValue, Rule, RuleConfig};
use crate::linter::{Level, LintContext, Problem};
use eyre::Result;
use std::collections::{HashMap, HashSet};

/// Rule that detects duplicate keys in YAML mappings
#[derive(Debug)]
pub struct KeyDuplicatesRule;

impl KeyDuplicatesRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for KeyDuplicatesRule {
    fn id(&self) -> &'static str {
        "key-duplicates"
    }

    fn description(&self) -> &'static str {
        "Forbids duplications of a particular key"
    }

    fn check(&self, context: &LintContext, _config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        // Parse line by line to detect duplicate keys before serde_yaml processes them
        self.check_duplicates_in_text(context, &mut problems)?;

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        RuleConfig::new(false, Level::Error) // Disabled by default for backward compatibility
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

impl KeyDuplicatesRule {
    fn check_duplicates_in_text(
        &self,
        context: &LintContext,
        problems: &mut Vec<Problem>,
    ) -> Result<()> {
        let mut current_level_keys: Vec<HashMap<String, usize>> = vec![HashMap::new()];
        let mut indent_stack = vec![0];

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Calculate indentation level
            let indent = line.len() - line.trim_start().len();

            // Adjust the stack based on indentation
            while indent_stack.len() > 1 && indent <= indent_stack[indent_stack.len() - 1] {
                indent_stack.pop();
                current_level_keys.pop();
            }

            if indent > indent_stack[indent_stack.len() - 1] {
                indent_stack.push(indent);
                current_level_keys.push(HashMap::new());
            }

            // Look for key-value pairs
            if let Some(colon_pos) = line.find(':') {
                let key_part = line[..colon_pos].trim();

                // Skip if this looks like a list item or complex key
                if key_part.starts_with('-') || key_part.contains('[') || key_part.contains('{') {
                    continue;
                }

                // Extract the key name (handle quoted keys)
                let key = if (key_part.starts_with('"') && key_part.ends_with('"'))
                    || (key_part.starts_with('\'') && key_part.ends_with('\''))
                {
                    key_part[1..key_part.len() - 1].to_string()
                } else {
                    key_part.to_string()
                };

                if !key.is_empty() {
                    let current_keys = current_level_keys.last_mut().unwrap();

                    if let Some(&first_line) = current_keys.get(&key) {
                        // Found duplicate key
                        problems.push(Problem::new(
                            line_number,
                            colon_pos + 1,
                            Level::Error,
                            self.id(),
                            format!("found duplicate key \"{key}\" (first occurrence at line {first_line})"),
                        ));
                    } else {
                        current_keys.insert(key, line_number);
                    }
                }
            }
        }

        Ok(())
    }
}

/// Rule that validates document structure (start/end markers)
#[derive(Debug)]
pub struct DocumentStructureRule;

impl DocumentStructureRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for DocumentStructureRule {
    fn id(&self) -> &'static str {
        "document-structure"
    }

    fn description(&self) -> &'static str {
        "Requires document start and end markers"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let require_start = config.get_bool("require-document-start").unwrap_or(true);
        let require_end = config.get_bool("require-document-end").unwrap_or(false);

        let lines: Vec<&str> = context.content.lines().collect();

        if require_start {
            let has_start = lines.first().is_some_and(|line| line.trim() == "---");
            if !has_start {
                problems.push(Problem::new(
                    1,
                    1,
                    Level::Error,
                    self.id(),
                    "missing document start \"---\"".to_string(),
                ));
            }
        }

        if require_end {
            let has_end = lines.last().is_some_and(|line| {
                let trimmed = line.trim();
                trimmed == "..." || trimmed == "---"
            });
            if !has_end {
                problems.push(Problem::new(
                    lines.len(),
                    1,
                    Level::Error,
                    self.id(),
                    "missing document end \"...\" or \"---\"".to_string(),
                ));
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default for backward compatibility
        config.set_param(
            "require-document-start".to_string(),
            ConfigValue::Bool(true),
        );
        config.set_param("require-document-end".to_string(), ConfigValue::Bool(false));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

/// Rule that validates YAML anchors and aliases
#[derive(Debug)]
pub struct AnchorsRule;

impl AnchorsRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for AnchorsRule {
    fn id(&self) -> &'static str {
        "anchors"
    }

    fn description(&self) -> &'static str {
        "Validates YAML anchors and aliases"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let forbid_undeclared_aliases =
            config.get_bool("forbid-undeclared-aliases").unwrap_or(true);
        let forbid_duplicated_anchors = config
            .get_bool("forbid-duplicated-anchors")
            .unwrap_or(false);
        let forbid_unused_anchors = config.get_bool("forbid-unused-anchors").unwrap_or(false);

        let mut anchors = HashSet::new();
        let mut aliases = HashSet::new();
        let mut anchor_lines = HashMap::new();

        // Parse the content line by line to find anchors and aliases
        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;

            // Look for anchors (&anchor_name)
            if let Some(anchor_pos) = line.find('&')
                && let Some(anchor_name) = self.extract_anchor_name(&line[anchor_pos..])
            {
                if forbid_duplicated_anchors && anchors.contains(&anchor_name) {
                    problems.push(Problem::new(
                        line_number,
                        anchor_pos + 1,
                        Level::Error,
                        self.id(),
                        format!("found duplicate anchor \"{anchor_name}\""),
                    ));
                }
                anchors.insert(anchor_name.clone());
                anchor_lines.insert(anchor_name, line_number);
            }

            // Look for aliases (*alias_name)
            if let Some(alias_pos) = line.find('*')
                && let Some(alias_name) = self.extract_alias_name(&line[alias_pos..])
            {
                aliases.insert(alias_name.clone());

                if forbid_undeclared_aliases && !anchors.contains(&alias_name) {
                    problems.push(Problem::new(
                        line_number,
                        alias_pos + 1,
                        Level::Error,
                        self.id(),
                        format!("found undefined alias \"{alias_name}\""),
                    ));
                }
            }
        }

        // Check for unused anchors
        if forbid_unused_anchors {
            for anchor in &anchors {
                if !aliases.contains(anchor)
                    && let Some(&line_number) = anchor_lines.get(anchor)
                {
                    problems.push(Problem::new(
                        line_number,
                        1,
                        Level::Warning,
                        self.id(),
                        format!("found undefined anchor \"{anchor}\""),
                    ));
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default for backward compatibility
        config.set_param(
            "forbid-undeclared-aliases".to_string(),
            ConfigValue::Bool(true),
        );
        config.set_param(
            "forbid-duplicated-anchors".to_string(),
            ConfigValue::Bool(false),
        );
        config.set_param(
            "forbid-unused-anchors".to_string(),
            ConfigValue::Bool(false),
        );
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

impl AnchorsRule {
    fn extract_anchor_name(&self, text: &str) -> Option<String> {
        // Extract anchor name from &anchor_name
        if let Some(name_part) = text.strip_prefix('&') {
            let end = name_part
                .find(|c: char| c.is_whitespace() || c == ':' || c == ',' || c == ']' || c == '}')
                .unwrap_or(name_part.len());
            if end > 0 {
                Some(name_part[..end].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn extract_alias_name(&self, text: &str) -> Option<String> {
        // Extract alias name from *alias_name
        if let Some(name_part) = text.strip_prefix('*') {
            let end = name_part
                .find(|c: char| c.is_whitespace() || c == ':' || c == ',' || c == ']' || c == '}')
                .unwrap_or(name_part.len());
            if end > 0 {
                Some(name_part[..end].to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// Enhanced YAML syntax rule that catches parsing errors and syntax issues
#[derive(Debug)]
pub struct YamlSyntaxRule;

impl YamlSyntaxRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for YamlSyntaxRule {
    fn id(&self) -> &'static str {
        "yaml-syntax"
    }

    fn description(&self) -> &'static str {
        "Validates YAML syntax and catches parsing errors"
    }

    fn check(&self, context: &LintContext, _config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        // Try to parse the YAML and catch syntax errors
        match serde_yaml::from_str::<serde_yaml::Value>(context.content) {
            Ok(_) => {
                // YAML parsed successfully, check for other syntax issues
                self.check_syntax_issues(context, &mut problems);
            }
            Err(e) => {
                // Parse error occurred
                let error_msg = e.to_string();
                let (line, column) = self.extract_error_position(&error_msg);

                problems.push(Problem::new(
                    line,
                    column,
                    Level::Error,
                    self.id(),
                    format!("syntax error: {}", self.clean_error_message(&error_msg)),
                ));
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        RuleConfig::new(true, Level::Error) // Enabled by default
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

impl YamlSyntaxRule {
    fn extract_error_position(&self, error_msg: &str) -> (usize, usize) {
        // Try to extract line and column from error message
        // serde_yaml error format: "... at line X column Y"
        if let Some(line_pos) = error_msg.find("line ")
            && let Some(col_pos) = error_msg.find(" column ")
        {
            let line_str = &error_msg[line_pos + 5..col_pos];
            let col_str = &error_msg[col_pos + 8..];

            let line = line_str.parse::<usize>().unwrap_or(1);
            let column = col_str
                .split_whitespace()
                .next()
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);

            return (line, column);
        }
        (1, 1) // Default position
    }

    fn clean_error_message(&self, error_msg: &str) -> String {
        // Clean up the error message to make it more user-friendly
        error_msg
            .replace("invalid type: ", "")
            .replace("expected ", "")
            .split(" at line")
            .next()
            .unwrap_or(error_msg)
            .to_string()
    }

    fn check_syntax_issues(&self, context: &LintContext, problems: &mut Vec<Problem>) {
        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;

            // Check for common syntax issues
            if line.contains('\t') && line.trim_start().starts_with('\t') {
                problems.push(Problem::new(
                    line_number,
                    line.find('\t').unwrap() + 1,
                    Level::Warning,
                    self.id(),
                    "found tab character in indentation".to_string(),
                ));
            }

            // Check for trailing tabs
            if line.ends_with('\t') {
                problems.push(Problem::new(
                    line_number,
                    line.len(),
                    Level::Warning,
                    self.id(),
                    "found trailing tab character".to_string(),
                ));
            }
        }
    }
}

/// Rule that validates comment formatting
#[derive(Debug)]
pub struct CommentsRule;

impl CommentsRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for CommentsRule {
    fn id(&self) -> &'static str {
        "comments"
    }

    fn description(&self) -> &'static str {
        "Controls comment formatting and placement"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let require_starting_space = config.get_bool("require-starting-space").unwrap_or(true);
        let min_spaces_from_content =
            config.get_int("min-spaces-from-content").unwrap_or(2) as usize;

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;

            if let Some(hash_pos) = line.find('#') {
                // Check if this is a comment (not in a string)
                if self.is_real_comment(line, hash_pos) {
                    let comment_part = &line[hash_pos..];

                    // Check for space after #
                    if require_starting_space && comment_part.len() > 1 {
                        let next_char = comment_part.chars().nth(1).unwrap();
                        if next_char != ' ' && next_char != '\t' {
                            problems.push(Problem::new(
                                line_number,
                                hash_pos + 2,
                                Level::Error,
                                self.id(),
                                "missing starting space in comment".to_string(),
                            ));
                        }
                    }

                    // Check spacing from content (inline comments)
                    if hash_pos > 0 {
                        let content_before = &line[..hash_pos];
                        if !content_before.trim().is_empty() {
                            let spaces_before =
                                content_before.len() - content_before.trim_end().len();
                            if spaces_before < min_spaces_from_content {
                                problems.push(Problem::new(
                                    line_number,
                                    hash_pos + 1,
                                    Level::Error,
                                    self.id(),
                                    format!(
                                        "too few spaces before comment, expected at least {min_spaces_from_content}"
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("require-starting-space", true);
        config.set_param("min-spaces-from-content", 2i64);
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

impl CommentsRule {
    fn is_real_comment(&self, line: &str, hash_pos: usize) -> bool {
        // Simple check to see if # is inside a string
        let before_hash = &line[..hash_pos];
        let single_quotes = before_hash.matches('\'').count();
        let double_quotes = before_hash.matches('"').count();

        // If we have an odd number of quotes before the #, we're likely inside a string
        single_quotes % 2 == 0 && double_quotes % 2 == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_context<'a>(content: &'a str, path: &'a PathBuf) -> LintContext<'a> {
        LintContext::new(path, content)
    }

    #[test]
    fn test_key_duplicates_rule_no_duplicates() {
        let rule = KeyDuplicatesRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("key1: value1\nkey2: value2", &path);
        let mut config = rule.default_config();
        config.enabled = true; // Enable for testing

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_key_duplicates_rule_with_duplicates() {
        let rule = KeyDuplicatesRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("key1: value1\nkey1: value2", &path);
        let mut config = rule.default_config();
        config.enabled = true; // Enable for testing

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].rule, "key-duplicates");
        assert!(problems[0].message.contains("duplicate key"));
    }

    #[test]
    fn test_document_structure_rule_missing_start() {
        let rule = DocumentStructureRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("key: value", &path);
        let mut config = rule.default_config();
        config.enabled = true; // Enable for testing

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].rule, "document-structure");
        assert!(problems[0].message.contains("missing document start"));
    }

    #[test]
    fn test_document_structure_rule_with_start() {
        let rule = DocumentStructureRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("---\nkey: value", &path);
        let mut config = rule.default_config();
        config.enabled = true; // Enable for testing

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_anchors_rule_valid_anchor_alias() {
        let rule = AnchorsRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("anchor: &my_anchor value\nalias: *my_anchor", &path);
        let mut config = rule.default_config();
        config.enabled = true; // Enable for testing

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_anchors_rule_undefined_alias() {
        let rule = AnchorsRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("alias: *undefined_anchor", &path);
        let mut config = rule.default_config();
        config.enabled = true; // Enable for testing

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].rule, "anchors");
        assert!(problems[0].message.contains("undefined alias"));
    }

    #[test]
    fn test_anchors_rule_duplicate_anchor() {
        let rule = AnchorsRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context(
            "anchor1: &my_anchor value1\nanchor2: &my_anchor value2",
            &path,
        );
        let mut config = rule.default_config();
        config.enabled = true; // Enable for testing
        config.set_param(
            "forbid-duplicated-anchors".to_string(),
            ConfigValue::Bool(true),
        );

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].rule, "anchors");
        assert!(problems[0].message.contains("duplicate anchor"));
    }
}
