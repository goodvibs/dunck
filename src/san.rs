use crate::miscellaneous::PieceType;
use crate::r#move::Move;
use crate::r#move::move_flag::MoveFlag;
use crate::state::State;

impl Move {
    pub fn san(&self, initial_state: &State, final_state: &State, initial_state_moves: &Vec<Move>) -> String {
        let (dst, src, promotion, flag) = self.unpack();

        let dst_str = dst.readable();
        let src_str = src.readable();
        let (src_file, src_rank) = (src.get_file_char(), src.get_rank_char());

        let mut promotion_str = String::new();
        let is_capture;
        let moved_piece;

        match flag {
            MoveFlag::Castling => {
                return if dst_str.contains('g') {
                    "O-O".to_string()
                } else {
                    "O-O-O".to_string()
                }
            },
            MoveFlag::EnPassant => {
                is_capture = true;
                moved_piece = PieceType::Pawn;
            },
            MoveFlag::NormalMove | MoveFlag::Promotion => {
                is_capture = initial_state.board.bb_by_color[final_state.side_to_move as usize] != final_state.board.bb_by_color[final_state.side_to_move as usize];

                if flag == MoveFlag::Promotion {
                    promotion_str = format!("={}", promotion.to_char());
                    moved_piece = PieceType::Pawn;
                }
                else {
                    moved_piece = initial_state.board.get_piece_type_at(src.to_mask());
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

        let annotation_str;
        if final_state.board.is_color_in_check(final_state.side_to_move) {
            if final_state.get_legal_moves().is_empty() {
                annotation_str = "#";
            }
            else {
                annotation_str = "+";
            }
        }
        else {
            annotation_str = "";
        }

        let mut disambiguation_str = "".to_string();

        if moved_piece != PieceType::Pawn && moved_piece != PieceType::King {
            let mut clashes = Vec::new();

            for other_move in initial_state_moves.iter() {
                let other_src = other_move.get_source();
                let other_dst = other_move.get_destination();
                if src == other_src { // same move
                    continue;
                }
                if dst == other_move.get_destination() && moved_piece == initial_state.board.get_piece_type_at(other_src.to_mask()) {
                    clashes.push(other_move);
                }
            }

            if !clashes.is_empty() {
                let mut is_file_unique = true;
                let mut is_rank_unique = true;

                for other_move in clashes {
                    if other_move.get_source().get_file() == src.get_file() {
                        is_file_unique = false;
                    }
                    if other_move.get_source().get_rank() == src.get_rank() {
                        is_rank_unique = false;
                    }
                }

                if is_file_unique {
                    disambiguation_str = src_file.to_string();
                }
                else if is_rank_unique {
                    disambiguation_str = src_rank.to_string();
                }
                else {
                    disambiguation_str = src_str.to_string();
                }
            }
        }

        format!("{}{}{}{}{}{}", piece_str, disambiguation_str, capture_str, dst_str, promotion_str, annotation_str)
    }
}