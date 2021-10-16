use super::pos::*;

use std::cmp::max;

pub struct Diagonal {
    x0: i8,
    y0: i8,

    i: i8,
    n: i8,
}

impl Diagonal {
    pub fn new(pos: Pos) -> Self {
        let c: Coord = pos.into();

        let x0 = max(c.1 - (7 - c.0), 0);
        let y0 = 7 - max((7 - c.0) - c.1, 0);

        Self {
            x0,
            y0,
            i: 0,
            n: 8 - ((7 - x0) - y0).abs()
        }
    }
}

impl Iterator for Diagonal {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.n {
            let c = Coord(self.x0 + self.i, self.y0 - self.i);
            self.i += 1;
            Some(c.into())
        } else {
            None
        }
    }
}


pub struct Antidiagonal {
    x0: i8,
    y0: i8,

    i: i8,
    n: i8,
}

impl Antidiagonal {
    pub fn new(pos: Pos) -> Self {
        let c: Coord = pos.into();

        let x0 = max(c.0 - c.1, 0);
        let y0 = max(c.1 - c.0, 0);

        Self {
            x0,
            y0,
            i: 0,
            n: 8 - (x0 - y0).abs()
        }
    }
}

impl Iterator for Antidiagonal {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.n {
            let c = Coord(self.x0 + self.i, self.y0 + self.i);
            self.i += 1;
            Some(c.into())
        } else {
            None
        }
    }
}