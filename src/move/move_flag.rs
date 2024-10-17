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