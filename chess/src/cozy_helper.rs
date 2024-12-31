use cozy_chess::{File, Move, Rank, Square, Board};
use log::warn;
use vampirc_uci::{UciMove, UciSquare};

pub fn available_moves(board: &Board) -> Vec<Move> {
    let mut move_list = Vec::new();
    board.generate_moves(|moves| {
        move_list.extend(moves);
        false
    });
    return move_list;
}

pub fn uci_square_of_cozy_square(cozy: &Square) -> UciSquare {
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

pub fn uci_move_of_cozy_move(cozy: &Move, is_promotion: bool) -> UciMove {
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

pub fn cozy_square_of_uci_square(cozy: &UciSquare) -> Square {
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
            panic!("Invalid file {}", f);
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
            panic!("Invalid rank {}", r);
        }
    };
    Square::new(file, rank)
}

pub fn cozy_move_of_uci_move(uci_move: &UciMove) -> Move {
    Move {
        from: cozy_square_of_uci_square(&uci_move.from),
        to: cozy_square_of_uci_square(&uci_move.to),
        promotion: None
    }
}


