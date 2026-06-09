#[derive(Copy, Clone)]
pub struct Settings {
    // Keep pawn value out so it stays at 100
    pub knightValue: i64,
    pub bishopValue: i64,
    pub rookValue: i64,
    pub queenValue: i64,
    pub lmrMinDepth: i32,
    pub sortcnt: usize,
    pub aspirationWindowSize: i64,
}
