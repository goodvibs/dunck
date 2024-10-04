use crate::masks::{STARTING_KING_ROOK_GAP_SHORT, STARTING_KING_SIDE_ROOK, STARTING_QUEEN_SIDE_ROOK};
use crate::miscellaneous::{Color, PieceType, Square};
use crate::r#move::Move;
use crate::r#move::move_flag::MoveFlag;
use crate::state::State;
use crate::state::context::Context;
use crate::state::termination::Termination;

impl State {
    pub fn make_move(&mut self, mv: Move) { // todo: split into smaller functions for unit testing
        let (dst_square, src_square, promotion, flag) = mv.unpack();
        let dst_mask = dst_square.to_mask();
        let src_mask = src_square.to_mask();
        let src_dst = src_mask | dst_mask;

        let mut new_context = Context::new(
            self.context.halfmove_clock + 1,
            -1,
            self.context.castling_rights.clone(),
            PieceType::NoPieceType,
            Some(self.context.clone())
        );

        let castling_color_adjustment = self.side_to_move as usize * 2;
        let opposite_color = self.side_to_move.flip();
        let previous_castling_rights = self.context.castling_rights.clone();

        self.board.bb_by_color[self.side_to_move as usize] ^= src_dst; // sufficient for all moves except the rook in castling

        match flag {
            MoveFlag::NormalMove | MoveFlag::Promotion => {
                self.board.bb_by_color[opposite_color as usize] &= !dst_mask; // clear opposite color piece presence

                // remove captured piece and get captured piece type
                let captured_piece = self.board.remove_and_get_captured_piece_type_at(dst_mask);
                if captured_piece != PieceType::NoPieceType {
                    new_context.captured_piece = captured_piece;
                    new_context.halfmove_clock = 0;
                    if captured_piece == PieceType::Rook {
                        let king_side_rook_mask = STARTING_KING_SIDE_ROOK[opposite_color as usize];
                        let queen_side_rook_mask = STARTING_QUEEN_SIDE_ROOK[opposite_color as usize];
                        let right_shift: u8 = match opposite_color {
                            Color::White => 0,
                            Color::Black => 2
                        };
                        if dst_mask & king_side_rook_mask != 0 {
                            new_context.castling_rights &= !(0b00001000 >> right_shift);
                        }
                        else if dst_mask & queen_side_rook_mask != 0 {
                            new_context.castling_rights &= !(0b00000100 >> right_shift);
                        }
                    }
                }

                if flag == MoveFlag::Promotion {
                    new_context.halfmove_clock = 0;
                    self.board.bb_by_piece_type[PieceType::Pawn as usize] &= !src_mask;
                    self.board.bb_by_piece_type[promotion as usize] |= dst_mask;
                }
                else { // flag == MoveFlag::NormalMove
                    let moved_piece = self.board.get_piece_type_at(src_mask);

                    self.board.bb_by_piece_type[moved_piece as usize] &= !src_mask;
                    self.board.bb_by_piece_type[moved_piece as usize] |= dst_mask;

                    match moved_piece {
                        PieceType::Pawn => {
                            new_context.halfmove_clock = 0;
                            if dst_mask & (src_mask << 16) != 0 || dst_mask & (src_mask >> 16) != 0 { // double pawn push
                                new_context.double_pawn_push = (src_square as u8 % 8) as i8;
                            }
                        },
                        PieceType::King => {
                            new_context.castling_rights &= !0b00001100 >> castling_color_adjustment;
                        },
                        PieceType::Rook => {
                            let is_king_side = src_mask & (1u64 << (self.side_to_move as u64 * 7 * 8));
                            let is_queen_side = src_mask & (0b10000000u64 << (self.side_to_move as u64 * 7 * 8));
                            let king_side_mask = (is_king_side != 0) as u8 * (0b00001000 >> castling_color_adjustment);
                            let queen_side_mask = (is_queen_side != 0) as u8 * (0b00000100 >> castling_color_adjustment);
                            new_context.castling_rights &= !(king_side_mask | queen_side_mask);
                        },
                        _ => {}
                    }
                }
            },
            MoveFlag::EnPassant => { // en passant capture
                let en_passant_capture = ((dst_mask << 8) * self.side_to_move as u64) | ((dst_mask >> 8) * opposite_color as u64);
                self.board.bb_by_piece_type[PieceType::Pawn as usize] ^= src_dst | en_passant_capture;
                self.board.bb_by_color[opposite_color as usize] &= !en_passant_capture;
                new_context.captured_piece = PieceType::Pawn;
                new_context.halfmove_clock = 0;
            },
            MoveFlag::Castling => { // src is king's origin square, dst is king's destination square
                new_context.castling_rights &= !0b00001100 >> castling_color_adjustment;

                self.board.bb_by_piece_type[PieceType::King as usize] ^= src_dst;

                let is_king_side = dst_mask & STARTING_KING_ROOK_GAP_SHORT[self.side_to_move as usize] != 0;

                let rook_src_square = match is_king_side {
                    true => unsafe { Square::from(src_square as u8 + 3) },
                    false => unsafe { Square::from(src_square as u8 - 4) }
                };
                let rook_dst_square = match is_king_side {
                    true => unsafe { Square::from(src_square as u8 + 1) },
                    false => unsafe { Square::from(src_square as u8 - 1) }
                };
                let rook_src_dst = rook_src_square.to_mask() | rook_dst_square.to_mask();

                self.board.bb_by_color[self.side_to_move as usize] ^= rook_src_dst;
                self.board.bb_by_piece_type[PieceType::Rook as usize] ^= rook_src_dst;
            }
        }

        // update data members
        self.halfmove += 1;
        self.side_to_move = opposite_color;
        self.context = Box::new(new_context);
        self.in_check = self.board.is_color_in_check(self.side_to_move);
        self.board.bb_by_piece_type[PieceType::AllPieceTypes as usize] = self.board.bb_by_color[Color::White as usize] | self.board.bb_by_color[Color::Black as usize];

        if self.board.are_both_sides_insufficient_material() {
            self.termination = Some(Termination::InsufficientMaterial);
        }
        else if self.context.halfmove_clock == 100 { // fifty move rule
            self.termination = Some(Termination::FiftyMoveRule);
        }
        else {
            // update Zobrist table
            let position_count = self.increment_position_count();

            // check for repetition
            if position_count == 3 {
                self.termination = Some(Termination::ThreefoldRepetition);
            }
        }
    }

    pub fn unmake_move(&mut self, mv: Move) {
        // todo
    }
}