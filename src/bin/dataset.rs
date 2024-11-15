const INPUT_DIRECTORY: &str = "data/lichess_elite_db_multi_pgn";

use std::fs;
use dunck::pgn::state_trees_from_pgn;

fn main() {
    let paths = fs::read_dir(INPUT_DIRECTORY).unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "pgn" {
                let buffer = fs::read_to_string(&path).unwrap();
                let trees = state_trees_from_pgn(&buffer).unwrap();
            }
        }
    }
}