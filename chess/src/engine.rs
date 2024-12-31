use cozy_chess::{Board, Move};

pub trait Engine {
    fn play(&self, board: &Board) -> Option<Move>;
}
