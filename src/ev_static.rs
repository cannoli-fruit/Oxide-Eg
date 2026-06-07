use crate::pst::*;
use crate::score::Score;
use crate::settings::Settings;

use chess::{Board, Color, MoveGen, Piece};
use std::ops::BitAnd;

pub fn eval_static(b: Board, options: Settings) -> Score {
    let white = b.color_combined(Color::White);
    let black = b.color_combined(Color::Black);

    let pawns = b.pieces(Piece::Pawn);
    let knights = b.pieces(Piece::Knight);
    let bishops = b.pieces(Piece::Bishop);
    let rooks = b.pieces(Piece::Rook);
    let queens = b.pieces(Piece::Queen);

    let white_pawns = pawns.bitand(white).popcnt() as i32;
    let white_knights = knights.bitand(white).popcnt() as i32;
    let white_bishops = bishops.bitand(white).popcnt() as i32;
    let white_rooks = rooks.bitand(white).popcnt() as i32;
    let white_queens = queens.bitand(white).popcnt() as i32;

    let black_pawns = pawns.bitand(black).popcnt() as i32;
    let black_knights = knights.bitand(black).popcnt() as i32;
    let black_bishops = bishops.bitand(black).popcnt() as i32;
    let black_rooks = rooks.bitand(black).popcnt() as i32;
    let black_queens = queens.bitand(black).popcnt() as i32;

    let white_king = b.king_square(Color::White).to_int();
    let black_king = b.king_square(Color::Black).to_int();

    let white_rank = (white_king / 8) as i32;
    let white_file = (white_king % 8) as i32;

    let black_rank = (black_king / 8) as i32;
    let black_file = (black_king % 8) as i32;

    let white_dr = (white_rank - 3).abs().min((white_rank - 4).abs());
    let white_df = (white_file - 3).abs().min((white_file - 4).abs());
    let white_dist = white_dr + white_df;

    let black_dr = (black_rank - 3).abs().min((black_rank - 4).abs());
    let black_df = (black_file - 3).abs().min((black_file - 4).abs());
    let black_dist = black_dr + black_df;

    let mut movcount = MoveGen::new_legal(&b).len() as i32;
    let child = b.null_move();
    let mut movcount2 = 0;
    if let Some(b2) = child {
        movcount2 = MoveGen::new_legal(&b2).len() as i32;
    }

    if b.side_to_move() == Color::Black {
        movcount *= -1;
        movcount2 *= -1;
    }

    let v: i64 = (100 * (white_pawns - black_pawns)
        + options.knightValue * (white_knights - black_knights)
        + options.bishopValue * (white_bishops - black_bishops)
        + options.rookValue * (white_rooks - black_rooks)
        + options.queenValue * (white_queens - black_queens)
        + options.kingPosValue * (black_dist - white_dist)
        + options.mobilityValue * (movcount - movcount2))
        .into();
    // let v = (movcount - movcount2) as i64;
    let mut sc = Score::new();
    sc.val = v;

    if b.side_to_move() == Color::Black {
        sc.val *= -1;
    }

    return sc;
}
