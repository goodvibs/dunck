const INPUT_DIRECTORY: &str = "data/lichess_elite_db_multi_pgn";

use std::fs;
use std::str::FromStr;
use dunck::pgn::PgnStateTree;
use dunck::r#move::Move;
use dunck::state::State;
use dunck::utils::Color;

pub fn extract_pgns(multi_pgn_file_content: &str) -> Vec<String> {
    let mut pgns = Vec::new();
    let initial_split = multi_pgn_file_content.trim().split("\n\n");
    for split in initial_split {
        let split = split.trim();
        if split.starts_with("1.") {
            pgns.push(split.to_string());
        }
        else if !split.starts_with("[") {
            println!("Error in split: {}", split);
        }
    }
    pgns
}


pub struct TrainingItem {
    pub state: State,
    pub best_move: Move,
    pub winner: Option<Color>
}


fn main() {
    let mut num_pgns_read = 0;
    let mut num_invalid_pgns = 0;
    
    let paths = fs::read_dir(INPUT_DIRECTORY).unwrap();
    
    for path in paths {
        let path = path.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "pgn" {
                println!("Reading: {:?}", path);
                
                let mut num_pgns_read_for_file = 0;
                let mut num_invalid_pgns_for_file = 0;
                
                let buffer = fs::read_to_string(&path).unwrap();
                let pgns = extract_pgns(&buffer);
                num_pgns_read_for_file += pgns.len();
                
                for pgn in pgns {
                    let state_tree = match PgnStateTree::from_str(pgn.as_str()) {
                        Ok(state_tree) => state_tree,
                        Err(e) => {
                            num_invalid_pgns_for_file += 1;
                            // println!("Error: {:?}", e);
                            // println!("{}\n", pgn);
                            continue;
                        }
                    };
                }
                
                println!("Number of valid pgns read for file: {}", num_pgns_read_for_file - num_invalid_pgns_for_file);
                println!("Number of pgns read for file: {}", num_pgns_read_for_file);
                
                num_pgns_read += num_pgns_read_for_file;
                num_invalid_pgns += num_invalid_pgns_for_file;
            }
        }
    }
    println!("Number of valid pgns read: {}", num_pgns_read - num_invalid_pgns);
    println!("Number of pgns read: {}", num_pgns_read);
}