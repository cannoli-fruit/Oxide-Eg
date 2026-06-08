#[derive(Copy, Clone)]
pub struct Settings {
    // Keep pawn value out so it stays at 100
    pub knightValue: i64,
    pub bishopValue: i64,
    pub rookValue: i64,
    pub queenValue: i64,
    pub nmpDepthMin: i32,
    pub nmpStaticSafety: i64,
    pub nmpMinPieces: u32,
    pub lmrMinIdx: i32,
    pub lmrMinDepth: i32,
    pub lmrMaxRedux: i32,
    pub kingAttackValue: i64,
    pub razoringMargin: i64,
    pub deltaStaticSafety: i64,
    pub quietFutilitySafety: i64,
    pub futilitySafety: i64,
    pub futilityDepth: i32,
    pub revFutilityDepth: i32,
    pub revFutilityFactor: i64,
    pub aspirationWindowSize: i64,
}
