//! Layered auto-layout engine for FSM diagrams.
//!
//! Arranges states into horizontal layers using a longest-path algorithm,
//! centering each layer within the canvas. Back-edges (transitions where
//! the source layer is below the target layer) are detected and handled
//! separately during rendering.

use std::collections::{HashMap, HashSet};

use crate::diagrams::state::types::{StateNode, StateType, Transition};
use crate::render::widget::{Rect, Size};

/// Layout information for a single state.
#[derive(Debug, Clone)]
pub struct StateLayout {
    pub id: String,
    pub label: String,
    pub state_type: StateType,
    pub rect: Rect,
}

/// Global layout parameters.
pub struct LayoutParams {
    pub gap_x: usize,
    pub gap_y: usize,
    pub left_margin: usize,
    pub top_margin: usize,
    pub min_state_width: usize,
    pub state_height: usize,
    /// Extra height added for accepting states (to fit the double border).
    pub accepting_extra_height: usize,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            gap_x: 6,
            gap_y: 3,
            left_margin: 8,
            top_margin: 2,
            min_state_width: 14,
            state_height: 3,
            accepting_extra_height: 2,
        }
    }
}

/// State size in cells (width, height). Width depends on label length.
fn state_size(label: &str, st: StateType, min_w: usize, base_h: usize, extra_h: usize) -> Size {
    let w = (label.chars().count() + 4).max(min_w);
    let h = match st {
        StateType::Simple => base_h,
        StateType::Accepting => base_h + extra_h,
    };
    Size::new(w, h)
}

/// Compute the layered layout for an FSM diagram.
///
/// Returns a vector of positioned state layouts. The initial pseudo-state
/// indicator is handled separately during rendering.
pub fn layout_states(
    states: &[StateNode],
    transitions: &[Transition],
    initial: Option<&str>,
    canvas_width: usize,
    params: &LayoutParams,
) -> Vec<StateLayout> {
    if states.is_empty() {
        return Vec::new();
    }

    let node_count = states.len();
    let id_to_idx: HashMap<&str, usize> =
        states.iter().enumerate().map(|(i, s)| (s.id.as_str(), i)).collect();
    let idx_to_id: Vec<&str> = states.iter().map(|s| s.id.as_str()).collect();

    // Build adjacency list.
    let mut adj = vec![Vec::new(); node_count];
    for t in transitions {
        if let (Some(&from), Some(&to)) =
            (id_to_idx.get(t.from.as_str()), id_to_idx.get(t.to.as_str()))
        {
            adj[from].push(to);
        }
    }

    // Detect back-edges via DFS.
    let (order, back_edges) = detect_back_edges(node_count, &adj);

    // Assign layers via longest path (ignoring back-edges).
    let layers = assign_layers(node_count, &adj, &order, &back_edges);

    // Group nodes by layer.
    let max_layer = *layers.iter().max().unwrap_or(&0);
    let mut layer_nodes: Vec<Vec<usize>> = vec![Vec::new(); max_layer + 1];
    for (idx, &layer) in layers.iter().enumerate() {
        layer_nodes[layer].push(idx);
    }

    // Position nodes within each layer.
    let sizes: Vec<Size> = states
        .iter()
        .map(|s| {
            state_size(
                &s.label,
                s.state_type,
                params.min_state_width,
                params.state_height,
                params.accepting_extra_height,
            )
        })
        .collect();

    let mut y = params.top_margin;

    // Shift layers if the initial pseudo-state needs room to the left.
    let dx = if initial.is_some() { 5 } else { 0 };

    let mut layouts = Vec::with_capacity(node_count);

    for indices in layer_nodes.iter() {
        if indices.is_empty() {
            continue;
        }
        let max_h = indices.iter().map(|&i| sizes[i].h).max().unwrap_or(params.state_height);
        let total_w: usize =
            indices.iter().map(|&i| sizes[i].w).sum::<usize>() + (indices.len() - 1) * params.gap_x;
        let start_x = canvas_width.saturating_sub(total_w) / 2;

        let mut x = start_x + dx;
        for &idx in indices {
            let size = sizes[idx];
            layouts.push(StateLayout {
                id: idx_to_id[idx].to_string(),
                label: states[idx].label.clone(),
                state_type: states[idx].state_type,
                rect: Rect::new(x, y, size.w, size.h),
            });
            x += size.w + params.gap_x;
        }
        y += max_h + params.gap_y;
    }

    layouts
}

/// Detect back-edges via DFS. Returns a topological order (post-order)
/// and the set of back-edges (node → node pairs that close cycles).
fn detect_back_edges(
    node_count: usize,
    adj: &[Vec<usize>],
) -> (Vec<usize>, HashSet<(usize, usize)>) {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum State {
        Unvisited,
        Visiting,
        Visited,
    }
    let mut states = vec![State::Unvisited; node_count];
    let mut back_edges = HashSet::new();
    let mut order = Vec::new();

    fn dfs(
        node: usize,
        adj: &[Vec<usize>],
        states: &mut Vec<State>,
        back_edges: &mut HashSet<(usize, usize)>,
        order: &mut Vec<usize>,
    ) {
        states[node] = State::Visiting;
        for &next in &adj[node] {
            match states[next] {
                State::Unvisited => dfs(next, adj, states, back_edges, order),
                State::Visiting => {
                    back_edges.insert((node, next));
                }
                State::Visited => {}
            }
        }
        states[node] = State::Visited;
        order.push(node);
    }

    for i in 0..node_count {
        if matches!(states[i], State::Unvisited) {
            dfs(i, adj, &mut states, &mut back_edges, &mut order);
        }
    }
    (order, back_edges)
}

/// Assign each node to a layer using longest path in the DAG (ignoring
/// back-edges). Nodes with no incoming edges are placed at layer 0.
fn assign_layers(
    node_count: usize,
    adj: &[Vec<usize>],
    order: &[usize],
    back_edges: &HashSet<(usize, usize)>,
) -> Vec<usize> {
    let mut layers = vec![0usize; node_count];

    // Process in reverse topological order (bottom-up) to compute
    // longest path distances in the DAG.
    for &node in order.iter().rev() {
        for &next in &adj[node] {
            if back_edges.contains(&(node, next)) {
                continue;
            }
            let candidate = layers[node] + 1;
            if candidate > layers[next] {
                layers[next] = candidate;
            }
        }
    }
    layers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrams::state::types::StateType;

    fn state(id: &str, label: &str) -> StateNode {
        StateNode { id: id.into(), label: label.into(), state_type: StateType::Simple }
    }

    fn accepting(id: &str, label: &str) -> StateNode {
        StateNode { id: id.into(), label: label.into(), state_type: StateType::Accepting }
    }

    #[test]
    fn layout_empty_returns_empty() {
        let params = LayoutParams::default();
        let layouts = layout_states(&[], &[], None, 80, &params);
        assert!(layouts.is_empty());
    }

    #[test]
    fn layout_single_state_is_centered() {
        let params = LayoutParams::default();
        let states = vec![state("a", "A")];
        let layouts = layout_states(&states, &[], None, 80, &params);
        assert_eq!(layouts.len(), 1);
        assert!(layouts[0].rect.x > params.left_margin);
        assert_eq!(layouts[0].rect.h, params.state_height);
    }

    #[test]
    fn layout_two_states_different_layers() {
        let params = LayoutParams::default();
        let states = vec![state("a", "A"), state("b", "B")];
        let transitions = vec![Transition { from: "a".into(), to: "b".into(), label: None }];
        let layouts = layout_states(&states, &transitions, None, 80, &params);
        assert_eq!(layouts.len(), 2);
        // a must be above b (a in layer 0, b in layer 1).
        assert!(layouts[0].rect.y < layouts[1].rect.y);
    }

    #[test]
    fn layout_two_independent_states_same_layer() {
        let params = LayoutParams::default();
        let states = vec![state("a", "A"), state("b", "B")];
        let layouts = layout_states(&states, &[], None, 80, &params);
        assert_eq!(layouts.len(), 2);
        assert_eq!(layouts[0].rect.y, layouts[1].rect.y);
        assert!(layouts[1].rect.x > layouts[0].rect.x);
    }

    #[test]
    fn layout_respects_canvas_width() {
        let params = LayoutParams::default();
        let states = vec![state("a", "A")];
        let layouts = layout_states(&states, &[], None, 40, &params);
        assert_eq!(layouts.len(), 1);
        assert!(layouts[0].rect.right() <= 40);
    }

    #[test]
    fn layout_with_initial_reserves_left_margin() {
        let params = LayoutParams::default();
        let states = vec![state("a", "A")];
        let layouts = layout_states(&states, &[], Some("a"), 80, &params);
        assert_eq!(layouts.len(), 1);
        // With initial, states are shifted right by 5.
        assert!(layouts[0].rect.x >= 5);
    }

    #[test]
    fn layout_self_loop_still_positions_state() {
        let params = LayoutParams::default();
        let states = vec![state("a", "A")];
        let transitions =
            vec![Transition { from: "a".into(), to: "a".into(), label: Some("tick".into()) }];
        let layouts = layout_states(&states, &transitions, None, 80, &params);
        assert_eq!(layouts.len(), 1);
    }

    #[test]
    fn layout_accepting_state_is_taller() {
        let params = LayoutParams::default();
        let layouts = layout_states(&[accepting("done", "Done")], &[], None, 80, &params);
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].rect.h, params.state_height + params.accepting_extra_height);
    }
}
