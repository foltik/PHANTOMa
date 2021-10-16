#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PaletteColor {
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

impl PaletteColor {
    pub fn byte(&self) -> u8 {
        match self {
            PaletteColor::Index(b) => *b,
            PaletteColor::Red     => 72,
            PaletteColor::Orange  => 84,
            PaletteColor::Yellow  => 74,
            PaletteColor::Pea     => 17,
            PaletteColor::Lime    => 87,
            PaletteColor::Mint    => 77,
            PaletteColor::Cyan    => 78,
            PaletteColor::Blue    => 67,
            PaletteColor::Violet  => 81,
            PaletteColor::Magenta => 53,
            PaletteColor::Pink    => 95,
            PaletteColor::White   => 3,
            PaletteColor::Off     => 0,
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