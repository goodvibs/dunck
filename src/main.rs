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

    // let cb: Charboard = [
    //     ['r', 'n', ' ', ' ', 'k', ' ', ' ', 'r'],
    //     ['p', ' ', ' ', ' ', 'p', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '],
    //     [' ', ' ', ' ', ' ', ' ', ' ', 'P', 'p'],
    //     [' ', 'B', ' ', ' ', ' ', ' ', ' ', ' '],
    //     ['P', ' ', ' ', ' ', 'P', ' ', ' ', 'P'],
    //     ['R', 'N', 'B', ' ', 'K', ' ', ' ', 'R']
    // ];
    // let mut game = State::initial();
    // game.board = Board::from_cb(cb);
    // game.double_pawn_push = 6;
    // game.turn = Color::Black;
    // game.board.print();
    // let moves = game.get_moves();
    // for (i, mv) in moves.iter().enumerate() {
    //     let (from, to, info) = mv.to_readable();
    //     println!("{}: {}{} {}", i, from, to, info);
    // }

    let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6
4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5
Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5
hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5
35. Ra7 g6 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6
Nf2 42. g4 Bd3 43. Re6";
    let (game, moves) = State::from_pgn(pgn);
    game.board.print();

//     let pgn = "1. e4 e5 2. Nf3 Nc6 3. Bb5 a6
// 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
// 11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5
// Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
// 23. Ne5 Rae8 24. Bxf7+ Rxf7";
//     let (game, moves) = State::from_pgn(pgn);
//     game.board.print();
//     let possible_moves = game.get_moves();
//     for (i, mv) in possible_moves.iter().enumerate() {
//         let (from, to, info) = mv.to_readable();
//         println!("{}: {}{} {}", i, from, to, info);
//     }
}
