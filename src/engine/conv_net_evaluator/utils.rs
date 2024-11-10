use lazy_static::lazy_static;
use tch::{Device, Kind, Tensor};
use crate::engine::conv_net_evaluator::constants::{MAX_RAY_LENGTH, NUM_BITS_PER_BOARD, NUM_PIECE_TYPE_BITS, NUM_POSITION_BITS, NUM_QUEEN_LIKE_MOVES, NUM_SIDE_TO_MOVE_BITS, NUM_TARGET_SQUARE_POSSIBILITIES, NUM_UNDERPROMOTIONS, NUM_WAYS_OF_UNDERPROMOTION};
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
/// Assumes that the direction is from the perspective of the current player.
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
/// Assumes that the direction is from the perspective of the current player.
pub const fn get_policy_index_for_knight_move(direction: KnightMoveDirection) -> u8 {
    direction as u8 + NUM_QUEEN_LIKE_MOVES + NUM_WAYS_OF_UNDERPROMOTION
}

/// Maps a move to an index in the policy tensor's 73 possible moves per square.
/// Assumes that the move is from the perspective of the current player.
pub const fn get_policy_index_for_move(src_square: Square, dst_square: Square, vetted_promotion: Option<PieceType>) -> u8 {
    if is_knight_jump(src_square, dst_square) {
        // Knight move
        get_policy_index_for_knight_move(KnightMoveDirection::calc(src_square, dst_square))
    } else {
        // Queen-like move
        let (direction, distance) = QueenLikeMoveDirection::calc_and_measure_distance(src_square, dst_square);
        get_policy_index_for_queen_like_move(direction, distance, vetted_promotion)
    }
}

/// Fills the tensor channels for a given color's pieces.
/// `offset` determines the starting channel for this color's pieces in the tensor.
fn fill_pieces_for_color(tensor: &mut Tensor, state: &State, color: Color, offset: i64) {
    for piece_type in PieceType::iter_pieces() {
        let mask = state.board.color_masks[color as usize] & state.board.piece_type_masks[piece_type as usize];
        for square in get_squares_from_mask_iter(mask) {
            let square_from_perspective = square.to_perspective_from_white(state.side_to_move);
            let channel_index = offset + piece_type as i64 - PieceType::Pawn as i64;
            let _ = tensor
                .get(channel_index)
                .get(square_from_perspective.get_rank() as i64)
                .get(square_from_perspective.get_file() as i64)
                .fill_(1.);
        }
    }
}

fn fill_pieces(tensor: &mut Tensor, state: &State) {
    // Channels 0-5: Player's pieces
    fill_pieces_for_color(tensor, state, state.side_to_move, 0);

    // Channels 6-11: Opponent's pieces
    fill_pieces_for_color(tensor, state, state.side_to_move.flip(), NUM_PIECE_TYPE_BITS as i64);
}

fn fill_side_to_move(tensor: &mut Tensor, side_to_move: Color) {
    let _ = tensor.get(NUM_BITS_PER_BOARD as i64).fill_(
        if side_to_move == Color::White { 1. } else { 0. }
    );
}

fn fill_castling_rights(tensor: &mut Tensor, castling_rights: u8) {
    for (i, bit) in [0b1000, 0b0100, 0b0010, 0b0001].iter().enumerate() {
        let _ = tensor.get((NUM_BITS_PER_BOARD + NUM_SIDE_TO_MOVE_BITS + i as u8) as i64).fill_(
            if castling_rights & bit != 0 { 1. } else { 0. }
        );
    }
}

pub fn state_to_tensor(state: &State) -> Tensor {
    // Initialize a tensor with shape [17, 8, 8], where:
    // - 17 is the number of channels
    // - 8x8 is the board size
    let mut tensor = Tensor::zeros(&[NUM_POSITION_BITS as i64, 8, 8], (Kind::Float, *DEVICE));
    
    // Channels 0-11: Pieces
    fill_pieces(&mut tensor, state);

    // Channel 12: Side to move (1 if white to move, 0 if black to move)
    fill_side_to_move(&mut tensor, state.side_to_move);

    // Channel 13-16: Castling rights
    fill_castling_rights(&mut tensor, state.context.borrow().castling_rights);

    tensor
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::attacks::single_knight_attacks;
    use crate::engine::conv_net_evaluator::constants::{MAX_NUM_KNIGHT_MOVES, NUM_PAWN_MOVE_DIRECTIONS};
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
    fn test_get_policy_index_for_promotions() {
        let mut used_underpromotion_indices = [false; NUM_WAYS_OF_UNDERPROMOTION as usize];
        let mut used_queen_promotion_indices = HashSet::new();
        for direction in [QueenLikeMoveDirection::UpLeft, QueenLikeMoveDirection::Up, QueenLikeMoveDirection::UpRight].iter() {
            for promotion in [PieceType::Knight, PieceType::Bishop, PieceType::Rook].iter() {
                let index = get_policy_index_for_queen_like_move(*direction, 1, Some(*promotion));
                assert!(index >= NUM_QUEEN_LIKE_MOVES);
                assert!(index < NUM_TARGET_SQUARE_POSSIBILITIES);
                let modified_index = index - NUM_QUEEN_LIKE_MOVES;
                assert!(!used_underpromotion_indices[modified_index as usize]);
                used_underpromotion_indices[modified_index as usize] = true;
            }
            let index = get_policy_index_for_queen_like_move(*direction, 1, Some(PieceType::Queen));
            assert!(index < NUM_QUEEN_LIKE_MOVES);
            assert!(!used_queen_promotion_indices.contains(&index));
            used_queen_promotion_indices.insert(index);
        }
        assert!(used_underpromotion_indices.iter().all(|&used| used));
        assert_eq!(used_queen_promotion_indices.len(), NUM_PAWN_MOVE_DIRECTIONS as usize);
    }

    #[test]
    fn test_get_policy_index_for_knight_move() {
        let mut used_indices = [false; MAX_NUM_KNIGHT_MOVES as usize];
        for direction in KnightMoveDirection::iter() {
            let index = get_policy_index_for_knight_move(direction);
            assert!(index >= NUM_QUEEN_LIKE_MOVES + NUM_WAYS_OF_UNDERPROMOTION);
            assert!(index < NUM_TARGET_SQUARE_POSSIBILITIES);
            let modified_index = index - NUM_QUEEN_LIKE_MOVES - NUM_WAYS_OF_UNDERPROMOTION;
            assert!(!used_indices[modified_index as usize]);
            used_indices[modified_index as usize] = true;
        }
        assert!(used_indices.iter().all(|&used| used));
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