use std::fmt::Debug;
use std::result::Result;
use thiserror::Error;

use parking_lot::Mutex;
use crossbeam_queue::SegQueue;
use std::sync::Arc;
use std::thread;

use midir::{MidiInput, MidiInputConnection, MidiOutput};

pub mod device;

#[derive(Error, Debug)]
pub enum MidiError {
    #[error("failed to initialize MIDI context")]
    InitializationFailed(#[from] midir::InitError),
    #[error("failed to connect to device '{0}'")]
    ConnectionFailed(String),
    #[error("device '{0}' not found")]
    DeviceNotFound(String),
}

#[derive(Clone)]
pub struct Midi<D: Device> {
    state: Arc<Mutex<D>>,

    input: Arc<SegQueue<D::Input>>,
    output: Arc<SegQueue<D::Output>>,
    // in_tx: broadcast::Sender<D::Input>,
    // out_tx: mpsc::Sender<D::Output>,

    _conn: Arc<Mutex<MidiInputConnection<()>>>,
}

impl<D: Device + Clone + Send> Midi<D> {
    pub fn open(name: &str) -> Result<Self, MidiError> {
        let state = Arc::new(Mutex::new(D::new()));

        let input = Arc::new(SegQueue::new());
        let output = Arc::new(SegQueue::new());

        let midi_in =
            MidiInput::new(&format!("StageBridge_in_{}", name)).map_err(MidiError::from)?;
        let midi_out = MidiOutput::new(&format!("StageBridge_out_{}", name))
            .map_err(MidiError::from)?;
            
        // midi_in
        //     .ports()
        //     .into_iter()
        //     .for_each(|p| log::info!("input: '{}'", midi_in.port_name(&p).unwrap()));
        // midi_out
        //     .ports()
        //     .into_iter()
        //     .for_each(|p| log::info!("output: '{}'", midi_out.port_name(&p).unwrap()));

        let in_port = midi_in
            .ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).unwrap().contains(name))
            .ok_or_else(|| MidiError::DeviceNotFound(name.to_owned()))?;
        let in_name = midi_in
            .port_name(&in_port)
            .expect("failed to query MIDI port name");

        let out_port = midi_out
            .ports()
            .into_iter()
            .find(|p| midi_out.port_name(p).unwrap().contains(name))
            .ok_or_else(|| MidiError::DeviceNotFound(name.to_owned()))?;
        let out_name = midi_out
            .port_name(&out_port)
            .expect("failed to query MIDI port name");

        let mut out_conn = midi_out
            .connect(&out_port, "out")
            .map_err(|_| MidiError::ConnectionFailed(out_name.clone()))?;

        // Output sender loop
        let _output = Arc::clone(&output);
        let _state = Arc::clone(&state);
        thread::spawn(move || {
            loop {
                if let Some(output) = _output.pop() {
                    // let output = out_rx
                    //     .recv()
                    //     .await
                    //     .expect(&format!("{}: output receiver dropped", &_out_name));

                    let data = _state.lock().process_output(output);
                    // log::trace!("{} -> {:X?}", &_out_name, &data);

                    if out_conn.send(&data).is_err() { 
                        log::error!("{}: failed to send output", out_name) 
                    }
                }
            }
        });

        // Input receiver loop
        let _input = Arc::clone(&input);
        let _state = Arc::clone(&state);
        let input_conn = midi_in
            .connect(
                &in_port,
                "in",
                move |_, data, _| {
                    if let Some(i) = _state.lock().process_input(data) {
                        // log::trace!("{} <- [{:X?}]", &_in_name, data);
                        _input.push(i);
                    }
                },
                (),
            )
            .map_err(move |_| MidiError::ConnectionFailed(in_name))?;

        Ok(Self {
            state,

            input,
            output,

            _conn: Arc::new(Mutex::new(input_conn)),
        })
    }

    pub fn state(&self) -> Arc<Mutex<D>> {
        Arc::clone(&self.state)
    }

    pub fn send(&self, output: D::Output) {
        self.output.push(output);
    }

    pub fn recv(&self) -> Vec<D::Input> {
        let mut input = Vec::with_capacity(self.input.len());
        while let Some(i) = self.input.pop() {
            input.push(i);
        }
        input
    }
}

pub trait Device: Send + Debug + 'static {
    type Input: Send + Clone + Debug = Vec<u8>;
    type Output: Send + Clone + Debug = Vec<u8>;

    fn new() -> Self;

    fn process_input(&mut self, data: &[u8]) -> Option<<Self as Device>::Input>;
    fn process_output(&mut self, output: <Self as Device>::Output) -> Vec<u8>;
}