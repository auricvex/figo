//! Command handlers for packet, tree, and arrow diagrams.

use super::JsonCharset;
use figo::diagrams::packet::{PacketDiagram, PacketField};
use figo::diagrams::{arrow, tree};
use figo::error::Result;
use figo::style::LineStyle;
use serde::Deserialize;

// -- Packet -------------------------------------------------------------------

#[derive(Deserialize)]
struct PacketInput {
    width: usize,
    charset: JsonCharset,
    fields: Vec<PacketFieldJson>,
    #[serde(default)]
    color: bool,
}

#[derive(Deserialize)]
struct PacketFieldJson {
    name: String,
    bits: usize,
}

pub fn run_packet(input: &str) -> Result<String> {
    let inp: PacketInput = serde_json::from_str(input)?;
    let fields: Vec<PacketField> =
        inp.fields.into_iter().map(|f| PacketField { name: f.name, bits: f.bits }).collect();
    PacketDiagram::new(inp.width, inp.charset.into()).fields(&fields).color(inp.color).build()
}

// -- Tree -------------------------------------------------------------------

#[derive(Deserialize)]
struct TreeInput {
    width: usize,
    charset: JsonCharset,
    #[serde(default)]
    root: Option<String>,
    #[serde(default)]
    nodes: Vec<TreeNodeJson>,
    #[serde(default)]
    #[allow(dead_code)]
    color: bool,
}

#[derive(Deserialize)]
struct TreeNodeJson {
    label: String,
    #[serde(default)]
    children: Vec<TreeNodeJson>,
}

fn convert_tree_nodes(json_nodes: &[TreeNodeJson]) -> Vec<tree::TreeNode> {
    json_nodes
        .iter()
        .map(|n| tree::TreeNode {
            label: n.label.clone(),
            children: convert_tree_nodes(&n.children),
        })
        .collect()
}

pub fn run_tree(input: &str) -> Result<String> {
    let inp: TreeInput = serde_json::from_str(input)?;
    let nodes = convert_tree_nodes(&inp.nodes);
    tree::draw_tree(inp.root.as_deref(), &nodes, inp.width, inp.charset.into())
}

// -- Arrow -------------------------------------------------------------------

#[derive(Deserialize)]
struct ArrowInput {
    #[allow(dead_code)]
    width: usize,
    charset: JsonCharset,
    direction: String,
    length: usize,
    #[serde(default)]
    style: Option<String>,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    color: bool,
}

pub fn run_arrow(input: &str) -> Result<String> {
    let inp: ArrowInput = serde_json::from_str(input)?;
    let style = match inp.style.as_deref() {
        Some("bold") => LineStyle::Bold,
        Some("box_drawing") => LineStyle::BoxDrawing,
        _ => LineStyle::Simple,
    };
    arrow::draw_arrow(&inp.direction, inp.length, style, inp.charset.into(), inp.label.as_deref())
}
