//! Command handler for UML state machine diagrams.

use super::JsonCharset;
use figo::diagrams::state::{StateDiagram, StateNode, StateType};
use figo::error::{FigoError, Result};
use serde::Deserialize;

#[derive(Deserialize)]
struct StateInput {
    width: usize,
    charset: JsonCharset,
    states: Vec<StateJson>,
    #[serde(default)]
    initial: Option<String>,
    #[serde(default)]
    transitions: Vec<TransitionJson>,
    #[serde(default)]
    color: bool,
}

#[derive(Deserialize)]
struct StateJson {
    id: String,
    label: String,
    #[serde(default = "default_state_type")]
    #[serde(rename = "type")]
    stype: String,
    #[serde(default)]
    row: usize,
    #[serde(default)]
    col: usize,
    #[serde(default)]
    children: Vec<StateJson>,
}

fn default_state_type() -> String {
    "simple".into()
}

#[derive(Deserialize)]
struct TransitionJson {
    from: String,
    to: String,
    #[serde(default)]
    label: Option<String>,
}

fn convert_states(json_states: Vec<StateJson>) -> Result<Vec<StateNode>> {
    let mut out = Vec::with_capacity(json_states.len());
    for s in json_states {
        let children = convert_states(s.children)?;
        let stype = parse_state_type(&s.id, &s.stype)?;
        out.push(StateNode {
            id: s.id,
            label: s.label,
            state_type: stype,
            row: s.row,
            col: s.col,
            children,
        });
    }
    Ok(out)
}

fn parse_state_type(id: &str, raw: &str) -> Result<StateType> {
    match raw {
        "simple" | "" => Ok(StateType::Simple),
        "composite" => Ok(StateType::Composite),
        "initial" => Ok(StateType::Initial),
        "final" => Ok(StateType::Final),
        "history" => Ok(StateType::History),
        other => Err(FigoError::InvalidInput(format!(
            "state '{id}' uses unsupported type {other:?}; supported: simple, composite, initial, final, history"
        ))),
    }
}

pub fn run_state(input: &str) -> Result<String> {
    let inp: StateInput = serde_json::from_str(input)?;
    let states = convert_states(inp.states)?;
    let initial_state = inp.initial.clone();
    let transitions = inp.transitions;
    let charset = inp.charset;

    let mut sd = StateDiagram::new(inp.width, charset.into()).color(inp.color);
    for s in states {
        sd = sd.add_state(s);
    }
    if let Some(ref init_id) = initial_state {
        sd = sd.initial(init_id);
    }
    for t in &transitions {
        sd = sd.add_transition(&t.from, &t.to, t.label.as_deref());
    }
    sd.build()
}
