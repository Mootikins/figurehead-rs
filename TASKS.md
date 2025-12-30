---
description: Figurehead improvement loop - clippy, syntax, tests, layout
context_files:
  - crates/figurehead/src/plugins/class/chumsky_parser.rs
  - crates/figurehead/src/core/chumsky_utils.rs
  - thoughts/backlog.md
verify: just ci
tdd: true
---

## Phase 1: Fix Clippy Warnings

### 1.1 Class Parser Cleanup

- [x] Remove unused import `inline_whitespace` [id:: 1.1.1]
  - File: `plugins/class/chumsky_parser.rs:6`
  - [tests:: cargo clippy]

- [x] Remove unnecessary `.clone()` calls on Copy types [id:: 1.1.2]
  - Parsers that impl Copy don't need clone
  - Lines: 106, 166, 168, 171, 218, 223, 225, 227, 229
  - [tests:: cargo clippy]

- [x] Replace redundant closure with function reference [id:: 1.1.3]
  - Line 133: `|c| Visibility::from_char(c)` → `Visibility::from_char`
  - [tests:: cargo clippy]

- [x] Verify zero warnings [id:: 1.1.4] [deps:: 1.1.1, 1.1.2, 1.1.3]
  - Run: `cargo clippy --all-targets`
  - [tests:: just ci]

### 1.2 Phase 1 QA

- [x] All tests pass (`just ci`) [id:: 1.2.1] [deps:: 1.1.4]
- [x] No regressions in snapshot tests [id:: 1.2.2] [deps:: 1.1.4]
- [ ] Commit changes with descriptive message [id:: 1.2.3] [deps:: 1.2.1, 1.2.2]

**STOP HERE** - Output `<promise>PHASE 1 COMPLETE</promise>` for human review.

---

## Phase 2: Missing Syntax Support

### 2.1 Research Current Gaps

- [ ] Compare mermaid.js class diagram syntax to current parser [id:: 2.1.1]
  - Check: annotations, generics, namespaces, notes
  - Document gaps in this file

- [ ] Compare mermaid.js flowchart syntax to current parser [id:: 2.1.2]
  - Check: click events, styling, fontawesome icons, subgraph direction
  - Document gaps in this file

### 2.2 Implement Priority Gaps (TDD)

- [ ] Write failing tests for chosen syntax gaps [id:: 2.2.1] [deps:: 2.1.1, 2.1.2]
  - Pick 1-2 high-value missing features
  - Tests first, implementation second

- [ ] Implement syntax support [id:: 2.2.2] [deps:: 2.2.1]
  - Follow existing parser patterns
  - [tests:: cargo test]

### 2.3 Phase 2 QA

- [ ] All tests pass (`just ci`) [id:: 2.3.1] [deps:: 2.2.2]
- [ ] New syntax documented in Progress Notes [id:: 2.3.2] [deps:: 2.2.2]
- [ ] Commit changes [id:: 2.3.3] [deps:: 2.3.1, 2.3.2]

**STOP HERE** - Output `<promise>PHASE 2 COMPLETE</promise>` for human review.

---

## Phase 3: Test Coverage Improvement

### 3.1 Coverage Analysis

- [ ] Identify modules with low test coverage [id:: 3.1.1]
  - Use `cargo llvm-cov` or manual inspection
  - Focus on core/ and plugins/

### 3.2 Add Tests (TDD)

- [ ] Add edge case tests for identified gaps [id:: 3.2.1] [deps:: 3.1.1]
  - Error conditions
  - Boundary cases
  - Unicode handling

- [ ] Add integration tests for full pipelines [id:: 3.2.2] [deps:: 3.1.1]
  - End-to-end diagram rendering
  - [tests:: cargo test]

### 3.3 Phase 3 QA

- [ ] All tests pass (`just ci`) [id:: 3.3.1] [deps:: 3.2.1, 3.2.2]
- [ ] Test count increased (document delta) [id:: 3.3.2] [deps:: 3.2.1, 3.2.2]
- [ ] Commit changes [id:: 3.3.3] [deps:: 3.3.1, 3.3.2]

**STOP HERE** - Output `<promise>PHASE 3 COMPLETE</promise>` for human review.

---

## Phase 4: Layout Algorithm Study

### 4.1 Research

- [ ] Study dagre layout algorithm (MIT) [id:: 4.1.1]
  - Source: https://github.com/dagrejs/dagre
  - Document key concepts

- [ ] Study mermaid.js layout adaptations [id:: 4.1.2]
  - Source: https://github.com/mermaid-js/mermaid
  - Note differences from dagre

### 4.2 Document Porting Plan

- [ ] Create layout porting plan in thoughts/ [id:: 4.2.1] [deps:: 4.1.1, 4.1.2]
  - Identify algorithms to port
  - Map JS concepts to Rust idioms
  - Estimate scope

### 4.3 Phase 4 QA

- [ ] Research notes saved to thoughts/ [id:: 4.3.1] [deps:: 4.2.1]
- [ ] Porting plan is actionable (concrete tasks) [id:: 4.3.2] [deps:: 4.2.1]
- [ ] Commit research artifacts [id:: 4.3.3] [deps:: 4.3.1, 4.3.2]

**STOP HERE** - Output `<promise>PHASE 4 COMPLETE</promise>` for human review.

---

## Progress Notes

<!-- Ralph will append notes here as work progresses -->

