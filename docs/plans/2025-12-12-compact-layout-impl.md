# Compact Layout Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement compact node spacing, layer alignment, and T-junction edge routing for flowcharts.

**Architecture:** Modify layout algorithm to use minimal spacing (1h/0v gaps), normalize node sizes within layers, and group multi-target edges for junction rendering. Renderer draws T-junctions for splits.

**Tech Stack:** Rust, existing layout.rs and renderer.rs in flowchart plugin

---

## Phase 1: Compact Spacing

### Task 1: Update LayoutConfig Defaults

**Files:**
- Modify: `crates/figurehead/src/plugins/flowchart/layout.rs:43-60`
- Test: `crates/figurehead/tests/layout_spacing.rs` (new)

**Step 1: Write failing test for compact spacing**

Create new test file:

```rust
// crates/figurehead/tests/layout_spacing.rs
use figurehead::plugins::flowchart::{FlowchartDatabase, FlowchartLayoutAlgorithm};
use figurehead::core::{Database, Direction, LayoutAlgorithm};

#[test]
fn test_compact_vertical_gap_is_zero() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    db.add_simple_edge("A", "B").unwrap();

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    // B should start immediately after A ends (0 gap)
    assert_eq!(node_b.y, node_a.y + node_a.height,
        "Vertical gap should be 0: B.y={} should equal A.y+A.height={}",
        node_b.y, node_a.y + node_a.height);
}

#[test]
fn test_compact_horizontal_gap_is_one() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("A", "A").unwrap();
    db.add_simple_node("B", "B").unwrap();
    // No edges - both in same layer

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    // Horizontal gap should be 1
    let gap = if node_a.x < node_b.x {
        node_b.x - (node_a.x + node_a.width)
    } else {
        node_a.x - (node_b.x + node_b.width)
    };
    assert_eq!(gap, 1, "Horizontal gap should be 1, got {}", gap);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p figurehead --test layout_spacing`
Expected: FAIL (gaps are currently 4 and 8)

**Step 3: Update LayoutConfig defaults**

```rust
// crates/figurehead/src/plugins/flowchart/layout.rs:51-60
impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            node_sep: 1,      // was 4: horizontal gap between nodes
            rank_sep: 0,      // was 8: vertical gap between layers
            min_node_width: 5,
            min_node_height: 3,
            padding: 1,       // was 2: canvas edge padding
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p figurehead --test layout_spacing`
Expected: PASS

**Step 5: Run full test suite to check for regressions**

Run: `cargo test -p figurehead`
Expected: Some snapshot tests may fail due to spacing changes

**Step 6: Commit**

```bash
git add crates/figurehead/src/plugins/flowchart/layout.rs crates/figurehead/tests/layout_spacing.rs
git commit -m "feat(layout): reduce spacing for compact diagrams"
```

---

## Phase 2: Layer Normalization

### Task 2: Add Height Normalization for TD/BT

**Files:**
- Modify: `crates/figurehead/src/plugins/flowchart/layout.rs:187-247`
- Test: `crates/figurehead/tests/layout_spacing.rs`

**Step 1: Write failing test for height normalization**

Add to `layout_spacing.rs`:

```rust
#[test]
fn test_td_layer_nodes_have_same_height() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("A", "Short").unwrap();
    db.add_simple_node("B", "Much Longer Label").unwrap();
    // No edges - both in layer 0

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    assert_eq!(node_a.height, node_b.height,
        "Nodes in same layer should have same height: A={}, B={}",
        node_a.height, node_b.height);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p figurehead --test layout_spacing test_td_layer_nodes_have_same_height`
Expected: FAIL (heights differ based on label)

**Step 3: Add layer height normalization**

In `layout.rs`, after calculating node_sizes, add normalization:

```rust
// After line ~141 (after node_sizes HashMap is built)
// Normalize heights within each layer for TD/BT
if matches!(direction, Direction::TopDown | Direction::BottomUp) {
    // Group nodes by layer and find max height per layer
    let mut layer_max_heights: HashMap<usize, usize> = HashMap::new();
    for &node_id in &sorted {
        if let (Some(&layer), Some(&(_, height))) = (layers.get(node_id), node_sizes.get(node_id)) {
            let max = layer_max_heights.entry(layer).or_insert(0);
            *max = (*max).max(height);
        }
    }

    // Update node sizes to match layer max
    for &node_id in &sorted {
        if let (Some(&layer), Some((width, _))) = (layers.get(node_id), node_sizes.get(node_id).copied()) {
            if let Some(&max_height) = layer_max_heights.get(&layer) {
                node_sizes.insert(node_id, (width, max_height));
            }
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p figurehead --test layout_spacing test_td_layer_nodes_have_same_height`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/figurehead/src/plugins/flowchart/layout.rs crates/figurehead/tests/layout_spacing.rs
git commit -m "feat(layout): normalize node heights within TD/BT layers"
```

---

### Task 3: Add Width Normalization for LR/RL

**Files:**
- Modify: `crates/figurehead/src/plugins/flowchart/layout.rs` (same area)
- Test: `crates/figurehead/tests/layout_spacing.rs`

**Step 1: Write failing test for width normalization**

Add to `layout_spacing.rs`:

```rust
#[test]
fn test_lr_layer_nodes_have_same_width() {
    let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);
    db.add_simple_node("A", "Hi").unwrap();
    db.add_simple_node("B", "Hello World").unwrap();
    // No edges - both in layer 0

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    let node_a = result.nodes.iter().find(|n| n.id == "A").unwrap();
    let node_b = result.nodes.iter().find(|n| n.id == "B").unwrap();

    assert_eq!(node_a.width, node_b.width,
        "Nodes in same layer should have same width: A={}, B={}",
        node_a.width, node_b.width);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p figurehead --test layout_spacing test_lr_layer_nodes_have_same_width`
Expected: FAIL

**Step 3: Add layer width normalization**

Extend the normalization code:

```rust
// Replace the TD/BT normalization with direction-aware version
match direction {
    Direction::TopDown | Direction::BottomUp => {
        // Normalize heights within layers
        let mut layer_max_heights: HashMap<usize, usize> = HashMap::new();
        for &node_id in &sorted {
            if let (Some(&layer), Some(&(_, height))) = (layers.get(node_id), node_sizes.get(node_id)) {
                let max = layer_max_heights.entry(layer).or_insert(0);
                *max = (*max).max(height);
            }
        }
        for &node_id in &sorted {
            if let (Some(&layer), Some((width, _))) = (layers.get(node_id), node_sizes.get(node_id).copied()) {
                if let Some(&max_height) = layer_max_heights.get(&layer) {
                    node_sizes.insert(node_id, (width, max_height));
                }
            }
        }
    }
    Direction::LeftRight | Direction::RightLeft => {
        // Normalize widths within layers
        let mut layer_max_widths: HashMap<usize, usize> = HashMap::new();
        for &node_id in &sorted {
            if let (Some(&layer), Some(&(width, _))) = (layers.get(node_id), node_sizes.get(node_id)) {
                let max = layer_max_widths.entry(layer).or_insert(0);
                *max = (*max).max(width);
            }
        }
        for &node_id in &sorted {
            if let (Some(&layer), Some((_, height))) = (layers.get(node_id), node_sizes.get(node_id).copied()) {
                if let Some(&max_width) = layer_max_widths.get(&layer) {
                    node_sizes.insert(node_id, (max_width, height));
                }
            }
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p figurehead --test layout_spacing test_lr_layer_nodes_have_same_width`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/figurehead/src/plugins/flowchart/layout.rs crates/figurehead/tests/layout_spacing.rs
git commit -m "feat(layout): normalize node widths within LR/RL layers"
```

---

## Phase 3: Edge Junction Routing

### Task 4: Add Edge Grouping Data Structure

**Files:**
- Modify: `crates/figurehead/src/plugins/flowchart/layout.rs:26-39`

**Step 1: Extend PositionedEdge struct**

```rust
// crates/figurehead/src/plugins/flowchart/layout.rs:26-39
/// Position data for a laid out edge
#[derive(Debug, Clone)]
pub struct PositionedEdge {
    pub from_id: String,
    pub to_id: String,
    pub waypoints: Vec<(usize, usize)>,
    /// For grouped edges from same source, the shared junction point
    pub junction: Option<(usize, usize)>,
    /// Index within the edge group (0 = first/leftmost in TD)
    pub group_index: Option<usize>,
    /// Total edges in this group
    pub group_size: Option<usize>,
}
```

**Step 2: Update edge creation to initialize new fields**

Find where PositionedEdge is created (around line 350) and update:

```rust
positioned_edges.push(PositionedEdge {
    from_id: edge.from.clone(),
    to_id: edge.to.clone(),
    waypoints: vec![(exit_x, exit_y), (entry_x, entry_y)],
    junction: None,
    group_index: None,
    group_size: None,
});
```

**Step 3: Run tests to ensure no regression**

Run: `cargo test -p figurehead`
Expected: PASS (structural change only)

**Step 4: Commit**

```bash
git add crates/figurehead/src/plugins/flowchart/layout.rs
git commit -m "refactor(layout): extend PositionedEdge with junction fields"
```

---

### Task 5: Implement Edge Grouping Logic

**Files:**
- Modify: `crates/figurehead/src/plugins/flowchart/layout.rs:310-360`
- Test: `crates/figurehead/tests/layout_spacing.rs`

**Step 1: Write failing test for edge grouping**

Add to `layout_spacing.rs`:

```rust
#[test]
fn test_split_edges_are_grouped() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("D", "Decision").unwrap();
    db.add_simple_node("Y", "Yes").unwrap();
    db.add_simple_node("N", "No").unwrap();
    db.add_simple_edge("D", "Y").unwrap();
    db.add_simple_edge("D", "N").unwrap();

    let layout = FlowchartLayoutAlgorithm::new();
    let result = layout.layout(&db).unwrap();

    // Both edges from D should have group info
    let edges_from_d: Vec<_> = result.edges.iter()
        .filter(|e| e.from_id == "D")
        .collect();

    assert_eq!(edges_from_d.len(), 2);

    for edge in &edges_from_d {
        assert!(edge.junction.is_some(), "Edge to {} should have junction", edge.to_id);
        assert_eq!(edge.group_size, Some(2), "Group size should be 2");
    }

    // Group indices should be 0 and 1
    let indices: Vec<_> = edges_from_d.iter()
        .filter_map(|e| e.group_index)
        .collect();
    assert!(indices.contains(&0) && indices.contains(&1));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p figurehead --test layout_spacing test_split_edges_are_grouped`
Expected: FAIL (junction is None)

**Step 3: Implement edge grouping**

Replace the edge routing section (around line 310-360):

```rust
// Route edges with grouping for splits
let edge_span = span!(Level::DEBUG, "route_edges");
let _edge_enter = edge_span.enter();

// Group edges by source node
let mut edges_by_source: HashMap<&str, Vec<&crate::core::EdgeData>> = HashMap::new();
for edge in database.edges() {
    edges_by_source.entry(&edge.from).or_default().push(edge);
}

let mut positioned_edges = Vec::new();
let node_positions: HashMap<&str, &PositionedNode> = positioned_nodes
    .iter()
    .map(|n| (n.id.as_str(), n))
    .collect();

for (source_id, edges) in edges_by_source {
    let Some(from) = node_positions.get(source_id) else { continue };

    let group_size = edges.len();
    let is_split = group_size > 1;

    // Calculate junction point for splits
    let junction = if is_split {
        match direction {
            Direction::TopDown => Some((from.x + from.width / 2, from.y + from.height + 1)),
            Direction::BottomUp => Some((from.x + from.width / 2, from.y.saturating_sub(1))),
            Direction::LeftRight => Some((from.x + from.width + 1, from.y + from.height / 2)),
            Direction::RightLeft => Some((from.x.saturating_sub(1), from.y + from.height / 2)),
        }
    } else {
        None
    };

    // Sort edges for consistent ordering (by target position)
    let mut sorted_edges: Vec<_> = edges.into_iter().enumerate().collect();
    sorted_edges.sort_by_key(|(_, e)| {
        node_positions.get(e.to.as_str()).map(|n| (n.x, n.y)).unwrap_or((0, 0))
    });

    for (group_index, edge) in sorted_edges {
        let Some(to) = node_positions.get(edge.to.as_str()) else { continue };

        // Calculate exit and entry points
        let (exit_x, exit_y, entry_x, entry_y) = match direction {
            Direction::TopDown => (
                from.x + from.width / 2,
                from.y + from.height,
                to.x + to.width / 2,
                to.y,
            ),
            Direction::BottomUp => (
                from.x + from.width / 2,
                from.y,
                to.x + to.width / 2,
                to.y + to.height,
            ),
            Direction::LeftRight => (
                from.x + from.width,
                from.y + from.height / 2,
                to.x,
                to.y + to.height / 2,
            ),
            Direction::RightLeft => (
                from.x,
                from.y + from.height / 2,
                to.x + to.width,
                to.y + to.height / 2,
            ),
        };

        positioned_edges.push(PositionedEdge {
            from_id: edge.from.clone(),
            to_id: edge.to.clone(),
            waypoints: vec![(exit_x, exit_y), (entry_x, entry_y)],
            junction,
            group_index: if is_split { Some(group_index) } else { None },
            group_size: if is_split { Some(group_size) } else { None },
        });
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p figurehead --test layout_spacing test_split_edges_are_grouped`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/figurehead/src/plugins/flowchart/layout.rs crates/figurehead/tests/layout_spacing.rs
git commit -m "feat(layout): group edges from same source with junction points"
```

---

## Phase 4: Junction Rendering

### Task 6: Add T-Junction Drawing

**Files:**
- Modify: `crates/figurehead/src/plugins/flowchart/renderer.rs:623-700`
- Test: `crates/figurehead/tests/layout_spacing.rs`

**Step 1: Write test for junction characters in output**

Add to `layout_spacing.rs`:

```rust
use figurehead::core::{CharacterSet, Renderer};
use figurehead::plugins::flowchart::FlowchartRenderer;

#[test]
fn test_split_renders_junction_character() {
    let mut db = FlowchartDatabase::with_direction(Direction::TopDown);
    db.add_simple_node("D", "D").unwrap();
    db.add_simple_node("Y", "Y").unwrap();
    db.add_simple_node("N", "N").unwrap();
    db.add_simple_edge("D", "Y").unwrap();
    db.add_simple_edge("D", "N").unwrap();

    let renderer = FlowchartRenderer::new();
    let output = renderer.render(&db).unwrap();

    // Should contain a T-junction character
    assert!(
        output.contains('┬') || output.contains('┴') ||
        output.contains('├') || output.contains('┤') ||
        output.contains('+'),  // ASCII fallback
        "Output should contain junction character:\n{}", output
    );
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p figurehead --test layout_spacing test_split_renders_junction_character`
Expected: FAIL (no junction character)

**Step 3: Update draw_edge to handle junctions**

Add new method and update draw_edge:

```rust
// Add after draw_edge_label method (around line 735)

fn draw_junction(
    &self,
    canvas: &mut AsciiCanvas,
    junction: (usize, usize),
    direction: Direction,
    group_size: usize,
) {
    let (jx, jy) = junction;

    // Draw the junction point
    let junction_char = match direction {
        Direction::TopDown => if self.style.is_ascii() { '+' } else { '┬' },
        Direction::BottomUp => if self.style.is_ascii() { '+' } else { '┴' },
        Direction::LeftRight => if self.style.is_ascii() { '+' } else { '├' },
        Direction::RightLeft => if self.style.is_ascii() { '+' } else { '┤' },
    };
    canvas.set_char(jx, jy, junction_char);
}

fn draw_split_edge(
    &self,
    canvas: &mut AsciiCanvas,
    from_center: (usize, usize),
    junction: (usize, usize),
    to_center: (usize, usize),
    edge_type: EdgeType,
    direction: Direction,
) {
    let chars = EdgeChars::for_type(edge_type, self.style);
    if chars.is_invisible() {
        return;
    }

    let (fx, fy) = from_center;
    let (jx, jy) = junction;
    let (tx, ty) = to_center;
    let has_arrow = edge_type.has_arrow();

    match direction {
        Direction::TopDown => {
            // Vertical from source to junction
            self.draw_vertical_line(canvas, fx, fy, jy, &chars);
            // Horizontal from junction toward target
            let corner_x = tx;
            if corner_x != jx {
                self.draw_horizontal_line(canvas, jy, jx.min(corner_x), jx.max(corner_x), &chars);
            }
            // Corner
            let corner = if self.style.is_ascii() {
                '+'
            } else if tx < jx {
                '┐'
            } else if tx > jx {
                '┌'
            } else {
                '│'
            };
            if corner_x != jx {
                canvas.set_char(corner_x, jy, corner);
            }
            // Vertical down to target
            let end_y = if has_arrow { ty.saturating_sub(1) } else { ty };
            self.draw_vertical_line(canvas, corner_x, jy, end_y, &chars);
            if has_arrow {
                canvas.set_char(corner_x, end_y, chars.arrow_down);
            }
        }
        Direction::BottomUp => {
            // Similar but reversed
            self.draw_vertical_line(canvas, fx, jy, fy, &chars);
            let corner_x = tx;
            if corner_x != jx {
                self.draw_horizontal_line(canvas, jy, jx.min(corner_x), jx.max(corner_x), &chars);
            }
            let corner = if self.style.is_ascii() {
                '+'
            } else if tx < jx {
                '┘'
            } else if tx > jx {
                '└'
            } else {
                '│'
            };
            if corner_x != jx {
                canvas.set_char(corner_x, jy, corner);
            }
            let end_y = if has_arrow { ty + 1 } else { ty };
            self.draw_vertical_line(canvas, corner_x, end_y, jy, &chars);
            if has_arrow {
                canvas.set_char(corner_x, end_y, chars.arrow_up);
            }
        }
        Direction::LeftRight => {
            // Horizontal from source to junction
            self.draw_horizontal_line(canvas, fy, fx, jx, &chars);
            // Vertical from junction toward target
            let corner_y = ty;
            if corner_y != jy {
                self.draw_vertical_line(canvas, jx, jy.min(corner_y), jy.max(corner_y), &chars);
            }
            let corner = if self.style.is_ascii() {
                '+'
            } else if ty < jy {
                '└'
            } else if ty > jy {
                '┌'
            } else {
                '─'
            };
            if corner_y != jy {
                canvas.set_char(jx, corner_y, corner);
            }
            // Horizontal to target
            let end_x = if has_arrow { tx.saturating_sub(1) } else { tx };
            self.draw_horizontal_line(canvas, corner_y, jx, end_x, &chars);
            if has_arrow {
                canvas.set_char(end_x, corner_y, chars.arrow_right);
            }
        }
        Direction::RightLeft => {
            // Similar but reversed
            self.draw_horizontal_line(canvas, fy, jx, fx, &chars);
            let corner_y = ty;
            if corner_y != jy {
                self.draw_vertical_line(canvas, jx, jy.min(corner_y), jy.max(corner_y), &chars);
            }
            let corner = if self.style.is_ascii() {
                '+'
            } else if ty < jy {
                '┘'
            } else if ty > jy {
                '┐'
            } else {
                '─'
            };
            if corner_y != jy {
                canvas.set_char(jx, corner_y, corner);
            }
            let end_x = if has_arrow { tx + 1 } else { tx };
            self.draw_horizontal_line(canvas, corner_y, end_x, jx, &chars);
            if has_arrow {
                canvas.set_char(end_x, corner_y, chars.arrow_left);
            }
        }
    }
}
```

**Step 4: Update render method to use junction drawing**

In the render method (around line 827), update edge drawing:

```rust
// Draw edges first (so nodes overlay them)
// Track which junctions we've drawn
let mut drawn_junctions: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

for edge in &layout_result.edges {
    let edge_data = database.edges().find(|e| e.from == edge.from_id && e.to == edge.to_id);
    let edge_type = edge_data.map(|e| e.edge_type).unwrap_or(EdgeType::Arrow);

    if let Some(junction) = edge.junction {
        // Draw junction if not already drawn
        if !drawn_junctions.contains(&junction) {
            self.draw_junction(&mut canvas, junction, database.direction(), edge.group_size.unwrap_or(1));
            drawn_junctions.insert(junction);
        }

        // Draw split edge through junction
        let from_node = layout_result.nodes.iter().find(|n| n.id == edge.from_id);
        let to_node = layout_result.nodes.iter().find(|n| n.id == edge.to_id);

        if let (Some(from), Some(to)) = (from_node, to_node) {
            let from_center = match database.direction() {
                Direction::TopDown => (from.x + from.width / 2, from.y + from.height),
                Direction::BottomUp => (from.x + from.width / 2, from.y),
                Direction::LeftRight => (from.x + from.width, from.y + from.height / 2),
                Direction::RightLeft => (from.x, from.y + from.height / 2),
            };
            let to_center = match database.direction() {
                Direction::TopDown => (to.x + to.width / 2, to.y),
                Direction::BottomUp => (to.x + to.width / 2, to.y + to.height),
                Direction::LeftRight => (to.x, to.y + to.height / 2),
                Direction::RightLeft => (to.x + to.width, to.y + to.height / 2),
            };

            self.draw_split_edge(&mut canvas, from_center, junction, to_center, edge_type, database.direction());
        }
    } else {
        // Regular edge
        self.draw_edge(&mut canvas, &edge.waypoints, edge_type);
    }

    // Draw label if present
    if let Some(edge_data) = edge_data {
        if let Some(ref label) = edge_data.label {
            self.draw_edge_label(&mut canvas, &edge.waypoints, label);
        }
    }
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p figurehead --test layout_spacing test_split_renders_junction_character`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/figurehead/src/plugins/flowchart/renderer.rs crates/figurehead/tests/layout_spacing.rs
git commit -m "feat(renderer): draw T-junctions for edge splits"
```

---

### Task 7: Update Snapshot Tests

**Files:**
- Update: `crates/figurehead/tests/snapshots/*.snap` (run with UPDATE_FIXTURES=1)

**Step 1: Run all tests to see failures**

Run: `cargo test -p figurehead`
Expected: Snapshot tests fail due to layout changes

**Step 2: Review and update snapshots**

Run: `UPDATE_FIXTURES=1 cargo test -p figurehead`
Expected: Snapshots updated

**Step 3: Manually verify updated snapshots look correct**

Run: `git diff crates/figurehead/tests/snapshots/`
Review each change for correct compact layout

**Step 4: Commit**

```bash
git add crates/figurehead/tests/snapshots/
git commit -m "test: update snapshots for compact layout"
```

---

### Task 8: Rebuild WASM and Test Web Editor

**Step 1: Build WASM**

Run: `just wasm-build`
Expected: Build succeeds

**Step 2: Start web server and test**

Run: `just web`
Test with various diagrams in browser

**Step 3: Final commit if any fixes needed**

```bash
git add -A
git commit -m "fix: address compact layout issues found in testing"
```

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1 | Update spacing defaults | layout.rs |
| 2 | Height normalization TD/BT | layout.rs |
| 3 | Width normalization LR/RL | layout.rs |
| 4 | Edge junction data structure | layout.rs |
| 5 | Edge grouping logic | layout.rs |
| 6 | T-junction rendering | renderer.rs |
| 7 | Update snapshots | tests/snapshots/ |
| 8 | WASM rebuild and test | - |
