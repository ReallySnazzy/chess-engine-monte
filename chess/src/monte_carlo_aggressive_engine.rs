use cozy_chess::{Board, Move};
use log::{info, warn};

use crate::random;

pub struct MonteCarloAggressiveEngine;

const MOVES_TO_SIMULATE: i32 = 15;
const STEPS_TO_SIMULATE: i32 = 6;
const DOWN_STEPS_PER_LAYER: i32 = 2;

// FIXME: Since first move has no variety and only is random. This is effectively just more costly
// random lol.

fn simulate_random(board: Board, remaining_steps: i32, breadth: i32, current_score: f32) -> (f32, Vec<Move>) {
    let mut board = board;
    if remaining_steps <= 0 {
        return (current_score, vec![]);
    }
    // Make a random play
    let Some(next_move) = random::random_move(&board) else {
        info!("Player out of moves");
        return (current_score, vec![]);
    };
    match board.try_play(next_move) {
        Ok(_) => (),
        Err(e) => {
            warn!("Failed to make play: {}", e);
            panic!("Failed to make play: {}", e);
        }
    }
    // Record play score
    let piece = board.piece_on(next_move.to);
    let additional_score = match piece {
        Some(cozy_chess::Piece::King) => 100f32,
        Some(cozy_chess::Piece::Queen) => 25f32,
        Some(cozy_chess::Piece::Bishop) => 5f32,
        Some(cozy_chess::Piece::Rook) => 5f32,
        Some(cozy_chess::Piece::Knight) => 5f32,
        Some(cozy_chess::Piece::Pawn) => 1f32,
        None => -1f32,
    };
    // Pick random opponent move
    let Some(opponent_move) = random::random_move(&board) else {
        info!("Opponent out of moves");
        return (current_score, vec![]);
    };
    match board.try_play(opponent_move) {
        Err(e) => {
            warn!("Failed to play random move: {}", e);
            panic!("Failed to play random move: {}", e);
        },
        _ => ()
    }
    // Sample random moves
    let mut best_score = -100f32;
    let mut best_moves = vec!();
    for _ in 0..breadth {
        let (score, moves) = simulate_random(
            board.clone(), 
            remaining_steps - 1, 
            breadth - DOWN_STEPS_PER_LAYER, 
            current_score + additional_score
        );
        if score > best_score {
            best_score = score;
            best_moves = moves;
        }
    }
    let mut last_move_and_best_moves = Vec::new();
    last_move_and_best_moves.push(next_move);
    last_move_and_best_moves.extend(best_moves.into_iter());
    return (best_score, last_move_and_best_moves);
}

impl crate::engine::Engine for MonteCarloAggressiveEngine {
    fn play(&self, board: &Board) -> Option<Move> {
        simulate_random(board.clone(), STEPS_TO_SIMULATE, MOVES_TO_SIMULATE, 0f32)
            .1
            .into_iter()
            .next()
    }
}
