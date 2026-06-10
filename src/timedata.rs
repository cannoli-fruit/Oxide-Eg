use std::time::Instant;

#[derive(Copy, Clone)]
pub struct Timer {
    pub start: Instant,
    pub len: u64,
    pub stop: bool,
}

impl Timer {
    pub fn is_done(self) -> bool {
        self.stop
    }

    pub fn recalc(&mut self) {
        self.stop = (self.start.elapsed().as_millis() >= self.len.into());
    }
}
