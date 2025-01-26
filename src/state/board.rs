//! Board struct and methods

use crate::utils::*;
use crate::attacks::*;
use crate::utils::{Bitboard, get_set_bit_mask_iter};
use crate::utils::masks::*;
use crate::state::zobrist::get_piece_zobrist_hash;

/// A struct representing the positions of all pieces on the board, for both colors,
/// as well as the zobrist hash of the position.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Board {
    pub piece_type_masks: [Bitboard; PieceType::LIMIT as usize],
    pub color_masks: [Bitboard; 2],
    pub zobrist_hash: Bitboard
}

impl Board {
    /// The board for the initial position.
    pub fn initial() -> Board {
        let mut res = Board {
            piece_type_masks: [
                STARTING_ALL,
                STARTING_WP | STARTING_BP,
                STARTING_WN | STARTING_BN,
                STARTING_WB | STARTING_BB,
                STARTING_WR | STARTING_BR,
                STARTING_WQ | STARTING_BQ,
                STARTING_WK | STARTING_BK
            ],
            color_masks: [
                STARTING_WHITE,
                STARTING_BLACK
            ],
            zobrist_hash: 0
        };
        res.zobrist_hash = res.calc_zobrist_hash();
        res
    }

    /// The board for a blank position with no pieces on it.
    pub fn blank() -> Board {
        Board {
            piece_type_masks: [0; PieceType::LIMIT as usize],
            color_masks: [0; 2],
            zobrist_hash: 0
        }
    }
    
    pub fn count_piece_type(&self, piece_type: PieceType) -> u32 {
        self.piece_type_masks[piece_type as usize].count_ones()
    }
    
    pub fn count_colored_piece(&self, colored_piece: ColoredPiece) -> u32 {
        (self.piece_type_masks[colored_piece.get_piece_type() as usize] & 
            self.color_masks[colored_piece.get_color() as usize]).count_ones()
    }
    
    pub fn count_color(&self, color: Color) -> u32 {
        self.color_masks[color as usize].count_ones()
    }
    
    pub fn count_all(&self) -> u32 {
        self.piece_type_masks[PieceType::AllPieceTypes as usize].count_ones()
    }
    
    /// Returns true if there is insufficient material on both sides to checkmate.
    /// This is the case if both sides have any one of the following, and there are no pawns on the board:
    /// A lone king
    /// A king and bishop
    /// A king and knight
    /// A king and two knights, only if the other side is a lone king
    pub fn are_both_sides_insufficient_material(&self, use_uscf_rules: bool) -> bool {
        if self.piece_type_masks[PieceType::Pawn as usize] | self.piece_type_masks[PieceType::Rook as usize] | self.piece_type_masks[PieceType::Queen as usize] != 0 {
            return false;
        }
        
        for color_int in Color::White as u8.. Color::Black as u8 + 1 {
            let bishops = self.piece_type_masks[PieceType::Bishop as usize] & self.color_masks[color_int as usize];
            let num_bishops = bishops.count_ones();
            if num_bishops > 1 {
                return false;
            }
            
            let knights = self.piece_type_masks[PieceType::Knight as usize] & self.color_masks[color_int as usize];
            let num_knights = knights.count_ones();
            
            if use_uscf_rules && num_knights == 2 && num_bishops == 0 { // king and two knights
                let opposite_side_bb = self.color_masks[Color::from(color_int != 0).flip() as usize];
                let all_occupancy = self.piece_type_masks[PieceType::AllPieceTypes as usize];
                let opposite_side_is_lone_king = (opposite_side_bb & all_occupancy).count_ones() == 1;
                return opposite_side_is_lone_king;
            }
            if num_knights + num_bishops > 1 {
                return false;
            }
        }
        
        true
    }
    
    /// Returns true if `mask` is attacked by any piece of the given color.
    /// Else, returns false.
    pub fn is_mask_in_check(&self, mask: Bitboard, by_color: Color) -> bool {
        let attacking_color_mask = self.color_masks[by_color as usize];
        let occupied_mask = self.piece_type_masks[PieceType::AllPieceTypes as usize];
        
        let pawns_mask = self.piece_type_masks[PieceType::Pawn as usize];
        let knights_mask = self.piece_type_masks[PieceType::Knight as usize];
        let bishops_mask = self.piece_type_masks[PieceType::Bishop as usize];
        let rooks_mask = self.piece_type_masks[PieceType::Rook as usize];
        let queens_mask = self.piece_type_masks[PieceType::Queen as usize];
        let kings_mask = self.piece_type_masks[PieceType::King as usize];

        let mut attacks = multi_pawn_attacks(pawns_mask & attacking_color_mask, by_color);
        
        attacks |= multi_knight_attacks(knights_mask & attacking_color_mask);
        
        for src_square in get_squares_from_mask_iter((bishops_mask | queens_mask) & attacking_color_mask) {
            attacks |= single_bishop_attacks(src_square, occupied_mask);
        }
        
        for src_square in get_squares_from_mask_iter((rooks_mask | queens_mask) & attacking_color_mask) {
            attacks |= single_rook_attacks(src_square, occupied_mask);
        }
        
        attacks |= multi_king_attacks(kings_mask & attacking_color_mask);
        
        attacks & mask != 0
    }

    /// Returns true if the given color's king is in check.
    pub fn is_color_in_check(&self, color: Color) -> bool { // including by king
        self.is_mask_in_check(
            self.piece_type_masks[PieceType::King as usize] & self.color_masks[color as usize],
            color.flip()
        )
    }
    
    /// Populates a square with `color`, but no piece type.
    /// Does not update the zobrist hash.
    pub fn put_color_at(&mut self, color: Color, square: Square) {
        let mask = square.get_mask();
        self.color_masks[color as usize] |= mask;
    }
    
    /// Populates a square with `piece_type`, but no color.
    /// Updates the zobrist hash.
    pub fn put_piece_type_at(&mut self, piece_type: PieceType, square: Square) {
        let mask = square.get_mask();
        self.piece_type_masks[piece_type as usize] |= mask;
        self.piece_type_masks[PieceType::AllPieceTypes as usize] |= mask;
        self.xor_piece_zobrist_hash(square, piece_type);
    }

    /// Populates a square with `colored_piece`.
    /// Updates the zobrist hash.
    pub fn put_colored_piece_at(&mut self, colored_piece: ColoredPiece, square: Square) {
        let piece_type = colored_piece.get_piece_type();
        let color = colored_piece.get_color();

        self.put_color_at(color, square);
        self.put_piece_type_at(piece_type, square);
    }
    
    /// Removes `color` from a square, but not piece type.
    /// Does not update the zobrist hash.
    pub fn remove_color_at(&mut self, color: Color, square: Square) {
        let mask = square.get_mask();
        self.color_masks[color as usize] &= !mask;
    }
    
    /// Removes `piece_type` from a square, but not color.
    /// Updates the zobrist hash.
    pub fn remove_piece_type_at(&mut self, piece_type: PieceType, square: Square) {
        let mask = square.get_mask();
        self.piece_type_masks[piece_type as usize] &= !mask;
        self.piece_type_masks[PieceType::AllPieceTypes as usize] &= !mask;
        self.xor_piece_zobrist_hash(square, piece_type);
    }

    /// Removes `colored_piece` from a square.
    /// Updates the zobrist hash.
    pub fn remove_colored_piece_at(&mut self, colored_piece: ColoredPiece, square: Square) {
        let piece_type = colored_piece.get_piece_type();
        let color = colored_piece.get_color();

        self.remove_color_at(color, square);
        self.remove_piece_type_at(piece_type, square);
    }
    
    /// Moves `piece_type` from `src_square` to `dst_square`.
    /// Does not update color.
    /// Updates the zobrist hash.
    pub fn move_piece_type(&mut self, piece_type: PieceType, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.get_mask();
        let src_mask = src_square.get_mask();
        let src_dst_mask = src_mask | dst_mask;
        
        self.piece_type_masks[piece_type as usize] ^= src_dst_mask;
        self.piece_type_masks[PieceType::AllPieceTypes as usize] ^= src_dst_mask;
        
        self.xor_piece_zobrist_hash(dst_square, piece_type);
        self.xor_piece_zobrist_hash(src_square, piece_type);
    }
    
    /// Moves `color` from `src_square` to `dst_square`.
    /// Does not update color.
    /// Does not update the zobrist hash.
    pub fn move_color(&mut self, color: Color, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.get_mask();
        let src_mask = src_square.get_mask();
        let src_dst_mask = src_mask | dst_mask;
        
        self.color_masks[color as usize] ^= src_dst_mask;
    }
    
    /// Moves a `colored_piece` from `src_square` to `dst_square`.
    /// Updates the zobrist hash.
    pub fn move_colored_piece(&mut self, colored_piece: ColoredPiece, dst_square: Square, src_square: Square) {
        let piece_type = colored_piece.get_piece_type();
        let color = colored_piece.get_color();
        
        self.move_color(color, dst_square, src_square);
        self.move_piece_type(piece_type, dst_square, src_square);
    }
    
    /// Returns the piece type at `square`.
    pub fn get_piece_type_at(&self, square: Square) -> PieceType {
        let mask = square.get_mask();
        for piece_type in PieceType::iter_pieces() {
            if self.piece_type_masks[*piece_type as usize] & mask != 0 {
                return *piece_type;
            }
        }
        PieceType::NoPieceType
    }
    
    /// Returns the color at `square`.
    pub fn get_color_at(&self, square: Square) -> Color {
        let mask = square.get_mask();
        Color::from(self.color_masks[Color::Black as usize] & mask != 0)
    }
    
    /// Returns the colored piece at `square`.
    pub fn get_colored_piece_at(&self, square: Square) -> ColoredPiece {
        let piece_type = self.get_piece_type_at(square);
        let color = self.get_color_at(square);
        ColoredPiece::from(color, piece_type)
    }
    
    /// Checks if the board is consistent (color masks, individual piece type masks, all occupancy).
    pub fn is_consistent(&self) -> bool {
        let white_bb = self.color_masks[Color::White as usize];
        let black_bb = self.color_masks[Color::Black as usize];
        if white_bb & black_bb != 0 {
            return false;
        }

        let all_occupancy_bb = self.piece_type_masks[PieceType::AllPieceTypes as usize];

        if (white_bb | black_bb) != all_occupancy_bb {
            return false;
        }

        let mut all_occupancy_bb_reconstructed: Bitboard = 0;

        for piece_type in PieceType::iter_pieces() {
            let piece_bb = self.piece_type_masks[*piece_type as usize];

            if piece_bb & all_occupancy_bb != piece_bb {
                return false;
            }

            if (piece_bb & white_bb) | (piece_bb & black_bb) != piece_bb {
                return false;
            }

            if piece_bb & all_occupancy_bb_reconstructed != 0 {
                return false;
            }
            all_occupancy_bb_reconstructed |= piece_bb;
        }

        all_occupancy_bb_reconstructed == all_occupancy_bb
    }
    
    /// Checks if the board has one king of each color.
    pub const fn has_valid_kings(&self) -> bool {
        let white_bb = self.color_masks[Color::White as usize];
        let kings_bb = self.piece_type_masks[PieceType::King as usize];

        kings_bb.count_ones() == 2 && (white_bb & kings_bb).count_ones() == 1
    }
    
    /// Checks if the zobrist hash is correctly calculated.
    pub fn is_zobrist_valid(&self) -> bool {
        self.zobrist_hash == self.calc_zobrist_hash()
    }
    
    /// Rigorous check for the validity and consistency of the board.
    pub fn is_unequivocally_valid(&self) -> bool {
        self.has_valid_kings() && self.is_consistent() && self.is_zobrist_valid()
    }

    /// Prints the board to the console.
    pub fn print(&self) {
        println!("{}", self);
    }
}
