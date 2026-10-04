# wo-x2t Conversion Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement real format-to-format conversion in wo-x2t by adding converter implementations that parse source formats into an IntermediateDocument, then serialize to target formats, replacing the current identity stub in roundtrip.rs.

**Architecture:** Each converter parses source bytes using the parser crate's native struct (e.g., `RtfDocument`, `HtmlDocument`, `TxtDocument`), converts it to our `IntermediateDocument` type, then serializes the `IntermediateDocument` to the target format using the target crate's serializer. A `ConverterRegistry` maps `(source, target)` format pairs to converter instances. The `ConversionRouter` uses this registry to dispatch actual conversions.

**Tech Stack:** Rust, wo-rtf (RtfParser/RtfSerializer), wo-html (HtmlParser/HtmlSerializer), wo-txt (TxtParser/TxtSerializer), wo-common (CoreError), thiserror

---

## Verified Type Reference

### RTF types (wo-rtf/src/model.rs)
```rust
pub struct RtfDocument {
    pub version: u32,
    pub ansi_codepage: Option<u32>,
    pub fonts: Vec<RtfFont>,
    pub colors: Vec<RtfColor>,
    pub body: Vec<RtfBlock>,
    pub info: Option<RtfInfo>,
}

pub enum RtfBlock {
    Paragraph {
        content: Vec<RtfInline>,
        alignment: Option<RtfAlignment>,
        indent_left: Option<i32>,
        indent_first: Option<i32>,
    },
    Table { rows: Vec<RtfTableRow> },
}

pub enum RtfInline {
    Text { text: String },
    Bold { content: Vec<RtfInline> },
    Italic { content: Vec<RtfInline> },
    Underline { content: Vec<RtfInline> },
    Strikethrough { content: Vec<RtfInline> },
    Superscript { content: Vec<RtfInline> },
    Subscript { content: Vec<RtfInline> },
    Font { index: u32, content: Vec<RtfInline> },
    FontSize { half_points: u32, content: Vec<RtfInline> },
    Color { index: u32, content: Vec<RtfInline> },
    LineBreak,
    PageBreak,
    Tab,
}
```

### HTML types (wo-html/src/model.rs)
```rust
pub struct HtmlDocument {
    pub doc_type: Option<String>,
    pub html_attributes: Vec<(String, String)>,
    pub head: HtmlHead,
    pub body: HtmlBody,
}

pub enum BlockElement {
    Heading { level: u8, content: Vec<InlineElement>, id: Option<String> },
    Paragraph { content: Vec<InlineElement>, id: Option<String> },
    Div { elements: Vec<BlockElement>, id: Option<String>, class: Option<String> },
    UnorderedList { items: Vec<ListItem>, id: Option<String> },
    OrderedList { items: Vec<ListItem>, id: Option<String>, start: Option<u32> },
    Table { rows: Vec<TableRow>, id: Option<String> },
    Blockquote { elements: Vec<BlockElement>, id: Option<String> },
    Pre { content: String, id: Option<String> },
    HorizontalRule,
    RawHtml { tag: String, content: String },
}

pub enum InlineElement {
    Text { text: String },
    Bold { content: Vec<InlineElement> },
    Italic { content: Vec<InlineElement> },
    Underline { content: Vec<InlineElement> },
    Strikethrough { content: Vec<InlineElement> },
    Subscript { content: Vec<InlineElement> },
    Superscript { content: Vec<InlineElement> },
    Code { content: String },
    Link { href: String, title: Option<String>, content: Vec<InlineElement> },
    Image { src: String, alt: Option<String>, title: Option<String> },
    LineBreak,
}
```

### TXT types (wo-txt/src/parser.rs)
```rust
pub struct TxtDocument {
    pub lines: Vec<String>,
    pub encoding: Encoding,
    pub had_bom: bool,
}
```

### Parser/Serializer APIs
- `RtfParser::new().parse(&[u8]) -> Result<RtfDocument>` (uses `wo_common::Result`)
- `RtfSerializer::new().serialize(&RtfDocument) -> String`
- `HtmlParser::new().parse(&[u8]) -> Result<HtmlDocument>` (uses `wo_common::Result`)
- `HtmlSerializer::new().serialize(&HtmlDocument) -> String`
- `TxtParser::new().parse(&[u8]) -> Result<TxtDocument>` (uses `wo_common::Result`)
- `TxtSerializer::new().serialize(&TxtDocument) -> Result<Vec<u8>>` (uses `wo_common::Result`)
- `TxtSerializer::with_options(SerializeOptions::unix()) -> TxtSerializer` (LF, no BOM)

### Existing wo-x2t model (DO NOT CHANGE)
```rust
pub struct ConversionInput {
    pub source_format: String,
    pub target_format: String,
    pub data: Vec<u8>,
    pub options: ConversionOptions,
}
pub struct ConversionOutput { pub data: Vec<u8>, pub format: String, pub page_count: Option<u32>, pub warnings: Vec<String> }
pub struct ConversionResult { pub status: ConversionStatus, pub output: Option<ConversionOutput>, pub error: Option<String>, pub duration_ms: u64 }
pub enum ConversionStatus { Success, PartialSuccess, Failed, UnsupportedFormat, Timeout }
pub struct ConversionOptions { pub page_range: Option<(u32, u32)>, pub quality: Option<u8>, pub password: Option<String>, pub embed_fonts: bool, pub pdfa_compliant: bool }
```

### Test command (Windows)
```bash
wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t"
```

---

### Task 1: Error types in error.rs

**Files:**
- Create: `core/crates/wo-x2t/src/error.rs`

- [ ] **Step 1: Write error.rs**

```rust
//! Conversion error types for wo-x2t.

use thiserror::Error;

/// Errors that can occur during format conversion.
#[derive(Debug, Error)]
pub enum ConversionError {
    /// The source format could not be parsed.
    #[error("parse error: {0}")]
    Parse(String),

    /// The target format could not be serialized.
    #[error("serialization error: {0}")]
    Serialize(String),

    /// The conversion path is not supported.
    #[error("unsupported conversion: {source} -> {target}")]
    UnsupportedConversion {
        source: String,
        target: String,
    },

    /// No converter registered for the given format pair.
    #[error("no converter registered for {source} -> {target}")]
    NoConverter {
        source: String,
        target: String,
    },
}
```

- [ ] **Step 2: Verify compilation**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo check -p wo-x2t"`
Expected: Compilation error because error.rs is not yet referenced from lib.rs — that's fine, next task adds it.

- [ ] **Step 3: Commit**

```bash
git add core/crates/wo-x2t/src/error.rs
git commit -m "feat(wo-x2t): add ConversionError enum"
```

---

### Task 2: IntermediateDocument type and FormatConverter trait in converter.rs

**Files:**
- Create: `core/crates/wo-x2t/src/converter.rs`

- [ ] **Step 1: Write converter.rs with IntermediateDocument, FormatConverter trait, and helpers**

```rust
//! Format converter trait and intermediate document model.
//!
//! Converters parse a source format into IntermediateDocument,
//! then serialize IntermediateDocument to a target format.

use crate::error::ConversionError;
use crate::model::{ConversionOutput, ConversionResult, ConversionStatus};

/// Intermediate document representation used as the common format
/// between all conversions.
///
/// This is a lossy representation: it captures the most common document
/// elements (paragraphs, headings, formatting) but does not preserve
/// every detail of the source format.
#[derive(Debug, Clone)]
pub struct IntermediateDocument {
    /// Document title (from metadata, if available).
    pub title: Option<String>,
    /// Document author (from metadata, if available).
    pub author: Option<String>,
    /// Document blocks (paragraphs, headings, etc.).
    pub blocks: Vec<Block>,
}

/// A block-level element in the intermediate document.
#[derive(Debug, Clone)]
pub enum Block {
    /// A heading with a level (1-6).
    Heading {
        level: u8,
        content: Vec<Span>,
    },
    /// A paragraph of text.
    Paragraph {
        content: Vec<Span>,
    },
    /// A horizontal rule.
    HorizontalRule,
}

/// Inline formatting within a block.
#[derive(Debug, Clone)]
pub enum Span {
    /// Plain text.
    Text(String),
    /// Bold text.
    Bold(Vec<Span>),
    /// Italic text.
    Italic(Vec<Span>),
    /// Underlined text.
    Underline(Vec<Span>),
    /// Strikethrough text.
    Strikethrough(Vec<Span>),
    /// A line break.
    LineBreak,
}

/// Trait for format-to-format converters.
///
/// Each converter handles one specific (source, target) pair.
/// The conversion goes: source bytes -> source native type -> IntermediateDocument -> target bytes.
pub trait FormatConverter: Send + Sync {
    /// The source format identifier (e.g., "rtf", "html", "txt").
    fn source_format(&self) -> &str;

    /// The target format identifier.
    fn target_format(&self) -> &str;

    /// Convert input bytes from source format to target format.
    ///
    /// Returns the output bytes on success.
    fn convert(&self, data: &[u8]) -> Result<Vec<u8>, ConversionError>;
}

/// Build a successful ConversionResult from output bytes.
pub fn success_result(data: Vec<u8>, format: &str, duration_ms: u64) -> ConversionResult {
    ConversionResult {
        status: ConversionStatus::Success,
        output: Some(ConversionOutput {
            data,
            format: format.to_string(),
            page_count: None,
            warnings: Vec::new(),
        }),
        error: None,
        duration_ms,
    }
}

/// Build a failed ConversionResult from an error message.
pub fn error_result(message: String, duration_ms: u64) -> ConversionResult {
    ConversionResult {
        status: ConversionStatus::Failed,
        output: None,
        error: Some(message),
        duration_ms,
    }
}

/// Build an unsupported-format ConversionResult.
pub fn unsupported_result(source: &str, target: &str) -> ConversionResult {
    ConversionResult {
        status: ConversionStatus::UnsupportedFormat,
        output: None,
        error: Some(format!("Conversion from {} to {} is not supported", source, target)),
        duration_ms: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intermediate_document_creation() {
        let doc = IntermediateDocument {
            title: Some("Test".to_string()),
            author: Some("Author".to_string()),
            blocks: vec![Block::Paragraph {
                content: vec![Span::Text("Hello world".to_string())],
            }],
        };
        assert_eq!(doc.title.as_deref(), Some("Test"));
        assert_eq!(doc.blocks.len(), 1);
    }

    #[test]
    fn test_success_result() {
        let result = success_result(vec![1, 2, 3], "txt", 10);
        assert_eq!(result.status, ConversionStatus::Success);
        assert!(result.output.is_some());
        assert_eq!(result.output.unwrap().format, "txt");
    }

    #[test]
    fn test_error_result() {
        let result = error_result("something failed".to_string(), 5);
        assert_eq!(result.status, ConversionStatus::Failed);
        assert_eq!(result.error.as_deref(), Some("something failed"));
    }

    #[test]
    fn test_unsupported_result() {
        let result = unsupported_result("pdf", "docx");
        assert_eq!(result.status, ConversionStatus::UnsupportedFormat);
        assert!(result.error.unwrap().contains("pdf"));
    }
}
```

- [ ] **Step 2: Verify tests compile and pass**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t -- converter::tests"`
Expected: May fail to compile because lib.rs doesn't reference the new modules yet — we'll wire that in Step 4 after adding Cargo.toml deps. For now, skip to Step 3.

- [ ] **Step 3: Commit**

```bash
git add core/crates/wo-x2t/src/converter.rs
git commit -m "feat(wo-x2t): add IntermediateDocument, FormatConverter trait, and result helpers"
```

---

### Task 3: Add parser crate dependencies to Cargo.toml

**Files:**
- Modify: `core/crates/wo-x2t/Cargo.toml`

- [ ] **Step 1: Add wo-rtf, wo-html, wo-txt as dependencies**

Replace the entire Cargo.toml contents with:

```toml
[package]
name = "wo-x2t"
version = "0.1.0"
edition = "2021"
license = "AGPL-3.0-or-later"

[dependencies]
anyhow = { workspace = true }
thiserror = { workspace = true }
serde = { workspace = true }
wo-common = { path = "../wo-common" }
serde_json = { workspace = true }
wo-rtf = { path = "../wo-rtf" }
wo-html = { path = "../wo-html" }
wo-txt = { path = "../wo-txt" }
```

- [ ] **Step 2: Verify dependencies resolve**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo check -p wo-x2t"`
Expected: Compiles (may have warnings about unused modules, which is fine).

- [ ] **Step 3: Commit**

```bash
git add core/crates/wo-x2t/Cargo.toml
git commit -m "feat(wo-x2t): add wo-rtf, wo-html, wo-txt as dependencies"
```

---

### Task 4: Update lib.rs to export new modules

**Files:**
- Modify: `core/crates/wo-x2t/src/lib.rs`

- [ ] **Step 1: Add module declarations and re-exports**

Replace the entire lib.rs contents with:

```rust
// wo-x2t -- World-Office format conversion orchestrator
//!
//! Routes conversion requests between format modules and manages
//! the conversion pipeline. Replaces the C++ X2tConverter (38 files).

pub mod converter;
pub mod error;
pub mod model;
pub mod roundtrip;
pub mod router;

pub use converter::{
    Block, FormatConverter, IntermediateDocument, Span,
    error_result, success_result, unsupported_result,
};
pub use error::ConversionError;
pub use model::{ConversionInput, ConversionOutput, ConversionResult, ConversionStatus};
pub use roundtrip::X2tRoundtrip;
pub use router::ConversionRouter;

pub const FORMAT_NAME: &str = "x2t";
```

- [ ] **Step 2: Verify compilation**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo check -p wo-x2t"`
Expected: Compiles successfully.

- [ ] **Step 3: Run existing tests to ensure no regressions**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t"`
Expected: All existing tests pass (router tests, roundtrip tests, converter tests).

- [ ] **Step 4: Commit**

```bash
git add core/crates/wo-x2t/src/lib.rs
git commit -m "feat(wo-x2t): export converter, error modules from lib.rs"
```

---

### Task 5: RTF -> TXT converter

**Files:**
- Create: `core/crates/wo-x2t/src/converters/rtf_to_txt.rs`

- [ ] **Step 1: Write the failing test**

```rust
//! RTF to TXT converter.

use crate::converter::FormatConverter;
use crate::error::ConversionError;

/// Converts RTF documents to plain text.
pub struct RtfToTxtConverter;

impl RtfToTxtConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RtfToTxtConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatConverter for RtfToTxtConverter {
    fn source_format(&self) -> &str {
        "rtf"
    }

    fn target_format(&self) -> &str {
        "txt"
    }

    fn convert(&self, data: &[u8]) -> Result<Vec<u8>, ConversionError> {
        let parser = wo_rtf::RtfParser::new();
        let doc = parser.parse(data).map_err(|e| ConversionError::Parse(e.to_string()))?;

        let mut lines: Vec<String> = Vec::new();
        let mut current_line = String::new();

        for block in &doc.body {
            match block {
                wo_rtf::model::RtfBlock::Paragraph { content, .. } => {
                    current_line.clear();
                    extract_text_from_inlines(content, &mut current_line);
                    lines.push(current_line.clone());
                }
                wo_rtf::model::RtfBlock::Table { rows } => {
                    for row in rows {
                        current_line.clear();
                        for cell in &row.cells {
                            extract_text_from_inlines(&cell.content, &mut current_line);
                            current_line.push('\t');
                        }
                        lines.push(current_line.clone());
                    }
                }
            }
        }

        let output = lines.join("\n");
        Ok(output.into_bytes())
    }
}

/// Recursively extract plain text from RtfInline nodes.
fn extract_text_from_inlines(inlines: &[wo_rtf::model::RtfInline], out: &mut String) {
    for inline in inlines {
        match inline {
            wo_rtf::model::RtfInline::Text { text } => out.push_str(text),
            wo_rtf::model::RtfInline::Bold { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::Italic { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::Underline { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::Strikethrough { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::Superscript { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::Subscript { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::Font { content, .. } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::FontSize { content, .. } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::Color { content, .. } => {
                extract_text_from_inlines(content, out);
            }
            wo_rtf::model::RtfInline::LineBreak => out.push('\n'),
            wo_rtf::model::RtfInline::Tab => out.push('\t'),
            wo_rtf::model::RtfInline::PageBreak => out.push_str("\n--- page break ---\n"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_rtf_to_txt() {
        let converter = RtfToTxtConverter::new();
        let rtf = br#"{\rtf1\ansi\f0\fs24 Hello World\par}"#;
        let result = converter.convert(rtf).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_convert_rtf_with_bold_to_txt() {
        let converter = RtfToTxtConverter::new();
        let rtf = br#"{\rtf1\ansi\f0\fs24 \b Bold\b0  text\par}"#;
        let result = converter.convert(rtf).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("Bold"));
        assert!(text.contains("text"));
    }

    #[test]
    fn test_convert_rtf_multiple_paragraphs() {
        let converter = RtfToTxtConverter::new();
        let rtf = br#"{\rtf1\ansi\f0\fs24 First\par Second\par Third\par}"#;
        let result = converter.convert(rtf).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("First"));
        assert!(text.contains("Second"));
        assert!(text.contains("Third"));
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines.len() >= 3);
    }

    #[test]
    fn test_convert_invalid_rtf_fails() {
        let converter = RtfToTxtConverter::new();
        let result = converter.convert(b"not rtf at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_format_identifiers() {
        let converter = RtfToTxtConverter::new();
        assert_eq!(converter.source_format(), "rtf");
        assert_eq!(converter.target_format(), "txt");
    }
}
```

- [ ] **Step 2: Create the converters module directory and mod.rs**

Create file `core/crates/wo-x2t/src/converters/mod.rs`:

```rust
//! Individual format-to-format converters.

pub mod rtf_to_txt;
```

Then add to `core/crates/wo-x2t/src/lib.rs` — add this line after `pub mod converter;`:

```rust
pub mod converters;
```

- [ ] **Step 3: Run the test to verify it passes**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t -- converters::rtf_to_txt::tests"`
Expected: All 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add core/crates/wo-x2t/src/converters/ core/crates/wo-x2t/src/lib.rs
git commit -m "feat(wo-x2t): add RTF to TXT converter"
```

---

### Task 6: RTF -> HTML converter

**Files:**
- Create: `core/crates/wo-x2t/src/converters/rtf_to_html.rs`

- [ ] **Step 1: Write the converter with tests**

```rust
//! RTF to HTML converter.

use crate::converter::FormatConverter;
use crate::error::ConversionError;

/// Converts RTF documents to HTML.
pub struct RtfToHtmlConverter;

impl RtfToHtmlConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for RtfToHtmlConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatConverter for RtfToHtmlConverter {
    fn source_format(&self) -> &str {
        "rtf"
    }

    fn target_format(&self) -> &str {
        "html"
    }

    fn convert(&self, data: &[u8]) -> Result<Vec<u8>, ConversionError> {
        let parser = wo_rtf::RtfParser::new();
        let doc = parser.parse(data).map_err(|e| ConversionError::Parse(e.to_string()))?;

        let title = doc
            .info
            .as_ref()
            .and_then(|i| i.title.clone())
            .unwrap_or_else(|| "Converted Document".to_string());

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str(&format!("<title>{}</title>\n", escape_html(&title)));
        html.push_str("</head>\n<body>\n");

        for block in &doc.body {
            html.push_str(&serialize_block(block));
        }

        html.push_str("</body>\n</html>");
        Ok(html.into_bytes())
    }
}

fn serialize_block(block: &wo_rtf::model::RtfBlock) -> String {
    match block {
        wo_rtf::model::RtfBlock::Paragraph { content, .. } => {
            let mut out = String::from("<p>");
            out.push_str(&serialize_inlines(content));
            out.push_str("</p>\n");
            out
        }
        wo_rtf::model::RtfBlock::Table { rows } => {
            let mut out = String::from("<table>\n");
            for row in rows {
                out.push_str("<tr>\n");
                for cell in &row.cells {
                    out.push_str("<td>");
                    out.push_str(&serialize_inlines(&cell.content));
                    out.push_str("</td>\n");
                }
                out.push_str("</tr>\n");
            }
            out.push_str("</table>\n");
            out
        }
    }
}

fn serialize_inlines(inlines: &[wo_rtf::model::RtfInline]) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            wo_rtf::model::RtfInline::Text { text } => out.push_str(&escape_html(text)),
            wo_rtf::model::RtfInline::Bold { content } => {
                out.push_str("<strong>");
                out.push_str(&serialize_inlines(content));
                out.push_str("</strong>");
            }
            wo_rtf::model::RtfInline::Italic { content } => {
                out.push_str("<em>");
                out.push_str(&serialize_inlines(content));
                out.push_str("</em>");
            }
            wo_rtf::model::RtfInline::Underline { content } => {
                out.push_str("<u>");
                out.push_str(&serialize_inlines(content));
                out.push_str("</u>");
            }
            wo_rtf::model::RtfInline::Strikethrough { content } => {
                out.push_str("<s>");
                out.push_str(&serialize_inlines(content));
                out.push_str("</s>");
            }
            wo_rtf::model::RtfInline::Superscript { content } => {
                out.push_str("<sup>");
                out.push_str(&serialize_inlines(content));
                out.push_str("</sup>");
            }
            wo_rtf::model::RtfInline::Subscript { content } => {
                out.push_str("<sub>");
                out.push_str(&serialize_inlines(content));
                out.push_str("</sub>");
            }
            wo_rtf::model::RtfInline::Font { content, .. } => {
                out.push_str(&serialize_inlines(content));
            }
            wo_rtf::model::RtfInline::FontSize { content, .. } => {
                out.push_str(&serialize_inlines(content));
            }
            wo_rtf::model::RtfInline::Color { content, .. } => {
                out.push_str(&serialize_inlines(content));
            }
            wo_rtf::model::RtfInline::LineBreak => out.push_str("<br/>"),
            wo_rtf::model::RtfInline::Tab => out.push_str("\t"),
            wo_rtf::model::RtfInline::PageBreak => out.push_str("<hr/>"),
        }
    }
    out
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_rtf_to_html() {
        let converter = RtfToHtmlConverter::new();
        let rtf = br#"{\rtf1\ansi\f0\fs24 Hello World\par}"#;
        let result = converter.convert(rtf).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html>"));
        assert!(html.contains("Hello World"));
        assert!(html.contains("<p>"));
        assert!(html.contains("</p>"));
    }

    #[test]
    fn test_convert_rtf_bold_to_html() {
        let converter = RtfToHtmlConverter::new();
        let rtf = br#"{\rtf1\ansi\f0\fs24 \b Bold\b0  text\par}"#;
        let result = converter.convert(rtf).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<strong>Bold</strong>"));
        assert!(html.contains("text"));
    }

    #[test]
    fn test_convert_rtf_italic_to_html() {
        let converter = RtfToHtmlConverter::new();
        let rtf = br#"{\rtf1\ansi\f0\fs24 \i Italic\i0  text\par}"#;
        let result = converter.convert(rtf).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<em>Italic</em>"));
    }

    #[test]
    fn test_convert_rtf_with_title() {
        let converter = RtfToHtmlConverter::new();
        let rtf = br#"{\rtf1\ansi{\info{\title My Title}}\f0\fs24 Content\par}"#;
        let result = converter.convert(rtf).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<title>My Title</title>"));
        assert!(html.contains("Content"));
    }

    #[test]
    fn test_convert_invalid_rtf_fails() {
        let converter = RtfToHtmlConverter::new();
        let result = converter.convert(b"not rtf");
        assert!(result.is_err());
    }

    #[test]
    fn test_html_escaping() {
        let converter = RtfToHtmlConverter::new();
        let rtf = br#"{\rtf1\ansi\f0\fs24 A < B > C & D\par}"#;
        let result = converter.convert(rtf).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("&lt;"));
        assert!(html.contains("&gt;"));
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn test_format_identifiers() {
        let converter = RtfToHtmlConverter::new();
        assert_eq!(converter.source_format(), "rtf");
        assert_eq!(converter.target_format(), "html");
    }
}
```

- [ ] **Step 2: Register the module in converters/mod.rs**

Add to `core/crates/wo-x2t/src/converters/mod.rs`:

```rust
//! Individual format-to-format converters.

pub mod rtf_to_html;
pub mod rtf_to_txt;
```

- [ ] **Step 3: Run the test to verify it passes**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t -- converters::rtf_to_html::tests"`
Expected: All 7 tests pass.

- [ ] **Step 4: Commit**

```bash
git add core/crates/wo-x2t/src/converters/
git commit -m "feat(wo-x2t): add RTF to HTML converter"
```

---

### Task 7: HTML -> TXT converter

**Files:**
- Create: `core/crates/wo-x2t/src/converters/html_to_txt.rs`

- [ ] **Step 1: Write the converter with tests**

```rust
//! HTML to TXT converter.

use crate::converter::FormatConverter;
use crate::error::ConversionError;

/// Converts HTML documents to plain text.
pub struct HtmlToTxtConverter;

impl HtmlToTxtConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HtmlToTxtConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatConverter for HtmlToTxtConverter {
    fn source_format(&self) -> &str {
        "html"
    }

    fn target_format(&self) -> &str {
        "txt"
    }

    fn convert(&self, data: &[u8]) -> Result<Vec<u8>, ConversionError> {
        let parser = wo_html::HtmlParser::new();
        let doc = parser.parse(data).map_err(|e| ConversionError::Parse(e.to_string()))?;

        let mut lines: Vec<String> = Vec::new();

        for element in &doc.body.elements {
            extract_text_from_block(element, &mut lines);
        }

        let output = lines.join("\n");
        Ok(output.into_bytes())
    }
}

fn extract_text_from_block(element: &wo_html::model::BlockElement, lines: &mut Vec<String>) {
    match element {
        wo_html::model::BlockElement::Heading { content, .. } => {
            let mut text = String::new();
            extract_text_from_inlines(content, &mut text);
            // Add a blank line before headings for readability
            lines.push(String::new());
            lines.push(text);
        }
        wo_html::model::BlockElement::Paragraph { content, .. } => {
            let mut text = String::new();
            extract_text_from_inlines(content, &mut text);
            if !text.is_empty() {
                lines.push(text);
            }
        }
        wo_html::model::BlockElement::Div { elements, .. } => {
            for child in elements {
                extract_text_from_block(child, lines);
            }
        }
        wo_html::model::BlockElement::UnorderedList { items, .. } => {
            for (i, item) in items.iter().enumerate() {
                let mut text = String::from("  - ");
                extract_text_from_inlines(&item.content, &mut text);
                if i == 0 {
                    lines.push(String::new());
                }
                lines.push(text);
            }
        }
        wo_html::model::BlockElement::OrderedList { items, start, .. } => {
            for (i, item) in items.iter().enumerate() {
                let num = start.unwrap_or(1) + i as u32;
                let mut text = format!("  {}. ", num);
                extract_text_from_inlines(&item.content, &mut text);
                if i == 0 {
                    lines.push(String::new());
                }
                lines.push(text);
            }
        }
        wo_html::model::BlockElement::Table { rows, .. } => {
            lines.push(String::new());
            for row in rows {
                let mut text = String::new();
                for cell in &row.cells {
                    extract_text_from_inlines(&cell.content, &mut text);
                    text.push('\t');
                }
                lines.push(text);
            }
            lines.push(String::new());
        }
        wo_html::model::BlockElement::Blockquote { elements, .. } => {
            lines.push(String::new());
            for child in elements {
                let mut text = String::from("  > ");
                // For paragraphs, extract text and indent
                match child {
                    wo_html::model::BlockElement::Paragraph { content, .. } => {
                        extract_text_from_inlines(content, &mut text);
                        lines.push(text);
                    }
                    _ => extract_text_from_block(child, lines),
                }
            }
        }
        wo_html::model::BlockElement::Pre { content, .. } => {
            lines.push(String::new());
            for line in content.lines() {
                lines.push(format!("  {}", line));
            }
        }
        wo_html::model::BlockElement::HorizontalRule => {
            lines.push(String::new());
            lines.push("---".to_string());
            lines.push(String::new());
        }
        wo_html::model::BlockElement::RawHtml { content, .. } => {
            if !content.is_empty() {
                lines.push(content.clone());
            }
        }
    }
}

fn extract_text_from_inlines(elements: &[wo_html::model::InlineElement], out: &mut String) {
    for element in elements {
        match element {
            wo_html::model::InlineElement::Text { text } => out.push_str(text),
            wo_html::model::InlineElement::Bold { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_html::model::InlineElement::Italic { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_html::model::InlineElement::Underline { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_html::model::InlineElement::Strikethrough { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_html::model::InlineElement::Subscript { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_html::model::InlineElement::Superscript { content } => {
                extract_text_from_inlines(content, out);
            }
            wo_html::model::InlineElement::Code { content } => out.push_str(content),
            wo_html::model::InlineElement::Link { content, .. } => {
                extract_text_from_inlines(content, out);
            }
            wo_html::model::InlineElement::Image { alt, .. } => {
                if let Some(alt_text) = alt {
                    out.push_str(&format!("[{}]", alt_text));
                }
            }
            wo_html::model::InlineElement::LineBreak => out.push('\n'),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_html_to_txt() {
        let converter = HtmlToTxtConverter::new();
        let html = br#"<?xml version="1.0"?>
<html><head><title>Test</title></head>
<body><p>Hello World</p></body></html>"#;
        let result = converter.convert(html).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_convert_html_with_heading() {
        let converter = HtmlToTxtConverter::new();
        let html = br#"<?xml version="1.0"?>
<html><head></head><body>
<h1>Title</h1><p>Content</p></body></html>"#;
        let result = converter.convert(html).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("Title"));
        assert!(text.contains("Content"));
    }

    #[test]
    fn test_convert_html_bold_italic() {
        let converter = HtmlToTxtConverter::new();
        let html = br#"<?xml version="1.0"?>
<html><head></head><body>
<p><strong>Bold</strong> and <em>italic</em> text</p></body></html>"#;
        let result = converter.convert(html).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("Bold"));
        assert!(text.contains("italic"));
    }

    #[test]
    fn test_convert_html_list() {
        let converter = HtmlToTxtConverter::new();
        let html = br#"<?xml version="1.0"?>
<html><head></head><body>
<ul><li>Item 1</li><li>Item 2</li></ul></body></html>"#;
        let result = converter.convert(html).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("Item 1"));
        assert!(text.contains("Item 2"));
        assert!(text.contains("-"));
    }

    #[test]
    fn test_convert_html_image_alt() {
        let converter = HtmlToTxtConverter::new();
        let html = br#"<?xml version="1.0"?>
<html><head></head><body>
<p><img src="photo.jpg" alt="A photo"/></p></body></html>"#;
        let result = converter.convert(html).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("[A photo]"));
    }

    #[test]
    fn test_format_identifiers() {
        let converter = HtmlToTxtConverter::new();
        assert_eq!(converter.source_format(), "html");
        assert_eq!(converter.target_format(), "txt");
    }
}
```

- [ ] **Step 2: Register the module in converters/mod.rs**

Add to `core/crates/wo-x2t/src/converters/mod.rs`:

```rust
//! Individual format-to-format converters.

pub mod html_to_txt;
pub mod rtf_to_html;
pub mod rtf_to_txt;
```

- [ ] **Step 3: Run the test to verify it passes**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t -- converters::html_to_txt::tests"`
Expected: All 6 tests pass.

- [ ] **Step 4: Commit**

```bash
git add core/crates/wo-x2t/src/converters/
git commit -m "feat(wo-x2t): add HTML to TXT converter"
```

---

### Task 8: TXT -> HTML converter

**Files:**
- Create: `core/crates/wo-x2t/src/converters/txt_to_html.rs`

- [ ] **Step 1: Write the converter with tests**

```rust
//! TXT to HTML converter.

use crate::converter::FormatConverter;
use crate::error::ConversionError;

/// Converts plain text documents to HTML.
pub struct TxtToHtmlConverter;

impl TxtToHtmlConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TxtToHtmlConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl FormatConverter for TxtToHtmlConverter {
    fn source_format(&self) -> &str {
        "txt"
    }

    fn target_format(&self) -> &str {
        "html"
    }

    fn convert(&self, data: &[u8]) -> Result<Vec<u8>, ConversionError> {
        let parser = wo_txt::TxtParser::new();
        let doc = parser.parse(data).map_err(|e| ConversionError::Parse(e.to_string()))?;

        let mut html = String::new();
        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Text Document</title>\n");
        html.push_str("</head>\n<body>\n");

        for line in &doc.lines {
            html.push_str("<p>");
            html.push_str(&escape_html(line));
            html.push_str("</p>\n");
        }

        html.push_str("</body>\n</html>");
        Ok(html.into_bytes())
    }
}

fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_txt_to_html() {
        let converter = TxtToHtmlConverter::new();
        let txt = b"Hello World\nLine 2";
        let result = converter.convert(txt).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html>"));
        assert!(html.contains("<p>Hello World</p>"));
        assert!(html.contains("<p>Line 2</p>"));
    }

    #[test]
    fn test_convert_empty_txt() {
        let converter = TxtToHtmlConverter::new();
        let txt = b"";
        let result = converter.convert(txt).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<html>"));
        assert!(html.contains("</html>"));
        assert!(!html.contains("<p>"));
    }

    #[test]
    fn test_convert_txt_with_html_chars() {
        let converter = TxtToHtmlConverter::new();
        let txt = b"A < B > C & D";
        let result = converter.convert(txt).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("&lt;"));
        assert!(html.contains("&gt;"));
        assert!(html.contains("&amp;"));
    }

    #[test]
    fn test_convert_single_line() {
        let converter = TxtToHtmlConverter::new();
        let txt = b"just one line";
        let result = converter.convert(txt).expect("conversion should succeed");
        let html = String::from_utf8(result).unwrap();
        assert!(html.contains("<p>just one line</p>"));
        // Should have exactly one <p> tag
        assert_eq!(html.matches("<p>").count(), 1);
    }

    #[test]
    fn test_format_identifiers() {
        let converter = TxtToHtmlConverter::new();
        assert_eq!(converter.source_format(), "txt");
        assert_eq!(converter.target_format(), "html");
    }
}
```

- [ ] **Step 2: Register the module in converters/mod.rs**

Replace `core/crates/wo-x2t/src/converters/mod.rs` with:

```rust
//! Individual format-to-format converters.

pub mod html_to_txt;
pub mod rtf_to_html;
pub mod rtf_to_txt;
pub mod txt_to_html;
```

- [ ] **Step 3: Run the test to verify it passes**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t -- converters::txt_to_html::tests"`
Expected: All 5 tests pass.

- [ ] **Step 4: Commit**

```bash
git add core/crates/wo-x2t/src/converters/
git commit -m "feat(wo-x2t): add TXT to HTML converter"
```

---

### Task 9: ConverterRegistry

**Files:**
- Create: `core/crates/wo-x2t/src/converters/registry.rs`

- [ ] **Step 1: Write the ConverterRegistry with tests**

```rust
//! Converter registry that maps (source, target) format pairs to converter instances.

use std::collections::HashMap;
use std::sync::Arc;

use crate::converter::FormatConverter;
use crate::error::ConversionError;

/// Registry of format converters, indexed by (source, target) format pair.
pub struct ConverterRegistry {
    converters: HashMap<(String, String), Arc<dyn FormatConverter>>,
}

impl ConverterRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            converters: HashMap::new(),
        }
    }

    /// Register a converter.
    pub fn register<C: FormatConverter + 'static>(&mut self, converter: C) {
        let key = (converter.source_format().to_string(), converter.target_format().to_string());
        self.converters.insert(key, Arc::new(converter));
    }

    /// Look up a converter for the given (source, target) pair.
    pub fn get(&self, source: &str, target: &str) -> Option<&Arc<dyn FormatConverter>> {
        self.converters.get(&(source.to_string(), target.to_string()))
    }

    /// Check if a converter exists for the given pair.
    pub fn has_converter(&self, source: &str, target: &str) -> bool {
        self.converters.contains_key(&(source.to_string(), target.to_string()))
    }

    /// Convert data using the registered converter for the given pair.
    pub fn convert(&self, source: &str, target: &str, data: &[u8]) -> Result<Vec<u8>, ConversionError> {
        match self.get(source, target) {
            Some(converter) => converter.convert(data),
            None => Err(ConversionError::NoConverter {
                source: source.to_string(),
                target: target.to_string(),
            }),
        }
    }

    /// Create a registry with all built-in converters registered.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(super::rtf_to_txt::RtfToTxtConverter::new());
        registry.register(super::rtf_to_html::RtfToHtmlConverter::new());
        registry.register(super::html_to_txt::HtmlToTxtConverter::new());
        registry.register(super::txt_to_html::TxtToHtmlConverter::new());
        registry
    }

    /// List all registered conversion pairs.
    pub fn registered_pairs(&self) -> Vec<(&str, &str)> {
        self.converters
            .keys()
            .map(|(s, t)| (s.as_str(), t.as_str()))
            .collect()
    }
}

impl Default for ConverterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = ConverterRegistry::new();
        assert!(!registry.has_converter("rtf", "txt"));
        assert!(registry.get("rtf", "txt").is_none());
    }

    #[test]
    fn test_register_and_lookup() {
        let mut registry = ConverterRegistry::new();
        registry.register(super::rtf_to_txt::RtfToTxtConverter::new());
        assert!(registry.has_converter("rtf", "txt"));
        assert!(registry.get("rtf", "txt").is_some());
        assert!(!registry.has_converter("txt", "rtf"));
    }

    #[test]
    fn test_convert_via_registry() {
        let mut registry = ConverterRegistry::new();
        registry.register(super::rtf_to_txt::RtfToTxtConverter::new());
        let rtf = br#"{\rtf1\ansi\f0\fs24 Hello\par}"#;
        let result = registry.convert("rtf", "txt", rtf).expect("conversion should succeed");
        let text = String::from_utf8(result).unwrap();
        assert!(text.contains("Hello"));
    }

    #[test]
    fn test_convert_missing_pair() {
        let registry = ConverterRegistry::new();
        let result = registry.convert("pdf", "docx", b"test");
        assert!(result.is_err());
        match result.unwrap_err() {
            ConversionError::NoConverter { source, target } => {
                assert_eq!(source, "pdf");
                assert_eq!(target, "docx");
            }
            other => panic!("expected NoConverter error, got: {}", other),
        }
    }

    #[test]
    fn test_with_defaults() {
        let registry = ConverterRegistry::with_defaults();
        assert!(registry.has_converter("rtf", "txt"));
        assert!(registry.has_converter("rtf", "html"));
        assert!(registry.has_converter("html", "txt"));
        assert!(registry.has_converter("txt", "html"));
    }

    #[test]
    fn test_registered_pairs() {
        let registry = ConverterRegistry::with_defaults();
        let pairs = registry.registered_pairs();
        assert!(pairs.len() >= 4);
        assert!(pairs.contains(&("rtf", "txt")));
        assert!(pairs.contains(&("rtf", "html")));
        assert!(pairs.contains(&("html", "txt")));
        assert!(pairs.contains(&("txt", "html")));
    }

    #[test]
    fn test_case_sensitive_lookup() {
        let mut registry = ConverterRegistry::new();
        registry.register(super::rtf_to_txt::RtfToTxtConverter::new());
        // Uppercase should NOT match
        assert!(!registry.has_converter("RTF", "TXT"));
    }
}
```

- [ ] **Step 2: Register the module in converters/mod.rs**

Replace `core/crates/wo-x2t/src/converters/mod.rs` with:

```rust
//! Individual format-to-format converters.

pub mod html_to_txt;
pub mod rtf_to_html;
pub mod registry;
pub mod rtf_to_txt;
pub mod txt_to_html;
```

- [ ] **Step 3: Run the test to verify it passes**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t -- converters::registry::tests"`
Expected: All 7 tests pass.

- [ ] **Step 4: Commit**

```bash
git add core/crates/wo-x2t/src/converters/
git commit -m "feat(wo-x2t): add ConverterRegistry with default converters"
```

---

### Task 10: Update router.rs to wire converter dispatch into ConversionRouter

**Files:**
- Modify: `core/crates/wo-x2t/src/router.rs`

- [ ] **Step 1: Add conversion method to ConversionRouter**

The existing router tests must continue to pass. We add a `convert` method that delegates to the registry. Add the following import at the top of router.rs (after the existing `use crate::model::{ConversionResult, ConversionStatus};`):

```rust
use crate::converters::registry::ConverterRegistry;
use std::time::Instant;
```

Then add the `convert` method to the `impl ConversionRouter` block, right before the `fn get_pair` method (i.e., after the `unsupported_result` method closing brace and before `fn get_pair`):

```rust
    /// Perform a format conversion using the converter registry.
    ///
    /// Uses default converters. Returns a ConversionResult.
    pub fn convert(&self, input: &crate::model::ConversionInput) -> ConversionResult {
        let source = &input.source_format;
        let target = &input.target_format;

        // Same format = return input as-is
        if source == target {
            return crate::success_result(input.data.clone(), target, 0);
        }

        let registry = ConverterRegistry::with_defaults();

        if !registry.has_converter(source, target) {
            return Self::unsupported_result(source, target);
        }

        let start = Instant::now();
        match registry.convert(source, target, &input.data) {
            Ok(output_data) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                crate::success_result(output_data, target, duration_ms)
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                crate::error_result(e.to_string(), duration_ms)
            }
        }
    }
```

- [ ] **Step 2: Add a test for the convert method**

Add the following tests to the existing `#[cfg(test)] mod tests` block in router.rs (before the closing `}`):

```rust
    #[test]
    fn test_convert_rtf_to_txt() {
        let router = ConversionRouter::new();
        let input = crate::model::ConversionInput {
            source_format: "rtf".to_string(),
            target_format: "txt".to_string(),
            data: br#"{\rtf1\ansi\f0\fs24 Hello\par}"#.to_vec(),
            options: Default::default(),
        };
        let result = router.convert(&input);
        assert_eq!(result.status, ConversionStatus::Success);
        let output = result.output.unwrap();
        let text = String::from_utf8(output.data).unwrap();
        assert!(text.contains("Hello"));
    }

    #[test]
    fn test_convert_same_format() {
        let router = ConversionRouter::new();
        let input = crate::model::ConversionInput {
            source_format: "txt".to_string(),
            target_format: "txt".to_string(),
            data: b"hello".to_vec(),
            options: Default::default(),
        };
        let result = router.convert(&input);
        assert_eq!(result.status, ConversionStatus::Success);
        assert_eq!(result.output.unwrap().data, b"hello".to_vec());
    }

    #[test]
    fn test_convert_unsupported_pair() {
        let router = ConversionRouter::new();
        let input = crate::model::ConversionInput {
            source_format: "pdf".to_string(),
            target_format: "docx".to_string(),
            data: b"%PDF-1.4".to_vec(),
            options: Default::default(),
        };
        let result = router.convert(&input);
        assert_eq!(result.status, ConversionStatus::UnsupportedFormat);
    }

    #[test]
    fn test_convert_invalid_source_data() {
        let router = ConversionRouter::new();
        let input = crate::model::ConversionInput {
            source_format: "rtf".to_string(),
            target_format: "txt".to_string(),
            data: b"not rtf at all".to_vec(),
            options: Default::default(),
        };
        let result = router.convert(&input);
        assert_eq!(result.status, ConversionStatus::Failed);
        assert!(result.error.is_some());
    }
```

- [ ] **Step 3: Run all router tests to verify**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t -- router::tests"`
Expected: All existing tests pass (8 original + 4 new = 12 total).

- [ ] **Step 4: Commit**

```bash
git add core/crates/wo-x2t/src/router.rs
git commit -m "feat(wo-x2t): wire converter dispatch into ConversionRouter"
```

---

### Task 11: Update roundtrip.rs to use real converters

**Files:**
- Modify: `core/crates/wo-x2t/src/roundtrip.rs`

- [ ] **Step 1: Update the parse method in FormatRoundtrip impl**

The current `parse` method is an identity stub. Replace the entire `impl FormatRoundtrip for X2tRoundtrip` block with a version that uses the `ConverterRegistry`:

Replace the existing `impl FormatRoundtrip for X2tRoundtrip { ... }` block (lines 98-157) with:

```rust
impl FormatRoundtrip for X2tRoundtrip {
    /// Parse raw bytes and run the conversion.
    fn parse(&self, data: &[u8]) -> Result<(), String> {
        // Store the input data
        let input = self.input.as_ref().ok_or_else(|| {
            "No conversion context set. Use with_formats() or set_source_format()/set_target_format().".to_string()
        })?;

        let registry = crate::converters::registry::ConverterRegistry::with_defaults();

        let result = if self
            .router
            .is_supported(&input.source_format, &input.target_format)
        {
            match registry.convert(&input.source_format, &input.target_format, data) {
                Ok(output_data) => ConversionResult {
                    status: ConversionStatus::Success,
                    output: Some(crate::model::ConversionOutput {
                        data: output_data,
                        format: input.target_format.clone(),
                        page_count: None,
                        warnings: Vec::new(),
                    }),
                    error: None,
                    duration_ms: 0,
                },
                Err(e) => ConversionResult {
                    status: ConversionStatus::Failed,
                    output: None,
                    error: Some(e.to_string()),
                    duration_ms: 0,
                },
            }
        } else {
            ConversionRouter::unsupported_result(&input.source_format, &input.target_format)
        };

        // Store the result using interior mutability
        *self.result.borrow_mut() = Some(result);

        Ok(())
    }

    /// Serialize the conversion result back to bytes.
    fn serialize(&self) -> Result<Vec<u8>, String> {
        let borrowed = self.result.borrow();
        let result = borrowed
            .as_ref()
            .ok_or_else(|| "No conversion result available. Call parse() first.".to_string())?;

        match &result.output {
            Some(output) => Ok(output.data.clone()),
            None => Err(format!(
                "Conversion failed with status: {:?}. Error: {:?}",
                result.status, result.error
            )),
        }
    }
}
```

- [ ] **Step 2: Update existing roundtrip tests to reflect real conversion behavior**

The existing `test_roundtrip_with_identity_conversion` and `test_roundtrip_preserves_data` tests expect identity behavior (same bytes in = same bytes out). Since we now do real conversion, update these tests:

Replace the existing `#[cfg(test)] mod tests` block entirely with:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_requires_formats() {
        let roundtrip = X2tRoundtrip::new();
        let input_data = b"test data";

        // Parse should fail without format context
        let result = roundtrip.parse(input_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No conversion context set"));
    }

    #[test]
    fn test_roundtrip_unsupported_conversion() {
        let roundtrip = X2tRoundtrip::with_formats("pdf", "docx");
        let input_data = b"test pdf content";

        // Parse succeeds (creates error result)
        roundtrip.parse(input_data).expect("Parse should succeed");

        // Serialize should fail for unsupported format
        let result = roundtrip.serialize();
        assert!(result.is_err());
    }

    #[test]
    fn test_roundtrip_same_format_identity() {
        let roundtrip = X2tRoundtrip::with_formats("docx", "docx");
        let input_data = b"test document content";

        roundtrip.parse(input_data).expect("Parse should succeed");
        let output_data = roundtrip.serialize().expect("Serialize should succeed");
        assert_eq!(output_data, input_data);
    }

    #[test]
    fn test_roundtrip_rtf_to_txt() {
        let roundtrip = X2tRoundtrip::with_formats("rtf", "txt");
        let input_data = br#"{\rtf1\ansi\f0\fs24 Hello World\par}"#;

        roundtrip.parse(input_data).expect("Parse should succeed");
        let output_data = roundtrip.serialize().expect("Serialize should succeed");
        let text = String::from_utf8(output_data).unwrap();
        assert!(text.contains("Hello World"));
    }

    #[test]
    fn test_roundtrip_rtf_to_html() {
        let roundtrip = X2tRoundtrip::with_formats("rtf", "html");
        let input_data = br#"{\rtf1\ansi\f0\fs24 Hello World\par}"#;

        roundtrip.parse(input_data).expect("Parse should succeed");
        let output_data = roundtrip.serialize().expect("Serialize should succeed");
        let html = String::from_utf8(output_data).unwrap();
        assert!(html.contains("<html>"));
        assert!(html.contains("Hello World"));
    }

    #[test]
    fn test_roundtrip_invalid_source_data() {
        let roundtrip = X2tRoundtrip::with_formats("rtf", "txt");
        let input_data = b"not rtf at all";

        // Parse succeeds (creates failed result)
        roundtrip.parse(input_data).expect("Parse should succeed");

        // Serialize should fail
        let result = roundtrip.serialize();
        assert!(result.is_err());
    }

    #[test]
    fn test_new_with_default() {
        let roundtrip = X2tRoundtrip::new();
        assert!(roundtrip.input().is_none());
        assert!(roundtrip.result().is_none());
        assert_eq!(roundtrip.router().is_supported("docx", "pdf"), true);
    }

    #[test]
    fn test_with_formats_creates_context() {
        let roundtrip = X2tRoundtrip::with_formats("xlsx", "pdf");
        let input = roundtrip.input().expect("Input should be set");
        assert_eq!(input.source_format, "xlsx");
        assert_eq!(input.target_format, "pdf");
    }

    #[test]
    fn test_set_source_and_target() {
        let roundtrip = X2tRoundtrip::new()
            .set_source_format("pptx")
            .set_target_format("pdf");

        let input = roundtrip.input().expect("Input should be set");
        assert_eq!(input.source_format, "pptx");
        assert_eq!(input.target_format, "pdf");
    }
}
```

- [ ] **Step 3: Run all roundtrip tests**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t -- roundtrip::tests"`
Expected: All 9 tests pass.

- [ ] **Step 4: Commit**

```bash
git add core/crates/wo-x2t/src/roundtrip.rs
git commit -m "feat(wo-x2t): replace identity stub with real converter dispatch in roundtrip"
```

---

### Task 12: Create RTF test fixture

**Files:**
- Create: `core/crates/wo-x2t/tests/fixtures/minimal.rtf`

- [ ] **Step 1: Create the fixtures directory and RTF file**

Create directory `core/crates/wo-x2t/tests/fixtures/` (if it doesn't exist), then create file `core/crates/wo-x2t/tests/fixtures/minimal.rtf`:

```rtf
{\rtf1\ansi{\fonttbl{\f0\fswiss Arial;}}
{\info{\title Test Document}{\author World Office}}
\f0\fs24
\b Hello\par
\b0 This is a test document with \i italic\i0{} and \ul underline\ulnone{} text.\par
\par
Second paragraph with plain text.\par
}
```

- [ ] **Step 2: Commit**

```bash
git add core/crates/wo-x2t/tests/fixtures/minimal.rtf
git commit -m "test(wo-x2t): add minimal RTF test fixture"
```

---

### Task 13: Create HTML test fixture

**Files:**
- Create: `core/crates/wo-x2t/tests/fixtures/minimal.html`

- [ ] **Step 1: Create the HTML test fixture**

Create file `core/crates/wo-x2t/tests/fixtures/minimal.html`:

```html
<!DOCTYPE html>
<html>
<head>
    <title>Test Document</title>
    <meta charset="utf-8"/>
</head>
<body>
    <h1>Chapter One</h1>
    <p>This is a <strong>test</strong> document with <em>formatting</em>.</p>
    <h2>Section 1.1</h2>
    <p>Second paragraph with <u>underlined</u> text and a <a href="https://example.com">link</a>.</p>
    <ul>
        <li>Item one</li>
        <li>Item two</li>
    </ul>
</body>
</html>
```

- [ ] **Step 2: Commit**

```bash
git add core/crates/wo-x2t/tests/fixtures/minimal.html
git commit -m "test(wo-x2t): add minimal HTML test fixture"
```

---

### Task 14: Create TXT test fixture

**Files:**
- Create: `core/crates/wo-x2t/tests/fixtures/minimal.txt`

- [ ] **Step 1: Create the TXT test fixture**

Create file `core/crates/wo-x2t/tests/fixtures/minimal.txt`:

```
Hello World
This is a plain text document.
It has multiple lines.

Fourth line after a blank line.
```

- [ ] **Step 2: Commit**

```bash
git add core/crates/wo-x2t/tests/fixtures/minimal.txt
git commit -m "test(wo-x2t): add minimal TXT test fixture"
```

---

### Task 15: Integration tests — end-to-end conversion with sample files

**Files:**
- Create: `core/crates/wo-x2t/tests/integration_test.rs`

- [ ] **Step 1: Write the integration test file**

```rust
//! End-to-end integration tests for wo-x2t format conversion.

use std::path::PathBuf;
use wo_x2t::ConversionInput;
use wo_x2t::ConversionRouter;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

fn read_fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name))
        .unwrap_or_else(|e| panic!("Failed to read fixture '{}': {}", name, e))
}

#[test]
fn test_rtf_to_txt_with_fixture() {
    let router = ConversionRouter::new();
    let input = ConversionInput {
        source_format: "rtf".to_string(),
        target_format: "txt".to_string(),
        data: read_fixture("minimal.rtf"),
        options: Default::default(),
    };
    let result = router.convert(&input);
    assert_eq!(result.status, wo_x2t::ConversionStatus::Success);
    let output = result.output.unwrap();
    let text = String::from_utf8(output.data).unwrap();
    assert!(text.contains("Hello"), "Output should contain 'Hello'");
    assert!(text.contains("italic"), "Output should contain 'italic'");
    assert!(text.contains("underline"), "Output should contain 'underline'");
    assert!(text.contains("Second paragraph"), "Output should contain 'Second paragraph'");
}

#[test]
fn test_rtf_to_html_with_fixture() {
    let router = ConversionRouter::new();
    let input = ConversionInput {
        source_format: "rtf".to_string(),
        target_format: "html".to_string(),
        data: read_fixture("minimal.rtf"),
        options: Default::default(),
    };
    let result = router.convert(&input);
    assert_eq!(result.status, wo_x2t::ConversionStatus::Success);
    let output = result.output.unwrap();
    let html = String::from_utf8(output.data).unwrap();
    assert!(html.contains("<!DOCTYPE html>"), "Output should be valid HTML");
    assert!(html.contains("<html>"));
    assert!(html.contains("Hello"));
    assert!(html.contains("<strong>"));
    assert!(html.contains("<em>"));
}

#[test]
fn test_html_to_txt_with_fixture() {
    let router = ConversionRouter::new();
    let input = ConversionInput {
        source_format: "html".to_string(),
        target_format: "txt".to_string(),
        data: read_fixture("minimal.html"),
        options: Default::default(),
    };
    let result = router.convert(&input);
    assert_eq!(result.status, wo_x2t::ConversionStatus::Success);
    let output = result.output.unwrap();
    let text = String::from_utf8(output.data).unwrap();
    assert!(text.contains("Chapter One"), "Output should contain heading text");
    assert!(text.contains("test"), "Output should contain bold text");
    assert!(text.contains("formatting"), "Output should contain italic text");
    assert!(text.contains("underlined"), "Output should contain underline text");
    assert!(text.contains("Item one"), "Output should contain list items");
}

#[test]
fn test_txt_to_html_with_fixture() {
    let router = ConversionRouter::new();
    let input = ConversionInput {
        source_format: "txt".to_string(),
        target_format: "html".to_string(),
        data: read_fixture("minimal.txt"),
        options: Default::default(),
    };
    let result = router.convert(&input);
    assert_eq!(result.status, wo_x2t::ConversionStatus::Success);
    let output = result.output.unwrap();
    let html = String::from_utf8(output.data).unwrap();
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("Hello World"));
    assert!(html.contains("plain text document"));
}

#[test]
fn test_chain_rtf_to_txt_via_roundtrip() {
    use wo_x2t::X2tRoundtrip;

    let data = read_fixture("minimal.rtf");
    let roundtrip = X2tRoundtrip::with_formats("rtf", "txt");
    roundtrip.parse(&data).expect("Parse should succeed");
    let output = roundtrip.serialize().expect("Serialize should succeed");
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("Hello"));
}

#[test]
fn test_chain_html_to_txt_via_roundtrip() {
    use wo_x2t::X2tRoundtrip;

    let data = read_fixture("minimal.html");
    let roundtrip = X2tRoundtrip::with_formats("html", "txt");
    roundtrip.parse(&data).expect("Parse should succeed");
    let output = roundtrip.serialize().expect("Serialize should succeed");
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("Chapter One"));
}

#[test]
fn test_converter_registry_has_all_pairs() {
    use wo_x2t::converters::registry::ConverterRegistry;

    let registry = ConverterRegistry::with_defaults();
    let pairs = registry.registered_pairs();
    assert!(pairs.contains(&("rtf", "txt")));
    assert!(pairs.contains(&("rtf", "html")));
    assert!(pairs.contains(&("html", "txt")));
    assert!(pairs.contains(&("txt", "html")));
}
```

- [ ] **Step 2: Run the integration tests**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t --test integration_test"`
Expected: All 7 tests pass.

- [ ] **Step 3: Commit**

```bash
git add core/crates/wo-x2t/tests/integration_test.rs
git commit -m "test(wo-x2t): add end-to-end integration tests with fixture files"
```

---

### Task 16: Full workspace test run — verify no regressions

- [ ] **Step 1: Run all wo-x2t tests**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-x2t"`
Expected: All tests pass (unit tests + integration tests).

- [ ] **Step 2: Run clippy on wo-x2t**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo clippy -p wo-x2t"`
Expected: No errors (warnings are acceptable).

- [ ] **Step 3: Verify dependent crates still compile**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo check -p wo-x2t-wasm"`
Expected: Compiles (wo-x2t-wasm depends on wo-x2t).

- [ ] **Step 4: Run tests for the parser crates we depend on**

Run: `wsl bash -c "cd /mnt/c/Users/Tobias/git/World-Office && ~/.cargo/bin/cargo test -p wo-rtf -p wo-html -p wo-txt"`
Expected: All tests pass.

- [ ] **Step 5: Final commit if any fixes were needed**

If any issues were found and fixed during the verification steps, commit them:

```bash
git add -A
git commit -m "fix(wo-x2t): address clippy warnings and test fixes"
```

If no issues found, this step is skipped.
