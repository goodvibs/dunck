use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use crate::pgn::state_tree::PgnStateTree;
use crate::pgn::state_tree_node::PgnStateTreeNode;
use crate::r#move::Move;
use crate::state::State;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PgnStateTreeTraverseError {
    NoMovePlayed,
    NoNextNode,
    NoPreviousNode,
    VariationDoesNotExist
}

impl Display for PgnStateTreeTraverseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnStateTreeTraverseError::NoMovePlayed => write!(f, "No move played"),
            PgnStateTreeTraverseError::NoNextNode => write!(f, "No next node"),
            PgnStateTreeTraverseError::NoPreviousNode => write!(f, "No previous node"),
            PgnStateTreeTraverseError::VariationDoesNotExist => write!(f, "Variation does not exist")
        }
    }
}

impl Error for PgnStateTreeTraverseError {}

pub struct PgnStateTreeTraverser<'a> {
    pub tree: &'a PgnStateTree,
    pub current_move_node: Rc<RefCell<PgnStateTreeNode>>
}

impl<'a> PgnStateTreeTraverser<'a> {
    
    pub fn new(tree: &'a PgnStateTree) -> PgnStateTreeTraverser<'a> {
        PgnStateTreeTraverser {
            tree,
            current_move_node: tree.head.clone()
        }
    }
    
    pub fn get_current_state(&self) -> State {
        self.current_move_node.borrow().state_after_move.clone()
    }
    
    pub fn get_played_move(&self) -> Result<(Move, String), PgnStateTreeTraverseError> {
        match self.current_move_node.borrow().move_and_san_and_previous_node.clone() {
            None => Err(PgnStateTreeTraverseError::NoMovePlayed),
            Some((mv, san, _)) => Ok((mv, san))
        }
    }

    pub fn has_next(&self) -> bool {
        self.current_move_node.borrow().has_next()
    }

    pub fn has_variation(&self) -> bool {
        self.current_move_node.borrow().has_variation()
    }
    
    pub fn get_all_next(&self) -> Vec<(Move, String)> {
        self.current_move_node.borrow().next_nodes.iter().map(|node| {
            let (mv, san, _): (Move, String, _) = node.borrow().move_and_san_and_previous_node.clone().unwrap();
            (mv, san)
        }).collect()
    }
    
    pub fn get_next_main(&self) -> Result<(Move, String), PgnStateTreeTraverseError> {
        match self.current_move_node.borrow().next_main_node() {
            None => Err(PgnStateTreeTraverseError::NoNextNode),
            Some(node) => {
                let (mv, san, _): (Move, String, _) = node.borrow().move_and_san_and_previous_node.clone().unwrap();
                Ok((mv, san))
            }
        }
    }

    pub fn get_next_variations(&self) -> Vec<(Move, String)> {
        self.current_move_node.borrow().next_variation_nodes().iter().map(|node| {
            let (mv, san, _): (Move, String, _) = node.borrow().move_and_san_and_previous_node.clone().unwrap();
            (mv, san)
        }).collect()
    }
    
    pub fn step_forward_with_main_line(&mut self) -> Result<(), PgnStateTreeTraverseError> {
        self.current_move_node = match self.current_move_node.clone().borrow().next_main_node() {
            None => return Err(PgnStateTreeTraverseError::NoNextNode),
            Some(node) => node
        };
        Ok(())
    }
    
    // pub fn step_forward_with_variation_by_move(&mut self, variation: Move) -> Result<(), PgnStateTreeTraverseError> {
    //     // todo
    // }
    // 
    // pub fn step_forward_with_variation_by_san(&mut self, variation_san: &str) -> Result<(), PgnStateTreeTraverseError> {
    //     // todo
    // }
    // 
    // pub fn step_forward_with_variation_by_index(&mut self, variation_index: usize) -> Result<(), PgnStateTreeTraverseError> {
    //     // todo
    // }
    // 
    // pub fn step_backward(&mut self) -> Result<(), PgnStateTreeTraverseError> {
    //     // todo
    // }
}