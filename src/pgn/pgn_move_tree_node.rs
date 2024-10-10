use crate::miscellaneous::Color;
use crate::r#move::Move;
use crate::state::State;
// pub(crate) enum PgnMoveTreeNodeError {
//     MismatchedPreviousNodeAndMoveOptionTypes
// }

pub(crate) struct PgnMoveTreeNode {
    pub(crate) move_and_san_and_previous_node: Option<(Move, String, *mut PgnMoveTreeNode)>, // None if it's the root node
    pub(crate) state_after_move: State, // initial state if it's the root node
    pub(crate) next_nodes: Vec<*mut PgnMoveTreeNode>
}

impl PgnMoveTreeNode {
    pub(crate) fn new_root() -> *mut PgnMoveTreeNode {
        Box::into_raw(Box::new(PgnMoveTreeNode {
            move_and_san_and_previous_node: None,
            state_after_move: State::initial(),
            next_nodes: Vec::new()
        }))
    }
    
    pub(crate) unsafe fn new_raw_linked_to_previous(move_: Move, san: String, previous_node: *mut PgnMoveTreeNode, state_after_move: State) -> *mut PgnMoveTreeNode {
        let move_and_san_and_previous_node = Some((move_, san, previous_node));
        let raw = Box::into_raw(Box::new(PgnMoveTreeNode {
            move_and_san_and_previous_node,
            state_after_move,
            next_nodes: Vec::new()
        }));
        (*previous_node).next_nodes.push(raw);
        
        raw
    }

    pub(crate) fn has_next(&self) -> bool {
        !self.next_nodes.is_empty()
    }

    pub(crate) fn has_variation(&self) -> bool {
        self.next_nodes.len() > 1
    }

    pub(crate) fn next_main_node(&self) -> Option<*mut PgnMoveTreeNode> {
        self.next_nodes.last().cloned()
    }

    pub(crate) fn next_variation_nodes(&self) -> Vec<*mut PgnMoveTreeNode> {
        if self.next_nodes.len() < 2 {
            return Vec::new();
        }
        self.next_nodes[..self.next_nodes.len() - 1].to_vec()
    }

    pub(crate) fn pgn(&self, should_render_variations: bool, mut should_remind_fullmove: bool, prepend_tabs: u8) -> String {
        let mut res = String::new();
        let (_, san, _): (Option<u8>, String, Option<u8>) = match self.move_and_san_and_previous_node.clone() {
            None => (None, "".to_string(), None),
            Some((_, s, _)) => (None, s, None)
        };
        if self.state_after_move.halfmove != 0 {
            res += match self.state_after_move.side_to_move {
                Color::White => match should_remind_fullmove {
                    true => format!("{}...{}", self.state_after_move.get_fullmove() - 1, san),
                    false => format!(" {}", san),
                },
                Color::Black => format!("{}{}.{}", if self.state_after_move.halfmove > 1 && !should_remind_fullmove { " " } else { "" }, self.state_after_move.get_fullmove(), san)
            }.as_str();
        }
        should_remind_fullmove = false;
        if should_render_variations && self.has_variation() {
            let variations = self.next_variation_nodes();
            res += format!("\n{}( ", "    ".repeat(prepend_tabs as usize + 1)).as_str();
            for variation in variations {
                unsafe {
                    res += &*format!("{}", (*variation).pgn(true, true, prepend_tabs + 1));
                }
            }
            res += format!(" )\n{}", "    ".repeat(prepend_tabs as usize)).as_str();
            should_remind_fullmove = true;
        }
        if let Some(next_node) = self.next_main_node() {
            unsafe {
                res += &*format!("{}", (*next_node).pgn(should_render_variations, should_remind_fullmove, prepend_tabs));
            }
        }
        res
    }
}

// impl Drop for PgnMoveTreeNode {
//     fn drop(&mut self) {
//         for node in self.next_nodes.iter() {
//             unsafe {
//                 drop(Box::from_raw(*node));
//             }
//         }
//     }
// }