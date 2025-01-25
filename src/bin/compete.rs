use dunck::engine::evaluators::neural::conv_net_evaluator::ConvNetEvaluator;
use dunck::engine::mcts::mcts::{calc_puct_score, calc_uct_score, MCTS};
use dunck::engine::evaluators::random_rollout::RolloutEvaluator;
use dunck::state::State;

const MAX_GAME_DEPTH: usize = 400;

fn play_move(
    current_mcts: &mut MCTS,
    current_iterations: usize,
    opponent_mcts: &mut MCTS,
) -> bool {
    // Run the MCTS search for the current player
    current_mcts.run(current_iterations);

    // Attempt to take the best move; return false if no moves are found
    if let Ok((new_state, move_played)) = current_mcts.take_best_child() {
        // Clone the state to avoid borrow conflicts
        let initial_state = opponent_mcts.root.borrow().state_after_move.clone();

        // Generate the SAN notation for the move and print it
        let san = move_played.to_san(&initial_state, &new_state, &initial_state.calc_legal_moves());
        println!("Move played: {}", san);
        new_state.board.print();

        // Apply the move to the opponent's MCTS
        opponent_mcts
            .take_child_with_move(move_played, true)
            .expect("Failed to take child with move");

        true
    } else {
        false
    }
}

fn compete(
    mcts1: &mut MCTS,
    mcts1_num_iterations_per_move: usize,
    mcts2: &mut MCTS,
    mcts2_num_iterations_per_move: usize,
) {
    assert_eq!(mcts1.root.borrow().state_after_move, mcts2.root.borrow().state_after_move);

    for i in 0..MAX_GAME_DEPTH {
        println!("Move: {}", i);
        // Determine which MCTS instance is playing in the current turn
        if i % 2 == 0 {
            if !play_move(mcts1, mcts1_num_iterations_per_move, mcts2) {
                break;
            }
        } else {
            if !play_move(mcts2, mcts2_num_iterations_per_move, mcts1) {
                break;
            }
        }
        println!();
    }
}

fn main() {
    let rollout_evaluator = RolloutEvaluator::new(300);
    let mut rollout_mcts = MCTS::new(
        State::initial(),
        1.5,
        &rollout_evaluator,
        &calc_uct_score,
        false
    );
    
    let conv_net_evaluator = ConvNetEvaluator::new(4, 8);
    let mut conv_net_mcts = MCTS::new(
        State::initial(),
        1.5,
        &conv_net_evaluator,
        &calc_puct_score,
        false
    );
    
    compete(&mut rollout_mcts, 1000, &mut conv_net_mcts, 800);
}