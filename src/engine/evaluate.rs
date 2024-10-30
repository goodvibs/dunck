use crate::state::State;
use crate::utils::{Color, PieceType};

pub fn evaluate_non_terminal_state(state: &State, for_color: Color) -> f64 {
    let mut scores = [0.0, 0.0];
    for color in Color::iter() {
        let color_mask = state.board.color_masks[color as usize];
        for piece_type in PieceType::iter_between(PieceType::Pawn, PieceType::Queen) {
            let piece_mask = state.board.piece_type_masks[piece_type as usize];
            let mask = color_mask & piece_mask;
            let count = mask.count_ones() as f64;
            scores[color as usize] += PIECE_VALUES[piece_type as usize - 1] * count;
        }
    }

    // Calculate score difference from perspective of for_color
    let score_diff = scores[for_color as usize] - scores[for_color.flip() as usize];

    1.0 / (1.0 + (-0.5 * score_diff).exp())
}

const PIECE_VALUES: [f64; 5] = [
    1.0,  // Pawn
    3.0,  // Knight
    3.0,  // Bishop
    5.0,  // Rook
    9.0   // Queen
];