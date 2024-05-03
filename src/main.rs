#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]

mod board;
mod attacks;
mod charboard;
mod preload;
mod masks;
mod magic;
mod state;
mod manual_attacks;
mod r#move;
mod enums;
mod zobrist;
mod pgn;
mod bitboard;

use crate::charboard::*;
use crate::attacks::*;
use crate::board::Board;
use crate::pgn::PgnMoveTree;
use crate::state::State;

fn main() {
    let mut state = State::initial();
    loop {
        state.board.print();
        let moves = state.get_pseudolegal_moves();
        println!("Moves: ");
        for mv in moves.iter() {
            let initial_state = state.clone();
            let mut final_state = state.clone();
            final_state.play_move(*mv);
            print!("{} ", mv.san(&initial_state, &final_state));
        }
        println!();
        println!("Enter move (q to quit): ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "q" {
            break;
        }
        let mut found = false;
        for mv in moves.iter() {
            if mv.matches(input) {
                state.play_move(*mv);
                found = true;
                break;
            }
        }
        if !found {
            println!("Invalid move");
        }
    }
}
