use std::fmt::Debug;
use std::result::Result;
use thiserror::Error;
use std::iter::{self, Iterator};

use tokio::sync::mpsc;
use tokio::task;
use parking_lot::Mutex;
use crossbeam_queue::SegQueue;
use std::sync::Arc;

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

    out_tx: mpsc::Sender<D::Output>,
    in_buf: Arc<SegQueue<D::Input>>,

    _conn: Arc<Mutex<MidiInputConnection<()>>>,
}

impl<D: Device + Clone + Send> Midi<D> {
    pub fn open(input: &str, output: &str) -> Result<Self, MidiError> {
        let state = Arc::new(Mutex::new(D::new()));

        let midi_in =
            MidiInput::new(&format!("StageBridge_in_{}", input)).map_err(MidiError::from)?;
        let midi_out = MidiOutput::new(&format!("StageBridge_out_{}", output))
            .map_err(MidiError::from)?;
            
        midi_in
            .ports()
            .into_iter()
            .for_each(|p| log::debug!("input: '{}'", midi_in.port_name(&p).unwrap()));
        midi_out
            .ports()
            .into_iter()
            .for_each(|p| log::debug!("output: '{}'", midi_out.port_name(&p).unwrap()));

        let in_port = midi_in
            .ports()
            .into_iter()
            .find(|p| midi_in.port_name(p).unwrap().contains(input))
            .ok_or_else(|| MidiError::DeviceNotFound(input.to_owned()))?;
        let in_name = midi_in
            .port_name(&in_port)
            .expect("failed to query MIDI port name");

        let out_port = midi_out
            .ports()
            .into_iter()
            .find(|p| midi_out.port_name(p).unwrap().contains(output))
            .ok_or_else(|| MidiError::DeviceNotFound(output.to_owned()))?;
        let out_name = midi_out
            .port_name(&out_port)
            .expect("failed to query MIDI port name");

        let mut out_conn = midi_out
            .connect(&out_port, "out")
            .map_err(|_| MidiError::ConnectionFailed(out_name.clone()))?;

        let in_buf = Arc::new(SegQueue::new());
        let (out_tx, mut out_rx) = mpsc::channel(16);

        // Output sender loop
        let _state = Arc::clone(&state);
        let _name = output.to_owned();
        task::spawn(async move {
            loop {
                let output = out_rx.recv().await.unwrap();

                let data = _state.lock().process_output(output);
                log::trace!("{} -> {:X?}", &_name, &data);

                if out_conn.send(&data).is_err() { 
                    log::error!("{}: failed to send output", out_name) 
                }
            }
        });

        // Input receiver loop
        let _in_buf = Arc::clone(&in_buf);
        let _state = Arc::clone(&state);
        let _name = input.to_owned();
        let input_conn = midi_in
            .connect(
                &in_port,
                "in",
                move |_, data, _| {
                    log::trace!("{} <- [{:X?}]", &_name, data);
                    if let Some(i) = _state.lock().process_input(data) {
                        _in_buf.push(i);
                    }
                },
                (),
            )
            .map_err(move |_| MidiError::ConnectionFailed(in_name))?;

        Ok(Self {
            state,

            in_buf,
            out_tx,

            _conn: Arc::new(Mutex::new(input_conn)),
        })
    }

    pub fn maybe_open(input: &str, output: &str) -> Option<Self> {
        match Self::open(input, output) {
            Ok(device) => Some(device),
            Err(e) => {
                log::warn!("Failed to open MIDI device '{}'/'{}' : {:?}", input, output, e);
                None
            }
        }
    }

    pub fn state(&self) -> Arc<Mutex<D>> {
        Arc::clone(&self.state)
    }

    pub async fn send(&self, output: D::Output) {
        self.out_tx.send(output).await.unwrap();
    }

    pub fn recv(&self) -> Vec<D::Input> {
        iter::from_fn(|| self.in_buf.pop()).collect()
    }
}

pub trait Device: Send + Debug + 'static {
    type Input: Send + Clone + Debug = Vec<u8>;
    type Output: Send + Clone + Debug = Vec<u8>;

    fn new() -> Self;

    fn process_input(&mut self, data: &[u8]) -> Option<<Self as Device>::Input>;
    fn process_output(&mut self, output: <Self as Device>::Output) -> Vec<u8>;
}