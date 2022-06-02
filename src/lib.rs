#![feature(associated_type_defaults)]
#![feature(async_closure)]

pub mod prelude;
pub mod app;
pub mod math;
pub mod resource;
pub mod window;
pub mod gfx;
// pub mod audio;
// pub mod interp;
// pub mod midi;
// pub mod midi2;
pub mod osc;
pub mod time;
pub mod twitch;
// pub mod wavefront;
pub mod procedural;

pub(crate) mod async_closure;

pub use cgmath;
pub use tokio;
pub use futures;
pub use image;
