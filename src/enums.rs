#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Square {
    A8=0, B8=1, C8=2, D8=3, E8=4, F8=5, G8=6, H8=7,
    A7=8, B7=9, C7=10, D7=11, E7=12, F7=13, G7=14, H7=15,
    A6=16, B6=17, C6=18, D6=19, E6=20, F6=21, G6=22, H6=23,
    A5=24, B5=25, C5=26, D5=27, E5=28, F5=29, G5=30, H5=31,
    A4=32, B4=33, C4=34, D4=35, E4=36, F4=37, G4=38, H4=39,
    A3=40, B3=41, C3=42, D3=43, E3=44, F3=45, G3=46, H3=47,
    A2=48, B2=49, C2=50, D2=51, E2=52, F2=53, G2=54, H2=55,
    A1=56, B1=57, C1=58, D1=59, E1=60, F1=61, G1=62, H1=63
}

impl Square {
    pub const unsafe fn from(square_number: u8) -> Square {
        assert!(square_number < 64, "Square number out of bounds");
        std::mem::transmute::<u8, Square>(square_number)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Color {
    White=0, Black=1
}

impl Color {
    pub const fn from(is_black: bool) -> Color {
        unsafe { std::mem::transmute::<bool, Color>(is_black) }
    }
    
    pub const fn flip(&self) -> Color {
        unsafe { std::mem::transmute::<u8, Color>(!(self.clone() as u8)) }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceType {
    NoPieceType=0, Pawn=1, Knight=2, Bishop=3, Rook=4, Queen=5, King=6
}

impl PieceType {
    pub const LIMIT: usize = 7;
    pub const AllPieceTypes: PieceType = PieceType::NoPieceType;
    
    pub const unsafe fn from(piece_type_number: u8) -> PieceType {
        assert!(piece_type_number < PieceType::LIMIT as u8, "Piece type number out of bounds");
        std::mem::transmute::<u8, PieceType>(piece_type_number)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColoredPiece {
    NoPiece=0,
    WhitePawn=1, WhiteKnight=2, WhiteBishop=3, WhiteRook=4, WhiteQueen=5, WhiteKing=6,
    BlackPawn=9, BlackKnight=10, BlackBishop=11, BlackRook=12, BlackQueen=13, BlackKing=14
}

impl ColoredPiece {
    pub const LIMIT: usize = 15;
    pub const COLOR_DIFFERENCE: u8 = 8;

    pub const unsafe fn from(color: Color, piece_type: PieceType) -> ColoredPiece {
        std::mem::transmute::<u8, ColoredPiece>((color as u8) << 3 | piece_type as u8)
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
