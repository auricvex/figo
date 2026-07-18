//! FSM state diagrams with automatic layered layout.

pub mod layout;
pub mod render;
pub mod types;

pub use render::StateDiagram;
pub use types::{StateNode, StateType, Transition};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Charset;

    #[test]
    fn test_simple_fsm() {
        let sd = StateDiagram::new(80, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "done".into(),
                label: "Done".into(),
                state_type: StateType::Accepting,
            })
            .initial("idle")
            .add_transition("idle", "done", Some("finish"));
        let out = sd.build().unwrap();
        assert!(out.contains("Idle"));
        assert!(out.contains("Done"));
        assert!(out.contains("finish"));
        // Accepting state should have double border (two sets of rounded corners).
        assert!(
            out.matches('╭').count() >= 2,
            "accepting state needs outer and inner top-left corners"
        );
    }

    #[test]
    fn test_serial_fsm_unicode() {
        let out = StateDiagram::new(80, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "running".into(),
                label: "Running".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "done".into(),
                label: "Done".into(),
                state_type: StateType::Accepting,
            })
            .initial("idle")
            .add_transition("idle", "running", Some("start"))
            .add_transition("running", "done", Some("finish"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_serial_fsm_ascii() {
        let out = StateDiagram::new(80, Charset::Ascii)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "running".into(),
                label: "Running".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "done".into(),
                label: "Done".into(),
                state_type: StateType::Accepting,
            })
            .initial("idle")
            .add_transition("idle", "running", Some("start"))
            .add_transition("running", "done", Some("finish"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_self_loop_fsm_unicode() {
        let out = StateDiagram::new(40, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
            })
            .initial("idle")
            .add_transition("idle", "idle", Some("tick"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_two_independent_states_unicode() {
        let out = StateDiagram::new(80, Charset::Unicode)
            .add_state(StateNode {
                id: "a".into(),
                label: "A".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "b".into(),
                label: "B".into(),
                state_type: StateType::Accepting,
            })
            .initial("a")
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_fork_fsm_unicode() {
        // One state transitions to two independent states — they
        // should appear in the same layer, side by side.
        let out = StateDiagram::new(80, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "a".into(),
                label: "A".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "b".into(),
                label: "B".into(),
                state_type: StateType::Accepting,
            })
            .initial("idle")
            .add_transition("idle", "a", Some("go_a"))
            .add_transition("idle", "b", Some("go_b"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_overlapping_labels_unicode() {
        // Transitions with long labels must be placed on different
        // rows to avoid collisions.
        let out = StateDiagram::new(120, Charset::Unicode)
            .add_state(StateNode {
                id: "a".into(),
                label: "A".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "b".into(),
                label: "B".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "c".into(),
                label: "C".into(),
                state_type: StateType::Simple,
            })
            .add_state(StateNode {
                id: "d".into(),
                label: "D".into(),
                state_type: StateType::Accepting,
            })
            .initial("a")
            .add_transition("a", "b", Some("very_long_label_one"))
            .add_transition("b", "c", Some("very_long_label_two"))
            .add_transition("c", "d", Some("very_long_label_three"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_empty_states_is_error() {
        let result = StateDiagram::new(80, Charset::Unicode).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_accepting_state_has_double_border() {
        let out = StateDiagram::new(40, Charset::Unicode)
            .add_state(StateNode {
                id: "done".into(),
                label: "Done".into(),
                state_type: StateType::Accepting,
            })
            .initial("done")
            .build()
            .unwrap();
        // Accepting state must have both outer and inner borders.
        assert!(out.contains("Done"));
        // There should be at least 2 top-left rounded corners (outer + inner).
        let tl_count = out.chars().filter(|&c| c == '╭').count();
        assert!(
            tl_count >= 2,
            "accepting state needs double border, got {tl_count} top-left corners"
        );
    }
}
