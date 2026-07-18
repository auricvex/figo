//! FSM state diagram renderer.
//!
//! Renders states as rounded pills with centered labels. Accepting states
//! get a double rounded border. Transitions are routed orthogonally with
//! arrowheads and optional labels.

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

/// Builder for FSM state diagrams.
pub struct StateDiagram<'a> {
    width: usize,
    charset: Charset,
    states: Vec<StateNode>,
    initial: Option<&'a str>,
    transitions: Vec<Transition>,
    color: bool,
}

impl<'a> StateDiagram<'a> {
    /// Create a new FSM diagram builder.
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

    /// Set the initial state (entry point).
    pub fn initial(mut self, state_id: &'a str) -> Self {
        self.initial = Some(state_id);
        self
    }

    /// Add a directed transition between two states.
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
        let mut layouts =
            layout_states(&self.states, &self.transitions, self.initial, self.width, &params);
        let label_rows = compute_label_rows(&self.transitions, &layouts);
        let max_label_row = label_rows.values().copied().max().unwrap_or(0);
        // Shift states down to make room for label rows above the topmost states.
        if max_label_row > 0 {
            shift_layouts(&mut layouts, max_label_row + 1);
        }

        let id_to_layout = build_id_map(&layouts);
        let total_w = compute_canvas_width(&layouts, &params).max(self.width);
        let total_h = compute_canvas_height(&layouts, &params, max_label_row);

        let mut canvas = Canvas::new(total_w, total_h);

        {
            let mut surface = Surface::new(&mut canvas);
            let ctx = PaintContext { charset: self.charset, color: self.color };

            draw_initial_arrow(&mut surface, &id_to_layout, self.initial, self.charset);
            draw_states(&mut surface, &layouts, &ctx)?;
            draw_transitions(&mut surface, &self.transitions, &id_to_layout, &label_rows, &ctx);
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

// ── ID mapping ────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct StateLayoutRef {
    rect: Rect,
}

fn build_id_map(layouts: &[StateLayout]) -> HashMap<String, StateLayoutRef> {
    layouts.iter().map(|l| (l.id.clone(), StateLayoutRef { rect: l.rect })).collect()
}

// ── Canvas sizing ─────────────────────────────────────────────────────

fn compute_canvas_width(layouts: &[StateLayout], params: &LayoutParams) -> usize {
    let rightmost = layouts.iter().map(|l| l.rect.right()).max().unwrap_or(0);
    rightmost + params.gap_x
}

fn compute_canvas_height(
    layouts: &[StateLayout],
    params: &LayoutParams,
    max_label_row: usize,
) -> usize {
    let bottommost = layouts.iter().map(|l| l.rect.bottom()).max().unwrap_or(0);
    bottommost + params.gap_y * 2 + max_label_row * 2
}

// ── Label row computation ─────────────────────────────────────────────

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
) -> HashMap<usize, usize> {
    let id_to_layout: HashMap<&str, &StateLayout> =
        layouts.iter().map(|l| (l.id.as_str(), l)).collect();
    // In FSM diagrams there are no composite states (all transitions are external).
    let mut labels: Vec<LabelInfo> = Vec::new();

    for (idx, t) in transitions.iter().enumerate() {
        if t.from == t.to || t.label.is_none() {
            continue;
        }
        let Some(from) = id_to_layout.get(t.from.as_str()) else { continue };
        let Some(to) = id_to_layout.get(t.to.as_str()) else { continue };
        let text = t.label.as_ref().unwrap();

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

fn shift_layouts(layouts: &mut [StateLayout], dy: usize) {
    for layout in layouts.iter_mut() {
        layout.rect.y += dy;
    }
}

// ── Drawing helpers ───────────────────────────────────────────────────

fn draw_initial_arrow(
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
        draw_state(surface, layout, ctx, &measure_ctx, &mut layout_ctx);
    }
    Ok(())
}

fn draw_state(
    surface: &mut Surface<'_>,
    layout: &StateLayout,
    ctx: &PaintContext,
    measure_ctx: &MeasureContext,
    layout_ctx: &mut LayoutContext,
) {
    match layout.state_type {
        StateType::Simple => draw_simple_state(surface, layout, ctx, measure_ctx, layout_ctx),
        StateType::Accepting => draw_accepting_state(surface, layout, ctx, measure_ctx, layout_ctx),
    }
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

fn draw_accepting_state(
    surface: &mut Surface<'_>,
    layout: &StateLayout,
    ctx: &PaintContext,
    measure_ctx: &MeasureContext,
    layout_ctx: &mut LayoutContext,
) {
    // Outer rounded border.
    draw_simple_state(surface, layout, ctx, measure_ctx, layout_ctx);

    // Inner rounded border — inset by 1 cell on all sides.
    if layout.rect.w >= 4 && layout.rect.h >= 4 {
        let glyphs = BorderStyle::Rounded.glyphs(ctx.charset);
        let ix = layout.rect.x + 1;
        let iy = layout.rect.y + 1;
        let iw = layout.rect.w - 2;
        let ih = layout.rect.h - 2;
        surface.draw_rect(ix, iy, iw, ih, &glyphs);
        // Clear interior of inner border so label is readable.
        for ry in 1..ih.saturating_sub(1) {
            surface.put_horizontal(ix + 1, iy + ry, iw.saturating_sub(2), ' ', Layer::NodeContent);
        }
        // Re-draw the label at the center.
        let lx = layout.rect.x + (layout.rect.w.saturating_sub(layout.label.chars().count())) / 2;
        let ly = layout.rect.y + layout.rect.h / 2;
        surface.put_str_layered(lx, ly, &layout.label, Layer::NodeContent);
    }
}

// ── Transition drawing ────────────────────────────────────────────────

fn draw_transitions(
    surface: &mut Surface<'_>,
    transitions: &[Transition],
    id_to_layout: &HashMap<String, StateLayoutRef>,
    label_rows: &HashMap<usize, usize>,
    ctx: &PaintContext,
) {
    for (idx, t) in transitions.iter().enumerate() {
        let Some(from) = id_to_layout.get(&t.from) else { continue };
        let Some(to) = id_to_layout.get(&t.to) else { continue };

        if t.from == t.to {
            draw_self_loop(surface, from.rect, ctx);
            continue;
        }

        let row = label_rows.get(&idx).copied().unwrap_or(0);
        draw_external_transition(surface, from.rect, to.rect, t.label.as_deref(), row, ctx);
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
    let forward = from.y < to.y;

    // Anchor points: exit from the edge of `from` that faces `to`.
    let from_anchor = if forward {
        from.y + from.h // source bottom, one cell below
    } else {
        from.y // source top
    };
    let to_anchor = if forward {
        to.y // target top
    } else {
        to.y + to.h - 1 // target bottom
    };

    // Horizontal corridor in the gap between the two anchor points.
    let route_y = (from_anchor + to_anchor) / 2;

    // Vertical legs from each anchor to the route corridor.
    let (from_start, from_len) = if from_anchor < route_y {
        (from_anchor, route_y - from_anchor + 1)
    } else {
        (route_y, from_anchor - route_y + 1)
    };
    let (to_start, to_len) = if to_anchor < route_y {
        (to_anchor, route_y - to_anchor + 1)
    } else {
        (route_y, to_anchor - route_y + 1)
    };
    surface.put_vertical(from_cx, from_start, from_len, glyphs.vertical, Layer::Connector);
    surface.put_vertical(to_cx, to_start, to_len, glyphs.vertical, Layer::Connector);

    // Horizontal corridor connecting the two vertical legs.
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

    // Arrowhead pointing into the target.
    let arrow_ch = match (forward, ctx.charset) {
        (true, Charset::Ascii) => 'v',
        (true, Charset::Unicode) => '▼',
        (false, Charset::Ascii) => '^',
        (false, Charset::Unicode) => '▲',
    };
    let arrow_y = if forward { to.y } else { to.y + to.h - 1 };
    surface.put_layered(to_cx, arrow_y, arrow_ch, Layer::ConnectorEnd);

    // Label near the horizontal corridor.
    if let Some(text) = label {
        let label_x = (from_cx + to_cx) / 2;
        let label_x = label_x.saturating_sub(text.chars().count() / 2);
        let label_y = route_y.saturating_sub(1 + row * 2);
        surface.put_str_layered(label_x, label_y, text, Layer::Label);
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
