use std::io::{self, BufRead, StdinLock};
use cozy_chess::Board;
use engine::Engine;
use log::{debug, info, warn};
use simplelog::{CombinedLogger, WriteLogger};
use vampirc_uci::{parse_one, UciMessage};

mod random;
mod cozy_helper;
mod monte_carlo_aggressive_engine;
mod random_engine;
mod engine;

fn initialize_engine(stdin: &mut StdinLock<'static>) -> Box<dyn Engine> {
    let mut mode = String::new();
    stdin.read_line(&mut mode).unwrap();
    match mode.trim() {
        "monteattack" => {
            CombinedLogger::init(vec![
                WriteLogger::new(
                    log::LevelFilter::Debug, 
                    simplelog::Config::default(), 
                    std::fs::File::create("mylog.log").unwrap()
                )
            ]).unwrap();
            Box::new(monte_carlo_aggressive_engine::MonteCarloAggressiveEngine {})
        },
        "random" => {
            CombinedLogger::init(vec![]).unwrap();
            Box::new(random_engine::RandomEngine {})
        },
        _ => {
            warn!("Unknown engine selection");
            panic!("Unknown engine selection");
        }
    }
}

fn startup_uci() {
    println!("{}", UciMessage::Uci);
    info!("Printing UCI message");
}

fn handle_uci_command(cmd: UciMessage, engine: &Box<dyn Engine>, board: &mut Board) {
    match cmd {
        UciMessage::Uci => {
            println!("{}", UciMessage::Id { 
                name: Some("Coolbot".to_owned()),
                author: Some("Snazzy".to_owned())
            });
            info!("Responding UciOk");
            println!("{}", UciMessage::UciOk);
        },
        UciMessage::UciNewGame => {
            *board = Board::default();
        },
        UciMessage::IsReady => {
            info!("Responding ReadyOk");
            println!("{}", UciMessage::ReadyOk);
        },
        UciMessage::Go {time_control: _, search_control: _} => {
            info!("In Go");
            let next_move_optional = engine.play(&board);
            let Some(next_move) = next_move_optional else {
                info!("No best move found");
                return;
            };
            board.play_unchecked(next_move);
            let is_promotion = next_move.promotion.is_some();
            let best_move = cozy_helper::uci_move_of_cozy_move(&next_move, is_promotion);
            println!("{}", UciMessage::BestMove { 
                best_move, 
                ponder: None
            });
        },
        UciMessage::Position { startpos: _, fen: _, moves } => {
            debug!("Got moves {:?}", moves);
            let Some(next_move) = moves.last() else {
                return;
            };
            info!("Applying move {}", next_move);
            board.play_unchecked(cozy_helper::cozy_move_of_uci_move(&next_move));
        },
        UciMessage::Quit => {
            info!("Received quit command");
            panic!("Terminate");
        },
        message => {
            warn!("Unimplemented message! {}", message);
        }
    }
}

fn main() {
    let mut stdin = io::stdin().lock();
    let engine: Box<dyn Engine> = initialize_engine(&mut stdin);
    startup_uci();
    let mut board = Board::default();
    for line in stdin.lines() {
        let msg: UciMessage = match line {
            Ok(line) => parse_one(&line),
            _ => {
                warn!("Failed to parse line");
                panic!("Failed to parse line");
            }
        };
        handle_uci_command(msg, &engine, &mut board);
    }
}
