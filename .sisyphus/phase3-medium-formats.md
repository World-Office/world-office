# Phase 3: Medium Format Roundtrips

**Goal:** Implement `FormatRoundtrip` trait for remaining medium-complexity format crates (wo-odf, wo-docx-renderer).

**Status:** Phase2 completed 10 small formats. Phase3 focuses on the two remaining medium formats.

---

## Task 1: Implement FormatRoundtrip for wo-odf (841 C++ files → Rust)

**Corpus:** ODF (OpenDocument Format) files (.odt, .ods, .odp)

**Steps:**
1. Read `core/crates/wo-odf/src/lib.rs` to understand current structure
2. Create `core/crates/wo-odf/src/roundtrip.rs` implementing `FormatRoundtrip` trait
3. Add ODF test corpus files to `test_corpus/odf/` directory
4. Implement roundtrip tests: parse → serialize → parse again → compare
5. Export `OdfRoundtrip` from lib.rs
6. Run tests: `cargo test -p wo-odf -- --test-threads=1`
7. Fix any failures
8. Commit: `feat(wo-odf): implement FormatRoundtrip trait for ODF formats`

**Expected challenges:**
- ODF is ZIP + XML structure (like OOXML but different schema)
- Multiple sub-formats: .odt (text), .ods (spreadsheets), .odp (presentations)
- Need to handle manifest.xml, content.xml, styles.xml, meta.xml

---

## Task 2: Implement FormatRoundtrip for wo-docx-renderer (46 files)

**Corpus:** DOCX rendering pipeline (DOCX → PDF conversion)

**Steps:**
1. Read `core/crates/wo-docx-renderer/src/lib.rs` to understand structure
2. Create `core/crates/wo-docx-renderer/src/roundtrip.rs` implementing `FormatRoundtrip`
3. Add test corpus: sample DOCX files with various features
4. Implement roundtrip: render DOCX to PDF, verify output
5. Export `DocxRendererRoundtrip` from lib.rs
6. Run tests: `cargo test -p wo-docx-renderer`
7. Fix any failures
8. Commit: `feat(wo-docx-renderer): implement FormatRoundtrip trait`

**Expected challenges:**
- Rendering involves layout engine, fonts, graphics
- May need headless browser or PDF comparison tools
- PDF comparison is non-trivial (binary format)

---

## Task 3: Verify cumulative format coverage

**Goal:** Ensure all 15 format crates (10 small + 2 medium + 3 large) implement FormatRoundtrip

**Verification:**
```bash
cargo test -p wo-txt -p wo-unicode -p wo-fb2 -p wo-html -p wo-epub -p wo-rtf -p wo-hwp -p wo-djvu -p wo-xps -p wo-ofd -p wo-pdf -p wo-odf -p wo-docx-renderer -- --test-threads=1
```

**Expected:** All tests pass

**Commit:** `test: add cumulative format roundtrip verification`

---

## Exit Criteria

- [x] wo-odf: FormatRoundtrip implemented + tests passing
- [x] wo-docx-renderer: FormatRoundtrip implemented + tests passing
- [x] All 15 format crates have passing roundtrip tests (16/16 have roundtrip.rs; 15/15 pass, wo-pdf excluded due to known rustc ICE)
- [x] Documentation updated with ODF and DOCX rendering patterns
