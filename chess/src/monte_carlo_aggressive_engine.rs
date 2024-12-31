use cozy_chess::{Board, Move, Piece};
use log::{debug, info, warn};

use crate::random;

pub struct MonteCarloAggressiveEngine;

const MOVES_TO_SIMULATE: i32 = 30;
const STEPS_TO_SIMULATE: i32 = 5;
// The number to shrink the breadth of the search each move in the future
const DOWN_MOVES_PER_STEP: i32 = 4;
// Each move in the future has their score discounted by this factor
const FUTURE_DISCOUNT: f32 = 0.52f32;

fn score_capture(piece: &Option<Piece>) -> f32 {
    match piece {
        Some(cozy_chess::Piece::King) => 100f32,
        Some(cozy_chess::Piece::Queen) => 35f32,
        Some(cozy_chess::Piece::Rook) => 10f32,
        Some(cozy_chess::Piece::Knight) => 10f32,
        Some(cozy_chess::Piece::Bishop) => 5f32,
        Some(cozy_chess::Piece::Pawn) => 2.5f32,
        None => -1f32,
    }
}

fn score_move(board: &Board, next_move: &Move) -> f32 {
    let piece = board.piece_on(next_move.to);
    score_capture(&piece)
}

fn play_random_move(board: &mut Board) -> Option<(f32, Move)> {
    let Some(next_move) = random::random_move(&board) else {
        info!("Out of moves");
        return None;
    };
    let score = score_move(board, &next_move);
    match board.try_play(next_move) {
        Ok(_) => {
            Some((score, next_move))
        },
        Err(e) => {
            warn!("Failed to make play: {}", e);
            None
        }
    }
}

fn score_move_set(moves: &Vec<(f32, Move)>) -> f32 {
    let mut multiplier = 1f32;
    let mut total_score = 0f32;
    for a_move in moves {
        total_score += a_move.0 * multiplier;
        multiplier = multiplier * FUTURE_DISCOUNT;
    }
    return total_score;
}

fn simulate_random(board: Board, remaining_steps: i32, breadth: i32) -> Vec<(f32, Move)> {
    if remaining_steps <= 0 {
        return vec![];
    }
    let mut best_score = -10000f32;
    let mut best_move_set = vec![];
    for _ in 0..breadth {
        let mut temp_board = board.clone();
        // Play random player move and keep score
        let Some(next_move_and_score) = play_random_move(&mut temp_board) else {
            continue;
        };
        // Play random opponent move
        play_random_move(&mut temp_board);
        // Measure score of opportunities provided by current move
        let additional_moves = simulate_random(
            temp_board.clone(), 
            remaining_steps - 1, 
            breadth - DOWN_MOVES_PER_STEP,
        );
        let mut all_moves = vec![next_move_and_score];
        all_moves.extend(additional_moves);
        let overall_score = score_move_set(&all_moves);
        if overall_score > best_score {
            best_score = overall_score;
            best_move_set = all_moves;
        }
    }
    return best_move_set;
}

impl crate::engine::Engine for MonteCarloAggressiveEngine {
    fn play(&self, board: &Board) -> Option<Move> {
        let next_move_set = simulate_random(board.clone(), STEPS_TO_SIMULATE, MOVES_TO_SIMULATE);
        let total_score = score_move_set(&next_move_set);
        info!("Selecting move set with score {}: {:?}", total_score, next_move_set);
        next_move_set.into_iter().next().map(|x| x.1)
    }
}
