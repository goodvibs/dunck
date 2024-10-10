mod pgn_move_tree_node;
pub mod pgn_move_tree_traverser;
pub mod render;
pub mod parse;
pub mod tokenize;
pub mod error;

use indexmap::IndexMap;
pub use pgn_move_tree_traverser::PgnMoveTreeTraverser;
use crate::pgn::pgn_move_tree_node::PgnMoveTreeNode;

pub struct PgnMoveTree {
    pub tags: IndexMap<String, String>,
    pub head: *mut PgnMoveTreeNode,
}

impl PgnMoveTree {
    pub fn new() -> PgnMoveTree {
        PgnMoveTree {
            tags: IndexMap::new(),
            head: PgnMoveTreeNode::new_root()
        }
    }
}

impl Drop for PgnMoveTree {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.head));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::str::FromStr;
    use super::*;

    fn load_input_and_expected_pgn(file_name: &str) -> (String, String) {
        let input_pgn = fs::read_to_string(format!("src/pgn/test_pgn_files/{}.pgn", file_name)).expect("Could not read file");
        let expected_pgn = fs::read_to_string(format!("src/pgn/test_pgn_files/{}_formatted.pgn", file_name)).expect("Could not read file");
        (input_pgn, expected_pgn)
    }

    fn test_pgn(input_pgn: &str, expected_pgn: &str) {
        let pgn_tree = PgnMoveTree::from_str(input_pgn).unwrap();
        assert_eq!(pgn_tree.pgn(), expected_pgn);
    }

    fn generic_pgn_test(file_name: &str) {
        let (input_pgn, expected_pgn) = load_input_and_expected_pgn(file_name);
        test_pgn(&input_pgn, &expected_pgn);
    }

    #[test]
    fn empty_pgn_test() {
        let input_pgn = "";
        let pgn_tree = PgnMoveTree::from_str(input_pgn).unwrap();
        assert!(pgn_tree.tags.is_empty());
        unsafe {
            assert!((*pgn_tree.head).move_and_san_and_previous_node.is_none());
        }
        assert_eq!(pgn_tree.pgn(), "");
    }

    #[test]
    fn complex_pgn_test() {
        generic_pgn_test("complex");
    }
    
    #[test]
    fn rosen1_pgn_test() {
        generic_pgn_test("rosen1");
    }
}