use crate::score::Score;
use chess::Board;
use chess::ChessMove;

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct TTEntry {
    hash: u64,
}

impl TTEntry {
    pub fn new(b: Board) -> TTEntry {
        TTEntry { hash: b.get_hash() }
    }
}

#[derive(Copy, Clone)]
pub enum TTData_BoundType {
    LOWER,
    UPPER,
    EXACT,
}

#[derive(Copy, Clone)]
pub struct TTData {
    pub pvMove: ChessMove,
    pub eval: Score,
    pub depth: i32,
    pub bound: TTData_BoundType,
}

impl TTData {
    pub fn new(mov: ChessMove, ev: Score, d: i32, b: TTData_BoundType) -> TTData {
        TTData {
            pvMove: mov,
            eval: ev,
            depth: d,
            bound: b,
        }
    }
}
