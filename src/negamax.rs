use crate::ev_static::eval_static;
use crate::score::Score;
use crate::settings::Settings;
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
    table: &mut HashMap<TTEntry, TTData>,
    counter: &mut i64,
    options: Settings,
) -> Score {
    *counter += 1;
    let ev = eval_static(*b, options);

    let mut val = ev;
    if !beta.is_greater(val) {
        return val;
    }
    let mut al = alpha.clone();
    let mut be = beta.clone();
    if val.is_greater(alpha) {
        al = val;
    }

    let currEntry = TTEntry::new(*b);
    if let Some(dat) = table.get(&currEntry) {
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

    moves.sort_by_key(|x| {
        let victim_sq = x.get_dest();
        let victim_piece = b.piece_on(victim_sq);
        let victim_value = value(victim_piece, options);

        let attacker_sq = x.get_source();
        let attacker_piece = b.piece_on(attacker_sq);
        let attacker_value = value(attacker_piece, options);

        Reverse(10 * victim_value - attacker_value)
    });

    // PV move first
    if let Some(cached) = table.get(&TTEntry::new(*b)) {
        let pv_move = cached.pvMove;

        if moves.contains(&pv_move) {
            if let Some(p) = moves.iter().position(|m| *m == pv_move) {
                moves.swap(0, p);
            }
        }
    }

    for mov in moves {
        let victim_value = value(b.piece_on(mov.get_dest()), options) as i64;
        let attacker_value = value(b.piece_on(mov.get_source()), options) as i64;
        b.make_move(mov, &mut child);
        let score = quiesce(
            &child,
            be.clone().inverse(),
            al.clone().inverse(),
            table,
            counter,
            options,
        )
        .inverse()
        .step();
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

    table.insert(currEntry, TTData::new(bestMove, val, 0, entry_bound));
    return val;
}

fn is_repetition(history: &Vec<u64>, current: u64) -> bool {
    history.iter().filter(|&&h| h == current).count() >= 2
}

pub fn eval_negamax(
    b: &Board,
    history: &mut Vec<u64>,
    depth: i32,
    alpha: Score,
    beta: Score,
    table: &mut HashMap<TTEntry, TTData>,
    counter: &mut i64,
    options: Settings,
) -> Score {
    *counter += 1;
    let s = b.status();
    if s == BoardStatus::Checkmate {
        let mut val = Score::new();
        val.val = -4294967296;
        return val;
    }
    if s == BoardStatus::Stalemate {
        let mut val = Score::new();
        val.val = 0;
        return val;
    }
    if is_repetition(history, b.get_hash()) {
        let mut val = Score::new();
        val.val = 0;
        return val;
    }
    if depth == 0 {
        return quiesce(b, alpha, beta, table, counter, options);
    }

    // Ram limiter
    let approx_bytes = table.len() * std::mem::size_of::<TTEntry>();
    if approx_bytes >= 157903209 {
        // ~1gb there's a factor off
        let mut to_remove = Vec::new();

        for key in table.keys() {
            // cheap deterministic "random"
            let mut h = std::collections::hash_map::RandomState::new().build_hasher();
            key.hash(&mut h);
            h.write_u64(42);
            let r = h.finish();

            // ~20% eviction rate
            if (r & 7) == 0 {
                to_remove.push(key.clone());
            }
        }

        for k in to_remove {
            table.remove(&k);
            table.shrink_to_fit();
        }
    }
    let mut al = alpha.clone();
    let mut be = beta.clone();

    let currEntry = TTEntry::new(*b);

    if let Some(dat) = table.get(&currEntry) {
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
    let mut moves: Vec<ChessMove> = MoveGen::new_legal(b).collect();
    let movcnt = moves.len();

    let mut val = Score::new();
    val.val = i64::MIN + 1;

    // MVV-LVA
    moves.select_nth_unstable_by_key(options.sortcnt.min(movcnt - 1), |m| {
        let victim_sq = m.get_dest();
        let victim_piece = b.piece_on(victim_sq);
        let victim_value = value(victim_piece, options);

        let attacker_sq = m.get_source();
        let attacker_piece = b.piece_on(attacker_sq);
        let attacker_value = value(attacker_piece, options);

        Reverse(10 * victim_value - attacker_value)
    });

    // PV move first
    if let Some(cached) = table.get(&TTEntry::new(*b)) {
        let pv_move = cached.pvMove;

        if moves.contains(&pv_move) {
            if let Some(p) = moves.iter().position(|m| *m == pv_move) {
                moves.swap(0, p);
            }
        }
    }

    let mut child = b.clone();
    let mut bestMove = ChessMove::default();
    let mut movidx = 0;
    for mov in &mut moves {
        let isCapture = b.piece_on(mov.get_dest()).is_some();
        b.make_move(*mov, &mut child);
        history.push(child.get_hash());
        let isCheck = child.checkers().popcnt() != 0;

        let mut nextdepth = depth - 1;

        let mut ev;

        if movidx == 0 {
            ev = eval_negamax(
                &child,
                history,
                nextdepth,
                be.clone().inverse(),
                al.clone().inverse(),
                table,
                counter,
                options,
            )
            .inverse()
            .step();
        } else {
            // LMR
            //if movidx >= options.lmrMinIdx && depth >= options.lmrMinDepth {
            //    nextdepth -= options.lmrMaxRedux.min(depth - 1);
            //}
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
                null_beta.inverse(),
                al.clone().inverse(),
                table,
                counter,
                options,
            )
            .inverse()
            .step();

            // LMR fail-high
            if ev.is_greater(al.clone()) {
                ev = eval_negamax(
                    &child,
                    history,
                    depth - 1,
                    null_beta.inverse(),
                    al.clone().inverse(),
                    table,
                    counter,
                    options,
                )
                .inverse()
                .step();

                // PVS fail-high
                if ev.is_greater(al.clone()) {
                    ev = eval_negamax(
                        &child,
                        history,
                        depth - 1,
                        be.clone().inverse(),
                        al.clone().inverse(),
                        table,
                        counter,
                        options,
                    )
                    .inverse()
                    .step();
                }
            }
        }
        history.pop();

        if ev.is_greater(val) {
            val = ev.clone();
            bestMove = *mov;
        }

        if val.is_greater(al) {
            al = val.clone();
        }

        if !be.is_greater(al) {
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
    table.insert(currEntry, TTData::new(bestMove, val, depth, entry_bound));
    return val;
}
