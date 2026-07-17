//! Command handlers for box and table diagrams.

use super::{
    JsonAlignment, JsonBorder, JsonCharset, JsonPadding, border_from_json, parse_halign,
    parse_valign,
};
use figo::diagrams::{box_art, table};
use figo::error::Result;
use figo::style::{BorderStyle, HAlign};
use serde::Deserialize;

// -- Box -------------------------------------------------------------------

#[derive(Deserialize)]
struct BoxInput {
    width: usize,
    charset: JsonCharset,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    border: Option<JsonBorder>,
    #[serde(default)]
    padding: Option<JsonPadding>,
    #[serde(default)]
    align: Option<JsonAlignment>,
    #[serde(default)]
    color: bool,
}

pub fn run_box(input: &str) -> Result<String> {
    let inp: BoxInput = serde_json::from_str(input)?;
    let border = inp.border.map(border_from_json).unwrap_or(BorderStyle::Single);
    let mut b = box_art::BoxArt::new(inp.width, inp.charset.into())
        .title(inp.title.as_deref())
        .content(inp.content.as_deref())
        .border(border)
        .color(inp.color);
    if let Some(p) = inp.padding {
        b = b.padding(p.horizontal, p.vertical);
    }
    if let Some(a) = inp.align {
        b = b.align(parse_halign(&a.horizontal), parse_valign(&a.vertical));
    }
    b.build()
}

// -- Table -------------------------------------------------------------------

#[derive(Deserialize)]
struct TableInput {
    width: usize,
    charset: JsonCharset,
    #[serde(default)]
    headers: Vec<String>,
    #[serde(default)]
    rows: Vec<Vec<String>>,
    #[serde(default)]
    border: Option<JsonBorder>,
    #[serde(default)]
    padding: Option<JsonPadding>,
    #[serde(default)]
    align: Option<Vec<String>>,
    #[serde(default)]
    color: bool,
}

pub fn run_table(input: &str) -> Result<String> {
    let inp: TableInput = serde_json::from_str(input)?;
    let border = inp.border.map(border_from_json).unwrap_or(BorderStyle::Single);
    let hdrs_str: Vec<String> = inp.headers;
    let rows_str: Vec<Vec<String>> = inp.rows;
    let hdrs: Vec<&str> = hdrs_str.iter().map(|s| s.as_str()).collect();
    let rows: Vec<Vec<&str>> =
        rows_str.iter().map(|r| r.iter().map(|s| s.as_str()).collect()).collect();
    let mut tb = table::Table::new(inp.width, inp.charset.into()).border(border).color(inp.color);
    if !hdrs.is_empty() {
        tb = tb.headers(&hdrs);
    }
    if !rows.is_empty() {
        tb = tb.rows(&rows);
    }
    if let Some(p) = inp.padding {
        tb = tb.padding(p.horizontal, p.vertical);
    }
    if let Some(align) = inp.align {
        let aligns: Vec<HAlign> = align.iter().map(|s| parse_halign(s)).collect();
        tb = tb.align(aligns);
    }
    tb.build()
}
