//! Contains the Termination enum and its implementation.

/// Represents the different ways a game can end.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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