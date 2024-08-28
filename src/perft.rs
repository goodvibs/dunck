#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::iter::zip;
    use std::str::FromStr;
    use chess;
    use crate::miscellaneous::{PieceType, Square};
    use crate::r#move::{Move, MoveFlag};
    use crate::state::State;

    #[test]
    fn test_chess() {
        let board = chess::Board::default();
        let movegen = chess::MoveGen::new_legal(&board);
        assert_eq!(movegen.len(), 20);
    }

    #[test]
    fn test_initial() {
        let board = chess::Board::default();
        let movegen = chess::MoveGen::new_legal(&board);
        let state = State::initial();
        let possible_moves = state.get_legal_moves();
        assert_eq!(movegen.len(), possible_moves.len());
    }

    fn are_squares_equal(square: Square, chess_square: chess::Square) -> bool {
        let file = square.get_file();
        let rank = square.get_rank();
        let chess_square_number = rank * 8 + file;
        let created_chess_square = unsafe { chess::Square::new(chess_square_number) };
        created_chess_square == chess_square
    }

    fn are_moves_equal(mv: Move, chess_mv: chess::ChessMove) -> bool {
        let uci = mv.uci();
        let flag = mv.get_flag();
        let created_chess_mv = chess::ChessMove::from_str(uci.as_str()).unwrap();
        created_chess_mv == chess_mv
    }

    #[test]
    fn test_are_moves_equal() {
        let mv = Move::new(Square::E4, Square::E2, Move::DEFAULT_PROMOTION_VALUE, MoveFlag::NormalMove);
        let chess_mv = chess::ChessMove::from_str("e2e4").unwrap();
        assert!(are_moves_equal(mv, chess_mv));

        let mv = Move::new(Square::H8, Square::G7, PieceType::Knight, MoveFlag::Promotion);
        let chess_mv = chess::ChessMove::from_str("g7h8n").unwrap();
        assert!(are_moves_equal(mv, chess_mv));

        let mv = Move::new(Square::F1, Square::F2, PieceType::Bishop, MoveFlag::Promotion);
        let chess_mv = chess::ChessMove::from_str("f2f1b").unwrap();
        assert!(are_moves_equal(mv, chess_mv));
    }

    fn count_moves_and_test(state: &State, validation_board: chess::Board, depth: u32) -> (u64, u64) {
        if depth == 0 {
            return (1, 1);
        }

        let found_fen = state.to_fen(); // for debugging
        let expected_fen = validation_board.to_string(); // for debugging
        // assert_eq!(found_fen, expected_fen);
        let found_fen_split = found_fen.split_ascii_whitespace().collect::<Vec<_>>();
        let expected_fen_split = expected_fen.split_ascii_whitespace().collect::<Vec<_>>();
        let (found_fen_board, found_fen_side_to_move, found_fen_castling_rights) = (found_fen_split[0], found_fen_split[1], found_fen_split[2]);
        let (expected_fen_board, expected_fen_side_to_move, expected_fen_castling_rights) = (expected_fen_split[0], expected_fen_split[1], expected_fen_split[2]);
        assert_eq!(found_fen_split[0..3], expected_fen_split[0..3]); // ensure board, side to move, and castling rights are the same

        let found_moves_unordered = state.get_legal_moves();
        let expected_moves = chess::MoveGen::new_legal(&validation_board);
        let mut found_moves_ordered;

        let (mut found_count, mut known_count) = (found_moves_unordered.len() as u64, expected_moves.len() as u64);

        found_moves_ordered = Vec::with_capacity(found_count as usize);
        for expected_move in expected_moves {
            let expected_move_uci = expected_move.to_string(); // for debugging
            let mut corresponding_move = None;
            for found_move in &found_moves_unordered {
                if are_moves_equal(*found_move, expected_move) {
                    assert!(corresponding_move.is_none());
                    corresponding_move = Some(*found_move);
                    break;
                }
            }
            match corresponding_move {
                None => {
                    let moves_uci = found_moves_unordered.iter().map(|mv| mv.uci()).collect::<Vec<_>>(); // for debugging
                    panic!()
                },
                Some(found_move) => {
                    assert!(!found_moves_ordered.contains(&found_move));
                    found_moves_ordered.push(found_move);
                }
            }
        }
        if found_moves_ordered.len() != found_moves_unordered.len() {
            let found_moves_set = found_moves_unordered.iter().collect::<HashSet<_>>();
            let expected_moves_set = found_moves_ordered.iter().collect::<HashSet<_>>();
            let missing_moves = found_moves_set.difference(&expected_moves_set).collect::<Vec<_>>();
            let missing_moves_uci = missing_moves.iter().map(|mv| mv.uci()).collect::<Vec<_>>(); // for debugging
            assert!(missing_moves.is_empty());
            panic!();
        }

        for (found_move, expected_move) in zip(found_moves_ordered, chess::MoveGen::new_legal(&validation_board)) {
            let mut new_state = state.clone();
            new_state.make_move(found_move);
            let mut new_validation_board = validation_board.clone();
            validation_board.make_move(expected_move, &mut new_validation_board);
            let (found_count_inc, known_count_inc) = count_moves_and_test(&new_state, new_validation_board, depth - 1);
            found_count += found_count_inc;
            known_count += known_count_inc;
        }

        assert_eq!(found_count, known_count);
        (found_count, known_count)
    }
    
    fn generic_depth_test(fen: Option<&str>, depth: u32) {
        let (state, validation_board) = match fen {
            Some(fen) => {
                let state = State::from_fen(fen).unwrap();
                let validation_board = chess::Board::from_str(fen).unwrap();
                (state, validation_board)
            }
            None => {
                let state = State::initial();
                let validation_board = chess::Board::default();
                (state, validation_board)
            }
        };
        let possible_moves = state.get_legal_moves();
        let (found_count, known_count) = count_moves_and_test(&state, validation_board, depth);
        assert_eq!(found_count, known_count);
        println!("{} moves", found_count);
    }

    #[test]
    fn test_initial_depth_4() {
        generic_depth_test(None, 4);
    }
    
    #[test]
    fn test_arb_depth_5() {
        let fen = "1k6/1P6/3P1N2/2K3R1/8/3p3B/8/8 w - - 0 1";
        generic_depth_test(Some(fen), 5);
    }
    
    #[test]
    fn test_kiwipete_depth_4() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        generic_depth_test(Some(fen), 4);
    }

    #[test]
    fn test_p3_depth_5() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        generic_depth_test(Some(fen), 5);
    }

    #[test]
    fn test_p4_depth_4() {
        let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
        generic_depth_test(Some(fen), 4);
    }

    #[test]
    fn test_p5_depth_4() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        generic_depth_test(Some(fen), 4);
    }

    #[test]
    fn test_p6_depth_4() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        generic_depth_test(Some(fen), 4);
    }
}