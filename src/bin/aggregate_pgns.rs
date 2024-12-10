const INPUT_DIRECTORY: &str = "data/lichess_elite_db_multi_pgn";

use std::fs;
use std::str::FromStr;
use dunck::pgn::{tokenize_pgn, PgnStateTree, PgnToken};
use dunck::r#move::Move;
use dunck::state::State;
use dunck::utils::Color;

fn extract_pgns(multi_pgn_file_content: &str, num_read: &mut usize) -> Vec<String> {
    let mut pgns = Vec::new();
    let initial_split = multi_pgn_file_content.trim().split("\n\n");
    for split in initial_split {
        let split = split.trim();
        if split.starts_with("1.") {
            *num_read += 1;
            if quick_validate_pgn(split) {
                pgns.push(split.to_string());
            }
        }
        else if !split.starts_with("[") {
            println!("Error in split: {}", split);
        }
    }
    pgns
}


fn quick_validate_pgn(pgn: &str) -> bool {
    let tokens = match tokenize_pgn(pgn) {
        Ok(tokens) => tokens,
        Err(_) => return false
    };
    
    let acceptable_results: [PgnToken; 3] = [
        PgnToken::Result("1-0".to_string()),
        PgnToken::Result("0-1".to_string()),
        PgnToken::Result("1/2-1/2".to_string())
    ];
    
    tokens.len() > 10 && acceptable_results.contains(tokens.last().unwrap())
}


fn write_to_file(file_path: &str, pgns: Vec<String>) {
    let content = pgns.join("\n\n");
    fs::write(file_path, content).unwrap();
}


pub struct TrainingItem {
    pub state: State,
    pub best_move: Move,
    pub winner: Option<Color>
}


fn main() {
    let mut num_pgns_read = 0;
    let mut num_accepted_pgns = 0;
    
    let paths = fs::read_dir(INPUT_DIRECTORY).unwrap();
    let mut accepted_pgns = Vec::new();
    
    for path in paths {
        let path = path.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "pgn" {
                println!("Reading: {:?}", path);
                let buffer = fs::read_to_string(&path).unwrap();

                let mut num_pgns_read_from_file = 0;
                let pgns_from_file = extract_pgns(&buffer, &mut num_pgns_read_from_file);
                let num_accepted_pgns_from_file = pgns_from_file.len();
                
                accepted_pgns.extend(pgns_from_file);
                
                println!("Number of pgns accepted from file: {}", num_accepted_pgns_from_file);
                println!("Number of pgns read from file: {}", num_pgns_read_from_file);
                println!();
                
                num_pgns_read += num_pgns_read_from_file;
                num_accepted_pgns += num_accepted_pgns;
            }
        }
    }
    println!("Number of pgns accepted: {}", num_accepted_pgns);
    println!("Number of pgns read: {}", num_pgns_read);
    
    let output_file_path = "data/lichess_elite_db_multi_pgn/accepted.pgn";
    write_to_file(output_file_path, accepted_pgns);
}