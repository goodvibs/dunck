#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Termination {
    Checkmate,
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule
}

impl Termination {
    pub fn is_decisive(&self) -> bool {
        self == &Termination::Checkmate
    }

    pub fn is_draw(&self) -> bool {
        !self.is_decisive()
    }
}