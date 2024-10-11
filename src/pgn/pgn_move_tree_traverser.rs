// use std::error::Error;
// use std::fmt::{Debug, Display, Formatter};
// use crate::pgn::PgnMoveTree;
// use crate::pgn::pgn_move_tree_node::{PgnMoveTreeNode, PgnMoveTreeNodePtr};
// use crate::r#move::Move;
// use crate::state::State;
// 
// #[derive(Debug, Clone, Copy, Eq, PartialEq)]
// pub enum PgnMoveTreeTraverseError {
//     NoMovePlayed,
//     NoNextNode,
//     NoPreviousNode,
//     VariationDoesNotExist
// }
// 
// impl Display for PgnMoveTreeTraverseError {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             PgnMoveTreeTraverseError::NoMovePlayed => write!(f, "No move played"),
//             PgnMoveTreeTraverseError::NoNextNode => write!(f, "No next node"),
//             PgnMoveTreeTraverseError::NoPreviousNode => write!(f, "No previous node"),
//             PgnMoveTreeTraverseError::VariationDoesNotExist => write!(f, "Variation does not exist")
//         }
//     }
// }
// 
// impl Error for PgnMoveTreeTraverseError {}
// 
// pub struct PgnMoveTreeTraverser<'a> {
//     tree: &'a PgnMoveTree,
//     current_move_node: PgnMoveTreeNodePtr
// }
// 
// impl<'a> PgnMoveTreeTraverser<'a> {
//     
//     pub fn new(tree: &'a PgnMoveTree) -> PgnMoveTreeTraverser<'a> {
//         PgnMoveTreeTraverser {
//             tree,
//             current_move_node: tree.head.clone()
//         }
//     }
//     
//     pub fn get_current_state(&self) -> &State {
//         unsafe {
//             &(*self.current_move_node).state_after_move
//         }
//     }
//     
//     pub fn get_played_move(&self) -> Result<(Move, String), PgnMoveTreeTraverseError> {
//         unsafe {
//             match (*self.current_move_node).move_and_san_and_previous_node.clone() {
//                 None => Err(PgnMoveTreeTraverseError::NoMovePlayed),
//                 Some((mv, san, _)) => Ok((mv, san))
//             }
//         }
//     }
// 
//     pub fn has_next(&self) -> bool {
//         unsafe {
//             (*self.current_move_node).has_next()
//         }
//     }
// 
//     pub fn has_variation(&self) -> bool {
//         unsafe {
//             (*self.current_move_node).has_variation()
//         }
//     }
//     
//     pub fn get_next_main_line_move(&self) -> Result<(Move, String), PgnMoveTreeTraverseError> {
//         unsafe {
//             match (*self.current_move_node).next_main_node() {
//                 None => Err(PgnMoveTreeTraverseError::NoNextNode),
//                 Some(node) => {
//                     let (mv, san, _): (Move, String, *mut PgnMoveTreeNode) = (*node).move_and_san_and_previous_node.clone().unwrap();
//                     Ok((mv, san))
//                 }
//             }
//         }
//     }
// 
//     pub fn get_all_next_variations(&self) -> Vec<(Move, String)> {
//         unsafe {
//             (*self.current_move_node).next_variation_nodes().iter().map(|node| {
//                 let (mv, san, _): (Move, String, *mut PgnMoveTreeNode) = (**node).move_and_san_and_previous_node.clone().unwrap();
//                 (mv, san)
//             }).collect()
//         }
//     }
//     
//     pub fn step_forward_with_main_line(&mut self) -> Result<(), PgnMoveTreeTraverseError> {
//         unsafe {
//             if let Some(next_node) = (*self.current_move_node).next_main_node() {
//                 self.current_move_node = next_node;
//                 Ok(())
//             }
//             else {
//                 Err(PgnMoveTreeTraverseError::NoNextNode)
//             }
//         }
//     }
//     
//     pub fn step_forward_with_variation_by_move(&mut self, variation: Move) -> Result<(), PgnMoveTreeTraverseError> {
//         let variations = self.get_all_next_variations();
//         if let Some((variation, _)) = variations.iter().find(|(mv, _)| *mv == variation) {
//             unsafe {
//                 self.current_move_node = *(*self.current_move_node).next_variation_nodes().iter().find(|&&node| (*node).move_and_san_and_previous_node.clone().unwrap().0 == *variation).unwrap();
//                 Ok(())
//             }
//         }
//         else {
//             Err(PgnMoveTreeTraverseError::VariationDoesNotExist)
//         }
//     }
//     
//     pub fn step_forward_with_variation_by_san(&mut self, variation_san: &str) -> Result<(), PgnMoveTreeTraverseError> {
//         let variations = self.get_all_next_variations();
//         if let Some(variation) = variations.iter().find(|(_, san)| *san == variation_san) {
//             unsafe {
//                 self.current_move_node = *(*self.current_move_node).next_variation_nodes().iter().find(|&&node| (*node).move_and_san_and_previous_node.clone().unwrap().1 == variation_san).unwrap();
//                 Ok(())
//             }
//         }
//         else {
//             Err(PgnMoveTreeTraverseError::VariationDoesNotExist)
//         }
//     }
//     
//     pub fn step_forward_with_variation_by_index(&mut self, variation_index: usize) -> Result<(), PgnMoveTreeTraverseError> {
//         let variations = self.get_all_next_variations();
//         if let Some(variation) = variations.get(variation_index) {
//             unsafe {
//                 self.current_move_node = *(*self.current_move_node).next_variation_nodes().iter().find(|&&node| (*node).move_and_san_and_previous_node.clone().unwrap().1 == variation.1).unwrap();
//                 Ok(())
//             }
//         }
//         else {
//             Err(PgnMoveTreeTraverseError::VariationDoesNotExist)
//         }
//     }
//     
//     pub fn step_backward(&mut self) -> Result<(), PgnMoveTreeTraverseError> {
//         unsafe {
//             match (*self.current_move_node).move_and_san_and_previous_node.clone() {
//                 None => Err(PgnMoveTreeTraverseError::NoPreviousNode),
//                 Some((_, _, previous_node)) => {
//                     self.current_move_node = previous_node;
//                     Ok(())
//                 }
//             }
//         }
//     }
// }