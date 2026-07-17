//! Layout engine for UML state machine diagrams.
//!
//! Computes the position and size of every state in a grid, including
//! nested children inside composite states.

use std::collections::HashMap;

use crate::diagrams::state::types::{StateNode, StateType};
use crate::render::widget::{Rect, Size};

/// Layout information for a single state.
#[derive(Debug, Clone)]
pub struct StateLayout {
    pub id: String,
    pub label: String,
    pub state_type: StateType,
    pub rect: Rect,
    pub children: Vec<StateLayout>,
}

/// Global layout parameters.
pub struct LayoutParams {
    pub gap_x: usize,
    pub gap_y: usize,
    pub left_margin: usize,
    pub top_margin: usize,
    pub child_gap: usize,
    pub child_padding: usize,
    pub min_state_width: usize,
    pub min_simple_width: usize,
    pub simple_height: usize,
}

impl Default for LayoutParams {
    fn default() -> Self {
        Self {
            gap_x: 4,
            gap_y: 2,
            left_margin: 8,
            top_margin: 3,
            child_gap: 2,
            child_padding: 2,
            min_state_width: 6,
            min_simple_width: 14,
            simple_height: 3,
        }
    }
}

/// Compute the layout for a list of top-level states.
pub fn layout_states(states: &[StateNode], params: &LayoutParams) -> Vec<StateLayout> {
    if states.is_empty() {
        return Vec::new();
    }

    let mut measured: Vec<MeasuredState> =
        states.iter().map(|s| measure_state(s, params)).collect();

    // If no explicit grid positions were supplied, assign sequential columns
    // so states are laid out side-by-side by default.
    if measured.iter().all(|m| m.row == 0 && m.col == 0) {
        for (i, m) in measured.iter_mut().enumerate() {
            m.col = i;
        }
    }

    let mut col_widths: HashMap<usize, usize> = HashMap::new();
    let mut row_heights: HashMap<usize, usize> = HashMap::new();
    for m in &measured {
        *col_widths.entry(m.col).or_insert(0) =
            col_widths.get(&m.col).copied().unwrap_or(0).max(m.size.w);
        *row_heights.entry(m.row).or_insert(0) =
            row_heights.get(&m.row).copied().unwrap_or(0).max(m.size.h);
    }

    let max_col = col_widths.keys().copied().max().unwrap_or(0);
    let max_row = row_heights.keys().copied().max().unwrap_or(0);

    let mut col_x: Vec<usize> = Vec::with_capacity(max_col + 1);
    let mut x = params.left_margin;
    for col in 0..=max_col {
        col_x.push(x);
        x += col_widths.get(&col).copied().unwrap_or(0) + params.gap_x;
    }

    let mut row_y: Vec<usize> = Vec::with_capacity(max_row + 1);
    let mut y = params.top_margin;
    for row in 0..=max_row {
        row_y.push(y);
        y += row_heights.get(&row).copied().unwrap_or(0) + params.gap_y;
    }

    measured.into_iter().map(|m| build_layout(m, &col_x, &row_y, params)).collect()
}

/// Internal measured state before final positioning.
struct MeasuredState {
    id: String,
    label: String,
    state_type: StateType,
    row: usize,
    col: usize,
    size: Size,
    children: Vec<MeasuredState>,
}

fn measure_state(state: &StateNode, params: &LayoutParams) -> MeasuredState {
    let (w, h) = match state.state_type {
        StateType::Initial => (1, 1),
        StateType::Final | StateType::History => (3, 3),
        StateType::Simple => {
            let w = state.label.len() + 4;
            (w.max(params.min_simple_width), params.simple_height)
        }
        StateType::Composite => measure_composite(state, params),
    };

    MeasuredState {
        id: state.id.clone(),
        label: state.label.clone(),
        state_type: state.state_type,
        row: state.row,
        col: state.col,
        size: Size::new(w, h),
        children: state.children.iter().map(|c| measure_state(c, params)).collect(),
    }
}

fn measure_composite(state: &StateNode, params: &LayoutParams) -> (usize, usize) {
    let title_w = state.label.len() + 4;
    let header_h = 3; // title row + separator + top padding

    let child_sizes: Vec<Size> =
        state.children.iter().map(|c| measure_state(c, params).size).collect();

    let total_child_w: usize = child_sizes.iter().map(|s| s.w).sum();
    let total_gap = (state.children.len().saturating_sub(1)) * params.child_gap;
    let inner_w = total_child_w + total_gap + params.child_padding * 2;
    let inner_h = child_sizes.iter().map(|s| s.h).max().unwrap_or(params.simple_height);

    let w = title_w.max(inner_w).max(params.min_state_width);
    let h = header_h + inner_h + params.child_padding;
    (w, h)
}

fn build_layout(
    state: MeasuredState,
    col_x: &[usize],
    row_y: &[usize],
    params: &LayoutParams,
) -> StateLayout {
    let x = col_x.get(state.col).copied().unwrap_or(params.left_margin);
    let y = row_y.get(state.row).copied().unwrap_or(params.top_margin);
    let children = layout_children(state.children, x, y, state.size.w, state.size.h, params);

    StateLayout {
        id: state.id,
        label: state.label,
        state_type: state.state_type,
        rect: Rect::new(x, y, state.size.w, state.size.h),
        children,
    }
}

fn layout_children(
    children: Vec<MeasuredState>,
    parent_x: usize,
    parent_y: usize,
    parent_w: usize,
    parent_h: usize,
    params: &LayoutParams,
) -> Vec<StateLayout> {
    if children.is_empty() {
        return Vec::new();
    }

    let total_child_w: usize = children.iter().map(|c| c.size.w).sum();
    let total_gap = (children.len().saturating_sub(1)) * params.child_gap;
    let content_w = total_child_w + total_gap;
    let available_w = parent_w.saturating_sub(params.child_padding * 2);
    let start_x = params.child_padding + available_w.saturating_sub(content_w) / 2;

    let max_child_h = children.iter().map(|c| c.size.h).max().unwrap_or(params.simple_height);
    let start_y = parent_h.saturating_sub(max_child_h + 1);

    let mut layouts = Vec::with_capacity(children.len());
    let mut x = parent_x + start_x;
    for child in children {
        let y = parent_y + start_y;
        let w = child.size.w;
        let h = child.size.h;
        let nested = layout_children(child.children, x, y, w, h, params);
        layouts.push(StateLayout {
            id: child.id,
            label: child.label,
            state_type: child.state_type,
            rect: Rect::new(x, y, w, h),
            children: nested,
        });
        x += w + params.child_gap;
    }

    layouts
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple(id: &str, label: &str) -> StateNode {
        StateNode {
            id: id.into(),
            label: label.into(),
            state_type: StateType::Simple,
            row: 0,
            col: 0,
            children: vec![],
        }
    }

    fn composite(id: &str, label: &str, children: Vec<StateNode>) -> StateNode {
        StateNode {
            id: id.into(),
            label: label.into(),
            state_type: StateType::Composite,
            row: 0,
            col: 0,
            children,
        }
    }

    #[test]
    fn layout_empty_returns_empty() {
        let params = LayoutParams::default();
        let layouts = layout_states(&[], &params);
        assert!(layouts.is_empty());
    }

    #[test]
    fn layout_simple_state_uses_min_width() {
        let params = LayoutParams::default();
        let layouts = layout_states(&[simple("a", "A")], &params);
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].rect.w, params.min_simple_width);
        assert_eq!(layouts[0].rect.h, params.simple_height);
    }

    #[test]
    fn layout_simple_states_are_side_by_side() {
        let params = LayoutParams::default();
        let layouts = layout_states(&[simple("a", "A"), simple("b", "B")], &params);
        assert_eq!(layouts.len(), 2);
        assert!(layouts[1].rect.x > layouts[0].rect.x);
        assert_eq!(layouts[0].rect.y, layouts[1].rect.y);
    }

    #[test]
    fn layout_composite_state_contains_children() {
        let params = LayoutParams::default();
        let children = vec![simple("sub1", "Sub1"), simple("sub2", "Sub2")];
        let layouts = layout_states(&[composite("active", "Active", children)], &params);
        assert_eq!(layouts.len(), 1);

        let active = &layouts[0];
        assert_eq!(active.children.len(), 2);
        for child in &active.children {
            assert!(child.rect.x >= active.rect.x);
            assert!(child.rect.right() <= active.rect.right());
            assert!(child.rect.y >= active.rect.y);
            assert!(child.rect.bottom() <= active.rect.bottom());
        }
    }

    #[test]
    fn layout_nested_composite_state_contains_grandchildren() {
        let params = LayoutParams::default();
        let inner = composite("inner", "Inner", vec![simple("g1", "G1"), simple("g2", "G2")]);
        let layouts = layout_states(&[composite("outer", "Outer", vec![inner])], &params);
        assert_eq!(layouts.len(), 1);

        let outer = &layouts[0];
        assert_eq!(outer.children.len(), 1);
        let inner_layout = &outer.children[0];
        assert_eq!(inner_layout.children.len(), 2);
        for grandchild in &inner_layout.children {
            assert!(grandchild.rect.x >= inner_layout.rect.x);
            assert!(grandchild.rect.right() <= inner_layout.rect.right());
        }
    }

    #[test]
    fn layout_history_state_matches_final_size() {
        let params = LayoutParams::default();
        let history = StateNode {
            id: "hist".into(),
            label: "H".into(),
            state_type: StateType::History,
            row: 0,
            col: 0,
            children: vec![],
        };
        let final_state = StateNode {
            id: "done".into(),
            label: "Done".into(),
            state_type: StateType::Final,
            row: 0,
            col: 0,
            children: vec![],
        };
        let layouts = layout_states(&[history, final_state], &params);
        assert_eq!(layouts.len(), 2);
        assert_eq!(layouts[0].rect.w, layouts[1].rect.w);
        assert_eq!(layouts[0].rect.h, layouts[1].rect.h);
    }

    #[test]
    fn layout_initial_state_has_unit_size() {
        let params = LayoutParams::default();
        let initial = StateNode {
            id: "init".into(),
            label: "init".into(),
            state_type: StateType::Initial,
            row: 0,
            col: 0,
            children: vec![],
        };
        let layouts = layout_states(&[initial], &params);
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].rect.w, 1);
        assert_eq!(layouts[0].rect.h, 1);
    }

    #[test]
    fn layout_long_label_simple_state_expands_width() {
        let params = LayoutParams::default();
        let layouts = layout_states(&[simple("long", "VeryLongLabel")], &params);
        assert_eq!(layouts.len(), 1);
        assert!(layouts[0].rect.w > params.min_simple_width);
    }

    #[test]
    fn layout_composite_state_expands_to_fit_children() {
        let params = LayoutParams::default();
        let children = vec![simple("sub1", "Sub1"), simple("sub2", "Sub2")];
        let layouts = layout_states(&[composite("active", "Active", children)], &params);
        assert_eq!(layouts.len(), 1);
        let active = &layouts[0];
        let expected_min_w = "Active".len() + 4;
        let children_w: usize = active.children.iter().map(|c| c.rect.w).sum();
        let gaps = active.children.len().saturating_sub(1) * params.child_gap;
        assert!(active.rect.w >= expected_min_w);
        assert!(active.rect.w >= children_w + gaps + params.child_padding * 2);
    }

    #[test]
    fn layout_children_are_spaced_apart() {
        let params = LayoutParams::default();
        let children = vec![simple("sub1", "Sub1"), simple("sub2", "Sub2")];
        let layouts = layout_states(&[composite("active", "Active", children)], &params);
        assert_eq!(layouts.len(), 1);
        let active = &layouts[0];
        assert_eq!(active.children.len(), 2);
        let gap = active.children[1].rect.x - active.children[0].rect.right();
        assert_eq!(gap, params.child_gap);
    }

    #[test]
    fn layout_respects_explicit_grid_positions() {
        let params = LayoutParams::default();
        let mut second_row = simple("b", "B");
        second_row.row = 1;
        let mut second_col = simple("c", "C");
        second_col.col = 1;
        let layouts = layout_states(&[simple("a", "A"), second_row, second_col], &params);
        assert_eq!(layouts.len(), 3);
        let b = layouts.iter().find(|l| l.id == "b").unwrap();
        let c = layouts.iter().find(|l| l.id == "c").unwrap();
        assert!(b.rect.y > layouts[0].rect.y);
        assert!(c.rect.x > layouts[0].rect.x);
    }
}
