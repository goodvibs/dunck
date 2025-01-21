use lazy_static::lazy_static;
use tch::{Device, Kind, Tensor};
use crate::engine::evaluators::neural::constants::{MAX_RAY_LENGTH, NUM_BITS_PER_BOARD, NUM_PIECE_TYPE_BITS, NUM_POSITION_BITS, NUM_QUEEN_LIKE_MOVES, NUM_SIDE_TO_MOVE_BITS, NUM_UNDERPROMOTIONS, NUM_WAYS_OF_UNDERPROMOTION};
use crate::r#move::{Move, MoveFlag};
use crate::state::State;
use crate::utils::{get_squares_from_mask_iter, Color, KnightMoveDirection, PieceType, QueenLikeMoveDirection, Square};

lazy_static! {
    pub static ref DEVICE: Device = Device::cuda_if_available();
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PolicyIndex {
    pub source_rank_index: u8,
    pub source_file_index: u8,
    pub move_index: u8
}

impl PolicyIndex {
    pub fn calc(mv: &Move, color: Color) -> Self {
        let src_square = mv.get_source();
        let dst_square = mv.get_destination();
        let vetted_promotion = match mv.get_flag() {
            MoveFlag::Promotion => Some(mv.get_promotion()),
            _ => None
        };
        
        let src_square_from_current_perspective = src_square.to_perspective_from_white(color);
        let dst_square_from_current_perspective = dst_square.to_perspective_from_white(color);
        
        let move_index = calc_move_index(
            src_square_from_current_perspective,
            dst_square_from_current_perspective,
            vetted_promotion
        );
        
        PolicyIndex {
            source_rank_index: src_square_from_current_perspective.get_rank(),
            source_file_index: src_square_from_current_perspective.get_file(),
            move_index
        }
    }
}

/// Checks if a move is a knight move based on its source and destination squares.
const fn is_knight_jump(src_square: Square, dst_square: Square) -> bool {
    // Calculate the difference in rank and file between the source and destination
    let rank_diff = (dst_square.get_rank() as i8 - src_square.get_rank() as i8).abs();
    let file_diff = (dst_square.get_file() as i8 - src_square.get_file() as i8).abs();

    // A knight move is either (±2, ±1) or (±1, ±2)
    (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
}

/// Maps a queen-like move to an index in the policy tensor's 73 possible moves per square.
/// Index is between 0 and 64 for queen-like moves (56 different target squares, 9 possible underpromotions).
/// Assumes that the direction is from the perspective of the current player.
const fn calc_move_index_for_queen_like_move(direction: QueenLikeMoveDirection, distance: u8, promotion: Option<PieceType>) -> u8 {
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
const fn calc_move_index_for_knight_move(direction: KnightMoveDirection) -> u8 {
    direction as u8 + NUM_QUEEN_LIKE_MOVES + NUM_WAYS_OF_UNDERPROMOTION
}

/// Maps a move to an index in the policy tensor's 73 possible moves per square.
/// Assumes that the move is from the perspective of the current player.
const fn calc_move_index(src_square_from_current_perspective: Square,
                             dst_square_from_current_perspective: Square,
                             vetted_promotion: Option<PieceType>) -> u8 {
    if is_knight_jump(src_square_from_current_perspective, dst_square_from_current_perspective) {
        // Knight move
        calc_move_index_for_knight_move(KnightMoveDirection::calc(src_square_from_current_perspective, dst_square_from_current_perspective))
    } else {
        // Queen-like move
        let (direction, distance) = QueenLikeMoveDirection::calc_and_measure_distance(src_square_from_current_perspective, dst_square_from_current_perspective);
        calc_move_index_for_queen_like_move(direction, distance, vetted_promotion)
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
    let val = if side_to_move == Color::White { 1. } else { 0. };
    let _ = tensor.get(NUM_BITS_PER_BOARD as i64).fill_(
        val
    );
}

fn fill_castling_rights(tensor: &mut Tensor, castling_rights: u8) { // todo: account for perspective
    for (i, bit) in [0b1000, 0b0100, 0b0010, 0b0001].iter().enumerate() {
        let val = if castling_rights & bit != 0 { 1. } else { 0. };
        let _ = tensor.get((NUM_BITS_PER_BOARD + NUM_SIDE_TO_MOVE_BITS + i as u8) as i64).fill_(
            val
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
    use crate::attacks::{single_bishop_attacks, single_knight_attacks, single_rook_attacks};
    use crate::engine::evaluators::neural::constants::{MAX_NUM_KNIGHT_MOVES, NUM_PAWN_MOVE_DIRECTIONS, NUM_TARGET_SQUARE_POSSIBILITIES};
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
    fn test_calc_move_index_for_sliding_pieces() {
        let mut used_indices = [false; NUM_QUEEN_LIKE_MOVES as usize];
        for direction in QueenLikeMoveDirection::iter() {
            for distance in 1..=MAX_RAY_LENGTH {
                let index = calc_move_index_for_queen_like_move(direction, distance, None);
                assert!(index < NUM_QUEEN_LIKE_MOVES);
                assert!(!used_indices[index as usize]);
                used_indices[index as usize] = true;
            }
        }
        assert!(used_indices.iter().all(|&used| used));
    }

    #[test]
    fn test_calc_move_index_for_promotions() {
        let mut used_underpromotion_indices = [false; NUM_WAYS_OF_UNDERPROMOTION as usize];
        let mut used_queen_promotion_indices = HashSet::new();
        for direction in [QueenLikeMoveDirection::UpLeft, QueenLikeMoveDirection::Up, QueenLikeMoveDirection::UpRight].iter() {
            for promotion in [PieceType::Knight, PieceType::Bishop, PieceType::Rook].iter() {
                let index = calc_move_index_for_queen_like_move(*direction, 1, Some(*promotion));
                assert!(index >= NUM_QUEEN_LIKE_MOVES);
                assert!(index < NUM_TARGET_SQUARE_POSSIBILITIES);
                let modified_index = index - NUM_QUEEN_LIKE_MOVES;
                assert!(!used_underpromotion_indices[modified_index as usize]);
                used_underpromotion_indices[modified_index as usize] = true;
            }
            let index = calc_move_index_for_queen_like_move(*direction, 1, Some(PieceType::Queen));
            assert!(index < NUM_QUEEN_LIKE_MOVES);
            assert!(!used_queen_promotion_indices.contains(&index));
            used_queen_promotion_indices.insert(index);
        }
        assert!(used_underpromotion_indices.iter().all(|&used| used));
        assert_eq!(used_queen_promotion_indices.len(), NUM_PAWN_MOVE_DIRECTIONS as usize);
    }

    #[test]
    fn test_calc_move_index_for_knight_move() {
        let mut used_indices = [false; MAX_NUM_KNIGHT_MOVES as usize];
        for direction in KnightMoveDirection::iter() {
            let index = calc_move_index_for_knight_move(direction);
            assert!(index >= NUM_QUEEN_LIKE_MOVES + NUM_WAYS_OF_UNDERPROMOTION);
            assert!(index < NUM_TARGET_SQUARE_POSSIBILITIES);
            let modified_index = index - NUM_QUEEN_LIKE_MOVES - NUM_WAYS_OF_UNDERPROMOTION;
            assert!(!used_indices[modified_index as usize]);
            used_indices[modified_index as usize] = true;
        }
        assert!(used_indices.iter().all(|&used| used));
    }

    #[test]
    fn test_calc_move_index_for_knight_moves() {
        for square_a in Square::iter_all() {
            for square_b in get_squares_from_mask_iter(single_knight_attacks(square_a)) {
                let index1 = calc_move_index(square_a, square_b, None);
                let index2 = calc_move_index(square_b.to_perspective_from_white(Color::Black), square_a.to_perspective_from_white(Color::Black), None);
                assert_eq!(index1, index2);
                assert!(index1 >= NUM_QUEEN_LIKE_MOVES);
                assert!(index1 < NUM_TARGET_SQUARE_POSSIBILITIES);
            }
        }
    }

    #[test]
    fn test_calc_move_index_for_queen_like_moves() {
        for square_a in Square::iter_all() {
            for square_b in get_squares_from_mask_iter(single_bishop_attacks(square_a, 0) | single_rook_attacks(square_a, 0)) {
                let index1 = calc_move_index(square_a, square_b, None);
                let index2 = calc_move_index(square_b.to_perspective_from_white(Color::Black), square_a.to_perspective_from_white(Color::Black), None);
                assert_eq!(index1, index2);
                assert!(index1 < NUM_QUEEN_LIKE_MOVES);
            }
        }
    }

    #[test]
    fn test_state_to_tensor() {
        let state = State::initial();
        let tensor = state_to_tensor(&state);
        
        // check tensor shape
        assert_eq!(tensor.size(), vec![17, 8, 8]);
        
        // channel 0: player pawns
        assert_eq!(tensor.get(0).sum(Kind::Float).double_value(&[]), 8.);
        
        // channel 1: player knights
        assert_eq!(tensor.get(1).sum(Kind::Float).double_value(&[]), 2.);
        
        // channel 2: player bishops
        assert_eq!(tensor.get(2).sum(Kind::Float).double_value(&[]), 2.);
        
        // channel 3: player rooks
        assert_eq!(tensor.get(3).sum(Kind::Float).double_value(&[]), 2.);
        
        // channel 4: player queens
        assert_eq!(tensor.get(4).sum(Kind::Float).double_value(&[]), 1.);
        
        // channel 5: player kings
        assert_eq!(tensor.get(5).sum(Kind::Float).double_value(&[]), 1.);
        
        // channel 6: opponent pawns
        assert_eq!(tensor.get(6).sum(Kind::Float).double_value(&[]), 8.);
        
        // channel 7: opponent knights
        assert_eq!(tensor.get(7).sum(Kind::Float).double_value(&[]), 2.);
        
        // channel 8: opponent bishops
        assert_eq!(tensor.get(8).sum(Kind::Float).double_value(&[]), 2.);
        
        // channel 9: opponent rooks
        assert_eq!(tensor.get(9).sum(Kind::Float).double_value(&[]), 2.);
        
        // channel 10: opponent queens
        assert_eq!(tensor.get(10).sum(Kind::Float).double_value(&[]), 1.);
        
        // channel 11: opponent kings
        assert_eq!(tensor.get(11).sum(Kind::Float).double_value(&[]), 1.);
        
        // channel 12: side to move
        assert_eq!(tensor.get(12).sum(Kind::Float).double_value(&[]), 64.);
        
        // channel 13-16: castling rights
        assert_eq!(tensor.get(13).sum(Kind::Float).double_value(&[]), 64.);
        assert_eq!(tensor.get(14).sum(Kind::Float).double_value(&[]), 64.);
        assert_eq!(tensor.get(15).sum(Kind::Float).double_value(&[]), 64.);
        assert_eq!(tensor.get(16).sum(Kind::Float).double_value(&[]), 64.);
        
        let state = State::from_fen("1nbqkbnr/rp2pp1p/p1P5/8/1P5R/P7/2PP1PP1/RNBQKBN1 b Qk - 0 7").unwrap();
        let tensor = state_to_tensor(&state);

        // check tensor shape
        assert_eq!(tensor.size(), vec![17, 8, 8]);

        // channel 0: player pawns
        assert_eq!(tensor.get(0).sum(Kind::Float).double_value(&[]), 5.);

        // channel 1: player knights
        assert_eq!(tensor.get(1).sum(Kind::Float).double_value(&[]), 2.);

        // channel 2: player bishops
        assert_eq!(tensor.get(2).sum(Kind::Float).double_value(&[]), 2.);

        // channel 3: player rooks
        assert_eq!(tensor.get(3).sum(Kind::Float).double_value(&[]), 2.);

        // channel 4: player queens
        assert_eq!(tensor.get(4).sum(Kind::Float).double_value(&[]), 1.);

        // channel 5: player kings
        assert_eq!(tensor.get(5).sum(Kind::Float).double_value(&[]), 1.);

        // channel 6: opponent pawns
        assert_eq!(tensor.get(6).sum(Kind::Float).double_value(&[]), 7.);

        // channel 7: opponent knights
        assert_eq!(tensor.get(7).sum(Kind::Float).double_value(&[]), 2.);

        // channel 8: opponent bishops
        assert_eq!(tensor.get(8).sum(Kind::Float).double_value(&[]), 2.);

        // channel 9: opponent rooks
        assert_eq!(tensor.get(9).sum(Kind::Float).double_value(&[]), 2.);

        // channel 10: opponent queens
        assert_eq!(tensor.get(10).sum(Kind::Float).double_value(&[]), 1.);

        // channel 11: opponent kings
        assert_eq!(tensor.get(11).sum(Kind::Float).double_value(&[]), 1.);

        // channel 12: side to move
        assert_eq!(tensor.get(12).sum(Kind::Float).double_value(&[]), 0.);

        // channel 13-16: castling rights
        // todo: fix when perspective gets taken into account
        assert_eq!(tensor.get(13).sum(Kind::Float).double_value(&[]), 0.);
        assert_eq!(tensor.get(14).sum(Kind::Float).double_value(&[]), 64.);
        assert_eq!(tensor.get(15).sum(Kind::Float).double_value(&[]), 64.);
        assert_eq!(tensor.get(16).sum(Kind::Float).double_value(&[]), 0.);
    }
}