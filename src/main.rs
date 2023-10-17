#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::utils::*;
use crate::attacks::*;
use crate::board::Board;
use crate::state::State;

mod board;
mod attacks;
mod utils;
mod preload;
mod masks;
mod magic;
mod state;
mod manual_attacks;
mod r#move;
mod squares;

fn main() {
    // let cb: Charboard = [
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', 'x', ' ']
    // ];
    // let bb = cb_to_bb(&cb);
    // let occupied_cb: Charboard = [
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', 'x', ' ']
    // ];
    // let occupied = cb_to_bb(&occupied_cb);
    // let attacks = knight_attacks(bb);
    // pprint_bb(attacks);

    // let mut game = State::initial();
    // loop {
    //     println!("Game state:");
    //     game.board.print();
    //     println!();
    //     let moves = game.get_moves();
    //     for (i, mv) in moves.iter().enumerate() {
    //         let (from, to, info) = mv.to_readable();
    //         println!("{}: {}{} {}", i, from, to, info);
    //     }
    //     println!();
    //     println!("Which move?");
    //     let mut input = String::new();
    //     std::io::stdin().read_line(&mut input).unwrap();
    //     let input = input.trim();
    //     let mv = moves[input.parse::<usize>().unwrap()];
    //     game.play_move(mv);
    //     println!();
    // }

    let cb: Charboard = [
        ['r', 'n', ' ', ' ', 'k', ' ', ' ', 'r'],
        ['p', ' ', ' ', ' ', 'p', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
        [' ', ' ', ' ', ' ', ' ', ' ', 'P', 'p'],
        [' ', 'B', ' ', ' ', ' ', ' ', ' ', ' '],
        ['P', ' ', ' ', ' ', 'P', ' ', ' ', 'P'],
        ['R', 'N', 'B', ' ', 'K', ' ', ' ', 'R']
    ];
    let mut game = State::initial();
    game.board = Board::from_cb(cb);
    game.double_pawn_push = 6;
    game.turn = Color::Black;
    game.board.print();
    let moves = game.get_moves();
    for (i, mv) in moves.iter().enumerate() {
        let (from, to, info) = mv.to_readable();
        println!("{}: {}{} {}", i, from, to, info);
    }
}
