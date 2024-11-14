use crate::engine::evaluation::{Evaluation, Evaluator};
use crate::r#move::Move;
use crate::state::State;
use crate::utils::{Color, PieceType};

#[derive(Clone)]
pub struct MaterialEvaluator {

}

impl Evaluator for MaterialEvaluator {
    fn evaluate(&self, state: &State) -> Evaluation {
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

        let score_diff = scores[state.side_to_move as usize] - scores[state.side_to_move.flip() as usize];

        let value = 2. * sigmoid(score_diff, 0.5) - 1.; // Normalize to [-1, 1]

        let legal_moves = state.calc_legal_moves();
        let policy: Vec<(Move, f64)> = legal_moves.iter().map(|mv| (mv.clone(), 1. / legal_moves.len() as f64)).collect();

        Evaluation {
            policy,
            value,
        }
    }
}

fn sigmoid(x: f64, a: f64) -> f64 {
    1.0 / (1.0 + (-a * x).exp())
}

const PIECE_VALUES: [f64; 5] = [
    1.0,  // Pawn
    3.0,  // Knight
    3.0,  // Bishop
    5.0,  // Rook
    9.0   // Queen
];