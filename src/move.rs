use crate::miscellaneous::{PieceType, Square};
use crate::charboard::SQUARE_NAMES;
use crate::state::{State, Termination};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveFlag {
    NormalMove = 0,
    Promotion = 1,
    EnPassant = 2,
    Castling = 3
}

impl MoveFlag {
    pub const unsafe fn from(value: u8) -> MoveFlag {
        assert!(value < 4, "Invalid MoveFlag value");
        std::mem::transmute::<u8, MoveFlag>(value)
    }
    
    pub const fn to_readable(&self) -> &str {
        match self {
            MoveFlag::NormalMove => "",
            MoveFlag::Promotion => "[P to ?]",
            MoveFlag::EnPassant => "[e.p.]",
            MoveFlag::Castling => "[castling]"
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move {
    // format: DDDDDDSSSSSSPPMM (D: destination, S: source, P: promotion PieceType value minus 2, M: MoveFlag value)
    pub value: u16,
}

impl Move {
    pub const DEFAULT_PROMOTION_VALUE: PieceType = PieceType::Rook;
    
    pub fn new(dst: Square, src: Square, promotion: PieceType, flag: MoveFlag) -> Move {
        assert!(promotion != PieceType::King && promotion != PieceType::Pawn, "Invalid promotion piece type");
        Move {
            value: ((dst as u16) << 10) | ((src as u16) << 4) | ((promotion as u16 - 2) << 2) | flag as u16
        }
    }
    
    pub fn new_non_promotion(dst: Square, src: Square, flag: MoveFlag) -> Move {
        Move::new(dst, src, Move::DEFAULT_PROMOTION_VALUE, flag)
    }
    
    pub const fn get_destination(&self) -> Square {
        let dst_int = (self.value >> 10) as u8;
        unsafe { Square::from(dst_int) }
    }
    
    pub const fn get_source(&self) -> Square {
        let src_int = ((self.value & 0b0000001111110000) >> 4) as u8;
        unsafe { Square::from(src_int) }
    }
    
    pub const fn get_promotion(&self) -> PieceType {
        let promotion_int = ((self.value & 0b0000000000001100) >> 2) as u8;
        unsafe { PieceType::from(promotion_int + 2) }
    }
    
    pub const fn get_flag(&self) -> MoveFlag {
        let flag_int = (self.value & 0b0000000000000011) as u8;
        unsafe { MoveFlag::from(flag_int) }
    }
    
    pub const fn unpack(&self) -> (Square, Square, PieceType, MoveFlag) {
        (self.get_destination(), self.get_source(), self.get_promotion(), self.get_flag())
    }

    pub fn to_readable(&self) -> String {
        let (dst, src, promotion, flag) = self.unpack();
        let (dst_str, src_str, promotion_char, flag_str) = (src.to_readable(), dst.to_readable(), promotion.to_char(), flag.to_readable());
        format!("{}{}{}", dst_str, src_str, flag_str.replace('?', &promotion_char.to_string()))
    }

    pub fn san(&self, initial_state: &State, final_state: &State, initial_state_moves: &Vec<Move>) -> String {
        let (dst, src, promotion, flag) = self.unpack();
        
        let dst_str = dst.to_readable();
        let src_str = src.to_readable();
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
        if final_state.termination == Some(Termination::Checkmate) {
            annotation_str = "#";
        }
        else if final_state.board.is_color_in_check(final_state.side_to_move) {
            annotation_str = "+";
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
    
    pub fn matches(&self, move_str: &str) -> bool { // todo: this function is temporary, eventually remove
        true
    }

    // pub fn matches(&self, move_str: &str) -> bool {
    //     if move_str.len() < 2 {
    //         return false;
    //     }
    //     let (src_str, dst_str, flag_str) = self.to_readable();
    //     if move_str == "0-0" || move_str == "O-O" {
    //         return flag_str == "castling" && dst_str.starts_with('g');
    //     }
    //     if move_str == "0-0-0" || move_str == "O-O-O" {
    //         return flag_str == "castling" && dst_str.starts_with('c');
    //     }
    //     let bytes = move_str.as_bytes();
    //     let mut end = move_str.len() - move_str.ends_with('+') as usize - move_str.ends_with('#') as usize;
    //     if bytes[end - 1].is_ascii_uppercase() {
    //         if flag_str != "P to ?".replace('?', &move_str[end - 1..end]) {
    //             return false;
    //         }
    //         end -= (bytes[end - 2] == b'=') as usize;
    //     }
    //     let is_capture = move_str.contains('x');
    //     if &move_str[end - 2..end] != dst_str {
    //         return false;
    //     }
    //     let is_piece_move = bytes[0].is_ascii_uppercase();
    //     if is_piece_move {
    //         if flag_str != &move_str[0..1] {
    //             return false;
    //         }
    //     }
    //     else {
    //         if !flag_str.contains('P') {
    //             return false;
    //         }
    //     }
    //     return match end - is_piece_move as usize - is_capture as usize {
    //         2 => true,
    //         3 => src_str.contains(bytes[is_piece_move as usize] as char),
    //         _ => false
    //     }
    // }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_readable())
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Move, MoveFlag};
    use crate::miscellaneous::{PieceType, Square};
    
    #[test]
    fn test_move() {
        for dst_square_int in Square::A8 as u8..Square::H1 as u8 {
            let dst_square = unsafe { Square::from(dst_square_int) };

            for src_square_int in Square::A8 as u8..Square::H1 as u8 {
                let src_square = unsafe { Square::from(src_square_int) };

                for promotion_piece_int in PieceType::Knight as u8..PieceType::Queen as u8 + 1 {
                    let promotion_piece = unsafe { PieceType::from(promotion_piece_int) };

                    for flag_int in 0..4 {
                        let flag = unsafe { MoveFlag::from(flag_int) };

                        let mv = Move::new(dst_square, src_square, promotion_piece, flag);
                        assert_eq!(mv.get_destination(), dst_square);
                        assert_eq!(mv.get_source(), src_square);
                        assert_eq!(mv.get_promotion(), promotion_piece);
                        assert_eq!(mv.get_flag(), flag);
                    }
                }
            }
        }
    }

    #[test]
    fn test_san() {
        // todo: implement
    }
}