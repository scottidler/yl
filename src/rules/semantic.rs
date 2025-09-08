use super::{ConfigValue, Rule, RuleConfig};
use crate::linter::{Level, LintContext, Problem};
use eyre::Result;

/// Rule that enforces consistent boolean value representation
#[derive(Debug)]
pub struct TruthyRule;

impl TruthyRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for TruthyRule {
    fn id(&self) -> &'static str {
        "truthy"
    }

    fn description(&self) -> &'static str {
        "Enforces consistent boolean value representation"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let allowed_values = config
            .get_string("allowed-values")
            .unwrap_or("true,false")
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<String>>();

        let check_keys = config.get_bool("check-keys").unwrap_or(true);

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Look for key-value pairs
            if let Some(colon_pos) = line.find(':') {
                let key_part = line[..colon_pos].trim();
                let value_part = line[colon_pos + 1..].trim();

                // Check key for truthy values if enabled
                if check_keys {
                    self.check_truthy_value(key_part, line_number, &allowed_values, &mut problems);
                }

                // Check value for truthy values
                self.check_truthy_value(value_part, line_number, &allowed_values, &mut problems);
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param(
            "allowed-values".to_string(),
            ConfigValue::String("true,false".to_string()),
        );
        config.set_param("check-keys".to_string(), ConfigValue::Bool(true));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

impl TruthyRule {
    fn check_truthy_value(
        &self,
        value: &str,
        line_number: usize,
        allowed_values: &[String],
        problems: &mut Vec<Problem>,
    ) {
        let truthy_variants = [
            "yes", "Yes", "YES", "no", "No", "NO", "on", "On", "ON", "off", "Off", "OFF", "True", "TRUE", "False",
            "FALSE",
        ];

        for variant in &truthy_variants {
            if value == *variant && !allowed_values.contains(&variant.to_string()) {
                problems.push(Problem::new(
                    line_number,
                    1,
                    Level::Error,
                    self.id(),
                    format!(
                        "truthy value should be one of [{}], not \"{}\"",
                        allowed_values.join(", "),
                        variant
                    ),
                ));
            }
        }
    }
}

/// Rule that enforces consistent string quoting
#[derive(Debug)]
pub struct QuotedStringsRule;

impl QuotedStringsRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for QuotedStringsRule {
    fn id(&self) -> &'static str {
        "quoted-strings"
    }

    fn description(&self) -> &'static str {
        "Enforces consistent string quoting"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let quote_type = config.get_string("quote-type").unwrap_or("any");
        let required_only_when_needed = config.get_bool("required-only-when-needed").unwrap_or(false);

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Look for quoted strings
            self.check_quoted_strings_in_line(line, line_number, quote_type, required_only_when_needed, &mut problems);
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("quote-type".to_string(), ConfigValue::String("any".to_string()));
        config.set_param("required-only-when-needed".to_string(), ConfigValue::Bool(false));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

impl QuotedStringsRule {
    fn check_quoted_strings_in_line(
        &self,
        line: &str,
        line_number: usize,
        quote_type: &str,
        required_only_when_needed: bool,
        problems: &mut Vec<Problem>,
    ) {
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            if chars[i] == '"' || chars[i] == '\'' {
                let quote_char = chars[i];
                let start_pos = i;
                i += 1;

                // Find the end of the string
                while i < chars.len() && chars[i] != quote_char {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 2; // Skip escaped character
                    } else {
                        i += 1;
                    }
                }

                if i < chars.len() {
                    // Found complete quoted string
                    let string_content: String = chars[start_pos + 1..i].iter().collect();

                    match quote_type {
                        "single" if quote_char == '"' => {
                            problems.push(Problem::new(
                                line_number,
                                start_pos + 1,
                                Level::Error,
                                self.id(),
                                "string should be single-quoted".to_string(),
                            ));
                        }
                        "double" if quote_char == '\'' => {
                            problems.push(Problem::new(
                                line_number,
                                start_pos + 1,
                                Level::Error,
                                self.id(),
                                "string should be double-quoted".to_string(),
                            ));
                        }
                        _ => {}
                    }

                    if required_only_when_needed && !self.needs_quoting(&string_content) {
                        problems.push(Problem::new(
                            line_number,
                            start_pos + 1,
                            Level::Error,
                            self.id(),
                            "string should not be quoted".to_string(),
                        ));
                    }
                }
                i += 1;
            } else {
                i += 1;
            }
        }
    }

    fn needs_quoting(&self, content: &str) -> bool {
        // Check if string needs quoting (contains special characters, etc.)
        content.contains(':')
            || content.contains('#')
            || content.contains('[')
            || content.contains(']')
            || content.contains('{')
            || content.contains('}')
            || content.starts_with(' ')
            || content.ends_with(' ')
            || content.parse::<f64>().is_ok()
            || content.parse::<bool>().is_ok()
    }
}

/// Rule that enforces alphabetical key ordering
#[derive(Debug)]
pub struct KeyOrderingRule;

impl KeyOrderingRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for KeyOrderingRule {
    fn id(&self) -> &'static str {
        "key-ordering"
    }

    fn description(&self) -> &'static str {
        "Enforces alphabetical ordering of keys in mappings"
    }

    fn check(&self, context: &LintContext, _config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        if let Some(yaml_value) = context.yaml() {
            self.check_ordering_recursive(yaml_value, &mut Vec::new(), &mut problems);
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        RuleConfig::new(false, Level::Error) // Disabled by default
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

impl KeyOrderingRule {
    fn check_ordering_recursive(&self, value: &serde_yaml::Value, path: &mut Vec<String>, problems: &mut Vec<Problem>) {
        match value {
            serde_yaml::Value::Mapping(map) => {
                let keys: Vec<String> = map.keys().filter_map(|k| k.as_str().map(|s| s.to_string())).collect();

                let mut sorted_keys = keys.clone();
                sorted_keys.sort();

                if keys != sorted_keys {
                    problems.push(Problem::new(
                        1, // TODO: Get actual line number from YAML structure
                        1,
                        Level::Error,
                        self.id(),
                        format!(
                            "wrong ordering of key \"{}\" in mapping",
                            keys.first().unwrap_or(&"unknown".to_string())
                        ),
                    ));
                }

                // Recursively check nested structures
                for (key, nested_value) in map {
                    if let Some(key_str) = key.as_str() {
                        path.push(key_str.to_string());
                        self.check_ordering_recursive(nested_value, path, problems);
                        path.pop();
                    }
                }
            }
            serde_yaml::Value::Sequence(seq) => {
                for (index, item) in seq.iter().enumerate() {
                    path.push(index.to_string());
                    self.check_ordering_recursive(item, path, problems);
                    path.pop();
                }
            }
            _ => {}
        }
    }
}

/// Rule that validates float value formats
#[derive(Debug)]
pub struct FloatValuesRule;

impl FloatValuesRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for FloatValuesRule {
    fn id(&self) -> &'static str {
        "float-values"
    }

    fn description(&self) -> &'static str {
        "Validates float value formats"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let forbid_scientific_notation = config.get_bool("forbid-scientific-notation").unwrap_or(false);
        let require_numeral_before_decimal = config.get_bool("require-numeral-before-decimal").unwrap_or(false);

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;

            // Look for float values
            if let Some(colon_pos) = line.find(':') {
                let value_part = line[colon_pos + 1..].trim();

                if value_part.parse::<f64>().is_ok() {
                    if forbid_scientific_notation && (value_part.contains('e') || value_part.contains('E')) {
                        problems.push(Problem::new(
                            line_number,
                            colon_pos + 2,
                            Level::Error,
                            self.id(),
                            "scientific notation is forbidden".to_string(),
                        ));
                    }

                    if require_numeral_before_decimal && value_part.starts_with('.') {
                        problems.push(Problem::new(
                            line_number,
                            colon_pos + 2,
                            Level::Error,
                            self.id(),
                            "decimal number should have at least one numeral before decimal point".to_string(),
                        ));
                    }
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("forbid-scientific-notation".to_string(), ConfigValue::Bool(false));
        config.set_param("require-numeral-before-decimal".to_string(), ConfigValue::Bool(false));
        config
    }

    fn validate_config(&self, _config: &RuleConfig) -> Result<()> {
        Ok(())
    }
}

/// Rule that detects octal values
#[derive(Debug)]
pub struct OctalValuesRule;

impl OctalValuesRule {
    pub fn new() -> Self {
        Self
    }
}

impl Rule for OctalValuesRule {
    fn id(&self) -> &'static str {
        "octal-values"
    }

    fn description(&self) -> &'static str {
        "Detects and forbids octal values"
    }

    fn check(&self, context: &LintContext, config: &RuleConfig) -> Result<Vec<Problem>> {
        let mut problems = Vec::new();

        let forbid_implicit_octal = config.get_bool("forbid-implicit-octal").unwrap_or(true);
        let forbid_explicit_octal = config.get_bool("forbid-explicit-octal").unwrap_or(false);

        for (line_no, line) in context.content.lines().enumerate() {
            let line_number = line_no + 1;

            // Look for potential octal values
            if let Some(colon_pos) = line.find(':') {
                let value_part = line[colon_pos + 1..].trim();

                // Check for implicit octal (starts with 0 followed by digits)
                if forbid_implicit_octal
                    && value_part.len() > 1
                    && value_part.starts_with('0')
                    && value_part.chars().nth(1).unwrap().is_ascii_digit()
                {
                    // Make sure it's not a decimal number
                    if !value_part.contains('.') && value_part.parse::<i64>().is_ok() {
                        problems.push(Problem::new(
                            line_number,
                            colon_pos + 2,
                            Level::Error,
                            self.id(),
                             format!("found implicit octal value \"{value_part}\""),
                        ));
                    }
                }

                // Check for explicit octal (0o prefix)
                if forbid_explicit_octal && value_part.starts_with("0o") {
                    problems.push(Problem::new(
                        line_number,
                        colon_pos + 2,
                        Level::Error,
                        self.id(),
                         format!("found explicit octal value \"{value_part}\""),
                    ));
                }
            }
        }

        Ok(problems)
    }

    fn default_config(&self) -> RuleConfig {
        let mut config = RuleConfig::new(false, Level::Error); // Disabled by default
        config.set_param("forbid-implicit-octal".to_string(), ConfigValue::Bool(true));
        config.set_param("forbid-explicit-octal".to_string(), ConfigValue::Bool(false));
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
    fn test_truthy_rule_valid_values() {
        let rule = TruthyRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("enabled: true\ndisabled: false", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert!(problems.is_empty());
    }

    #[test]
    fn test_truthy_rule_invalid_values() {
        let rule = TruthyRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("enabled: yes\ndisabled: no", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 2);
        assert!(problems[0].message.contains("truthy value should be"));
    }

    #[test]
    fn test_octal_values_rule_implicit_octal() {
        let rule = OctalValuesRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("mode: 0755", &path);
        let mut config = rule.default_config();
        config.enabled = true;

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("implicit octal"));
    }

    #[test]
    fn test_float_values_rule_scientific_notation() {
        let rule = FloatValuesRule::new();
        let path = PathBuf::from("test.yaml");
        let context = create_test_context("value: 1.23e-4", &path);
        let mut config = rule.default_config();
        config.enabled = true;
        config.set_param("forbid-scientific-notation".to_string(), ConfigValue::Bool(true));

        let problems = rule.check(&context, &config).unwrap();
        assert_eq!(problems.len(), 1);
        assert!(problems[0].message.contains("scientific notation is forbidden"));
    }
}
