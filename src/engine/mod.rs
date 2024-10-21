mod syzygy;

use lazy_static::lazy_static;
use crate::r#move::Move;
use crate::state::{State, Termination};
use crate::utils::Color;

pub use mocats::*;
use pyrrhic_rs::TableBases;
use crate::engine::syzygy::DunckAdapter;

lazy_static! {
    pub static ref TB: TableBases<DunckAdapter> = TableBases::<DunckAdapter>::new("src/engine/syzygy/3-4-5").unwrap();
}

impl GameAction for Move {}

impl Player for Color {}

impl GameState<Move, Color> for State {
    fn get_actions(&self) -> Vec<Move> {
        self.calc_legal_moves()
    }

    fn apply_action(&mut self, action: &Move) {
        self.make_move(*action);
        // self.update_with_tb_if_eligible(&TB);
    }

    fn get_turn(&self) -> Color {
        self.side_to_move
    }

    fn get_reward_for_player(&self, player: Color) -> f32 {
        match &self.termination {
            Some(termination) => match termination {
                Termination::Checkmate => {
                    if self.side_to_move == player {
                        -1.0
                    } else {
                        1.0
                    }
                }
                _ => 0.0,
            },
            None => {
                match self.side_to_move == player {
                    true => -1.0,
                    false => 1.0,
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::State;
    use crate::utils::Color;

    #[test]
    fn test_initial_game_state() {
        let state = State::initial();
        let tree_policy = UctPolicy::new(2.0);
        let mut search_tree = SearchTree::new(state, tree_policy);
        search_tree.run(500);
        let best_action = search_tree.get_best_action();
        println!("{}", search_tree);
        println!("Best action: {}", best_action.unwrap());
    }

    #[test]
    fn test_game_state() {
        let state = State::from_fen("r1bqkb1r/ppppp1pp/n6n/5P2/P7/8/1PPP1PPP/RNBQKBNR w KQkq - 1 4").unwrap();
        let tree_policy = UctPolicy::new(2.0);
        let mut search_tree = SearchTree::new(state, tree_policy);
        search_tree.run(1000);
        let best_action = search_tree.get_best_action();
        println!("{}", search_tree);
        println!("Best action: {}", best_action.unwrap());
    }

    #[test]
    fn test_game_state2() {
        let state = State::from_fen("5K2/1p2pp2/P7/1b3Nk1/4B1pN/5r2/p3Pp2/4Q3 w - - 0 1").unwrap();
        let tree_policy = UctPolicy::new(2.0);
        let mut search_tree = SearchTree::new(state, tree_policy);
        search_tree.run(1000);
        let best_action = search_tree.get_best_action();
        println!("{}", search_tree);
        println!("Best action: {}", best_action.unwrap());
    }

    #[test]
    fn test_game_state3() {
        let state = State::from_fen("5K2/1p2pp2/P7/1b3Nk1/4B1p1/5N2/p3Pp2/4Q3 b - - 0 1").unwrap();
        let tree_policy = UctPolicy::new(2.0);
        let mut search_tree = SearchTree::new(state, tree_policy);
        search_tree.run(5000);
        let best_action = search_tree.get_best_action();
        println!("{}", search_tree);
        println!("Best action: {}", best_action.unwrap());
    }
}