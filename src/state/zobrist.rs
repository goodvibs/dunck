use lazy_static::lazy_static;
use rand::Rng;
use crate::bitboard::{get_squares_from_mask, Bitboard};
use crate::miscellaneous::{Color, PieceType, Square};
use crate::state::board::Board;
use crate::state::State;

lazy_static! {
    static ref ZOBRIST_TABLE: [[Bitboard; 12]; 64] = generate_zobrist_table();
}

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

pub fn get_piece_zobrist_hash(square: Square, piece_type: PieceType) -> Bitboard {
    ZOBRIST_TABLE[square as usize][piece_type as usize - 1]
}

impl Board {
    pub fn calc_zobrist_hash(&self) -> Bitboard {
        let mut hash: Bitboard = 0;
        for piece_type in PieceType::iter_between(PieceType::Pawn, PieceType::King) { // skip PieceType::NoPieceType, PieceType::King
            let piece_bb = self.bb_by_piece_type[piece_type as usize];
            for color in Color::iter() {
                let color_bb = self.bb_by_color[color as usize];
                let combined_bb = piece_bb & color_bb;
                for square in get_squares_from_mask(combined_bb) {
                    hash ^= get_piece_zobrist_hash(square, piece_type);
                }
            }
        }
        let kings_bb = self.bb_by_piece_type[PieceType::King as usize];
        for color in Color::iter() {
            let single_king_bb = kings_bb & self.bb_by_color[color as usize];
            if single_king_bb != 0 {
                let colored_king_square = unsafe { Square::from(single_king_bb.leading_zeros() as u8) };
                hash ^= get_piece_zobrist_hash(colored_king_square, PieceType::King);
            }
        }
        hash
    }
}

impl State {
    pub fn increment_position_count(&mut self) -> u8 {
        let position_count = self.position_counts.entry(self.board.zobrist_hash).or_insert(0);
        *position_count += 1;
        position_count.clone()
    }
    
    // pub fn decrement_position_count(&mut self) {
    //     let hash = self.board.zobrist_hash();
    //     match self.position_counts.get_mut(&hash) {
    //         Some(position_count) => {
    //             if *position_count == 0 {
    //                 self.position_counts.remove(&hash);
    //             }
    //         },
    //         None => {}
    //     }
    // }
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