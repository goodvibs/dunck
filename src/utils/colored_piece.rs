use crate::utils::{Color, PieceType};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColoredPiece {
    NoPiece=0,
    WhitePawn=1, WhiteKnight=2, WhiteBishop=3, WhiteRook=4, WhiteQueen=5, WhiteKing=6,
    BlackPawn=9, BlackKnight=10, BlackBishop=11, BlackRook=12, BlackQueen=13, BlackKing=14
}

impl ColoredPiece {
    pub const LIMIT: usize = 15;
    pub const COLOR_DIFFERENCE: u8 = 8;

    pub const fn from(color: Color, piece_type: PieceType) -> ColoredPiece {
        let piece_type_int = piece_type as u8;
        let is_piece = piece_type_int != PieceType::NoPieceType as u8;
        let color_int_shifted = (is_piece as u8 & color as u8) << 3;
        unsafe { std::mem::transmute::<u8, ColoredPiece>(color_int_shifted | piece_type_int) }
    }

    pub const fn get_color(&self) -> Color {
        unsafe { std::mem::transmute::<u8, Color>(*self as u8 >> 3) }
    }

    pub const fn get_piece_type(&self) -> PieceType {
        unsafe { std::mem::transmute::<u8, PieceType>(*self as u8 & 0b111) }
    }

    pub const fn from_char(c: char) -> ColoredPiece {
        match c {
            'P' => ColoredPiece::WhitePawn,
            'N' => ColoredPiece::WhiteKnight,
            'B' => ColoredPiece::WhiteBishop,
            'R' => ColoredPiece::WhiteRook,
            'Q' => ColoredPiece::WhiteQueen,
            'K' => ColoredPiece::WhiteKing,
            'p' => ColoredPiece::BlackPawn,
            'n' => ColoredPiece::BlackKnight,
            'b' => ColoredPiece::BlackBishop,
            'r' => ColoredPiece::BlackRook,
            'q' => ColoredPiece::BlackQueen,
            'k' => ColoredPiece::BlackKing,
            _ => ColoredPiece::NoPiece
        }
    }

    pub const fn to_char(&self) -> char {
        match self {
            ColoredPiece::NoPiece => ' ',
            ColoredPiece::WhitePawn => 'P',
            ColoredPiece::WhiteKnight => 'N',
            ColoredPiece::WhiteBishop => 'B',
            ColoredPiece::WhiteRook => 'R',
            ColoredPiece::WhiteQueen => 'Q',
            ColoredPiece::WhiteKing => 'K',
            ColoredPiece::BlackPawn => 'p',
            ColoredPiece::BlackKnight => 'n',
            ColoredPiece::BlackBishop => 'b',
            ColoredPiece::BlackRook => 'r',
            ColoredPiece::BlackQueen => 'q',
            ColoredPiece::BlackKing => 'k'
        }
    }

    pub const fn to_char_pretty(&self) -> char {
        match self {
            ColoredPiece::NoPiece => ' ',
            ColoredPiece::WhitePawn => '♙',
            ColoredPiece::WhiteKnight => '♘',
            ColoredPiece::WhiteBishop => '♗',
            ColoredPiece::WhiteRook => '♖',
            ColoredPiece::WhiteQueen => '♕',
            ColoredPiece::WhiteKing => '♔',
            ColoredPiece::BlackPawn => '♟',
            ColoredPiece::BlackKnight => '♞',
            ColoredPiece::BlackBishop => '♝',
            ColoredPiece::BlackRook => '♜',
            ColoredPiece::BlackQueen => '♛',
            ColoredPiece::BlackKing => '♚'
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colored_piece() {
        assert_eq!(ColoredPiece::NoPiece as u8, 0);
        assert_eq!(ColoredPiece::WhitePawn as u8, 1);
        assert_eq!(ColoredPiece::BlackPawn as u8, 9);

        assert_eq!(ColoredPiece::LIMIT, 15);
        assert_eq!(ColoredPiece::COLOR_DIFFERENCE, 8);

        assert_eq!(ColoredPiece::from(Color::White, PieceType::Pawn), ColoredPiece::WhitePawn);
        assert_eq!(ColoredPiece::from(Color::Black, PieceType::Pawn), ColoredPiece::BlackPawn);

        assert_eq!(ColoredPiece::WhitePawn.get_color(), Color::White);
        assert_eq!(ColoredPiece::BlackPawn.get_color(), Color::Black);

        assert_eq!(ColoredPiece::WhitePawn.get_piece_type(), PieceType::Pawn);
        assert_eq!(ColoredPiece::BlackPawn.get_piece_type(), PieceType::Pawn);

        assert_eq!(ColoredPiece::from_char('P'), ColoredPiece::WhitePawn);
        assert_eq!(ColoredPiece::from_char('p'), ColoredPiece::BlackPawn);
        assert_eq!(ColoredPiece::from_char(' '), ColoredPiece::NoPiece);

        assert_eq!(ColoredPiece::WhitePawn.to_char(), 'P');
        assert_eq!(ColoredPiece::BlackPawn.to_char(), 'p');
        assert_eq!(ColoredPiece::NoPiece.to_char(), ' ');

        assert_eq!(ColoredPiece::WhitePawn.to_char_pretty(), '♙');
        assert_eq!(ColoredPiece::BlackPawn.to_char_pretty(), '♟');
        assert_eq!(ColoredPiece::NoPiece.to_char_pretty(), ' ');
    }
}