use super::{Color, Delay, Pos, Coord};

type Op = Box<dyn FnMut(&Frame, &mut Frame) + Send + 'static>;

pub struct Peepline {
    ops: Vec<Op>,

    front: Frame,
    back: Frame
}

impl Peepline {
    pub fn new() -> Self {
        Self {
            ops: Vec::new(),
            front: Frame::new(),
            back: Frame::new(),
        }
    }

    pub fn run(&mut self, pos: Pos, color: Color) -> impl Iterator<Item = Event> + '_ {
        let mut front = &mut self.front;
        let mut back = &mut self.back;

        front.clear();
        back.clear();

        front.push(Event {
            pos,
            color,
            delay: Delay::zero(),
        });

        for op in &mut self.ops {
            op(front, back);
            std::mem::swap(&mut front, &mut back);
            back.clear();
        }

        front.iter()
    }

    pub fn with<F>(mut self, op: F) -> Self 
    where
        F: FnMut(&Frame, &mut Frame) + Send + 'static
    {
        self.ops.push(Box::new(op));
        self
    }

    pub fn filter<P>(self, mut pred: P) -> Self
    where
        P: FnMut(Event) -> bool + Send + 'static,
    {
        self.with(move |prev, next| {
            prev.iter()
                .filter(|e| pred(*e))
                .for_each(|e| next.push(e));
        })
    }
    pub fn filter_pos<P>(self, mut pred: P) -> Self
    where
        P: FnMut(Pos) -> bool + Send + 'static
    {
        self.filter(move |Event { pos, .. }| pred(pos))
    }

    pub fn map<M>(self, mut mapper: M) -> Self
    where
        M: FnMut(Event) -> Event + Send + 'static,
    {
        self.with(move |prev, next| {
            prev.iter()
                .map(|e| mapper(e))
                .for_each(|e| next.push(e))
        })
    }
    pub fn map_pos<M>(self, mut mapper: M) -> Self
    where
        M: FnMut(Pos) -> Pos + Send + 'static,
    {
        self.map(move |Event { pos, color, delay }| 
            Event { pos: mapper(pos), color, delay })
    }
    pub fn map_col<M>(self, mut mapper: M) -> Self
    where
        M: FnMut(Color) -> Color + Send + 'static,
    {
        self.map(move |Event { pos, color, delay }| 
            Event { pos, color: mapper(color), delay })
    }
    pub fn map_del<M>(self, mut mapper: M) -> Self
    where
        M: FnMut(Delay) -> Delay + Send + 'static,
    {
        self.map(move |Event { pos, color, delay }| 
            Event { pos, color, delay: mapper(delay) })
    }

    pub fn flat_map<M, I>(self, mut mapper: M) -> Self
    where
        M: FnMut(Event) -> I + Send + 'static,
        I: IntoIterator<Item = Event>
    {
        self.with(move |prev, next| {
            prev.iter()
                .flat_map(|e| mapper(e))
                .for_each(|e| next.push(e))
        })
    }
    pub fn flat_map_pos<M, I>(self, mut mapper: M) -> Self
    where
        M: FnMut(Pos) -> I + Send + 'static,
        I: IntoIterator<Item = Pos>
    {
        self.flat_map(move |Event { pos, color, delay }|
            mapper(pos).into_iter().map(move |pos| Event { pos, color, delay }))
    }
    pub fn flat_map_color<M, I>(self, mut mapper: M) -> Self
    where
        M: FnMut(Color) -> I + Send + 'static,
        I: IntoIterator<Item = Color>
    {
        self.flat_map(move |Event { pos, color, delay }|
            mapper(color).into_iter().map(move |color| Event { pos, color, delay }))
    }
    pub fn flat_map_delay<M, I>(self, mut mapper: M) -> Self
    where
        M: FnMut(Delay) -> I + Send + 'static,
        I: IntoIterator<Item = Delay>
    {
        self.flat_map(move |Event { pos, color, delay }|
            mapper(delay).into_iter().map(move |delay| Event { pos, color, delay }))
    }

    pub fn shift<M>(self, mapper: M) -> Self
    where
        M: FnMut(Pos) -> Pos + Send + 'static,
    {
        self.map_pos(mapper)
    }
    pub fn dupe<M, I>(self, mut mapper: M) -> Self
    where
        M: FnMut(Pos) -> I + Send + 'static,
        I: IntoIterator<Item = Pos>
    {
        self.flat_map_pos(move |p| [p].into_iter().chain(mapper(p)))
    }

    pub fn filter_8(self) -> Self {
        self.filter_pos(|p| {
            let Coord(x, y) = p.into();
            x < 8 && y < 8
        })
    }
}
