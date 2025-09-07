/// Common utilities for implementing rules

/// Check if a line is effectively empty (whitespace only)
pub fn is_empty_line(line: &str) -> bool {
    line.trim().is_empty()
}

/// Count the leading whitespace characters in a line
pub fn count_leading_whitespace(line: &str) -> usize {
    line.chars().take_while(|c| c.is_whitespace() && *c != '\n').count()
}

/// Check if a line contains only whitespace and a comment
pub fn is_comment_only_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#') || trimmed.is_empty()
}

/// Extract the comment portion from a line, if any
pub fn extract_comment(line: &str) -> Option<&str> {
    line.find('#').map(|pos| &line[pos..])
}

/// Check if a line has trailing whitespace
pub fn has_trailing_whitespace(line: &str) -> bool {
    !line.is_empty() && line.ends_with(|c: char| c.is_whitespace())
}

/// Get the position of the first trailing whitespace character
pub fn trailing_whitespace_start(line: &str) -> Option<usize> {
    if !has_trailing_whitespace(line) {
        return None;
    }

    let mut pos = line.len();
    for ch in line.chars().rev() {
        if !ch.is_whitespace() {
            break;
        }
        pos -= ch.len_utf8();
    }
    
    if pos < line.len() {
        Some(pos)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_empty_line() {
        assert!(is_empty_line(""));
        assert!(is_empty_line("   "));
        assert!(is_empty_line("\t\t"));
        assert!(is_empty_line(" \t \n"));
        assert!(!is_empty_line("content"));
        assert!(!is_empty_line("  content  "));
    }

    #[test]
    fn test_count_leading_whitespace() {
        assert_eq!(count_leading_whitespace(""), 0);
        assert_eq!(count_leading_whitespace("no_spaces"), 0);
        assert_eq!(count_leading_whitespace("  two_spaces"), 2);
        assert_eq!(count_leading_whitespace("\t\ttwo_tabs"), 2);
        assert_eq!(count_leading_whitespace("  \tmixed"), 3);
    }

    #[test]
    fn test_is_comment_only_line() {
        assert!(is_comment_only_line(""));
        assert!(is_comment_only_line("   "));
        assert!(is_comment_only_line("# comment"));
        assert!(is_comment_only_line("  # indented comment"));
        assert!(!is_comment_only_line("key: value # comment"));
        assert!(!is_comment_only_line("key: value"));
    }

    #[test]
    fn test_extract_comment() {
        assert_eq!(extract_comment("key: value # comment"), Some("# comment"));
        assert_eq!(extract_comment("# full comment"), Some("# full comment"));
        assert_eq!(extract_comment("key: value"), None);
        assert_eq!(extract_comment(""), None);
        assert_eq!(extract_comment("key: # empty comment"), Some("# empty comment"));
    }

    #[test]
    fn test_has_trailing_whitespace() {
        assert!(!has_trailing_whitespace(""));
        assert!(!has_trailing_whitespace("no_trailing"));
        assert!(has_trailing_whitespace("has_trailing "));
        assert!(has_trailing_whitespace("has_trailing\t"));
        assert!(has_trailing_whitespace("multiple   "));
        assert!(!has_trailing_whitespace("  leading_only"));
    }

    #[test]
    fn test_trailing_whitespace_start() {
        assert_eq!(trailing_whitespace_start(""), None);
        assert_eq!(trailing_whitespace_start("no_trailing"), None);
        assert_eq!(trailing_whitespace_start("trailing "), Some(8));
        assert_eq!(trailing_whitespace_start("trailing\t"), Some(8));
        assert_eq!(trailing_whitespace_start("multiple   "), Some(8));
        assert_eq!(trailing_whitespace_start("  leading_only"), None);
    }
}
