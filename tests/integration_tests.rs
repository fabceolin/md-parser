//! Integration tests for md-parser

use md_parser::{extract_checklist_items, ChecklistSummary, MarkdownParser, SectionType};
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

#[test]
fn test_parse_simple_document() {
    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&fixture_path("simple.md")).unwrap();

    assert_eq!(doc.title, Some("Simple Document".to_string()));
    assert!(doc.sections.len() >= 4); // H1, paragraph, H2, paragraph, H3, paragraph

    // Check heading levels
    let headings: Vec<_> = doc.sections_by_type(SectionType::Heading);
    assert!(headings.len() >= 3);

    // First heading should be level 1
    assert_eq!(headings[0].level, Some(1));
}

#[test]
fn test_parse_checklist_document() {
    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&fixture_path("with_checklist.md")).unwrap();

    // Should have 6 checklist items
    assert_eq!(doc.checklist_items.len(), 6);

    // Check completion summary
    let summary = doc.checklist_summary();
    assert_eq!(summary.total, 6);
    assert_eq!(summary.completed, 3); // Task 1.1, Task 2, Task 4
    assert_eq!(summary.pending, 3);

    // Check nested item indentation
    let nested = &doc.checklist_items[1]; // Subtask 1.1
    assert_eq!(nested.indent, 1);
    assert!(nested.checked);

    // Check AC references
    let task1 = &doc.checklist_items[0];
    assert_eq!(task1.ac_refs, vec!["1"]);

    let task2 = &doc.checklist_items[3];
    assert_eq!(task2.ac_refs, vec!["2", "3"]);
}

#[test]
fn test_parse_variables_document() {
    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&fixture_path("with_variables.md")).unwrap();

    // Should detect all unique variables
    assert!(doc.variables.contains(&"name".to_string()));
    assert!(doc.variables.contains(&"project_name".to_string()));
    assert!(doc.variables.contains(&"order_id".to_string()));
    assert!(doc.variables.contains(&"email".to_string()));

    // name appears twice but should be deduplicated
    assert_eq!(doc.variables.iter().filter(|v| *v == "name").count(), 1);
}

#[cfg(feature = "frontmatter")]
#[test]
fn test_parse_frontmatter_document() {
    let parser = MarkdownParser::new();
    let doc = parser
        .parse_file(&fixture_path("with_frontmatter.md"))
        .unwrap();

    assert!(doc.frontmatter.is_some());
    let fm = doc.frontmatter.unwrap();

    assert_eq!(
        fm.get("title"),
        Some(&serde_yaml::Value::String("Test Document".to_string()))
    );
    assert_eq!(
        fm.get("author"),
        Some(&serde_yaml::Value::String("John Doe".to_string()))
    );

    // Check nested values
    if let Some(serde_yaml::Value::Mapping(settings)) = fm.get("settings") {
        assert!(settings.contains_key(&serde_yaml::Value::String("debug".to_string())));
    }

    // Check arrays
    if let Some(serde_yaml::Value::Sequence(tags)) = fm.get("tags") {
        assert_eq!(tags.len(), 3);
    }

    // Variables should still be extracted from the content
    assert!(doc.variables.contains(&"title".to_string()));
    assert!(doc.variables.contains(&"author".to_string()));
}

#[test]
fn test_parse_malformed_document() {
    let parser = MarkdownParser::new();
    let result = parser.parse_file(&fixture_path("malformed.md"));

    // Should handle malformed content gracefully without panicking
    assert!(result.is_ok());
    let doc = result.unwrap();

    // Should have extracted what it could
    assert!(doc.sections.len() > 0);
}

#[test]
fn test_parse_bmad_story() {
    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&fixture_path("bmad_story.md")).unwrap();

    assert_eq!(doc.title, Some("Story TEA-TEST-001: Example Story".to_string()));

    // Check checklist items
    assert_eq!(doc.checklist_items.len(), 7);

    // Check completion
    let summary = doc.checklist_summary();
    assert_eq!(summary.completed, 3); // Create project, Initialize repo, Add deps

    // Check AC references
    let ac_refs: Vec<_> = doc
        .checklist_items
        .iter()
        .flat_map(|i| i.ac_refs.iter())
        .collect();
    assert!(ac_refs.contains(&&"1".to_string()));
    assert!(ac_refs.contains(&&"2".to_string()));
    assert!(ac_refs.contains(&&"3".to_string()));

    // Check variables
    assert!(doc.variables.contains(&"parser_type".to_string()));
    assert!(doc.variables.contains(&"template_name".to_string()));
}

#[test]
fn test_standalone_checklist_extraction() {
    let content = std::fs::read_to_string(fixture_path("with_checklist.md")).unwrap();
    let items = extract_checklist_items(&content);

    assert_eq!(items.len(), 6);

    let summary = ChecklistSummary::from_items(&items);
    assert_eq!(summary.total, 6);
    assert!((summary.percentage - 50.0).abs() < 0.1);
}

#[test]
fn test_edges_generated() {
    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&fixture_path("simple.md")).unwrap();

    // Should have n-1 edges for n sections
    assert_eq!(doc.edges.len(), doc.sections.len() - 1);

    // All edges should be "follows" type
    for edge in &doc.edges {
        assert_eq!(edge.edge_type, md_parser::EdgeType::Follows);
        assert_eq!(edge.target_idx, edge.source_idx + 1);
    }
}

#[test]
fn test_section_ids_unique() {
    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&fixture_path("bmad_story.md")).unwrap();

    let ids: Vec<_> = doc.sections.iter().map(|s| &s.id).collect();
    let unique_count = ids.iter().collect::<std::collections::HashSet<_>>().len();

    assert_eq!(ids.len(), unique_count, "All section IDs should be unique");
}

#[test]
fn test_order_idx_sequential() {
    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&fixture_path("simple.md")).unwrap();

    for (i, section) in doc.sections.iter().enumerate() {
        assert_eq!(
            section.order_idx, i as u32,
            "Section {} should have order_idx {}",
            i, i
        );
    }
}

#[cfg(feature = "serde")]
#[test]
fn test_json_serialization() {
    use md_parser::ParsedDocument;

    let parser = MarkdownParser::new();
    let doc = parser.parse_file(&fixture_path("simple.md")).unwrap();

    let json = serde_json::to_string(&doc).unwrap();
    assert!(json.contains("\"title\":\"Simple Document\""));

    // Deserialize and compare
    let deserialized: ParsedDocument = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.title, doc.title);
    assert_eq!(deserialized.sections.len(), doc.sections.len());
}
