use super::{ConfigValue, Rule, RuleConfig};
use crate::linter::{Level, LintContext, Problem};
use eyre::Result;

/// Rule that checks bracket spacing and style
#[derive(Debug)]
pub struct BracketsRule;

impl BracketsRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for BracketsRule {
    fn id(&self) -> &'static str {
        "brackets"
    }

    fn description(&self) -> &'static str {
        "Controls the use of brackets within arrays"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let min_spaces_inside = config.get_int("min-spaces-inside").unwrap_or(0) as usize;
        let max_spaces_inside = config.get_int("max-spaces-inside").unwrap_or(1) as usize;
        let min_spaces_inside_empty = config.get_int("min-spaces-inside-empty").unwrap_or(0) as usize;
        let max_spaces_inside_empty = config.get_int("max-spaces-inside-empty").unwrap_or(0) as usize;

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;

            // Find all bracket pairs in the line
            let mut bracket_positions = Vec::new();
            let chars: Vec<char> = line.chars().collect();

            for (i, &ch) in chars.iter().enumerate() {
                if ch == '[' {
                    // Find the matching closing bracket
                    let mut depth = 1;
                    let mut j = i + 1;
                    while j < chars.len() && depth > 0 {
                        match chars[j] {
                            '[' => depth += 1,
                            ']' => depth -= 1,
                            _ => {}
                        }
                        j += 1;
                    }
                    if depth == 0 {
                        bracket_positions.push((i, j - 1));
                    }
                }
            }

            // Check spacing for each bracket pair
            for (open_pos, close_pos) in bracket_positions {
                let content_between = &chars[open_pos + 1..close_pos];
                let content_str: String = content_between.iter().collect();
                let trimmed_content = content_str.trim();

                if trimmed_content.is_empty() {
                    // Empty brackets
                    let spaces_count = content_str.len();
                    if spaces_count < min_spaces_inside_empty {
                        problems.push(Problem::new(
                            line_number,
                            open_pos + 1,
                            Level::Error,
                            self.id(),
                            format!(
                                "too few spaces inside empty brackets, expected at least {min_spaces_inside_empty}"
                            ),
                        ));
                    } else if spaces_count > max_spaces_inside_empty {
                        problems.push(Problem::new(
                            line_number,
                            open_pos + 1,
                            Level::Error,
                            self.id(),
                            format!(
                                "too many spaces inside empty brackets, expected at most {max_spaces_inside_empty}"
                            ),
                        ));
                    }
                } else {
                    // Non-empty brackets
                    let leading_spaces = content_str.len() - content_str.trim_start().len();
                    let trailing_spaces = content_str.len() - content_str.trim_end().len();

                    if leading_spaces < min_spaces_inside {
                        problems.push(Problem::new(
                            line_number,
                            open_pos + 1,
                            Level::Error,
                            self.id(),
                            format!(
                                "too few spaces inside brackets, expected at least {min_spaces_inside}"
                            ),
                        ));
                    } else if leading_spaces > max_spaces_inside {
                        problems.push(Problem::new(
                            line_number,
                            open_pos + 1,
                            Level::Error,
                            self.id(),
                            format!(
                                "too many spaces inside brackets, expected at most {max_spaces_inside}"
                            ),
                        ));
                    }

                    if trailing_spaces < min_spaces_inside {
                        problems.push(Problem::new(
                            line_number,
                            close_pos + 1,
                            Level::Error,
                            self.id(),
                            format!(
                                "too few spaces inside brackets, expected at least {min_spaces_inside}"
                            ),
                        ));
                    } else if trailing_spaces > max_spaces_inside {
                        problems.push(Problem::new(
                            line_number,
                            close_pos + 1,
                            Level::Error,
                            self.id(),
                            format!(
                                "too many spaces inside brackets, expected at most {max_spaces_inside}"
                            ),
                        ));
                    }
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("min-spaces-inside".to_string(), ConfigValue::Int(0));
        config.set_param("max-spaces-inside".to_string(), ConfigValue::Int(1));
        config.set_param("min-spaces-inside-empty".to_string(), ConfigValue::Int(0));
        config.set_param("max-spaces-inside-empty".to_string(), ConfigValue::Int(0));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

/// Rule that checks brace spacing and style
#[derive(Debug)]
pub struct BracesRule;

impl BracesRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for BracesRule {
    fn id(&self) -> &'static str {
        "braces"
    }

    fn description(&self) -> &'static str {
        "Controls the use of braces within mappings"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let min_spaces_inside = config.get_int("min-spaces-inside").unwrap_or(0) as usize;
        let max_spaces_inside = config.get_int("max-spaces-inside").unwrap_or(1) as usize;
        let min_spaces_inside_empty = config.get_int("min-spaces-inside-empty").unwrap_or(0) as usize;
        let max_spaces_inside_empty = config.get_int("max-spaces-inside-empty").unwrap_or(0) as usize;

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;

            // Find all brace pairs in the line
            let mut brace_positions = Vec::new();
            let chars: Vec<char> = line.chars().collect();

            for (i, &ch) in chars.iter().enumerate() {
                if ch == '{' {
                    // Find the matching closing brace
                    let mut depth = 1;
                    let mut j = i + 1;
                    while j < chars.len() && depth > 0 {
                        match chars[j] {
                            '{' => depth += 1,
                            '}' => depth -= 1,
                            _ => {}
                        }
                        j += 1;
                    }
                    if depth == 0 {
                        brace_positions.push((i, j - 1));
                    }
                }
            }

            // Check spacing for each brace pair
            for (open_pos, close_pos) in brace_positions {
                let content_between = &chars[open_pos + 1..close_pos];
                let content_str: String = content_between.iter().collect();
                let trimmed_content = content_str.trim();

                if trimmed_content.is_empty() {
                    // Empty braces
                    let spaces_count = content_str.len();
                    if spaces_count < min_spaces_inside_empty {
                        problems.push(Problem::new(
                            line_number,
                            open_pos + 1,
                            Level::Error,
                            self.id(),
                            format!(
                                "too few spaces inside empty braces, expected at least {min_spaces_inside_empty}"
                            ),
                        ));
                    } else if spaces_count > max_spaces_inside_empty {
                        problems.push(Problem::new(
                            line_number,
                            open_pos + 1,
                            Level::Error,
                            self.id(),
                            format!(
                                "too many spaces inside empty braces, expected at most {max_spaces_inside_empty}"
                            ),
                        ));
                    }
                } else {
                    // Non-empty braces
                    let leading_spaces = content_str.len() - content_str.trim_start().len();
                    let trailing_spaces = content_str.len() - content_str.trim_end().len();

                    if leading_spaces < min_spaces_inside {
                        problems.push(Problem::new(
                            line_number,
                            open_pos + 1,
                            Level::Error,
                            self.id(),
                             format!("too few spaces inside braces, expected at least {min_spaces_inside}"),
                        ));
                    } else if leading_spaces > max_spaces_inside {
                        problems.push(Problem::new(
                            line_number,
                            open_pos + 1,
                            Level::Error,
                            self.id(),
                             format!("too many spaces inside braces, expected at most {max_spaces_inside}"),
                        ));
                    }

                    if trailing_spaces < min_spaces_inside {
                        problems.push(Problem::new(
                            line_number,
                            close_pos + 1,
                            Level::Error,
                            self.id(),
                             format!("too few spaces inside braces, expected at least {min_spaces_inside}"),
                        ));
                    } else if trailing_spaces > max_spaces_inside {
                        problems.push(Problem::new(
                            line_number,
                            close_pos + 1,
                            Level::Error,
                            self.id(),
                             format!("too many spaces inside braces, expected at most {max_spaces_inside}"),
                        ));
                    }
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("min-spaces-inside".to_string(), ConfigValue::Int(0));
        config.set_param("max-spaces-inside".to_string(), ConfigValue::Int(1));
        config.set_param("min-spaces-inside-empty".to_string(), ConfigValue::Int(0));
        config.set_param("max-spaces-inside-empty".to_string(), ConfigValue::Int(0));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

/// Rule that checks colon spacing
#[derive(Debug)]
pub struct ColonsRule;

impl ColonsRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for ColonsRule {
    fn id(&self) -> &'static str {
        "colons"
    }

    fn description(&self) -> &'static str {
        "Controls the use of colons within mappings"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let max_spaces_before = config.get_int("max-spaces-before").unwrap_or(0) as usize;
        let min_spaces_after = config.get_int("min-spaces-after").unwrap_or(1) as usize;
        let max_spaces_after = config.get_int("max-spaces-after").unwrap_or(1) as usize;

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Find colons that are part of key-value pairs (not in strings)
            let mut in_string = false;
            let mut string_char = '\0';
            let chars: Vec<char> = line.chars().collect();

            for (i, &ch) in chars.iter().enumerate() {
                match ch {
                    '"' | '\'' if !in_string => {
                        in_string = true;
                        string_char = ch;
                    }
                    c if in_string && c == string_char => {
                        in_string = false;
                    }
                    ':' if !in_string => {
                        // Check spaces before colon
                        let spaces_before = if i > 0 {
                            let mut count = 0;
                            let mut j = i;
                            while j > 0 && chars[j - 1] == ' ' {
                                count += 1;
                                j -= 1;
                            }
                            count
                        } else {
                            0
                        };

                        if spaces_before > max_spaces_before {
                            problems.push(Problem::new(
                                line_number,
                                i + 1,
                                Level::Error,
                                self.id(),
                                 format!("too many spaces before colon, expected at most {max_spaces_before}"),
                            ));
                        }

                        // Check spaces after colon
                        let spaces_after = if i + 1 < chars.len() {
                            let mut count = 0;
                            let mut j = i + 1;
                            while j < chars.len() && chars[j] == ' ' {
                                count += 1;
                                j += 1;
                            }
                            count
                        } else {
                            0
                        };

                        // Only check if there's content after the colon
                        if i + 1 + spaces_after < chars.len() {
                            if spaces_after < min_spaces_after {
                                problems.push(Problem::new(
                                    line_number,
                                    i + 2,
                                    Level::Error,
                                    self.id(),
                                     format!("too few spaces after colon, expected at least {min_spaces_after}"),
                                ));
                            } else if spaces_after > max_spaces_after {
                                problems.push(Problem::new(
                                    line_number,
                                    i + 2,
                                    Level::Error,
                                    self.id(),
                                     format!("too many spaces after colon, expected at most {max_spaces_after}"),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("max-spaces-before".to_string(), ConfigValue::Int(0));
        config.set_param("min-spaces-after".to_string(), ConfigValue::Int(1));
        config.set_param("max-spaces-after".to_string(), ConfigValue::Int(1));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

/// Rule that checks comma spacing
#[derive(Debug)]
pub struct CommasRule;

impl CommasRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for CommasRule {
    fn id(&self) -> &'static str {
        "commas"
    }

    fn description(&self) -> &'static str {
        "Controls the use of commas in sequences and mappings"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let max_spaces_before = config.get_int("max-spaces-before").unwrap_or(0) as usize;
        let min_spaces_after = config.get_int("min-spaces-after").unwrap_or(1) as usize;
        let max_spaces_after = config.get_int("max-spaces-after").unwrap_or(1) as usize;

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Find commas that are not in strings
            let mut in_string = false;
            let mut string_char = '\0';
            let chars: Vec<char> = line.chars().collect();

            for (i, &ch) in chars.iter().enumerate() {
                match ch {
                    '"' | '\'' if !in_string => {
                        in_string = true;
                        string_char = ch;
                    }
                    c if in_string && c == string_char => {
                        in_string = false;
                    }
                    ',' if !in_string => {
                        // Check spaces before comma
                        let spaces_before = if i > 0 {
                            let mut count = 0;
                            let mut j = i;
                            while j > 0 && chars[j - 1] == ' ' {
                                count += 1;
                                j -= 1;
                            }
                            count
                        } else {
                            0
                        };

                        if spaces_before > max_spaces_before {
                            problems.push(Problem::new(
                                line_number,
                                i + 1,
                                Level::Error,
                                self.id(),
                                 format!("too many spaces before comma, expected at most {max_spaces_before}"),
                            ));
                        }

                        // Check spaces after comma
                        let spaces_after = if i + 1 < chars.len() {
                            let mut count = 0;
                            let mut j = i + 1;
                            while j < chars.len() && chars[j] == ' ' {
                                count += 1;
                                j += 1;
                            }
                            count
                        } else {
                            0
                        };

                        // Only check if there's content after the comma
                        if i + 1 + spaces_after < chars.len() {
                            if spaces_after < min_spaces_after {
                                problems.push(Problem::new(
                                    line_number,
                                    i + 2,
                                    Level::Error,
                                    self.id(),
                                     format!("too few spaces after comma, expected at least {min_spaces_after}"),
                                ));
                            } else if spaces_after > max_spaces_after {
                                problems.push(Problem::new(
                                    line_number,
                                    i + 2,
                                    Level::Error,
                                    self.id(),
                                     format!("too many spaces after comma, expected at most {max_spaces_after}"),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("max-spaces-before".to_string(), ConfigValue::Int(0));
        config.set_param("min-spaces-after".to_string(), ConfigValue::Int(1));
        config.set_param("max-spaces-after".to_string(), ConfigValue::Int(1));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

/// Rule that checks hyphen spacing in sequences
#[derive(Debug)]
pub struct HyphensRule;

impl HyphensRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for HyphensRule {
    fn id(&self) -> &'static str {
        "hyphens"
    }

    fn description(&self) -> &'static str {
        "Controls the use of hyphens in sequences"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let max_spaces_after = config.get_int("max-spaces-after").unwrap_or(1) as usize;

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;
            let trimmed = line.trim_start();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check if this is a sequence item (starts with hyphen)
            if trimmed.starts_with('-') {
                let hyphen_pos = line.find('-').unwrap();
                let chars: Vec<char> = line.chars().collect();

                // Check spaces after hyphen
                let spaces_after = if hyphen_pos + 1 < chars.len() {
                    let mut count = 0;
                    let mut j = hyphen_pos + 1;
                    while j < chars.len() && chars[j] == ' ' {
                        count += 1;
                        j += 1;
                    }
                    count
                } else {
                    0
                };

                // Only check if there's content after the hyphen
                if hyphen_pos + 1 + spaces_after < chars.len() {
                    if spaces_after == 0 {
                        problems.push(Problem::new(
                            line_number,
                            hyphen_pos + 2,
                            Level::Error,
                            self.id(),
                            "missing space after hyphen".to_string(),
                        ));
                    } else if spaces_after > max_spaces_after {
                        problems.push(Problem::new(
                            line_number,
                            hyphen_pos + 2,
                            Level::Error,
                            self.id(),
                             format!("too many spaces after hyphen, expected at most {max_spaces_after}"),
                        ));
                    }
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("max-spaces-after".to_string(), ConfigValue::Int(1));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
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
    fn test_brackets_rule_correct_spacing() {
        let rule = BracketsRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("array: [ item1, item2 ]", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_brackets_rule_no_spacing() {
        let rule = BracketsRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("array: [item1, item2]", &path);
        let mut config = rule.default_config();
        config.enabled = true;
        config.set_param("min-spaces-inside".to_string(), ConfigValue::Int(1)); // Require at least 1 space

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 2); // Missing space after [ and before ]
    }

    #[test]
    fn test_braces_rule_correct_spacing() {
        let rule = BracesRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("mapping: { key: value }", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_colons_rule_correct_spacing() {
        let rule = ColonsRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("key: value", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_colons_rule_no_space_after() {
        let rule = ColonsRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("key:value", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too few spaces after colon"));
    }

    #[test]
    fn test_colons_rule_space_before() {
        let rule = ColonsRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("key : value", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too many spaces before colon"));
    }

    #[test]
    fn test_commas_rule_correct_spacing() {
        let rule = CommasRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("array: [item1, item2, item3]", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_commas_rule_no_space_after() {
        let rule = CommasRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("array: [item1,item2]", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("too few spaces after comma"));
    }

    #[test]
    fn test_hyphens_rule_correct_spacing() {
        let rule = HyphensRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("- item1\n- item2", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_hyphens_rule_no_space_after() {
        let rule = HyphensRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("-item1\n-item2", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 2);
        assert!(problems[0].message.contains("missing space after hyphen"));
    }
}
