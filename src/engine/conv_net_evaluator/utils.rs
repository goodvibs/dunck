use lazy_static::lazy_static;
use tch::{Device, Kind, Tensor};
use crate::engine::conv_net_evaluator::constants::{MAX_RAY_LENGTH, NUM_PIECE_TYPE_BITS, NUM_POSITION_BITS, NUM_QUEEN_LIKE_MOVES, NUM_TARGET_SQUARE_POSSIBILITIES, NUM_UNDERPROMOTIONS, NUM_WAYS_OF_UNDERPROMOTION};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;
use crate::utils::{get_squares_from_mask_iter, Color, KnightMoveDirection, PieceType, QueenLikeMoveDirection, Square};

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
pub const fn get_policy_index_for_queen_like_move(direction: QueenLikeMoveDirection, distance: u8, promotion: Option<PieceType>) -> u8 {
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
        QueenLikeMoveDirection::Up => 0,
        QueenLikeMoveDirection::UpRight => 1,
        QueenLikeMoveDirection::UpLeft => 2,
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
    let dst_square = mv.get_destination().to_perspective_from_white(side_to_move);
    let src_square = mv.get_source().to_perspective_from_white(side_to_move);
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
        let (direction, distance) = QueenLikeMoveDirection::calc_and_measure_distance(src_square, dst_square);

        let promotion = if is_promotion {
            Some(unvetted_promotion)
        } else {
            None
        };

        get_policy_index_for_queen_like_move(direction, distance as u8, promotion)
    }
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

#[cfg(test)]
mod tests {
    use crate::attacks::single_knight_attacks;
    use super::*;

    #[test]
    fn test_is_knight_jump() {
        for src_square in Square::iter_all() {
            for dst_square in get_squares_from_mask_iter(single_knight_attacks(src_square)) {
                assert!(is_knight_jump(src_square, dst_square));
            }
        }
    }
    
    #[test]
    fn test_get_policy_index_for_sliding_pieces() {
        let mut used_indices = [false; NUM_QUEEN_LIKE_MOVES as usize];
        for direction in QueenLikeMoveDirection::iter() {
            for distance in 1..=MAX_RAY_LENGTH {
                let index = get_policy_index_for_queen_like_move(direction, distance, None);
                assert!(index < NUM_QUEEN_LIKE_MOVES);
                assert!(!used_indices[index as usize]);
                used_indices[index as usize] = true;
            }
        }
        assert!(used_indices.iter().all(|&used| used));
    }

    #[test]
    fn test_get_policy_index_for_pawns() {
        // TODO
    }
    
    #[test]
    fn test_get_policy_index_for_knight_move() {
        // TODO
    }
    
    #[test]
    fn test_get_policy_index_for_move() {
        // TODO
    }

    #[test]
    fn test_state_to_tensor() {
        // TODO
    }
}