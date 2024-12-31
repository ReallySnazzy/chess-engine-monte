pub struct RandomEngine;

impl crate::engine::Engine for RandomEngine {
    fn play(&self, board: &cozy_chess::Board) -> Option<cozy_chess::Move> {
        crate::random::random_move(&board)
    }
}
