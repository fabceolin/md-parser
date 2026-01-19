//! Variable extraction from Markdown content

use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

/// Regex for matching template variables: `{{variable_name}}`
static VARIABLE_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{(\w+)\}\}").expect("Invalid variable regex"));

/// Extract all variable names from content
///
/// Finds all `{{variable_name}}` patterns and returns the variable names.
/// Results may contain duplicates - use `extract_unique_variables` for deduped results.
///
/// # Example
///
/// ```
/// use md_parser::extract_variables;
///
/// let content = "Hello {{name}}, your order {{order_id}} is ready. Thank you, {{name}}!";
/// let vars = extract_variables(content);
/// assert_eq!(vars, vec!["name", "order_id", "name"]);
/// ```
pub fn extract_variables(content: &str) -> Vec<String> {
    VARIABLE_REGEX
        .captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Extract unique variable names from content
///
/// Finds all `{{variable_name}}` patterns and returns deduplicated, sorted variable names.
///
/// # Example
///
/// ```
/// use md_parser::extract_unique_variables;
///
/// let content = "Hello {{name}}, your order {{order_id}} is ready. Thank you, {{name}}!";
/// let vars = extract_unique_variables(content);
/// assert_eq!(vars, vec!["name", "order_id"]);
/// ```
pub fn extract_unique_variables(content: &str) -> Vec<String> {
    let mut vars: Vec<String> = VARIABLE_REGEX
        .captures_iter(content)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    vars.sort();
    vars
}

/// Check if content contains any template variables
///
/// # Example
///
/// ```
/// use md_parser::has_variables;
///
/// assert!(has_variables("Hello {{name}}!"));
/// assert!(!has_variables("Hello world!"));
/// ```
pub fn has_variables(content: &str) -> bool {
    VARIABLE_REGEX.is_match(content)
}

/// Count the number of variable occurrences in content
///
/// # Example
///
/// ```
/// use md_parser::count_variables;
///
/// let content = "{{a}} {{b}} {{a}}";
/// assert_eq!(count_variables(content), 3);
/// ```
pub fn count_variables(content: &str) -> usize {
    VARIABLE_REGEX.find_iter(content).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_single_variable() {
        let content = "Hello {{name}}!";
        let vars = extract_variables(content);
        assert_eq!(vars, vec!["name"]);
    }

    #[test]
    fn test_extract_multiple_variables() {
        let content = "Hello {{first_name}} {{last_name}}!";
        let vars = extract_variables(content);
        assert_eq!(vars, vec!["first_name", "last_name"]);
    }

    #[test]
    fn test_extract_duplicate_variables() {
        let content = "{{a}} {{b}} {{a}} {{c}} {{a}}";
        let vars = extract_variables(content);
        assert_eq!(vars, vec!["a", "b", "a", "c", "a"]);
    }

    #[test]
    fn test_extract_unique_variables() {
        let content = "{{b}} {{a}} {{b}} {{c}} {{a}}";
        let vars = extract_unique_variables(content);
        assert_eq!(vars, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_no_variables() {
        let content = "Hello world!";
        let vars = extract_variables(content);
        assert!(vars.is_empty());
    }

    #[test]
    fn test_has_variables() {
        assert!(has_variables("Hello {{name}}!"));
        assert!(!has_variables("Hello world!"));
        assert!(has_variables("{{a}} and {{b}}"));
    }

    #[test]
    fn test_count_variables() {
        assert_eq!(count_variables("Hello {{name}}!"), 1);
        assert_eq!(count_variables("{{a}} {{b}} {{c}}"), 3);
        assert_eq!(count_variables("No vars here"), 0);
        assert_eq!(count_variables("{{x}} and {{x}}"), 2);
    }

    #[test]
    fn test_variable_with_underscores() {
        let content = "Order {{order_id}} for {{user_name}}";
        let vars = extract_variables(content);
        assert_eq!(vars, vec!["order_id", "user_name"]);
    }

    #[test]
    fn test_variable_with_numbers() {
        let content = "Value: {{var1}} {{var2}}";
        let vars = extract_variables(content);
        assert_eq!(vars, vec!["var1", "var2"]);
    }

    #[test]
    fn test_invalid_variable_patterns() {
        // These should NOT match
        assert!(extract_variables("{ {name} }").is_empty());
        assert!(extract_variables("{name}").is_empty());
        assert!(extract_variables("{{}}").is_empty());
        assert!(extract_variables("{{ name }}").is_empty()); // spaces not allowed
    }

    #[test]
    fn test_variables_in_code_block() {
        let content = "```\nconst x = {{value}};\n```";
        let vars = extract_variables(content);
        assert_eq!(vars, vec!["value"]);
    }

    #[test]
    fn test_multiline_content() {
        let content = "Line 1: {{a}}\nLine 2: {{b}}\nLine 3: {{c}}";
        let vars = extract_variables(content);
        assert_eq!(vars, vec!["a", "b", "c"]);
    }
}
