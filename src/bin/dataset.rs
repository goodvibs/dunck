const INPUT_DIRECTORY: &str = "data/lichess_elite_db_multi_pgn";

use std::fs;
use dunck::pgn::{tokenize_pgn, PgnParseError, PgnStateTree, PgnToken};

pub fn state_trees_from_pgn(pgn: &str) -> Result<Vec<PgnStateTree>, PgnParseError> {
    let mut trees = Vec::new();
    let tokens = tokenize_pgn(pgn)?;

    let mut start_idx = 0;

    for (idx, token) in tokens.iter().enumerate() {
        match token {
            PgnToken::Result(_) => {
                let tree_pgn = &tokens[start_idx..idx];
                let tree = match PgnStateTree::from_tokens(tree_pgn) {
                    Ok(tree) => tree,
                    Err(err) => { 
                        eprintln!("Error parsing PGN: {}", err);
                        eprintln!("PGN: {:?}", tree_pgn);
                        return Err(err);
                    },
                };
                trees.push(tree);

                start_idx = idx + 1;
            },
            _ => {},
        }
    }

    if start_idx < tokens.len() {
        trees.push(PgnStateTree::from_tokens(&tokens[start_idx..])?);
    }

    Ok(trees)
}

fn main() {
    let paths = fs::read_dir(INPUT_DIRECTORY).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "pgn" {
                let buffer = fs::read_to_string(&path).unwrap();
                let trees = match state_trees_from_pgn(&buffer) {
                    Ok(trees) => {
                        println!("Parsed {} trees from file: {:?}", trees.len(), path);
                        trees
                    },
                    Err(err) => {
                        eprintln!("Error occurred in file: {:?}", path);
                        continue;
                    }
                };
            }
        }
    }
}