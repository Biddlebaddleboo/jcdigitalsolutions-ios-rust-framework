use std::collections::VecDeque;

const MAX_SEMANTIC_NODES: usize = 4096;
const MAX_ACTIONS_PER_NODE: usize = 64;
const MAX_QUEUED_ACTIONS: usize = 256;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Stable identity for one caller-owned semantic node.
pub struct AccessibilityNodeId(
    /// Fixed-width identity selected and retained by the caller.
    pub u64,
);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Stable identity for one action within a semantic node.
pub struct AccessibilityActionId(
    /// Fixed-width identity selected and retained by the caller.
    pub u32,
);

#[derive(Clone, Copy, Debug, PartialEq)]
/// A rectangular accessibility frame in host-local UIKit points.
pub struct SemanticFrame {
    /// Horizontal origin relative to the host bounds origin.
    pub x: f64,
    /// Vertical origin relative to the host bounds origin.
    pub y: f64,
    /// Frame width in points.
    pub width: f64,
    /// Frame height in points.
    pub height: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// The meaning of an accessibility action event.
pub enum SemanticActionKind {
    /// The node's default accessibility activation.
    Activate,
    /// A named custom action on the node.
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// A generation-tagged action requested through UIKit.
pub struct AccessibilityActionEvent {
    /// Node whose action was requested.
    pub node_id: AccessibilityNodeId,
    /// Action identity supplied by the semantic-tree producer.
    pub action_id: AccessibilityActionId,
    /// Whether UIKit requested default activation or a custom action.
    pub kind: SemanticActionKind,
    /// Tree generation at native invocation time.
    pub generation: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct SemanticActionShape {
    pub id: AccessibilityActionId,
    pub kind: SemanticActionKind,
}

pub(crate) struct SemanticNodeShape {
    pub id: AccessibilityNodeId,
    pub frame: SemanticFrame,
    pub actions: Vec<SemanticActionShape>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error returned when a semantic tree cannot be represented safely by this adapter.
pub enum SemanticTreeError {
    /// The tree contains more than 4,096 nodes.
    TooManyNodes,
    /// One node contains more than 64 actions.
    TooManyActions,
    /// Two nodes in one tree use the same identity.
    DuplicateNodeId,
    /// One node reuses an action identity.
    DuplicateActionId,
    /// A frame has nonfinite coordinates or a negative extent.
    InvalidFrame,
    /// The requested UIKit trait/action combination is unsupported by this adapter.
    UnsupportedTraitOrAction,
    /// The host already has an accessibility element list owned by another component.
    ContainerAlreadyHasAccessibilityElements,
    /// The weak UIKit host has deallocated.
    HostUnavailable,
    /// A reentrant UIKit action currently holds the event-state borrow.
    StateBusy,
    /// The `u32` tree generation reached its maximum value.
    GenerationExhausted,
}

#[derive(Clone, Copy)]
struct ActiveAction {
    node_id: AccessibilityNodeId,
    action_id: AccessibilityActionId,
    kind: SemanticActionKind,
}

pub(crate) fn validate_nodes(nodes: &[SemanticNodeShape]) -> Result<(), SemanticTreeError> {
    if nodes.len() > MAX_SEMANTIC_NODES {
        return Err(SemanticTreeError::TooManyNodes);
    }
    let mut ids = Vec::with_capacity(nodes.len());
    for node in nodes {
        if !node.frame.x.is_finite()
            || !node.frame.y.is_finite()
            || !node.frame.width.is_finite()
            || !node.frame.height.is_finite()
            || node.frame.width < 0.0
            || node.frame.height < 0.0
        {
            return Err(SemanticTreeError::InvalidFrame);
        }
        ids.push(node.id);
        if node.actions.len() > MAX_ACTIONS_PER_NODE {
            return Err(SemanticTreeError::TooManyActions);
        }
        let mut action_ids = Vec::with_capacity(node.actions.len());
        for action in &node.actions {
            action_ids.push(action.id);
        }
        action_ids.sort_unstable();
        if action_ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(SemanticTreeError::DuplicateActionId);
        }
    }
    ids.sort_unstable();
    if ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(SemanticTreeError::DuplicateNodeId);
    }
    Ok(())
}

pub(crate) struct SemanticTreeState {
    alive: bool,
    generation: u32,
    active_nodes: Vec<AccessibilityNodeId>,
    active_actions: Vec<ActiveAction>,
    events: VecDeque<AccessibilityActionEvent>,
}

impl SemanticTreeState {
    pub(crate) fn new() -> Self {
        Self {
            alive: true,
            generation: 0,
            active_nodes: Vec::new(),
            active_actions: Vec::new(),
            events: VecDeque::with_capacity(MAX_QUEUED_ACTIONS),
        }
    }

    pub(crate) fn next_generation(&self) -> Result<u32, SemanticTreeError> {
        self.generation
            .checked_add(1)
            .ok_or(SemanticTreeError::GenerationExhausted)
    }

    pub(crate) fn replace(
        &mut self,
        nodes: &[SemanticNodeShape],
    ) -> Result<u32, SemanticTreeError> {
        validate_nodes(nodes)?;
        let generation = self.next_generation()?;
        self.active_nodes.clear();
        self.active_actions.clear();
        for node in nodes {
            self.active_nodes.push(node.id);
            self.active_actions
                .extend(node.actions.iter().map(|action| ActiveAction {
                    node_id: node.id,
                    action_id: action.id,
                    kind: action.kind,
                }));
        }
        self.generation = generation;
        self.events.clear();
        Ok(generation)
    }

    pub(crate) fn record_action(
        &mut self,
        generation: u32,
        node_id: AccessibilityNodeId,
        action_id: AccessibilityActionId,
        kind: SemanticActionKind,
    ) -> bool {
        if !self.alive || generation != self.generation {
            return false;
        }
        if !self.active_nodes.contains(&node_id)
            || !self.active_actions.iter().any(|active| {
                active.node_id == node_id && active.action_id == action_id && active.kind == kind
            })
            || self.events.len() == MAX_QUEUED_ACTIONS
        {
            return false;
        }
        self.events.push_back(AccessibilityActionEvent {
            node_id,
            action_id,
            kind,
            generation,
        });
        true
    }

    pub(crate) fn take_event(&mut self) -> Option<AccessibilityActionEvent> {
        self.events.pop_front()
    }

    pub(crate) fn contains_node(&self, id: AccessibilityNodeId) -> bool {
        self.alive && self.active_nodes.contains(&id)
    }

    pub(crate) fn close(&mut self) {
        self.alive = false;
        self.active_nodes.clear();
        self.active_actions.clear();
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AccessibilityActionEvent, AccessibilityActionId, AccessibilityNodeId, MAX_ACTIONS_PER_NODE,
        MAX_QUEUED_ACTIONS, MAX_SEMANTIC_NODES, SemanticActionKind, SemanticActionShape,
        SemanticFrame, SemanticNodeShape, SemanticTreeError, SemanticTreeState, validate_nodes,
    };

    fn frame(x: f64) -> SemanticFrame {
        SemanticFrame {
            x,
            y: 2.0,
            width: 30.0,
            height: 20.0,
        }
    }

    #[test]
    fn ordered_tree_updates_preserve_node_identity_and_advance_generation() {
        let first = [
            SemanticNodeShape {
                id: AccessibilityNodeId(1),
                frame: frame(0.0),
                actions: vec![
                    SemanticActionShape {
                        id: AccessibilityActionId(9),
                        kind: SemanticActionKind::Activate,
                    },
                    SemanticActionShape {
                        id: AccessibilityActionId(10),
                        kind: SemanticActionKind::Custom,
                    },
                ],
            },
            SemanticNodeShape {
                id: AccessibilityNodeId(2),
                frame: frame(40.0),
                actions: vec![],
            },
        ];
        let second = [
            SemanticNodeShape {
                id: AccessibilityNodeId(2),
                frame: frame(42.0),
                actions: vec![],
            },
            SemanticNodeShape {
                id: AccessibilityNodeId(1),
                frame: frame(1.0),
                actions: vec![SemanticActionShape {
                    id: AccessibilityActionId(11),
                    kind: SemanticActionKind::Custom,
                }],
            },
        ];
        let mut state = SemanticTreeState::new();
        assert_eq!(state.replace(&first), Ok(1));
        assert_eq!(state.replace(&second), Ok(2));
        assert_eq!(
            state.active_nodes.as_slice(),
            &[AccessibilityNodeId(2), AccessibilityNodeId(1)]
        );
        assert!(state.contains_node(AccessibilityNodeId(1)));
        assert!(state.contains_node(AccessibilityNodeId(2)));
        assert!(state.record_action(
            2,
            AccessibilityNodeId(1),
            AccessibilityActionId(11),
            SemanticActionKind::Custom
        ));
        assert_eq!(
            state.take_event(),
            Some(AccessibilityActionEvent {
                node_id: AccessibilityNodeId(1),
                action_id: AccessibilityActionId(11),
                kind: SemanticActionKind::Custom,
                generation: 2,
            })
        );
    }

    #[test]
    fn stale_removed_and_replaced_actions_are_rejected() {
        let first = [SemanticNodeShape {
            id: AccessibilityNodeId(7),
            frame: frame(0.0),
            actions: vec![
                SemanticActionShape {
                    id: AccessibilityActionId(2),
                    kind: SemanticActionKind::Activate,
                },
                SemanticActionShape {
                    id: AccessibilityActionId(3),
                    kind: SemanticActionKind::Custom,
                },
            ],
        }];
        let retained = [SemanticNodeShape {
            id: AccessibilityNodeId(7),
            frame: frame(1.0),
            actions: vec![SemanticActionShape {
                id: AccessibilityActionId(3),
                kind: SemanticActionKind::Custom,
            }],
        }];
        let removed: [SemanticNodeShape; 0] = [];
        let mut state = SemanticTreeState::new();
        let old_generation = state.replace(&first).unwrap();
        assert!(state.record_action(
            old_generation,
            AccessibilityNodeId(7),
            AccessibilityActionId(3),
            SemanticActionKind::Custom
        ));
        let new_generation = state.replace(&retained).unwrap();
        assert_eq!(state.take_event(), None);
        assert!(!state.record_action(
            old_generation,
            AccessibilityNodeId(7),
            AccessibilityActionId(3),
            SemanticActionKind::Custom
        ));
        assert_eq!(state.replace(&removed), Ok(3));
        assert!(!state.record_action(
            new_generation,
            AccessibilityNodeId(7),
            AccessibilityActionId(3),
            SemanticActionKind::Custom
        ));
    }

    #[test]
    fn duplicate_nodes_and_actions_fail_without_mutating_tree() {
        let duplicate_nodes = [
            SemanticNodeShape {
                id: AccessibilityNodeId(1),
                frame: frame(0.0),
                actions: vec![],
            },
            SemanticNodeShape {
                id: AccessibilityNodeId(1),
                frame: frame(1.0),
                actions: vec![],
            },
        ];
        let duplicate_actions = [SemanticNodeShape {
            id: AccessibilityNodeId(2),
            frame: frame(0.0),
            actions: vec![
                SemanticActionShape {
                    id: AccessibilityActionId(4),
                    kind: SemanticActionKind::Activate,
                },
                SemanticActionShape {
                    id: AccessibilityActionId(4),
                    kind: SemanticActionKind::Custom,
                },
            ],
        }];
        let mut state = SemanticTreeState::new();
        assert_eq!(
            state.replace(&duplicate_nodes),
            Err(SemanticTreeError::DuplicateNodeId)
        );
        assert_eq!(
            state.replace(&duplicate_actions),
            Err(SemanticTreeError::DuplicateActionId)
        );
        assert_eq!(state.replace(&[]), Ok(1));
    }

    #[test]
    fn geometry_rejects_nonfinite_and_negative_extents_but_allows_zero_size() {
        let invalid = [
            SemanticNodeShape {
                id: AccessibilityNodeId(1),
                frame: SemanticFrame {
                    x: f64::NAN,
                    ..frame(0.0)
                },
                actions: vec![],
            },
            SemanticNodeShape {
                id: AccessibilityNodeId(1),
                frame: SemanticFrame {
                    width: -1.0,
                    ..frame(0.0)
                },
                actions: vec![],
            },
        ];
        for node in invalid {
            assert_eq!(
                validate_nodes(&[node]),
                Err(SemanticTreeError::InvalidFrame)
            );
        }
        let zero = [SemanticNodeShape {
            id: AccessibilityNodeId(1),
            frame: SemanticFrame {
                width: 0.0,
                height: 0.0,
                ..frame(0.0)
            },
            actions: vec![],
        }];
        assert_eq!(validate_nodes(&zero), Ok(()));
    }

    #[test]
    fn action_events_preserve_order_and_teardown_invalidates_state() {
        let nodes = [SemanticNodeShape {
            id: AccessibilityNodeId(5),
            frame: frame(0.0),
            actions: vec![
                SemanticActionShape {
                    id: AccessibilityActionId(5),
                    kind: SemanticActionKind::Activate,
                },
                SemanticActionShape {
                    id: AccessibilityActionId(6),
                    kind: SemanticActionKind::Custom,
                },
            ],
        }];
        let mut state = SemanticTreeState::new();
        let generation = state.replace(&nodes).unwrap();
        assert!(state.record_action(
            generation,
            AccessibilityNodeId(5),
            AccessibilityActionId(5),
            SemanticActionKind::Activate
        ));
        assert!(state.record_action(
            generation,
            AccessibilityNodeId(5),
            AccessibilityActionId(6),
            SemanticActionKind::Custom
        ));
        assert_eq!(
            state.take_event().unwrap().kind,
            SemanticActionKind::Activate
        );
        assert_eq!(state.take_event().unwrap().kind, SemanticActionKind::Custom);
        state.close();
        assert!(!state.record_action(
            generation,
            AccessibilityNodeId(5),
            AccessibilityActionId(5),
            SemanticActionKind::Activate
        ));
        assert!(!state.contains_node(AccessibilityNodeId(5)));
    }

    #[test]
    fn node_action_and_event_limits_reject_excess_without_growth() {
        let too_many_nodes: Vec<_> = (0..=MAX_SEMANTIC_NODES)
            .map(|id| SemanticNodeShape {
                id: AccessibilityNodeId(id as u64),
                frame: frame(0.0),
                actions: vec![],
            })
            .collect();
        assert_eq!(
            validate_nodes(&too_many_nodes),
            Err(SemanticTreeError::TooManyNodes)
        );
        let too_many_actions = SemanticNodeShape {
            id: AccessibilityNodeId(1),
            frame: frame(0.0),
            actions: (0..=MAX_ACTIONS_PER_NODE as u32)
                .map(|id| SemanticActionShape {
                    id: AccessibilityActionId(id),
                    kind: SemanticActionKind::Custom,
                })
                .collect(),
        };
        assert_eq!(
            validate_nodes(&[too_many_actions]),
            Err(SemanticTreeError::TooManyActions)
        );

        let nodes: Vec<_> = (0..5)
            .map(|node_id| SemanticNodeShape {
                id: AccessibilityNodeId(node_id),
                frame: frame(0.0),
                actions: (0..MAX_ACTIONS_PER_NODE as u32)
                    .map(|action_id| SemanticActionShape {
                        id: AccessibilityActionId(action_id),
                        kind: SemanticActionKind::Custom,
                    })
                    .collect(),
            })
            .collect();
        let mut state = SemanticTreeState::new();
        let generation = state.replace(&nodes).unwrap();
        let mut accepted = 0;
        for node in &nodes {
            for action in &node.actions {
                accepted +=
                    usize::from(state.record_action(generation, node.id, action.id, action.kind));
            }
        }
        assert_eq!(accepted, MAX_QUEUED_ACTIONS);
        assert_eq!(state.events.len(), MAX_QUEUED_ACTIONS);
    }
}
