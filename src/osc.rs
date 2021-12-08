use crossbeam_queue::SegQueue;
use std::sync::Arc;
use std::thread;

use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

use rosc::OscPacket;
use rosc::OscType;
pub use rosc::OscMessage as RoscMessage;

#[derive(Debug, Clone)]
pub enum MixxxMessage {
    // Position { deck: usize, pos: f32 },
    Song { deck: usize, artist: String, title: String },
    // Volume { deck: usize, vol: f32 },
    Bpm { deck: usize, bpm: f32 },
    // Playing { deck: usize, playing: bool },
    // Beat { deck: usize },
}

#[derive(Debug, Clone)]
pub enum OscMessage {
    Mixxx(MixxxMessage),
    Other(String, Vec<OscType>),
}

pub struct Osc {
    pub queue: Arc<SegQueue<OscMessage>>,
}

impl Osc {
    pub fn new(addr: &'static str) -> Self {
        let queue = Arc::new(SegQueue::new());

        let tx = Arc::clone(&queue);
        thread::spawn(move || {
            let sock = UdpSocket::bind(SocketAddr::from_str(addr).unwrap()).unwrap();

            let mut buf = [0u8; rosc::decoder::MTU];

            loop {
                let (size, _addr) = sock.recv_from(&mut buf).unwrap();
                let mut packets = vec![rosc::decoder::decode(&buf[..size]).unwrap()];
                while !packets.is_empty() {
                    let packet = packets.pop().unwrap();
                    match packet {
                        OscPacket::Message(msg) => {
                            tx.push(parse(msg));
                        },
                        OscPacket::Bundle(bundle) => {
                            for p in bundle.content {
                                packets.push(p);
                            }
                        }
                    };
                }
            }
        });

        Self { queue }
    }

    pub fn poll(&self) -> Vec<OscMessage> {
        let mut messages = Vec::with_capacity(self.queue.len());

        while !self.queue.is_empty() {
            messages.push(self.queue.pop().unwrap());
        }

        messages
    }
}

fn parse(msg: RoscMessage) -> OscMessage {
    let RoscMessage { addr, args } = msg;

    match &addr[..] {
        "/mixxx/deck/song" => OscMessage::Mixxx(MixxxMessage::Song {
            deck: match args[0] {
                OscType::Int(i) => i as usize,
                _ => panic!()
            },
            artist: match &args[1] {
                OscType::String(s) => s.clone(),
                _ => panic!()
            },
            title: match &args[2] {
                OscType::String(s) => s.clone(),
                _ => panic!()
            }
        }),
        "/mixxx/deck/bpm" => OscMessage::Mixxx(MixxxMessage::Bpm {
            deck: match args[0] {
                OscType::Int(i) => i as usize,
                _ => panic!()
            },
            bpm: match args[1] {
                OscType::Float(f) => f,
                _ => panic!()
            },
        }),
        // "/mixxx/deck/pos" => OscMessage::Mixxx(MixxxMessage::Position {
        //     deck: match args[0] {
        //         OscType::Int(i) => i as usize,
        //         _ => panic!()
        //     },
        //     pos: match args[1] {
        //         OscType::Float(f) => f,
        //         _ => panic!()
        //     },
        // }),
        // "/mixxx/deck/volume" => OscMessage::Mixxx(MixxxMessage::Volume {
        //     deck: match args[0] {
        //         OscType::Int(i) => i as usize,
        //         _ => panic!()
        //     },
        //     vol: match args[1] {
        //         OscType::Float(f) => f,
        //         _ => panic!()
        //     },
        // }),
        // "/mixxx/deck/playing" => OscMessage::Mixxx(MixxxMessage::Playing {
        //     deck: match args[0] {
        //         OscType::Int(i) => i as usize,
        //         _ => panic!()
        //     },
        //     playing: match args[1] {
        //         OscType::Int(i) => match i {
        //             0 => false,
        //             1 => true,
        //             _ => panic!(),
        //         },
        //         _ => panic!()
        //     },
        // }),
        // "/mixxx/deck/beat" => match args[1] {
        //     OscType::Int(i) => match i {
        //         1 => OscMessage::Mixxx(MixxxMessage::Beat {
        //             deck: match args[0] {
        //                 OscType::Int(i) => i as usize,
        //                 _ => panic!()
        //             },
        //         }),
        //         _ => OscMessage::Other(addr, args),
        //     },
        //     _ => panic!()
        // },
        _ => OscMessage::Other(addr, args)
    }
}