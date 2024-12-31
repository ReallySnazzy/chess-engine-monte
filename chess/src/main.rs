use std::io::{self, BufRead};
use cozy_chess::{Board, Move};
use log::{debug, error, info, warn};
use simplelog::{CombinedLogger, WriteLogger};
use vampirc_uci::{parse_one, UciMessage};

mod random;
mod cozy_helper;

const MOVES_TO_SIMULATE: i32 = 15;
const STEPS_TO_SIMULATE: i32 = 6;
const DOWN_STEPS_PER_LAYER: i32 = 2;

#[derive(PartialEq, Debug)]
enum Engine {
    MonteCarloAggressive,
    Random
}

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

fn main() {
    let mut stdin = io::stdin().lock();
    let mut mode = String::new();
    stdin.read_line(&mut mode).unwrap();
    let engine_mode = match mode.trim() {
        "monteattack" => Engine::MonteCarloAggressive,
        "random" => Engine::Random,
        _ => {
            warn!("Unknown engine selection");
            panic!("Unknown engine selection");
        }
    };
    if engine_mode == Engine::MonteCarloAggressive {
        CombinedLogger::init(vec![
            WriteLogger::new(
                log::LevelFilter::Debug, 
                simplelog::Config::default(), 
                std::fs::File::create("mylog.log").unwrap()
            )
        ]).unwrap();
    } else {
        CombinedLogger::init(vec![]).unwrap();
    }
    info!("Started {:?}", engine_mode);
    let mut board = Board::default();
    println!("{}", UciMessage::Uci);
    info!("Printing UCI message");
    for line in stdin.lines() {
        let msg: UciMessage = match line {
            Ok(line) => parse_one(&line),
            _ => {
                warn!("Failed to parse line");
                panic!("Failed to parse line");
            }
        };
        match msg {
            UciMessage::Uci => {
                println!("{}", UciMessage::Id { 
                    name: Some("Coolbot".to_owned()),
                    author: Some("Snazzy".to_owned())
                });
                info!("Responding UciOk");
                println!("{}", UciMessage::UciOk);
            },
            UciMessage::UciNewGame => {
                board = Board::default();
            },
            UciMessage::IsReady => {
                info!("Responding ReadyOk");
                println!("{}", UciMessage::ReadyOk);
            },
            UciMessage::Go {time_control: _, search_control: _} => {
                info!("In Go");
                let first_move = if engine_mode == Engine::MonteCarloAggressive {
                    info!("MontoCarloSimulateAggressive");
                    match std::panic::catch_unwind(|| {
                        let Some(best_move_set) = simulate_random(board.clone(), STEPS_TO_SIMULATE, MOVES_TO_SIMULATE, 0f32)
                            .1
                            .into_iter()
                            .next() 
                        else {
                            warn!("No best move found");
                            panic!("No best move found");
                        };
                        best_move_set
                    }) {
                        Ok(m) => m,
                        Err(e) => {
                            let message = if let Some(s) = e.downcast_ref::<String>() {
                                s.clone()
                            } else if let Some(s) = e.downcast_ref::<&str>() {
                                (*s).to_owned()
                            } else {
                                format!("Unknown error: {:?}", e)
                            };
                            error!("Simulation paniced: {}", message);
                            panic!("Simulation paniced: {}", message);
                        }
                    }
                } else if engine_mode == Engine::Random {
                    let Some(next_move) = random::random_move(&board) else {
                        warn!("No next move found");
                        panic!("No next move found");
                    };
                    next_move
                } else {
                    warn!("Unimplemented engine");
                    panic!("Unimplemented engine");
                };
                debug!("Got best move {:?}", first_move);
                board.play_unchecked(first_move);
                let is_promotion = first_move.promotion.is_some();
                let best_move = cozy_helper::uci_move_of_cozy_move(&first_move, is_promotion);
                info!("Making move {}", best_move);
                println!("{}", UciMessage::BestMove { 
                    best_move, 
                    ponder: None
                });
            },
            UciMessage::Position { startpos: _, fen: _, moves } => {
                debug!("Got moves {:?}", moves);
                let Some(next_move) = moves.last() else {
                    continue;
                };
                info!("Applying move {}", next_move);
                board.play_unchecked(cozy_helper::cozy_move_of_uci_move(&next_move));
            },
            UciMessage::Quit => {
                info!("Received quit command");
                break;
            },
            message => {
                warn!("Unimplemented message! {}", message);
            }
        }
    }
}
