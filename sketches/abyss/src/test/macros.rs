macro_rules! tests {
    {$($Variant:ident => ($X:literal, $Y:literal), $Color:expr,)+} =>
    {
        #[derive(Clone, Copy, Debug)]
        pub enum Test {
            $($Variant),*
        }

        impl Test {
            pub fn xy(&self) -> (i8, i8) {
                match *self {
                    $(Test::$Variant => ($X, $Y)),*
                }
            }

            pub fn color(&self) -> launchpad_x::types::Color {
                match *self {
                    $(Test::$Variant => $Color),*
                }
            }

            pub fn pos(&self) -> launchpad_x::types::Pos {
                use launchpad_x::types::{Pos, Coord};
                let (x, y) = self.xy();
                Pos::from(Coord(x, y))
            }

            pub fn all() -> impl Iterator<Item = Test> {
                vec![$(Test::$Variant),*].into_iter()
            }
        }
    }
}