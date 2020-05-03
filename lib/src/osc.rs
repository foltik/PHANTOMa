use crossbeam_queue::SegQueue;
use nannou_osc as osc;
use std::sync::Arc;
use std::thread;

#[derive(Debug)]
pub enum OscMessage {
    Bpm(f32),
    Unknown(String, Option<Vec<osc::Type>>),
}

#[derive(Debug)]
enum MixxxMessage {
    BpmHigh(f32),
    BpmLow(f32),
    BpmDecimal(f32),
    Beat,
}

type OscQueue = Arc<SegQueue<OscMessage>>;

pub struct Osc {
    queue: OscQueue,
}

impl Osc {
    pub fn init(port: u16) -> Self {
        let queue = Arc::new(SegQueue::new());
        let receiver = osc::receiver(port).unwrap();

        let rqueue = Arc::clone(&queue);
        thread::spawn(move || {
            receive(&receiver, rqueue);
        });

        Self {
            queue,
        }
    }

    pub fn poll(&self) -> Vec<OscMessage> {
        let mut messages = Vec::with_capacity(self.queue.len());

        while !self.queue.is_empty() {
            messages.push(self.queue.pop().unwrap());
        }

        messages
    }
}

fn parse_mixxx(message: osc::Message) -> Option<MixxxMessage> {
    match message.addr.as_str() {
        "/midi/noteon50" => Some(MixxxMessage::Beat),
        "/midi/cc17" => {
            let lo = message.args.unwrap().remove(0).float().unwrap();
            Some(MixxxMessage::BpmLow(lo))
        },
        "/midi/cc18" => {
            let dec = message.args.unwrap().remove(0).float().unwrap();
            Some(MixxxMessage::BpmDecimal(dec))
        },
        "/midi/cc19" => {
            let hi = message.args.unwrap().remove(0).float().unwrap();
            Some(MixxxMessage::BpmHigh(hi))
        },
        _ => None,
    }
}

fn receive(receiver: &osc::Receiver, queue: OscQueue) {
    loop {
        let mut bpm = (None, None, None);

        for (packet, _addr) in receiver.try_iter() {
            for msg in packet.into_msgs() {
                if let Some(mixxx) = parse_mixxx(msg.clone()) {
                    match mixxx {
                        MixxxMessage::BpmHigh(hi) => {
                            bpm = (Some(hi), bpm.1, bpm.2);
                        },
                        MixxxMessage::BpmLow(lo) => {
                            bpm = (bpm.0, Some(lo), bpm.2);
                        },
                        MixxxMessage::BpmDecimal(d) => {
                            bpm = (bpm.0, bpm.1, Some(d));
                        },
                        _ => {}
                    }
                } else {
                    queue.push(OscMessage::Unknown(msg.addr, msg.args))
                }

                if let (Some(hi), Some(lo), Some(dec)) = bpm {
                    queue.push(OscMessage::Bpm((127.0 * hi) + lo + (0.01 * dec)));
                    bpm = (None, None, None);
                }
            }
        }

        thread::sleep(std::time::Duration::from_millis(1));
    }
}
