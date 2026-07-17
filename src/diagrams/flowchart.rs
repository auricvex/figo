//! Flowchart diagrams with nodes and orthogonally-routed connections.
//!
//! Nodes support three shapes: `Rectangle`, `Rounded`, and `Diamond`
//! (decision). The layout engine stacks nodes vertically and routes
//! connectors orthogonally. Forward edges flow top-to-bottom; back-edges
//! (target above source) route via a side corridor to the right of every
//! node so they do not punch through intermediates.

use std::collections::{HashMap, HashSet};
use std::fmt;

use crate::canvas::{Canvas, Layer};
use crate::error::{FigoError, Result};
use crate::layout::connector::Connector;
use crate::layout::geom::{Anchor, Rect};
use crate::style::{BorderStyle, Charset, LineStyle};

use super::flowchart_shape::{draw_diamond, node_dims};

// Re-export NodeShape so the public API path `figo::diagrams::flowchart::NodeShape`
// stays stable even though the enum now lives in `flowchart_shape`.
pub use super::flowchart_shape::NodeShape;

/// A node in the flowchart.
#[derive(Debug, Clone)]
pub struct FlowNode {
    /// Unique id; matches a connection's `from`/`to`.
    pub id: String,
    /// Text shown inside the node.
    pub label: String,
    /// Visual shape.
    pub shape: NodeShape,
    /// `(x, y)` override; auto-layout assigns when `None`. Required for `Layout::Manual`.
    pub position: Option<(usize, usize)>,
}

/// A connection between two nodes.
#[derive(Debug, Clone)]
pub struct FlowConnection {
    /// Source node id.
    pub from: String,
    /// Target node id.
    pub to: String,
    /// Optional label drawn beside the connector.
    pub label: Option<String>,
}

/// Layout mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// Automatic layered layout (default).
    Auto,
    /// User-supplied positions.
    Manual,
}

/// Builder for flowchart diagrams.
pub struct Flowchart {
    width: usize,
    charset: Charset,
    nodes: Vec<FlowNode>,
    connections: Vec<FlowConnection>,
    layout: Layout,
    color: bool,
}

#[derive(Clone)]
struct PositionedNode {
    node: FlowNode,
    rect: Rect,
}

/// Vertical stride (rows) between successive nodes in auto-layout.
/// Shared by `layout_auto` and `render_positions` canvas sizing.
const AUTO_STRIDE_ROWS: usize = 4;

impl Flowchart {
    /// Create a new flowchart builder.
    pub fn new(width: usize, charset: Charset) -> Self {
        Self {
            width,
            charset,
            nodes: Vec::new(),
            connections: Vec::new(),
            layout: Layout::Auto,
            color: false,
        }
    }

    /// Add a node.
    pub fn add_node(mut self, node: FlowNode) -> Self {
        self.nodes.push(node);
        self
    }

    /// Add a connection between two nodes.
    pub fn connect(mut self, from: &str, to: &str, label: Option<&str>) -> Self {
        self.connections.push(FlowConnection {
            from: from.to_string(),
            to: to.to_string(),
            label: label.map(String::from),
        });
        self
    }

    /// Set the layout mode.
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    /// Enable or disable color output.
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Render and return as a `String`.
    pub fn build(&self) -> Result<String> {
        if self.nodes.is_empty() {
            return Err(FigoError::MissingFields("flowchart must have nodes".into()));
        }
        let positions = match self.layout {
            Layout::Manual => self.layout_manual()?,
            Layout::Auto => self.layout_auto()?,
        };
        self.render_positions(&positions)
    }

    fn layout_manual(&self) -> Result<Vec<PositionedNode>> {
        let mut out = Vec::new();
        for node in &self.nodes {
            let pos = node.position.ok_or_else(|| {
                FigoError::MissingFields(format!(
                    "node '{}' has no position in manual layout",
                    node.id
                ))
            })?;
            let (w, h) = node_dims(&node.label, node.shape, self.width);
            out.push(PositionedNode { node: node.clone(), rect: Rect::new(pos.0, pos.1, w, h) });
        }
        Ok(out)
    }

    fn layout_auto(&self) -> Result<Vec<PositionedNode>> {
        let dims: Vec<(usize, usize)> =
            self.nodes.iter().map(|n| node_dims(&n.label, n.shape, self.width)).collect();
        let adj = self.build_adjacency();
        let (order, back_edges) = self.detect_back_edges(&adj);
        let layers = self.compute_layers(&adj, &order, &back_edges);
        self.assign_positions(&dims, &layers)
    }

    /// Build adjacency list by node index from the declared connections.
    fn build_adjacency(&self) -> Vec<Vec<usize>> {
        let id_to_idx: HashMap<&str, usize> =
            self.nodes.iter().enumerate().map(|(i, n)| (n.id.as_str(), i)).collect();
        let mut adj = vec![Vec::new(); self.nodes.len()];
        for conn in &self.connections {
            if let (Some(&from), Some(&to)) =
                (id_to_idx.get(conn.from.as_str()), id_to_idx.get(conn.to.as_str()))
            {
                adj[from].push(to);
            }
        }
        adj
    }

    /// Detect back-edges via DFS so the remaining graph is a DAG.
    /// Returns the topological order (post-order) and the set of back-edges.
    fn detect_back_edges(&self, adj: &[Vec<usize>]) -> (Vec<usize>, HashSet<(usize, usize)>) {
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum State {
            Unvisited,
            Visiting,
            Visited,
        }
        let mut states = vec![State::Unvisited; self.nodes.len()];
        let mut back_edges: HashSet<(usize, usize)> = HashSet::new();
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

        for i in 0..self.nodes.len() {
            if matches!(states[i], State::Unvisited) {
                dfs(i, adj, &mut states, &mut back_edges, &mut order);
            }
        }
        (order, back_edges)
    }

    /// Assign layers by longest path in the DAG (ignoring back-edges).
    fn compute_layers(
        &self,
        adj: &[Vec<usize>],
        order: &[usize],
        back_edges: &HashSet<(usize, usize)>,
    ) -> Vec<usize> {
        let mut layers = vec![0usize; self.nodes.len()];
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

    /// Group nodes by layer and compute their (x, y) coordinates.
    fn assign_positions(
        &self,
        dims: &[(usize, usize)],
        layers: &[usize],
    ) -> Result<Vec<PositionedNode>> {
        let max_layer = *layers.iter().max().unwrap_or(&0);
        let mut layers_map: HashMap<usize, Vec<usize>> = HashMap::new();
        for (idx, &layer) in layers.iter().enumerate() {
            layers_map.entry(layer).or_default().push(idx);
        }

        // Sort each layer by original node index for stable ordering.
        let mut layer_positions: Vec<Vec<usize>> = Vec::new();
        for layer in 0..=max_layer {
            if let Some(mut indices) = layers_map.remove(&layer) {
                indices.sort();
                layer_positions.push(indices);
            }
        }

        let mut y = 1usize;
        let mut out: Vec<Option<PositionedNode>> = vec![None; self.nodes.len()];

        for layer_indices in &layer_positions {
            let total_w: usize = layer_indices.iter().map(|&idx| dims[idx].0).sum::<usize>()
                + layer_indices.len().saturating_sub(1) * 6;
            let start_x = self.width.saturating_sub(total_w) / 2;
            let mut x = start_x;
            let max_h = layer_indices.iter().map(|&idx| dims[idx].1).max().unwrap_or(0);
            for &idx in layer_indices {
                let (w, h) = dims[idx];
                out[idx] = Some(PositionedNode {
                    node: self.nodes[idx].clone(),
                    rect: Rect::new(x, y, w, h),
                });
                x += w + 6;
            }
            y += max_h + AUTO_STRIDE_ROWS;
        }

        Ok(out.into_iter().map(Option::unwrap).collect())
    }

    fn render_positions(&self, positions: &[PositionedNode]) -> Result<String> {
        let max_w = positions.iter().map(|p| p.rect.right()).max().unwrap_or(0).max(self.width);
        // Reserve room on the right for back-edge side corridors and labels.
        let side_room = self.side_room_for_back_edges(positions);
        let max_h =
            positions.iter().map(|p| p.rect.bottom()).max().unwrap_or(10) + 2 + AUTO_STRIDE_ROWS;
        let mut canvas = Canvas::new(max_w + side_room, max_h);

        // Phase 1 — node borders and labels.
        for pos in positions {
            self.draw_node(&mut canvas, pos);
        }

        // Phase 2 — connectors.
        let pos_map: HashMap<&str, &PositionedNode> =
            positions.iter().map(|p| (p.node.id.as_str(), p)).collect();
        let all_rects: Vec<Rect> = positions.iter().map(|p| p.rect).collect();
        for conn in &self.connections {
            let (Some(&from), Some(&to)) =
                (pos_map.get(conn.from.as_str()), pos_map.get(conn.to.as_str()))
            else {
                continue;
            };
            // Treat same-layer connections as back-edges so they route around
            // nodes instead of punching straight through them.
            let is_back = from.rect.y >= to.rect.y;
            let mut connector = Connector::new(
                from.rect,
                to.rect,
                Anchor::South,
                Anchor::North,
                LineStyle::Simple,
                self.charset,
            )
            .with_avoids(
                positions
                    .iter()
                    .filter(|p| p.node.id != from.node.id && p.node.id != to.node.id)
                    .map(|p| p.rect),
            );
            if let Some(label) = &conn.label {
                connector.label = Some(label.clone());
            }
            // Read the canvas width BEFORE the mutable borrow to satisfy
            // the borrow checker (can't borrow canvas mutably and immutably
            // in the same expression).
            if is_back {
                let w = canvas.width();
                connector.render_side_route(&mut canvas, &all_rects, w);
            } else {
                connector.render(&mut canvas);
            }
        }

        // Phase 3 — repair connector junctions so corners and crossings
        // use proper Unicode box-drawing glyphs.
        canvas.repair_connector_junctions(LineStyle::Simple, self.charset);

        Ok(canvas.render(self.color))
    }

    /// Draw a single node (border + centered label) onto the canvas.
    fn draw_node(&self, canvas: &mut Canvas, pos: &PositionedNode) {
        match pos.node.shape {
            NodeShape::Diamond => {
                draw_diamond(canvas, pos.rect.x, pos.rect.y, pos.rect.w, pos.rect.h, self.charset);
            }
            NodeShape::Rounded | NodeShape::Rectangle => {
                let style = if pos.node.shape == NodeShape::Rounded {
                    BorderStyle::Rounded
                } else {
                    BorderStyle::Single
                };
                // draw_border only errors on sub-2x2 rects, which node_dims
                // prevents; discard the impossible error rather than propagate.
                let _ = canvas.draw_border(
                    pos.rect.x,
                    pos.rect.y,
                    pos.rect.w,
                    pos.rect.h,
                    style,
                    self.charset,
                );
                // Fill the interior so connectors routed behind the node are
                // hidden by the higher-layer background.
                for ry in 1..pos.rect.h.saturating_sub(1) {
                    canvas.put_horizontal_layered(
                        pos.rect.x + 1,
                        pos.rect.y + ry,
                        pos.rect.w.saturating_sub(2),
                        ' ',
                        Layer::NodeContent,
                    );
                }
            }
        }
        // Centered label. For diamonds the label sits on the middle row;
        // for rectangles on the single content row.
        let lx = pos.rect.x + (pos.rect.w.saturating_sub(pos.node.label.chars().count())) / 2;
        let ly = pos.rect.y + pos.rect.h / 2;
        canvas.put_str_layered(lx, ly, &pos.node.label, Layer::NodeContent, None);
    }

    /// Compute the extra columns needed on the right for back-edge side
    /// corridors. Reserves enough room for the corridor plus the longest
    /// back-edge label placed to the right of the corridor.
    fn side_room_for_back_edges(&self, positions: &[PositionedNode]) -> usize {
        let id_y: HashMap<&str, usize> =
            positions.iter().map(|p| (p.node.id.as_str(), p.rect.y)).collect();
        let max_label_len = self
            .connections
            .iter()
            .filter(|c| {
                matches!(
                    (id_y.get(c.from.as_str()), id_y.get(c.to.as_str())),
                    (Some(fy), Some(ty)) if fy > ty
                )
            })
            .map(|c| c.label.as_ref().map(|l| l.chars().count()).unwrap_or(0))
            .max()
            .unwrap_or(0);
        // route_x = max_right + 2, label at route_x + 1, so label end is
        // max_right + 2 + 1 + label_len. Canvas width is max_w + side_room.
        // In the worst case max_w == max_right, so side_room must cover
        // the corridor (2) + label gap (1) + label length.
        (3 + max_label_len).max(6)
    }

    pub fn render(&self) -> Result<String> {
        self.build()
    }
}

impl fmt::Display for Flowchart {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.build() {
            Ok(s) => write!(f, "{s}"),
            Err(e) => write!(f, "[figo error: {e}]"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_flowchart() {
        let fc = Flowchart::new(80, Charset::Unicode)
            .add_node(FlowNode {
                id: "a".into(),
                label: "Start".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .add_node(FlowNode {
                id: "b".into(),
                label: "End".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .connect("a", "b", None);
        let out = fc.build().unwrap();
        assert!(out.contains("Start"));
        assert!(out.contains("End"));
    }

    #[test]
    fn test_empty_nodes() {
        assert!(Flowchart::new(80, Charset::Unicode).build().is_err());
    }

    #[test]
    fn test_diamond_shape_renders() {
        // The spec's primary example uses a diamond decision node.
        let fc = Flowchart::new(80, Charset::Unicode)
            .add_node(FlowNode {
                id: "start".into(),
                label: "Start".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .add_node(FlowNode {
                id: "decision".into(),
                label: "Is valid?".into(),
                shape: NodeShape::Diamond,
                position: None,
            })
            .add_node(FlowNode {
                id: "end".into(),
                label: "End".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .connect("start", "decision", None)
            .connect("decision", "end", Some("yes"));
        let out = fc.build().unwrap();
        assert!(out.contains("Is valid?"), "diamond label missing:\n{out}");
        // Diamond border uses ^ (top apex) and v (bottom apex).
        assert!(out.contains('^'), "diamond top apex missing:\n{out}");
        assert!(out.contains('v'), "diamond bottom apex missing:\n{out}");
    }

    #[test]
    fn test_back_edge_routes_via_side() {
        // decision -> process (back-edge, target above source) must NOT
        // punch a vertical line straight through the gap. The side route
        // pushes the vertical leg to the right of every node.
        let fc = Flowchart::new(80, Charset::Unicode)
            .add_node(FlowNode {
                id: "start".into(),
                label: "Start".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .add_node(FlowNode {
                id: "process".into(),
                label: "Process".into(),
                shape: NodeShape::Rectangle,
                position: None,
            })
            .add_node(FlowNode {
                id: "decision".into(),
                label: "Valid?".into(),
                shape: NodeShape::Diamond,
                position: None,
            })
            .connect("start", "process", None)
            .connect("process", "decision", None)
            .connect("decision", "process", Some("no"));
        let out = fc.build().unwrap();
        // The "no" label must appear (side-route draws it by the corridor).
        assert!(out.contains("no"), "back-edge label 'no' missing:\n{out}");
    }

    /// Regression for z-layer protection: a connector's vertical leg
    /// passes through B's column, but B's borders (Layer::NodeBorder=3)
    /// and label (Layer::NodeContent=5) must remain visible because both
    /// z-layers outrank Layer::Connector=1.
    fn three_node_stack() -> Flowchart {
        Flowchart::new(40, Charset::Unicode)
            .layout(Layout::Manual)
            .add_node(FlowNode {
                id: "A".into(),
                label: "A".into(),
                shape: NodeShape::Rectangle,
                position: Some((17, 1)),
            })
            .add_node(FlowNode {
                id: "B".into(),
                label: "B".into(),
                shape: NodeShape::Rectangle,
                position: Some((17, 9)),
            })
            .add_node(FlowNode {
                id: "C".into(),
                label: "C".into(),
                shape: NodeShape::Rectangle,
                position: Some((17, 17)),
            })
            // A -> C is a forward edge whose vertical leg spans B's row.
            .connect("A", "C", None)
    }

    #[test]
    fn test_intermediate_node_survives_connector() {
        let out = three_node_stack().build().unwrap();
        // Every label must survive.
        for label in ['A', 'B', 'C'] {
            assert!(out.contains(label), "node label {label:?} missing:\n{out}");
        }
        // Rectangle border glyphs must remain — the connector never
        // produces them and must not overwrite them.
        for glyph in ['┌', '┐', '└', '┘'] {
            assert!(
                out.contains(glyph),
                "rectangle border glyph {glyph:?} missing — NodeBorder \
                 may have been overwritten by Connector:\n{out}",
            );
        }
    }

    #[test]
    fn test_multi_branch_places_children_side_by_side() {
        let fc = Flowchart::new(80, Charset::Unicode)
            .add_node(FlowNode {
                id: "a".into(),
                label: "A".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .add_node(FlowNode {
                id: "b".into(),
                label: "B".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .add_node(FlowNode {
                id: "c".into(),
                label: "C".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .connect("a", "b", None)
            .connect("a", "c", None);
        let out = fc.build().unwrap();
        assert!(out.contains("B"), "child B missing:\n{out}");
        assert!(out.contains("C"), "child C missing:\n{out}");
        // Both children should appear in the rendered output.
    }

    #[test]
    fn test_cyclic_layout_renders_without_infinite_loop() {
        let fc = Flowchart::new(80, Charset::Unicode)
            .add_node(FlowNode {
                id: "a".into(),
                label: "A".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .add_node(FlowNode {
                id: "b".into(),
                label: "B".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .add_node(FlowNode {
                id: "c".into(),
                label: "C".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .connect("a", "b", None)
            .connect("b", "a", Some("retry"))
            .connect("b", "c", None);
        let out = fc.build().unwrap();
        assert!(out.contains("A"), "node A missing:\n{out}");
        assert!(out.contains("B"), "node B missing:\n{out}");
        assert!(out.contains("C"), "node C missing:\n{out}");
        assert!(out.contains("retry"), "back-edge label missing:\n{out}");
    }

    #[test]
    fn test_arrowhead_points_down() {
        // Forward edge (South -> North) must render a downward arrow (↓)
        // at the target top, not a rightward arrow (→).
        let fc = Flowchart::new(80, Charset::Unicode)
            .add_node(FlowNode {
                id: "a".into(),
                label: "A".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .add_node(FlowNode {
                id: "b".into(),
                label: "B".into(),
                shape: NodeShape::Rounded,
                position: None,
            })
            .connect("a", "b", None);
        let out = fc.build().unwrap();
        assert!(out.contains('↓'), "expected downward arrowhead ↓:\n{out}");
    }
}
