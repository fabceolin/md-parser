//! Checklist extraction from Markdown content

use regex::Regex;
use std::sync::LazyLock;

/// Regex for matching checklist items: `- [ ]` or `- [x]`
static CHECKLIST_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\s*)- \[([ xX])\] (.+)$").expect("Invalid checklist regex"));

/// Regex for extracting AC references: `(AC: 1, 2, 3)`
static AC_REF_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\(AC:\s*([^)]+)\)").expect("Invalid AC reference regex"));

/// A single checklist item extracted from Markdown
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChecklistItem {
    /// The text content of the checklist item (without the checkbox)
    pub text: String,
    /// Whether the item is checked (`[x]` vs `[ ]`)
    pub checked: bool,
    /// Indentation level (0 = top level, 1 = nested once, etc.)
    /// Calculated as: (leading_spaces / 2)
    pub indent: u32,
    /// Acceptance criteria references extracted from `(AC: 1, 2, 3)` pattern
    pub ac_refs: Vec<String>,
}

impl ChecklistItem {
    /// Create a new checklist item
    pub fn new(text: String, checked: bool, indent: u32) -> Self {
        Self {
            text,
            checked,
            indent,
            ac_refs: Vec::new(),
        }
    }

    /// Create a checklist item with AC references
    pub fn with_ac_refs(mut self, ac_refs: Vec<String>) -> Self {
        self.ac_refs = ac_refs;
        self
    }
}

/// Summary of checklist completion status
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChecklistSummary {
    /// Total number of checklist items
    pub total: usize,
    /// Number of completed (checked) items
    pub completed: usize,
    /// Number of pending (unchecked) items
    pub pending: usize,
    /// Completion percentage (0.0 - 100.0)
    pub percentage: f64,
}

impl ChecklistSummary {
    /// Create a summary from a slice of checklist items
    pub fn from_items(items: &[ChecklistItem]) -> Self {
        let total = items.len();
        let completed = items.iter().filter(|item| item.checked).count();
        let pending = total - completed;
        let percentage = if total > 0 {
            (completed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            total,
            completed,
            pending,
            percentage,
        }
    }

    /// Check if all items are completed
    pub fn is_complete(&self) -> bool {
        self.total > 0 && self.completed == self.total
    }

    /// Check if no items are completed
    pub fn is_empty(&self) -> bool {
        self.total == 0 || self.completed == 0
    }
}

impl Default for ChecklistSummary {
    fn default() -> Self {
        Self {
            total: 0,
            completed: 0,
            pending: 0,
            percentage: 0.0,
        }
    }
}

/// Extract all checklist items from Markdown content
///
/// Parses lines matching `- [ ] text` or `- [x] text` patterns,
/// tracking indentation level and extracting AC references.
///
/// # Example
///
/// ```
/// use md_parser::extract_checklist_items;
///
/// let content = r#"
/// - [ ] Task 1 (AC: 1)
///   - [x] Subtask 1.1
/// - [x] Task 2 (AC: 2, 3)
/// "#;
///
/// let items = extract_checklist_items(content);
/// assert_eq!(items.len(), 3);
/// assert_eq!(items[0].text, "Task 1 (AC: 1)");
/// assert!(!items[0].checked);
/// assert_eq!(items[0].indent, 0);
/// assert_eq!(items[0].ac_refs, vec!["1"]);
/// ```
pub fn extract_checklist_items(content: &str) -> Vec<ChecklistItem> {
    let mut items = Vec::new();

    for line in content.lines() {
        if let Some(caps) = CHECKLIST_REGEX.captures(line) {
            let indent_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let checked_char = caps.get(2).map(|m| m.as_str()).unwrap_or(" ");
            let text = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            // Calculate indent level (2 spaces = 1 level)
            let indent = (indent_str.len() / 2) as u32;
            let checked = checked_char.eq_ignore_ascii_case("x");

            // Extract AC references
            let ac_refs = extract_ac_refs(text);

            items.push(ChecklistItem {
                text: text.to_string(),
                checked,
                indent,
                ac_refs,
            });
        }
    }

    items
}

/// Extract AC references from text content
///
/// Parses `(AC: 1, 2, 3)` pattern and returns the individual references.
fn extract_ac_refs(text: &str) -> Vec<String> {
    AC_REF_REGEX
        .captures(text)
        .and_then(|caps| caps.get(1))
        .map(|m| {
            m.as_str()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_simple_checklist() {
        let content = "- [ ] Task 1\n- [x] Task 2";
        let items = extract_checklist_items(content);

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].text, "Task 1");
        assert!(!items[0].checked);
        assert_eq!(items[1].text, "Task 2");
        assert!(items[1].checked);
    }

    #[test]
    fn test_extract_nested_checklist() {
        let content = "- [ ] Parent\n  - [x] Child 1\n  - [ ] Child 2\n    - [x] Grandchild";
        let items = extract_checklist_items(content);

        assert_eq!(items.len(), 4);
        assert_eq!(items[0].indent, 0);
        assert_eq!(items[1].indent, 1);
        assert_eq!(items[2].indent, 1);
        assert_eq!(items[3].indent, 2);
    }

    #[test]
    fn test_extract_ac_references() {
        let content = "- [ ] Task (AC: 1, 2, 3)";
        let items = extract_checklist_items(content);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].ac_refs, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_extract_single_ac_reference() {
        let content = "- [ ] Task (AC: 5)";
        let items = extract_checklist_items(content);

        assert_eq!(items[0].ac_refs, vec!["5"]);
    }

    #[test]
    fn test_extract_no_ac_reference() {
        let content = "- [ ] Task without AC";
        let items = extract_checklist_items(content);

        assert!(items[0].ac_refs.is_empty());
    }

    #[test]
    fn test_case_insensitive_checkbox() {
        let content = "- [X] Task with uppercase X";
        let items = extract_checklist_items(content);

        assert!(items[0].checked);
    }

    #[test]
    fn test_checklist_summary() {
        let items = vec![
            ChecklistItem::new("Task 1".to_string(), true, 0),
            ChecklistItem::new("Task 2".to_string(), true, 0),
            ChecklistItem::new("Task 3".to_string(), false, 0),
            ChecklistItem::new("Task 4".to_string(), false, 0),
        ];

        let summary = ChecklistSummary::from_items(&items);

        assert_eq!(summary.total, 4);
        assert_eq!(summary.completed, 2);
        assert_eq!(summary.pending, 2);
        assert!((summary.percentage - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_checklist_summary_all_complete() {
        let items = vec![
            ChecklistItem::new("Task 1".to_string(), true, 0),
            ChecklistItem::new("Task 2".to_string(), true, 0),
        ];

        let summary = ChecklistSummary::from_items(&items);

        assert!(summary.is_complete());
        assert!(!summary.is_empty());
        assert!((summary.percentage - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_checklist_summary_empty() {
        let items: Vec<ChecklistItem> = vec![];
        let summary = ChecklistSummary::from_items(&items);

        assert_eq!(summary.total, 0);
        assert!(!summary.is_complete());
        assert!(summary.is_empty());
    }

    #[test]
    fn test_non_checklist_lines_ignored() {
        let content = "# Heading\n\n- [ ] Task\n\nRegular text\n\n- Normal list item";
        let items = extract_checklist_items(content);

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].text, "Task");
    }

    #[test]
    fn test_ac_refs_with_spaces() {
        let content = "- [ ] Task (AC:  1 ,  2 ,  3  )";
        let items = extract_checklist_items(content);

        assert_eq!(items[0].ac_refs, vec!["1", "2", "3"]);
    }
}
