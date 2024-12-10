use std::cell::RefCell;
use std::rc::Rc;
use std::str::FromStr;
use indexmap::IndexMap;
use crate::pgn::state_tree_node::{PgnStateTreeNode};
use crate::pgn::{tokenize_pgn, PgnParseError};

pub struct PgnStateTree {
    pub tags: IndexMap<String, String>,
    pub head: Rc<RefCell<PgnStateTreeNode>>,
}

impl PgnStateTree {
    pub fn new() -> PgnStateTree {
        PgnStateTree {
            tags: IndexMap::new(),
            head: PgnStateTreeNode::new_root()
        }
    }
}

impl FromStr for PgnStateTree {
    type Err = PgnParseError;

    fn from_str(pgn: &str) -> Result<PgnStateTree, PgnParseError> {
        let tokens = tokenize_pgn(pgn)?;
        PgnStateTree::from_tokens(&tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::str::FromStr;
    use super::*;

    fn load_input_and_expected_pgn(file_name: &str) -> (String, String) {
        let input_pgn = fs::read_to_string(format!("data/pgn_test_files/{}.pgn", file_name)).expect("Could not read file");
        let expected_pgn = fs::read_to_string(format!("data/pgn_test_files/{}_formatted.pgn", file_name)).expect("Could not read file");
        (input_pgn, expected_pgn)
    }

    fn test_pgn(input_pgn: &str, expected_pgn: &str) {
        let pgn_tree = PgnStateTree::from_str(input_pgn).unwrap();
        assert_eq!(pgn_tree.to_string(), expected_pgn);
    }

    fn generic_pgn_test(file_name: &str) {
        let (input_pgn, expected_pgn) = load_input_and_expected_pgn(file_name);
        test_pgn(&input_pgn, &expected_pgn);
    }

    #[test]
    fn empty_pgn_test() {
        let input_pgn = "";
        let pgn_tree = PgnStateTree::from_str(input_pgn).unwrap();
        assert!(pgn_tree.tags.is_empty());
        assert!((*pgn_tree.head).borrow().move_and_san_and_previous_node.is_none());
        assert_eq!(pgn_tree.to_string(), "");
    }

    #[test]
    fn complex_pgn_test() {
        generic_pgn_test("complex");
    }

    #[test]
    fn rosen1_pgn_test() {
        generic_pgn_test("rosen1");
    }
    
    #[test]
    fn amirkhafan_vs_trickortreat_pgn_test() {
        generic_pgn_test("amirkhafan_vs_trickortreat");
    }

    #[test]
    fn blitzstream_twitch_vs_amirkhafan_test() {
        generic_pgn_test("blitzstream-twitch_vs_amirkhafan");
    }
    
    #[test]
    fn pinhead_larry_vs_orlando_gloom_test() {
        generic_pgn_test("pinhead-larry_vs_orlando_gloom");
    }
}