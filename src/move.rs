use crate::enums::{Color, Square};
use crate::charboard::SQUARE_NAMES;
use crate::state::{State, Termination};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveFlag {
    NoFlag = 0,
    KnightMove = 1,
    BishopMove = 2,
    RookMove = 3,
    QueenMove = 4,
    KingMove = 5,
    Castle = 6,
    PawnMove = 8,
    EnPassant = 10,
    PawnDoubleMove = 11,
    PromoteToQueen = 12,
    PromoteToKnight = 13,
    PromoteToRook = 14,
    PromoteToBishop = 15
}

impl MoveFlag {
    pub const unsafe fn from(value: u8) -> MoveFlag {
        // assert!(value < 16, "Invalid MoveFlag value: {}", value);
        std::mem::transmute::<u8, MoveFlag>(value)
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move {
    // format: FFFFTTTTTTSSSSSS
    // F = flag, T = target square, S = source square
    pub value: u16,
}

impl Move {
    pub fn new(src: Square, dst: Square, flag: MoveFlag) -> Move {
        Move {
            value: ((flag as u16) << 12) | ((dst as u16) << 6) | (src as u16)
        }
    }

    pub fn unpack(&self) -> (Square, Square, MoveFlag) {
        let src_int: u8 = (self.value & 0b0000000000111111) as u8;
        let dst_int: u8 = ((self.value & 0b0000111111000000) >> 6) as u8;
        let flag_int: u8 = ((self.value & 0b1111000000000000) >> 12) as u8;
        let src = unsafe { Square::from(src_int) };
        let dst = unsafe { Square::from(dst_int) };
        let flag = unsafe { MoveFlag::from(flag_int) };
        (src, dst, flag)
    }

    pub fn to_readable(&self) -> (&str, &str, &str) {
        let (src, dst, flag) = self.unpack();
        let src_str = SQUARE_NAMES[src as usize];
        let dst_str = SQUARE_NAMES[dst as usize];
        let flag_str = match flag {
            MoveFlag::PawnMove => "P",
            MoveFlag::PawnDoubleMove => "P2",
            MoveFlag::EnPassant => "Px",
            MoveFlag::KnightMove => "N",
            MoveFlag::BishopMove => "B",
            MoveFlag::RookMove => "R",
            MoveFlag::QueenMove => "Q",
            MoveFlag::KingMove => "K",
            MoveFlag::Castle => "castling",
            MoveFlag::PromoteToQueen => "P to Q",
            MoveFlag::PromoteToKnight => "P to N",
            MoveFlag::PromoteToRook => "P to R",
            MoveFlag::PromoteToBishop => "P to B",
            _ => ""
        };
        (src_str, dst_str, flag_str)
    }

    pub fn is_pawn_move(&self) -> bool {
        let (_, _, flag) = self.unpack();
        flag == MoveFlag::PawnMove ||
            flag == MoveFlag::PawnDoubleMove ||
            flag == MoveFlag::EnPassant ||
            flag as u8 >= MoveFlag::PromoteToQueen as u8
    }

    pub fn is_piece_move(&self) -> bool {
        !self.is_pawn_move()
    }

    pub fn san(&self, initial_state: &State, final_state: &State) -> String {
        let (src, dst, flag) = self.unpack();
        let src_str = SQUARE_NAMES[src as usize];
        let dst_str = SQUARE_NAMES[dst as usize];
        let (src_file, src_rank) = (src_str.chars().nth(0).unwrap(), src_str.chars().nth(1).unwrap());
        let (piece_str, promotion_str) = match flag {
            MoveFlag::PawnMove => ("", ""),
            MoveFlag::PawnDoubleMove => {
                return dst_str.to_string();
            },
            MoveFlag::EnPassant => {
                return format!("{}x{}", src_file, dst_str);
            },
            MoveFlag::KnightMove => ("N", ""),
            MoveFlag::BishopMove => ("B", ""),
            MoveFlag::RookMove => ("R", ""),
            MoveFlag::QueenMove => ("Q", ""),
            MoveFlag::KingMove => ("K", ""),
            MoveFlag::Castle => {
                return if dst_str.contains('g') {
                    "O-O".to_string()
                } else {
                    "O-O-O".to_string()
                }
            },
            MoveFlag::PromoteToQueen => ("", "=Q"),
            MoveFlag::PromoteToKnight => ("", "=N"),
            MoveFlag::PromoteToRook => ("", "=R"),
            MoveFlag::PromoteToBishop => ("", "=B"),
            _ => ("", "")
        };
        let is_capture = initial_state.board.bb_by_color[initial_state.turn.flip() as usize] != final_state.board.bb_by_color[initial_state.turn.flip() as usize];
        let capture_str = if is_capture { "x" } else { "" };
        let annotation_str;
        if final_state.termination == Some(Termination::Checkmate) {
            annotation_str = "#";
        }
        else if final_state.board.is_in_check(final_state.turn) {
            annotation_str = "+";
        }
        else {
            annotation_str = "";
        }
        let disambiguation_str;
        return match piece_str.is_empty() {
            true => {
                disambiguation_str = if is_capture {
                    src_file.to_string()
                } else {
                    "".to_string()
                };
                format!("{}{}{}{}{}{}", piece_str, disambiguation_str, capture_str, dst_str, promotion_str, annotation_str)
            }
            false => {
                let disambiguation_str_options = ["".to_string(), src_file.to_string(), src_rank.to_string()];
                let mut possible_sans = ["".to_string(), "".to_string(), "".to_string()];
                for (i, disambiguation_str) in disambiguation_str_options.iter().enumerate() {
                    possible_sans[i] = format!("{}{}{}{}{}{}", piece_str, disambiguation_str, capture_str, dst_str, promotion_str, annotation_str)
                }
                let possible_moves = initial_state.get_moves();
                for possible_san in possible_sans {
                    let mut is_ambiguous = false;
                    let mut has_match = false;
                    for mv in possible_moves.iter() {
                        if mv.matches(possible_san.as_str()) {
                            if has_match {
                                is_ambiguous = true;
                                break;
                            }
                            has_match = true;
                        }
                    }
                    if !is_ambiguous {
                        return possible_san;
                    }
                }
                format!("{}{}{}{}{}{}", piece_str, src_str, capture_str, dst_str, promotion_str, annotation_str)
            }
        };
    }

    pub fn matches(&self, move_str: &str) -> bool {
        if move_str.len() < 2 {
            return false;
        }
        let (src_str, dst_str, flag_str) = self.to_readable();
        if move_str == "0-0" || move_str == "O-O" {
            return flag_str == "castling" && dst_str.starts_with('g');
        }
        if move_str == "0-0-0" || move_str == "O-O-O" {
            return flag_str == "castling" && dst_str.starts_with('c');
        }
        let bytes = move_str.as_bytes();
        let mut end = move_str.len() - move_str.ends_with('+') as usize - move_str.ends_with('#') as usize;
        if bytes[end - 1].is_ascii_uppercase() {
            if flag_str != "P to ?".replace('?', &move_str[end - 1..end]) {
                return false;
            }
            end -= (bytes[end - 2] == b'=') as usize;
        }
        let is_capture = move_str.contains('x');
        if &move_str[end - 2..end] != dst_str {
            return false;
        }
        let is_piece_move = bytes[0].is_ascii_uppercase();
        if is_piece_move {
            if flag_str != &move_str[0..1] {
                return false;
            }
        }
        else {
            if !flag_str.contains('P') {
                return false;
            }
        }
        return match end - is_piece_move as usize - is_capture as usize {
            2 => true,
            3 => src_str.contains(bytes[is_piece_move as usize] as char),
            _ => false
        }
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (src_str, dst_str, flag_str) = self.to_readable();
        write!(f, "{}{}{}", src_str, dst_str, flag_str)
    }
}

impl std::fmt::Debug for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}