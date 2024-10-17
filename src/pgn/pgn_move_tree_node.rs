use std::cell::RefCell;
use std::rc::Rc;
use crate::utils::Color;
use crate::r#move::Move;
use crate::state::State;

pub type PgnMoveTreeNodePtr = Rc<RefCell<PgnMoveTreeNode>>;

pub struct PgnMoveTreeNode {
    pub(crate) move_and_san_and_previous_node: Option<(Move, String, PgnMoveTreeNodePtr)>,
    pub(crate) state_after_move: State,
    pub(crate) next_nodes: Vec<PgnMoveTreeNodePtr>,
}

impl PgnMoveTreeNode {
    pub(crate) fn new_root() -> PgnMoveTreeNodePtr {
        Rc::new(RefCell::new(PgnMoveTreeNode {
            move_and_san_and_previous_node: None,
            state_after_move: State::initial(),
            next_nodes: Vec::new(),
        }))
    }

    pub(crate) fn new_linked_to_previous(
        move_: Move,
        san: String,
        previous_node: PgnMoveTreeNodePtr,
        state_after_move: State,
    ) -> PgnMoveTreeNodePtr {
        let new_node = Rc::new(RefCell::new(PgnMoveTreeNode {
            move_and_san_and_previous_node: Some((move_, san, Rc::clone(&previous_node))),
            state_after_move,
            next_nodes: Vec::new(),
        }));

        // Add the new node to the previous node's children
        previous_node.borrow_mut().next_nodes.push(Rc::clone(&new_node));

        new_node
    }

    pub(crate) fn has_next(&self) -> bool {
        !self.next_nodes.is_empty()
    }

    pub(crate) fn has_variation(&self) -> bool {
        self.next_nodes.len() > 1
    }
    
    pub fn next_nodes(&self) -> Vec<PgnMoveTreeNodePtr> {
        self.next_nodes.clone()
    }

    pub(crate) fn next_main_node(&self) -> Option<PgnMoveTreeNodePtr> {
        self.next_nodes.first().cloned()
    }

    pub(crate) fn next_variation_nodes(&self) -> Vec<PgnMoveTreeNodePtr> {
        if self.next_nodes.len() < 2 {
            return Vec::new();
        }
        self.next_nodes[1..].to_vec()
    }
}