//! UML state machine diagrams with states, transitions, and composite states.

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
    fn test_simple_state() {
        let sd = StateDiagram::new(80, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 0,
                children: vec![],
            })
            .add_state(StateNode {
                id: "done".into(),
                label: "Done".into(),
                state_type: StateType::Final,
                row: 0,
                col: 1,
                children: vec![],
            })
            .initial("idle")
            .add_transition("idle", "done", Some("finish"));
        let out = sd.build().unwrap();
        assert!(out.contains("Idle"));
        assert!(out.contains("finish"));
    }

    #[test]
    fn snapshot_simple_state_unicode() {
        let out = StateDiagram::new(80, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 0,
                children: vec![],
            })
            .add_state(StateNode {
                id: "running".into(),
                label: "Running".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 1,
                children: vec![],
            })
            .add_state(StateNode {
                id: "done".into(),
                label: "Done".into(),
                state_type: StateType::Final,
                row: 0,
                col: 2,
                children: vec![],
            })
            .initial("idle")
            .add_transition("idle", "running", Some("start"))
            .add_transition("running", "done", Some("finish"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_simple_state_ascii() {
        let out = StateDiagram::new(80, Charset::Ascii)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 0,
                children: vec![],
            })
            .add_state(StateNode {
                id: "running".into(),
                label: "Running".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 1,
                children: vec![],
            })
            .add_state(StateNode {
                id: "done".into(),
                label: "Done".into(),
                state_type: StateType::Final,
                row: 0,
                col: 2,
                children: vec![],
            })
            .initial("idle")
            .add_transition("idle", "running", Some("start"))
            .add_transition("running", "done", Some("finish"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_composite_state_unicode() {
        let out = StateDiagram::new(80, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 0,
                children: vec![],
            })
            .add_state(StateNode {
                id: "active".into(),
                label: "Active".into(),
                state_type: StateType::Composite,
                row: 0,
                col: 1,
                children: vec![
                    StateNode {
                        id: "sub1".into(),
                        label: "Sub1".into(),
                        state_type: StateType::Simple,
                        row: 0,
                        col: 0,
                        children: vec![],
                    },
                    StateNode {
                        id: "sub2".into(),
                        label: "Sub2".into(),
                        state_type: StateType::Simple,
                        row: 0,
                        col: 1,
                        children: vec![],
                    },
                ],
            })
            .add_state(StateNode {
                id: "done".into(),
                label: "Done".into(),
                state_type: StateType::Final,
                row: 0,
                col: 2,
                children: vec![],
            })
            .initial("idle")
            .add_transition("idle", "active", Some("begin"))
            .add_transition("active", "done", Some("end"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_self_loop_unicode() {
        let out = StateDiagram::new(40, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 0,
                children: vec![],
            })
            .initial("idle")
            .add_transition("idle", "idle", Some("tick"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_nested_composite_unicode() {
        let out = StateDiagram::new(120, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 0,
                children: vec![],
            })
            .add_state(StateNode {
                id: "active".into(),
                label: "Active".into(),
                state_type: StateType::Composite,
                row: 0,
                col: 1,
                children: vec![StateNode {
                    id: "sub_active".into(),
                    label: "SubActive".into(),
                    state_type: StateType::Composite,
                    row: 0,
                    col: 0,
                    children: vec![
                        StateNode {
                            id: "sub1".into(),
                            label: "Sub1".into(),
                            state_type: StateType::Simple,
                            row: 0,
                            col: 0,
                            children: vec![],
                        },
                        StateNode {
                            id: "sub2".into(),
                            label: "Sub2".into(),
                            state_type: StateType::Simple,
                            row: 0,
                            col: 1,
                            children: vec![],
                        },
                    ],
                }],
            })
            .initial("idle")
            .add_transition("idle", "active", Some("begin"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_history_state_unicode() {
        let out = StateDiagram::new(80, Charset::Unicode)
            .add_state(StateNode {
                id: "idle".into(),
                label: "Idle".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 0,
                children: vec![],
            })
            .add_state(StateNode {
                id: "hist".into(),
                label: "H".into(),
                state_type: StateType::History,
                row: 0,
                col: 1,
                children: vec![],
            })
            .add_state(StateNode {
                id: "running".into(),
                label: "Running".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 2,
                children: vec![],
            })
            .initial("idle")
            .add_transition("idle", "hist", Some("pause"))
            .add_transition("hist", "running", Some("resume"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn snapshot_overlapping_labels_unicode() {
        let out = StateDiagram::new(120, Charset::Unicode)
            .add_state(StateNode {
                id: "a".into(),
                label: "A".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 0,
                children: vec![],
            })
            .add_state(StateNode {
                id: "b".into(),
                label: "B".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 1,
                children: vec![],
            })
            .add_state(StateNode {
                id: "c".into(),
                label: "C".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 2,
                children: vec![],
            })
            .add_state(StateNode {
                id: "d".into(),
                label: "D".into(),
                state_type: StateType::Simple,
                row: 0,
                col: 3,
                children: vec![],
            })
            .initial("a")
            .add_transition("a", "b", Some("very_long_label_one"))
            .add_transition("b", "c", Some("very_long_label_two"))
            .add_transition("c", "d", Some("very_long_label_three"))
            .build()
            .unwrap();
        insta::assert_snapshot!(out);
    }
}
