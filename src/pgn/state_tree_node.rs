use std::cell::RefCell;
use std::rc::Rc;
use crate::r#move::Move;
use crate::state::State;

pub struct PgnStateTreeNode {
    pub move_and_san_and_previous_node: Option<(Move, String, Rc<RefCell<PgnStateTreeNode>>)>,
    pub state_after_move: State,
    pub next_nodes: Vec<Rc<RefCell<PgnStateTreeNode>>>,
}

impl PgnStateTreeNode {
    pub fn new_root() -> Rc<RefCell<PgnStateTreeNode>> {
        Rc::new(RefCell::new(PgnStateTreeNode {
            move_and_san_and_previous_node: None,
            state_after_move: State::initial(),
            next_nodes: Vec::new(),
        }))
    }

    pub fn new_linked_to_previous(
        move_: Move,
        san: String,
        previous_node: Rc<RefCell<PgnStateTreeNode>>,
        state_after_move: State,
    ) -> Rc<RefCell<PgnStateTreeNode>> {
        let new_node = Rc::new(RefCell::new(PgnStateTreeNode {
            move_and_san_and_previous_node: Some((move_, san, Rc::clone(&previous_node))),
            state_after_move,
            next_nodes: Vec::new(),
        }));

        // Add the new node to the previous node's children
        previous_node.borrow_mut().next_nodes.push(Rc::clone(&new_node));

        new_node
    }

    pub fn has_next(&self) -> bool {
        !self.next_nodes.is_empty()
    }

    pub fn has_variation(&self) -> bool {
        self.next_nodes.len() > 1
    }
    
    pub fn next_nodes(&self) -> Vec<Rc<RefCell<PgnStateTreeNode>>> {
        self.next_nodes.clone()
    }

    pub fn next_main_node(&self) -> Option<Rc<RefCell<PgnStateTreeNode>>> {
        self.next_nodes.first().cloned()
    }

    pub fn next_variation_nodes(&self) -> Vec<Rc<RefCell<PgnStateTreeNode>>> {
        if self.next_nodes.len() < 2 {
            return Vec::new();
        }
        self.next_nodes[1..].to_vec()
    }
}