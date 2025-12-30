//! State diagram layout algorithm
//!
//! Positions states and transitions for rendering.

use super::database::StateDatabase;
use crate::core::{LayoutAlgorithm, NodeShape};
use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};

/// Positioned state for rendering
#[derive(Debug, Clone)]
pub struct PositionedState {
    pub id: String,
    pub label: String,
    pub shape: NodeShape,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
    pub rank: usize,
}

/// Positioned transition for rendering
#[derive(Debug, Clone)]
pub struct PositionedTransition {
    pub from_id: String,
    pub to_id: String,
    pub label: Option<String>,
    pub from_x: usize,
    pub from_y: usize,
    pub to_x: usize,
    pub to_y: usize,
}

/// Layout result containing positioned elements
#[derive(Debug, Clone)]
pub struct StateLayoutResult {
    pub states: Vec<PositionedState>,
    pub transitions: Vec<PositionedTransition>,
    pub width: usize,
    pub height: usize,
}

/// State diagram layout algorithm
pub struct StateLayoutAlgorithm {
    /// Minimum state box width
    min_state_width: usize,
    /// State box height (for normal states)
    state_height: usize,
    /// Terminal state size
    terminal_size: usize,
    /// Horizontal spacing between states
    h_spacing: usize,
    /// Vertical spacing between ranks
    v_spacing: usize,
    /// Padding around labels
    padding: usize,
}

impl StateLayoutAlgorithm {
    pub fn new() -> Self {
        Self {
            min_state_width: 8,
            state_height: 3,
            terminal_size: 3,
            h_spacing: 4,
            v_spacing: 3,
            padding: 2,
        }
    }

    /// Assign ranks to states using BFS from start state
    fn assign_ranks(&self, db: &StateDatabase) -> HashMap<String, usize> {
        let mut ranks: HashMap<String, usize> = HashMap::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut queue: VecDeque<(String, usize)> = VecDeque::new();

        // Find start states ([*] that only have outgoing edges)
        let mut start_states: Vec<String> = Vec::new();

        for state in db.states() {
            if state.id == "[*]" {
                // Check if this [*] is a start (has outgoing but check first occurrence)
                let has_outgoing = db.transitions().iter().any(|t| t.from == state.id);
                if has_outgoing {
                    start_states.push(state.id.clone());
                    break; // Only need one start
                }
            }
        }

        // If no [*] start, use first state
        if start_states.is_empty() {
            if let Some(first) = db.states().first() {
                start_states.push(first.id.clone());
            }
        }

        // BFS from start states
        for start in start_states {
            if !visited.contains(&start) {
                queue.push_back((start, 0));
            }

            while let Some((state_id, rank)) = queue.pop_front() {
                if visited.contains(&state_id) {
                    continue;
                }
                visited.insert(state_id.clone());
                ranks.insert(state_id.clone(), rank);

                // Add all states reachable from this one
                for edge in db.transitions() {
                    if edge.from == state_id && !visited.contains(&edge.to) {
                        queue.push_back((edge.to.clone(), rank + 1));
                    }
                }
            }
        }

        // Handle any unvisited states (disconnected)
        let max_rank = ranks.values().copied().max().unwrap_or(0);
        for state in db.states() {
            if !ranks.contains_key(&state.id) {
                ranks.insert(state.id.clone(), max_rank + 1);
            }
        }

        ranks
    }

    /// Calculate state dimensions
    fn calculate_state_size(&self, label: &str, shape: NodeShape) -> (usize, usize) {
        match shape {
            NodeShape::Terminal => (self.terminal_size, self.terminal_size),
            _ => {
                let label_width = label.chars().count();
                let width = (label_width + self.padding * 2).max(self.min_state_width);
                (width, self.state_height)
            }
        }
    }

    /// Layout the database
    pub fn layout(&self, db: &StateDatabase) -> Result<StateLayoutResult> {
        if db.state_count() == 0 {
            return Ok(StateLayoutResult {
                states: vec![],
                transitions: vec![],
                width: 0,
                height: 0,
            });
        }

        let ranks = self.assign_ranks(db);

        // Group states by rank
        let mut by_rank: HashMap<usize, Vec<&crate::core::NodeData>> = HashMap::new();
        for state in db.states() {
            let rank = ranks.get(&state.id).copied().unwrap_or(0);
            by_rank.entry(rank).or_default().push(state);
        }

        let max_rank = *ranks.values().max().unwrap_or(&0);

        // Calculate positions
        let mut positioned_states: Vec<PositionedState> = Vec::new();
        let mut state_positions: HashMap<String, (usize, usize, usize, usize)> = HashMap::new();

        let mut current_y = 0;

        for rank in 0..=max_rank {
            let states_in_rank = by_rank.get(&rank).map(|v| v.as_slice()).unwrap_or(&[]);

            if states_in_rank.is_empty() {
                continue;
            }

            // Calculate widths for this rank
            let mut max_height = 0;
            let mut state_dims: Vec<(usize, usize)> = Vec::new();

            for state in states_in_rank {
                let (w, h) = self.calculate_state_size(&state.label, state.shape);
                state_dims.push((w, h));
                max_height = max_height.max(h);
            }

            // Position states in this rank (centered)
            let mut current_x = 0;
            for (i, state) in states_in_rank.iter().enumerate() {
                let (w, h) = state_dims[i];

                // Center vertically within the row
                let y_offset = (max_height - h) / 2;

                let pos_state = PositionedState {
                    id: state.id.clone(),
                    label: state.label.clone(),
                    shape: state.shape,
                    x: current_x,
                    y: current_y + y_offset,
                    width: w,
                    height: h,
                    rank,
                };

                state_positions.insert(
                    state.id.clone(),
                    (current_x, current_y + y_offset, w, h),
                );
                positioned_states.push(pos_state);

                current_x += w + self.h_spacing;
            }

            current_y += max_height + self.v_spacing;
        }

        // Position transitions
        let mut positioned_transitions: Vec<PositionedTransition> = Vec::new();

        for edge in db.transitions() {
            if let (Some(&(fx, fy, fw, fh)), Some(&(tx, ty, tw, _th))) = (
                state_positions.get(&edge.from),
                state_positions.get(&edge.to),
            ) {
                // Connect from center-bottom of source to center-top of target
                let from_x = fx + fw / 2;
                let from_y = fy + fh;
                let to_x = tx + tw / 2;
                let to_y = ty;

                positioned_transitions.push(PositionedTransition {
                    from_id: edge.from.clone(),
                    to_id: edge.to.clone(),
                    label: edge.label.clone(),
                    from_x,
                    from_y,
                    to_x,
                    to_y,
                });
            }
        }

        // Calculate total dimensions
        let width = positioned_states
            .iter()
            .map(|s| s.x + s.width)
            .max()
            .unwrap_or(0);
        let height = positioned_states
            .iter()
            .map(|s| s.y + s.height)
            .max()
            .unwrap_or(0);

        Ok(StateLayoutResult {
            states: positioned_states,
            transitions: positioned_transitions,
            width,
            height,
        })
    }
}

impl Default for StateLayoutAlgorithm {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutAlgorithm<StateDatabase> for StateLayoutAlgorithm {
    type Output = StateLayoutResult;

    fn layout(&self, database: &StateDatabase) -> Result<Self::Output> {
        self.layout(database)
    }

    fn name(&self) -> &'static str {
        "state"
    }

    fn version(&self) -> &'static str {
        "0.1.0"
    }

    fn direction(&self) -> &'static str {
        "TB"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::EdgeData;

    #[test]
    fn test_empty_layout() {
        let db = StateDatabase::new();
        let algo = StateLayoutAlgorithm::new();
        let result = algo.layout(&db).unwrap();

        assert!(result.states.is_empty());
        assert!(result.transitions.is_empty());
    }

    #[test]
    fn test_single_state_layout() {
        let mut db = StateDatabase::new();
        db.add_state(crate::core::NodeData::new("Idle", "Idle"))
            .unwrap();

        let algo = StateLayoutAlgorithm::new();
        let result = algo.layout(&db).unwrap();

        assert_eq!(result.states.len(), 1);
        assert_eq!(result.states[0].id, "Idle");
    }

    #[test]
    fn test_linear_layout() {
        let mut db = StateDatabase::new();
        db.add_transition(EdgeData::new("[*]", "Idle")).unwrap();
        db.add_transition(EdgeData::new("Idle", "Running")).unwrap();
        db.add_transition(EdgeData::new("Running", "[*]")).unwrap();

        let algo = StateLayoutAlgorithm::new();
        let result = algo.layout(&db).unwrap();

        // Should have 3 states (2x [*] collapsed + Idle + Running)
        assert!(result.states.len() >= 3);

        // States should be in different ranks (y positions)
        let y_positions: Vec<usize> = result.states.iter().map(|s| s.y).collect();
        // Multiple y positions means vertical layout
        assert!(y_positions.iter().collect::<HashSet<_>>().len() > 1);
    }

    #[test]
    fn test_terminal_state_size() {
        let algo = StateLayoutAlgorithm::new();
        let (w, h) = algo.calculate_state_size("", NodeShape::Terminal);
        assert_eq!(w, algo.terminal_size);
        assert_eq!(h, algo.terminal_size);
    }
}
