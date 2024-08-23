#[cfg(test)]
mod tests {
    use std::iter::zip;
    use chess;
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

    fn count_moves_and_test(state: &State, validation_board: chess::Board, depth: u32) -> (u64, u64) {
        if depth == 0 {
            return (1, 1);
        }
        
        let moves_found = state.get_legal_moves();
        let moves_known = chess::MoveGen::new_legal(&validation_board);

        let mut found_count = 0;
        let mut known_count = 0;
        
        if depth == 1 {
            (found_count, known_count) = (moves_found.len() as u64, moves_known.len() as u64);
        }
        else {
            for (found_move, known_move) in zip(moves_found, moves_known) {
                let mut new_state = state.clone();
                new_state.make_move(found_move);
                let mut new_validation_board = validation_board.clone();
                validation_board.make_move(known_move, &mut new_validation_board);
                let (found_count_inc, known_count_inc) = count_moves_and_test(&new_state, new_validation_board, depth - 1);
                found_count += found_count_inc;
                known_count += known_count_inc;
            }
        }
        
        assert_eq!(found_count, known_count);
        (found_count, known_count)
    }

    #[test]
    fn test_initial_depth_3() {
        let state = State::initial();
        let validation_board = chess::Board::default();
        let possible_moves = state.get_legal_moves();
        let (found_count, known_count) = count_moves_and_test(&state, validation_board, 3);
        assert_eq!(found_count, known_count);
    }
}