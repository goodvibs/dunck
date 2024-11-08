///! All Zobrist hashing-related code.

use lazy_static::lazy_static;
use rand::Rng;
use crate::utils::{get_squares_from_mask_iter, Bitboard};
use crate::utils::{Color, PieceType, Square};
use crate::state::board::Board;
use crate::state::State;

lazy_static! {
    /// A table of random bitboards for each piece type on each square.
    static ref ZOBRIST_TABLE: [[Bitboard; 12]; 64] = generate_zobrist_table();
}

/// Generates a table of random bitboards for each piece type on each square.
pub fn generate_zobrist_table() -> [[Bitboard; 12]; 64] {
    let mut rng = rand::thread_rng();
    let mut zobrist: [[Bitboard; 12]; 64] = [[0; 12]; 64];
    for i in 0..64 {
        for j in 0..12 {
            zobrist[i][j] = rng.gen();
        }
    }
    zobrist
}

/// Gets the Zobrist hash for a piece on a square.
pub fn get_piece_zobrist_hash(square: Square, piece_type: PieceType) -> Bitboard {
    ZOBRIST_TABLE[square as usize][piece_type as usize - 1]
}

impl Board {
    /// Calculates the Zobrist hash scratch.
    pub fn calc_zobrist_hash(&self) -> Bitboard {
        let mut hash: Bitboard = 0;
        for piece_type in PieceType::iter_pieces() { // skip PieceType::NoPieceType
            let pieces_mask = self.piece_type_masks[piece_type as usize];
            for square in get_squares_from_mask_iter(pieces_mask) {
                hash ^= get_piece_zobrist_hash(square, piece_type);
            }
        }
        hash
    }
    
    /// Applies the xor of the Zobrist hash of a piece on a square
    pub fn xor_piece_zobrist_hash(&mut self, square: Square, piece_type: PieceType) {
        self.zobrist_hash ^= get_piece_zobrist_hash(square, piece_type)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_zobrist_hash() {
        // todo
    }

    #[test]
    fn test_increment_position_count() {
        // todo
    }

    #[test]
    fn test_decrement_position_count() {
        // todo
    }
}