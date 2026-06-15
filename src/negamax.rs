use crate::ev_static::eval_static;
use crate::score::Score;
use crate::settings::Settings;
use crate::timedata::Timer;
use crate::ttentry::*;

use chess::Board;
use chess::BoardStatus;
use chess::ChessMove;
use chess::Color;
use chess::MoveGen;
use chess::Piece;

use std::cmp::Reverse;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::Not;

pub static mut null_attempts: i32 = 0;
pub static mut null_cutoffs: i32 = 0;

pub fn get_null_attempts() -> i32 {
    unsafe { null_attempts }
}
pub fn get_null_cutoffs() -> i32 {
    unsafe { null_cutoffs }
}

fn value(p: Option<Piece>, options: Settings) -> i32 {
    match p {
        Some(Piece::Pawn) => 100,
        Some(Piece::Knight) => options.knightValue.try_into().unwrap(),
        Some(Piece::Bishop) => options.bishopValue.try_into().unwrap(),
        Some(Piece::Rook) => options.rookValue.try_into().unwrap(),
        Some(Piece::Queen) => options.queenValue.try_into().unwrap(),
        Some(Piece::King) => 99999, // why is this even here
        None => 0,
    }
}

fn int_log2(x: i32) -> i32 {
    if x == 0 {
        -1
    } else {
        (31 - x.leading_zeros()).try_into().unwrap()
    }
}

fn quiesce(
    b: &Board,
    alpha: Score,
    beta: Score,
    timer: &mut Timer,
    table: &mut Box<[TTData]>,
    counter: &mut i64,
    options: Settings,
) -> Score {
    *counter += 1;
    let ev = eval_static(*b, options);

    let scorez = Score::new();

    let mut val = ev;
    if !beta.is_greater(val) {
        return val;
    }
    let mut al = alpha.clone();
    let mut be = beta.clone();
    if val.is_greater(alpha) {
        al = val;
    }

    let currEntry = b.get_hash();
    if table[(currEntry & 33554431) as usize].hash == currEntry {
        let dat = table[(currEntry & 33554431) as usize];
        match dat.bound {
            TTData_BoundType::EXACT => {
                return dat.eval;
            }
            TTData_BoundType::LOWER => {
                if dat.eval.is_greater(al) {
                    al = dat.eval.clone();
                }
            }
            TTData_BoundType::UPPER => {
                if be.is_greater(dat.eval.clone()) {
                    be = dat.eval.clone();
                }
            }
        }

        if !be.is_greater(al.clone()) {
            return al;
        }
    }

    let mut iter = MoveGen::new_legal(b);
    let targets = b.color_combined(!b.side_to_move());
    iter.set_iterator_mask(*targets);
    let mut moves: Vec<ChessMove> = iter.collect();

    let mut child = b.clone();
    let mut bestMove = ChessMove::default();

    // MVV-LVA
    moves.sort_by_key(|m| {
        let victim_sq = m.get_dest();
        let victim_piece = b.piece_on(victim_sq);
        let victim_value = value(victim_piece, options);

        let attacker_sq = m.get_source();
        let attacker_piece = b.piece_on(attacker_sq);
        let attacker_value = value(attacker_piece, options);

        Reverse(10 * victim_value - attacker_value)
    });

    for mov in moves {
        let victim_value = value(b.piece_on(mov.get_dest()), options) as i64;
        let attacker_value = value(b.piece_on(mov.get_source()), options) as i64;
        b.make_move(mov, &mut child);
        let score = quiesce(
            &child,
            be.clone().inverse(),
            al.clone().inverse(),
            timer,
            table,
            counter,
            options,
        )
        .inverse()
        .step();
        if timer.stop {
            return scorez;
        }
        if !be.is_greater(score) {
            return beta;
        }
        if score.is_greater(val) {
            val = score;
            bestMove = mov;
        }
        if score.is_greater(al) {
            al = score;
        }
    }

    let entry_bound = if !be.is_greater(al) {
        TTData_BoundType::LOWER
    } else if !val.is_greater(alpha) {
        TTData_BoundType::UPPER
    } else if val.is_greater(be) {
        TTData_BoundType::LOWER
    } else {
        TTData_BoundType::EXACT
    };

    table[(currEntry & 33554431) as usize] =
        TTData::new(b.get_hash(), bestMove, val, 0, entry_bound);
    return val;
}

fn is_repetition(history: &Vec<u64>, current: u64) -> bool {
    history.iter().filter(|&&h| h == current).count() > 2
}

pub fn eval_negamax(
    b: &Board,
    history: &mut Vec<u64>,
    depth: i32,
    ply: usize,
    alpha: Score,
    beta: Score,
    allow_null: bool,
    killer: &mut [ChessMove; 256],
    timer: &mut Timer,
    table: &mut Box<[TTData]>,
    counter: &mut i64,
    options: Settings,
) -> Score {
    *counter += 1;
    let scorez = Score::new();
    history.push(b.get_hash());
    let s = b.status();
    if s == BoardStatus::Checkmate {
        let mut val = Score::new();
        val.val = -4294967296;
        return val;
    }
    if s == BoardStatus::Stalemate {
        return scorez;
    }
    if is_repetition(history, b.get_hash()) {
        history.pop();
        return scorez;
    }
    if *counter & 4095 == 0 {
        timer.recalc();
    }
    if depth <= 0 {
        let qev = quiesce(b, alpha, beta, timer, table, counter, options);
        history.pop();
        return qev;
    }

    let mut al = alpha.clone();
    let mut be = beta.clone();

    let currEntry = b.get_hash();

    if table[(currEntry & 33554431) as usize].hash == currEntry {
        let dat = table[(currEntry & 33554431) as usize];
        if dat.depth >= depth {
            match dat.bound {
                TTData_BoundType::EXACT => {
                    return dat.eval;
                }
                TTData_BoundType::LOWER => {
                    if dat.eval.is_greater(al) {
                        al = dat.eval.clone();
                    }
                }
                TTData_BoundType::UPPER => {
                    if be.is_greater(dat.eval.clone()) {
                        be = dat.eval.clone();
                    }
                }
            }

            if !be.is_greater(al.clone()) {
                return al;
            }
        }
    }

    let ev_static = eval_static(*b, options);

    let stm_pieces = b.color_combined(b.side_to_move());

    let stm_material = b
        .pieces(Piece::Knight)
        .bitor(b.pieces(Piece::Bishop))
        .bitor(b.pieces(Piece::Rook))
        .bitor(b.pieces(Piece::Queen))
        .bitand(stm_pieces);

    // NMP
    if depth >= 3
        && b.checkers().popcnt() == 0
        && allow_null
        && stm_material.popcnt() != 0
        && ev_static.val >= beta.val - 0
    {
        unsafe {
            null_attempts += 1;
        }
        let c = b.null_move().unwrap();
        let mut null_history = history.clone();

        //let mut R = 3 + (depth >> 1);
        let mut R = 3;

        R = R.min(depth - 2);
        let mut null_beta = be.clone();
        null_beta.val -= 1;

        let score = eval_negamax(
            &c,
            &mut null_history,
            depth - 1 - R,
            ply + 1,
            be.clone().inverse(),
            null_beta.inverse(),
            false,
            killer,
            timer,
            table,
            counter,
            options,
        )
        .inverse()
        .step();

        if score.val >= be.val {
            unsafe {
                null_cutoffs += 1;
            }
            return be;
        }
    }

    let mut moves: Vec<ChessMove> = MoveGen::new_legal(b).collect();
    let movcnt = moves.len();

    let mut val = Score::new();
    val.val = i64::MIN + 1;

    moves.sort_by_key(|x| {
        let mut is_pv = false;

        if table[(currEntry & 33554431) as usize].hash == currEntry {
            let pv_move = table[(currEntry & 33554431) as usize].pvMove;
            if *x == pv_move {
                return Reverse(1_000_000);
            }
        }

        let victim_sq = x.get_dest();
        let victim_piece = b.piece_on(victim_sq);
        let victim_value = value(victim_piece, options);

        if victim_value != 0 {
            let attacker_sq = x.get_source();
            let attacker_piece = b.piece_on(attacker_sq);
            let attacker_value = value(attacker_piece, options);

            return Reverse(10 * victim_value - attacker_value);
        } else {
            if killer[ply] == *x {
                return Reverse(5000);
            }
        }
        return Reverse(0);
    });

    let mut child = b.clone();
    let mut bestMove = ChessMove::default();
    let mut movidx = 0;
    for mov in &mut moves {
        let isCapture = b.piece_on(mov.get_dest()).is_some();
        b.make_move(*mov, &mut child);
        let isCheck = child.checkers().popcnt() != 0;

        let mut nextdepth = depth - 1;

        let mut ev;

        if movidx == 0 {
            ev = eval_negamax(
                &child,
                history,
                nextdepth,
                ply + 1,
                be.clone().inverse(),
                al.clone().inverse(),
                false,
                killer,
                timer,
                table,
                counter,
                options,
            )
            .inverse()
            .step();

            if timer.stop {
                return scorez;
            }
        } else {
            // LMR
            if depth >= options.lmrMinDepth {
                nextdepth -= 1 + int_log2(depth) + (int_log2(movidx) >> 2);
                nextdepth = nextdepth.max(0);
            }

            // null window around alpha
            let mut null_beta = al.clone();
            null_beta.val += 1;

            ev = eval_negamax(
                &child,
                history,
                nextdepth,
                ply + 1,
                null_beta.inverse(),
                al.clone().inverse(),
                true,
                killer,
                timer,
                table,
                counter,
                options,
            )
            .inverse()
            .step();
            if timer.stop {
                return scorez;
            }

            // LMR fail-high
            if ev.is_greater(al.clone()) {
                ev = eval_negamax(
                    &child,
                    history,
                    depth - 1,
                    ply + 1,
                    null_beta.inverse(),
                    al.clone().inverse(),
                    true,
                    killer,
                    timer,
                    table,
                    counter,
                    options,
                )
                .inverse()
                .step();
                if timer.stop {
                    return scorez;
                }

                // PVS fail-high
                if ev.is_greater(al.clone()) {
                    ev = eval_negamax(
                        &child,
                        history,
                        depth - 1,
                        ply + 1,
                        be.clone().inverse(),
                        al.clone().inverse(),
                        true,
                        killer,
                        timer,
                        table,
                        counter,
                        options,
                    )
                    .inverse()
                    .step();
                    if timer.stop {
                        return scorez;
                    }
                }
            }
        }

        if ev.is_greater(val) {
            val = ev.clone();
            bestMove = *mov;
        }

        if val.is_greater(al) {
            al = val.clone();
        }

        if !be.is_greater(al) {
            killer[ply] = *mov;
            break;
        }
        movidx += 1;
    }
    let entry_bound = if !val.is_greater(alpha) {
        TTData_BoundType::UPPER
    } else if !beta.is_greater(val) {
        TTData_BoundType::LOWER
    } else {
        TTData_BoundType::EXACT
    };
    table[(currEntry & 33554431) as usize] =
        TTData::new(b.get_hash(), bestMove, val, depth, entry_bound);
    history.pop();
    return val;
}
