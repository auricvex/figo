//! Core types for FSM state diagrams.
//!
//! FSMs have two state kinds: normal states drawn as rounded pills
//! and accepting states drawn as double-bordered rounded pills.

use crate::error::{FigoError, Result};

/// State type for finite state machines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateType {
    /// Normal state — drawn as a rounded pill with a single border.
    Simple,
    /// Accepting/final state — drawn as a double-bordered rounded pill.
    Accepting,
}

impl StateType {
    /// Parse a state type from its string representation.
    /// Accepts "simple", "accepting", or "final" (alias for accepting).
    pub fn parse(id: &str, raw: &str) -> Result<Self> {
        match raw {
            "simple" | "" => Ok(Self::Simple),
            "accepting" | "final" => Ok(Self::Accepting),
            other => Err(FigoError::InvalidInput(format!(
                "state '{id}' uses unsupported type {other:?}; supported: simple, accepting"
            ))),
        }
    }
}

/// A state in the FSM diagram.
#[derive(Debug, Clone)]
pub struct StateNode {
    /// Unique identifier used by transitions.
    pub id: String,
    /// Text displayed inside the state.
    pub label: String,
    /// Whether this is a normal or accepting state.
    pub state_type: StateType,
}

/// A directed transition between two states.
#[derive(Debug, Clone)]
pub struct Transition {
    /// Source state id.
    pub from: String,
    /// Target state id.
    pub to: String,
    /// Optional label (event, condition, or action).
    pub label: Option<String>,
}
