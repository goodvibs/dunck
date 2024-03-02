use crate::pgn::PgnHistoryTree;
use crate::pgn::pgn_move_node::PgnMoveNode;
use crate::state::State;

pub struct PgnHistoryTreeTraverser<'a> {
    history: &'a PgnHistoryTree,
    current_state: &'a State,
    next_node: Option<*mut PgnMoveNode>
}

impl<'a> PgnHistoryTreeTraverser<'a> {
    pub fn new(history: &'a PgnHistoryTree) -> Self {
        PgnHistoryTreeTraverser {
            history,
            current_state: &history.initial_state,
            next_node: history.head,
        }
    }

    pub fn has_next(&self) -> bool {
        if let Some(current_node) = self.next_node {
            unsafe {
                (*current_node).has_next()
            }
        }
        else {
            false
        }
    }

    pub fn has_variation(&self) -> bool {
        if let Some(current_node) = self.next_node {
            unsafe {
                (*current_node).has_variation()
            }
        }
        else {
            false
        }
    }
}