use std::fmt::Debug;
use std::cmp::min;

pub trait IPos: Copy + Clone + Debug {
    fn byte(&self) -> u8;
    fn from_byte(b: u8) -> Self;
}

#[derive(Copy, Clone, Debug)]
pub enum Pos {
    Index(Index),
    Index9(Index9),
    Coord(Coord),
    Column(Column),
}

impl Pos {
    pub fn shift(&self, x: i8, y: i8) -> Self {
        let Coord(x0, y0) = (*self).into();
        Self::Coord(Coord(x0 + x, y0 + y))
    }
    pub fn shift_x(&self, x: i8) -> Self {
        self.shift(x, 0)
    }
    pub fn shift_y(&self, y: i8) -> Self {
        self.shift(0, y)
    }

    pub fn iter_shift_x(&self, n: u8) -> impl Iterator<Item = Pos> {
        let Coord(x, y) = (*self).into();

        let n = n as i8;
        let sign = if n > 0 { 1 } else { -1 };

        let mut i = 0i8;
        std::iter::from_fn(move || {
            let _i = i;
            if i < n {
                i += 1;
                Some(Coord(x + _i*sign, y).into())
            } else {
                None
            }
        })
    }

    pub fn iter_shift_y(&self, n: u8) -> impl Iterator<Item = Pos> {
        let Coord(x, y) = (*self).into();

        let n = n as i8;
        let sign = if n > 0 { 1 } else { -1 };

        let mut i = 0i8;
        std::iter::from_fn(move || {
            let _i = i;
            if i < n {
                i += 1;
                Some(Coord(x, y + _i*sign).into())
            } else {
                None
            }
        })
    }
}

impl Eq for Pos {}
impl PartialEq for Pos {
    fn eq(&self, other: &Self) -> bool {
        self.byte() == other.byte()
    }
}

impl IPos for Pos {
    fn byte(&self) -> u8 {
        match *self {
            Pos::Index(i) => i.byte(),
            Pos::Index9(i) => i.byte(),
            Pos::Coord(c) => c.byte(),
            Pos::Column(c) => c.byte(),
        }
    }

    fn from_byte(b: u8) -> Self {
        Pos::Coord(Coord::from_byte(b))
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Index(pub i8);
impl IPos for Index {
    fn byte(&self) -> u8 {
        let i = min(self.0, 63);
        let x = i % 8;
        let y = i / 8;
        ((y + 1) * 10 + (x + 1)) as u8
    }

    fn from_byte(b: u8) -> Self {
        let x = b % 10 - 1;
        let y = b / 10 - 1;
        Self((y * 8 + x) as i8)
    }
}
impl From<Index> for Pos {
    fn from(i: Index) -> Self {
        Pos::Index(i)
    }
}
impl From<Pos> for Index {
    fn from(p: Pos) -> Self {
        Self::from_byte(p.byte())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Index9(pub i8);
impl IPos for Index9 {
    fn byte(&self) -> u8 {
        let i = min(self.0, 80);
        let x = i % 9;
        let y = i / 9;
        ((y + 1) * 10 + (x + 1)) as u8
    }

    fn from_byte(b: u8) -> Self {
        let x = b % 10 - 1;
        let y = b / 10 - 1;
        Self((y * 9 + x) as i8)
    }
}
impl From<Index9> for Pos {
    fn from(i: Index9) -> Self {
        Pos::Index9(i)
    }
}
impl From<Pos> for Index9 {
    fn from(p: Pos) -> Self {
        Self::from_byte(p.byte())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Coord(pub i8, pub i8);
impl IPos for Coord {
    fn byte(&self) -> u8 {
        let x = min(self.0, 8);
        let y = min(self.1, 8);
        ((y + 1) * 10 + (x + 1)) as u8
    }

    fn from_byte(b: u8) -> Self {
        let x = b % 10 - 1;
        let y = b / 10 - 1;
        Self(x as i8, y as i8)
    }
}
impl From<Coord> for Pos {
    fn from(c: Coord) -> Self {
        Pos::Coord(c)
    }
}
impl From<Pos> for Coord {
    fn from(p: Pos) -> Self {
        Self::from_byte(p.byte())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Column(pub i8);
impl IPos for Column {
    fn byte(&self) -> u8 {
        let i = min(self.0, 63);
        let x = i % 4;
        let y = i / 4;
        let offset = (i / 32) * 4;
        ((y + 1) * 10 + (x + 1) + offset) as u8
    }

    fn from_byte(b: u8) -> Self {
        let x = b % 10 - 1;
        let y = b / 10 - 1;
        let offset = (x / 4) * 4;
        Self((y * 4 + x + offset) as i8)
    }
}
impl From<Column> for Pos {
    fn from(c: Column) -> Self {
        Pos::Column(c)
    }
}
impl From<Pos> for Column {
    fn from(p: Pos) -> Self {
        Self::from_byte(p.byte())
    }
}
