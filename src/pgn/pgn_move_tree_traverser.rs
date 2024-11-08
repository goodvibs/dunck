use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::pgn::PgnMoveTree;
use crate::pgn::pgn_move_tree_node::{PgnMoveTreeNode, PgnMoveTreeNodePtr};
use crate::r#move::r#move;
use crate::state::State;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PgnMoveTreeTraverseError {
    NoMovePlayed,
    NoNextNode,
    NoPreviousNode,
    VariationDoesNotExist
}

impl Display for PgnMoveTreeTraverseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnMoveTreeTraverseError::NoMovePlayed => write!(f, "No move played"),
            PgnMoveTreeTraverseError::NoNextNode => write!(f, "No next node"),
            PgnMoveTreeTraverseError::NoPreviousNode => write!(f, "No previous node"),
            PgnMoveTreeTraverseError::VariationDoesNotExist => write!(f, "Variation does not exist")
        }
    }
}

impl Error for PgnMoveTreeTraverseError {}

pub struct PgnMoveTreeTraverser<'a> {
    tree: &'a PgnMoveTree,
    current_move_node: PgnMoveTreeNodePtr
}

impl<'a> PgnMoveTreeTraverser<'a> {
    
    pub fn new(tree: &'a PgnMoveTree) -> PgnMoveTreeTraverser<'a> {
        PgnMoveTreeTraverser {
            tree,
            current_move_node: tree.head.clone()
        }
    }
    
    pub fn get_current_state(&self) -> State {
        self.current_move_node.borrow().state_after_move.clone()
    }
    
    pub fn get_played_move(&self) -> Result<(Move, String), PgnMoveTreeTraverseError> {
        match self.current_move_node.borrow().move_and_san_and_previous_node.clone() {
            None => Err(PgnMoveTreeTraverseError::NoMovePlayed),
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
    
    pub fn get_next_main(&self) -> Result<(Move, String), PgnMoveTreeTraverseError> {
        match self.current_move_node.borrow().next_main_node() {
            None => Err(PgnMoveTreeTraverseError::NoNextNode),
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
    
    pub fn step_forward_with_main_line(&mut self) -> Result<(), PgnMoveTreeTraverseError> {
        self.current_move_node = match self.current_move_node.clone().borrow().next_main_node() {
            None => return Err(PgnMoveTreeTraverseError::NoNextNode),
            Some(node) => node
        };
        Ok(())
    }
    
    // pub fn step_forward_with_variation_by_move(&mut self, variation: Move) -> Result<(), PgnMoveTreeTraverseError> {
    //     // todo
    // }
    // 
    // pub fn step_forward_with_variation_by_san(&mut self, variation_san: &str) -> Result<(), PgnMoveTreeTraverseError> {
    //     // todo
    // }
    // 
    // pub fn step_forward_with_variation_by_index(&mut self, variation_index: usize) -> Result<(), PgnMoveTreeTraverseError> {
    //     // todo
    // }
    // 
    // pub fn step_backward(&mut self) -> Result<(), PgnMoveTreeTraverseError> {
    //     // todo
    // }
}