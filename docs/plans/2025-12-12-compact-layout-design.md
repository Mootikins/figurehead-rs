# Compact Layout Design

## Overview

Redesign the flowchart layout algorithm to produce more compact, readable diagrams with proper edge routing for splits/forks.

## Goals

1. Minimize whitespace between nodes
2. Proper visual handling of split/fork edges
3. Consistent node sizing within layers
4. Clear orthogonal edge routing

## Node Spacing Rules

### Internal Padding
- Minimum 1 cell padding inside nodes around text
- Padding is **even** (balanced left/right, top/bottom)
- Padding grows to match layer requirements

### External Gaps
- **Horizontal gap**: 1 cell between node borders
- **Vertical gap**: 0 cells (nodes touch, edge line occupies the junction)

### Visual Example (TD)
```
┌───────┐
│   A   │
└───┬───┘
┌───┴───┐
│   B   │
└───────┘
```

### Visual Example (LR)
```
┌───┐ ┌───┐
│ A │─│ B │
└───┘ └───┘
```

## Layer Alignment

Nodes in the same layer should have consistent sizing:

### TD/BT (Vertical Flow)
- Match **heights** within layer
- Minimal horizontal padding (don't over-expand width)

### LR/RL (Horizontal Flow)
- Match **widths** within layer
- Horizontal padding can grow to align nodes

### Example (TD with uneven labels)
```
┌─────────┐ ┌─────────┐
│  Short  │ │  Long   │
│         │ │  Label  │
└─────────┘ └─────────┘
```
Both nodes have same height, text centered.

## Edge Routing

### Connection Points
- Edges connect at **center of node edge**
- Never connect at corners
- Always orthogonal (horizontal or vertical segments only)

### Single Edge (TD)
```
┌───────┐
│   A   │
└───┬───┘
    │
┌───┴───┐
│   B   │
└───────┘
```

### Single Edge (LR)
```
┌───┐   ┌───┐
│ A │───│ B │
└───┘   └───┘
```

## Split/Fork Handling

When a node has multiple outgoing edges, use a **T-junction** element that is part of the line, not the node border.

### TD Split (Decision → Yes, No)
```
      ┌──────────┐
      │ Decision │
      └────┬─────┘
           │
         ┬─┴─┬
         │   │
      ┌──┴──┐ ┌──┴──┐
      │ Yes │ │ No  │
      └─────┘ └─────┘
```

### LR Split
```
┌──────────┐     ┌─────┐
│ Decision │──┬──│ Yes │
└──────────┘  │  └─────┘
              │  ┌─────┐
              └──│ No  │
                 └─────┘
```

### Junction Characters
- TD/BT: `┬`, `┴`, `├`, `┤` for horizontal splits
- LR/RL: `├`, `┤`, `┬`, `┴` for vertical splits
- Crossings: `┼`

## Implementation Changes

### layout.rs Changes
1. Update `LayoutConfig` defaults:
   - `node_sep: 1` (was 4)
   - `rank_sep: 0` (was 8) - vertical gap handled by edge routing
   - `padding: 1` (internal node padding)

2. Add layer normalization:
   - Calculate max height (TD) or width (LR) per layer
   - Expand nodes to match

3. Improve edge routing:
   - Track multiple edges from same source
   - Calculate junction points for splits
   - Generate waypoints for orthogonal routing

### renderer.rs Changes
1. Update edge drawing:
   - Handle junction/split points
   - Draw T-junctions with proper characters
   - Route around nodes if needed

2. Node rendering:
   - Ensure center connections
   - Handle variable padding

### New Data Structures
```rust
pub struct PositionedEdge {
    pub from_id: String,
    pub to_id: String,
    pub waypoints: Vec<(usize, usize)>,
    pub junction_point: Option<(usize, usize)>, // For splits
}

pub struct EdgeGroup {
    pub source: String,
    pub targets: Vec<String>,
    pub junction: (usize, usize),
}
```

## Testing Strategy

1. Unit tests for each spacing scenario
2. Visual snapshot tests for:
   - Simple linear flow
   - Binary split (Decision → Yes/No)
   - Triple+ split
   - Merge points (multiple edges → single node)
3. Test all directions (TD, BT, LR, RL)
