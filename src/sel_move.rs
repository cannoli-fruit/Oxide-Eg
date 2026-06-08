use chess::Board;
use chess::ChessMove;
use chess::Color;
use chess::MoveGen;

use crate::negamax::eval_negamax;
use crate::score::Score;
use crate::settings::Settings;
use crate::ttentry::*;

use std::collections::HashMap;
use std::time::Instant;

fn col_val(c: Color) -> i64 {
    if c == Color::White {
        -1
    } else {
        1
    }
}

pub fn select_move(
    b: &Board,
    history: &mut Vec<u64>,
    table: &mut HashMap<TTEntry, TTData>,
    max_time: u64,
    counter: &mut i64,
    options: Settings,
) -> ChessMove {
    let start = Instant::now();
    let max_depth = 12usize;

    let mut best_move = ChessMove::default();
    let mut best_score = Score::new();
    best_score = Score { val: i64::MIN + 1 };

    // collect legal moves once and keep the original order
    let moves: Vec<ChessMove> = MoveGen::new_legal(b).collect();
    let mut highest_depth = 0;

    for depth in 1..=max_depth {
        if depth % 2 == 1 && depth != max_depth {
            continue;
        }
        let mut local_best_move = ChessMove::default();
        let mut local_best_score = Score::new();
        local_best_score.val = i64::MAX - 1;

        let mut beta = Score::new();
        beta.val = i64::MAX - 1;

        let mut alpha = Score::new();
        alpha.val = i64::MIN + 1;

        if depth != 1 {
            // Aspiration Window
            if !best_score.isMate() {
                let window = options.aspirationWindowSize;
                beta.val = best_score.val + window;
                alpha.val = best_score.val - window;
            }
        }

        for mov in &moves {
            let child = b.make_move_new(*mov);
            let mut ev = eval_negamax(
                &child,
                history,
                depth as i32,
                alpha,
                beta,
                table,
                counter,
                options,
            );
            if start.elapsed().as_millis() > max_time.into() {
                break;
            }
            if !ev.is_greater(alpha) || !beta.is_greater(ev) {
                //Aspiration failure
                beta.val = 4294967296;
                alpha.val = -4294967296;
                ev = eval_negamax(
                    &child,
                    history,
                    depth as i32,
                    alpha,
                    beta,
                    table,
                    counter,
                    options,
                );
            }
            if local_best_score.is_greater(ev) {
                local_best_score = ev;
                local_best_move = *mov;
            }

            if start.elapsed().as_millis() > max_time.into() {
                break;
            }
        }
        if start.elapsed().as_millis() > max_time.into() {
            break;
        }

        // use the result from the completed depth
        best_move = local_best_move;
        best_score = local_best_score;
        highest_depth = depth;
    }
    println!("info depth {}", highest_depth);
    if best_score.isMate() {
        println!(
            "info score mate {}",
            (best_score.mateDist() as i64) * col_val(b.side_to_move())
        );
    } else {
        println!(
            "info score cp {}",
            best_score.val * col_val(b.side_to_move())
        );
    }

    best_move
}
