use crate::enums::*;
use crate::preload::ZOBRIST_TABLE;
use crate::charboard::*;
use crate::attacks::*;
use crate::bitboard::{bb_to_square_indices, Bitboard, unpack_bb};
use crate::masks::*;

#[derive(Eq, PartialEq, Clone)]
pub struct Board {
    pub bb_by_piece_type: [Bitboard; PieceType::LIMIT],
    pub bb_by_color: [Bitboard; 2],
    pub colored_piece_count: [u8; ColoredPiece::LIMIT],
}

impl Board {
    pub fn initial() -> Board {
        Board {
            bb_by_piece_type: [
                STARTING_ALL,
                STARTING_WP | STARTING_BP,
                STARTING_WN | STARTING_BN,
                STARTING_WB | STARTING_BB,
                STARTING_WR | STARTING_BR,
                STARTING_WQ | STARTING_BQ,
                STARTING_WK | STARTING_BK
            ],
            bb_by_color: [
                STARTING_WHITE,
                STARTING_BLACK
            ],
            colored_piece_count: [
                30,
                8,
                2,
                2,
                2,
                1,
                0,
                0,
                1,
                8,
                2,
                2,
                2,
                1,
                1
            ]
        }
    }

    pub fn blank() -> Board {
        Board {
            bb_by_piece_type: [0; PieceType::LIMIT],
            bb_by_color: [0; 2],
            colored_piece_count: [0; ColoredPiece::LIMIT]
        }
    }
    
    pub fn are_both_sides_insufficient_material(&self) -> bool {
        // If both sides have any one of the following, and there are no pawns on the board:
        // A lone king
        // A king and bishop
        // A king and knight
        // A king and two knights, only if the other side is a lone king
        
        if self.bb_by_piece_type[PieceType::Pawn as usize] | self.bb_by_piece_type[PieceType::Rook as usize] | self.bb_by_piece_type[PieceType::Queen as usize] != 0 {
            return false;
        }
        
        for color_int in Color::White as u8.. Color::Black as u8 + 1 {
            let bishops = self.bb_by_piece_type[PieceType::Bishop as usize] & self.bb_by_color[color_int as usize];
            let num_bishops = bishops.count_ones();
            if num_bishops > 1 {
                return false;
            }
            
            let knights = self.bb_by_piece_type[PieceType::Knight as usize] & self.bb_by_color[color_int as usize];
            let num_knights = knights.count_ones();
            
            if num_knights == 2 && num_bishops == 0 { // king and two knights
                let opposite_side_bb = self.bb_by_color[Color::from(color_int != 0).flip() as usize];
                let all_occupancy = self.bb_by_piece_type[PieceType::AllPieceTypes as usize];
                let opposite_side_is_lone_king = (opposite_side_bb & all_occupancy).count_ones() == 1;
                return opposite_side_is_lone_king;
            }
            if num_knights + num_bishops > 1 {
                return false;
            }
        }
        
        true
    }

    pub fn is_in_check(&self, color: Color) -> bool {
        let opposite_color_pieces = self.bb_by_color[color.flip() as usize];
        let all_occ = self.bb_by_color[Color::White as usize] | self.bb_by_color[Color::Black as usize];
        let mut attacks = pawn_attacks(self.bb_by_piece_type[PieceType::Pawn as usize] & opposite_color_pieces, color.flip());
        attacks |= knight_attacks(self.bb_by_piece_type[PieceType::Knight as usize] & opposite_color_pieces);
        for bb in unpack_bb(self.bb_by_piece_type[PieceType::Bishop as usize] & opposite_color_pieces) {
            attacks |= bishop_attacks(bb, all_occ);
        }
        for bb in unpack_bb(self.bb_by_piece_type[PieceType::Rook as usize] & opposite_color_pieces) {
            attacks |= rook_attacks(bb, all_occ);
        }
        for bb in unpack_bb(self.bb_by_piece_type[PieceType::Queen as usize] & opposite_color_pieces) {
            attacks |= bishop_attacks(bb, all_occ) | rook_attacks(bb, all_occ);
        }
        attacks |= king_attacks(self.bb_by_piece_type[PieceType::King as usize] & opposite_color_pieces);
        attacks & self.bb_by_piece_type[PieceType::King as usize] & self.bb_by_color[color as usize] != 0
    }
    
    pub fn piece_type_at(&self, square_mask: Bitboard) -> PieceType {
        for piece_type_int in PieceType::Pawn as usize..PieceType::LIMIT {
            if self.bb_by_piece_type[piece_type_int] & square_mask != 0 {
                return unsafe { PieceType::from(piece_type_int as u8) };
            }
        }
        PieceType::NoPieceType
    }
    
    pub fn get_colored_piece_bb(&self, colored_piece: ColoredPiece) -> Bitboard {
        self.bb_by_piece_type[colored_piece as usize & 0b0111] & self.bb_by_color[colored_piece.get_color() as usize]
    }

    pub fn zobrist_hash(&self) -> u64 {
        let mut hash: u64 = 0;
        for piece_type_int in PieceType::Pawn as u8..PieceType::King as u8 { // skip PieceType::NoPieceType, PieceType::King
            let piece_bb = self.bb_by_piece_type[piece_type_int as usize];
            for color_int in Color::White as u8..Color::Black as u8 + 1 {
                let color_bb = self.bb_by_color[color_int as usize];
                let combined_bb = piece_bb & color_bb;
                for index in bb_to_square_indices(combined_bb) {
                    hash ^= ZOBRIST_TABLE[index as usize][piece_type_int as usize - 1];
                }
            }
        }
        let kings_bb = self.bb_by_piece_type[PieceType::King as usize];
        for color_int in Color::White as u8..Color::Black as u8 + 1 {
            let colored_king_bb = kings_bb & self.bb_by_color[color_int as usize];
            hash ^= ZOBRIST_TABLE[colored_king_bb.leading_zeros() as usize][PieceType::King as usize - 1];
        }
        hash
    }

    pub fn from_cb(cb: Charboard) -> Board {
        let mut board = Board::blank();
        for i in 0..8 {
            for j in 0..8 {
                let mask = 1 << (63 - (i * 8 + j));
                let piece = ColoredPiece::from_char(cb[i][j]);
                if piece != ColoredPiece::NoPiece {
                    board.bb_by_piece_type[piece as usize] |= mask;
                    board.bb_by_color[piece.get_color() as usize] |= mask;
                    board.colored_piece_count[piece as usize] += 1;
                }
            }
        }
        board
    }

    pub fn to_cb(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for i in 0..64 {
            let mask = 1 << (63 - i);
            let piece_type = self.piece_type_at(mask);
            let color = if self.bb_by_color[Color::White as usize] & mask != 0 { Color::White } else { Color::Black };
            cb[i / 8][i % 8] = ColoredPiece::from(color, piece_type).to_char();
        }
        cb
    }

    pub fn print(&self) {
        println!("{}", self);
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", cb_to_string(&self.to_cb()).as_str())
    }
}