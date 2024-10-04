use crate::bitboard::Bitboard;
use crate::miscellaneous::Square;
use lazy_static::lazy_static;
use crate::attacks::manual_attacks;

lazy_static! {
    static ref SINGLE_KING_ATTACKS: [Bitboard; 64] = {
        let mut attacks = [0; 64];
        for square in Square::iter_all() {
            let king_mask = square.to_mask();
            attacks[square as usize] = manual_attacks::multi_king_attacks(king_mask);
        }
        attacks
    };
    
    static ref SINGLE_KNIGHT_ATTACKS: [Bitboard; 64] = {
        let mut attacks = [0; 64];
        for square in Square::iter_all() {
            let knight_mask = square.to_mask();
            attacks[square as usize] = manual_attacks::multi_knight_attacks(knight_mask);
        }
        attacks
    };
}

pub fn precomputed_single_king_attacks(src_mask: Bitboard) -> Bitboard {
    SINGLE_KING_ATTACKS[src_mask.leading_zeros() as usize]
}

pub fn precomputed_single_knight_attacks(src_mask: Bitboard) -> Bitboard {
    SINGLE_KNIGHT_ATTACKS[src_mask.leading_zeros() as usize]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::miscellaneous::Square;

    #[test]
    fn test_single_king_attacks() {
        for square in Square::iter_all() {
            assert_eq!(precomputed_single_king_attacks(square.to_mask()), manual_attacks::multi_king_attacks(square.to_mask()));
        }
    }

    #[test]
    fn test_single_knight_attacks() {
        for square in Square::iter_all() {
            assert_eq!(precomputed_single_knight_attacks(square.to_mask()), manual_attacks::multi_knight_attacks(square.to_mask()));
        }
    }
}