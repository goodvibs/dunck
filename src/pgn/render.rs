use crate::pgn::pgn_move_tree_node::PgnMoveTreeNode;
use crate::pgn::{PgnMoveTree, PgnMoveTreeTraverser};
use indexmap::IndexMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::str::FromStr;
use crate::state::State;

impl PgnMoveTree {
    fn tags_pgn(&self) -> String {
        let mut res = String::new();
        for tag in self.tags.iter() {
            res += &*format!("[{} \"{}\"]\n", tag.0, tag.1);
        }
        res
    }

    fn pgn_helper(&self, should_render_variations: bool) -> String {
        let mut res = String::new();
        unsafe {
            res += &*format!("{}", (*self.head).pgn(should_render_variations, true, 0));
        }
        res
    }

    pub fn pgn(&self) -> String {
        if self.tags.is_empty() {
            return self.pgn_helper(true);
        }
        format!("{}\n{}", self.tags_pgn(), self.pgn_helper(true))
    }

    pub fn main_line_pgn(&self) -> String {
        self.pgn_helper(false)
    }
    
    pub fn traverser(&self) -> PgnMoveTreeTraverser {
        PgnMoveTreeTraverser::new(self)
    }
}

impl Display for PgnMoveTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pgn())
    }
}

impl Debug for PgnMoveTree {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tags_pgn())
    }
}