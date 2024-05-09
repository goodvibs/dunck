use crate::enums::*;
use crate::preload::ZOBRIST_TABLE;
use crate::attacks::*;
use crate::bitboard::{get_squares_from_bb, Bitboard, unpack_bb};
use crate::charboard::{cb_to_string, Charboard};
use crate::masks::*;

#[derive(Eq, PartialEq, Clone)]
pub struct Board {
    pub bb_by_piece_type: [Bitboard; PieceType::LIMIT],
    pub bb_by_color: [Bitboard; 2],
    // pub colored_piece_count: [u8; ColoredPiece::LIMIT],
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
            // colored_piece_count: [
            //     30,
            //     8,
            //     2,
            //     2,
            //     2,
            //     1,
            //     0,
            //     0,
            //     1,
            //     8,
            //     2,
            //     2,
            //     2,
            //     1,
            //     1
            // ]
        }
    }

    pub fn blank() -> Board {
        Board {
            bb_by_piece_type: [0; PieceType::LIMIT],
            bb_by_color: [0; 2],
            // colored_piece_count: [0; ColoredPiece::LIMIT]
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

    pub fn clear_and_put_colored_piece_at(&mut self, colored_piece: ColoredPiece, mask: Bitboard) {
        self.clear_piece_at(mask);
        self.put_colored_piece_at(colored_piece, mask);
    }

    pub fn clear_piece_at(&mut self, mask: Bitboard) {
        for piece_type_int in PieceType::Pawn as usize..PieceType::LIMIT {
            self.bb_by_piece_type[piece_type_int] &= !mask;
        }
        for color_int in Color::White as usize..Color::Black as usize + 1 {
            self.bb_by_color[color_int] &= !mask;
        }
        self.bb_by_piece_type[PieceType::AllPieceTypes as usize] &= !mask;
    }

    pub fn put_colored_piece_at(&mut self, colored_piece: ColoredPiece, mask: Bitboard) {
        let piece_type = colored_piece.get_piece_type();
        let color = colored_piece.get_color();

        let piece_type_int = piece_type as usize;
        let color_int = color as usize;

        self.bb_by_piece_type[piece_type_int] |= mask;
        self.bb_by_color[color_int] |= mask;
        self.bb_by_piece_type[PieceType::AllPieceTypes as usize] |= mask;
    }
    
    pub fn get_piece_type_at(&self, square_mask: Bitboard) -> PieceType {
        for piece_type_int in PieceType::Pawn as usize..PieceType::LIMIT {
            if self.bb_by_piece_type[piece_type_int] & square_mask != 0 {
                return unsafe { PieceType::from(piece_type_int as u8) };
            }
        }
        PieceType::NoPieceType
    }
    
    pub const fn get_colored_piece_bb(&self, colored_piece: ColoredPiece) -> Bitboard {
        self.bb_by_piece_type[colored_piece as usize & 0b0111] & self.bb_by_color[colored_piece.get_color() as usize]
    }

    pub fn is_valid(&self) -> bool {
        let white_bb = self.bb_by_color[Color::White as usize];
        let black_bb = self.bb_by_color[Color::Black as usize];
        if white_bb & black_bb != 0 {
            return false;
        }

        let all_occupancy_bb = self.bb_by_piece_type[PieceType::AllPieceTypes as usize];

        if (white_bb | black_bb) != all_occupancy_bb {
            return false;
        }

        let mut all_occupancy_bb_test: Bitboard = 0;

        for piece_type_int in PieceType::Pawn as usize..PieceType::LIMIT {
            let piece_bb = self.bb_by_piece_type[piece_type_int];

            if piece_bb & all_occupancy_bb != piece_bb {
                return false;
            }

            if (piece_bb & white_bb) | (piece_bb & black_bb) != piece_bb {
                return false;
            }

            if piece_bb & all_occupancy_bb_test != 0 {
                return false;
            }
            all_occupancy_bb_test |= piece_bb;
        }

        if self.get_colored_piece_bb(ColoredPiece::WhiteKing).count_ones() != 1 {
            return false;
        }

        if self.get_colored_piece_bb(ColoredPiece::BlackKing).count_ones() != 1 {
            return false;
        }

        all_occupancy_bb_test == all_occupancy_bb
    }

    pub fn zobrist_hash(&self) -> u64 {
        let mut hash: u64 = 0;
        for piece_type_int in PieceType::Pawn as u8..PieceType::King as u8 { // skip PieceType::NoPieceType, PieceType::King
            let piece_bb = self.bb_by_piece_type[piece_type_int as usize];
            for color_int in Color::White as u8..Color::Black as u8 + 1 {
                let color_bb = self.bb_by_color[color_int as usize];
                let combined_bb = piece_bb & color_bb;
                for index in get_squares_from_bb(combined_bb) {
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

    // pub fn from_cb(cb: Charboard) -> Board {
    //     let mut board = Board::blank();
    //     for i in 0..8 {
    //         for j in 0..8 {
    //             let mask = 1 << (63 - (i * 8 + j));
    //             let piece = ColoredPiece::from_char(cb[i][j]);
    //             if piece != ColoredPiece::NoPiece {
    //                 board.bb_by_piece_type[piece as usize] |= mask;
    //                 board.bb_by_color[piece.get_color() as usize] |= mask;
    //                 board.bb_by_piece_type[PieceType::AllPieceTypes as usize] |= mask;
    //                 // board.colored_piece_count[piece as usize] += 1;
    //             }
    //         }
    //     }
    //     board
    // }

    pub fn to_cb(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for i in 0..64 {
            let mask = 1 << (63 - i);
            let piece_type = self.get_piece_type_at(mask);
            let color = if self.bb_by_color[Color::White as usize] & mask != 0 { Color::White } else { Color::Black };
            cb[i / 8][i % 8] = ColoredPiece::from(color, piece_type).to_char();
        }
        cb
    }

    pub fn to_cb_pretty(&self) -> Charboard {
        let mut cb: Charboard = [[' '; 8]; 8];
        for i in 0..64 {
            let mask = 1 << (63 - i);
            let piece_type = self.get_piece_type_at(mask);
            let color = if self.bb_by_color[Color::White as usize] & mask != 0 { Color::White } else { Color::Black };
            cb[i / 8][i % 8] = ColoredPiece::from(color, piece_type).to_char_pretty();
        }
        cb
    }

    pub fn print(&self) {
        println!("{}", self);
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", cb_to_string(&self.to_cb_pretty()).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::charboard::{EMPTY_CHARBOARD, INITIAL_CHARBOARD};
    
    #[test]
    fn test_put_colored_piece_at() {
        let mut board = Board::blank();
        let mask = Square::A1.to_mask();
        board.put_colored_piece_at(ColoredPiece::WhitePawn, mask);
        assert_eq!(board.get_piece_type_at(mask), PieceType::Pawn);
        assert_eq!(board.bb_by_piece_type[PieceType::Pawn as usize], mask);
        assert_eq!(board.bb_by_color[Color::White as usize], mask);
        assert_eq!(board.bb_by_piece_type[PieceType::AllPieceTypes as usize], mask);
        board.put_colored_piece_at(ColoredPiece::BlackPawn, mask);
        assert_eq!(board.get_piece_type_at(mask), PieceType::Pawn);
        assert_eq!(board.bb_by_piece_type[PieceType::Pawn as usize], mask);
        assert_eq!(board.bb_by_color[Color::Black as usize], mask);
        assert_eq!(board.bb_by_piece_type[PieceType::AllPieceTypes as usize], mask);
    }
    
    #[test]
    fn test_clear_piece_at() {
        let mut board = Board::blank();
        let mask = Square::A1.to_mask() | Square::B1.to_mask();
        board.put_colored_piece_at(ColoredPiece::WhitePawn, mask);
        board.clear_piece_at(mask);
        assert_eq!(board.get_piece_type_at(mask), PieceType::NoPieceType);
        assert_eq!(board.bb_by_piece_type[PieceType::Pawn as usize], 0);
        assert_eq!(board.bb_by_color[Color::White as usize], 0);
        assert_eq!(board.bb_by_piece_type[PieceType::AllPieceTypes as usize], 0);
        
        let mut board = Board::initial();
        board.clear_piece_at(mask);
        assert_eq!(board.get_piece_type_at(mask), PieceType::NoPieceType);
        assert_eq!(board.bb_by_piece_type[PieceType::Pawn as usize], STARTING_WP | STARTING_BP);
        assert_eq!(board.bb_by_color[Color::White as usize], STARTING_WHITE & !mask);
        assert_eq!(board.bb_by_color[Color::Black as usize], STARTING_BLACK);
        assert_eq!(board.bb_by_piece_type[PieceType::Rook as usize], (STARTING_WR & !mask) | STARTING_BR);
        assert_eq!(board.bb_by_piece_type[PieceType::Knight as usize], (STARTING_WN & !mask) | STARTING_BN);
        assert_eq!(board.bb_by_piece_type[PieceType::Bishop as usize], STARTING_WB | STARTING_BB);
        assert_eq!(board.bb_by_piece_type[PieceType::Queen as usize], STARTING_WQ | STARTING_BQ);
        assert_eq!(board.bb_by_piece_type[PieceType::King as usize], STARTING_WK | STARTING_BK);
        assert_eq!(board.bb_by_piece_type[PieceType::AllPieceTypes as usize], STARTING_ALL & !mask);
        assert!(board.is_valid());
    }

    #[test]
    fn test_board_is_valid() {
        let mut board = Board::blank();
        assert!(!board.is_valid());
        board.put_colored_piece_at(ColoredPiece::WhiteKing, Square::E1.to_mask());
        assert!(!board.is_valid());
        board.put_colored_piece_at(ColoredPiece::BlackKing, Square::F6.to_mask());
        assert!(board.is_valid());
        board.put_colored_piece_at(ColoredPiece::WhiteKing, Square::E8.to_mask());
        assert!(!board.is_valid());
        board.put_colored_piece_at(ColoredPiece::BlackKing, Square::E8.to_mask());
        assert!(!board.is_valid());

        let mut board = Board::initial();
        assert!(board.is_valid());

        board.put_colored_piece_at(ColoredPiece::BlackBishop, Square::C5.to_mask());
        assert!(board.is_valid());

        let mut board = Board::initial();
        board.put_colored_piece_at(ColoredPiece::WhitePawn, Square::A1.to_mask());
        assert!(!board.is_valid());

        let mut board = Board::initial();
        board.put_colored_piece_at(ColoredPiece::WhitePawn, Square::A1.to_mask());
        assert!(!board.is_valid());

        let mut board = Board::initial();
        board.clear_piece_at(Square::A1.to_mask());
        assert!(board.is_valid());
        board.clear_piece_at(Square::E1.to_mask());
        assert!(!board.is_valid());
    }

    #[test]
    fn test_blank_board() {
        let mut board = Board::blank();
        assert!(!board.is_valid());
        board.put_colored_piece_at(ColoredPiece::WhiteKing, Square::E1.to_mask());
        assert!(!board.is_valid());
        board.put_colored_piece_at(ColoredPiece::BlackKing, Square::F6.to_mask());
        assert!(board.is_valid());
        
        let cb = Board::blank().to_cb();
        assert_eq!(cb, EMPTY_CHARBOARD);
    }

    #[test]
    fn test_initial_board() {
        let board = Board::initial();
        assert!(board.is_valid());
        let cb = board.to_cb();
        assert_eq!(cb, INITIAL_CHARBOARD);
    }

    #[test]
    fn test_get_piece_type_at() {
        let board = Board::initial();
        for i in 0..64 {
            let mask = 1 << (63 - i);
            let colored_piece_expected = ColoredPiece::from_char(INITIAL_CHARBOARD[i / 8][i % 8]);
            let piece_type_expected = colored_piece_expected.get_piece_type();
            assert_eq!(board.get_piece_type_at(mask), piece_type_expected);
        }

        let board = Board::blank();
        for i in 0..64 {
            let mask = 1 << (63 - i);
            assert_eq!(board.get_piece_type_at(mask), PieceType::NoPieceType);
        }
    }
    
    #[test]
    fn test_get_colored_piece_bb() {
        let board = Board::initial();
        for piece_type_int in PieceType::Pawn as u8..PieceType::LIMIT as u8 {
            for color_int in Color::White as u8..Color::Black as u8 + 1 {
                let colored_piece = ColoredPiece::from(Color::from(color_int != 0), unsafe { PieceType::from(piece_type_int) });
                let colored_piece_bb = board.get_colored_piece_bb(colored_piece);
                let expected_bb = board.bb_by_piece_type[piece_type_int as usize] & board.bb_by_color[color_int as usize];
                assert_eq!(colored_piece_bb, expected_bb);
            }
        }
    }

    #[test]
    pub fn test_are_both_sides_insufficient_material() {
        // todo
    }
    
    #[test]
    pub fn test_is_in_check() {
        // todo
    }

    // #[test]
    // fn test_from_cb() {
    //     let board = Board::from_cb(INITIAL_CHARBOARD);
    //     let cb = board.to_cb();
    //     assert_eq!(cb, INITIAL_CHARBOARD);
    // }
}