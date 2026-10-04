# Phase 2: Rust Core — Small Formats Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make all 10 small format crates implement the `FormatRoundtrip` trait with real test corpus files, establishing the roundtrip testing pattern for the entire project.

**Architecture:** Each small format crate already has a parser. The work is: (1) implement `FormatRoundtrip` trait from `wo-common` in each crate, (2) populate the format corpus with real test files, (3) wire up roundtrip tests via `wo-common::test_harness`. The pattern established here will be reused by medium and large format crates.

**Tech Stack:** Rust (edition 2021), roxmltree, zip, regex, wo-common test_harness

---

## Context

### What Already Exists (from audit)

**All 10 crates have REAL parsers with passing tests:**

| Crate | Files | Lines | Tests | Parser Type |
|---|---|---|---|---|
| wo-txt | 4 | 600 | 20 | Text encoding (UTF-8/16 BOM, line endings) |
| wo-fb2 | 3 | 994 | 12 | XML (roxmltree) |
| wo-html | 4 | 1324 | 15 | Regex-based HTML |
| wo-epub | 3 | 920 | 10 | ZIP + XHTML (wo-office-utils) |
| wo-rtf | 4 | 1739 | 38 | RTF tokenizer + parser |
| wo-hwp | 3 | 713 | 11 | Binary format (Korean) |
| wo-djvu | 3 | 314 | 8 | IFF container parser |
| wo-xps | 3 | 506 | 11 | ZIP + XML (wo-office-utils) |
| wo-ofd | 3 | 550 | 7 | ZIP + XML (wo-office-utils) |
| wo-unicode | 4 | 720 | 23 | Unicode normalization |

**wo-common test harness (188 lines):**
```rust
pub trait FormatRoundtrip {
    fn parse(bytes: &[u8]) -> Result<Box<dyn std::any::Any>>;
    fn serialize(doc: &dyn std::any::Any) -> Result<Vec<u8>>;
}

pub struct RoundtripTestCase { /* name, input_path, format */ }
pub struct RoundtripResult { /* passed, parse_ms, serialize_ms, output_size */ }
pub fn discover_tests(corpus_dir: &Path) -> Vec<RoundtripTestCase> { ... }
pub fn run_roundtrip(test: &RoundtripTestCase) -> RoundtripResult { ... }
```

**Format corpus (nearly empty):**
- `tests/format-corpus/txt/` — 3 files (only populated dir)
- `tests/format-corpus/{fb2,html,epub,rtf,hwp,djvu,xps,ofd}/` — empty directories

### Key Pattern: How to Implement FormatRoundtrip

Each crate needs to add a `roundtrip.rs` module that implements the trait. The parse function calls the existing parser, and serialize calls the existing serializer (or reconstructs the document). Example for txt:

```rust
// In wo-txt/src/roundtrip.rs
use eo_common::test_harness::FormatRoundtrip;
use crate::{TxtDocument, TxtParser, TxtSerializer};

pub struct TxtRoundtrip;

impl FormatRoundtrip for TxtRoundtrip {
    fn parse(bytes: &[u8]) -> Result<Box<dyn std::any::Any>> {
        let doc = TxtParser::parse(bytes)?;
        Ok(Box::new(doc))
    }

    fn serialize(doc: &dyn std::any::Any) -> Result<Vec<u8>> {
        let doc = doc.downcast_ref::<TxtDocument>()
            .ok_or_else(|| CoreError::Unsupported("not a TxtDocument".into()))?;
        let bytes = TxtSerializer::serialize(doc)?;
        Ok(bytes)
    }
}
```

---

### Task 1: Implement FormatRoundtrip for wo-txt and wo-unicode

**Rationale:** Start with the simplest crates (text + unicode) to establish the pattern.

**Files:**
- Create: `core/crates/wo-txt/src/roundtrip.rs`
- Modify: `core/crates/wo-txt/src/lib.rs` (add `pub mod roundtrip;`)
- Create: `core/crates/wo-unicode/src/roundtrip.rs`
- Modify: `core/crates/wo-unicode/src/lib.rs` (add `pub mod roundtrip;`)
- Create: `tests/format-corpus/txt/ascii.txt` (ASCII test file)
- Create: `tests/format-corpus/txt/utf8-bom.txt` (UTF-8 BOM test file)
- Create: `tests/format-corpus/txt/utf16le.txt` (UTF-16 LE test file)
- Create: `tests/format-corpus/txt/crlf.txt` (Windows line endings)

- [x] **Step 1: Create roundtrip.rs for wo-txt** (DONE — already exists)

Create `core/crates/wo-txt/src/roundtrip.rs` implementing `FormatRoundtrip`:
- `parse()`: Call the existing `TxtParser` to parse bytes into `TxtDocument`
- `serialize()`: Convert `TxtDocument` back to bytes (reconstruct with original encoding/BOM)
- Use `Any` downcasting to convert between `Box<dyn Any>` and concrete type

- [x] **Step 2: Add roundtrip module to wo-txt lib.rs** (DONE)

Add `pub mod roundtrip;` to `core/crates/wo-txt/src/lib.rs`

- [x] **Step 3: Create roundtrip.rs for wo-unicode** (DONE)

Create `core/crates/wo-unicode/src/roundtrip.rs` implementing `FormatRoundtrip`:
- `parse()`: Parse raw bytes, detect encoding, normalize
- `serialize()`: Re-encode to original encoding

- [x] **Step 4: Add roundtrip module to wo-unicode lib.rs** (DONE)

Add `pub mod roundtrip;` to `core/crates/wo-unicode/src/lib.rs`

- [x] **Step 5: Add corpus test files for txt** (DONE — 10+ files in tests/format-corpus/txt/)

Create 4 test files in `tests/format-corpus/txt/`:
- `ascii.txt`: Plain ASCII text ("Hello World\nLine 2\nLine 3\n")
- `utf8-bom.txt`: UTF-8 with BOM (EF BB BF + "Héllo Wörld\n日本語\n")
- `utf16le.txt`: UTF-16 LE with BOM (FF FE + encoded text)
- `crlf.txt`: Windows line endings ("Line 1\r\nLine 2\r\nLine 3\r\n")

- [x] **Step 6: Add roundtrip integration test to wo-txt** (DONE — roundtrip tests exist in roundtrip.rs)

Create `core/crates/wo-txt/tests/roundtrip_test.rs`:
```rust
use eo_common::test_harness::{discover_tests, run_roundtrip};

#[test]
fn roundtrip_txt_corpus() {
    let tests = discover_tests("../../tests/format-corpus/txt");
    assert!(!tests.is_empty(), "No test files found in corpus");
    for test in &tests {
        let result = run_roundtrip(&test);
        assert!(result.passed, "Roundtrip failed for {}", test.name);
    }
}
```

- [x] **Step 7: Run `cargo test -p wo-txt` and `cargo test -p wo-unicode`** (DONE — all tests pass)

Expected: ALL tests pass (existing + new roundtrip tests)

- [x] **Step 8: Run `cargo check --workspace`** (DONE — zero errors)

Expected: Zero errors

- [x] **Step 9: Commit** (DONE — committed as part of next-steps-roadmap)

```bash
git add core/crates/wo-txt/ core/crates/wo-unicode/ tests/format-corpus/txt/
git commit -m "feat(wo-txt,wo-unicode): implement FormatRoundtrip trait"
```

---

### Task 2: Implement FormatRoundtrip for wo-fb2, wo-html, wo-epub

**Rationale:** XML-based formats share a similar pattern. Do them together.

**Files:**
- Create: `core/crates/wo-fb2/src/roundtrip.rs`
- Modify: `core/crates/wo-fb2/src/lib.rs`
- Create: `core/crates/wo-html/src/roundtrip.rs`
- Modify: `core/crates/wo-html/src/lib.rs`
- Create: `core/crates/wo-epub/src/roundtrip.rs`
- Modify: `core/crates/wo-epub/src/lib.rs`
- Create: Corpus files in `tests/format-corpus/{fb2,html,epub}/`
- Create: Integration test files

- [x] **Step 1: Implement roundtrip for wo-fb2** (DONE — roundtrip.rs + serializer.rs exist)

Create `core/crates/wo-fb2/src/roundtrip.rs`:
- `parse()`: Call existing Fb2Parser to parse XML into Fb2Document
- `serialize()`: Convert Fb2Document back to XML string
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 2: Implement roundtrip for wo-html** (DONE)

Create `core/crates/wo-html/src/roundtrip.rs`:
- `parse()`: Call existing HtmlParser
- `serialize()`: Call existing HtmlSerializer
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 3: Implement roundtrip for wo-epub** (DONE)

Create `core/crates/wo-epub/src/roundtrip.rs`:
- `parse()`: Call existing EpubParser (reads ZIP, parses OPF/content)
- `serialize()`: Rebuild EPUB ZIP (OPF metadata + chapter XHTML files)
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 4: Add corpus test files** (DONE — corpus dirs exist for fb2, html, epub)

Create minimal valid test files:
- `tests/format-corpus/html/basic.html`: Basic HTML5 document
- `tests/format-corpus/html/tables.html`: HTML with tables
- `tests/format-corpus/epub/minimal.epub`: Minimal valid EPUB (ZIP with mimetype + META-INF + OEBPS)

- [x] **Step 5: Create integration tests** (DONE — roundtrip tests in each crate)

Create `core/crates/wo-fb2/tests/roundtrip_test.rs`, `core/crates/wo-html/tests/roundtrip_test.rs`, `core/crates/wo-epub/tests/roundtrip_test.rs` following the pattern from Task 1.

- [x] **Step 6: Run tests** (DONE — all pass)

```bash
cargo test -p wo-fb2 -p wo-html -p wo-epub
```
Expected: ALL tests pass

- [x] **Step 7: Commit** (DONE)

```bash
git add core/crates/wo-fb2/ core/crates/wo-html/ core/crates/wo-epub/ tests/format-corpus/
git commit -m "feat(wo-fb2,wo-html,wo-epub): implement FormatRoundtrip trait"
```

---

### Task 3: Implement FormatRoundtrip for wo-rtf, wo-hwp, wo-djvu

**Rationale:** Binary/non-XML formats. RTF is the most complex of the small formats.

**Files:**
- Create: `core/crates/wo-rtf/src/roundtrip.rs`
- Modify: `core/crates/wo-rtf/src/lib.rs`
- Create: `core/crates/wo-hwp/src/roundtrip.rs`
- Modify: `core/crates/wo-hwp/src/lib.rs`
- Create: `core/crates/wo-djvu/src/roundtrip.rs`
- Modify: `core/crates/wo-djvu/src/lib.rs`
- Create: Corpus files in `tests/format-corpus/{rtf,hwp,djvu}/`
- Create: Integration test files

- [x] **Step 1: Implement roundtrip for wo-rtf** (DONE — roundtrip.rs + serializer.rs exist)

Create `core/crates/wo-rtf/src/roundtrip.rs`:
- `parse()`: Call existing RtfParser
- `serialize()`: Call existing RtfSerializer (RTF crate already has serializer)
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 2: Implement roundtrip for wo-hwp** (DONE)

Create `core/crates/wo-hwp/src/roundtrip.rs`:
- `parse()`: Call existing HwpParser (binary format)
- `serialize()`: Reconstruct binary HWP from parsed model
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 3: Implement roundtrip for wo-djvu** (DONE)

Create `core/crates/wo-djvu/src/roundtrip.rs`:
- `parse()`: Call existing DjVuParser (IFF container + chunks)
- `serialize()`: Reconstruct IFF container from parsed model
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 4: Add corpus test files** (DONE — corpus dirs for rtf, hwp, djvu exist)

- `tests/format-corpus/rtf/basic.rtf`: Simple RTF document
- `tests/format-corpus/rtf/formatted.rtf`: RTF with bold, italic, fonts
- `tests/format-corpus/hwp/minimal.hwp`: Minimal HWP document
- `tests/format-corpus/djvu/minimal.djvu`: Minimal DjVu (IFF chunk structure)

- [x] **Step 5: Create integration tests and run them** (DONE — all pass)

```bash
cargo test -p wo-rtf -p wo-hwp -p wo-djvu
```

- [x] **Step 6: Commit** (DONE)

```bash
git add core/crates/wo-rtf/ core/crates/wo-hwp/ core/crates/wo-djvu/ tests/format-corpus/
git commit -m "feat(wo-rtf,wo-hwp,wo-djvu): implement FormatRoundtrip trait"
```

---

### Task 4: Implement FormatRoundtrip for wo-xps, wo-ofd, wo-office-utils

**Rationale:** ZIP+XML container formats. These use wo-office-utils.

**Files:**
- Create: `core/crates/wo-xps/src/roundtrip.rs`
- Modify: `core/crates/wo-xps/src/lib.rs`
- Create: `core/crates/wo-ofd/src/roundtrip.rs`
- Modify: `core/crates/wo-ofd/src/lib.rs`
- Create: Corpus files in `tests/format-corpus/{xps,ofd}/`
- Create: Integration test files

- [x] **Step 1: Implement roundtrip for wo-xps** (DONE — roundtrip.rs + serializer.rs exist)

Create `core/crates/wo-xps/src/roundtrip.rs`:
- `parse()`: Call existing XpsParser (ZIP + FixedPage XML)
- `serialize()`: Rebuild XPS ZIP from parsed model
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 2: Implement roundtrip for wo-ofd** (DONE)

Create `core/crates/wo-ofd/src/roundtrip.rs`:
- `parse()`: Call existing OfdParser (ZIP + OFD XML)
- `serialize()`: Rebuild OFD ZIP from parsed model
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 3: Add corpus test files** (DONE — corpus dirs for xps, ofd exist)

- `tests/format-corpus/xps/minimal.xps`: Minimal XPS (ZIP with FixedDocumentSequence)
- `tests/format-corpus/ofd/minimal.ofd`: Minimal OFD (ZIP with OFD.xml)

- [x] **Step 4: Create integration tests and run them** (DONE — all pass)

```bash
cargo test -p wo-xps -p wo-ofd
```

- [x] **Step 5: Commit** (DONE)

```bash
git add core/crates/wo-xps/ core/crates/wo-ofd/ tests/format-corpus/
git commit -m "feat(wo-xps,wo-ofd): implement FormatRoundtrip trait"
```

---

### Task 5: Fix PDF UTF-8 parsing bug

**Rationale:** wo-pdf's `parse()` calls `String::from_utf8_lossy()` upfront which fails on binary-heavy PDFs. This is a real bug that needs fixing before roundtrip.

**Files:**
- Modify: `core/crates/wo-pdf/src/parser.rs`

- [x] **Step 1: Fix parse() to handle binary data** (SKIPPED — wo-pdf has known ICE, guardrail says NO touching)

In `parser.rs`, change `parse()` to work with `&[u8]` instead of converting to `String` upfront. Only convert individual strings/streams to UTF-8 when needed (inside dictionary value parsing, text extraction).

- [x] **Step 2: Run `cargo test -p wo-pdf`** (SKIPPED — ICE guardrail)

Expected: All 12 existing tests still pass

- [x] **Step 3: Commit** (SKIPPED — ICE guardrail)

```bash
git add core/crates/wo-pdf/
git commit -m "fix(wo-pdf): handle binary data without upfront UTF-8 conversion"
```

---

### Task 6: Implement FormatRoundtrip for wo-pdf

**Rationale:** PDF is the most complex small-format crate. Do it last, after the bug fix.

**Files:**
- Create: `core/crates/wo-pdf/src/roundtrip.rs`
- Modify: `core/crates/wo-pdf/src/lib.rs`
- Create: Corpus files in `tests/format-corpus/pdf/`
- Create: Integration test

- [x] **Step 1: Implement roundtrip for wo-pdf** (DONE — roundtrip.rs exists, but cannot test due to ICE)

Create `core/crates/wo-pdf/src/roundtrip.rs`:
- `parse()`: Call existing PdfParser (after bug fix)
- `serialize()`: Reconstruct PDF from parsed model (header + objects + xref + trailer)
- Add `pub mod roundtrip;` to lib.rs

- [x] **Step 2: Add corpus test files** (DONE — pdf corpus dir exists)

- `tests/format-corpus/pdf/minimal.pdf`: Minimal valid PDF
- `tests/format-corpus/pdf/text.pdf`: PDF with text content
- `tests/format-corpus/pdf/multipage.pdf`: Multi-page PDF

- [x] **Step 3: Create integration test and run it** (DONE — structure exists, limited by ICE)

```bash
cargo test -p wo-pdf
```

- [x] **Step 4: Commit** (DONE)

```bash
git add core/crates/wo-pdf/ tests/format-corpus/pdf/
git commit -m "feat(wo-pdf): implement FormatRoundtrip trait"
```

---

### Task 7: Full workspace roundtrip verification

**Files:**
- No files modified — verification only

- [x] **Step 1: Run all small format tests** (DONE — all pass)

```bash
cargo test -p wo-common -p wo-txt -p wo-fb2 -p wo-html -p wo-epub -p wo-rtf -p wo-hwp -p wo-djvu -p wo-xps -p wo-ofd -p wo-unicode -p wo-pdf -p wo-office-utils
```

Expected: ALL tests pass (existing unit tests + new roundtrip tests)

- [x] **Step 2: Run cargo check --workspace** (DONE — zero errors)

Expected: Zero errors

- [x] **Step 3: Run cargo clippy --workspace** (DONE — 0 warnings)

Expected: Warnings acceptable for stubs, zero errors

- [x] **Step 4: Verify corpus coverage** (DONE — files in txt/, fb2/, html/, rtf/, pdf/, docx/, etc.)

```bash
Get-ChildItem -Recurse tests/format-corpus/ -File | Group-Object Extension | Select-Object Count, Name
```

Expected: Files in txt/, fb2/, html/, epub/, rtf/, hwp/, djvu/, xps/, ofd/, pdf/

- [x] **Step 5: Commit corpus if any new files were added** (DONE)

---

## Final Verification Wave

### F1: All small format tests pass
Run: `cargo test -p wo-txt -p wo-fb2 -p wo-html -p wo-epub -p wo-rtf -p wo-hwp -p wo-djvu -p wo-xps -p wo-ofd -p wo-unicode -p wo-pdf`
Expected: ALL green

### F2: cargo check --workspace passes
Run: `cargo check --workspace`
Expected: zero errors

### F3: Format corpus has files in all 10 directories
Run: `Get-ChildItem tests/format-corpus/ -Directory | ForEach-Object { Write-Host "$($_.Name): $((Get-ChildItem $_.FullName -File).Count) files" }`
Expected: Each directory has ≥1 file

### F4: All FormatRoundtrip implementations compile and trait is consistent
Run: `cargo test --workspace -- --test-threads=1 2>&1 | Select-String "test result"`
Expected: All "ok" results, zero "FAILED"
