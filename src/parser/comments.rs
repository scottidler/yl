use eyre::Result;
use regex::Regex;
use std::collections::HashMap;

/// Scope of a directive's effect
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Some variants are for future phases
pub enum Scope {
    /// Current line only
    Line,
    /// Until next directive or end of current block
    Block,
    /// Until end of current YAML section
    Section,
    /// Rest of file
    File,
}

/// A parsed comment directive
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    /// Disable rules with specified scope
    Disable { rules: Vec<String>, scope: Scope },
    /// Disable rules for current line only
    DisableLine { rules: Vec<String> },
    /// Set rule parameters
    Set { rule: String, params: HashMap<String, String> },
    /// Configure rule with parameters
    Config { rule: String, params: HashMap<String, String> },
    /// Ignore entire file
    IgnoreFile,
    /// Ignore rules for current YAML section
    IgnoreSection { rules: Vec<String> },
    /// Enable previously disabled rules
    Enable { rules: Vec<String>, scope: Scope },
}

/// Processes comments to extract linting directives
pub struct CommentProcessor {
    directive_regex: Regex,
    param_regex: Regex,
}

impl CommentProcessor {
    /// Create a new comment processor
    pub fn new() -> Self {
        let directive_regex = Regex::new(
            r"#\s*yl:(disable-line|ignore-file|ignore-section|disable|enable|config|set)(?:\s+(.+))?"
        ).expect("Invalid directive regex");

        let param_regex = Regex::new(
            r"([a-zA-Z0-9_-]+)\.([a-zA-Z0-9_-]+)=([^\s,]+)"
        ).expect("Invalid parameter regex");

        Self { directive_regex, param_regex }
    }

    /// Parse a comment line for directives
    pub fn parse_directive(&self, comment: &str) -> Result<Option<Directive>> {
        let comment = comment.trim();

        // Check if this is a yl directive
        if let Some(captures) = self.directive_regex.captures(comment) {
            let directive_type = captures.get(1).unwrap().as_str();
            let args = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");


            match directive_type {
                "disable" => self.parse_disable(args, Scope::Block),
                "disable-line" => self.parse_disable(args, Scope::Line),
                "enable" => self.parse_enable(args, Scope::Block),
                "set" => self.parse_set(args),
                "config" => self.parse_config(args),
                "ignore-file" => Ok(Some(Directive::IgnoreFile)),
                "ignore-section" => self.parse_ignore_section(args),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Parse disable directive
    fn parse_disable(&self, args: &str, scope: Scope) -> Result<Option<Directive>> {
        let rules = if args.is_empty() {
            vec![] // Empty means all rules
        } else {
            self.parse_rule_list(args)
        };

        Ok(Some(match scope {
            Scope::Line => Directive::DisableLine { rules },
            _ => Directive::Disable { rules, scope },
        }))
    }

    /// Parse enable directive
    fn parse_enable(&self, args: &str, scope: Scope) -> Result<Option<Directive>> {
        let rules = if args.is_empty() {
            vec![] // Enable all rules
        } else {
            self.parse_rule_list(args)
        };

        Ok(Some(Directive::Enable { rules, scope }))
    }

    /// Parse set directive (rule.param=value)
    fn parse_set(&self, args: &str) -> Result<Option<Directive>> {
        if let Some(captures) = self.param_regex.captures(args) {
            let rule = captures.get(1).unwrap().as_str().to_string();
            let param = captures.get(2).unwrap().as_str().to_string();
            let value = captures.get(3).unwrap().as_str().to_string();

            let mut params = HashMap::new();
            params.insert(param, value);

            Ok(Some(Directive::Set { rule, params }))
        } else {
            Err(eyre::eyre!("Invalid set directive format. Expected: rule.param=value"))
        }
    }

    /// Parse config directive (rule param1=value1,param2=value2)
    fn parse_config(&self, args: &str) -> Result<Option<Directive>> {
        let parts: Vec<&str> = args.splitn(2, ' ').collect();
        if parts.is_empty() {
            return Err(eyre::eyre!("Config directive requires rule name"));
        }

        let rule = parts[0].to_string();
        let mut params = HashMap::new();

        if parts.len() > 1 {
            // Parse parameters
            for param_str in parts[1].split(',') {
                let param_str = param_str.trim();
                if let Some(eq_pos) = param_str.find('=') {
                    let key = param_str[..eq_pos].trim().to_string();
                    let value = param_str[eq_pos + 1..].trim().to_string();
                    params.insert(key, value);
                }
            }
        }

        Ok(Some(Directive::Config { rule, params }))
    }

    /// Parse ignore-section directive
    fn parse_ignore_section(&self, args: &str) -> Result<Option<Directive>> {
        let rules = if args.is_empty() {
            vec![] // Ignore all rules for section
        } else {
            self.parse_rule_list(args)
        };

        Ok(Some(Directive::IgnoreSection { rules }))
    }

    /// Parse a comma-separated list of rule names
    fn parse_rule_list(&self, args: &str) -> Vec<String> {
        args.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

impl Default for CommentProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn processor() -> CommentProcessor {
        CommentProcessor::new()
    }

    #[test]
    fn test_parse_disable_all() {
        let processor = processor();
        let directive = processor.parse_directive("# yl:disable").unwrap().unwrap();

        match directive {
            Directive::Disable { rules, scope } => {
                assert!(rules.is_empty());
                assert_eq!(scope, Scope::Block);
            }
            _ => panic!("Expected Disable directive"),
        }
    }

    #[test]
    fn test_parse_disable_specific_rules() {
        let processor = processor();
        let directive = processor.parse_directive("# yl:disable line-length,trailing-spaces").unwrap().unwrap();

        match directive {
            Directive::Disable { rules, scope } => {
                assert_eq!(rules, vec!["line-length", "trailing-spaces"]);
                assert_eq!(scope, Scope::Block);
            }
            _ => panic!("Expected Disable directive"),
        }
    }

    #[test]
    fn test_parse_disable_line() {
        let processor = processor();
        let directive = processor.parse_directive("# yl:disable-line line-length").unwrap().unwrap();

        match directive {
            Directive::DisableLine { rules } => {
                assert_eq!(rules, vec!["line-length"]);
            }
            _ => panic!("Expected DisableLine directive, got: {:?}", directive),
        }
    }

    #[test]
    fn test_parse_enable() {
        let processor = processor();
        let directive = processor.parse_directive("# yl:enable line-length").unwrap().unwrap();

        match directive {
            Directive::Enable { rules, scope } => {
                assert_eq!(rules, vec!["line-length"]);
                assert_eq!(scope, Scope::Block);
            }
            _ => panic!("Expected Enable directive"),
        }
    }

    #[test]
    fn test_parse_set() {
        let processor = processor();
        let directive = processor.parse_directive("# yl:set line-length.max=120").unwrap().unwrap();

        match directive {
            Directive::Set { rule, params } => {
                assert_eq!(rule, "line-length");
                assert_eq!(params.get("max"), Some(&"120".to_string()));
            }
            _ => panic!("Expected Set directive"),
        }
    }

    #[test]
    fn test_parse_config() {
        let processor = processor();
        let directive = processor.parse_directive("# yl:config line-length max=120,allow-non-breakable-words=false").unwrap().unwrap();

        match directive {
            Directive::Config { rule, params } => {
                assert_eq!(rule, "line-length");
                assert_eq!(params.get("max"), Some(&"120".to_string()));
                assert_eq!(params.get("allow-non-breakable-words"), Some(&"false".to_string()));
            }
            _ => panic!("Expected Config directive"),
        }
    }

    #[test]
    fn test_parse_ignore_file() {
        let processor = processor();
        let directive = processor.parse_directive("# yl:ignore-file").unwrap().unwrap();

        match directive {
            Directive::IgnoreFile => {}
            _ => panic!("Expected IgnoreFile directive"),
        }
    }

    #[test]
    fn test_parse_ignore_section() {
        let processor = processor();
        let directive = processor.parse_directive("# yl:ignore-section line-length").unwrap().unwrap();

        match directive {
            Directive::IgnoreSection { rules } => {
                assert_eq!(rules, vec!["line-length"]);
            }
            _ => panic!("Expected IgnoreSection directive"),
        }
    }

    #[test]
    fn test_parse_non_directive_comment() {
        let processor = processor();
        let result = processor.parse_directive("# This is just a regular comment").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_invalid_set_format() {
        let processor = processor();
        let result = processor.parse_directive("# yl:set invalid-format");
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_handling() {
        let processor = processor();

        // Test various whitespace scenarios
        let directive1 = processor.parse_directive("  #   yl:disable   line-length  ").unwrap().unwrap();
        let directive2 = processor.parse_directive("#yl:disable line-length").unwrap().unwrap();

        // Both should parse the same way
        match (directive1, directive2) {
            (Directive::Disable { rules: r1, .. }, Directive::Disable { rules: r2, .. }) => {
                assert_eq!(r1, r2);
                assert_eq!(r1, vec!["line-length"]);
            }
            _ => panic!("Expected Disable directives"),
        }
    }

    #[test]
    fn test_rule_list_parsing() {
        let processor = processor();

        // Test comma-separated rules with various spacing
        let directive = processor.parse_directive("# yl:disable rule1, rule2 ,rule3,  rule4  ").unwrap().unwrap();

        match directive {
            Directive::Disable { rules, .. } => {
                assert_eq!(rules, vec!["rule1", "rule2", "rule3", "rule4"]);
            }
            _ => panic!("Expected Disable directive"),
        }
    }
}
