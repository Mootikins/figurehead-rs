//! Flowchart layout implementation
//!
//! Arranges flowchart elements in a coordinate system.

use anyhow::Result;

use super::FlowchartDatabase;
use crate::core::{Database, LayoutAlgorithm};

/// Position data for layout
#[derive(Debug, Clone)]
pub struct PositionedNode {
    pub id: String,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Layout output containing positioned elements
#[derive(Debug)]
pub struct FlowchartLayout {
    pub nodes: Vec<PositionedNode>,
    pub width: usize,
    pub height: usize,
}

/// Flowchart layout algorithm implementation
pub struct FlowchartLayoutAlgorithm;

impl FlowchartLayoutAlgorithm {
    pub fn new() -> Self {
        Self
    }
}

impl LayoutAlgorithm<FlowchartDatabase> for FlowchartLayoutAlgorithm {
    type Output = FlowchartLayout;

    fn layout(&self, database: &FlowchartDatabase) -> Result<Self::Output> {
        layout_nodes(database)
    }

    fn name(&self) -> &'static str {
        "grid"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn direction(&self) -> &'static str {
        "LR"
    }
}

/// Layout configuration
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub node_spacing_x: usize,
    pub node_spacing_y: usize,
    pub edge_spacing: usize,
    pub min_node_width: usize,
    pub min_node_height: usize,
    pub padding: usize,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            node_spacing_x: 4,
            node_spacing_y: 2,
            edge_spacing: 1,
            min_node_width: 8,
            min_node_height: 3,
            padding: 2,
        }
    }
}

/// Perform Dagre-inspired layout on flowchart nodes
fn layout_nodes(database: &FlowchartDatabase) -> Result<FlowchartLayout> {
    let config = LayoutConfig::default();
    let nodes = database.get_nodes();
    let edges = database.get_edges();

    if nodes.is_empty() {
        return Ok(FlowchartLayout {
            nodes: Vec::new(),
            width: 0,
            height: 0,
        });
    }

    // Step 1: Calculate node dimensions based on labels
    let mut node_sizes: std::collections::HashMap<&str, (usize, usize)> =
        std::collections::HashMap::new();
    for (id, label) in &nodes {
        let width = std::cmp::max(config.min_node_width, label.len() + 4); // Add padding
        let height = std::cmp::max(config.min_node_height, 3);
        node_sizes.insert(id, (width, height));
    }

    // Step 2: Assign nodes to ranks (levels)
    let mut ranks: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();

    // Find source nodes (nodes with no incoming edges)
    let source_nodes = database.get_source_nodes();

    // If no source nodes (cycle), use all nodes at rank 0
    let initial_nodes = if source_nodes.is_empty() {
        nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>()
    } else {
        source_nodes
    };

    for node_id in initial_nodes {
        ranks.insert(node_id, 0);
    }

    // Assign ranks using iterative approach to propagate through graph
    // Add iteration limit to prevent infinite loops in cyclic graphs
    let mut changed = true;
    let max_iterations = nodes.len() * 2; // Reasonable limit
    let mut iteration = 0;

    while changed && iteration < max_iterations {
        changed = false;
        for (from, to) in &edges {
            if let Some(&from_rank) = ranks.get(from) {
                let to_rank = ranks.entry(to).or_insert(from_rank + 1);
                if *to_rank <= from_rank {
                    *to_rank = from_rank + 1;
                    changed = true;
                }
            }
        }
        iteration += 1;
    }

    // Step 4: Group nodes by rank and sort within ranks for consistent ordering
    let mut rank_groups: std::collections::HashMap<usize, Vec<&str>> =
        std::collections::HashMap::new();
    for (node_id, &rank) in &ranks {
        rank_groups
            .entry(rank)
            .or_insert_with(Vec::new)
            .push(node_id);
    }

    // Sort nodes within each rank for consistent left-to-right ordering
    for (_, nodes_at_rank) in &mut rank_groups {
        nodes_at_rank.sort();
    }

    // Step 5: Position nodes within each rank for LR layout
    let mut positioned_nodes = Vec::new();
    let mut max_width = 0;
    let mut max_height = 0;

    let max_rank = ranks.values().max().copied().unwrap_or(0);
    for rank in 0..=max_rank {
        if let Some(nodes_at_rank) = rank_groups.get(&rank) {
            // For LR layout, each rank is a vertical column
            let x = rank * (config.min_node_width + config.node_spacing_x) + config.padding;

            for (i, node_id) in nodes_at_rank.iter().enumerate() {
                let (width, height) = node_sizes[node_id];

                // Position nodes vertically within the rank column
                let y = i * (config.min_node_height + config.node_spacing_y) + config.padding;

                positioned_nodes.push(PositionedNode {
                    id: node_id.to_string(),
                    x,
                    y,
                    width,
                    height,
                });

                // Update canvas dimensions
                let node_right = x + width;
                let node_bottom = y + height;
                max_width = std::cmp::max(max_width, node_right);
                max_height = std::cmp::max(max_height, node_bottom);
            }
        }
    }

    // Step 6: Add padding to canvas dimensions
    let canvas_width = max_width + config.padding;
    let canvas_height = max_height + config.padding;

    Ok(FlowchartLayout {
        nodes: positioned_nodes,
        width: canvas_width,
        height: canvas_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Database, LayoutAlgorithm};

    #[test]
    fn test_basic_linear_layout() {
        let mut db = FlowchartDatabase::new();

        // Create linear graph: A --> B --> C
        db.add_node("A", "Start").unwrap();
        db.add_node("B", "Process").unwrap();
        db.add_node("C", "End").unwrap();
        db.add_edge("A", "B").unwrap();
        db.add_edge("B", "C").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.nodes.len(), 3);
        assert!(result.width > 0);
        assert!(result.height > 0);

        // Debug print positions
        println!("Debug - node positions:");
        for node in &result.nodes {
            println!(
                "  {}: x={}, y={}, width={}, height={}",
                node.id, node.x, node.y, node.width, node.height
            );
        }

        // Check that nodes are positioned in order (left to right)
        let node_by_id: std::collections::HashMap<_, _> = result
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();

        // Check that A, B, C are in different ranks (should be for linear graph)
        let a_node = node_by_id["A"];
        let b_node = node_by_id["B"];
        let c_node = node_by_id["C"];

        println!(
            "Debug - y positions: A={}, B={}, C={}",
            a_node.y, b_node.y, c_node.y
        );

        // Check x positions increase (left to right layout)
        assert!(a_node.x < b_node.x, "A.x should be less than B.x");
        assert!(b_node.x < c_node.x, "B.x should be less than C.x");
    }

    #[test]
    fn test_diamond_layout() {
        let mut db = FlowchartDatabase::new();

        // Create diamond graph: A --> B --> D, A --> C --> D
        db.add_node("A", "Start").unwrap();
        db.add_node("B", "Path1").unwrap();
        db.add_node("C", "Path2").unwrap();
        db.add_node("D", "End").unwrap();
        db.add_edge("A", "B").unwrap();
        db.add_edge("A", "C").unwrap();
        db.add_edge("B", "D").unwrap();
        db.add_edge("C", "D").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.nodes.len(), 4);

        // Check rank assignments for LR layout (A at rank 0, B/C at rank 1, D at rank 2)
        let node_by_id: std::collections::HashMap<_, _> = result
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();

        let a_x = node_by_id["A"].x;
        let b_x = node_by_id["B"].x;
        let c_x = node_by_id["C"].x;
        let d_x = node_by_id["D"].x;

        assert!(a_x < b_x); // A should be left of B
        assert!(a_x < c_x); // A should be left of C
        assert_eq!(b_x, c_x); // B and C should be in same column (rank 1)
        assert!(b_x < d_x); // B should be left of D
        assert!(c_x < d_x); // C should be left of D

        // B and C should be stacked vertically in the same column
        let b_y = node_by_id["B"].y;
        let c_y = node_by_id["C"].y;
        assert!(b_y != c_y); // B and C should be at different y positions
    }

    #[test]
    fn test_node_sizing() {
        let mut db = FlowchartDatabase::new();

        db.add_node("A", "Short").unwrap();
        db.add_node("B", "A very long label").unwrap();
        db.add_node("C", "X").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        let node_by_id: std::collections::HashMap<_, _> = result
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();

        // Longer label should result in wider node
        assert!(node_by_id["B"].width > node_by_id["A"].width);
        assert!(node_by_id["B"].width > node_by_id["C"].width);

        // All nodes should have at least minimum width
        assert!(node_by_id["A"].width >= 8);
        assert!(node_by_id["B"].width >= 8);
        assert!(node_by_id["C"].width >= 8);

        // All nodes should have minimum height
        assert!(node_by_id["A"].height >= 3);
        assert!(node_by_id["B"].height >= 3);
        assert!(node_by_id["C"].height >= 3);
    }

    #[test]
    fn test_empty_database() {
        let db = FlowchartDatabase::new();
        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.nodes.len(), 0);
        assert_eq!(result.width, 0);
        assert_eq!(result.height, 0);
    }

    #[test]
    fn test_cyclic_graph_handling() {
        let mut db = FlowchartDatabase::new();

        // Create cycle: A --> B --> C --> A
        db.add_node("A", "Node A").unwrap();
        db.add_node("B", "Node B").unwrap();
        db.add_node("C", "Node C").unwrap();
        db.add_edge("A", "B").unwrap();
        db.add_edge("B", "C").unwrap();
        db.add_edge("C", "A").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        // Should still layout the graph (all at rank 0 due to cycle)
        assert_eq!(result.nodes.len(), 3);
        assert!(result.width > 0);
        assert!(result.height > 0);
    }

    #[test]
    fn test_multiple_nodes_same_rank() {
        let mut db = FlowchartDatabase::new();

        // Create multiple source nodes: A --> C, B --> C
        db.add_node("A", "Source1").unwrap();
        db.add_node("B", "Source2").unwrap();
        db.add_node("C", "Merge").unwrap();
        db.add_edge("A", "C").unwrap();
        db.add_edge("B", "C").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        let node_by_id: std::collections::HashMap<_, _> = result
            .nodes
            .iter()
            .map(|node| (node.id.as_str(), node))
            .collect();

        // A and B should be on the same rank (both source nodes) - same x position in LR layout
        assert_eq!(node_by_id["A"].x, node_by_id["B"].x);

        // C should be to the right of A and B in LR layout
        assert!(node_by_id["C"].x > node_by_id["A"].x);
        assert!(node_by_id["C"].x > node_by_id["B"].x);

        // A and B should be positioned vertically apart in the same column
        assert!(node_by_id["A"].y != node_by_id["B"].y);
    }

    #[test]
    fn test_layout_algorithm_properties() {
        let layout = FlowchartLayoutAlgorithm::new();

        assert_eq!(layout.name(), "grid");
        assert_eq!(layout.version(), "0.1.0");
        assert_eq!(layout.direction(), "LR");
    }
}
