# Coverage Summary

## Overall Coverage: 83.24% (1589/1909 lines)

## ✅ Fully Covered Modules (100%)
- `core/error.rs` - 5/5 lines ✅
- `core/syntax.rs` - 7/7 lines ✅
- `core/types.rs` - 75/75 lines ✅ (Display implementations now covered)
- `lib.rs` - 16/16 lines ✅ (Public APIs now covered)
- `plugins/flowchart/mod.rs` - 8/8 lines ✅
- `plugins/flowchart/whitespace.rs` - 5/5 lines ✅
- `plugins/gitgraph/mod.rs` - 8/8 lines ✅

## ⚠️ Partially Covered Modules (Need Attention)

### core/logging.rs (6/52 lines - 11.5%)
**Uncovered:** Most of `init_logging()` function
- WASM path (line 154) - hard to test without WASM target
- Native path with different format options (Compact, Pretty, Json)
- Environment variable handling paths
- Error recovery paths

**Recommendation:** Add tests for different logging configurations, but WASM path may remain uncovered without WASM target.

### flowchart/chumsky_parser.rs (186/237 lines - 78.5%)
**Uncovered:** Parser edge cases and error handling
- Various parser combinator branches
- Error recovery paths  
- Edge case syntax handling

**Recommendation:** Add tests for malformed input, edge cases, and error recovery.

### gitgraph/database.rs (64/102 lines - 62.7%)
**Uncovered:** Error paths and edge cases
- Error paths in `add_commit()` and `add_parent_edge()`
- Some graph analysis method branches
- Edge cases in graph operations

**Recommendation:** Add tests for error cases and edge cases in graph operations.

### gitgraph/renderer.rs (56/99 lines - 56.6%)
**Uncovered:** Renderer edge cases
- Different character set rendering paths
- Edge drawing in different directions
- Corner cases in canvas operations

**Recommendation:** Add tests for all character sets, all directions, edge cases in rendering.

### gitgraph/layout.rs (57/96 lines - 59.4%)
**Uncovered:** Layout algorithm branches
- Different direction handling (RL, BT especially)
- Edge case coordinate calculations
- Empty graph handling

**Recommendation:** Add tests for all directions (especially RL and BT), edge cases.

### gitgraph/parser.rs (30/41 lines - 73.2%)
**Uncovered:** Parser error paths and edge cases
- Error handling
- Edge cases in parsing

**Recommendation:** Add tests for error cases and edge cases.

### gitgraph/syntax_parser.rs (98/117 lines - 83.8%)
**Uncovered:** Syntax parser edge cases
- Error handling
- Edge cases in syntax parsing

**Recommendation:** Add tests for error cases and edge cases.

### gitgraph/detector.rs (33/41 lines - 80.5%)
**Uncovered:** Detector edge cases
- Some confidence scoring branches
- Edge cases in detection

**Recommendation:** Add tests for edge cases in detection.

### orchestrator.rs (94/97 lines - 96.9%)
**Uncovered:** Error handling paths
- Error propagation
- Edge cases in processing

**Recommendation:** Add tests for error cases in orchestrator.

## 📊 Coverage by Category

### Core Modules: 87.5% (91/104 lines)
- ✅ error.rs: 100%
- ✅ syntax.rs: 100%
- ✅ types.rs: 100%
- ⚠️ logging.rs: 11.5% (WASM path hard to test)

### Public API: 100% (16/16 lines)
- ✅ lib.rs: 100%

### Flowchart Plugin: 82.3% (974/1183 lines)
- ✅ mod.rs: 100%
- ✅ whitespace.rs: 100%
- ⚠️ chumsky_parser.rs: 78.5%
- ⚠️ database.rs: 93.1%
- ⚠️ detector.rs: 80.3%
- ⚠️ layout.rs: 95.9%
- ⚠️ parser.rs: 97.5%
- ⚠️ renderer.rs: 92.7%

### GitGraph Plugin: 66.2% (508/767 lines)
- ✅ mod.rs: 100%
- ⚠️ database.rs: 62.7%
- ⚠️ detector.rs: 80.5%
- ⚠️ layout.rs: 59.4%
- ⚠️ parser.rs: 73.2%
- ⚠️ renderer.rs: 56.6%
- ⚠️ syntax_parser.rs: 83.8%

## 🎯 Priority Areas for Improvement

1. **gitgraph/renderer.rs** (56.6%) - Add tests for all character sets and directions
2. **gitgraph/layout.rs** (59.4%) - Add tests for RL and BT directions
3. **gitgraph/database.rs** (62.7%) - Add tests for error paths and edge cases
4. **gitgraph/parser.rs** (73.2%) - Add tests for error cases
5. **flowchart/chumsky_parser.rs** (78.5%) - Add tests for parser edge cases
6. **core/logging.rs** (11.5%) - Add tests for native logging paths (WASM may remain uncovered)

## 📝 Notes

- Integration tests exist for many modules but aren't counted in `--lib` coverage
- Some code paths (WASM) are hard to test without WASM target
- Error paths are often uncovered - consider adding error injection tests
- Edge cases in parsers and renderers need more coverage
