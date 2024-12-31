use cozy_chess::{Board, Move};
use rand::Rng;

pub fn rand_num(max: usize) -> usize {
    if max <= 0 {
        panic!("Expected non-zero range. Got {}", max);
    }
    rand::thread_rng().gen_range(0..max)
}

pub fn rand_item<'l, T>(items: &'l Vec<T>) -> Option<&T> {
    if items.is_empty() {
        None 
    } else {
        Some(&items[rand_num(items.len())])
    }
}

pub fn random_move(board: &Board) -> Option<Move> {
    rand_item(&available_moves(board)).cloned()
}
