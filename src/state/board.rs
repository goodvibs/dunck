use crate::utils::*;
use crate::attacks::*;
use crate::utils::{Bitboard, unpack_mask};
use crate::utils::masks::*;
use crate::state::zobrist::get_piece_zobrist_hash;

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct Board {
    pub piece_type_masks: [Bitboard; PieceType::LIMIT as usize],
    pub color_masks: [Bitboard; 2],
    pub zobrist_hash: Bitboard
}

impl Board {
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

    pub fn blank() -> Board {
        Board {
            piece_type_masks: [0; PieceType::LIMIT as usize],
            color_masks: [0; 2],
            zobrist_hash: 0
        }
    }
    
    pub fn are_both_sides_insufficient_material(&self) -> bool {
        // If both sides have any one of the following, and there are no pawns on the board:
        // A lone king
        // A king and bishop
        // A king and knight
        // A king and two knights, only if the other side is a lone king
        
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
            
            if num_knights == 2 && num_bishops == 0 { // king and two knights
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
    
    pub fn is_mask_in_check(&self, mask: Bitboard, by_color: Color) -> bool {
        let attacking_color_pieces = self.color_masks[by_color as usize];
        let all_occ = self.piece_type_masks[PieceType::AllPieceTypes as usize];
        let queens_bb = self.piece_type_masks[PieceType::Queen as usize];

        let mut attacks = multi_pawn_attacks(self.piece_type_masks[PieceType::Pawn as usize] & attacking_color_pieces, by_color);
        attacks |= multi_knight_attacks(self.piece_type_masks[PieceType::Knight as usize] & attacking_color_pieces);
        for src_square in get_squares_from_mask((self.piece_type_masks[PieceType::Bishop as usize] | queens_bb) & attacking_color_pieces) {
            attacks |= single_bishop_attacks(src_square, all_occ);
        }
        for src_square in get_squares_from_mask((self.piece_type_masks[PieceType::Rook as usize] | queens_bb) & attacking_color_pieces) {
            attacks |= single_rook_attacks(src_square, all_occ);
        }
        attacks |= multi_king_attacks(self.piece_type_masks[PieceType::King as usize] & attacking_color_pieces);
        attacks & mask != 0
    }

    pub fn is_color_in_check(&self, color: Color) -> bool { // including by king
        self.is_mask_in_check(
            self.piece_type_masks[PieceType::King as usize] & self.color_masks[color as usize],
            color.flip()
        )
    }
    
    pub fn put_color_at(&mut self, color: Color, square: Square) {
        let mask = square.to_mask();
        self.color_masks[color as usize] |= mask;
    }
    
    pub fn put_piece_type_at(&mut self, piece_type: PieceType, square: Square) {
        let mask = square.to_mask();
        self.piece_type_masks[piece_type as usize] |= mask;
        self.piece_type_masks[PieceType::AllPieceTypes as usize] |= mask;
        self.xor_piece_zobrist_hash(square, piece_type);
    }

    pub fn put_colored_piece_at(&mut self, colored_piece: ColoredPiece, square: Square) {
        let piece_type = colored_piece.get_piece_type();
        let color = colored_piece.get_color();

        self.put_color_at(color, square);
        self.put_piece_type_at(piece_type, square);
    }
    
    pub fn remove_color_at(&mut self, color: Color, square: Square) {
        let mask = square.to_mask();
        self.color_masks[color as usize] &= !mask;
    }
    
    pub fn remove_piece_type_at(&mut self, piece_type: PieceType, square: Square) {
        let mask = square.to_mask();
        self.piece_type_masks[piece_type as usize] &= !mask;
        self.piece_type_masks[PieceType::AllPieceTypes as usize] &= !mask;
        self.xor_piece_zobrist_hash(square, piece_type);
    }

    pub fn remove_colored_piece_at(&mut self, colored_piece: ColoredPiece, square: Square) {
        let piece_type = colored_piece.get_piece_type();
        let color = colored_piece.get_color();

        self.remove_color_at(color, square);
        self.remove_piece_type_at(piece_type, square);
    }
    
    pub fn move_piece_type(&mut self, piece_type: PieceType, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.to_mask();
        let src_mask = src_square.to_mask();
        let src_dst_mask = src_mask | dst_mask;
        
        self.piece_type_masks[piece_type as usize] ^= src_dst_mask;
        self.piece_type_masks[PieceType::AllPieceTypes as usize] ^= src_dst_mask;
        
        self.xor_piece_zobrist_hash(dst_square, piece_type);
        self.xor_piece_zobrist_hash(src_square, piece_type);
    }
    
    pub fn move_color(&mut self, color: Color, dst_square: Square, src_square: Square) {
        let dst_mask = dst_square.to_mask();
        let src_mask = src_square.to_mask();
        let src_dst_mask = src_mask | dst_mask;
        
        self.color_masks[color as usize] ^= src_dst_mask;
    }
    
    pub fn move_colored_piece(&mut self, colored_piece: ColoredPiece, dst_square: Square, src_square: Square) {
        let piece_type = colored_piece.get_piece_type();
        let color = colored_piece.get_color();
        
        self.move_color(color, dst_square, src_square);
        self.move_piece_type(piece_type, dst_square, src_square);
    }
    
    pub fn get_piece_type_at(&self, square: Square) -> PieceType {
        let mask = square.to_mask();
        for piece_type in PieceType::iter_pieces() {
            if self.piece_type_masks[piece_type as usize] & mask != 0 {
                return piece_type;
            }
        }
        PieceType::NoPieceType
    }
    
    pub fn get_color_at(&self, square: Square) -> Color {
        let mask = square.to_mask();
        Color::from(self.color_masks[Color::Black as usize] & mask != 0)
    }
    
    pub fn get_colored_piece_at(&self, square: Square) -> ColoredPiece {
        let piece_type = self.get_piece_type_at(square);
        let color = self.get_color_at(square);
        ColoredPiece::from(color, piece_type)
    }
    
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
            let piece_bb = self.piece_type_masks[piece_type as usize];

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
    
    pub const fn has_valid_kings(&self) -> bool {
        let white_bb = self.color_masks[Color::White as usize];
        let kings_bb = self.piece_type_masks[PieceType::King as usize];

        kings_bb.count_ones() == 2 && (white_bb & kings_bb).count_ones() == 1
    }

    pub fn is_valid(&self) -> bool {
        self.is_consistent() && self.has_valid_kings()
    }

    pub fn print(&self) {
        println!("{}", self);
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::charboard::{EMPTY_CHARBOARD, INITIAL_CHARBOARD};
    // 
    // #[test]
    // fn test_put_colored_piece_at() {
    //     let mut board = Board::blank();
    //     let mask = Square::A1.to_mask();
    //     board.put_colored_piece_at(ColoredPiece::WhitePawn, mask);
    //     assert_eq!(board.get_piece_type_at(mask), PieceType::Pawn);
    //     assert_eq!(board.piece_type_masks[PieceType::Pawn as usize], mask);
    //     assert_eq!(board.color_masks[Color::White as usize], mask);
    //     assert_eq!(board.piece_type_masks[PieceType::AllPieceTypes as usize], mask);
    //     board.put_colored_piece_at(ColoredPiece::BlackPawn, mask);
    //     assert_eq!(board.get_piece_type_at(mask), PieceType::Pawn);
    //     assert_eq!(board.piece_type_masks[PieceType::Pawn as usize], mask);
    //     assert_eq!(board.color_masks[Color::Black as usize], mask);
    //     assert_eq!(board.piece_type_masks[PieceType::AllPieceTypes as usize], mask);
    // }
    // 
    // #[test]
    // fn test_clear_piece_at() {
    //     let mut board = Board::blank();
    //     let mask = Square::A1.to_mask() | Square::B1.to_mask();
    //     board.put_colored_piece_at(ColoredPiece::WhitePawn, mask);
    //     board.clear_piece_at(mask);
    //     assert_eq!(board.get_piece_type_at(mask), PieceType::NoPieceType);
    //     assert_eq!(board.piece_type_masks[PieceType::Pawn as usize], 0);
    //     assert_eq!(board.color_masks[Color::White as usize], 0);
    //     assert_eq!(board.piece_type_masks[PieceType::AllPieceTypes as usize], 0);
    // 
    //     let mut board = Board::initial();
    //     board.clear_piece_at(mask);
    //     assert_eq!(board.get_piece_type_at(mask), PieceType::NoPieceType);
    //     assert_eq!(board.piece_type_masks[PieceType::Pawn as usize], STARTING_WP | STARTING_BP);
    //     assert_eq!(board.color_masks[Color::White as usize], STARTING_WHITE & !mask);
    //     assert_eq!(board.color_masks[Color::Black as usize], STARTING_BLACK);
    //     assert_eq!(board.piece_type_masks[PieceType::Rook as usize], (STARTING_WR & !mask) | STARTING_BR);
    //     assert_eq!(board.piece_type_masks[PieceType::Knight as usize], (STARTING_WN & !mask) | STARTING_BN);
    //     assert_eq!(board.piece_type_masks[PieceType::Bishop as usize], STARTING_WB | STARTING_BB);
    //     assert_eq!(board.piece_type_masks[PieceType::Queen as usize], STARTING_WQ | STARTING_BQ);
    //     assert_eq!(board.piece_type_masks[PieceType::King as usize], STARTING_WK | STARTING_BK);
    //     assert_eq!(board.piece_type_masks[PieceType::AllPieceTypes as usize], STARTING_ALL & !mask);
    //     assert!(board.is_valid());
    // }
    // 
    // #[test]
    // fn test_board_is_valid() {
    //     let mut board = Board::blank();
    //     assert!(!board.is_valid());
    //     board.put_colored_piece_at(ColoredPiece::WhiteKing, Square::E1.to_mask());
    //     assert!(!board.is_valid());
    //     board.put_colored_piece_at(ColoredPiece::BlackKing, Square::F6.to_mask());
    //     assert!(board.is_valid());
    //     board.put_colored_piece_at(ColoredPiece::WhiteKing, Square::E8.to_mask());
    //     assert!(!board.is_valid());
    //     board.put_colored_piece_at(ColoredPiece::BlackKing, Square::E8.to_mask());
    //     assert!(!board.is_valid());
    // 
    //     let mut board = Board::initial();
    //     assert!(board.is_valid());
    // 
    //     board.put_colored_piece_at(ColoredPiece::BlackBishop, Square::C5.to_mask());
    //     assert!(board.is_valid());
    // 
    //     let mut board = Board::initial();
    //     board.put_colored_piece_at(ColoredPiece::WhitePawn, Square::A1.to_mask());
    //     assert!(!board.is_valid());
    // 
    //     let mut board = Board::initial();
    //     board.put_colored_piece_at(ColoredPiece::WhitePawn, Square::A1.to_mask());
    //     assert!(!board.is_valid());
    // 
    //     let mut board = Board::initial();
    //     board.clear_piece_at(Square::A1.to_mask());
    //     assert!(board.is_valid());
    //     board.clear_piece_at(Square::E1.to_mask());
    //     assert!(!board.is_valid());
    // }
    // 
    // #[test]
    // fn test_blank_board() {
    //     let mut board = Board::blank();
    //     assert!(!board.is_valid());
    //     board.put_colored_piece_at(ColoredPiece::WhiteKing, Square::E1.to_mask());
    //     assert!(!board.is_valid());
    //     board.put_colored_piece_at(ColoredPiece::BlackKing, Square::F6.to_mask());
    //     assert!(board.is_valid());
    //     
    //     let cb = Board::blank().to_cb();
    //     assert_eq!(cb, EMPTY_CHARBOARD);
    // }
    // 
    // #[test]
    // fn test_initial_board() {
    //     let board = Board::initial();
    //     assert!(board.is_valid());
    //     let cb = board.to_cb();
    //     assert_eq!(cb, INITIAL_CHARBOARD);
    // }
    // 
    // #[test]
    // fn test_get_piece_type_at() {
    //     let board = Board::initial();
    //     for i in 0..64 {
    //         let mask = 1 << (63 - i);
    //         let colored_piece_expected = ColoredPiece::from_char(INITIAL_CHARBOARD[i / 8][i % 8]);
    //         let piece_type_expected = colored_piece_expected.get_piece_type();
    //         assert_eq!(board.get_piece_type_at(mask), piece_type_expected);
    //     }
    // 
    //     let board = Board::blank();
    //     for i in 0..64 {
    //         let mask = 1 << (63 - i);
    //         assert_eq!(board.get_piece_type_at(mask), PieceType::NoPieceType);
    //     }
    // }
    // 
    // #[test]
    // fn test_get_colored_piece_bb() {
    //     let board = Board::initial();
    //     for piece_type in PieceType::iter_pieces() {
    //         let piece_type_int = piece_type as u8;
    //         for color in Color::iter() {
    //             let color_int = color as u8;
    //             let colored_piece = ColoredPiece::from(Color::from(color_int != 0), unsafe { PieceType::from(piece_type_int) });
    //             let colored_piece_bb = board.get_colored_piece_bb(colored_piece);
    //             let expected_bb = board.piece_type_masks[piece_type_int as usize] & board.color_masks[color_int as usize];
    //             assert_eq!(colored_piece_bb, expected_bb);
    //         }
    //     }
    // }
    // 
    // #[test]
    // pub fn test_are_both_sides_insufficient_material() {
    //     // todo
    // }
    // 
    // #[test]
    // pub fn test_is_in_check() {
    //     // todo
    // }
    // 
    // // #[test]
    // // fn test_from_cb() {
    // //     let board = Board::from_cb(INITIAL_CHARBOARD);
    // //     let cb = board.to_cb();
    // //     assert_eq!(cb, INITIAL_CHARBOARD);
    // // }
}