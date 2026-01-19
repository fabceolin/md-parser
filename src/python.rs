//! Python bindings via PyO3
//!
//! This module provides Python wrappers for the md-parser types and functions.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::checklist::{self, ChecklistItem, ChecklistSummary};
use crate::document::{ParsedDocument, ParsedEdge};
use crate::parser::MarkdownParser;
use crate::section::ParsedSection;
use crate::variables;

/// Python wrapper for MarkdownParser
#[pyclass(name = "MarkdownParser")]
pub struct PyMarkdownParser {
    inner: MarkdownParser,
}

#[pymethods]
impl PyMarkdownParser {
    /// Create a new MarkdownParser
    #[new]
    pub fn new() -> Self {
        Self {
            inner: MarkdownParser::new(),
        }
    }

    /// Parse Markdown content into a structured document
    ///
    /// Args:
    ///     content: The Markdown content to parse
    ///
    /// Returns:
    ///     ParsedDocument containing sections, variables, edges, and checklist items
    ///
    /// Raises:
    ///     ValueError: If the markdown structure is invalid
    pub fn parse(&self, content: &str) -> PyResult<PyParsedDocument> {
        self.inner
            .parse(content)
            .map(PyParsedDocument::from)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
    }

    /// Parse Markdown from a file path
    ///
    /// Args:
    ///     path: Path to the Markdown file
    ///
    /// Returns:
    ///     ParsedDocument containing sections, variables, edges, and checklist items
    ///
    /// Raises:
    ///     FileNotFoundError: If the file doesn't exist
    ///     ValueError: If the markdown structure is invalid
    pub fn parse_file(&self, path: &str) -> PyResult<PyParsedDocument> {
        self.inner
            .parse_file(std::path::Path::new(path))
            .map(PyParsedDocument::from)
            .map_err(|e| match e {
                crate::error::ParseError::IoError(_) => {
                    pyo3::exceptions::PyFileNotFoundError::new_err(e.to_string())
                }
                _ => pyo3::exceptions::PyValueError::new_err(e.to_string()),
            })
    }
}

/// Python wrapper for ParsedDocument
#[pyclass(name = "ParsedDocument")]
#[derive(Clone)]
pub struct PyParsedDocument {
    /// Document title (from first H1)
    #[pyo3(get)]
    pub title: Option<String>,
    /// All sections in the document
    #[pyo3(get)]
    pub sections: Vec<PyParsedSection>,
    /// All unique variable names
    #[pyo3(get)]
    pub variables: Vec<String>,
    /// Edges between sections
    #[pyo3(get)]
    pub edges: Vec<PyParsedEdge>,
    /// Checklist items
    #[pyo3(get)]
    pub checklist_items: Vec<PyChecklistItem>,
    /// YAML frontmatter (when frontmatter feature is enabled)
    #[cfg(feature = "frontmatter")]
    frontmatter_inner: Option<std::collections::HashMap<String, serde_yaml::Value>>,
}

#[pymethods]
impl PyParsedDocument {
    /// Get a summary of checklist completion
    pub fn checklist_summary(&self) -> PyChecklistSummary {
        let items: Vec<ChecklistItem> = self
            .checklist_items
            .iter()
            .map(|i| ChecklistItem {
                text: i.text.clone(),
                checked: i.checked,
                indent: i.indent,
                ac_refs: i.ac_refs.clone(),
            })
            .collect();
        PyChecklistSummary::from(ChecklistSummary::from_items(&items))
    }

    /// Get frontmatter as a Python dict (requires frontmatter feature)
    #[cfg(feature = "frontmatter")]
    #[getter]
    pub fn frontmatter(&self, py: Python<'_>) -> PyResult<Option<Py<PyDict>>> {
        match &self.frontmatter_inner {
            Some(fm) => {
                let dict = PyDict::new(py);
                for (key, value) in fm {
                    dict.set_item(key, yaml_value_to_py(py, value)?)?;
                }
                Ok(Some(dict.into()))
            }
            None => Ok(None),
        }
    }

    /// Get section by index
    pub fn get_section(&self, idx: usize) -> Option<PyParsedSection> {
        self.sections.get(idx).cloned()
    }

    /// Get section by ID
    pub fn get_section_by_id(&self, id: &str) -> Option<PyParsedSection> {
        self.sections.iter().find(|s| s.id == id).cloned()
    }

    /// Get all sections of a specific type
    pub fn sections_by_type(&self, section_type: &str) -> Vec<PyParsedSection> {
        self.sections
            .iter()
            .filter(|s| s.section_type == section_type)
            .cloned()
            .collect()
    }

    /// Convert to JSON string (requires serde feature)
    pub fn to_json(&self) -> PyResult<String> {
        // Build a simple JSON manually
        let mut json = String::from("{");

        // Title
        match &self.title {
            Some(t) => json.push_str(&format!("\"title\":{},", serde_json::json!(t))),
            None => json.push_str("\"title\":null,"),
        }

        // Variables
        json.push_str(&format!(
            "\"variables\":{},",
            serde_json::json!(self.variables)
        ));

        // Sections count
        json.push_str(&format!("\"sections_count\":{},", self.sections.len()));

        // Checklist count
        json.push_str(&format!(
            "\"checklist_items_count\":{},",
            self.checklist_items.len()
        ));

        // Edges count
        json.push_str(&format!("\"edges_count\":{}", self.edges.len()));

        json.push('}');
        Ok(json)
    }

    fn __repr__(&self) -> String {
        format!(
            "ParsedDocument(title={:?}, sections={}, variables={:?}, checklist_items={})",
            self.title,
            self.sections.len(),
            self.variables,
            self.checklist_items.len()
        )
    }
}

impl From<ParsedDocument> for PyParsedDocument {
    fn from(doc: ParsedDocument) -> Self {
        Self {
            title: doc.title,
            sections: doc
                .sections
                .into_iter()
                .map(PyParsedSection::from)
                .collect(),
            variables: doc.variables,
            edges: doc.edges.into_iter().map(PyParsedEdge::from).collect(),
            checklist_items: doc
                .checklist_items
                .into_iter()
                .map(PyChecklistItem::from)
                .collect(),
            #[cfg(feature = "frontmatter")]
            frontmatter_inner: doc.frontmatter,
        }
    }
}

/// Python wrapper for ParsedSection
#[pyclass(name = "ParsedSection")]
#[derive(Clone)]
pub struct PyParsedSection {
    /// Unique identifier
    #[pyo3(get)]
    pub id: String,
    /// Section type as string
    #[pyo3(get)]
    pub section_type: String,
    /// Heading level (1-6) or None
    #[pyo3(get)]
    pub level: Option<u8>,
    /// Raw content
    #[pyo3(get)]
    pub content: String,
    /// Order index
    #[pyo3(get)]
    pub order_idx: u32,
    /// Variables found in content
    #[pyo3(get)]
    pub variables: Vec<String>,
}

#[pymethods]
impl PyParsedSection {
    fn __repr__(&self) -> String {
        format!(
            "ParsedSection(id={:?}, type={:?}, level={:?}, content={:?})",
            self.id,
            self.section_type,
            self.level,
            if self.content.len() > 50 {
                format!("{}...", &self.content[..50])
            } else {
                self.content.clone()
            }
        )
    }
}

impl From<ParsedSection> for PyParsedSection {
    fn from(section: ParsedSection) -> Self {
        Self {
            id: section.id,
            section_type: section.section_type.as_str().to_string(),
            level: section.level,
            content: section.content,
            order_idx: section.order_idx,
            variables: section.variables,
        }
    }
}

/// Python wrapper for ChecklistItem
#[pyclass(name = "ChecklistItem")]
#[derive(Clone)]
pub struct PyChecklistItem {
    /// Item text
    #[pyo3(get)]
    pub text: String,
    /// Whether checked
    #[pyo3(get)]
    pub checked: bool,
    /// Indentation level
    #[pyo3(get)]
    pub indent: u32,
    /// AC references
    #[pyo3(get)]
    pub ac_refs: Vec<String>,
}

#[pymethods]
impl PyChecklistItem {
    fn __repr__(&self) -> String {
        let status = if self.checked { "x" } else { " " };
        format!(
            "ChecklistItem([{}] {}, indent={})",
            status, self.text, self.indent
        )
    }
}

impl From<ChecklistItem> for PyChecklistItem {
    fn from(item: ChecklistItem) -> Self {
        Self {
            text: item.text,
            checked: item.checked,
            indent: item.indent,
            ac_refs: item.ac_refs,
        }
    }
}

/// Python wrapper for ChecklistSummary
#[pyclass(name = "ChecklistSummary")]
#[derive(Clone)]
pub struct PyChecklistSummary {
    /// Total items
    #[pyo3(get)]
    pub total: usize,
    /// Completed items
    #[pyo3(get)]
    pub completed: usize,
    /// Pending items
    #[pyo3(get)]
    pub pending: usize,
    /// Completion percentage
    #[pyo3(get)]
    pub percentage: f64,
}

#[pymethods]
impl PyChecklistSummary {
    /// Check if all items are completed
    pub fn is_complete(&self) -> bool {
        self.total > 0 && self.completed == self.total
    }

    /// Check if no items are completed
    pub fn is_empty(&self) -> bool {
        self.total == 0 || self.completed == 0
    }

    fn __repr__(&self) -> String {
        format!(
            "ChecklistSummary(total={}, completed={}, percentage={:.1}%)",
            self.total, self.completed, self.percentage
        )
    }
}

impl From<ChecklistSummary> for PyChecklistSummary {
    fn from(summary: ChecklistSummary) -> Self {
        Self {
            total: summary.total,
            completed: summary.completed,
            pending: summary.pending,
            percentage: summary.percentage,
        }
    }
}

/// Python wrapper for ParsedEdge
#[pyclass(name = "ParsedEdge")]
#[derive(Clone)]
pub struct PyParsedEdge {
    /// Source section index
    #[pyo3(get)]
    pub source_idx: usize,
    /// Target section index
    #[pyo3(get)]
    pub target_idx: usize,
    /// Edge type as string
    #[pyo3(get)]
    pub edge_type: String,
}

#[pymethods]
impl PyParsedEdge {
    fn __repr__(&self) -> String {
        format!(
            "ParsedEdge({} -> {}, type={:?})",
            self.source_idx, self.target_idx, self.edge_type
        )
    }
}

impl From<ParsedEdge> for PyParsedEdge {
    fn from(edge: ParsedEdge) -> Self {
        Self {
            source_idx: edge.source_idx,
            target_idx: edge.target_idx,
            edge_type: edge.edge_type.as_str().to_string(),
        }
    }
}

// Standalone functions

/// Extract checklist items from Markdown content
///
/// Args:
///     content: The Markdown content to parse
///
/// Returns:
///     List of ChecklistItem objects
#[pyfunction]
#[pyo3(name = "extract_checklist_items")]
pub fn py_extract_checklist_items(content: &str) -> Vec<PyChecklistItem> {
    checklist::extract_checklist_items(content)
        .into_iter()
        .map(PyChecklistItem::from)
        .collect()
}

/// Extract variable names from content
///
/// Args:
///     content: The content to search for variables
///
/// Returns:
///     List of unique variable names (sorted)
#[pyfunction]
#[pyo3(name = "extract_variables")]
pub fn py_extract_variables(content: &str) -> Vec<String> {
    variables::extract_unique_variables(content)
}

// Helper function to convert serde_yaml::Value to Python object
#[cfg(feature = "frontmatter")]
fn yaml_value_to_py(py: Python<'_>, value: &serde_yaml::Value) -> PyResult<PyObject> {
    use pyo3::types::{PyList, PyString};
    use pyo3::IntoPy;

    match value {
        serde_yaml::Value::Null => Ok(py.None()),
        serde_yaml::Value::Bool(b) => Ok(b.into_py(py)),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Ok(py.None())
            }
        }
        serde_yaml::Value::String(s) => Ok(PyString::new(py, s).into_any().unbind()),
        serde_yaml::Value::Sequence(seq) => {
            let list = PyList::empty(py);
            for item in seq {
                list.append(yaml_value_to_py(py, item)?)?;
            }
            Ok(list.into_any().unbind())
        }
        serde_yaml::Value::Mapping(map) => {
            let dict = PyDict::new(py);
            for (k, v) in map {
                if let serde_yaml::Value::String(key) = k {
                    dict.set_item(key, yaml_value_to_py(py, v)?)?;
                }
            }
            Ok(dict.into_any().unbind())
        }
        serde_yaml::Value::Tagged(tagged) => yaml_value_to_py(py, &tagged.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_py_markdown_parser() {
        let parser = PyMarkdownParser::new();
        let doc = parser.parse("# Test\n\nContent").unwrap();

        assert_eq!(doc.title, Some("Test".to_string()));
        assert_eq!(doc.sections.len(), 2);
    }

    #[test]
    fn test_py_checklist_extraction() {
        let items = py_extract_checklist_items("- [ ] A\n- [x] B");

        assert_eq!(items.len(), 2);
        assert!(!items[0].checked);
        assert!(items[1].checked);
    }

    #[test]
    fn test_py_variable_extraction() {
        let vars = py_extract_variables("Hello {{name}} and {{place}}!");

        assert_eq!(vars, vec!["name", "place"]);
    }

    #[test]
    fn test_py_checklist_summary() {
        let parser = PyMarkdownParser::new();
        let doc = parser.parse("- [ ] A\n- [x] B\n- [x] C").unwrap();

        let summary = doc.checklist_summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.completed, 2);
        assert!((summary.percentage - 66.66666666666667).abs() < 0.001);
    }
}
