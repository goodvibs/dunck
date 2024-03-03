use crate::r#move::Move;
use crate::state::State;
use crate::utils::Color;

#[derive(Eq)]
pub(crate) struct PgnMoveNode {
    pub(crate) move_: Move,
    pub(crate) san: String,
    pub(crate) state_after_move: State,
    pub(crate) previous_node: Option<*mut PgnMoveNode>,
    pub(crate) next_nodes: Vec<*mut PgnMoveNode>
}

impl PgnMoveNode {
    pub(crate) fn new(move_: Move, san: String, state_after_move: State, previous_node: Option<*mut PgnMoveNode>) -> *mut PgnMoveNode {
        Box::into_raw(Box::new(PgnMoveNode {
            move_,
            san,
            state_after_move,
            previous_node,
            next_nodes: Vec::new()
        }))
    }

    pub(crate) fn has_next(&self) -> bool {
        !self.next_nodes.is_empty()
    }

    pub(crate) fn has_variation(&self) -> bool {
        self.next_nodes.len() > 1
    }

    pub(crate) fn next_main_node(&self) -> Option<*mut PgnMoveNode> {
        self.next_nodes.last().cloned()
    }

    pub(crate) fn next_variation_nodes(&self) -> Vec<*mut PgnMoveNode> {
        if self.next_nodes.len() < 2 {
            return Vec::new();
        }
        self.next_nodes[..self.next_nodes.len() - 1].to_vec()
    }

    pub(crate) fn remove_line(&mut self) {
        if self.previous_node.is_some() {
            unsafe {
                (*self.previous_node.unwrap()).next_nodes.retain(|&node| node != self as *mut PgnMoveNode);
            }
        }
    }

    pub(crate) fn pgn(&self, initial_state: State, should_render_variations: bool, mut should_remind_fullmove: bool, prepend_tabs: u8) -> String {
        let mut res = String::new();
        let san = self.move_.san(&initial_state, &self.state_after_move);
        res += match initial_state.turn {
            Color::White => format!("{}. {} ", self.state_after_move.get_fullmove(), san),
            Color::Black => match should_remind_fullmove {
                true => format!("{}... {} ", self.state_after_move.get_fullmove() - 1, san),
                false => format!("{} ", san),
            }
        }.as_str();
        should_remind_fullmove = false;
        if should_render_variations && self.has_variation() {
            let variations = self.next_variation_nodes();
            res += format!("\n{}(", "    ".repeat(prepend_tabs as usize + 1)).as_str();
            for variation in variations {
                unsafe {
                    res += &*format!("{} ", (*variation).pgn(initial_state.clone(), true, true, prepend_tabs + 1));
                }
            }
            res += format!(")\n{}", "    ".repeat(prepend_tabs as usize)).as_str();
            should_remind_fullmove = true;
        }
        if let Some(next_node) = self.next_main_node() {
            unsafe {
                res += &*format!("{}", (*next_node).pgn(self.state_after_move.clone(), should_render_variations, should_remind_fullmove, prepend_tabs));
            }
        }
        res
    }
}

impl Drop for PgnMoveNode {
    fn drop(&mut self) {
        for node in self.next_nodes.iter() {
            unsafe {
                drop(Box::from_raw(*node));
            }
        }
    }
}

impl PartialEq<Self> for PgnMoveNode {
    fn eq(&self, other: &Self) -> bool {
        if !(self.move_ == other.move_ &&
            self.state_after_move == other.state_after_move) {
            return false;
        }
        if self.previous_node.is_some() ^ other.previous_node.is_some() {
            return false;
        }
        unsafe {
            for (i, node) in self.next_nodes.iter().enumerate() {
                if !(**node == *other.next_nodes[i]) {
                    return false;
                }
            }
        }
        true
    }
}