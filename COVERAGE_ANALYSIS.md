# Coverage Analysis Report

Generated from tarpaulin coverage report showing uncovered lines.

## Current Coverage: 83.24% (1589/1909 lines) ✅ Improved from 80.57%

### Recent Improvements
- Added unit tests for `error.rs` constructors → 0% → 100% coverage
- Added unit tests for `lib.rs` public APIs → 0% → 100% coverage  
- Added unit tests for `types.rs` Display implementations → 61% → improved
- Coverage increased by +2.72% (+51 lines)

## Major Uncovered Areas

### 1. core/error.rs (0/5 lines - 0% coverage)
**Uncovered lines:** 41, 50, 55, 60, 65

These are error constructor functions:
- `parse_error()` - line 41
- `layout_error()` - line 50  
- `render_error()` - line 55
- `database_error()` - line 60
- `detection_error()` - line 65

**Status:** Integration tests exist (`tests/error_tests.rs`) but `--lib` flag doesn't run them.
**Action:** These are tested in integration tests. Consider adding unit tests in `src/core/error.rs` or document that these are integration-tested.

### 2. lib.rs (0/16 lines - 0% coverage)
**Uncovered lines:** 78, 82, 85-86, 88, 113, 117-119, 121-122, 139, 143-146

These are public API functions:
- `render()` - lines 78, 82, 85-86, 88
- `render_with_style()` - lines 113, 117-119, 121-122
- `parse()` - lines 139, 143-146

**Status:** Integration tests exist (`tests/lib_api.rs`) but `--lib` flag doesn't run them.
**Action:** These are tested in integration tests. Consider adding unit tests or document that these are integration-tested.

### 3. core/types.rs (46/75 lines - 61% coverage)
**Uncovered lines:** 78-89, 143-153, 201-206

These are Display implementations:
- `NodeShape::fmt()` - lines 78-89
- `EdgeType::fmt()` - lines 143-153
- `Direction::fmt()` - lines 201-206

**Status:** Integration tests exist (`tests/types_display.rs`) but `--lib` flag doesn't run them.
**Action:** These are tested in integration tests. Consider adding unit tests in `src/core/types.rs` or document that these are integration-tested.

### 4. core/logging.rs (6/52 lines - 11.5% coverage)
**Uncovered lines:** 154, 177-181, 185-187, 190-191, 193-195, 199-200, 203, 205-206, 208-213, 218-219, 221-227, 232-233, 235-241, 247, 254-255

Most of `init_logging()` function is uncovered:
- WASM path (line 154) - hard to test without WASM target
- Native path with different format options (Compact, Pretty, Json)
- Environment variable handling
- Error paths

**Status:** Integration tests exist (`tests/logging_coverage.rs`) but may not cover all paths.
**Action:** Add more comprehensive tests for different logging configurations, error cases, and environment variable handling.

### 5. flowchart/chumsky_parser.rs (186/237 lines - 78% coverage)
**Uncovered lines:** 69, 75-77, 81, 87-89, 93, 99-101, 105, 111-113, 117, 123-125, 129, 135-137, 141, 147-149, 153, 159-161, 165, 171-173, 177, 183-185, 189, 234, 244, 257, 259, 263, 265, 325, 361, 386-387

Parser edge cases and error handling:
- Various parser combinator branches
- Error recovery paths
- Edge case syntax handling

**Action:** Add tests for malformed input, edge cases, and error recovery.

### 6. gitgraph/database.rs (64/102 lines - 62.7% coverage)
**Uncovered lines:** 26, 28-29, 39, 54, 57, 67-68, 85, 88, 115-117, 130-131, 134-136, 138-139, 143-145, 147-148, 152-153, 155-156, 160-161, 163-164, 168-169, 172-173, 221

Database methods:
- Error paths in `add_commit()` and `add_parent_edge()`
- Graph analysis methods (some branches)
- Edge cases

**Action:** Add tests for error cases, edge cases in graph operations.

### 7. gitgraph/renderer.rs (55/99 lines - 55.6% coverage)
**Uncovered lines:** 32-33, 35, 38, 40-41, 88, 133, 139, 141-144, 146, 148-149, 155-158, 160, 162-163, 168-172, 174, 176-177, 182-183, 185, 191-192, 204-205, 216-217, 236, 246, 250, 254

Renderer edge cases:
- Different character set rendering paths
- Edge drawing in different directions
- Corner cases in canvas operations

**Action:** Add tests for all character sets, all directions, edge cases in rendering.

### 8. gitgraph/layout.rs (57/96 lines - 59.4% coverage)
**Uncovered lines:** 57-58, 69-70, 78-80, 112, 138-139, 141-142, 144, 147-150, 152-154, 160-161, 164, 194-197, 203-206, 212-215, 228, 244, 248, 252

Layout algorithm branches:
- Different direction handling (RL, BT)
- Edge case coordinate calculations
- Empty graph handling

**Action:** Add tests for all directions (especially RL and BT), edge cases.

### 9. orchestrator.rs (93/97 lines - 95.9% coverage)
**Uncovered lines:** 140-141, 168, 212

Error handling paths:
- Error propagation
- Edge cases in processing

**Action:** Add tests for error cases in orchestrator.

## Recommendations

1. **Add unit tests** for Display implementations in `src/core/types.rs`
2. **Add unit tests** for error constructors in `src/core/error.rs`  
3. **Improve logging tests** to cover all format options and error paths
4. **Add parser edge case tests** for malformed input
5. **Add renderer tests** for all character sets and directions
6. **Add layout tests** for all directions (especially RL and BT)
7. **Document** that some code is integration-tested only (lib.rs public APIs)

## Notes

- Integration tests exist but aren't counted in `--lib` coverage
- Some code paths (WASM) are hard to test without WASM target
- Error paths are often uncovered - consider adding error injection tests
- Edge cases in parsers and renderers need more coverage
