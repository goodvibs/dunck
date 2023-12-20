#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use crate::utils::*;
use crate::attacks::*;
use crate::board::Board;
use crate::history::History;
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
mod consts;
mod zobrist;
mod history;

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

    let pgn = "[Event \"Wch1\"]
[Site \"U.S.A.\"]
[Date \"1886.??.??\"]
[Round \"9\"]
[White \"Zukertort, Johannes\"]
[Black \"Steinitz, Wilhelm\"]
[Result \"0-1\"]
[ECO \"D26h\"]
[Annotator \"JvR\"]

1.d4 d5 2.c4 e6 3.Nc3 Nf6 4.Nf3 dxc4 5.e3 c5 6.Bxc4 cxd4 7.exd4 Be7 8.O-O
O-O 9.Qe2 Nbd7 {This knight wants to blockades on d5.} 10.Bb3 Nb6 11.Bf4
( 11.Re1 {keeps the initiative.} )
11...Nbd5 12.Bg3 Qa5 13.Rac1 Bd7 14.Ne5 Rfd8 15.Qf3 Be8 16.Rfe1 Rac8 17.
Bh4 {Intends 18.Nxd5 exd5.} 17...Nxc3 18.bxc3 Qc7 {Black pressures on the
hanging pawns.} 19.Qd3
( 19.Bg3 {!} 19...Bd6 20.c4 {(Lasker).} )
19...Nd5 20.Bxe7 Qxe7 21.Bxd5 {?!}
( 21.c4 Qg5 22.Rcd1 Nf4 23.Qg3 {steers towards a slight advantage in
the endgame.} )
21...Rxd5 22.c4 Rdd8 23.Re3 {The attack will fail.}
( 23.Rcd1 {is solid.} )
23...Qd6 24.Rd1 f6 25.Rh3 {!?} 25...h6 {!}
( 25...fxe5 26.Qxh7+ Kf8 27.Rg3 {!} 27...Rd7
( 27...Rc7 28.Qh8+ Ke7 29.Rxg7+ Bf7 30.Qh4+ {(Euwe)} )
28.Qh8+ Ke7 29.Qh4+ Kf7 30.Qh7 {} )
26.Ng4 Qf4 {!} 27.Ne3 Ba4 {!} 28.Rf3 Qd6 29.Rd2
( 29.Rxf6 {?} 29...Bxd1 {!} )
29...Bc6 {?}
( 29...b5 {!} 30.Qg6 {!?}
( 30.cxb5 Rc1+ 31.Nd1 Qxd4 32.Qxd4 Rxd4 33.Rxd4 Bxd1 $19 {
(Vukovic).} )
30...Qf8 31.Ng4 Rxc4 {!} 32.Nxh6+ Kh8 33.h3 gxh6 34.Rxf6 Qg7 {is good
for Black).} )
30.Rg3 {?}
( 30.d5 {!} 30...Qe5 {!}
( 30...exd5 {(Steinitz)} 31.Nf5 {(Euwe)} )
31.Qb1 {Forestalls ..b5 and protects the first rank.} 31...exd5 32.
cxd5 {} 32...Bxd5 {??} 33.Rf5 )
30...f5 {Threatens ..f4.} 31.Rg6 {!?}
( 31.Nd1 f4 32.Rh3 e5 {!} 33.d5 Bd7 $19 )
31...Be4 32.Qb3 Kh7
( 32...Kf7 {(protects e6)} 33.c5 Qe7 {!} 34.Rg3 f4 )
33.c5 Rxc5 34.Rxe6
( 34.Qxe6 Rc1+ $19 )
34...Rc1+ 35.Nd1
( 35.Nf1 Qc7 $19 {!} )
35...Qf4 36.Qb2 Rb1 37.Qc3 Rc8 {Utilises the unprotected first rank.} 38.
Rxe4 Qxe4 {Many authors praise the high level of this positional game. The
score had become 4-4. The match continued in New Orleans.}";

    // let (_, moves) = State::from_pgn(pgn);
    // let mut game = State::initial();
    // game.board.print();
    // println!();
    // for mv in moves {
    //     game.play_move(mv);
    //     println!("{}", mv);
    //     println!("{}", game.board);
    //     println!();
    // }

    // let history = History::from_pgn(pgn);
    // println!();
    // match history {
    //     Ok(hist) => {
    //         for tag in hist.tags {
    //             println!("{}", tag);
    //         }
    //         println!("{}", (*hist.head.unwrap()).borrow().final_state.board);
    //     }
    //     Err(parse_error) => {
    //         println!("{:?}", parse_error);
    //     }
    // }

    let fen = "3r1q1k/p5p1/4ppQN/1p6/b1rP4/5R1P/P2R1PP1/6K1 b - - 0 33";
    let game = State::from_fen(fen).unwrap();
    game.board.print();
    let moves = game.get_moves();
    for (i, mv) in moves.iter().enumerate() {
        let (from, to, info) = mv.to_readable();
        println!("{}: {}{} {}", i, from, to, info);
    }
}
