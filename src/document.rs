//! Document types for parsed Markdown

use crate::checklist::{ChecklistItem, ChecklistSummary};
use crate::section::ParsedSection;

/// Type of edge relationship between sections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeType {
    /// Sequential relationship - one section follows another
    Follows,
    /// Containment relationship - one section contains another
    Contains,
}

impl EdgeType {
    /// Get string representation of edge type
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeType::Follows => "follows",
            EdgeType::Contains => "contains",
        }
    }
}

impl std::fmt::Display for EdgeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// An edge connecting two sections in the document
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedEdge {
    /// Index of the source section
    pub source_idx: usize,
    /// Index of the target section
    pub target_idx: usize,
    /// Type of relationship
    pub edge_type: EdgeType,
}

impl ParsedEdge {
    /// Create a new edge
    pub fn new(source_idx: usize, target_idx: usize, edge_type: EdgeType) -> Self {
        Self {
            source_idx,
            target_idx,
            edge_type,
        }
    }

    /// Create a "follows" edge between two sections
    pub fn follows(source_idx: usize, target_idx: usize) -> Self {
        Self::new(source_idx, target_idx, EdgeType::Follows)
    }

    /// Create a "contains" edge between two sections
    pub fn contains(source_idx: usize, target_idx: usize) -> Self {
        Self::new(source_idx, target_idx, EdgeType::Contains)
    }
}

/// A fully parsed Markdown document
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParsedDocument {
    /// Document title (extracted from first H1)
    pub title: Option<String>,
    /// All sections in the document
    pub sections: Vec<ParsedSection>,
    /// All unique variable names found in the document
    pub variables: Vec<String>,
    /// Edges representing relationships between sections
    pub edges: Vec<ParsedEdge>,
    /// All checklist items found in the document
    pub checklist_items: Vec<ChecklistItem>,
    /// YAML frontmatter (when frontmatter feature is enabled)
    #[cfg(feature = "frontmatter")]
    pub frontmatter: Option<std::collections::HashMap<String, serde_yaml::Value>>,
}

impl ParsedDocument {
    /// Create a new empty document
    pub fn new() -> Self {
        Self {
            title: None,
            sections: Vec::new(),
            variables: Vec::new(),
            edges: Vec::new(),
            checklist_items: Vec::new(),
            #[cfg(feature = "frontmatter")]
            frontmatter: None,
        }
    }

    /// Get a summary of checklist completion
    pub fn checklist_summary(&self) -> ChecklistSummary {
        ChecklistSummary::from_items(&self.checklist_items)
    }

    /// Get section by index
    pub fn get_section(&self, idx: usize) -> Option<&ParsedSection> {
        self.sections.get(idx)
    }

    /// Get section by ID
    pub fn get_section_by_id(&self, id: &str) -> Option<&ParsedSection> {
        self.sections.iter().find(|s| s.id == id)
    }

    /// Get all sections of a specific type
    pub fn sections_by_type(
        &self,
        section_type: crate::section::SectionType,
    ) -> Vec<&ParsedSection> {
        self.sections
            .iter()
            .filter(|s| s.section_type == section_type)
            .collect()
    }
}

impl Default for ParsedDocument {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::section::SectionType;

    #[test]
    fn test_edge_type_as_str() {
        assert_eq!(EdgeType::Follows.as_str(), "follows");
        assert_eq!(EdgeType::Contains.as_str(), "contains");
    }

    #[test]
    fn test_parsed_edge_constructors() {
        let follows = ParsedEdge::follows(0, 1);
        assert_eq!(follows.source_idx, 0);
        assert_eq!(follows.target_idx, 1);
        assert_eq!(follows.edge_type, EdgeType::Follows);

        let contains = ParsedEdge::contains(0, 1);
        assert_eq!(contains.edge_type, EdgeType::Contains);
    }

    #[test]
    fn test_parsed_document_new() {
        let doc = ParsedDocument::new();
        assert!(doc.title.is_none());
        assert!(doc.sections.is_empty());
        assert!(doc.variables.is_empty());
        assert!(doc.edges.is_empty());
        assert!(doc.checklist_items.is_empty());
    }

    #[test]
    fn test_parsed_document_checklist_summary() {
        let mut doc = ParsedDocument::new();
        doc.checklist_items.push(ChecklistItem {
            text: "Task 1".to_string(),
            checked: true,
            indent: 0,
            ac_refs: vec![],
        });
        doc.checklist_items.push(ChecklistItem {
            text: "Task 2".to_string(),
            checked: false,
            indent: 0,
            ac_refs: vec![],
        });

        let summary = doc.checklist_summary();
        assert_eq!(summary.total, 2);
        assert_eq!(summary.completed, 1);
        assert!((summary.percentage - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_sections_by_type() {
        let mut doc = ParsedDocument::new();
        doc.sections.push(crate::section::ParsedSection::new(
            SectionType::Heading,
            "Title".to_string(),
            0,
        ));
        doc.sections.push(crate::section::ParsedSection::new(
            SectionType::Paragraph,
            "Text".to_string(),
            1,
        ));
        doc.sections.push(crate::section::ParsedSection::new(
            SectionType::Heading,
            "Subtitle".to_string(),
            2,
        ));

        let headings = doc.sections_by_type(SectionType::Heading);
        assert_eq!(headings.len(), 2);
    }
}
