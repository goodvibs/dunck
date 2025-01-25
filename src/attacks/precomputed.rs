//! Precomputed attack tables for non-sliding pieces.

use crate::utils::Bitboard;
use crate::utils::Square;
use static_init::dynamic;
use crate::attacks::manual;

/// Precomputed attacks table for kings.
#[dynamic]
static SINGLE_KING_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [0; 64];
    for square in Square::iter_all() {
        let king_mask = square.get_mask();
        attacks[*square as usize] = manual::multi_king_attacks(king_mask);
    }
    attacks
};

/// Precomputed attacks table for knights.
#[dynamic]
static SINGLE_KNIGHT_ATTACKS: [Bitboard; 64] = {
    let mut attacks = [0; 64];
    for square in Square::iter_all() {
        let knight_mask = square.get_mask();
        attacks[*square as usize] = manual::multi_knight_attacks(knight_mask);
    }
    attacks
};

/// Returns a precomputed bitboard with all squares attacked by a knight on `src_square`
pub fn precomputed_single_king_attacks(src_square: Square) -> Bitboard {
    SINGLE_KING_ATTACKS[src_square as usize]
}

/// Returns a precomputed bitboard with all squares attacked by a knight on `src_square`
pub fn precomputed_single_knight_attacks(src_square: Square) -> Bitboard {
    SINGLE_KNIGHT_ATTACKS[src_square as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::Square;

    #[test]
    fn test_single_king_attacks() {
        for square in Square::iter_all() {
            assert_eq!(precomputed_single_king_attacks(*square), manual::multi_king_attacks(square.get_mask()));
        }
    }

    #[test]
    fn test_single_knight_attacks() {
        for square in Square::iter_all() {
            assert_eq!(precomputed_single_knight_attacks(*square), manual::multi_knight_attacks(square.get_mask()));
        }
    }
}