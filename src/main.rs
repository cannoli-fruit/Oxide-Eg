mod ev_static;
mod negamax;
mod score;
mod sel_move;
mod settings;
mod timedata;
mod ttentry;

use chess::Board;
use chess::ChessMove;
use chess::Color;
use std::io;
use std::str::FromStr;

use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

fn append_to_file(path: String, contents: String) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(Path::new(&path))?;
    file.write_all(contents.as_bytes())?;
    Ok(())
}

fn main() {
    let mut pos = Board::default();
    let stdin = io::stdin();
    let mut history: Vec<u64> = Vec::new();

    let mut table = HashMap::new();
    let mut options = settings::Settings {
        knightValue: 300,
        bishopValue: 300,
        rookValue: 500,
        queenValue: 900,
        sortcnt: 6,
        lmrMinDepth: 2,
        aspirationWindowSize: 35,
    };

    loop {
        let mut buf = String::new();
        stdin.read_line(&mut buf);
        //append_to_file("/tmp/oxeg_in_log".to_string(), buf.clone());
        buf.pop();
        if buf == "uci" {
            println!("id name The Oxidized Engine");
            println!("id author codycodes1234");
            println!("option name knightValue type spin default 300 min 0 max 1500");
            println!("option name bishopValue type spin default 300 min 0 max 1500");
            println!("option name rookValue type spin default 500 min 0 max 1500");
            println!("option name queenValue type spin default 900 min 0 max 1500");
            println!("option name lmrMinDepth type spin default 2 min 1 max 24");
            println!("option name sortcnt type spin default 6 min 1 max 24");
            println!("option name aspirationWindowSize type spin default 35 min 1 max 500");
            println!("uciok");
        }
        if buf == "isready" {
            println!("readyok");
        }
        if buf == "ucinewgame" {
            pos = Board::default();
        }
        if buf.starts_with("position startpos") {
            if buf == "position startpos" {
                pos = Board::default();
            } else {
                pos = Board::default();
                let movstrs = buf[24..].split(" ");
                for mov in movstrs {
                    let mv = ChessMove::from_str(mov).expect("Valid move");
                    let mut bbuf = Board::default();
                    pos.make_move(mv, &mut bbuf);
                    pos = bbuf;
                }
            }
        }
        if buf.starts_with("position fen") {
            let idx = buf.find("moves");
            match idx {
                Some(x) => {
                    let fen = &buf[13..(x - 1)];
                    pos = Board::from_str(fen).expect("Valid fen");
                    let movstrs = buf[(x + 6)..].split(" ");
                    for mov in movstrs {
                        let mv = ChessMove::from_str(mov).expect("Valid move");
                        let mut bbuf = Board::default();
                        pos.make_move(mv, &mut bbuf);
                        pos = bbuf;
                    }
                }
                None => {
                    let fen = &buf[13..];
                    pos = Board::from_str(&fen).expect("Valid fen");
                }
            }
        }
        if buf.starts_with("go infinite") {
            let mut counter = 0;
            let mov =
                sel_move::select_move(&pos, &mut history, &mut table, 150, &mut counter, options);
            println!("info time 150");
            println!("info nodes {}", counter);
            println!("bestmove {}", mov,);
        }
        if buf.starts_with("go movetime") {
            let time: u64 = buf[12..].parse().unwrap();
            let mut counter = 0;
            let mov = sel_move::select_move(
                &pos,
                &mut history,
                &mut table,
                (time >> 1) + (time >> 2),
                &mut counter,
                options,
            );
            println!("info nodes {}", counter);
            println!("bestmove {}", mov,);
        }
        if buf.starts_with("go wtime ") {
            let parts: Vec<&str> = buf.split(' ').collect();

            let wtime: u64 = parts[2].parse().unwrap();
            let btime: u64 = parts[4].parse().unwrap();
            let winc: u64 = parts[6].parse().unwrap();
            let binc: u64 = parts[8].parse().unwrap();

            let movetime = if pos.side_to_move() == Color::White {
                wtime / 20 + winc / 2
            } else {
                btime / 20 + binc / 2
            };

            let mut counter = 0;
            let mov = sel_move::select_move(
                &pos,
                &mut history,
                &mut table,
                movetime,
                &mut counter,
                options,
            );
            println!("info time {}", movetime);
            println!("info nodes {}", counter);
            println!("bestmove {}", mov,);
        }
        if buf.starts_with("setoption name ") {
            let rest = &buf[15..];

            if let Some(idx) = rest.find(" value ") {
                let name = &rest[..idx];
                let value = &rest[idx + 7..];

                match name {
                    "knightValue" => options.knightValue = value.parse().unwrap(),
                    "bishopValue" => options.bishopValue = value.parse().unwrap(),
                    "rookValue" => options.rookValue = value.parse().unwrap(),
                    "queenValue" => options.queenValue = value.parse().unwrap(),
                    "sortcnt" => options.sortcnt = value.parse().unwrap(),
                    "lmrMinDepth" => options.lmrMinDepth = value.parse().unwrap(),
                    "aspirationWindowSize" => options.aspirationWindowSize = value.parse().unwrap(),

                    _ => {}
                }
            }
        }
        if buf.starts_with("quit") {
            std::process::exit(0)
        }
    }
}
