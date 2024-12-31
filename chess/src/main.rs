use std::io::{self, BufRead};
use cozy_chess::{Board, File, Move, Rank, Square};
use log::{debug, info, warn};
use rand::Rng;
use simplelog::{CombinedLogger, WriteLogger};
use vampirc_uci::{parse_one, UciMessage, UciMove, UciSquare};

const MOVES_TO_SIMULATE: i32 = 100;
const STEPS_TO_SIMULATE: i32 = 4;
const DOWN_STEPS_PER_LAYER: i32 = 20;

#[derive(PartialEq, Debug)]
enum Engine {
    MonteCarloAggressive,
    Random
}

fn rand_num(max: usize) -> usize {
    rand::thread_rng().gen_range(0..max)
}

fn uci_square_of_cozy_square(cozy: &Square) -> UciSquare {
    let file = match cozy.file() {
        cozy_chess::File::A => 'a',
        cozy_chess::File::B => 'b',
        cozy_chess::File::C => 'c',
        cozy_chess::File::D => 'd',
        cozy_chess::File::E => 'e',
        cozy_chess::File::F => 'f',
        cozy_chess::File::G => 'g',
        cozy_chess::File::H => 'h',
    };
    let rank = match cozy.rank() {
        cozy_chess::Rank::First => 1,
        cozy_chess::Rank::Second => 2,
        cozy_chess::Rank::Third => 3,
        cozy_chess::Rank::Fourth => 4,
        cozy_chess::Rank::Fifth => 5,
        cozy_chess::Rank::Sixth => 6,
        cozy_chess::Rank::Seventh => 7,
        cozy_chess::Rank::Eighth => 8
    };
    UciSquare::from(file, rank)
}

fn uci_move_of_cozy_move(cozy: &Move, is_promotion: bool) -> UciMove {
    let mut new_move = UciMove::from_to(
        uci_square_of_cozy_square(&cozy.from),
        uci_square_of_cozy_square(&cozy.to)
    );
    new_move.promotion = if is_promotion {
        Some(vampirc_uci::UciPiece::Queen)
    } else {
        None
    };
    return new_move;
}

fn cozy_square_of_uci_square(cozy: &UciSquare) -> Square {
    let file = match cozy.file {
        'a' => File::A,
        'b' => File::B,
        'c' => File::C,
        'd' => File::D,
        'e' => File::E,
        'f' => File::F,
        'g' => File::G,
        'h' => File::H,
        f => {
            warn!("Invalid file {}", f);
            panic!("cry");
        }
    };
    let rank = match cozy.rank {
        1 => Rank::First,
        2 => Rank::Second,
        3 => Rank::Third,
        4 => Rank::Fourth,
        5 => Rank::Fifth,
        6 => Rank::Sixth,
        7 => Rank::Seventh,
        8 => Rank::Eighth,
        r => {
            warn!("Invalid rank {}", r);
            panic!("cry!");
        }
    };
    Square::new(file, rank)
}

fn cozy_move_of_uci_move(uci_move: &UciMove) -> Move {
    Move {
        from: cozy_square_of_uci_square(&uci_move.from),
        to: cozy_square_of_uci_square(&uci_move.to),
        promotion: None
    }
}

fn next_moves(board: &Board) -> Vec<Move> {
    let mut move_list = Vec::new();
    board.generate_moves(|moves| {
        move_list.extend(moves);
        false
    });
    return move_list;
}

fn simulate_random(board: Board, remaining_steps: i32, breadth: i32, current_score: f32) -> (f32, Vec<Move>) {
    let mut board = board;
    if remaining_steps <= 0 {
        return (current_score, vec![]);
    }
    let move_list = next_moves(&board);
    let Some(next_move) = move_list.get(rand_num(move_list.len())) else {
        return (current_score, vec![]);
    };
    board.play_unchecked(*next_move);
    board.null_move();
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
    last_move_and_best_moves.push(*next_move);
    last_move_and_best_moves.extend(best_moves.into_iter());
    return (best_score, last_move_and_best_moves);
}

fn main() {
    CombinedLogger::init(vec![
        WriteLogger::new(
            log::LevelFilter::Debug, 
            simplelog::Config::default(), 
            std::fs::File::create("mylog.log").unwrap()
        )
    ]).unwrap();
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
    info!("Started {:?}", engine_mode);
    let mut board = Board::default();
    println!("{}", UciMessage::Uci);
    info!("Printing UCI message");
    for line in stdin.lines() {
        let msg: UciMessage = parse_one(&line.unwrap());
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
                    let best_move_set = simulate_random(board.clone(), STEPS_TO_SIMULATE, MOVES_TO_SIMULATE, 0f32);
                    best_move_set.1.into_iter().next().unwrap()
                } else if engine_mode == Engine::Random {
                    let move_list = next_moves(&board);
                    move_list.get(rand_num(move_list.len())).unwrap().clone()
                } else {
                    warn!("Unimplemented engine");
                    panic!("Unimplemented engine");
                };
                debug!("Got best move {:?}", first_move);
                board.play_unchecked(first_move);
                let is_promotion = first_move.promotion.is_some();
                let best_move = uci_move_of_cozy_move(&first_move, is_promotion);
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
                board.play_unchecked(cozy_move_of_uci_move(&next_move));
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
