use crate::state::State;
use crate::utils::{Color, PieceType};

impl State {
    pub fn evaluate(&self) -> f32 {
        let mut scores = [0.0, 0.0];
        for color in Color::iter() {
            let color_mask = self.board.color_masks[color as usize];
            for piece_type in PieceType::iter_between(PieceType::Pawn, PieceType::Queen) {
                let piece_mask = self.board.piece_type_masks[piece_type as usize];
                let mask = color_mask & piece_mask;
                let count = mask.count_ones() as f32;
                scores[color as usize] += PIECE_VALUES[piece_type as usize - 1] * count;
            }
        }
        scores[self.side_to_move as usize] - scores[self.side_to_move.flip() as usize]
    }
}

// fn normalize(x: f32) -> f32 {
//     1.0 / (1.0 + (-x).exp()) - 0.5
// }

const PIECE_VALUES: [f32; 5] = [
    1.0, 3.0, 3.0, 5.0, 9.0
];

#[cfg(test)]
mod tests {
    use crate::utils::{ColoredPiece, Square};
    use super::*;

    #[test]
    fn test_initial() {
        let state = State::initial();
        let score = state.evaluate();
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_foo() {
        let mut state = State::initial();
        state.board.remove_colored_piece_at(ColoredPiece::WhiteBishop, Square::C1);
        let score = state.evaluate();
        assert!(score < 0.0);
        state.board.put_colored_piece_at(ColoredPiece::WhiteQueen, Square::D4);
        let score = state.evaluate();
        assert!(score > 0.0);
    }
}