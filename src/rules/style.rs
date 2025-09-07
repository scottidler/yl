use super::{Rule, RuleConfig};
use crate::linter::{Level, LintContext, Problem};
use crate::rules::common;
use eyre::Result;

/// Rule that checks line length limits
#[derive(Debug)]
pub struct LineLengthRule {
    default_max: usize,
}

#[allow(dead_code)] // Some methods are part of API for future phases
impl LineLengthRule {
    /// Create a new line length rule with default maximum of 80 characters
    pub fn new() -> Self {
        Self { default_max: 80 }
    }

    /// Create a new line length rule with a custom default maximum
    pub fn with_default_max(max: usize) -> Self {
        Self { default_max: max }
    }

    /// Get the maximum line length from configuration
    fn get_max_length(&self, config: &RuleConfig) -> usize {
        config
            .get_int("max")
            .and_then(|i| if i > 0 { Some(i as usize) } else { None })
            .unwrap_or(self.default_max)
    }

    /// Check if non-breakable words should be allowed to exceed the limit
    fn allow_non_breakable_words(&self, config: &RuleConfig) -> bool {
        config.get_bool("allow-non-breakable-words").unwrap_or(true)
    }

    /// Check if a line contains only non-breakable content
    fn is_non_breakable_line(&self, line: &str) -> bool {
        let trimmed = line.trim_start();

        // Skip comment prefix if present
        let content = if let Some(comment) = common::extract_comment(trimmed) {
            comment.trim_start_matches('#').trim_start()
        } else {
            trimmed
        };

        // Check if the line contains spaces (indicating breakable content)
        !content.contains(' ')
    }
}

impl Default for LineLengthRule {
    fn default() -> Self {
        Self::new()
    }
}

impl Rule for LineLengthRule {
    fn id(&self) -> &'static str {
        "line-length"
    }

    fn description(&self) -> &'static str {
        "Checks that lines do not exceed a maximum length"
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(true, Level::Error);
        config.set_param("max", self.default_max as i64);
        config.set_param("allow-non-breakable-words", true);
        config
    }

    fn validate_config(&self, config: &RuleConfig) -> Result<()> {
        if let Some(max) = config.get_int("max") {
            if max <= 0 {
                return Err(eyre::eyre!("max must be a positive integer, got {}", max));
            }
        }
        Ok(())
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        if !config.enabled {
            return Ok(Vec::new());
        }

        let max_length = self.get_max_length(config);
        let allow_non_breakable = self.allow_non_breakable_words(config);
        let mut problems = Vec::new();

        for (line_no, line) in context.lines() {
            let line_length = line.chars().count();

            if line_length > max_length {
                // If non-breakable words are allowed, check if this line qualifies
                let is_non_breakable = self.is_non_breakable_line(line);
                if allow_non_breakable && is_non_breakable {
                    continue;
                }

                problems.push(Problem::new(
                    line_no,
                    max_length + 1,
                    config.level.clone(),
                    self.id(),
                    format!(
                        "line too long ({} > {} characters)",
                        line_length,
                        max_length
                    ),
                ));
            }
        }

        Ok(problems)
    }
}

/// Rule that checks for trailing whitespace
#[derive(Debug, Default)]
pub struct TrailingSpacesRule;

impl TrailingSpacesRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for TrailingSpacesRule {
    fn id(&self) -> &'static str {
        "trailing-spaces"
    }

    fn description(&self) -> &'static str {
        "Checks for trailing whitespace at the end of lines"
    }

    fn default_config(&self) -> RuleConfig {
        RuleConfig::new(true, Level::Error)
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        if !config.enabled {
            return Ok(Vec::new());
        }

        let mut problems = Vec::new();

        for (line_no, line) in context.lines() {
            if common::has_trailing_whitespace(line) {
                if let Some(start_pos) = common::trailing_whitespace_start(line) {
                    problems.push(Problem::new(
                        line_no,
                        start_pos + 1, // Convert to 1-based column
                        config.level.clone(),
                        self.id(),
                        "trailing whitespace",
                    ));
                }
            }
        }

        Ok(problems)
    }
}

/// Rule that manages empty lines
#[derive(Debug, Default)]
pub struct EmptyLinesRule;

impl EmptyLinesRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for EmptyLinesRule {
    fn id(&self) -> &'static str {
        "empty-lines"
    }

    fn description(&self) -> &'static str {
        "Controls the number of empty lines"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        if !config.enabled {
            return Ok(Vec::new());
        }

        let mut problems = Vec::new();
        let max_empty = config.get_int("max").unwrap_or(2) as usize;
        let max_start = config.get_int("max-start").unwrap_or(0) as usize;
        let max_end = config.get_int("max-end").unwrap_or(1) as usize;

        let lines: Vec<&str> = context.content.lines().collect();
        if lines.is_empty() {
            return Ok(problems);
        }

        // Check empty lines at start
        let mut start_empty_count = 0;
        for line in &lines {
            if line.trim().is_empty() {
                start_empty_count += 1;
            } else {
                break;
            }
        }

        if start_empty_count > max_start {
            problems.push(Problem::new(
                1,
                1,
                config.level.clone(),
                self.id(),
                format!("too many blank lines at beginning of file ({} > {})", start_empty_count, max_start),
            ));
        }

        // Check empty lines at end
        let mut end_empty_count = 0;
        for line in lines.iter().rev() {
            if line.trim().is_empty() {
                end_empty_count += 1;
            } else {
                break;
            }
        }

        if end_empty_count > max_end {
            problems.push(Problem::new(
                lines.len(),
                1,
                config.level.clone(),
                self.id(),
                format!("too many blank lines at end of file ({} > {})", end_empty_count, max_end),
            ));
        }

        // Check consecutive empty lines in middle
        let mut consecutive_empty = 0;
        for (line_no, line) in lines.iter().enumerate() {
            if line.trim().is_empty() {
                consecutive_empty += 1;
            } else {
                if consecutive_empty > max_empty {
                    problems.push(Problem::new(
                        line_no,
                        1,
                        config.level.clone(),
                        self.id(),
                        format!("too many blank lines ({} > {})", consecutive_empty, max_empty),
                    ));
                }
                consecutive_empty = 0;
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("max", 2i64);
        config.set_param("max-start", 0i64);
        config.set_param("max-end", 1i64);
        config
    }
}

/// Rule that checks indentation consistency
#[derive(Debug, Default)]
pub struct IndentationRule;

impl IndentationRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for IndentationRule {
    fn id(&self) -> &'static str {
        "indentation"
    }

    fn description(&self) -> &'static str {
        "Controls indentation consistency"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        if !config.enabled {
            return Ok(Vec::new());
        }

        let mut problems = Vec::new();
        let spaces = config.get_int("spaces").unwrap_or(2) as usize;
        let indent_sequences = config.get_bool("indent-sequences").unwrap_or(true);
        let check_multi_line_strings = config.get_bool("check-multi-line-strings").unwrap_or(false);

        let _expected_indent = 0;
        let _in_sequence = false;

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;

            // Skip empty lines and comments unless checking multi-line strings
            if line.trim().is_empty() || (line.trim().starts_with('#') && !check_multi_line_strings) {
                continue;
            }

            let actual_indent = common::count_leading_whitespace(line);
            let trimmed = line.trim_start();

            // Check for tabs
            if line.contains('\t') {
                problems.push(Problem::new(
                    line_number,
                    line.find('\t').unwrap() + 1,
                    Level::Error,
                    self.id(),
                    "found character '\\t' instead of spaces".to_string(),
                ));
                continue;
            }

            // Determine if this is a sequence item
            let is_sequence_item = trimmed.starts_with('-') &&
                                  trimmed.len() > 1 &&
                                  trimmed.chars().nth(1).unwrap().is_whitespace();

            if is_sequence_item {
                let _in_sequence = true;
                if indent_sequences {
                    // Sequence items should be indented
                    if actual_indent % spaces != 0 {
                        problems.push(Problem::new(
                            line_number,
                            1,
                            Level::Error,
                            self.id(),
                            format!("wrong indentation: expected multiple of {}, got {}", spaces, actual_indent),
                        ));
                    }
                }
            } else {
                // Regular key-value pairs
                if actual_indent % spaces != 0 {
                    problems.push(Problem::new(
                        line_number,
                        1,
                        Level::Error,
                        self.id(),
                        format!("wrong indentation: expected multiple of {}, got {}", spaces, actual_indent),
                    ));
                }
                let _in_sequence = false;
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("spaces", 2i64);
        config.set_param("indent-sequences", true);
        config.set_param("check-multi-line-strings", false);
        config
    }
}

/// Rule that ensures files end with a newline
#[derive(Debug, Default)]
pub struct NewLineAtEndOfFileRule;

impl NewLineAtEndOfFileRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for NewLineAtEndOfFileRule {
    fn id(&self) -> &'static str {
        "new-line-at-end-of-file"
    }

    fn description(&self) -> &'static str {
        "Requires a new line character at the end of files"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        if !config.enabled {
            return Ok(Vec::new());
        }

        let mut problems = Vec::new();

        if !context.content.is_empty() && !context.content.ends_with('\n') {
            problems.push(Problem::new(
                context.line_count(),
                context.get_line(context.line_count()).map(|l| l.len()).unwrap_or(0) + 1,
                config.level.clone(),
                self.id(),
                "missing newline at end of file".to_string(),
            ));
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        RuleConfig::new(false, Level::Error) // Disabled by default
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
    fn test_line_length_rule_creation() {
        let rule = LineLengthRule::new();
        assert_eq!(rule.id(), "line-length");
        assert_eq!(rule.default_max, 80);

        let rule = LineLengthRule::with_default_max(120);
        assert_eq!(rule.default_max, 120);
    }

    #[test]
    fn test_line_length_rule_default_config() {
        let rule = LineLengthRule::new();
        let config = rule.default_config();

        assert!(config.enabled);
        assert_eq!(config.level, Level::Error);
        assert_eq!(config.get_int("max"), Some(80));
        assert_eq!(config.get_bool("allow-non-breakable-words"), Some(true));
    }

    #[test]
    fn test_line_length_rule_config_validation() {
        let rule = LineLengthRule::new();

        let mut valid_config = rule.default_config();
        valid_config.set_param("max", 100i64);
        assert!(rule.validate_config(&valid_config).is_ok());

        let mut invalid_config = rule.default_config();
        invalid_config.set_param("max", -1i64);
        assert!(rule.validate_config(&invalid_config).is_err());

        let mut zero_config = rule.default_config();
        zero_config.set_param("max", 0i64);
        assert!(rule.validate_config(&zero_config).is_err());
    }

    #[test]
    fn test_line_length_rule_check_short_lines() {
        let rule = LineLengthRule::new();
        let config = rule.default_config();
        let path = PathBuf::from("test.yaml");
        let content = "short line\nanother short line";
        let context = create_test_context(content, &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert!(problems.is_empty());
    }

    #[test]
    fn test_line_length_rule_check_long_lines() {
        let rule = LineLengthRule::new();
        let config = rule.default_config();
        let path = PathBuf::from("test.yaml");
        // Create a long line with spaces (breakable)
        let long_line = "this is a very long line with many words that definitely exceeds the eighty character limit";
        let context = create_test_context(long_line, &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
        assert_eq!(problems[0].column, 81); // max + 1
        assert_eq!(problems[0].rule, "line-length");
        assert!(problems[0].message.contains("line too long"));
    }

    #[test]
    fn test_line_length_rule_custom_max() {
        let rule = LineLengthRule::new();
        let mut config = rule.default_config();
        config.set_param("max", 50i64);

        let path = PathBuf::from("test.yaml");
        // Use a line with spaces that exceeds 50 characters
        let line = "this is a line with spaces that exceeds fifty characters";
        let context = create_test_context(line, &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].column, 51); // custom max + 1
    }

    #[test]
    fn test_line_length_rule_non_breakable_words() {
        let rule = LineLengthRule::new();
        let config = rule.default_config();

        // Long URL without spaces should be allowed
        let path = PathBuf::from("test.yaml");
        let url_line = "https://example.com/very/long/path/that/exceeds/eighty/characters/but/should/be/allowed/because/no/spaces";
        let context = create_test_context(url_line, &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert!(problems.is_empty());

        // Long line with spaces should not be allowed
        let breakable_line = "this is a very long line with many words that definitely exceeds the eighty character limit";
        let context = create_test_context(breakable_line, &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn test_line_length_rule_disabled() {
        let rule = LineLengthRule::new();
        let mut config = rule.default_config();
        config.enabled = false;

        let path = PathBuf::from("test.yaml");
        let long_line = "a".repeat(200);
        let context = create_test_context(&long_line, &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert!(problems.is_empty());
    }

    #[test]
    fn test_trailing_spaces_rule_creation() {
        let rule = TrailingSpacesRule::new();
        assert_eq!(rule.id(), "trailing-spaces");
    }

    #[test]
    fn test_trailing_spaces_rule_check_clean_lines() {
        let rule = TrailingSpacesRule::new();
        let config = rule.default_config();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("clean line\nanother clean line", &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert!(problems.is_empty());
    }

    #[test]
    fn test_trailing_spaces_rule_check_trailing_spaces() {
        let rule = TrailingSpacesRule::new();
        let config = rule.default_config();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("line with trailing spaces   \nclean line", &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].line, 1);
        assert_eq!(problems[0].rule, "trailing-spaces");
        assert_eq!(problems[0].message, "trailing whitespace");
    }

    #[test]
    fn test_trailing_spaces_rule_disabled() {
        let rule = TrailingSpacesRule::new();
        let mut config = rule.default_config();
        config.enabled = false;

        let path = PathBuf::from("test.yaml");
        let context = create_test_context("line with trailing spaces   ", &path);

        let problems = rule.check(&context, &config).expect("Check failed");
        assert!(problems.is_empty());
    }

    #[test]
    fn test_is_non_breakable_line() {
        let rule = LineLengthRule::new();

        assert!(rule.is_non_breakable_line("https://example.com/very/long/url"));
        assert!(rule.is_non_breakable_line("  https://example.com/very/long/url"));
        assert!(rule.is_non_breakable_line("# https://example.com/very/long/url"));
        assert!(rule.is_non_breakable_line("very-long-hyphenated-identifier-without-spaces"));

        assert!(!rule.is_non_breakable_line("this has spaces"));
        assert!(!rule.is_non_breakable_line("key: value with spaces"));
        assert!(!rule.is_non_breakable_line("# comment with spaces"));
    }
}
