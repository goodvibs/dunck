use crate::utils::{PieceType, Square};
use crate::r#move::{Move};
use crate::r#move::move_flag::MoveFlag;
use crate::state::{Board, State, Termination};

impl Move {
    /// Returns the SAN (Standard Algebraic Notation) representation of the move.
    /// Assumes that `final_state` has an updated termination
    pub fn to_san(&self, initial_state: &State, final_state: &State, initial_state_moves: &[Move]) -> String {
        let dst_square = self.get_destination();
        let src_square = self.get_source();
        let promotion = self.get_promotion();
        let flag = self.get_flag();

        let src_file = src_square.get_file_char();

        let mut promotion_str = String::new();
        let is_capture;
        let moved_piece;

        let annotation_str = match final_state.termination {
            Some(Termination::Checkmate) => "#",
            _ => if final_state.board.is_color_in_check(final_state.side_to_move) { "+" } else { "" },
        };

        match flag {
            MoveFlag::Castling => {
                return if dst_square.get_file() == 6 {
                    format!("O-O{}", annotation_str)
                } else {
                    format!("O-O-O{}", annotation_str)
                }
            },
            MoveFlag::EnPassant => {
                is_capture = true;
                moved_piece = PieceType::Pawn;
            },
            MoveFlag::NormalMove | MoveFlag::Promotion => {
                is_capture = initial_state.board.color_masks[final_state.side_to_move as usize] != final_state.board.color_masks[final_state.side_to_move as usize];

                if flag == MoveFlag::Promotion {
                    promotion_str = format!("={}", promotion.to_char());
                    moved_piece = PieceType::Pawn;
                }
                else {
                    moved_piece = initial_state.board.get_piece_type_at(src_square);
                }
            }
        }

        let capture_str = if is_capture { "x" } else { "" };

        let piece_str = match moved_piece {
            PieceType::Pawn => {
                if is_capture {
                    src_file.to_string()
                }
                else {
                    "".to_string()
                }
            },
            _ => moved_piece.to_char().to_string()
        };
        
        let disambiguation_str = get_disambiguation(moved_piece, src_square, dst_square, initial_state_moves, &initial_state.board);

        format!("{}{}{}{}{}{}", piece_str, disambiguation_str, capture_str, dst_square.to_string(), promotion_str, annotation_str)
    }
}

fn get_disambiguation(moved_piece: PieceType, src_square: Square, dst_square: Square, initial_state_moves: &[Move], initial_state_board: &Board) -> String {
    if moved_piece != PieceType::Pawn && moved_piece != PieceType::King {
        let mut clashes = Vec::new();

        for other_move in initial_state_moves.iter() {
            let other_src_square = other_move.get_source();
            let other_dst_square = other_move.get_destination();
            if src_square == other_src_square { // same move
                continue;
            }
            if dst_square == other_dst_square && moved_piece == initial_state_board.get_piece_type_at(other_src_square) {
                clashes.push(other_move);
            }
        }

        if !clashes.is_empty() {
            let mut is_file_unique = true;
            let mut is_rank_unique = true;

            for other_move in clashes {
                if other_move.get_source().get_file() == src_square.get_file() {
                    is_file_unique = false;
                }
                if other_move.get_source().get_rank() == src_square.get_rank() {
                    is_rank_unique = false;
                }
            }

            return if is_file_unique {
                src_square.get_file_char().to_string()
            } else if is_rank_unique {
                (src_square.get_rank() + 1).to_string()
            } else {
                src_square.to_string()
            }
        }
    }

    String::new()
}

#[cfg(test)]
mod tests {
    
}