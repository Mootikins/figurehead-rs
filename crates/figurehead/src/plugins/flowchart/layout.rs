//! Flowchart layout implementation
//!
//! Arranges flowchart elements in a coordinate system using a Sugiyama-style
//! layered layout algorithm.

use anyhow::Result;
use std::collections::HashMap;
use unicode_width::UnicodeWidthStr;

use super::FlowchartDatabase;
use crate::core::{Database, Direction, LayoutAlgorithm, NodeShape};

/// Position data for a laid out node
#[derive(Debug, Clone)]
pub struct PositionedNode {
    pub id: String,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

/// Position data for a laid out edge
#[derive(Debug, Clone)]
pub struct PositionedEdge {
    pub from_id: String,
    pub to_id: String,
    pub waypoints: Vec<(usize, usize)>,
}

/// Layout output containing positioned elements
#[derive(Debug)]
pub struct FlowchartLayoutResult {
    pub nodes: Vec<PositionedNode>,
    pub edges: Vec<PositionedEdge>,
    pub width: usize,
    pub height: usize,
}

/// Layout configuration
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    pub node_sep: usize,
    pub rank_sep: usize,
    pub min_node_width: usize,
    pub min_node_height: usize,
    pub padding: usize,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            node_sep: 4,
            rank_sep: 8,
            min_node_width: 8,
            min_node_height: 3,
            padding: 2,
        }
    }
}

/// Flowchart layout algorithm implementation
pub struct FlowchartLayoutAlgorithm {
    config: LayoutConfig,
}

impl FlowchartLayoutAlgorithm {
    pub fn new() -> Self {
        Self {
            config: LayoutConfig::default(),
        }
    }

    pub fn with_config(config: LayoutConfig) -> Self {
        Self { config }
    }

    /// Calculate node dimensions based on shape and label
    fn calculate_node_size(&self, label: &str, shape: NodeShape) -> (usize, usize) {
        let label_width = UnicodeWidthStr::width(label);

        let (extra_width, extra_height) = match shape {
            NodeShape::Rectangle | NodeShape::RoundedRect | NodeShape::Subroutine => (4, 0),
            NodeShape::Diamond => (6, 2), // Diamonds need more space
            NodeShape::Circle => (4, 0),
            NodeShape::Hexagon => (6, 0),
            NodeShape::Asymmetric | NodeShape::Parallelogram | NodeShape::Trapezoid => (6, 0),
            NodeShape::Cylinder => (6, 2),
        };

        let width = (label_width + extra_width).max(self.config.min_node_width);
        let height = (3 + extra_height).max(self.config.min_node_height);

        (width, height)
    }
}

impl Default for FlowchartLayoutAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutAlgorithm<FlowchartDatabase> for FlowchartLayoutAlgorithm {
    type Output = FlowchartLayoutResult;

    fn layout(&self, database: &FlowchartDatabase) -> Result<Self::Output> {
        let direction = database.direction();

        // Collect nodes and calculate sizes
        let nodes: Vec<_> = database.nodes().collect();
        if nodes.is_empty() {
            return Ok(FlowchartLayoutResult {
                nodes: Vec::new(),
                edges: Vec::new(),
                width: 0,
                height: 0,
            });
        }

        let mut node_sizes: HashMap<&str, (usize, usize)> = HashMap::new();
        for node in &nodes {
            let size = self.calculate_node_size(&node.label, node.shape);
            node_sizes.insert(&node.id, size);
        }

        // Assign layers using topological sort
        let sorted = database.topological_sort();
        let mut layers: HashMap<&str, usize> = HashMap::new();

        for &node_id in &sorted {
            // Layer = max layer of predecessors + 1
            let preds = database.predecessors(node_id);
            let layer = if preds.is_empty() {
                0
            } else {
                preds
                    .iter()
                    .filter_map(|&p| layers.get(p))
                    .max()
                    .map(|&l| l + 1)
                    .unwrap_or(0)
            };
            layers.insert(node_id, layer);
        }

        // Group nodes by layer
        let max_layer = layers.values().max().copied().unwrap_or(0);
        let mut layer_nodes: Vec<Vec<&str>> = vec![Vec::new(); max_layer + 1];
        for (&node_id, &layer) in &layers {
            layer_nodes[layer].push(node_id);
        }

        // Sort nodes within each layer for determinism
        for layer in &mut layer_nodes {
            layer.sort();
        }

        // Calculate positions based on direction
        let mut positioned_nodes = Vec::new();
        let mut max_width = 0;
        let mut max_height = 0;

        match direction {
            Direction::TopDown | Direction::BottomUp => {
                // Vertical layout: layers are rows (Y), nodes distributed on X
                // Find the widest node to establish the center line for alignment
                let widest_node_width = node_sizes.values().map(|(w, _)| *w).max().unwrap_or(0);
                // Center X is at padding + widest_node_width / 2
                let center_x = self.config.padding + widest_node_width / 2;

                let mut y = self.config.padding;

                let layer_iter: Box<dyn Iterator<Item = &Vec<&str>>> = if direction.is_reversed() {
                    Box::new(layer_nodes.iter().rev())
                } else {
                    Box::new(layer_nodes.iter())
                };

                for layer in layer_iter {
                    let mut layer_height = 0;

                    if layer.len() == 1 {
                        // Single node - center it on the center line
                        let node_id = layer[0];
                        let (width, height) = node_sizes[node_id];
                        let x = center_x.saturating_sub(width / 2);
                        positioned_nodes.push(PositionedNode {
                            id: node_id.to_string(),
                            x,
                            y,
                            width,
                            height,
                        });
                        layer_height = height;
                        max_width = max_width.max(x + width + self.config.padding);
                    } else {
                        // Multiple nodes - distribute across from center
                        let total_width: usize = layer.iter()
                            .map(|&id| node_sizes[id].0)
                            .sum::<usize>()
                            + (layer.len() - 1) * self.config.node_sep;
                        let start_x = center_x.saturating_sub(total_width / 2);
                        let mut x = start_x;

                        for &node_id in layer {
                            let (width, height) = node_sizes[node_id];
                            positioned_nodes.push(PositionedNode {
                                id: node_id.to_string(),
                                x,
                                y,
                                width,
                                height,
                            });

                            x += width + self.config.node_sep;
                            layer_height = layer_height.max(height);
                            max_width = max_width.max(x);
                        }
                    }

                    y += layer_height + self.config.rank_sep;
                    max_height = max_height.max(y);
                }
            }
            Direction::LeftRight | Direction::RightLeft => {
                // Horizontal layout: layers are columns (X), nodes distributed on Y
                // First, calculate the maximum height needed for any layer
                let mut layer_max_heights: Vec<usize> = Vec::new();
                for layer in &layer_nodes {
                    let layer_height: usize = layer.iter()
                        .map(|&id| node_sizes[id].1)
                        .sum::<usize>()
                        + layer.len().saturating_sub(1) * self.config.node_sep;
                    layer_max_heights.push(layer_height);
                }
                let total_max_height = *layer_max_heights.iter().max().unwrap_or(&0);

                let mut x = self.config.padding;

                let layer_iter: Box<dyn Iterator<Item = (usize, &Vec<&str>)>> = if direction.is_reversed() {
                    Box::new(layer_nodes.iter().enumerate().rev().map(|(i, l)| (i, l)))
                } else {
                    Box::new(layer_nodes.iter().enumerate().map(|(i, l)| (i, l)))
                };

                for (layer_idx, layer) in layer_iter {
                    // Calculate total height of this layer's nodes
                    let layer_height = layer_max_heights[layer_idx];
                    // Center the layer vertically
                    let start_y = self.config.padding + (total_max_height.saturating_sub(layer_height)) / 2;
                    let mut y = start_y;
                    let mut layer_width = 0;

                    for &node_id in layer {
                        let (width, height) = node_sizes[node_id];
                        positioned_nodes.push(PositionedNode {
                            id: node_id.to_string(),
                            x,
                            y,
                            width,
                            height,
                        });

                        y += height + self.config.node_sep;
                        layer_width = layer_width.max(width);
                        max_height = max_height.max(y);
                    }

                    x += layer_width + self.config.rank_sep;
                    max_width = max_width.max(x);
                }
                // Ensure max_height accounts for the centered layout
                max_height = max_height.max(self.config.padding + total_max_height);
            }
        }

        // Route edges (simple straight-line for now)
        let mut positioned_edges = Vec::new();
        let node_positions: HashMap<&str, &PositionedNode> = positioned_nodes
            .iter()
            .map(|n| (n.id.as_str(), n))
            .collect();

        for edge in database.edges() {
            if let (Some(from), Some(to)) =
                (node_positions.get(edge.from.as_str()), node_positions.get(edge.to.as_str()))
            {
                // Calculate exit and entry points based on direction
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
                });
            }
        }

        Ok(FlowchartLayoutResult {
            nodes: positioned_nodes,
            edges: positioned_edges,
            width: max_width + self.config.padding,
            height: max_height + self.config.padding,
        })
    }

    fn name(&self) -> &'static str {
        "sugiyama"
    }

    fn version(&self) -> &'static str {
        "0.2.0"
    }

    fn direction(&self) -> &'static str {
        "configurable"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_linear_layout_lr() {
        let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);

        db.add_simple_node("A", "Start").unwrap();
        db.add_simple_node("B", "Process").unwrap();
        db.add_simple_node("C", "End").unwrap();
        db.add_simple_edge("A", "B").unwrap();
        db.add_simple_edge("B", "C").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.nodes.len(), 3);
        assert!(result.width > 0);
        assert!(result.height > 0);

        let node_by_id: HashMap<_, _> = result
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), n))
            .collect();

        // LR layout: x should increase left to right
        assert!(node_by_id["A"].x < node_by_id["B"].x);
        assert!(node_by_id["B"].x < node_by_id["C"].x);
    }

    #[test]
    fn test_basic_linear_layout_td() {
        let mut db = FlowchartDatabase::with_direction(Direction::TopDown);

        db.add_simple_node("A", "Start").unwrap();
        db.add_simple_node("B", "Process").unwrap();
        db.add_simple_node("C", "End").unwrap();
        db.add_simple_edge("A", "B").unwrap();
        db.add_simple_edge("B", "C").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        let node_by_id: HashMap<_, _> = result
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), n))
            .collect();

        // TD layout: y should increase top to bottom
        assert!(node_by_id["A"].y < node_by_id["B"].y);
        assert!(node_by_id["B"].y < node_by_id["C"].y);
    }

    #[test]
    fn test_diamond_layout() {
        let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);

        db.add_simple_node("A", "Start").unwrap();
        db.add_simple_node("B", "Path1").unwrap();
        db.add_simple_node("C", "Path2").unwrap();
        db.add_simple_node("D", "End").unwrap();
        db.add_simple_edge("A", "B").unwrap();
        db.add_simple_edge("A", "C").unwrap();
        db.add_simple_edge("B", "D").unwrap();
        db.add_simple_edge("C", "D").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.nodes.len(), 4);

        let node_by_id: HashMap<_, _> = result
            .nodes
            .iter()
            .map(|n| (n.id.as_str(), n))
            .collect();

        // B and C should be in the same column (same x)
        assert_eq!(node_by_id["B"].x, node_by_id["C"].x);
        // B and C should have different y positions
        assert_ne!(node_by_id["B"].y, node_by_id["C"].y);
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
    fn test_edges_are_routed() {
        let mut db = FlowchartDatabase::with_direction(Direction::LeftRight);

        db.add_simple_node("A", "Start").unwrap();
        db.add_simple_node("B", "End").unwrap();
        db.add_simple_edge("A", "B").unwrap();

        let layout = FlowchartLayoutAlgorithm::new();
        let result = layout.layout(&db).unwrap();

        assert_eq!(result.edges.len(), 1);
        assert_eq!(result.edges[0].from_id, "A");
        assert_eq!(result.edges[0].to_id, "B");
        assert!(result.edges[0].waypoints.len() >= 2);
    }
}
