use lazy_static::lazy_static;
use rand::Rng;
use crate::bitboard::get_squares_from_bb;
use crate::board::Board;
use crate::miscellaneous::{Color, PieceType};
use crate::state::State;

lazy_static! {
    pub static ref ZOBRIST_TABLE: [[u64; 12]; 64] = generate_zobrist_table();
}

pub fn generate_zobrist_table() -> [[u64; 12]; 64] {
    let mut rng = rand::thread_rng();
    let mut zobrist: [[u64; 12]; 64] = [[0; 12]; 64];
    for i in 0..64 {
        for j in 0..12 {
            zobrist[i][j] = rng.gen();
        }
    }
    zobrist
}

impl Board {
    pub fn zobrist_hash(&self) -> u64 {
        let mut hash: u64 = 0;
        for piece_type in PieceType::iter_between(PieceType::Pawn, PieceType::King) { // skip PieceType::NoPieceType, PieceType::King
            let piece_bb = self.bb_by_piece_type[piece_type as usize];
            for color in Color::iter() {
                let color_bb = self.bb_by_color[color as usize];
                let combined_bb = piece_bb & color_bb;
                for index in get_squares_from_bb(combined_bb) {
                    hash ^= ZOBRIST_TABLE[index as usize][piece_type as usize - 1];
                }
            }
        }
        let kings_bb = self.bb_by_piece_type[PieceType::King as usize];
        for color_int in Color::White as u8..Color::Black as u8 + 1 {
            let colored_king_bb = kings_bb & self.bb_by_color[color_int as usize];
            if colored_king_bb != 0 {
                hash ^= ZOBRIST_TABLE[colored_king_bb.leading_zeros() as usize][PieceType::King as usize - 1];
            }
        }
        hash
    }
}

impl State {
    pub fn increment_position_count(&mut self) -> u8 {
        let position_count = self.position_counts.entry(self.board.zobrist_hash()).or_insert(0);
        *position_count += 1;
        position_count.clone()
    }
    
    pub fn decrement_position_count(&mut self) {
        let hash = self.board.zobrist_hash();
        match self.position_counts.get_mut(&hash) {
            Some(position_count) => {
                if *position_count == 0 { 
                    self.position_counts.remove(&hash);
                }
            },
            None => {}
        }
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