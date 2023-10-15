#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::utils::*;
use crate::attacks::*;
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

    let mut game = State::initial();
    game.board.wp = 0;
    println!("Game state:");
    game.board.print();
    println!();
    let moves = game.get_moves();
    for mv in moves {
        let (src_sq, dst_sq, flag) = mv.unpack();
        let src = 1 << (63 - src_sq);
        let dst = 1 << (63 - dst_sq);
        println!("source: {}", src_sq);
        pprint_bb(src);
        println!("destination: {}", dst_sq);
        pprint_bb(dst);
        println!();
    }
}
