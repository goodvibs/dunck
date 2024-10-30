use crate::state::State;
use crate::utils::{Color, PieceType};

pub fn evaluate_non_terminal_state(state: &State, for_color: Color) -> f64 {
    let mut scores = [0.0, 0.0];
    for color in Color::iter() {
        let color_mask = state.board.color_masks[color as usize];
        for piece_type in PieceType::iter_between(PieceType::Pawn, PieceType::Queen) {
            let piece_mask = state.board.piece_type_masks[piece_type as usize];
            let mask = color_mask & piece_mask;
            let count = mask.count_ones() as f32;
            scores[color as usize] += PIECE_VALUES[piece_type as usize - 1] * count;
        }
    }
    match scores[for_color as usize] - scores[for_color.flip() as usize] {
        x if x > 0.0 => 0.5,
        x if x < 0.0 => -0.5,
        _ => 0.
    }
}

const PIECE_VALUES: [f32; 5] = [
    1.0, 3.0, 3.0, 5.0, 9.0
];
