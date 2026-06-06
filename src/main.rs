mod ev_static;
mod negamax;
mod pst;
mod score;
mod sel_move;
mod settings;
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
        mobilityValue: 5,
        kingPosValue: 6,
        nmpDepthMin: 4,
        nmpStaticSafety: 200,
        nmpMinPieces: 12,
        lmrMinIdx: 3,
        lmrMinDepth: 2,
        lmrMaxRedux: 2,
        razoringMargin: 400,
        deltaStaticSafety: 300,
        quietFutilitySafety: 200,
        futilitySafety: 200,
        futilityDepth: 2,
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
            println!("option name mobilityValue type spin default 5 min -50 max 50");
            println!("option name kingPosValue type spin default 6 min -50 max 50");
            println!("option name nmpDepthMin type spin default 4 min 3 max 6");
            println!("option name nmpStaticSafety type spin default 200 min 0 max 1500");
            println!("option name nmpMinPieces type spin default 12 min 0 max 32");
            println!("option name lmrMinIdx type spin default 5 min 2 max 30");
            println!("option name lmrMinDepth type spin default 2 min 1 max 6");
            println!("option name lmrMaxRedux type spin default 2 min 1 max 6");
            println!("option name razoringMargin type spin default 400 min 0 max 1500");
            println!("option name deltaStaticSafety type spin default 300 min 0 max 1500");
            println!("option name quietFutilitySafety type spin default 200 min 0 max 1500");
            println!("option name futilitySafety type spin default 200 min 0 max 1500");
            println!("option name futilityDepth type spin default 200 min 0 max 1500");
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
                sel_move::select_move(&pos, &mut history, &mut table, 1500, &mut counter, options);
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
                    "mobilityValue" => options.mobilityValue = value.parse().unwrap(),
                    "kingPosValue" => options.kingPosValue = value.parse().unwrap(),
                    "nmpDepthMin" => options.nmpDepthMin = value.parse().unwrap(),
                    "nmpStaticSafety" => options.nmpStaticSafety = value.parse().unwrap(),
                    "lmrMinIdx" => options.lmrMinIdx = value.parse().unwrap(),
                    "lmrMinDepth" => options.lmrMinDepth = value.parse().unwrap(),
                    "lmrMaxRedux" => options.lmrMaxRedux = value.parse().unwrap(),
                    "razoringMargin" => options.razoringMargin = value.parse().unwrap(),
                    "deltaStaticSafety" => options.deltaStaticSafety = value.parse().unwrap(),
                    "quietFutilitySafety" => options.quietFutilitySafety = value.parse().unwrap(),
                    "futilitySafety" => options.futilitySafety = value.parse().unwrap(),
                    "futilityDepth" => options.futilityDepth = value.parse().unwrap(),
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
