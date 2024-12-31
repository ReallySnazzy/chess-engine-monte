use std::{io::{self, BufRead}, time::Duration};
use cozy_chess::{Board, File, Move, Rank, Square};
use log::{debug, info, warn};
use rand::Rng;
use simplelog::{CombinedLogger, WriteLogger};
use vampirc_uci::{parse_one, UciMessage, UciMove, UciSquare};

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
            println!("Invalid file {}", f);
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
            println!("Invalid rank {}", r);
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

fn main() {
    CombinedLogger::init(vec![
        WriteLogger::new(
            log::LevelFilter::Debug, 
            simplelog::Config::default(), 
            std::fs::File::create("mylog.log").unwrap()
        )
    ]).unwrap();
    info!("Started");
    let mut board = Board::default();
    println!("{}", UciMessage::Uci);
    info!("Printing UCI message");
    for line in io::stdin().lock().lines() {
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
            UciMessage::IsReady => {
                info!("Responding ReadyOk");
                println!("{}", UciMessage::ReadyOk);
            },
            UciMessage::Go {time_control: _, search_control: _} => {
                info!("In Go");
                let mut move_list = Vec::new();
                board.generate_moves(|moves| {
                    move_list.extend(moves);
                    false
                });
                let rand_indx = rand_num(move_list.len());
                debug!("Have move options: {:?}, selected {} of {}", move_list, rand_indx, move_list.len());
                let Some(first_move) = move_list.get(rand_indx) else {
                    warn!("No move found");
                    continue;
                };
                board.play_unchecked(*first_move);
                let is_promotion = first_move.promotion.is_some();
                let best_move = uci_move_of_cozy_move(first_move, is_promotion);
                info!("Making move {}", best_move);
                std::thread::sleep(Duration::from_millis(500));
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
            message => {
                warn!("Unimplemented message! {}", message);
            }
        }
    }
}
