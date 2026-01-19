# md-parser

A Rust crate for structured Markdown parsing with sections, variables, and checklists.

## Features

- **Core Parsing**: Parse Markdown into structured sections (heading, paragraph, list, code, blockquote, hr)
- **Checklist Extraction**: Extract `- [ ]` and `- [x]` items with completion status and nesting
- **Variable Detection**: Detect `{{variable_name}}` template variables
- **Frontmatter**: Parse YAML frontmatter (feature-gated)
- **PyO3 Bindings**: Python bindings via PyO3 (feature-gated)
- **WASM Compatible**: Builds for wasm32-unknown-unknown

## Installation

### Rust

```toml
[dependencies]
md-parser = { git = "https://github.com/fabceolin/md-parser" }

# With optional features
md-parser = { git = "https://github.com/fabceolin/md-parser", features = ["serde", "frontmatter"] }
```

### Python

Download wheels from [GitHub Releases](https://github.com/fabceolin/md-parser/releases):

```bash
pip install https://github.com/fabceolin/md-parser/releases/download/v0.1.0/md_parser-0.1.0-cp311-cp311-manylinux_2_17_x86_64.whl
```

## Usage

### Rust

```rust
use md_parser::{MarkdownParser, extract_checklist_items, ChecklistSummary};

let content = r#"
# My Document

## Tasks
- [ ] Task 1 (AC: 1)
  - [x] Subtask 1.1
- [x] Task 2 (AC: 2, 3)
"#;

// Full document parsing
let parser = MarkdownParser::new();
let doc = parser.parse(content).unwrap();

println!("Title: {:?}", doc.title);
println!("Sections: {}", doc.sections.len());

// Standalone checklist extraction
let items = extract_checklist_items(content);
let summary = ChecklistSummary::from_items(&items);
println!("Completion: {:.1}%", summary.percentage);
```

### Python

```python
from md_parser import MarkdownParser

content = """
# My Document

## Tasks
- [ ] Task 1 (AC: 1)
  - [x] Subtask 1.1
- [x] Task 2 (AC: 2, 3)

Some text with {{variable}} template.
"""

parser = MarkdownParser()
doc = parser.parse(content)

print(f"Title: {doc.title}")
print(f"Sections: {len(doc.sections)}")
print(f"Variables: {doc.variables}")

# Get completion summary
summary = doc.checklist_summary()
print(f"Completion: {summary.percentage:.1f}%")
```

## Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `serde` | Enable serde serialization | No |
| `frontmatter` | Enable YAML frontmatter parsing | No |
| `pyo3` | Enable Python bindings | No |

## License

MIT
