use lazy_static::lazy_static;
use tch::{Device, Kind, Tensor};
use crate::engine::conv_net_evaluator::constants::{MAX_RAY_LENGTH, NUM_PIECE_TYPE_BITS, NUM_POSITION_BITS, NUM_QUEEN_LIKE_MOVES, NUM_TARGET_SQUARE_POSSIBILITIES, NUM_UNDERPROMOTIONS, NUM_WAYS_OF_UNDERPROMOTION};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;
use crate::utils::{get_squares_from_mask_iter, Color, KnightMoveDirection, PieceType, QueenMoveDirection, Square};

lazy_static! {
    pub static ref DEVICE: Device = Device::cuda_if_available();
}

/// Checks if a move is a knight move based on its source and destination squares.
pub const fn is_knight_jump(src_square: Square, dst_square: Square) -> bool {
    // Calculate the difference in rank and file between the source and destination
    let rank_diff = (dst_square.get_rank() as i8 - src_square.get_rank() as i8).abs();
    let file_diff = (dst_square.get_file() as i8 - src_square.get_file() as i8).abs();

    // A knight move is either (±2, ±1) or (±1, ±2)
    (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
}

/// Maps a queen-like move to an index in the policy tensor's 73 possible moves per square.
/// Index is between 0 and 64 for queen-like moves (56 different target squares, 9 possible underpromotions).
pub const fn get_policy_index_for_queen_like_move(direction: QueenMoveDirection, distance: u8, promotion: Option<PieceType>) -> u8 {
    // Calculate the index based on the direction and distance
    let direction_index = direction as u8;
    let distance_index = distance - 1; // Distance is 1-indexed

    let promotion_index = match promotion {
        Some(PieceType::Knight) => 0,
        Some(PieceType::Bishop) => 1,
        Some(PieceType::Rook) => 2,
        _ => return direction_index * MAX_RAY_LENGTH + distance_index,
    };

    let promotion_direction_index = match direction {
        QueenMoveDirection::Up => 0,
        QueenMoveDirection::UpRight => 1,
        QueenMoveDirection::UpLeft => 2,
        _ => panic!()
    };

    NUM_QUEEN_LIKE_MOVES + promotion_direction_index * NUM_UNDERPROMOTIONS + promotion_index
}

/// Maps a knight move to an index in the policy tensor's 73 possible moves per square.
/// Index is between 65 and 72 for knight moves (8 possible moves).
pub const fn get_policy_index_for_knight_move(direction: KnightMoveDirection) -> u8 {
    direction as u8 + NUM_QUEEN_LIKE_MOVES + NUM_WAYS_OF_UNDERPROMOTION
}

/// Maps a move to an index in the policy tensor's 73 possible moves per square.
pub const fn get_policy_index_for_move(mv: &Move, side_to_move: Color) -> u8 {
    // Extract destination, source, promotion, and flag from the move
    let dst_square = match side_to_move {
        Color::White => mv.get_destination(),
        Color::Black => mv.get_destination().rotated_perspective()
    };
    let src_square = match side_to_move {
        Color::White => mv.get_source(),
        Color::Black => mv.get_source().rotated_perspective()
    };
    let unvetted_promotion = mv.get_promotion();
    let flag = mv.get_flag();
    
    let (is_normal_move, is_promotion) = match flag {
        MoveFlag::NormalMove => (true, false),
        MoveFlag::Promotion => (false, true),
        _ => (false, false)
    };

    if is_normal_move && is_knight_jump(src_square, dst_square) {
        // Knight move
        get_policy_index_for_knight_move(KnightMoveDirection::calc(src_square, dst_square))
    } else {
        // Queen-like move
        let (direction, distance) = QueenMoveDirection::calc_and_measure_distance(src_square, dst_square);

        let promotion = if is_promotion {
            Some(unvetted_promotion)
        } else {
            None
        };

        get_policy_index_for_queen_like_move(direction, distance as u8, promotion)
    }
}

/// Generates a move mask tensor, marking legal moves with 1 and others with 0.
pub fn get_move_mask(moves: &Vec<Move>, side_to_move: Color) -> Tensor {
    // Initialize a mask tensor with shape [8, 8, 73] (8x8 board, 73 possible moves)
    let mask = Tensor::zeros(&[8, 8, NUM_TARGET_SQUARE_POSSIBILITIES as i64], (Kind::Float, *DEVICE));

    for mv in moves {
        // Get the source square from which the move is made
        let src_square = match side_to_move {
            Color::White => mv.get_source(),
            Color::Black => mv.get_source().rotated_perspective()
        };

        // Determine the policy index using get_policy_index_for_move
        let policy_index = get_policy_index_for_move(mv, side_to_move);

        // Set the mask at the corresponding source square and policy index to 1
        let _ = mask.get(src_square.get_rank() as i64)
            .get(src_square.get_file() as i64)
            .get(policy_index as i64)
            .fill_(1.0);
    }

    mask
}

pub fn state_to_tensor(state: &State) -> Tensor {
    // Initialize a tensor with shape [17, 8, 8], where:
    // - 17 is the number of channels
    // - 8x8 is the board size
    let tensor = Tensor::zeros(&[NUM_POSITION_BITS as i64, 8, 8], (Kind::Float, *DEVICE));

    // Determine if we need to rotate the board
    let rotate = state.side_to_move == Color::Black;

    // Channels 0-11: Piece types for both colors
    for piece_type in PieceType::iter_pieces() {
        // Get the bitboard mask for the specific piece type and color
        let player_piece_type_mask = state.board.color_masks[state.side_to_move as usize] & state.board.piece_type_masks[piece_type as usize];
        let opponent_piece_type_mask = state.board.color_masks[state.side_to_move.flip() as usize] & state.board.piece_type_masks[piece_type as usize];

        // Channels 0-5: Player's pieces
        for square in get_squares_from_mask_iter(player_piece_type_mask) {
            let square_from_unified_perspective = if rotate {
                square.rotated_perspective()
            } else {
                square
            };
            let _ = tensor
                .get(piece_type as i64 - PieceType::Pawn as i64)
                .get(square_from_unified_perspective.get_rank() as i64)
                .get(square_from_unified_perspective.get_file() as i64)
                .fill_(1.);
        }

        // Channels 6-11: Opponent's pieces
        for square in get_squares_from_mask_iter(opponent_piece_type_mask) {
            let square_from_unified_perspective = if rotate {
                square.rotated_perspective()
            } else {
                square
            };
            let _ = tensor
                .get(NUM_PIECE_TYPE_BITS as i64 + piece_type as i64 - PieceType::Pawn as i64)
                .get(square_from_unified_perspective.get_rank() as i64)
                .get(square_from_unified_perspective.get_file() as i64)
                .fill_(1.);
        }
    }

    // Channel 12: Side to move (1 if white to move, 0 if black to move)
    let _ = tensor.get(12).fill_(
        if state.side_to_move == Color::White { 1. } else { 0. }
    );

    // Channel 13-16: Castling rights
    let castling_rights = state.context.borrow().castling_rights;
    let _ = tensor.get(13).fill_(
        if castling_rights & 0b1000 != 0 { 1. } else { 0. }
    );
    let _ = tensor.get(14).fill_(
        if castling_rights & 0b0100 != 0 { 1. } else { 0. }
    );
    let _ = tensor.get(15).fill_(
        if castling_rights & 0b0010 != 0 { 1. } else { 0. }
    );
    let _ = tensor.get(16).fill_(
        if castling_rights & 0b0001 != 0 { 1. } else { 0. }
    );

    tensor
}

pub fn renormalize_policy(policy_output: Tensor, legal_move_mask: Tensor) -> Tensor {
    // Apply the mask to zero out illegal moves
    let masked_policy = policy_output * &legal_move_mask;

    // Sum the masked probabilities to get the total probability of legal moves
    let sum_legal_probs_tensor = masked_policy.sum(Kind::Float);
    let sum_legal_probs = sum_legal_probs_tensor.double_value(&[]);

    // Avoid division by zero in case all moves are illegal
    if sum_legal_probs > 0. {
        // Renormalize the masked probabilities by dividing by the total sum
        masked_policy / sum_legal_probs
    } else {
        // If there are no legal moves, return the mask itself as probabilities (all zero)
        legal_move_mask
    }
}

#[cfg(test)]
mod tests {
    use chess::Piece;
    use crate::attacks::single_knight_attacks;
    use crate::engine::conv_net_evaluator::constants::{MAX_RAY_LENGTH, NUM_POSITION_BITS};
    use crate::engine::conv_net_evaluator::utils::{is_knight_jump, state_to_tensor};
    use crate::state::{Board, State};
    use crate::utils::{get_squares_from_mask_iter, Color, ColoredPiece, PieceType, QueenMoveDirection, Square};

    #[test]
    fn test_is_knight_jump() {
        for src_square in Square::iter_all() {
            for dst_square in get_squares_from_mask_iter(single_knight_attacks(src_square)) {
                assert!(is_knight_jump(src_square, dst_square));
            }
        }
    }
    
    #[test]
    fn test_get_policy_index_for_queen_like_move() {
        let src_square = Square::A1;
        let dst_square = Square::H8;

        for direction in QueenMoveDirection::iter() {
            for distance in 1..=MAX_RAY_LENGTH {
                for promotion in PieceType::iter_promotion_pieces() {
                    let index = super::get_policy_index_for_queen_like_move(direction, distance, Some(promotion));
                    assert!(index < 73);
                }
            }
        }
    }

    #[test]
    fn test_state_to_tensor() {
        let state = State::initial();
        let tensor = state_to_tensor(&state);

        assert_eq!(tensor.size(), [NUM_POSITION_BITS as i64, 8, 8]);
    }
}