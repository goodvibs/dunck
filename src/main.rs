// #![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]

use crate::state::State;

pub mod attacks;
pub mod state;
pub mod pgn;
pub mod perft;
pub mod r#move;
pub mod utils;
mod engine;

fn main() {
    let mut state = State::initial();
    loop {
        println!();
        println!("{}", state.to_fen());
        state.board.print();
        let moves = state.calc_legal_moves();
        let mut move_sans = Vec::with_capacity(moves.len());
        println!("Moves: ");
        for mv in moves.iter() {
            let initial_state = state.clone();
            let mut final_state = state.clone();
            final_state.make_move(*mv);
            assert!(final_state.is_valid());
            let san = mv.san(&initial_state, &final_state, &moves);
            move_sans.push(san.clone());
            print!("{}, ", san);
        }
        println!();
        println!("Enter move (q|QUIT to quit, n|NEW for new position from fen, b|BEST for best position according to engine): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        match input {
            "q" | "QUIT" => {
                break;
            },
            "n" | "NEW" => {
                loop {
                    println!("Enter fen (q to cancel): ");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).unwrap();
                    let input = input.trim();
                    if input == "q" {
                        break;
                    }
                    let state_result = State::from_fen(input);
                    match state_result {
                        Ok(s) => {
                            state = s;
                            assert!(state.is_valid());
                            break;
                        }
                        Err(e) => {
                            println!("{:?}", e);
                        }
                    }
                }
            }
            // "b" | "BEST" => {
            //     let tree_policy = engine::UctPolicy::new(2.0);
            //     let mut search_tree = engine::SearchTree::new(state.clone(), tree_policy);
            //     search_tree.run(500);
            //     let best_action = search_tree.get_best_action();
            //     println!("Best action: {}", best_action.unwrap());
            //     state.make_move(best_action.unwrap());
            // }
            _ => {
                let mut found = false;
                for i in 0..moves.len() {
                    if move_sans[i] == input {
                        state.make_move(moves[i]);
                        found = true;
                        break;
                    }
                }
                if !found {
                    println!("Invalid move");
                }
            }
        }
    }
}
