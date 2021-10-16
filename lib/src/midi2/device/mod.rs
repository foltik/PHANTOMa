pub use super::Device;

#[derive(Copy, Clone, Debug)]
pub struct Raw;

impl Device for Raw {
    fn new() -> Self {
        Self
    }

    fn process_input(&mut self, raw: &[u8]) -> Option<Vec<u8>> {
        Some(raw.to_owned())
    }

    fn process_output(&mut self, output: Vec<u8>) -> Vec<u8> {
        output
    }
}

pub mod worlde_easycontrol9;
pub mod launchpad_x;