#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]

mod board;
mod attacks;
mod charboard;
mod masks;
mod magic;
mod state;
mod manual_attacks;
mod r#move;
mod miscellaneous;
mod zobrist;
mod pgn;
mod bitboard;
mod movegen;
mod fen;
mod perft;

use crate::charboard::*;
use crate::attacks::*;
use crate::board::Board;
use crate::pgn::PgnMoveTree;
use crate::state::State;

fn main() {
    let mut state = State::initial();
    loop {
        println!();
        println!("{}", state.to_fen());
        state.board.print();
        let moves = state.get_legal_moves();
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
        println!("Enter move (q to quit, n for new position from fen): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "q" {
            break;
        }
        if input == "n" {
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
            continue;
        }
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
