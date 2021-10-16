use super::pos::*;

pub struct Vertical {
    x: i8,
    i: i8
}

impl Vertical {
    pub fn new(pos: Pos) -> Self {
        Self { 
            x: Coord::from(pos).0,
            i: 0
        }
    }
}

impl Iterator for Vertical {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < 8 {
            let c = Coord(self.x, self.i);
            self.i += 1;
            Some(c.into())
        } else {
            None
        }
    }
}


pub struct Horizontal {
    y: i8,
    i: i8
}

impl Horizontal {
    pub fn new(pos: Pos) -> Self {
        Self { 
            y: Coord::from(pos).1,
            i: 0
        }
    }
}

impl Iterator for Horizontal {
    type Item = Pos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < 8 {
            let c = Coord(self.i, self.y);
            self.i += 1;
            Some(c.into())
        } else {
            None
        }
    }
}