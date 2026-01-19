//! YAML frontmatter parsing (feature-gated)
//!
//! This module provides functionality to parse YAML frontmatter from Markdown files.
//! Frontmatter is a section at the beginning of the file delimited by `---` markers.
//!
//! # Example
//!
//! ```yaml
//! ---
//! title: My Document
//! author: John Doe
//! tags:
//!   - rust
//!   - markdown
//! ---
//!
//! # Document Content
//! ```

use std::collections::HashMap;

use crate::error::ParseError;

/// Strip frontmatter from content and parse it as YAML
///
/// Returns a tuple of (remaining content, parsed frontmatter).
/// If no frontmatter is present, returns the original content with None.
///
/// # Errors
///
/// Returns `ParseError::FrontmatterError` if the YAML is malformed.
pub fn strip_frontmatter(
    content: &str,
) -> Result<(String, Option<HashMap<String, serde_yaml::Value>>), ParseError> {
    let trimmed = content.trim_start();

    // Check if content starts with frontmatter delimiter
    if !trimmed.starts_with("---") {
        return Ok((content.to_string(), None));
    }

    // Find the end of frontmatter - everything after the first "---"
    let after_first_delimiter = &trimmed[3..];

    // Find the closing delimiter (\n---) in what follows the opening ---
    if let Some(end_idx) = after_first_delimiter.find("\n---") {
        // yaml_content is between opening --- and \n---
        // It includes the leading newline from after_first_delimiter
        let yaml_content = &after_first_delimiter[..end_idx];
        // Remove leading newline if present
        let yaml_content = yaml_content.strip_prefix('\n').unwrap_or(yaml_content);

        let remaining_content = &after_first_delimiter[end_idx + 4..]; // Skip \n---

        // Skip any trailing newlines after the closing delimiter
        let remaining = remaining_content.trim_start_matches('\n');

        // Parse YAML (empty string parses to empty HashMap)
        let frontmatter: HashMap<String, serde_yaml::Value> = serde_yaml::from_str(yaml_content)
            .map_err(|e| ParseError::FrontmatterError(format!("Invalid YAML: {}", e)))?;

        Ok((remaining.to_string(), Some(frontmatter)))
    } else {
        // No closing delimiter found, treat as regular content
        Ok((content.to_string(), None))
    }
}

/// Parse frontmatter from content without stripping it
///
/// Returns the parsed frontmatter if present, or None.
///
/// # Errors
///
/// Returns `ParseError::FrontmatterError` if the YAML is malformed.
pub fn parse_frontmatter(
    content: &str,
) -> Result<Option<HashMap<String, serde_yaml::Value>>, ParseError> {
    let (_, frontmatter) = strip_frontmatter(content)?;
    Ok(frontmatter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value;

    #[test]
    fn test_strip_simple_frontmatter() {
        let content = "---\ntitle: Test\n---\n\n# Content";
        let (remaining, frontmatter) = strip_frontmatter(content).unwrap();

        assert!(frontmatter.is_some());
        let fm = frontmatter.unwrap();
        assert_eq!(fm.get("title"), Some(&Value::String("Test".to_string())));
        assert!(remaining.contains("# Content"));
    }

    #[test]
    fn test_strip_complex_frontmatter() {
        let content = r#"---
title: My Document
author: John Doe
tags:
  - rust
  - markdown
count: 42
---

# Content here"#;

        let (remaining, frontmatter) = strip_frontmatter(content).unwrap();

        assert!(frontmatter.is_some());
        let fm = frontmatter.unwrap();
        assert_eq!(
            fm.get("title"),
            Some(&Value::String("My Document".to_string()))
        );
        assert_eq!(
            fm.get("author"),
            Some(&Value::String("John Doe".to_string()))
        );
        assert_eq!(fm.get("count"), Some(&Value::Number(42.into())));

        // Check tags is a sequence
        if let Some(Value::Sequence(tags)) = fm.get("tags") {
            assert_eq!(tags.len(), 2);
        } else {
            panic!("tags should be a sequence");
        }

        assert!(remaining.contains("# Content here"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a heading\n\nSome content";
        let (remaining, frontmatter) = strip_frontmatter(content).unwrap();

        assert!(frontmatter.is_none());
        assert_eq!(remaining, content);
    }

    #[test]
    fn test_incomplete_frontmatter() {
        let content = "---\ntitle: Test\n# No closing delimiter";
        let (remaining, frontmatter) = strip_frontmatter(content).unwrap();

        // Without closing delimiter, treat as regular content
        assert!(frontmatter.is_none());
        assert_eq!(remaining, content);
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "---\n---\n\n# Content";
        let (remaining, frontmatter) = strip_frontmatter(content).unwrap();

        assert!(frontmatter.is_some());
        assert!(frontmatter.unwrap().is_empty());
        assert!(remaining.contains("# Content"));
    }

    #[test]
    fn test_frontmatter_with_leading_whitespace() {
        let content = "\n\n---\ntitle: Test\n---\n\n# Content";
        let (_remaining, frontmatter) = strip_frontmatter(content).unwrap();

        assert!(frontmatter.is_some());
        let fm = frontmatter.unwrap();
        assert_eq!(fm.get("title"), Some(&Value::String("Test".to_string())));
    }

    #[test]
    fn test_parse_frontmatter_only() {
        let content = "---\ntitle: Test\nauthor: Me\n---\n\n# Content";
        let frontmatter = parse_frontmatter(content).unwrap();

        assert!(frontmatter.is_some());
        let fm = frontmatter.unwrap();
        assert_eq!(fm.len(), 2);
    }

    #[test]
    fn test_invalid_yaml() {
        let _content = "---\n  invalid:\n    - unclosed\n---\n";
        // This YAML is actually valid, let's use truly invalid YAML
        let invalid_content = "---\n:\ninvalid yaml\n---\n";
        let result = strip_frontmatter(invalid_content);

        assert!(result.is_err());
        if let Err(ParseError::FrontmatterError(msg)) = result {
            assert!(msg.contains("Invalid YAML"));
        }
    }

    #[test]
    fn test_frontmatter_with_triple_dash_in_content() {
        let content = "---\ntitle: Test\n---\n\n# Content\n\nSome text with --- in it";
        let (remaining, frontmatter) = strip_frontmatter(content).unwrap();

        assert!(frontmatter.is_some());
        assert!(remaining.contains("Some text with --- in it"));
    }
}
