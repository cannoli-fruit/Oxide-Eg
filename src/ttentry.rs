use crate::score::Score;
use chess::Board;
use chess::ChessMove;

#[derive(Copy, Clone)]
pub enum TTData_BoundType {
    LOWER,
    UPPER,
    EXACT,
}

#[derive(Copy, Clone)]
pub struct TTData {
    pub hash: u64,
    pub pvMove: ChessMove,
    pub eval: Score,
    pub depth: i32,
    pub bound: TTData_BoundType,
}

impl TTData {
    pub fn new(hash: u64, mov: ChessMove, ev: Score, d: i32, b: TTData_BoundType) -> TTData {
        TTData {
            hash: hash,
            pvMove: mov,
            eval: ev,
            depth: d,
            bound: b,
        }
    }

    pub fn default() -> TTData {
        TTData {
            hash: 0,
            pvMove: ChessMove::default(),
            eval: Score::new(),
            depth: 0,
            bound: TTData_BoundType::EXACT,
        }
    }
}
