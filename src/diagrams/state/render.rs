//! UML state machine diagram renderer.

use std::collections::HashMap;
use std::fmt;

use crate::canvas::{Canvas, Layer};
use crate::diagrams::state::layout::{LayoutParams, StateLayout, layout_states};
use crate::diagrams::state::types::{StateNode, StateType, Transition};
use crate::error::{FigoError, Result};
use crate::render::node::Node;
use crate::render::surface::Surface;
use crate::render::widget::{LayoutContext, MeasureContext, PaintContext, Rect, Widget};
use crate::style::{BorderStyle, Charset, LineStyle};

/// Builder for state machine diagrams.
pub struct StateDiagram<'a> {
    width: usize,
    charset: Charset,
    states: Vec<StateNode>,
    initial: Option<&'a str>,
    transitions: Vec<Transition>,
    color: bool,
}

impl<'a> StateDiagram<'a> {
    /// Create a new state diagram builder.
    pub fn new(width: usize, charset: Charset) -> Self {
        Self {
            width,
            charset,
            states: Vec::new(),
            initial: None,
            transitions: Vec::new(),
            color: false,
        }
    }

    /// Add a state.
    pub fn add_state(mut self, state: StateNode) -> Self {
        self.states.push(state);
        self
    }

    /// Set the initial state.
    pub fn initial(mut self, state_id: &'a str) -> Self {
        self.initial = Some(state_id);
        self
    }

    /// Add a transition.
    pub fn add_transition(mut self, from: &str, to: &str, label: Option<&str>) -> Self {
        self.transitions.push(Transition {
            from: from.to_string(),
            to: to.to_string(),
            label: label.map(String::from),
        });
        self
    }

    /// Enable or disable color output.
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Render and return as a `String`.
    pub fn build(&self) -> Result<String> {
        if self.states.is_empty() {
            return Err(FigoError::MissingFields("no states specified".into()));
        }

        let params = LayoutParams::default();
        let mut layouts = layout_states(&self.states, &params);
        let child_to_parent = build_parent_map(&layouts);
        let label_rows = compute_label_rows(&self.transitions, &layouts, &child_to_parent);
        let max_label_row = label_rows.values().copied().max().unwrap_or(0);
        if max_label_row > 0 {
            shift_layouts(&mut layouts, max_label_row + 1);
        }
        let id_to_layout = build_id_map(&layouts);
        let child_to_parent = build_parent_map(&layouts);

        let total_w = compute_canvas_width(&layouts, &params).max(self.width);
        let total_h = compute_canvas_height(&layouts, &params);
        let mut canvas = Canvas::new(total_w, total_h);

        {
            let mut surface = Surface::new(&mut canvas);
            let ctx = PaintContext { charset: self.charset, color: self.color };

            draw_initial_pseudostate(&mut surface, &id_to_layout, self.initial, self.charset);
            draw_states(&mut surface, &layouts, &ctx)?;
            draw_transitions(
                &mut surface,
                &self.transitions,
                &id_to_layout,
                &child_to_parent,
                &label_rows,
                &ctx,
            );
        }

        canvas.repair_connector_junctions(LineStyle::Simple, self.charset);
        Ok(canvas.render(self.color))
    }

    pub fn render(&self) -> Result<String> {
        self.build()
    }
}

impl fmt::Display for StateDiagram<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build() {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "[figo error: {e}]"),
        }
    }
}

fn build_id_map(layouts: &[StateLayout]) -> HashMap<String, StateLayoutRef> {
    let mut map = HashMap::new();
    for layout in layouts {
        insert_layout(layout, &mut map);
    }
    map
}

fn insert_layout(layout: &StateLayout, map: &mut HashMap<String, StateLayoutRef>) {
    map.insert(layout.id.clone(), StateLayoutRef { rect: layout.rect });
    for child in &layout.children {
        insert_layout(child, map);
    }
}

fn build_parent_map(layouts: &[StateLayout]) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for layout in layouts {
        for child in &layout.children {
            map.insert(child.id.clone(), layout.id.clone());
        }
    }
    map
}

#[derive(Clone, Copy)]
struct StateLayoutRef {
    rect: Rect,
}

fn compute_canvas_width(layouts: &[StateLayout], params: &LayoutParams) -> usize {
    let rightmost = layouts.iter().map(|l| l.rect.right()).max().unwrap_or(0);
    rightmost + params.gap_x
}

fn compute_canvas_height(layouts: &[StateLayout], params: &LayoutParams) -> usize {
    let bottommost = layouts.iter().map(|l| l.rect.bottom()).max().unwrap_or(0);
    bottommost + params.gap_y * 2
}

fn shift_layouts(layouts: &mut [StateLayout], dy: usize) {
    for layout in layouts.iter_mut() {
        layout.rect.y += dy;
        shift_layouts(&mut layout.children, dy);
    }
}

#[derive(Debug)]
struct LabelInfo {
    transition_index: usize,
    x: usize,
    width: usize,
    row: usize,
}

fn compute_label_rows(
    transitions: &[Transition],
    layouts: &[StateLayout],
    child_to_parent: &HashMap<String, String>,
) -> HashMap<usize, usize> {
    let id_to_layout = build_id_map(layouts);
    let mut labels: Vec<LabelInfo> = Vec::new();

    for (idx, transition) in transitions.iter().enumerate() {
        if transition.from == transition.to {
            continue;
        }
        if is_internal_transition(transition, child_to_parent) {
            continue;
        }
        let Some(from) = id_to_layout.get(&transition.from) else { continue };
        let Some(to) = id_to_layout.get(&transition.to) else { continue };
        let Some(text) = transition.label.as_ref() else { continue };

        let from_cx = from.rect.x + from.rect.w / 2;
        let to_cx = to.rect.x + to.rect.w / 2;
        let label_x = (from_cx + to_cx) / 2;
        let label_x = label_x.saturating_sub(text.chars().count() / 2);
        labels.push(LabelInfo {
            transition_index: idx,
            x: label_x,
            width: text.chars().count(),
            row: 0,
        });
    }

    if labels.is_empty() {
        return HashMap::new();
    }

    let mut indices: Vec<usize> = (0..labels.len()).collect();
    indices.sort_by(|a, b| labels[*a].x.cmp(&labels[*b].x));

    let mut rows: Vec<Vec<usize>> = Vec::new();
    for orig_idx in indices {
        let label = &labels[orig_idx];
        let mut placed = false;
        for (row_idx, row) in rows.iter_mut().enumerate() {
            let overlaps = row.iter().any(|&other_idx| {
                let other = &labels[other_idx];
                label.x < other.x + other.width && label.x + label.width > other.x
            });
            if !overlaps {
                labels[orig_idx].row = row_idx;
                row.push(orig_idx);
                placed = true;
                break;
            }
        }
        if !placed {
            let row_idx = rows.len();
            rows.push(vec![orig_idx]);
            labels[orig_idx].row = row_idx;
        }
    }

    labels.into_iter().map(|l| (l.transition_index, l.row)).collect()
}

fn draw_initial_pseudostate(
    surface: &mut Surface<'_>,
    id_to_layout: &HashMap<String, StateLayoutRef>,
    initial: Option<&str>,
    charset: Charset,
) {
    let Some(init_id) = initial else { return };
    let Some(layout) = id_to_layout.get(init_id) else { return };

    let init_x = layout.rect.x.saturating_sub(6);
    let init_y = layout.rect.y + 1;
    let dot = if charset == Charset::Ascii { '*' } else { '●' };
    surface.put(init_x, init_y, dot);
    for dx in 1..5 {
        surface.put(init_x + dx, init_y, '─');
    }
    surface.put(init_x + 5, init_y, '>');
}

fn draw_states(
    surface: &mut Surface<'_>,
    layouts: &[StateLayout],
    ctx: &PaintContext,
) -> Result<()> {
    let measure_ctx = MeasureContext { charset: ctx.charset };
    let mut layout_ctx = LayoutContext { charset: ctx.charset, bounds: Rect::default() };

    for layout in layouts {
        draw_state(surface, layout, ctx, &measure_ctx, &mut layout_ctx)?;
    }
    Ok(())
}

fn draw_state(
    surface: &mut Surface<'_>,
    layout: &StateLayout,
    ctx: &PaintContext,
    measure_ctx: &MeasureContext,
    layout_ctx: &mut LayoutContext,
) -> Result<()> {
    match layout.state_type {
        StateType::Initial => {
            let dot = if ctx.charset == Charset::Ascii { '*' } else { '●' };
            surface.put(layout.rect.x, layout.rect.y, dot);
        }
        StateType::Final => draw_final_state(surface, layout, ctx, measure_ctx, layout_ctx),
        StateType::History => draw_history_state(surface, layout, ctx, measure_ctx, layout_ctx),
        StateType::Simple => draw_simple_state(surface, layout, ctx, measure_ctx, layout_ctx),
        StateType::Composite => {
            draw_composite_state(surface, layout, ctx, measure_ctx, layout_ctx)?
        }
    }
    Ok(())
}

fn draw_simple_state(
    surface: &mut Surface<'_>,
    layout: &StateLayout,
    ctx: &PaintContext,
    measure_ctx: &MeasureContext,
    layout_ctx: &mut LayoutContext,
) {
    let mut node = Node::new(ctx.charset)
        .border(BorderStyle::Rounded)
        .content(vec![layout.label.clone()])
        .align(crate::style::HAlign::Center, crate::style::VAlign::Middle);
    node.measure(measure_ctx);
    node.layout(layout_ctx, layout.rect);
    node.paint(ctx, surface);
}

fn draw_final_state(
    surface: &mut Surface<'_>,
    layout: &StateLayout,
    ctx: &PaintContext,
    measure_ctx: &MeasureContext,
    layout_ctx: &mut LayoutContext,
) {
    let mut node = Node::new(ctx.charset)
        .border(BorderStyle::Rounded)
        .align(crate::style::HAlign::Center, crate::style::VAlign::Middle);
    node.measure(measure_ctx);
    node.layout(layout_ctx, layout.rect);
    node.paint(ctx, surface);

    let (cx, cy) = layout.rect.center();
    let symbol = if ctx.charset == Charset::Ascii { 'O' } else { '◎' };
    surface.put_layered(cx, cy, symbol, Layer::NodeContent);
}

fn draw_history_state(
    surface: &mut Surface<'_>,
    layout: &StateLayout,
    ctx: &PaintContext,
    measure_ctx: &MeasureContext,
    layout_ctx: &mut LayoutContext,
) {
    let mut node = Node::new(ctx.charset)
        .border(BorderStyle::Rounded)
        .align(crate::style::HAlign::Center, crate::style::VAlign::Middle);
    node.measure(measure_ctx);
    node.layout(layout_ctx, layout.rect);
    node.paint(ctx, surface);

    let (cx, cy) = layout.rect.center();
    surface.put_layered(cx, cy, 'H', Layer::NodeContent);
}

fn draw_composite_state(
    surface: &mut Surface<'_>,
    layout: &StateLayout,
    ctx: &PaintContext,
    measure_ctx: &MeasureContext,
    layout_ctx: &mut LayoutContext,
) -> Result<()> {
    let mut outer = Node::new(ctx.charset)
        .border(BorderStyle::Rounded)
        .title(layout.label.clone())
        .align(crate::style::HAlign::Center, crate::style::VAlign::Top);
    outer.measure(measure_ctx);
    outer.layout(layout_ctx, layout.rect);
    outer.paint(ctx, surface);

    // Separator between title and children.
    let sep_y = layout.rect.y + 2;
    let glyphs = BorderStyle::Rounded.glyphs(ctx.charset);
    surface.put_layered(layout.rect.x, sep_y, glyphs.tee_right, Layer::NodeBorder);
    surface.put_layered(
        layout.rect.x + layout.rect.w - 1,
        sep_y,
        glyphs.tee_left,
        Layer::NodeBorder,
    );
    surface.put_horizontal(
        layout.rect.x + 1,
        sep_y,
        layout.rect.w - 2,
        glyphs.horizontal,
        Layer::NodeBorder,
    );

    for child in &layout.children {
        draw_state(surface, child, ctx, measure_ctx, layout_ctx)?;
    }
    Ok(())
}

fn draw_transitions(
    surface: &mut Surface<'_>,
    transitions: &[Transition],
    id_to_layout: &HashMap<String, StateLayoutRef>,
    child_to_parent: &HashMap<String, String>,
    label_rows: &HashMap<usize, usize>,
    ctx: &PaintContext,
) {
    for (idx, transition) in transitions.iter().enumerate() {
        let Some(from) = id_to_layout.get(&transition.from) else { continue };
        let Some(to) = id_to_layout.get(&transition.to) else { continue };

        if transition.from == transition.to {
            draw_self_loop(surface, from.rect, ctx);
            continue;
        }

        let is_internal = is_internal_transition(transition, child_to_parent);
        if is_internal {
            draw_internal_transition(surface, from.rect, to.rect, transition.label.as_deref(), ctx);
        } else {
            let row = label_rows.get(&idx).copied().unwrap_or(0);
            draw_external_transition(
                surface,
                from.rect,
                to.rect,
                transition.label.as_deref(),
                row,
                ctx,
            );
        }
    }
}

fn is_internal_transition(
    transition: &Transition,
    child_to_parent: &HashMap<String, String>,
) -> bool {
    match (child_to_parent.get(&transition.from), child_to_parent.get(&transition.to)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

fn draw_external_transition(
    surface: &mut Surface<'_>,
    from: Rect,
    to: Rect,
    label: Option<&str>,
    row: usize,
    ctx: &PaintContext,
) {
    let glyphs = BorderStyle::Single.glyphs(ctx.charset);
    let from_cx = from.x + from.w / 2;
    let to_cx = to.x + to.w / 2;
    let route_y = from.y.saturating_sub(1);

    if from.y > route_y {
        surface.put_vertical(from_cx, route_y, from.y - route_y, glyphs.vertical, Layer::Connector);
    }
    if to.y > route_y {
        surface.put_vertical(to_cx, route_y, to.y - route_y, glyphs.vertical, Layer::Connector);
    }

    let (left_x, right_x) = if from_cx < to_cx { (from_cx, to_cx) } else { (to_cx, from_cx) };
    if right_x > left_x {
        surface.put_horizontal(
            left_x,
            route_y,
            right_x - left_x + 1,
            glyphs.horizontal,
            Layer::Connector,
        );
    }

    let arrow = if ctx.charset == Charset::Ascii { 'v' } else { '▼' };
    surface.put_layered(to_cx, to.y, arrow, Layer::ConnectorEnd);

    if let Some(text) = label {
        let label_x = (from_cx + to_cx) / 2;
        let label_x = label_x.saturating_sub(text.chars().count() / 2);
        let label_y = route_y.saturating_sub(1 + row * 2);
        surface.put_str_layered(label_x, label_y, text, Layer::Label);
    }
}

fn draw_internal_transition(
    surface: &mut Surface<'_>,
    from: Rect,
    to: Rect,
    label: Option<&str>,
    ctx: &PaintContext,
) {
    let glyphs = BorderStyle::Single.glyphs(ctx.charset);
    let cy = from.y + from.h / 2;

    let (left_x, right_x, arrow_x, arrow_ch) = if to.x > from.x + from.w {
        (from.x + from.w, to.x, to.x, '>')
    } else if from.x > to.x + to.w {
        (to.x + to.w, from.x, from.x, '<')
    } else {
        (from.x + from.w, to.x, to.x, '>')
    };

    if right_x > left_x {
        surface.put_horizontal(left_x, cy, right_x - left_x, glyphs.horizontal, Layer::Connector);
    }
    surface.put_layered(arrow_x, cy, arrow_ch, Layer::ConnectorEnd);

    if let Some(text) = label {
        let label_x = (left_x + right_x) / 2;
        let label_x = label_x.saturating_sub(text.chars().count() / 2);
        surface.put_str_layered(label_x, cy.saturating_sub(2), text, Layer::Label);
    }
}

fn draw_self_loop(surface: &mut Surface<'_>, rect: Rect, ctx: &PaintContext) {
    let glyphs = BorderStyle::Single.glyphs(ctx.charset);
    let loop_x = rect.x + rect.w + 1;
    let top = rect.y;
    let mid = rect.y + rect.h / 2;
    let bot = rect.y + rect.h - 1;

    surface.put_vertical(loop_x, top, rect.h, glyphs.vertical, Layer::Connector);
    surface.put_horizontal(
        rect.x + rect.w,
        bot,
        loop_x - (rect.x + rect.w) + 1,
        glyphs.horizontal,
        Layer::Connector,
    );

    let top_right = if ctx.charset == Charset::Ascii { '+' } else { '┐' };
    let bottom_right = if ctx.charset == Charset::Ascii { '+' } else { '┘' };
    surface.put_layered(loop_x, top, top_right, Layer::Connector);
    surface.put_layered(loop_x, bot, bottom_right, Layer::Connector);

    surface.put_layered(rect.x + rect.w, mid, '<', Layer::ConnectorEnd);
}
