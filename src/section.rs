//! Section types for parsed Markdown documents

use uuid::Uuid;

/// Type of Markdown section
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SectionType {
    /// Heading (H1-H6)
    Heading,
    /// Regular paragraph
    Paragraph,
    /// List (ordered or unordered)
    List,
    /// Code block
    Code,
    /// Table
    Table,
    /// Blockquote
    Blockquote,
    /// Horizontal rule
    HorizontalRule,
    /// Checklist (task list)
    Checklist,
    /// Choice/selection
    Choice,
}

impl SectionType {
    /// Get string representation of section type
    pub fn as_str(&self) -> &'static str {
        match self {
            SectionType::Heading => "heading",
            SectionType::Paragraph => "paragraph",
            SectionType::List => "list",
            SectionType::Code => "code",
            SectionType::Table => "table",
            SectionType::Blockquote => "blockquote",
            SectionType::HorizontalRule => "hr",
            SectionType::Checklist => "checklist",
            SectionType::Choice => "choice",
        }
    }
}

impl std::fmt::Display for SectionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A parsed section from a Markdown document
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedSection {
    /// Unique identifier for this section
    pub id: String,
    /// Type of section
    pub section_type: SectionType,
    /// Heading level (1-6) if this is a heading, None otherwise
    pub level: Option<u8>,
    /// Raw content of the section
    pub content: String,
    /// Zero-based index indicating section order in document
    pub order_idx: u32,
    /// Variable names found in this section's content
    pub variables: Vec<String>,
}

impl ParsedSection {
    /// Create a new ParsedSection with a generated UUID
    pub fn new(section_type: SectionType, content: String, order_idx: u32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            section_type,
            level: None,
            content,
            order_idx,
            variables: Vec::new(),
        }
    }

    /// Create a new ParsedSection with a specific ID
    pub fn with_id(id: String, section_type: SectionType, content: String, order_idx: u32) -> Self {
        Self {
            id,
            section_type,
            level: None,
            content,
            order_idx,
            variables: Vec::new(),
        }
    }

    /// Set the heading level
    pub fn with_level(mut self, level: u8) -> Self {
        self.level = Some(level);
        self
    }

    /// Set the variables found in this section
    pub fn with_variables(mut self, variables: Vec<String>) -> Self {
        self.variables = variables;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_type_as_str() {
        assert_eq!(SectionType::Heading.as_str(), "heading");
        assert_eq!(SectionType::Paragraph.as_str(), "paragraph");
        assert_eq!(SectionType::List.as_str(), "list");
        assert_eq!(SectionType::Code.as_str(), "code");
        assert_eq!(SectionType::Table.as_str(), "table");
        assert_eq!(SectionType::Blockquote.as_str(), "blockquote");
        assert_eq!(SectionType::HorizontalRule.as_str(), "hr");
        assert_eq!(SectionType::Checklist.as_str(), "checklist");
        assert_eq!(SectionType::Choice.as_str(), "choice");
    }

    #[test]
    fn test_section_type_display() {
        assert_eq!(format!("{}", SectionType::Heading), "heading");
        assert_eq!(format!("{}", SectionType::Code), "code");
    }

    #[test]
    fn test_parsed_section_new() {
        let section = ParsedSection::new(SectionType::Paragraph, "Hello".to_string(), 0);
        assert!(!section.id.is_empty());
        assert_eq!(section.section_type, SectionType::Paragraph);
        assert_eq!(section.content, "Hello");
        assert_eq!(section.order_idx, 0);
        assert!(section.level.is_none());
        assert!(section.variables.is_empty());
    }

    #[test]
    fn test_parsed_section_builder() {
        let section = ParsedSection::new(SectionType::Heading, "Title".to_string(), 0)
            .with_level(1)
            .with_variables(vec!["name".to_string()]);

        assert_eq!(section.level, Some(1));
        assert_eq!(section.variables, vec!["name"]);
    }
}
