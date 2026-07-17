//! Core types for UML state machine diagrams.

use crate::error::{FigoError, Result};

/// State type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateType {
    Simple,
    Composite,
    Initial,
    Final,
    History,
}

impl StateType {
    /// Parse a state type from its string representation.
    pub fn parse(id: &str, raw: &str) -> Result<Self> {
        match raw {
            "simple" | "" => Ok(Self::Simple),
            "composite" => Ok(Self::Composite),
            "initial" => Ok(Self::Initial),
            "final" => Ok(Self::Final),
            "history" => Ok(Self::History),
            other => Err(FigoError::InvalidInput(format!(
                "state '{id}' uses unsupported type {other:?}; supported: simple, composite, initial, final, history"
            ))),
        }
    }
}

/// A state in the state diagram.
#[derive(Debug, Clone)]
pub struct StateNode {
    pub id: String,
    pub label: String,
    pub state_type: StateType,
    pub row: usize,
    pub col: usize,
    pub children: Vec<StateNode>,
}

/// A transition between states.
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}
