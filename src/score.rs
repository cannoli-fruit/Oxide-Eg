#![allow(non_snake_case)]

use std::cmp::Ordering;
use std::fmt::*;

#[derive(Clone, Copy)]
pub struct Score {
    pub val: i64,
}

impl Score {
    pub fn is_greater(&self, other: Score) -> bool {
        self.val > other.val
    }

    pub fn inverse(self) -> Score {
        let mut clone = self.clone();
        clone.val *= -1;

        return clone;
    }

    pub fn isMate(self) -> bool {
        return self.val > 2147483648 || self.val < -2147483648;
    }

    pub fn step(self) -> Score {
        let mut clone = self.clone();

        if !clone.isMate() {
            return clone;
        }

        if clone.val > 0 {
            clone.val -= 1;
        } else if clone.val < 0 {
            clone.val += 1;
        }

        return clone;
    }

    pub fn infp(self) -> bool {
        return self.val > 4294967296;
    }

    pub fn infn(self) -> bool {
        return self.val < -4294967296;
    }

    pub fn mateDist(self) -> i32 {
        if !self.isMate() {
            return 0;
        }

        if self.val > 0 {
            return (4294967296 - self.val).try_into().unwrap();
        } else {
            return (4294967296 + self.val).try_into().unwrap();
        }
    }

    pub fn print(self) {
        if self.infp() {
            println!("+Inf");
            return;
        }
        if self.infn() {
            println!("-Inf");
            return;
        }
        if self.isMate() {
            println!("Mate in {}", self.mateDist());
            return;
        }
        println!("{} Cp", self.val);
    }

    pub fn new() -> Score {
        Score { val: 0 }
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.val.cmp(&other.val)
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.val == other.val
    }
}

impl Eq for Score {}

impl Display for Score {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.infp() {
            return write!(f, "+Inf");
        }
        if self.infn() {
            return write!(f, "-Inf");
        }
        if self.isMate() {
            return write!(f, "M: {}", self.mateDist());
        }
        return write!(f, "{} Cp", self.val);
    }
}
