//! Command handler for flowchart diagrams.

use super::{JsonCharset, JsonPosition};
use figo::diagrams::flowchart::{FlowNode, Flowchart, Layout, NodeShape};
use figo::error::{FigoError, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct FlowchartInput {
    width: usize,
    charset: JsonCharset,
    #[serde(default)]
    layout: Option<String>,
    nodes: Vec<FlowchartNodeJson>,
    #[serde(default)]
    connections: Vec<FlowchartConnectionJson>,
    #[serde(default)]
    color: bool,
}

#[derive(Deserialize)]
struct FlowchartNodeJson {
    id: String,
    label: String,
    #[serde(default = "default_shape")]
    shape: String,
    position: Option<JsonPosition>,
}

fn default_shape() -> String {
    "rectangle".into()
}

#[derive(Deserialize)]
struct FlowchartConnectionJson {
    from: String,
    to: String,
    #[serde(default)]
    label: Option<String>,
}

pub fn run_flowchart(input: &str) -> Result<String> {
    let inp: FlowchartInput = serde_json::from_str(input)?;
    let mut fc = Flowchart::new(inp.width, inp.charset.into()).color(inp.color);

    if inp.layout.as_deref() == Some("manual") {
        fc = fc.layout(Layout::Manual);
    }

    for n in inp.nodes {
        let shape = match n.shape.as_str() {
            "rounded" => NodeShape::Rounded,
            "diamond" => NodeShape::Diamond,
            "rectangle" | "" => NodeShape::Rectangle,
            other => {
                return Err(FigoError::InvalidInput(format!(
                    "node '{}' uses unsupported shape {:?}; supported shapes are 'rectangle', 'rounded', and 'diamond'",
                    n.id, other
                )));
            }
        };
        fc = fc.add_node(FlowNode {
            id: n.id,
            label: n.label,
            shape,
            position: n.position.map(|p| (p.x, p.y)),
        });
    }

    for c in inp.connections {
        fc = fc.connect(&c.from, &c.to, c.label.as_deref());
    }

    fc.build()
}
