use crate::pgn::state_tree::PgnStateTree;
use crate::pgn::{tokenize_pgn, PgnParseError, PgnToken};

pub fn state_trees_from_pgn(pgn: &str) -> Result<Vec<PgnStateTree>, PgnParseError> {
    let mut trees = Vec::new();
    let tokens = tokenize_pgn(pgn)?;
    
    let mut start_idx = 0;
    
    for (idx, token) in tokens.iter().enumerate() {
        match token {
            PgnToken::Result(_) => {
                let tree_pgn = &tokens[start_idx..idx];
                let tree = PgnStateTree::from_tokens(tree_pgn)?;
                trees.push(tree);
                
                start_idx += 1;
            },
            _ => {},
        }
    }
    
    if start_idx < tokens.len() {
        trees.push(PgnStateTree::from_tokens(&tokens[start_idx..])?);
    }
    
    Ok(trees)
}