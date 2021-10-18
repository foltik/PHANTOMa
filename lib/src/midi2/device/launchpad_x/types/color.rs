#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Color {
    Index(u8),
    Off,
    White,
    Red,
    Orange,
    Yellow,
    Pea,
    Lime,
    Mint,
    Cyan,
    Blue,
    Violet,
    Magenta,
    Pink,
}

impl Color {
    pub fn byte(&self) -> u8 {
        match self {
            Color::Index(b) => *b,
            Color::Red     => 72,
            Color::Orange  => 84,
            Color::Yellow  => 74,
            Color::Pea     => 17,
            Color::Lime    => 87,
            Color::Mint    => 77,
            Color::Cyan    => 78,
            Color::Blue    => 67,
            Color::Violet  => 81,
            Color::Magenta => 53,
            Color::Pink    => 95,
            Color::White   => 3,
            Color::Off     => 0,
        }
    }
}

// use rand::Rng;
// use rand::distributions::{Distribution, Standard};
// impl Distribution<PaletteColor> for Standard {
//     fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> PaletteColor {
//         match rng.gen_range(0..=10) {
//             0  => PaletteColor::Red,
//             1  => PaletteColor::Orange,
//             2  => PaletteColor::Yellow,
//             3  => PaletteColor::Pea,
//             4  => PaletteColor::Lime,
//             5  => PaletteColor::Mint,
//             6  => PaletteColor::Cyan,
//             7  => PaletteColor::Blue,
//             8  => PaletteColor::Violet,
//             9  => PaletteColor::Magenta,
//             10 => PaletteColor::Pink,
//             _  => PaletteColor::White,
//         }
//     }
// }