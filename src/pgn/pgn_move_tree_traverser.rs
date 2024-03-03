use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::pgn::PgnMoveTree;
use crate::pgn::pgn_move_node::PgnMoveNode;
use crate::r#move::Move;
use crate::state::State;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PgnMoveTreeTraverseError {
    TreeIsEmpty,
    NoNextNode,
    NoPreviousNode,
    VariationDoesNotExist
}

impl Display for PgnMoveTreeTraverseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnMoveTreeTraverseError::TreeIsEmpty => write!(f, "Tree is empty"),
            PgnMoveTreeTraverseError::NoNextNode => write!(f, "No next node"),
            PgnMoveTreeTraverseError::NoPreviousNode => write!(f, "No previous node"),
            PgnMoveTreeTraverseError::VariationDoesNotExist => write!(f, "Variation does not exist")
        }
    }
}

impl Error for PgnMoveTreeTraverseError {}

pub struct PgnMoveTreeTraverser<'a> {
    history: &'a PgnMoveTree,
    state_before_move: &'a State,
    current_move_node: *mut PgnMoveNode
}

impl<'a> PgnMoveTreeTraverser<'a> {
    pub fn new(history: &'a PgnMoveTree) -> Result<Self, PgnMoveTreeTraverseError> {
        match history.head {
            None => Err(PgnMoveTreeTraverseError::TreeIsEmpty),
            Some(head) => Ok(PgnMoveTreeTraverser {
                history,
                state_before_move: &history.initial_state,
                current_move_node: head
            })
        }
    }

    pub fn has_next(&self) -> bool {
        unsafe {
            (*self.current_move_node).has_next()
        }
    }

    pub fn has_variation(&self) -> bool {
        unsafe {
            (*self.current_move_node).has_variation()
        }
    }
    
    pub fn get_next_main_line_move(&self) -> Result<(Move, String), PgnMoveTreeTraverseError> {
        unsafe {
            match (*self.current_move_node).next_main_node() {
                Some(next_main_node) => Ok(((*next_main_node).move_, (*next_main_node).san.clone())),
                None => Err(PgnMoveTreeTraverseError::NoNextNode)
            }
        }
    }

    pub fn get_all_next_variations(&self) -> Vec<(Move, String)> {
        let mut variations = Vec::new();
        unsafe {
            for variation_node in (*self.current_move_node).next_variation_nodes() {
                variations.push(((*variation_node).move_, (*variation_node).san.clone()));
            }
        }
        variations
    }
    
    pub fn step_forward_with_main_line(&mut self) -> Result<(), PgnMoveTreeTraverseError> {
        if self.has_next() {
            unsafe {
                self.state_before_move = &(*self.current_move_node).state_after_move;
                self.current_move_node = (*self.current_move_node).next_main_node().ok_or(PgnMoveTreeTraverseError::NoNextNode)?;
                Ok(())
            }
        }
        else {
            Err(PgnMoveTreeTraverseError::NoNextNode)
        }
    }
    
    pub fn step_forward_with_variation_by_move(&mut self, variation: Move) -> Result<(), PgnMoveTreeTraverseError> {
        let variations = self.get_all_next_variations();
        if let Some(variation) = variations.iter().find(|(mv, _)| mv == &variation) {
            unsafe {
                self.state_before_move = &(*self.current_move_node).state_after_move;
                self.current_move_node = *(*self.current_move_node).next_variation_nodes().iter().find(|&&node| (*node).move_ == variation.0).unwrap();
                Ok(())
            }
        }
        else {
            Err(PgnMoveTreeTraverseError::VariationDoesNotExist)
        }
    }
    
    pub fn step_forward_with_variation_by_san(&mut self, variation_san: &str) -> Result<(), PgnMoveTreeTraverseError> {
        let variations = self.get_all_next_variations();
        if let Some(variation) = variations.iter().find(|(_, san)| san == variation_san) {
            unsafe {
                self.state_before_move = &(*self.current_move_node).state_after_move;
                self.current_move_node = *(*self.current_move_node).next_variation_nodes().iter().find(|&&node| (*node).san == variation_san).unwrap();
                Ok(())
            }
        }
        else {
            Err(PgnMoveTreeTraverseError::VariationDoesNotExist)
        }
    }
    
    pub fn step_forward_with_variation_by_index(&mut self, variation_index: usize) -> Result<(), PgnMoveTreeTraverseError> {
        let variations = self.get_all_next_variations();
        if variation_index < variations.len() {
            unsafe {
                self.state_before_move = &(*self.current_move_node).state_after_move;
                self.current_move_node = *(*self.current_move_node).next_variation_nodes().get(variation_index).unwrap();
                Ok(())
            }
        }
        else {
            Err(PgnMoveTreeTraverseError::VariationDoesNotExist)
        }
    }
    
    pub fn step_backward(&mut self) -> Result<(), PgnMoveTreeTraverseError> {
        unsafe {
            if let Some(previous_node) = (*self.current_move_node).previous_node {
                self.state_before_move = &(*previous_node).state_after_move;
                self.current_move_node = previous_node;
                Ok(())
            }
            else {
                Err(PgnMoveTreeTraverseError::NoPreviousNode)
            }
        }
    }
}