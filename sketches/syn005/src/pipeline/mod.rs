mod phong;
pub use phong::Phong;

mod animated;
pub use animated::{Animated, Material as AnimatedMaterial, MaterialDesc as AnimatedMaterialDesc};

mod fx;
pub use fx::{Fx, FxState};