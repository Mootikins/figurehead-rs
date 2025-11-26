//! Flowchart layout implementation
//!
//! Arranges flowchart elements in a coordinate system.

use anyhow::Result;

use crate::core::{LayoutAlgorithm, Database};
use super::FlowchartDatabase;

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
        // TODO: Implement proper Dagre-inspired layout
        // For now, just a simple grid layout to make tests compile

        let nodes = database.get_nodes();
        let mut positioned_nodes = Vec::new();

        let mut x = 0;
        let mut y = 0;
        let max_width = 100;

        for (id, _label) in nodes {
            positioned_nodes.push(PositionedNode {
                id: id.to_string(),
                x,
                y,
                width: 10,
                height: 3,
            });

            x += 15;
            if x > max_width {
                x = 0;
                y += 5;
            }
        }

        Ok(FlowchartLayout {
            nodes: positioned_nodes,
            width: max_width + 15,
            height: y + 5,
        })
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