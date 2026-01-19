//! # md-parser
//!
//! A Rust crate for structured Markdown parsing with sections, variables, and checklists.
//!
//! ## Features
//!
//! - **Core Parsing**: Parse Markdown into structured sections (heading, paragraph, list, code, blockquote, hr)
//! - **Checklist Extraction**: Extract `- [ ]` and `- [x]` items with completion status and nesting
//! - **Variable Detection**: Detect `{{variable_name}}` template variables
//! - **Frontmatter**: Parse YAML frontmatter (feature-gated with `frontmatter`)
//! - **PyO3 Bindings**: Python bindings via PyO3 (feature-gated with `pyo3`)
//! - **Serde Support**: Serialization support (feature-gated with `serde`)
//!
//! ## Quick Start
//!
//! ```rust
//! use md_parser::{MarkdownParser, extract_checklist_items, ChecklistSummary};
//!
//! let content = r#"
//! # My Document
//!
//! ## Tasks
//! - [ ] Task 1 (AC: 1)
//!   - [x] Subtask 1.1
//! - [x] Task 2 (AC: 2, 3)
//! "#;
//!
//! // Full document parsing
//! let parser = MarkdownParser::new();
//! let doc = parser.parse(content).unwrap();
//!
//! println!("Title: {:?}", doc.title);
//! println!("Sections: {}", doc.sections.len());
//! println!("Variables: {:?}", doc.variables);
//!
//! // Standalone checklist extraction
//! let items = extract_checklist_items(content);
//! let summary = ChecklistSummary::from_items(&items);
//! println!("Completion: {:.1}%", summary.percentage);
//! ```
//!
//! ## Feature Flags
//!
//! - `serde`: Enable serde serialization for all types
//! - `frontmatter`: Enable YAML frontmatter parsing (requires `serde`)
//! - `pyo3`: Enable Python bindings (requires `serde`)

// Modules
mod checklist;
mod document;
mod error;
mod parser;
mod section;
mod variables;

#[cfg(feature = "frontmatter")]
pub mod frontmatter;

#[cfg(feature = "pyo3")]
mod python;

// Re-exports
pub use checklist::{extract_checklist_items, ChecklistItem, ChecklistSummary};
pub use document::{EdgeType, ParsedDocument, ParsedEdge};
pub use error::ParseError;
pub use parser::MarkdownParser;
pub use section::{ParsedSection, SectionType};
pub use variables::{count_variables, extract_unique_variables, extract_variables, has_variables};

#[cfg(feature = "frontmatter")]
pub use frontmatter::{parse_frontmatter, strip_frontmatter};

// PyO3 module definition
#[cfg(feature = "pyo3")]
use pyo3::prelude::*;

#[cfg(feature = "pyo3")]
#[pymodule]
fn md_parser(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<python::PyMarkdownParser>()?;
    m.add_class::<python::PyParsedDocument>()?;
    m.add_class::<python::PyParsedSection>()?;
    m.add_class::<python::PyChecklistItem>()?;
    m.add_class::<python::PyChecklistSummary>()?;
    m.add_class::<python::PyParsedEdge>()?;

    // Add standalone functions
    m.add_function(wrap_pyfunction!(python::py_extract_checklist_items, m)?)?;
    m.add_function(wrap_pyfunction!(python::py_extract_variables, m)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        let content = r#"
# My Document

## Tasks

- [ ] Task 1 (AC: 1)
  - [x] Subtask 1.1
- [x] Task 2 (AC: 2, 3)

Some text with {{variable}} template.
"#;

        let parser = MarkdownParser::new();
        let doc = parser.parse(content).unwrap();

        assert_eq!(doc.title, Some("My Document".to_string()));
        assert!(!doc.sections.is_empty());
        assert!(doc.variables.contains(&"variable".to_string()));
        assert_eq!(doc.checklist_items.len(), 3);

        let summary = doc.checklist_summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.completed, 2);
    }

    #[test]
    fn test_standalone_checklist_extraction() {
        let content = "- [ ] A\n- [x] B\n- [ ] C";
        let items = extract_checklist_items(content);

        assert_eq!(items.len(), 3);

        let summary = ChecklistSummary::from_items(&items);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.completed, 1);
        assert!((summary.percentage - 33.333333333333336).abs() < 0.001);
    }

    #[test]
    fn test_variable_extraction() {
        let content = "Hello {{name}}, your {{order}} is ready!";

        assert!(has_variables(content));
        assert_eq!(count_variables(content), 2);

        let vars = extract_unique_variables(content);
        assert_eq!(vars, vec!["name", "order"]);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_serialization() {
        let parser = MarkdownParser::new();
        let doc = parser.parse("# Test\n\nContent").unwrap();

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("\"title\":\"Test\""));

        let deserialized: ParsedDocument = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, Some("Test".to_string()));
    }

    #[cfg(feature = "frontmatter")]
    #[test]
    fn test_frontmatter_parsing() {
        let content = "---\ntitle: Test Doc\nauthor: Me\n---\n\n# Content";
        let parser = MarkdownParser::new();
        let doc = parser.parse(content).unwrap();

        assert!(doc.frontmatter.is_some());
        let fm = doc.frontmatter.unwrap();
        assert_eq!(
            fm.get("title"),
            Some(&serde_yaml::Value::String("Test Doc".to_string()))
        );
    }
}
