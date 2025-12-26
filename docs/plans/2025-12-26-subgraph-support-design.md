# Subgraph Support Design

## Overview

Add visual subgraph rendering to the flowchart plugin. Parsing already works but nodes are currently flattened - this design adds membership storage, grouped layout, and boundary rendering.

## Scope

- **In scope:** Single-level subgraphs with bounding box, centered title, edge pass-through
- **Out of scope:** Nested subgraphs (parse but flatten with warning)

## Data Model

### New `Subgraph` struct

```rust
// In database.rs
pub struct Subgraph {
    pub id: String,           // e.g., "subgraph_0" or slugified title
    pub title: String,        // Display title: "Cluster A"
    pub members: Vec<String>, // Node IDs contained in this subgraph
}
```

### Database additions

```rust
impl FlowchartDatabase {
    subgraphs: Vec<Subgraph>,

    pub fn add_subgraph(&mut self, title: String, members: Vec<String>) -> String;
    pub fn get_subgraph(&self, id: &str) -> Option<&Subgraph>;
    pub fn subgraphs(&self) -> impl Iterator<Item = &Subgraph>;
    pub fn node_subgraph(&self, node_id: &str) -> Option<&Subgraph>;
}
```

### Parser change

Instead of flattening `Statement::Subgraph(title, children)`, register it with the database and track membership. Collect node IDs from child statements.

## Layout Algorithm

### Two-phase approach

**Phase 1 - Subgraph internal layout:**
- For each subgraph, layout its member nodes as a standalone graph
- Calculate bounding box from member node positions
- Add padding: 1 cell each side for border, 1 cell top for title

**Phase 2 - Main layout:**
- Exclude subgraph members from main layout (already positioned)
- Treat subgraph bounding box as a "super-node" for positioning
- Position non-subgraph nodes and other subgraphs around it

### New layout types

```rust
pub struct PositionedSubgraph {
    pub id: String,
    pub title: String,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

// Add to LayoutResult
pub struct LayoutResult {
    pub nodes: Vec<PositionedNode>,
    pub edges: Vec<PositionedEdge>,
    pub subgraphs: Vec<PositionedSubgraph>,  // NEW
    pub width: usize,
    pub height: usize,
}
```

### Edge routing

Edges crossing subgraph boundaries get waypoints adjusted to pass through the border at the correct intersection point.

## Renderer

### Target output

```
┌────────────── Cluster A ──────────────┐
│  ┌─────┐    ┌─────┐                   │
│  │ One │───▶│ Two │───────────────────┼───▶  Outside
│  └─────┘    └─────┘                   │
└───────────────────────────────────────┘
```

### Rendering order

1. Draw subgraph borders (background layer)
2. Draw nodes inside subgraphs
3. Draw nodes outside subgraphs
4. Draw edges last, handling boundary crossings

### Boundary crossing

When edge crosses subgraph border:
- Calculate intersection point with border
- Use junction character: `┼` for crossing, `├` `┤` `┬` `┴` for touching

**Implementation note:** Take extra care when multiple edges cross near the same point or near corners - may need special handling for readability.

### New renderer methods

```rust
fn draw_subgraph(&self, canvas: &mut AsciiCanvas, subgraph: &PositionedSubgraph) {
    // Draw border box with corners: ┌ ┐ └ ┘
    // Draw centered title in top border: ┌─── Title ───┐
}

fn draw_boundary_crossing(&self, canvas: &mut AsciiCanvas,
    edge: &PositionedEdge, subgraph: &PositionedSubgraph) {
    // Calculate intersection point
    // Draw appropriate junction character
}
```

## Testing

Combined fixture approach - fewer comprehensive tests:

1. `subgraph_lr` - LR layout with internal nodes, edge to outside, multiple subgraphs
2. `subgraph_td` - TD layout equivalent
3. `subgraph_edge_cases` - Empty subgraph, edges in both directions, near-corner crossings

## Error Handling

| Case | Behavior |
|------|----------|
| Nested subgraphs | Parse OK, warn, flatten to outermost |
| Empty subgraph | Valid, render empty box with title |
| Node in multiple subgraphs | First wins, warn |

## Documentation Updates

- **README.md**: Add subgraphs to features, note "single level only" limitation
- **CLAUDE.md**: Note subgraph architecture for future AI work

## Files to Modify

1. `database.rs` - Add Subgraph struct and methods
2. `parser.rs` - Register subgraphs instead of flattening
3. `layout.rs` - Two-phase layout, PositionedSubgraph
4. `renderer.rs` - draw_subgraph, boundary crossing logic
5. `README.md` - Documentation
6. New test fixtures
